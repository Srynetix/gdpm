//! I/O crate.

mod error;
mod implementation;
mod interface;

pub use crate::error::Error;
pub use crate::implementation::DefaultIoAdapter;
pub use crate::interface::{IoAdapter, MockIoAdapter};
