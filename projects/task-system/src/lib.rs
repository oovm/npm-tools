#![warn(missing_docs)]
#![doc = include_str!("../Readme.md")]

pub use crate::sync_queue::{TaskSender, TaskSystem};

mod sender;
mod sync_queue;

/// A trait for objects that can be cancelled.
pub trait Cancellable {
    /// Cancel the object.
    fn cancel(&mut self);
}
