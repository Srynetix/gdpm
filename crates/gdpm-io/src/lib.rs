//! I/O crate.

#![warn(missing_docs)]

mod error;
mod implementation;
mod interface;

pub use crate::error::IoError;
pub use crate::implementation::IoImpl;
pub use crate::interface::IoAdapter;
