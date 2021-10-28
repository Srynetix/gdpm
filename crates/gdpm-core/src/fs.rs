//! fs module

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use dirs;

const ROOT_CONFIG_FOLDER: &str = "gdpm";

/// Get configuration directory.
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
pub fn get_configuration_file(path: &Path) -> PathBuf {
    get_configuration_directory().join(path)
}

/// Create configuration file.
pub fn create_configuration_file(path: &Path) -> Result<File, std::io::Error> {
    let config_file = get_configuration_file(path);
    File::create(&config_file)
}

/// Read configuration file to string.
pub fn read_configuration_file_to_string(path: &Path) -> Result<String, std::io::Error> {
    let config_file = get_configuration_file(path);
    read_file_to_string(&config_file)
}

/// Read file to string.
pub fn read_file_to_string(path: &Path) -> Result<String, std::io::Error> {
    if !path.exists() {
        File::create(path)?;
    }

    let mut contents = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

/// Write string to configuration file.
pub fn write_string_to_configuration_file(
    path: &Path,
    contents: &str,
) -> Result<File, std::io::Error> {
    let config_file = get_configuration_file(path);
    write_string_to_file(&config_file, contents)
}

/// Write string to file.
pub fn write_string_to_file(path: &Path, contents: &str) -> Result<File, std::io::Error> {
    if !path.exists() {
        File::create(path)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(file)
}
