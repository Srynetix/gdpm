//! Info module

use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::Error;

use gdsettings_parser::{parse_gdsettings_file, GdSettings};

/// InfoError
#[derive(Debug, Fail)]
pub enum InfoError {
    /// Project not found
    #[fail(display = "project not found: {}", path)]
    ProjectNotFound { path: String },
    /// Malformed project
    #[fail(display = "malformed project")]
    MalformedProject,
}

/// Godot project info
#[derive(Debug)]
pub struct GdProjectInfo {
    project_name: String,
    config_version: i32,
    version: Option<String>,
    main_scene: Option<String>,
}

impl GdProjectInfo {
    /// Extract project info from settings
    pub fn from_settings(settings: &GdSettings) -> Result<Self, Error> {
        let project_name = settings
            .get_property("application", "config/name")
            .and_then(|x| x.to_str())
            .ok_or(InfoError::MalformedProject)?;
        let config_version = settings
            .get_property("", "config_version")
            .and_then(|x| x.to_i32())
            .ok_or(InfoError::MalformedProject)?;
        let version = settings
            .get_property("application", "config/version")
            .and_then(|x| x.to_str());
        let main_scene = settings
            .get_property("application", "run/main_scene")
            .and_then(|x| x.to_str());

        Ok(Self {
            project_name,
            config_version,
            version,
            main_scene,
        })
    }

    /// Show project info
    pub fn show(&self) {
        println!("Project: {}", self.project_name);
        if let Some(v) = &self.version {
            println!("  - Version: {}", v);
        }

        if let Some(s) = &self.main_scene {
            println!("  - Main scene: {}", s);
        }
    }
}

/// Get project info.
///
/// Read the project.godot file from a Godot project
///
/// # Arguments
///
/// * `path` - Project path
///
pub fn get_project_info(path: &Path) -> Result<GdProjectInfo, Error> {
    // Check for project.godot
    let project = path.join("project.godot");
    if !project.exists() {
        bail!(InfoError::ProjectNotFound {
            path: project.to_string_lossy().to_string()
        });
    }

    // Open file
    let mut contents = String::new();
    let mut file = File::open(project)?;
    file.read_to_string(&mut contents)?;

    parse_gdsettings_file(&contents).and_then(|data| GdProjectInfo::from_settings(&data))
}
