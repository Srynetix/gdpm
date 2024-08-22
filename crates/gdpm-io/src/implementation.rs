use crate::{error::Error, interface::IoAdapter};
use colored::Colorize;

use std::{
    fs::{File, OpenOptions, ReadDir},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tracing::debug;
use zip::ZipArchive;

/// IO adapter implementation.
pub struct DefaultIoAdapter;

impl DefaultIoAdapter {
    fn open_file_read(&self, path: &Path) -> Result<File, Error> {
        File::open(path).map_err(|e| Error::OpenFileError(path.to_owned(), e.to_string()))
    }

    fn open_file_write(&self, path: &Path) -> Result<File, Error> {
        OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(path)
            .map_err(|e| Error::OpenFileError(path.to_owned(), e.to_string()))
    }
}

impl IoAdapter for DefaultIoAdapter {
    fn get_user_configuration_directory(&self) -> Result<PathBuf, Error> {
        dirs::config_dir().ok_or(Error::UnavailableUserDirError)
    }

    fn create_file(&self, path: &Path) -> Result<(), Error> {
        debug!(
            "Creating file at path '{}' ...",
            path.display().to_string().color("green")
        );
        File::create(path)
            .map_err(|e| Error::CreateFileError(path.into(), e.to_string()))
            .map(|_| ())
    }

    fn create_dir(&self, path: &Path) -> Result<(), Error> {
        debug!(
            "Creating folder at path '{}' ...",
            path.display().to_string().color("green")
        );
        std::fs::create_dir(path).map_err(|e| Error::CreateFolderError(path.into(), e.to_string()))
    }

    fn read_file_to_string(&self, path: &Path) -> Result<String, Error> {
        let mut contents = String::new();
        let mut file = self.open_file_read(path)?;

        file.read_to_string(&mut contents)
            .map_err(|e| Error::ReadFileError(path.to_owned(), e.to_string()))?;

        Ok(contents)
    }

    fn write_string_to_file(&self, path: &Path, contents: &str) -> Result<(), Error> {
        self.write_bytes_to_file(path, contents.as_bytes())
    }

    fn write_bytes_to_file(&self, path: &Path, contents: &[u8]) -> Result<(), Error> {
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
            .map_err(|e| Error::WriteFileError(path.to_owned(), e.to_string()))?;

        Ok(())
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn path_is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn remove_file(&self, path: &Path) -> Result<(), Error> {
        debug!(
            "Removing file '{}' ...",
            path.display().to_string().color("green")
        );
        fs_extra::file::remove(path)
            .map_err(|e| Error::RemoveFileError(path.to_owned(), e.to_string()))
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), Error> {
        debug!(
            "Removing directory '{}' ...",
            path.display().to_string().color("green")
        );

        ::remove_dir_all::remove_dir_all(path)
            .map_err(|e| Error::RemoveFolderError(path.to_owned(), e.to_string()))
    }

    fn copy_file(&self, source: &Path, destination: &Path) -> Result<(), Error> {
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
            .map_err(|e| Error::CopyFileError(source.into(), destination.into(), e.to_string()))
    }

    fn copy_dir(&self, source: &Path, destination: &Path) -> Result<(), Error> {
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
            .map_err(|e| Error::CopyFolderError(source.into(), destination.into(), e.to_string()))
    }

    fn read_dir(&self, path: &Path) -> Result<ReadDir, Error> {
        debug!(
            "Reading files from directory '{}' ...",
            path.display().to_string().color("green")
        );
        std::fs::read_dir(path).map_err(|e| Error::ReadDirError(path.into(), e.to_string()))
    }

    fn open_and_extract_zip(&self, source: &Path, destination: &Path) -> Result<(), Error> {
        let file = self.open_file_read(source)?;

        debug!(
            "Reading ZIP archive from '{}' ...",
            source.display().to_string().color("green")
        );
        let mut archive = ZipArchive::new(file)
            .map_err(|e| Error::OpenZipError(source.to_owned(), e.to_string()))?;

        debug!(
            "Extracting archive to folder '{}' ...",
            destination.display().to_string().color("green")
        );
        archive.extract(destination).map_err(|e| {
            Error::ExtractZipError(source.to_owned(), destination.to_owned(), e.to_string())
        })?;

        Ok(())
    }

    fn write_stderr(&self, message: String) -> Result<(), Error> {
        write!(std::io::stderr(), "{}", message).map_err(|e| Error::WriteError(e.to_string()))
    }

    fn write_stdout(&self, message: String) -> Result<(), Error> {
        write!(std::io::stdout(), "{}", message).map_err(|e| Error::WriteError(e.to_string()))
    }
}
