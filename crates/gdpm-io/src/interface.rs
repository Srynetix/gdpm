use crate::error::Error;

use std::fs::ReadDir;
use std::path::{Path, PathBuf};

/// IO adapter.
#[mockall::automock]
pub trait IoAdapter {
    /// Get user configuration directory.
    fn get_user_configuration_directory(&self) -> Result<PathBuf, Error>;

    /// Create file.
    fn create_file(&self, path: &Path) -> Result<(), Error>;

    /// Create directory.
    fn create_dir(&self, path: &Path) -> Result<(), Error>;

    /// Read file to string.
    fn read_file_to_string(&self, path: &Path) -> Result<String, Error>;

    /// Write string to file.
    fn write_string_to_file(&self, path: &Path, contents: &str) -> Result<(), Error>;

    /// Write bytes to file.
    fn write_bytes_to_file(&self, path: &Path, contents: &[u8]) -> Result<(), Error>;

    /// Remove file.
    fn remove_file(&self, path: &Path) -> Result<(), Error>;

    /// Remove dir will all its contents.
    fn remove_dir_all(&self, path: &Path) -> Result<(), Error>;

    /// Check if path exists.
    fn path_exists(&self, path: &Path) -> bool;

    /// Check if path is a file.
    fn path_is_file(&self, path: &Path) -> bool;

    /// Copy file.
    fn copy_file(&self, source: &Path, destination: &Path) -> Result<(), Error>;

    /// Copy directory.
    fn copy_dir(&self, source: &Path, destination: &Path) -> Result<(), Error>;

    /// Read directory contents.
    fn read_dir(&self, path: &Path) -> Result<ReadDir, Error>;

    /// Open and extract ZIP file.
    fn open_and_extract_zip(&self, source: &Path, destination: &Path) -> Result<(), Error>;
}
