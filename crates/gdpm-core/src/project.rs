//! Project module

use std::path::Path;

use colored::Colorize;
use gdsettings_parser::{GdSettings, GdValue};

use crate::config::{read_project_configuration, write_project_configuration, ConfigError};

/// Godot project info
#[derive(Debug)]
pub struct GdProjectInfo {
    project_name: String,
    version: Option<String>,
    main_scene: Option<String>,
    engine_version: Option<String>,
}

impl GdProjectInfo {
    /// Extract project info from settings
    pub fn from_settings(settings: &GdSettings) -> Result<Self, ConfigError> {
        let project_name = settings
            .get_property("application", "config/name")
            .and_then(|x| x.to_str())
            .ok_or(ConfigError::MalformedProject)?;
        let version = settings
            .get_property("application", "config/version")
            .and_then(|x| x.to_str());
        let main_scene = settings
            .get_property("application", "run/main_scene")
            .and_then(|x| x.to_str());
        let engine_version = settings
            .get_property("engine", "version")
            .and_then(|x| x.to_str());

        Ok(Self {
            project_name,
            version,
            main_scene,
            engine_version,
        })
    }

    /// Get versioned name
    pub fn get_versioned_name(&self) -> String {
        if let Some(v) = &self.version {
            format!("{} (v{})", self.project_name, v)
        } else {
            self.project_name.clone()
        }
    }

    /// Get engine version
    pub fn get_engine_version(&self) -> Option<&str> {
        self.engine_version.as_deref()
    }

    /// Show project info
    pub fn show(&self) {
        println!("Project: {}", self.project_name.color("green"));
        if let Some(v) = &self.version {
            println!("- Version: {}", v.color("green"));
        }

        if let Some(v) = &self.engine_version {
            println!("- Engine version: v{}", v.color("green"));
        }

        if let Some(s) = &self.main_scene {
            println!("- Main scene: {}", s.color("green"));
        }
    }
}

/// Get project info.
///
/// Read the project.godot file from a Godot project.
pub fn get_project_info(path: &Path) -> Result<GdProjectInfo, ConfigError> {
    // Get project configuration
    read_project_configuration(path).and_then(|data| GdProjectInfo::from_settings(&data))
}

/// Set project engine
pub fn set_project_engine(path: &Path, version: &str) -> Result<(), ConfigError> {
    let mut conf = read_project_configuration(path)?;
    conf.set_property("engine", "version", GdValue::String(version.into()));

    write_project_configuration(path, conf)?;
    Ok(())
}

/// Unset project engine
pub fn unset_project_engine(path: &Path) -> Result<(), ConfigError> {
    let mut conf = read_project_configuration(path)?;
    conf.remove_property("engine", "version")?;

    write_project_configuration(path, conf)?;
    Ok(())
}
