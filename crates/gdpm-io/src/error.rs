//! Errors.

use std::path::PathBuf;

/// I/O error.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unavailable user configuration directory.")]
    UnavailableUserDirError,

    #[error("Could not open file '{0}': {1}")]
    OpenFileError(PathBuf, String),

    #[error("Could not read file '{0}': {1}")]
    ReadFileError(PathBuf, String),

    #[error("Could not write to file '{0}': {1}")]
    WriteFileError(PathBuf, String),

    #[error("Could not create file '{0}': {1}")]
    CreateFileError(PathBuf, String),

    #[error("Could not create folder '{0}': {1}")]
    CreateFolderError(PathBuf, String),

    #[error("Could not remove folder '{0}': {1}")]
    RemoveFolderError(PathBuf, String),

    #[error("Could not remove file '{0}': {1}")]
    RemoveFileError(PathBuf, String),

    #[error("Could not copy folder '{0}' to '{1}': {2}")]
    CopyFolderError(PathBuf, PathBuf, String),

    #[error("Could not copy file '{0}' to '{1}': {2}")]
    CopyFileError(PathBuf, PathBuf, String),

    #[error("Could not read folder '{0}': {1}")]
    ReadDirError(PathBuf, String),

    #[error("Could not read folder entry in '{0}': {1}.")]
    ReadDirEntryError(PathBuf, String),

    #[error("Failed to execute command: {0}")]
    CommandExecutionError(String),

    #[error("Cannot get parent path for '{0}'")]
    NoParentFolder(PathBuf),

    #[error("Could not open zip file '{0}': {1}")]
    OpenZipError(PathBuf, String),

    #[error("Could not extract zip file '{0}' to '{1}': {2}")]
    ExtractZipError(PathBuf, PathBuf, String),

    #[error("Write error: '{0}'")]
    WriteError(String),
}
