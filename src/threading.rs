//! Multi-threading module.
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};
use tracing::{error, info};

/// An orchastrator of worker threads sending jobs to the workers.
///
/// # Examples
///
/// ```
/// use rws::ThreadPool;
/// let pool = ThreadPool::new(4);
/// pool.execute(|| println!("Printing this in a thread"));
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new `ThreadPool` with a number of `size` (>0) worker threads.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Send function `f` as a job to the worker threads.
    ///
    /// The job will be executed by the first available worker.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .expect("ThreadPool has been shut down")
            .send(job)
            .expect("Failed to send job to worker thread");
    }
}

impl Drop for ThreadPool {
    /// Drop the `ThreadPool`.
    ///
    /// When dropping the `ThreadPool`, we will wait for all worker threads to
    /// finish execution.
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            info!(worker.id, "Shutting down worker");
            worker
                .thread
                .join()
                .expect("Worker {worker.id} failed to join");
        }
    }
}

/// Wrapper around a `thread`.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Create a new worker.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier of the worker
    /// * `receiver` - Receiver via which the worker receives new jobs
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        info!(worker.id = id, "Worker got a job; executing");

                        job();
                    }
                    Err(_) => {
                        error!(
                            worker.id = id,
                            "Worker disconnected; shutting down"
                        );
                        break;
                    }
                }
            }
        });
        Worker { id, thread }
    }
}
