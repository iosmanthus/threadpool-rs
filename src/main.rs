use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

trait FnBox {
    fn call_box(self: Box<Self>) {}
}

impl<F> FnBox for F
where
    F: FnOnce(),
{
    fn call_box(self: Box<Self>) {
        (*self)();
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for job in receiver.iter() {
                job.call_box();
            }
        });
        Worker { id, thread }
    }
}

impl ThreadPool {
    pub fn new(count: usize) -> Self {
        let mut workers = Vec::with_capacity(count);
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..count {
            let worker = Worker::new(id, Arc::clone(&receiver));
            workers.push(worker);
        }
        Self { workers, sender }
    }

    pub fn excute<F>(&self, f: F)
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

fn main() {
    let pool = ThreadPool::new(4);
    pool.excute(|| {
        println!("1");
    });
}
