use std::fmt::Display;

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

impl<T: Display> Display for TaskSystem<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.queue.try_lock() {
            Ok(o) => f.debug_list().entries(o.iter()).finish(),
            Err(_) => f.debug_struct("TaskSystem").field("queue", &"<LOCKED>").finish(),
        }
    }
}

impl<T> Default for TaskSystem<T> {
    fn default() -> Self {
        Self { interrupt: false, queue: Arc::new(Mutex::new(VecDeque::new())) }
    }
}

impl<T> TaskSystem<T> {
    #[cfg(feature = "tokio")]
    pub fn start<F>(&self, callback: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn(T) -> bool + Send + 'static,
        T: Send + 'static,
    {
        let queue = self.queue.clone();
        tokio::task::spawn_blocking(move || {
            loop {
                let task = match queue.try_lock() {
                    Ok(mut o) => match o.pop_front() {
                        Some(s) => s,
                        None => continue,
                    },
                    Err(_) => continue,
                };
                if self.interrupt || !callback(task) {
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
    pub fn consume<F>(&self, callback: F) -> bool
    where
        F: Fn(T) -> bool + Send + 'static,
        T: Send + 'static,
    {
        match self.receive() {
            Some(s) => {
                callback(s);
                true
            }
            None => false,
        }
    }
}
