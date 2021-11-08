//! Core crate.

#![warn(missing_docs)]

pub mod config;
pub mod engine;
pub mod error;
pub mod plugins;
pub mod project;

pub mod downloader {
    //! Downloader module.
    pub use gdpm_downloader::*;
}
