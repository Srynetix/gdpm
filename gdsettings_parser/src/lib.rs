//! gdsettings_parser
//!
//! Godot Settings parser

#![deny(missing_docs)]

#[macro_use]
extern crate failure;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod gdvalue;
mod parser;

pub use gdvalue::GdValue;
pub use parser::{parse_gdsettings_file, serialize_gdsettings, GdSettings};
