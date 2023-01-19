#![forbid(missing_docs)]
#![doc = include_str!("../Readme.md")]

use std::{path::Path, str::FromStr};

use url::{ParseError, Url};

mod sync_queue;
pub mod third_party;

#[derive(Default, Debug)]
pub struct TaskSystem<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    sender: Arc<Mutex<Sender<T>>>,
    receiver: Receiver<T>,
}
