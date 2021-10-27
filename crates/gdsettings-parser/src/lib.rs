//! Godot Settings parser

#![deny(missing_docs)]

mod gdvalue;
mod parser;

pub use gdvalue::GdValue;
pub use parser::{parse_gdsettings_file, serialize_gdsettings, GdSettings, GdSettingsError, ParserError};
