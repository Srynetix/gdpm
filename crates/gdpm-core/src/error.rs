//! Errors.

use gdpm_io::error::IoError;
use gdsettings_parser::{GdSettingsError, ParserError};
use thiserror::Error;

/// Config error
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ConfigError {
    #[error("Incomplete settings.")]
    IncompleteSettings(#[from] GdSettingsError),
    #[error("Malformed settings.")]
    MalformedSettings(#[from] ParserError),
    #[error(transparent)]
    IoError(#[from] IoError),
}

/// Engine error
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum EngineError {
    #[error("Engine version '{0}' is not found.")]
    EngineNotFound(String),
    #[error("Engine version '{0}' is not installed.")]
    EngineNotInstalled(String),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    IoError(#[from] IoError),
}

/// Project error
#[derive(Debug, Error)]
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
    IoError(#[from] IoError),
}

/// Plugin error
#[derive(Debug, Error)]
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
    IoError(#[from] IoError),
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
