//! gdpm
//!
//! Godot Package manager

#![deny(missing_docs)]

extern crate structopt;
#[macro_use]
extern crate failure;
extern crate dirs;
extern crate question;
extern crate slugify;

pub mod actions;
pub mod config;
pub mod fs;
mod shell;

pub use shell::run_shell;
