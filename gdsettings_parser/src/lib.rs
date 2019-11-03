//! gdsettings_parser
//!
//! Godot Settings parser

#![deny(missing_docs)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;

pub use parser::GdSettings;
pub use parser::GdValue;
pub use parser::{parse_gdsettings_file, serialize_gdsettings, serialize_gdvalue};
