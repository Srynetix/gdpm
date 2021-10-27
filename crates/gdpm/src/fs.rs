//! fs module

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use color_eyre::Report as Error;
use dirs;

const ROOT_CONFIG_FOLDER: &str = "gdpm";

/// Get configuration directory
pub fn get_configuration_directory() -> PathBuf {
    let config_directory = dirs::config_dir()
        .expect("No access to the configuration directory.")
        .join(ROOT_CONFIG_FOLDER);
    if !config_directory.exists() {
        fs::create_dir(&config_directory).unwrap_or_else(|_| {
            panic!(
                "Failed to create configuration directory: {:?}",
                config_directory
            )
        });
    }

    config_directory
}

/// Get file from configuration.
///
/// # Arguments
///
/// * `path` - Path to configuration file
///
pub fn get_configuration_file(path: &Path) -> PathBuf {
    get_configuration_directory().join(path)
}

/// Create configuration file.
///
/// # Arguments
///
/// * `path`- Path to configuration file
///
pub fn create_configuration_file(path: &Path) -> Result<(), Error> {
    let config_file = get_configuration_file(path);
    create_file(&config_file)
}

/// Create file.
///
/// # Arguments
///
/// * `path`- Path to file
///
pub fn create_file(path: &Path) -> Result<(), Error> {
    File::create(path)?;

    Ok(())
}

/// Read configuration file to string.
///
/// # Arguments
///
/// * `path` - Path to configuration file
///
pub fn read_configuration_file_to_string(path: &Path) -> Result<String, Error> {
    let config_file = get_configuration_file(path);
    read_file_to_string(&config_file)
}

/// Read file to string.
///
/// # Arguments
///
/// * `path` - Path to file
///
pub fn read_file_to_string(path: &Path) -> Result<String, Error> {
    if !path.exists() {
        create_file(path)?;
    }

    let mut contents = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

/// Write string to configuration file.
///
/// # Arguments
///
/// * `path` - Path to configuration file
/// * `contents` - String contents
///
pub fn write_string_to_configuration_file(path: &Path, contents: &str) -> Result<(), Error> {
    let config_file = get_configuration_file(path);
    write_string_to_file(&config_file, contents)
}

/// Write string to file.
///
/// # Arguments
///
/// * `path` - Path to file
/// * `contents` - String contents
///
pub fn write_string_to_file(path: &Path, contents: &str) -> Result<(), Error> {
    if !path.exists() {
        create_file(path)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}
