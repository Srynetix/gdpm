//! Core crate.

#![warn(missing_docs)]

pub mod config;
pub mod engine;
pub mod error;
pub mod plugins;
pub mod project;

pub mod io {
    //! IO module.
    pub use gdpm_io::{IoAdapter, IoImpl};
}

pub mod downloader {
    //! Downloader module.
    pub use gdpm_downloader::*;
}

pub mod types {
    //! Types module.
    pub use gdpm_types::*;
}
