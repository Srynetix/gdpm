//! gdpm
//!
//! Godot Package manager

#![deny(missing_docs)]

pub mod actions;
pub mod config;
pub mod fs;
mod shell;

pub use shell::run_shell;
