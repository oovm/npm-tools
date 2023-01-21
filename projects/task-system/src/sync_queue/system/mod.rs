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

impl<T: Debug> Display for TaskSystem<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.queue.try_lock() {
            Ok(o) => f.debug_list().entries(o.iter()).finish(),
            Err(_) => f.debug_struct("TaskSystem").field("queue", &"<LOCKED>").finish(),
        }
    }
}

impl<T> Default for TaskSystem<T> {
    fn default() -> Self {
        Self { interrupt: Arc::new(Mutex::new(false)), queue: Arc::new(Mutex::new(VecDeque::new())) }
    }
}

impl<T> TaskSystem<T> {
    /// Create a new task system.
    #[cfg(feature = "tokio")]
    pub fn start<F>(&self, callback: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn(T) -> bool + Send + 'static,
        T: Send + 'static,
    {
        self.resume();
        let queue = self.queue.clone();
        let interrupt = self.interrupt.clone();
        tokio::task::spawn_blocking(move || {
            loop {
                let task = match queue.try_lock() {
                    Ok(mut o) => match o.pop_front() {
                        Some(s) => s,
                        None => continue,
                    },
                    Err(_) => continue,
                };
                if can_interrupt(&interrupt) || !callback(task) {
                    break;
                }
            }
        })
    }

    ///  Cancel all tasks that match the condition.
    ///
    /// # Arguments
    ///
    /// * `condition`:
    ///
    /// returns: Vec<T, Global>
    ///
    /// # Examples
    ///
    /// ```
    /// use task_system::TaskSystem;
    /// let ts = TaskSystem::default();
    /// (1..10).for_each(|i| {
    ///     ts.send(i);
    /// });
    /// assert_eq!(ts.cancel_if(|i| i % 2 == 0), vec![2, 4, 6, 8]);
    /// ```
    pub fn cancel_if<F>(&self, condition: F) -> Vec<T>
    where
        F: Fn(&T) -> bool + Send + 'static,
        T: Send + 'static,
    {
        let mut result = Vec::new();
        match self.queue.try_lock() {
            Ok(mut o) => {
                let mut i = 0;
                while i < o.len() {
                    if condition(&o[i]) {
                        match o.remove(i) {
                            Some(s) => {
                                result.push(s);
                            }
                            None => continue,
                        }
                    }
                    else {
                        i += 1;
                    }
                }
            }
            Err(_) => {}
        }
        result
    }
    /// Cancel the first task that matches the condition.
    ///
    /// # Arguments
    ///
    /// * `condition`:
    ///
    /// returns: Option<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use task_system::TaskSystem;
    /// let ts = TaskSystem::default();
    /// (1..10).for_each(|i| {
    ///     ts.send(i);
    /// });
    /// assert_eq!(ts.cancel_first(|i| *i == 5), Some(5));
    /// ```
    pub fn cancel_first<F>(&self, condition: F) -> Option<T>
    where
        F: Fn(&T) -> bool + Send + 'static,
        T: Send + 'static,
    {
        match self.queue.try_lock() {
            Ok(mut o) => {
                let mut i = 0;
                while i < o.len() {
                    if condition(&o[i]) {
                        return o.remove(i);
                    }
                    else {
                        i += 1;
                    }
                }
                None
            }
            Err(_) => None,
        }
    }
    /// Send a new task to the task system.
    pub fn send(&self, task: T) -> bool {
        send_task(&self.queue, task).is_ok()
    }
    /// Get a sender for the task system.
    pub fn sender(&self) -> TaskSender<T> {
        TaskSender { refer: TaskSystem { interrupt: self.interrupt.clone(), queue: self.queue.clone() } }
    }
    /// Receive a task from the task system.
    pub fn receive(&self) -> Option<T> {
        match self.queue.try_lock() {
            Ok(mut o) => o.pop_front(),
            Err(_) => None,
        }
    }
    /// Consume a task from the task system.
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
    /// Interrupt all task runner.
    pub fn interrupt(&self) {
        match self.interrupt.try_lock() {
            Ok(mut o) => *o = true,
            Err(_) => (),
        }
    }
    /// Stop interrupting task runner.
    pub fn resume(&self) {
        match self.interrupt.try_lock() {
            Ok(mut o) => *o = false,
            Err(_) => (),
        }
    }
}

#[allow(dead_code)]
fn can_interrupt(interrupt: &Arc<Mutex<bool>>) -> bool {
    match interrupt.try_lock() {
        Ok(o) => *o,
        Err(_) => false,
    }
}
