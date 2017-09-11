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

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize, command_vector: Vec<OsString>) -> ThreadPool {
        assert!(size > 0);

        // channel for exchanging job messages inside ThreadPool
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let command_vector = Arc::new(command_vector);

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(
                id,
                receiver.clone(),
                Arc::clone(&command_vector),
            ));
        }

        ThreadPool { workers, sender }
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
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        command_vector: Arc<Vec<OsString>>,
    ) -> Worker {

        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            let program = &command_vector[0];
            let program_arguments = &command_vector[1..];

            match message {
                Message::Payload(payload) => {
                    println!("[worker-{}] Got payload: {}.", id, payload);

                    // spawn child command
                    // TODO: if a thread panics, does the threadpool replaces them?
                    let mut child =
                        Command::new(&program)
                            .args(program_arguments)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .unwrap_or_else(|e| panic!("failed to execute process: {}\n", e));

                    // pass payload data through child process stdin
                    child
                        .stdin
                        .take()
                        .unwrap()
                        .write_all(payload.as_bytes())
                        .expect("couldn't write to child process stdin");

                    // wait for child process to finish and propagate exit status code
                    let exit_status = child.wait().unwrap();
                    match exit_status.success() {
                        true => {
                            println!(
                                "[worker-{}] Command succeded with status code {}.",
                                id,
                                exit_status.code().unwrap()
                            );
                        }
                        false => {
                            // TODO: ExitStatus.code() will return None if process was terminated by a signal.
                            eprintln!(
                                "[worker-{}] Command {} failed with status code {}.",
                                id,
                                program.to_str().unwrap(),
                                exit_status.code().unwrap()
                            );
                        }
                    }
                    // propagate standard streams
                    for line in BufReader::new(child.stderr.take().unwrap()).lines() {
                        eprintln!("[{}-{}]! {}", program.to_str().unwrap(), id, line.unwrap())
                    }
                    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
                        println!("[{}-{}] {}", program.to_str().unwrap(), id, line.unwrap());
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
