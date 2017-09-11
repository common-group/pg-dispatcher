use std::process::Command;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::ffi::OsString;

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

                    // build & spawn command
                    // TODO: if a thread panics, does the threadpool replaces them?
                    let output =
                        Command::new(&program)
                            .args(program_arguments)
                            .env("PG_DISPATCH_PAYLOAD", payload)
                            .output()
                            .unwrap_or_else(|e| panic!("failed to execute process: {}\n", e));

                    // collect stdout, stderr, status_code
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let exit_status = output.status;

                    // propagate standard streams and exit status code
                    for line in stderr.lines() {
                        eprintln!("[{}-{}]! {}", program.to_str().unwrap(), id, line);
                    }

                    for line in stdout.lines() {
                        println!("[{}-{}] {}", program.to_str().unwrap(), id, line);
                    }
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
