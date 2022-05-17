//! User .gdpm config module.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use gdpm_io::{IoAdapter, IoError};
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
pub struct UserDir<'a, I: IoAdapter> {
    io_adapter: &'a I,
}

impl<'a, I: IoAdapter> UserDir<'a, I> {
    /// Creates a new UserDir.
    pub fn new(io_adapter: &'a I) -> Self {
        Self { io_adapter }
    }

    /// Get or create global directory.
    pub fn get_or_create_global_directory(&self) -> Result<PathBuf, IoError> {
        let config_directory = self
            .io_adapter
            .get_user_configuration_directory()?
            .join(ROOT_CONFIG_FOLDER_NAME);
        if !config_directory.exists() {
            self.io_adapter.create_dir(&config_directory)?;
        }

        Ok(config_directory)
    }

    /// Get or create directory in global directory.
    pub fn get_or_create_directory(&self, path: &Path) -> Result<PathBuf, IoError> {
        let path = self.get_or_create_global_directory()?.join(path);
        if !path.exists() {
            self.io_adapter.create_dir(&path)?;
        }

        Ok(path)
    }

    /// Get file in global directory.
    pub fn get_file(&self, path: &Path) -> Result<PathBuf, IoError> {
        Ok(self.get_or_create_global_directory()?.join(path))
    }

    /// Create file in global directory.
    pub fn create_file(&self, path: &Path) -> Result<File, IoError> {
        File::create(self.get_file(path)?).map_err(|e| IoError::CreateFileError(path.into(), e))
    }

    /// Read file to string from global directory.
    pub fn read_file_to_string(&self, path: &Path) -> Result<String, IoError> {
        self.io_adapter.read_file_to_string(&self.get_file(path)?)
    }

    /// Write string tp file in global directory.
    pub fn write_string_to_file(&self, path: &Path, contents: &str) -> Result<(), IoError> {
        self.io_adapter
            .write_string_to_file(&self.get_file(path)?, contents)
            .map(|_| ())
    }
}

/// Global configuration handler.
pub struct GlobalConfig<'a, I: IoAdapter> {
    io_adapter: &'a I,
}

impl<'a, I: IoAdapter> GlobalConfig<'a, I> {
    /// Creates a new global config.
    pub fn new(io_adapter: &'a I) -> Self {
        Self { io_adapter }
    }

    /// Get global configuration path.
    pub fn get_global_config_path(&self) -> &'static Path {
        Path::new(GLOBAL_CONFIG_FILENAME)
    }

    /// Load global configuration.
    pub fn load(&self) -> Result<GdSettings, ConfigError> {
        let udir = UserDir::new(self.io_adapter);
        let contents = udir.read_file_to_string(self.get_global_config_path())?;
        parse_gdsettings_file(&contents).map_err(Into::into)
    }

    /// Save global configuration.
    pub fn save(&self, settings: GdSettings) -> Result<(), ConfigError> {
        let udir = UserDir::new(self.io_adapter);
        udir.write_string_to_file(self.get_global_config_path(), &settings.to_string())
            .map_err(Into::into)
    }
}

/// Project configuration handler.
pub struct ProjectConfig<'a, I: IoAdapter> {
    io_adapter: &'a I,
}

impl<'a, I: IoAdapter> ProjectConfig<'a, I> {
    /// Creates a new project config.
    pub fn new(io_adapter: &'a I) -> Self {
        Self { io_adapter }
    }

    /// Get project configuration path.
    pub fn get_project_config_path(&self, path: &Path) -> PathBuf {
        path.join(PROJECT_CONFIG_FILENAME)
    }

    /// Ensure a project exists.
    pub fn ensure_project_exists(&self, path: &Path) -> Result<PathBuf, ProjectError> {
        let project = self.get_project_config_path(path);
        if !project.exists() {
            return Err(ProjectError::ProjectNotFound(
                project.to_string_lossy().to_string(),
            ));
        }

        Ok(project)
    }

    /// Load project configuration.
    pub fn load(&self, path: &Path) -> Result<GdSettings, ProjectError> {
        let project = self.ensure_project_exists(path)?;
        let contents = self.io_adapter.read_file_to_string(&project)?;
        parse_gdsettings_file(&contents).map_err(ProjectError::MalformedProject)
    }

    /// Save project configuration.
    pub fn save(&self, path: &Path, settings: GdSettings) -> Result<(), ProjectError> {
        let project = self.ensure_project_exists(path)?;
        self.io_adapter
            .write_string_to_file(&project, &settings.to_string())
            .map(|_| ())
            .map_err(Into::into)
    }
}
