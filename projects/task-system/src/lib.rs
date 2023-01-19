#![forbid(missing_docs)]
#![doc = include_str!("../Readme.md")]

use std::str::FromStr;

pub use crate::sync_queue::TaskSystem;

mod sync_queue;

pub trait Cancellable {
    fn cancel(&mut self);
}
