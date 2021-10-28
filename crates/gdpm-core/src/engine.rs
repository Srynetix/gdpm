//! Engine module

use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

use colored::Colorize;
use gdsettings_parser::{GdSettings, GdValue};
use slugify::slugify;

use crate::config::{
    read_gdpm_configuration, write_gdpm_configuration, ConfigError, ENGINES_SECTION,
};

/// Engine info
#[derive(Debug, PartialEq, Clone)]
pub struct EngineInfo {
    /// Version of engine
    pub version: String,
    /// Path to engine
    pub path: PathBuf,
    /// Mono compatible?
    pub has_mono: bool,
    /// Built from source?
    pub from_source: bool,
}

impl EngineInfo {
    /// Create new engine info
    pub fn new(
        version: String,
        path: PathBuf,
        has_mono: bool,
        from_source: bool,
    ) -> Result<Self, ConfigError> {
        if !path.is_file() {
            Err(ConfigError::EngineNotFound {
                path: path.to_string_lossy().to_string(),
            })
        } else {
            Ok(Self {
                version,
                path,
                has_mono,
                from_source,
            })
        }
    }

    /// Extract engine info from settings
    ///
    /// # Arguments
    ///
    /// * `settings` - GdSettings
    ///
    pub fn from_settings(settings: GdSettings) -> Vec<Self> {
        let mut engines = vec![];
        let properties = settings.get_section(ENGINES_SECTION);
        if let Some(props) = properties {
            for (_, value) in props.into_iter() {
                if let Some(e) = EngineInfo::from_gdvalue(value) {
                    engines.push(e);
                }
            }
        }

        engines
    }

    /// Clone from other engine info
    ///
    /// # Arguments
    ///
    /// * `other` - Other engine info
    ///
    pub fn clone(&mut self, other: Self) {
        self.path = other.path;
        self.version = other.version;
        self.has_mono = other.has_mono;
        self.from_source = other.from_source;
    }

    /// Get engine info slug
    pub fn get_slug(&self) -> String {
        slugify!(&self.version)
    }

    /// Compare slug
    pub fn has_same_slug(&self, name: &str) -> bool {
        self.get_slug() == slugify!(name)
    }

    /// To GdValue
    pub fn to_gdvalue(&self) -> GdValue {
        let mut props: Vec<(String, GdValue)> = Vec::new();

        props.push((
            "path".to_string(),
            GdValue::String(self.path.to_string_lossy().to_string()),
        ));
        props.push(("version".to_string(), GdValue::String(self.version.clone())));
        props.push(("has_mono".to_string(), GdValue::Boolean(self.has_mono)));
        props.push((
            "from_source".to_string(),
            GdValue::Boolean(self.from_source),
        ));
        GdValue::Object(props)
    }

    /// From gdvalue.
    ///
    /// # Arguments
    ///
    /// * `value`- GdValue
    ///
    pub fn from_gdvalue(value: GdValue) -> Option<Self> {
        if let Some(map) = value.to_object() {
            let path = PathBuf::from(
                map.get("path")
                    .and_then(|x| x.to_str())
                    .unwrap_or_else(|| String::from("unknown")),
            );
            let version = map
                .get("version")
                .and_then(|x| x.to_str())
                .unwrap_or_else(|| String::from("unknown"));
            let has_mono = map
                .get("has_mono")
                .and_then(|x| x.to_bool())
                .unwrap_or(false);
            let from_source = map
                .get("from_source")
                .and_then(|x| x.to_bool())
                .unwrap_or(false);

            Some(Self {
                path,
                version,
                has_mono,
                from_source,
            })
        } else {
            None
        }
    }

    /// Show
    pub fn get_name(&self) -> String {
        format!("Godot Engine v{}", self.version.color("green"))
    }

    /// Show verbose
    pub fn get_verbose_name(&self) -> String {
        let mono_str = if self.has_mono {
            "Yes".color("green")
        } else {
            "No".color("red")
        };
        let source_str = if self.from_source {
            "Yes".color("green")
        } else {
            "No".color("red")
        };

        format!(
            "Godot Engine v{} ({}) [mono: {} - source: {}]",
            self.version.color("green"),
            self.path.to_string_lossy().color("yellow"),
            mono_str,
            source_str
        )
    }
}

