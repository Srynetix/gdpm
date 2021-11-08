//! Engine module.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use colored::Colorize;
use gdpm_downloader::version::{GodotVersion, SystemVersion};
use gdpm_io::{
    copy_dir, copy_file, error::IoError, open_and_extract_zip, remove_dir_all, remove_file,
    write_bytes_to_file,
};
use gdsettings_parser::{GdSettings, GdValue};
use slugify::slugify;
use tracing::{debug, info};

use crate::{
    config::{GlobalConfig, UserDir, ENGINES_SECTION},
    error::EngineError,
};

const ENGINE_DIR: &str = "engines";
const GODOT_EXECUTABLE_NAME: &str = "godot";

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
    ) -> Result<Self, EngineError> {
        if !path.is_file() {
            Err(EngineError::EngineNotFound(
                path.to_string_lossy().to_string(),
            ))
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
        GdValue::Object(vec![
            (
                "path".into(),
                GdValue::String(self.path.to_string_lossy().to_string()),
            ),
            ("version".into(), GdValue::String(self.version.clone())),
            ("has_mono".into(), GdValue::Boolean(self.has_mono)),
            ("from_source".into(), GdValue::Boolean(self.from_source)),
        ])
    }

    /// From gdvalue.
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
        self.version.color("green").to_string()
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
            "{} ({}) [mono: {} - built from source: {}]",
            self.version.color("green"),
            self.path.to_string_lossy().color("yellow"),
            mono_str,
            source_str
        )
    }
}

/// Engine handler.
pub struct EngineHandler;

impl EngineHandler {
    /// List engines info.
    pub fn list() -> Result<Vec<EngineInfo>, EngineError> {
        let config = GlobalConfig::load()?;
        Ok(EngineInfo::from_settings(config))
    }

    /// Update multiple engines info.
    pub fn update_all(entries: Vec<EngineInfo>) -> Result<(), EngineError> {
        let mut configuration = GlobalConfig::load()?;
        for entry in entries {
            configuration.set_property(ENGINES_SECTION, &entry.get_slug(), entry.to_gdvalue())
        }

        GlobalConfig::save(configuration).map_err(Into::into)
    }

    /// Register engine entry.
    pub fn register(entry: EngineInfo) -> Result<(), EngineError> {
        let mut engine_list = Self::list()?;
        let version = entry.version.clone();
        if let Some(other_entry) = engine_list
            .iter_mut()
            .find(|x| x.get_slug() == entry.get_slug())
        {
            other_entry.clone(entry);
        } else {
            engine_list.push(entry);
        }

        debug!(
            "Registering entry for version '{}' ...",
            version.color("green")
        );
        Self::update_all(engine_list)?;

        // Check if default engine is not defined
        if Self::get_default()?.is_none() {
            Self::set_as_default(&version)?;
        }

        Ok(())
    }

    /// Unregister engine entry.
    pub fn unregister(version: &str) -> Result<(), EngineError> {
        // Check if engine exists
        Self::get_version(version)?;
        // Check for default engine
        let default_engine = Self::get_default()?;

        // Unset default if same version
        if let Some(e) = default_engine {
            if slugify!(&e) == slugify!(version) {
                Self::unset_default()?;
            }
        }

        // Remove version
        debug!("Unregistering entry {} ...", version.color("green"));
        let mut conf = GlobalConfig::load()?;
        conf.remove_property(ENGINES_SECTION, &slugify!(version))?;
        GlobalConfig::save(conf).map_err(Into::into)
    }

    /// Get engine version.
    pub fn get_version(version: &str) -> Result<EngineInfo, EngineError> {
        let engine_list = Self::list()?;
        if let Some(entry) = engine_list.iter().find(|x| x.version == version) {
            Ok(entry.clone())
        } else {
            Err(EngineError::EngineNotFound(version.to_string()))
        }
    }

    /// Has version.
    pub fn has_version(version: &str) -> Result<Option<EngineInfo>, EngineError> {
        match Self::get_version(version) {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e {
                EngineError::EngineNotFound(_) => Ok(None),
                e => Err(e),
            },
        }
    }

