#![warn(missing_docs)]
#![doc = include_str!("../Readme.md")]

pub use crate::sync_queue::{TaskSender, TaskSystem};

mod sender;
mod sync_queue;

pub trait Cancellable {
    fn cancel(&mut self);
}
