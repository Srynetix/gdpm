//! Downloader crate.

#![warn(missing_docs)]

pub mod download;
pub mod error;
mod implementation;
mod interface;

pub use implementation::DefaultDownloadAdapter;
pub use interface::{DownloadAdapter, MockDownloadAdapter};
