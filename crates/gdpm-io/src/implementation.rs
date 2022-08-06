use crate::{error::IoError, interface::IoAdapter};
use colored::Colorize;
use std::{
    fs::{File, OpenOptions, ReadDir},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tracing::debug;
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

/// IO adapter implementation.
pub struct IoImpl;

impl IoImpl {
    fn open_file_read(&self, path: &Path) -> Result<File, IoError> {
        File::open(path).map_err(|e| IoError::OpenFileError(path.to_owned(), e))
    }

    fn open_file_write(&self, path: &Path) -> Result<File, IoError> {
        OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(path)
            .map_err(|e| IoError::OpenFileError(path.to_owned(), e))
    }
}

impl IoAdapter for IoImpl {
    fn get_user_configuration_directory(&self) -> Result<PathBuf, IoError> {
        dirs::config_dir().ok_or(IoError::UnavailableUserDirError)
    }

    fn create_file(&self, path: &Path) -> Result<(), IoError> {
        debug!(
            "Creating file at path '{}' ...",
            path.display().to_string().color("green")
        );
        File::create(path)
            .map_err(|e| IoError::CreateFileError(path.into(), e))
            .map(|_| ())
    }

    fn create_dir(&self, path: &Path) -> Result<(), IoError> {
        debug!(
            "Creating folder at path '{}' ...",
            path.display().to_string().color("green")
        );
        std::fs::create_dir(path).map_err(|e| IoError::CreateFolderError(path.into(), e))
    }

    fn read_file_to_string(&self, path: &Path) -> Result<String, IoError> {
        let mut contents = String::new();
        let mut file = self.open_file_read(path)?;

        file.read_to_string(&mut contents)
            .map_err(|e| IoError::ReadFileError(path.to_owned(), e))?;

        Ok(contents)
    }

    fn write_string_to_file(&self, path: &Path, contents: &str) -> Result<(), IoError> {
        self.write_bytes_to_file(path, contents.as_bytes())
    }

    fn write_bytes_to_file(&self, path: &Path, contents: &[u8]) -> Result<(), IoError> {
        if !self.path_exists(path) {
            self.create_file(path)?;
        }

        let mut file = self.open_file_write(path)?;

        debug!(
            "Writing {} bytes to file '{}' ...",
            contents.len().to_string().color("green"),
            path.display().to_string().color("green")
        );
        file.write_all(contents)
            .map_err(|e| IoError::WriteFileError(path.to_owned(), e))?;

        Ok(())
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn path_is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn remove_file(&self, path: &Path) -> Result<(), IoError> {
        debug!(
            "Removing file '{}' ...",
            path.display().to_string().color("green")
        );
        fs_extra::file::remove(path).map_err(|e| IoError::RemoveFileError(path.to_owned(), e))
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), IoError> {
        debug!(
            "Removing directory '{}' ...",
            path.display().to_string().color("green")
        );

        ::remove_dir_all::remove_dir_all(path)
            .map_err(|e| IoError::RemoveFolderError(path.to_owned(), e))
    }

    fn copy_file(&self, source: &Path, destination: &Path) -> Result<(), IoError> {
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

    fn copy_dir(&self, source: &Path, destination: &Path) -> Result<(), IoError> {
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

    fn read_dir(&self, path: &Path) -> Result<ReadDir, IoError> {
        debug!(
            "Reading files from directory '{}' ...",
            path.display().to_string().color("green")
        );
        std::fs::read_dir(path).map_err(|e| IoError::ReadDirError(path.into(), e))
    }

    fn open_and_extract_zip(&self, source: &Path, destination: &Path) -> Result<(), IoError> {
        let file = self.open_file_read(source)?;

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

    fn find_files_in_dir(&self, source: &Path, extension: &str) -> Result<Vec<PathBuf>, IoError> {
        let filter_extension = |entry: &DirEntry| -> bool {
            if entry.path().is_dir() {
                true
            } else {
                entry
                    .file_name()
                    .to_str()
                    .map(|s| s.ends_with(extension))
                    .unwrap_or(false)
            }
        };

        let mut output = vec![];
        for entry in WalkDir::new(source)
            .into_iter()
            .filter_entry(filter_extension)
        {
            let entry = entry.map_err(IoError::WalkDirError)?;
            if entry.path().is_dir() {
                continue;
            }

            output.push(entry.path().to_owned());
        }

        Ok(output)
    }
}
