//! User .gdpm config module.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use gdpm_io::{
    create_dir, error::IoError, get_user_configuration_directory, read_file_to_string,
    write_string_to_file,
};
use gdsettings_parser::{parse_gdsettings_file, GdSettings};

use crate::error::{ConfigError, ProjectError};

/// Root config folder name.
pub const ROOT_CONFIG_FOLDER_NAME: &str = "gdpm";
/// Global config filename.
pub const GLOBAL_CONFIG_FILENAME: &str = "gdpm.cfg";
/// Engines section name.
pub const ENGINES_SECTION: &str = "engines";
/// Project config filename.
pub const PROJECT_CONFIG_FILENAME: &str = "project.godot";

/// User directory handler.
pub struct UserDir;

impl UserDir {
    /// Get or create global directory.
    pub fn get_or_create_global_directory() -> Result<PathBuf, IoError> {
        let config_directory = get_user_configuration_directory()?.join(ROOT_CONFIG_FOLDER_NAME);
        if !config_directory.exists() {
            create_dir(&config_directory)?;
        }

        Ok(config_directory)
    }

    /// Get or create directory in global directory.
    pub fn get_or_create_directory(path: &Path) -> Result<PathBuf, IoError> {
        let path = Self::get_or_create_global_directory()?.join(path);
        if !path.exists() {
            create_dir(&path)?;
        }

        Ok(path)
    }

    /// Get file in global directory.
    pub fn get_file(path: &Path) -> Result<PathBuf, IoError> {
        Ok(Self::get_or_create_global_directory()?.join(path))
    }

    /// Create file in global directory.
    pub fn create_file(path: &Path) -> Result<File, IoError> {
        File::create(Self::get_file(path)?).map_err(|e| IoError::CreateFileError(path.into(), e))
    }

    /// Read file to string from global directory.
    pub fn read_file_to_string(path: &Path) -> Result<String, IoError> {
        read_file_to_string(&Self::get_file(path)?)
    }

    /// Write string tp file in global directory.
    pub fn write_string_to_file(path: &Path, contents: &str) -> Result<(), IoError> {
        write_string_to_file(&Self::get_file(path)?, contents).map(|_| ())
    }
}

/// Global configuration handler.
pub struct GlobalConfig;

impl GlobalConfig {
    /// Get global configuration path.
    pub fn get_global_config_path() -> &'static Path {
        Path::new(GLOBAL_CONFIG_FILENAME)
    }

    /// Load global configuration.
    pub fn load() -> Result<GdSettings, ConfigError> {
        let contents = UserDir::read_file_to_string(Self::get_global_config_path())?;
        parse_gdsettings_file(&contents).map_err(Into::into)
    }

    /// Save global configuration.
    pub fn save(settings: GdSettings) -> Result<(), ConfigError> {
        UserDir::write_string_to_file(Self::get_global_config_path(), &settings.to_string())
            .map_err(Into::into)
    }
}

/// Project configuration handler.
pub struct ProjectConfig;

impl ProjectConfig {
    /// Get project configuration path.
    pub fn get_project_config_path(path: &Path) -> PathBuf {
        path.join(PROJECT_CONFIG_FILENAME)
    }

    /// Ensure a project exists.
    pub fn ensure_project_exists(path: &Path) -> Result<PathBuf, ProjectError> {
        let project = Self::get_project_config_path(path);
        if !project.exists() {
            return Err(ProjectError::ProjectNotFound(
                project.to_string_lossy().to_string(),
            ));
        }

        Ok(project)
    }

    /// Load project configuration.
    pub fn load(path: &Path) -> Result<GdSettings, ProjectError> {
        let project = Self::ensure_project_exists(path)?;
        let contents = read_file_to_string(&project)?;
        parse_gdsettings_file(&contents).map_err(ProjectError::MalformedProject)
    }

    /// Save project configuration.
    pub fn save(path: &Path, settings: GdSettings) -> Result<(), ProjectError> {
        let project = Self::ensure_project_exists(path)?;
        write_string_to_file(&project, &settings.to_string())
            .map(|_| ())
            .map_err(Into::into)
    }
}
