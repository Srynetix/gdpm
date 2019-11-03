//! gdpm
//!
//! Godot Package manager

extern crate structopt;
#[macro_use]
extern crate failure;

pub mod actions;
mod shell;

pub use shell::run_shell;
