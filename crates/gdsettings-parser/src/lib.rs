//! Godot Settings parser

#![warn(missing_docs)]

mod error;
mod gdvalue;
mod parser;

pub use error::{GdSettingsError, ParserError};
pub use gdvalue::GdValue;
pub use parser::{
    parse_gdsettings_file, serialize_gdsettings, GdSettings, GdSettingsMap, GdSettingsType,
};
