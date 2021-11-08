//! Errors.

use reqwest::StatusCode;
use thiserror::Error;

/// Download error.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum DownloadError {
    #[error("File not found at URL '{0}'.")]
    NotFound(String),
    #[error("Unexpected status code '{0}'.")]
    UnexpectedStatusCode(StatusCode),
    #[error("Could not download file at URL '{0}'.")]
    ReqwestError(String, #[source] reqwest::Error),
    #[error("Could not create async runtime.")]
    AsyncRuntimeError(#[source] std::io::Error),
    #[error("I/O error.")]
    IoError(#[source] std::io::Error),
}
