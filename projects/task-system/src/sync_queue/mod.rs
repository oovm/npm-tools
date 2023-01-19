use std::{
    collections::VecDeque,
    sync::{
        mpsc::{channel, Receiver, SendError, Sender},
        Arc, Mutex,
    },
};

use crate::Cancellable;

/// A task system that can be used to send tasks to a thread pool.
#[derive(Debug)]
pub struct TaskSystem<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
}

pub struct TaskSender<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
}

impl<T> Default for TaskSystem<T> {
    fn default() -> Self {
        let (tx, rx) = channel::<T>();
        Self { queue: Arc::new(Mutex::new(VecDeque::new())), senders: Arc::new(Mutex::new(tx)), receiver: rx }
    }
}

impl<T> TaskSystem<T> {
    pub fn start<F>(&self, callback: F)
    where
        F: Fn(T) + Send + 'static,
    {
        todo!()
    }

    pub fn send(&self, task: T) {
        let sender = self.sender();
    }

    pub fn sender(&self) -> TaskSender<T> {
        TaskSender { queue: self.queue.clone(), senders: self.senders.clone() }
    }
}

impl<T> TaskSender<T> {
    pub fn send(&self, task: T) -> Result<(), SendError<T>> {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(task);
        let mut sender = self.senders.lock().unwrap();
        sender.send(task)
    }
}
