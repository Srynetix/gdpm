//! Errors.

use std::path::PathBuf;

use gdpm_types::version::GodotVersion;
use gdsettings_parser::{GdSettingsError, ParserError};

/// Config error
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum ConfigError {
    #[error("Incomplete settings.")]
    IncompleteSettings(#[from] GdSettingsError),
    #[error("Malformed settings.")]
    MalformedSettings(#[from] ParserError),
    #[error(transparent)]
    IoError(#[from] gdpm_io::Error),
}

/// Engine error
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum EngineError {
    #[error("Engine version '{0}' is not found.")]
    EngineNotFound(GodotVersion),
    #[error("Engine version '{0}' is missing from path '{1}'.")]
    EngineMissingFromPath(GodotVersion, PathBuf),
    #[error("Engine version '{0}' is not installed.")]
    EngineNotInstalled(GodotVersion),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    IoError(#[from] gdpm_io::Error),
    #[error(transparent)]
    VersionError(#[from] gdpm_types::version::Error),
}

/// Project error
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum ProjectError {
    #[error("Project not found at path '{0}'.")]
    ProjectNotFound(String),
    #[error("Malformed project.")]
    MalformedProject(#[source] ParserError),
    #[error("Missing project property '{0}'.")]
    MissingProperty(String),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    IoError(#[from] gdpm_io::Error),
    #[error(transparent)]
    VersionError(#[from] gdpm_types::version::Error),
}

/// Plugin error
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum PluginError {
    #[error("Missing property '{0}'.")]
    MissingProperty(String),
    #[error("Malformed dependency '{0}'.")]
    MalformedDependency(String),
    #[error("Missing dependency '{0}'.")]
    MissingDependency(String),
    #[error("Cannot desync dependency '{0}'.")]
    CannotDesync(String),
    #[error("Plugin '{0}' already installed.")]
    AlreadyInstalled(String),
    #[error(transparent)]
    ProjectError(#[from] ProjectError),
    #[error(transparent)]
    IoError(#[from] gdpm_io::Error),
}

impl From<GdSettingsError> for EngineError {
    fn from(e: GdSettingsError) -> Self {
        EngineError::ConfigError(ConfigError::IncompleteSettings(e))
    }
}

impl From<GdSettingsError> for ProjectError {
    fn from(e: GdSettingsError) -> Self {
        ProjectError::ConfigError(ConfigError::IncompleteSettings(e))
    }
}
