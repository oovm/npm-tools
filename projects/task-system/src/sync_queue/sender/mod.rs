use super::*;

impl<T> Clone for TaskSender<T> {
    fn clone(&self) -> Self {
        TaskSender { refer: TaskSystem { interrupt: self.refer.interrupt.clone(), queue: self.refer.queue.clone() } }
    }
}

impl<T> From<TaskSystem<T>> for TaskSender<T> {
    fn from(refer: TaskSystem<T>) -> Self {
        TaskSender { refer }
    }
}

impl<T> TaskSender<T> {
    /// Send a task to the task system.
    ///
    /// # Arguments
    ///
    /// * `task`:
    ///
    /// returns: Result<(), PoisonError<MutexGuard<VecDeque<T, Global>>>>
    ///
    /// # Examples
    ///
    /// ```
    /// use task_system::TaskSystem;
    /// let ts = TaskSystem::default();
    /// let ss = ts.sender();
    /// assert!(ss.send(1))
    /// ```
    pub fn send(&self, task: T) -> bool {
        send_task(&self.refer.queue, task).is_ok()
    }
}
