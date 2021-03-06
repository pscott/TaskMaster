use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new `ThreadPool`.
    ///
    /// The size is the number of threads in the pool.
    pub fn new(size: usize) -> Result<Self, &'static str> {
        if size < 1 {
            return Err("Number of threads too small");
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(Self { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Err(e) = self.sender.send(Message::NewJob(job)) {
            eprintln!("Failed to send message: {:?}", e);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Iterate through the Workers once to make sure that they all receive a Terminate message.
        for _ in &self.workers {
            if let Err(e) = self.sender.send(Message::Terminate) {
                eprintln!("Failed to send Terminate: {:?}", e);
            }
        }

        // Iterate a second time through the workers to gracefully exit.
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    eprintln!("Worker #{}: {:?}", worker.id, e);
                }
            }
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let lock = match receiver.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    eprintln!("Failed to acquire lock: {:?}", e);
                    continue;
                }
            };

            let message = match lock.recv() {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("Failed to receive: {:?}", e);
                    continue;
                }
            };

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}
