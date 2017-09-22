extern crate base64;

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};

/// For exchanging in the job channel
enum Message {
    Payload(String),
    Terminate,
}

pub enum WorkerMessage {
    ProgramNotFound(String),
    StdinFailed(String),
    DoneTask(String),
}

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    pub workers_channel: mpsc::Receiver<WorkerMessage>,
    pub idle_counter: Arc<Mutex<usize>>
}

impl ThreadPool {
    pub fn new(size: usize, command_vector: Vec<OsString>) -> ThreadPool {
        assert!(size > 0);

        // channel for exchanging job messages inside ThreadPool
        let (sender, receiver) = mpsc::channel();
        let (workers_sender, workers_channel) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let idle_counter = Arc::new(Mutex::new(size));

        let command_vector = Arc::new(command_vector);

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(
                    id,
                    idle_counter.clone(),
                    workers_sender.clone(),
                    receiver.clone(),
                    Arc::clone(&command_vector),
                    ));
        }

        ThreadPool { workers, sender, workers_channel, idle_counter}
    }

    pub fn execute(&self, payload: String) {
        self.sender.send(Message::Payload(payload)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        idle_counter: Arc<Mutex<usize>>,
        workers_sender: mpsc::Sender<WorkerMessage>,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        command_vector: Arc<Vec<OsString>>,
        ) -> Worker {

        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            let program = &command_vector[0];
            let program_arguments = &command_vector[1..];

            match message {
                Message::Payload(payload) => {
                    {
                        let guard_idle_counter = idle_counter.clone();
                        *guard_idle_counter.lock().unwrap() -= 1;
                    }
                    let payload_base64 = base64::encode(&payload);
                    println!("[worker-{}] Got payload: {}.", id, payload);

                    // spawn child command
                    // TODO: if a thread panics, does the threadpool replaces them?
                    let child_command =
                        Command::new(&program)
                        .args(program_arguments)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn();

                    if let Ok(mut child) = child_command {
                        // pass payload data through child process stdin
                        let write_to_child = child
                            .stdin
                            .take()
                            .unwrap()
                            .write_all(payload.as_bytes());
                        if let Ok(_) = write_to_child {
                            let exit_status = child.wait().unwrap();
                            match exit_status.success() {
                                true => {
                                    println!(
                                        "[worker-{}] Command succeded with status code {}.",
                                        id, exit_status.code().unwrap());
                                }
                                false => {
                                    // TODO: ExitStatus.code() will return None if process was terminated by a signal.
                                    eprintln!(
                                        "[worker-{}] Command {} failed with status code {}.",
                                        id, program.to_str().unwrap(), exit_status.code().unwrap());
                                }
                            }
                            // propagate standard streams
                            for line in BufReader::new(child.stderr.take().unwrap()).lines() {
                                eprintln!("[{}-{}]! {}", program.to_str().unwrap(), id, line.unwrap())
                            }
                            for line in BufReader::new(child.stdout.take().unwrap()).lines() {
                                println!("[{}-{}] {}", program.to_str().unwrap(), id, line.unwrap());
                            }
                            workers_sender
                                .send(WorkerMessage::DoneTask(payload_base64))
                                .unwrap();
                        } else {
                            workers_sender
                                .send(WorkerMessage::StdinFailed(payload_base64))
                                .unwrap();
                            eprintln!("couldn't write to child process stdin");
                        }
                    } else {
                        workers_sender
                            .send(WorkerMessage::ProgramNotFound(payload_base64))
                            .unwrap();
                        eprintln!("couldn't execute program {:?}", program);
                    }

                {
                    let guard_idle_counter = idle_counter.clone();
                    *guard_idle_counter.lock().unwrap() += 1;
                }
            }
            Message::Terminate => {
                println!("[worker-{}] Terminating.", id);
                break;
            }
        }
    });
    Worker {
        id,
        thread: Some(thread),
    }
}
}
