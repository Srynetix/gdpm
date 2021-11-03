//! Errors.

use std::path::PathBuf;

use thiserror::Error;

/// I/O error.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum IoError {
    #[error("Unavailable user configuration directory.")]
    UnavailableUserDirError,
    #[error("Could not open file '{0}'.")]
    OpenFileError(PathBuf, #[source] std::io::Error),
    #[error("Could not read file '{0}'.")]
    ReadFileError(PathBuf, #[source] std::io::Error),
    #[error("Could not write to file '{0}'.")]
    WriteFileError(PathBuf, #[source] std::io::Error),
    #[error("Could not create file '{0}'.")]
    CreateFileError(PathBuf, #[source] std::io::Error),
    #[error("Could not create folder '{0}'.")]
    CreateFolderError(PathBuf, #[source] std::io::Error),
    #[error("Could not remove folder '{0}'.")]
    RemoveFolderError(PathBuf, #[source] std::io::Error),
    #[error("Could not remove file '{0}'.")]
    RemoveFileError(PathBuf, #[source] fs_extra::error::Error),
    #[error("Could not copy folder '{0}' to '{1}'.")]
    CopyFolderError(PathBuf, PathBuf, #[source] fs_extra::error::Error),
    #[error("Could not copy file '{0}' to '{1}'.")]
    CopyFileError(PathBuf, PathBuf, #[source] fs_extra::error::Error),
    #[error("Could not read folder '{0}'.")]
    ReadDirError(PathBuf, #[source] std::io::Error),
    #[error("Could not read folder entry in '{0}'.")]
    ReadDirEntryError(PathBuf, #[source] std::io::Error),
    #[error("Failed to execute command.")]
    CommandExecutionError(#[source] std::io::Error),
    #[error("Could not open zip file '{0}'.")]
    OpenZipError(PathBuf, #[source] zip::result::ZipError),
    #[error("Could not extract zip file '{0}' to '{1}'.")]
    ExtractZipError(PathBuf, PathBuf, #[source] zip::result::ZipError),
}
