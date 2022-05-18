//! Downloader crate.

#![warn(missing_docs)]

pub mod download;
pub mod error;
mod implementation;
mod interface;
pub mod version;

pub use implementation::DownloadImpl;
pub use interface::DownloadAdapter;
