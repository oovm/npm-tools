use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

mod sender;
mod system;

/// A task system that can be used to send tasks to a thread pool.
pub struct TaskSystem<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
}

/// Sender for a task system.
pub struct TaskSender<T> {
    refer: TaskSystem<T>,
}

fn send_task<T>(queue: &Arc<Mutex<VecDeque<T>>>, task: T) -> Result<(), PoisonError<MutexGuard<'_, VecDeque<T>>>> {
    match queue.lock() {
        Ok(mut o) => Ok(o.push_back(task)),
        Err(e) => Err(e),
    }
}