    /// Run engine version for project.
    pub fn run_version_for_project(version: &str, path: &Path) -> Result<(), EngineError> {
        let engine = Self::get_version(version)?;
        Command::new(engine.path)
            .arg("--path")
            .arg(path)
            .arg("-e")
            .status()
            .map_err(IoError::CommandExecutionError)?;

        Ok(())
    }

    /// Execute engine version command for project.
    pub fn exec_version_for_project(
        version: &str,
        args: &[String],
        path: &Path,
    ) -> Result<(), EngineError> {
        let engine = Self::get_version(version)?;
        Command::new(engine.path)
            .arg("--path")
            .arg(path)
            .args(args)
            .status()
            .map_err(IoError::CommandExecutionError)?;

        Ok(())
    }

    /// Set engine as default.
    pub fn set_as_default(version: &str) -> Result<(), EngineError> {
        // Assert the engine exists
        Self::get_version(version)?;

        debug!(
            "Setting version '{}' as default ...",
            version.color("green")
        );
        let mut configuration = GlobalConfig::load()?;
        configuration.set_property("", "default_engine", GdValue::String(version.into()));
        GlobalConfig::save(configuration).map_err(Into::into)
    }

    /// Unset default engine.
    pub fn unset_default() -> Result<(), EngineError> {
        let mut configuration = GlobalConfig::load()?;
        configuration.remove_property("", "default_engine")?;
        GlobalConfig::save(configuration).map_err(Into::into)
    }

    /// Get default engine.
    pub fn get_default() -> Result<Option<String>, EngineError> {
        let default_engine = GlobalConfig::load().map(|x| {
            x.get_property("", "default_engine")
                .and_then(|x| x.to_str())
        })?;
        if let Some(e) = &default_engine {
            // Assert the version exist
            Self::get_version(e)?;
        }

        Ok(default_engine)
    }

    /// Install engine version from official zip.
    pub fn install_from_official_zip(
        zip_data: Vec<u8>,
        version: GodotVersion,
        system: SystemVersion,
    ) -> Result<PathBuf, EngineError> {
        let engine_path = UserDir::get_or_create_directory(Path::new(ENGINE_DIR))?;
        let version_name = format!("{}", version);
        let version_path = UserDir::get_or_create_directory(&engine_path.join(&version_name))?;
        let zip_path = version_path.join("download").with_extension("zip");
        write_bytes_to_file(&zip_path, &zip_data)?;

        // Unzip
        open_and_extract_zip(&zip_path, &version_path)?;

        // Folder name
        let zip_folder_name = format!(
            "Godot_v{}-{}_{}",
            version.version(),
            version.kind(),
            system.get_archive_basename(version.mono())
        );
        let zip_folder_path = version_path.join(&zip_folder_name);
        let zip_exec_name = format!("{}.{}", &zip_folder_name, system.get_extension());
        let zip_exec_path = version_path.join(&zip_folder_name).join(&zip_exec_name);
        let zip_exec_target = Path::new(&version_path)
            .join(&GODOT_EXECUTABLE_NAME)
            .with_extension(system.get_extension());

        // Copy to current dir
        copy_file(&zip_exec_path, &zip_exec_target)?;
        if version.mono() {
            let mono_folder_src = zip_folder_path.join("GodotSharp");
            let mono_folder_dst = &version_path;
            copy_dir(&mono_folder_src, mono_folder_dst)?;
        }

        // Cleaning
        remove_dir_all(&zip_folder_path)?;
        remove_file(&zip_path)?;

        // Register
        Self::register(EngineInfo::new(
            version_name,
            zip_exec_target.clone(),
            version.mono(),
            false,
        )?)?;

        Ok(zip_exec_target)
    }

    /// Uninstall version.
    pub fn uninstall(version: GodotVersion) -> Result<(), EngineError> {
        let engine_path = UserDir::get_or_create_directory(Path::new(ENGINE_DIR))?;
        let version_name = format!("{}", version);
        let version_path = engine_path.join(&version_name);
        Self::get_version(&version_name)?;

        if version_path.exists() {
            Self::unregister(&version_name)?;

            info!(
                "Removing {} ...",
                version_path.display().to_string().color("green")
            );
            remove_dir_all(&version_path)?;
        } else {
            return Err(EngineError::EngineNotInstalled(version_name));
        }

        Ok(())
    }
}
