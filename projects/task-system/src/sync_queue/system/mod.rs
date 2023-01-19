use std::{
    fmt::{Debug, Formatter},
    thread::{spawn, JoinHandle},
};
use tokio::task::spawn_blocking;

use super::*;

impl<T: Debug> Debug for TaskSystem<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = &mut f.debug_struct("TaskSystem");
        match self.queue.try_lock() {
            Ok(o) => s.field("queue", &o).finish(),
            Err(_) => s.field("queue", &"<LOCKED>").finish(),
        }
    }
}

impl<T> Default for TaskSystem<T> {
    fn default() -> Self {
        Self { queue: Arc::new(Mutex::new(VecDeque::new())) }
    }
}

impl<T> TaskSystem<T> {
    pub fn start<F>(&self, callback: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn(T) -> bool + Send + 'static,
        T: Send + 'static,
    {
        let queue = self.queue.clone();
        spawn_blocking(move || {
            loop {
                let task = match queue.try_lock() {
                    Ok(mut o) => match o.pop_front() {
                        Some(s) => s,
                        None => continue,
                    },
                    Err(_) => continue,
                };
                if !callback(task) {
                    break;
                }
            }
        })
    }

    pub fn send(&self, task: T) -> bool {
        send_task(&self.queue, task).is_ok()
    }
    pub fn sender(&self) -> TaskSender<T> {
        TaskSender { refer: TaskSystem { queue: self.queue.clone() } }
    }
    pub fn receive(&self) -> Option<T> {
        match self.queue.try_lock() {
            Ok(mut o) => o.pop_front(),
            Err(_) => None,
        }
    }
}
