use std::sync::{
    Arc, Mutex,
    mpsc::{self, Sender},
};

use crate::worker::{Job, Worker};

pub struct ThreadPool {
    sender: Option<Sender<Job>>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(count: usize) -> ThreadPool {
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

    pub fn execute<F>(&self, f: F)
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
