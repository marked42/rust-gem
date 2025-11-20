use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::JoinHandle,
};

pub type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    pub id: usize,
    pub handle: JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
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
