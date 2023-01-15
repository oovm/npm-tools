#![doc = include_str!("readme.md")]

#[cfg(feature = "trauma")]
pub use trauma::downloader::{Downloader, DownloaderBuilder};

#[cfg(feature = "serde")]
mod der;
mod display;
#[cfg(feature = "trauma")]
mod download;
#[cfg(feature = "serde")]
mod ser;
