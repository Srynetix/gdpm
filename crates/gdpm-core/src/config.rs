//! Config module

use std::{fs::File, path::{Path, PathBuf}};

use gdsettings_parser::{GdSettings, GdSettingsError, ParserError, parse_gdsettings_file};
use thiserror::Error;

use crate::fs::{
    read_configuration_file_to_string, read_file_to_string, write_string_to_configuration_file,
    write_string_to_file,
};

/// Config error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Project not found
    #[error("project not found: {}", path)]
    ProjectNotFound {
        /// Project path
        path: String,
    },
    /// Malformed project
    #[error("malformed project")]
    MalformedProject,
    /// Malformed engine configuration
    #[error("engine not found: {}", path)]
    EngineNotFound {
        /// Engine path or version
        path: String,
    },
    /// IO Error
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Parser error
    #[error(transparent)]
    ParserError(#[from] ParserError),
    /// GdSettings error
    #[error(transparent)]
    GdSettingsError(#[from] GdSettingsError),
}

/// Engines section in settings
pub const ENGINES_SECTION: &str = "engines";

const CONFIG_PATH: &str = "gdpm.cfg";

/// Get project configuration path
pub fn get_project_configuration(path: &Path) -> PathBuf {
    path.join("project.godot")
}

/// Read gdpm configuration
pub fn read_gdpm_configuration() -> Result<GdSettings, ConfigError> {
    let contents = read_configuration_file_to_string(Path::new(CONFIG_PATH))?;

    parse_gdsettings_file(&contents).map_err(Into::into)
}

/// Write gdpm configuration
pub fn write_gdpm_configuration(settings: GdSettings) -> Result<File, ConfigError> {
    let contents = settings.to_string();

    println!("Writing gdpm configuration ...");
    write_string_to_configuration_file(Path::new(CONFIG_PATH), &contents).map_err(Into::into)
}

/// Read project configuration
pub fn read_project_configuration(path: &Path) -> Result<GdSettings, ConfigError> {
    // Check for project.godot
    let project = get_project_configuration(path);
    if !project.exists() {
        return Err(ConfigError::ProjectNotFound {
            path: project.to_string_lossy().to_string(),
        })?;
    }

    let contents = read_file_to_string(&project)?;
    parse_gdsettings_file(&contents).map_err(Into::into)
}

/// Write project configuration.
pub fn write_project_configuration(path: &Path, settings: GdSettings) -> Result<File, ConfigError> {
    let contents = settings.to_string();

    let project = get_project_configuration(path);
    if !project.exists() {
        return Err(ConfigError::ProjectNotFound {
            path: project.to_string_lossy().to_string(),
        })?;
    }

    println!(
        "Writing project configuration to path: {}",
        project.to_string_lossy()
    );
    write_string_to_file(&project, &contents).map_err(Into::into)
}