/// List engines info.
pub fn list_engines_info() -> Result<Vec<EngineInfo>, ConfigError> {
    let config = read_gdpm_configuration()?;
    Ok(EngineInfo::from_settings(config))
}

/// Update engines info
///
/// # Arguments
///
/// * `entries` - Engine info entries
///
pub fn update_engines_info(entries: Vec<EngineInfo>) -> Result<File, ConfigError> {
    let mut configuration = read_gdpm_configuration()?;
    for entry in entries {
        configuration.set_property(ENGINES_SECTION, &entry.get_slug(), entry.to_gdvalue())
    }

    write_gdpm_configuration(configuration)
}

/// Register engine entry.
///
/// # Arguments
///
/// * `entry` - Engine info
///
pub fn register_engine_entry(entry: EngineInfo) -> Result<(), ConfigError> {
    let mut engine_list = list_engines_info()?;
    let version = entry.version.clone();
    if let Some(other_entry) = engine_list
        .iter_mut()
        .find(|x| x.get_slug() == entry.get_slug())
    {
        other_entry.clone(entry);
    } else {
        engine_list.push(entry);
    }

    update_engines_info(engine_list)?;

    // Check if default engine is not defined
    if get_default_engine()?.is_none() {
        set_default_engine(&version)?;
    }

    Ok(())
}

/// Unregister engine entry.
///
/// # Arguments
///
/// * `version` - Version
///
pub fn unregister_engine_entry(version: &str) -> Result<(), ConfigError> {
    // Check if engine exists
    get_engine_version(&version)?;
    // Check for default engine
    let default_engine = get_default_engine()?;

    // Unset default if same version
    if let Some(e) = default_engine {
        if slugify!(&e) == slugify!(&version) {
            unset_default_engine()?;
        }
    }

    // Remove version
    let mut conf = read_gdpm_configuration()?;
    conf.remove_property(ENGINES_SECTION, &slugify!(&version))?;
    write_gdpm_configuration(conf)?;

    Ok(())
}

/// Get engine version.
///
/// # Arguments
///
/// * `version` - Version
///
pub fn get_engine_version(version: &str) -> Result<EngineInfo, ConfigError> {
    let engine_list = list_engines_info()?;
    if let Some(entry) = engine_list.iter().find(|x| x.version == version) {
        Ok(entry.clone())
    } else {
        Err(ConfigError::EngineNotFound {
            path: version.to_string(),
        })?
    }
}

/// Run engine version for project.
///
/// # Arguments
///
/// * `version` - Version
/// * `path` - Path
///
pub fn run_engine_version_for_project(version: &str, path: &Path) -> Result<(), ConfigError> {
    let engine = get_engine_version(version)?;
    Command::new(engine.path)
        .arg("--path")
        .arg(path)
        .arg("-e")
        .status()?;

    Ok(())
}

/// Execute engine version command for project.
///
/// # Arguments
///
/// * `version` - Version
/// * `args` - Arguments
/// * `path` - Path
///
pub fn exec_engine_version_command_for_project(
    version: &str,
    args: &[String],
    path: &Path,
) -> Result<(), ConfigError> {
    let engine = get_engine_version(version)?;
    Command::new(engine.path)
        .arg("--path")
        .arg(path)
        .args(args)
        .status()?;

    Ok(())
}

/// Set engine as default
///
/// # Arguments
///
/// * `version` - Version
///
pub fn set_default_engine(version: &str) -> Result<(), ConfigError> {
    // Assert the engine exists
    get_engine_version(version)?;

    let mut configuration = read_gdpm_configuration()?;
    configuration.set_property("", "default_engine", GdValue::String(version.into()));

    write_gdpm_configuration(configuration)?;
    Ok(())
}

/// Unset default engine
pub fn unset_default_engine() -> Result<(), ConfigError> {
    let mut configuration = read_gdpm_configuration()?;
    configuration.remove_property("", "default_engine")?;
    write_gdpm_configuration(configuration)?;
    Ok(())
}

/// Get default engine
pub fn get_default_engine() -> Result<Option<String>, ConfigError> {
    let default_engine = read_gdpm_configuration().and_then(|x| {
        Ok(x.get_property("", "default_engine")
            .and_then(|x| x.to_str()))
    })?;
    if let Some(e) = &default_engine {
        // Assert the version exist
        get_engine_version(&e)?;
    }

    Ok(default_engine)
}
