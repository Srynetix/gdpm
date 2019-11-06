//! gdpm
//!
//! Godot Package manager

#![deny(missing_docs)]

pub mod config;
pub mod engine;
pub mod fs;
pub mod project;
mod shell;

pub use shell::run_shell;
