//! I/O crate.

#![warn(missing_docs)]

pub mod error;

use std::{
    fs::{File, OpenOptions, ReadDir},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;
use tracing::debug;
use zip::read::ZipArchive;

use crate::error::IoError;

/// Get user configuration directory.
pub fn get_user_configuration_directory() -> Result<PathBuf, IoError> {
    dirs::config_dir().ok_or(IoError::UnavailableUserDirError)
}

/// Create file.
pub fn create_file(path: &Path) -> Result<File, IoError> {
    debug!(
        "Creating file at path '{}' ...",
        path.display().to_string().color("green")
    );
    File::create(path).map_err(|e| IoError::CreateFileError(path.into(), e))
}

/// Create directory.
pub fn create_dir(path: &Path) -> Result<(), IoError> {
    debug!(
        "Creating folder at path '{}' ...",
        path.display().to_string().color("green")
    );
    std::fs::create_dir(path).map_err(|e| IoError::CreateFolderError(path.into(), e))
}

/// Read file to string.
pub fn read_file_to_string(path: &Path) -> Result<String, IoError> {
    let mut contents = String::new();
    let mut file = open_file_read(path)?;

    file.read_to_string(&mut contents)
        .map_err(|e| IoError::ReadFileError(path.to_owned(), e))?;

    Ok(contents)
}

/// Write string to file.
pub fn write_string_to_file(path: &Path, contents: &str) -> Result<File, IoError> {
    write_bytes_to_file(path, contents.as_bytes())
}

/// Write bytes to file.
pub fn write_bytes_to_file(path: &Path, contents: &[u8]) -> Result<File, IoError> {
    if !path.exists() {
        create_file(path)?;
    }

    let mut file = open_file_write(path)?;

    debug!(
        "Writing {} bytes to file '{}' ...",
        contents.len().to_string().color("green"),
        path.display().to_string().color("green")
    );
    file.write_all(contents)
        .map_err(|e| IoError::WriteFileError(path.to_owned(), e))?;

    Ok(file)
}

/// Open file.
pub fn open_file_read(path: &Path) -> Result<File, IoError> {
    File::open(path).map_err(|e| IoError::OpenFileError(path.to_owned(), e))
}

/// Open file.
pub fn open_file_write(path: &Path) -> Result<File, IoError> {
    OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|e| IoError::OpenFileError(path.to_owned(), e))
}

/// Remove file.
pub fn remove_file(path: &Path) -> Result<(), IoError> {
    debug!(
        "Removing file '{}' ...",
        path.display().to_string().color("green")
    );
    fs_extra::file::remove(path).map_err(|e| IoError::RemoveFileError(path.to_owned(), e))
}

/// Remove directory.
pub fn remove_dir_all(path: &Path) -> Result<(), IoError> {
    debug!(
        "Removing directory '{}' ...",
        path.display().to_string().color("green")
    );

    ::remove_dir_all::remove_dir_all(path)
        .map_err(|e| IoError::RemoveFolderError(path.to_owned(), e))
}

/// Copy file.
pub fn copy_file(source: &Path, destination: &Path) -> Result<(), IoError> {
    let options = fs_extra::file::CopyOptions {
        overwrite: true,
        ..Default::default()
    };

    debug!(
        "Copying file from '{}' to '{}' ...",
        source.display().to_string().color("green"),
        destination.display().to_string().color("green")
    );
    fs_extra::file::copy(source, destination, &options)
        .map(|_| ())
        .map_err(|e| IoError::CopyFileError(source.into(), destination.into(), e))
}

/// Copy directory.
pub fn copy_dir(source: &Path, destination: &Path) -> Result<(), IoError> {
    let options = fs_extra::dir::CopyOptions {
        overwrite: true,
        ..Default::default()
    };

    debug!(
        "Copying directory from '{}' to '{}' ...",
        source.display().to_string().color("green"),
        destination.display().to_string().color("green")
    );
    fs_extra::dir::copy(source, destination, &options)
        .map(|_| ())
        .map_err(|e| IoError::CopyFolderError(source.into(), destination.into(), e))
}

/// Read directory.
pub fn read_dir(path: &Path) -> Result<ReadDir, IoError> {
    debug!(
        "Reading files from directory '{}' ...",
        path.display().to_string().color("green")
    );
    std::fs::read_dir(path).map_err(|e| IoError::ReadDirError(path.into(), e))
}

/// Open and extract ZIP archive.
pub fn open_and_extract_zip(source: &Path, destination: &Path) -> Result<(), IoError> {
    let file = open_file_read(source)?;

    debug!(
        "Reading ZIP archive from '{}' ...",
        source.display().to_string().color("green")
    );
    let mut archive =
        ZipArchive::new(file).map_err(|e| IoError::OpenZipError(source.to_owned(), e))?;

    debug!(
        "Extracting archive to folder '{}' ...",
        destination.display().to_string().color("green")
    );
    archive
        .extract(&destination)
        .map_err(|e| IoError::ExtractZipError(source.to_owned(), destination.to_owned(), e))?;

    Ok(())
}
