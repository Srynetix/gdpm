//! Config module

use std::path::{Path, PathBuf};

use color_eyre::Report as Error;

use crate::fs::{
    read_configuration_file_to_string, read_file_to_string, write_string_to_configuration_file,
    write_string_to_file,
};
use gdsettings_parser::{parse_gdsettings_file, GdSettings};

/// Config error
#[derive(Debug, thiserror::Error)]
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
}

/// Engines section in settings
pub const ENGINES_SECTION: &str = "engines";

const CONFIG_PATH: &str = "gdpm.cfg";

/// Get project configuration path
///
/// # Arguments
///
/// * `path` - Project path
///
pub fn get_project_configuration(path: &Path) -> PathBuf {
    path.join("project.godot")
}

/// Read gdpm configuration
pub fn read_gdpm_configuration() -> Result<GdSettings, Error> {
    let contents = read_configuration_file_to_string(Path::new(CONFIG_PATH))?;

    parse_gdsettings_file(&contents).map_err(Into::into)
}

/// Write gdpm configuration
///
/// # Arguments
///
/// * `settings` - Settings
///
pub fn write_gdpm_configuration(settings: GdSettings) -> Result<(), Error> {
    let contents = settings.to_string();

    println!("Writing gdpm configuration ...");
    write_string_to_configuration_file(Path::new(CONFIG_PATH), &contents)
}

/// Read project configuration
///
/// # Arguments
///
/// * `path` - Project path
///
pub fn read_project_configuration(path: &Path) -> Result<GdSettings, Error> {
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
///
/// # Arguments
///
/// * `path` - Project path
/// * `settings` - Settings
///
pub fn write_project_configuration(path: &Path, settings: GdSettings) -> Result<(), Error> {
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
    write_string_to_file(&project, &contents)
}