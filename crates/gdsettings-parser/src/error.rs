use std::{
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use thiserror::Error;

/// Parser error
#[derive(Debug, Error)]
pub enum ParserError {
    /// Parse error
    #[error("Parse error")]
    ParseError,

    /// Parse error
    #[error("Parse error: {0}")]
    PestError(String),

    /// Type conversion error
    #[error("Type conversion error: '{0}'.")]
    TypeConversionError(String),
}

impl<E> From<pest::error::Error<E>> for ParserError
where
    E: std::fmt::Debug,
{
    fn from(error: pest::error::Error<E>) -> Self {
        Self::PestError(format!("{:?}", error))
    }
}

impl From<ParseIntError> for ParserError {
    fn from(error: ParseIntError) -> Self {
        Self::TypeConversionError(format!("{:?}", error))
    }
}

impl From<ParseFloatError> for ParserError {
    fn from(error: ParseFloatError) -> Self {
        Self::TypeConversionError(format!("{:?}", error))
    }
}

impl From<ParseBoolError> for ParserError {
    fn from(error: ParseBoolError) -> Self {
        Self::TypeConversionError(format!("{:?}", error))
    }
}

/// GdSettings error
#[derive(Debug, Error)]
pub enum GdSettingsError {
    /// Missing section
    #[error("Missing section '{0}' in settings.")]
    MissingSection(String),
    /// Missing property
    #[error("Missing property '{0}' in settings.")]
    MissingProperty(String),
}
