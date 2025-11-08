use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

pub struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let handle = std::thread::spawn(move || loop {
            // lock is released immediately after let statement so other threads can acquire lock
            // again when current job is running
            let job = receiver.lock().unwrap().recv();

            // using while let will hold lock until while let loop ends, which means current thread
            // blocks other threads from acquiring lock when running job, so mutli-thread not working
            match job {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    eprintln!("Worker {id} disconnected, shuting down");
                }
            }
        });
        Worker { id, handle }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Option<Sender<Job>>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    fn new(count: usize) -> ThreadPool {
        assert!(count > 0);

        let mut workers = Vec::with_capacity(count);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for i in 0..count {
            let worker = Worker::new(i, Arc::clone(&receiver));
            workers.push(worker)
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            worker.handle.join().unwrap();
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut message = String::new();
        match reader.read_line(&mut message) {
            Ok(0) => {
                println!("End");
                break;
            }
            Ok(bytes_read) => {
                println!("Received message {bytes_read} bytes : {message}");
                let _ = stream.write_all(message.as_bytes());

                message.clear();
            }
            Err(e) => {
                eprintln!("Error: {e}")
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:8000").unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Server started on {}", addr);

    let pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
                continue;
            }
        }
    }
}
