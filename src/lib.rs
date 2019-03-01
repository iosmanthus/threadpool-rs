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

type Task = Box<dyn FnBox + Send + 'static>;

enum Message {
    NewJob(Task),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(task) => {
                    task.call_box();
                }
                Message::Terminate => {
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
        self.sender.send(Message::NewJob(Box::new(f))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in self.workers.iter_mut() {
            worker.thread.take().unwrap().join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let pool = ThreadPool::new(4);
        for i in 0..4 {
            pool.excute(move || loop {
                println!("{}", i);
            });
        }
        pool.excute(move || loop {
            println!("{}", "2333333");
        });
    }
}
