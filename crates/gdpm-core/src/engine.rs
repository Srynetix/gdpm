//! Engine module.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use colored::Colorize;
use gdpm_io::{Error, IoAdapter};
use gdpm_types::version::{GodotVersion, SystemVersion};
use gdsettings_parser::{
    parse_gdsettings_file, GdSettings, GdSettingsMap, GdSettingsType, GdValue,
};
use slugify::slugify;
use tracing::{debug, info};

use crate::{
    config::{GlobalConfig, UserDir, ENGINES_SECTION},
    error::{ConfigError, EngineError},
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
}

impl EngineInfo {
    /// Create new engine info
    pub fn new<I: IoAdapter>(
        io_adapter: &I,
        version: String,
        path: PathBuf,
    ) -> Result<Self, EngineError> {
        if !io_adapter.path_is_file(&path) {
            Err(EngineError::EngineNotFound(
                path.to_string_lossy().to_string(),
            ))
        } else {
            Ok(Self { version, path })
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

            Some(Self { path, version })
        } else {
            None
        }
    }

    /// Check if version is version 4.
    pub fn is_version_4(&self) -> bool {
        self.version.starts_with("4.")
    }

    /// Show
    pub fn get_name(&self) -> String {
        self.version.color("green").to_string()
    }

    /// Show verbose
    pub fn get_verbose_name(&self) -> String {
        format!(
            "{} ({})",
            self.version.color("green"),
            self.path.to_string_lossy().color("yellow"),
        )
    }
}

/// Engine handler.
pub struct EngineHandler<'a, I: IoAdapter> {
    io_adapter: &'a I,
}

impl<'a, I: IoAdapter> EngineHandler<'a, I> {
    /// Creates a new engine handler.
    pub fn new(io_adapter: &'a I) -> Self {
        Self { io_adapter }
    }

    /// List engines info.
    pub fn list(&self) -> Result<Vec<EngineInfo>, EngineError> {
        let gconf = GlobalConfig::new(self.io_adapter);
        let config = gconf.load()?;
        Ok(EngineInfo::from_settings(config))
    }

    /// Update multiple engines info.
    pub fn update_all(&self, entries: Vec<EngineInfo>) -> Result<(), EngineError> {
        let gconf = GlobalConfig::new(self.io_adapter);
        let mut configuration = gconf.load()?;
        for entry in entries {
            configuration.set_property(ENGINES_SECTION, &entry.get_slug(), entry.to_gdvalue())
        }

        gconf.save(configuration).map_err(Into::into)
    }

    /// Register engine entry.
    pub fn register(&self, entry: EngineInfo) -> Result<(), EngineError> {
        let mut engine_list = self.list()?;
        let version = entry.version.clone();
        if let Some(other_entry) = engine_list
            .iter_mut()
            .find(|x| x.get_slug() == entry.get_slug())
        {
            *other_entry = entry;
        } else {
            engine_list.push(entry);
        }

        debug!(
            "Registering entry for version '{}' ...",
            version.color("green")
        );
        self.update_all(engine_list)?;

        // Check if default engine is not defined
        if self.get_default()?.is_none() {
            self.set_as_default(&version)?;
        }

        Ok(())
    }

    /// Unregister engine entry.
    pub fn unregister(&self, version: &str) -> Result<(), EngineError> {
        // Check if engine exists
        self.get_version(version)?;
        // Check for default engine
        let default_engine = self.get_default()?;

        // Unset default if same version
        if let Some(e) = default_engine {
            if slugify!(&e) == slugify!(version) {
                self.unset_default()?;
            }
        }

        // Remove version
        debug!("Unregistering entry {} ...", version.color("green"));
        let gconf = GlobalConfig::new(self.io_adapter);
        let mut conf = gconf.load()?;
        conf.remove_property(ENGINES_SECTION, &slugify!(version))?;
        gconf.save(conf).map_err(Into::into)
    }

    /// Get engine version.
    pub fn get_version(&self, version: &str) -> Result<EngineInfo, EngineError> {
        let engine_list = self.list()?;
        if let Some(entry) = engine_list.iter().find(|x| x.version == version) {
            Ok(entry.clone())
        } else {
            Err(EngineError::EngineNotFound(version.to_string()))
        }
    }

    /// Has version.
    pub fn has_version(&self, version: &str) -> Result<Option<EngineInfo>, EngineError> {
        match self.get_version(version) {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e {
                EngineError::EngineNotFound(_) => Ok(None),
                e => Err(e),
            },
        }
    }

    /// Run engine version for project.
    pub fn run_version_for_project(&self, version: &str, path: &Path) -> Result<(), EngineError> {
        let engine = self.get_version(version)?;
        Command::new(engine.path)
            .arg("--path")
            .arg(path)
            .arg("-e")
            .status()
            .map_err(|e| Error::CommandExecutionError(e.to_string()))?;

        Ok(())
    }

    /// Run engine version for project, no editor.
    pub fn run_version_for_project_no_editor(
        &self,
        version: &str,
        path: &Path,
    ) -> Result<(), EngineError> {
        let engine = self.get_version(version)?;
        Command::new(engine.path)
            .arg("--path")
            .arg(path)
            .status()
            .map_err(|e| Error::CommandExecutionError(e.to_string()))?;

        Ok(())
    }

    /// Execute engine version command for project.
    pub fn exec_version_for_project(
        &self,
        version: &str,
        args: &[String],
        path: &Path,
    ) -> Result<(), EngineError> {
        let engine = self.get_version(version)?;
        Command::new(engine.path)
            .arg("--path")
            .arg(path)
            .args(args)
            .status()
            .map_err(|e| Error::CommandExecutionError(e.to_string()))?;

        Ok(())
    }

    /// Set engine as default.
    pub fn set_as_default(&self, version: &str) -> Result<(), EngineError> {
        // Assert the engine exists
        self.get_version(version)?;

        debug!(
            "Setting version '{}' as default ...",
            version.color("green")
        );

        let gconf = GlobalConfig::new(self.io_adapter);
        let mut configuration = gconf.load()?;
        configuration.set_property("", "default_engine", GdValue::String(version.into()));
        gconf.save(configuration).map_err(Into::into)
    }

    /// Unset default engine.
    pub fn unset_default(&self) -> Result<(), EngineError> {
        let gconf = GlobalConfig::new(self.io_adapter);
        let mut configuration = gconf.load()?;
        configuration.remove_property("", "default_engine")?;
        gconf.save(configuration).map_err(Into::into)
    }

    /// Get default engine.
    pub fn get_default(&self) -> Result<Option<String>, EngineError> {
        let gconf = GlobalConfig::new(self.io_adapter);
        let default_engine = gconf.load().map(|x| {
            x.get_property("", "default_engine")
                .and_then(|x| x.to_str())
        })?;
        if let Some(e) = &default_engine {
            // Assert the version exist
            self.get_version(e)?;
        }

        Ok(default_engine)
    }

    /// Install engine version from official zip.
    pub fn install_from_official_zip(
        &self,
        zip_data: Vec<u8>,
        version: GodotVersion,
        system: SystemVersion,
    ) -> Result<PathBuf, EngineError> {
        let udir = UserDir::new(self.io_adapter);
        let engine_path = udir.get_or_create_directory(Path::new(ENGINE_DIR))?;
        let version_name = format!("{}", version);
        let temp_name = "temp";
        let version_path = udir.get_or_create_directory(&engine_path.join(&version_name))?;
        let extraction_path =
            udir.get_or_create_directory(&engine_path.join(&version_name).join(temp_name))?;
        let zip_path = version_path.join("download").with_extension("zip");
        self.io_adapter.write_bytes_to_file(&zip_path, &zip_data)?;

        // Unzip
        self.io_adapter
            .open_and_extract_zip(&zip_path, &extraction_path)?;

        // Folder name
        let zip_folder_name = format!(
            "Godot_v{}-{}_{}",
            version.version(),
            version.kind(),
            system.get_archive_basename(version.mono())
        );
        let zip_folder_path = extraction_path.join(&zip_folder_name);

        // Mono versions have an additional folder
        let zip_exec_path = if version.mono() {
            let zip_exec_name = format!("{}.{}", &zip_folder_name, system.get_extension());
            extraction_path.join(&zip_folder_name).join(zip_exec_name)
        } else {
            zip_folder_path.with_extension(system.get_extension())
        };

        let zip_exec_target = Path::new(&version_path)
            .join(GODOT_EXECUTABLE_NAME)
            .with_extension(system.get_extension());

        // Copy to current dir
        self.io_adapter
            .copy_file(&zip_exec_path, &zip_exec_target)?;
        if version.mono() {
            let mono_folder_src = zip_folder_path.join("GodotSharp");
            let mono_folder_dst = &version_path;
            self.io_adapter
                .copy_dir(&mono_folder_src, mono_folder_dst)?;
        }

        // Cleaning
        self.io_adapter.remove_dir_all(&extraction_path)?;
        self.io_adapter.remove_file(&zip_path)?;

        // Register
        self.register(EngineInfo::new(
            self.io_adapter,
            version_name,
            zip_exec_target.clone(),
        )?)?;

        Ok(zip_exec_target)
    }

    /// Uninstall version.
    pub fn uninstall(&self, version: GodotVersion) -> Result<(), EngineError> {
        let udir = UserDir::new(self.io_adapter);
        let engine_path = udir.get_or_create_directory(Path::new(ENGINE_DIR))?;
        let version_name = format!("{}", version);
        let version_path = engine_path.join(&version_name);
        self.get_version(&version_name)?;

        if self.io_adapter.path_exists(&version_path) {
            self.unregister(&version_name)?;

            info!(
                "Removing {} ...",
                version_path.display().to_string().color("green")
            );
            self.io_adapter.remove_dir_all(&version_path)?;
        } else {
            return Err(EngineError::EngineNotInstalled(version_name));
        }

        Ok(())
    }

    /// Cache versions.
    pub fn write_versions_in_cache(&self, versions: Vec<GodotVersion>) -> Result<(), EngineError> {
        let udir = UserDir::new(self.io_adapter);
        let path = udir.get_or_create_file(Path::new("remote-cache.cfg"))?;

        let mut data = GdSettingsMap::new();
        for version in versions {
            data.insert(version.to_string(), version.to_gdvalue());
        }
        let mut sections = GdSettingsType::new();
        sections.insert("remote".into(), data);
        let settings = GdSettings::new(sections);

        udir.write_string_to_file(&path, &settings.to_string())?;
        Ok(())
    }

    /// Read versions from cache.
    pub fn read_versions_from_cache(&self) -> Result<Vec<GodotVersion>, EngineError> {
        let udir = UserDir::new(self.io_adapter);
        let path = udir.get_or_create_file(Path::new("remote-cache.cfg"))?;
        let contents = udir.read_file_to_string(&path)?;
        let settings = parse_gdsettings_file(&contents)
            .map_err(|e| EngineError::ConfigError(ConfigError::MalformedSettings(e)))?;
        let mut versions = vec![];

        if let Some(section) = settings.get_section("remote") {
            let values = section
                .into_values()
                .filter_map(GodotVersion::from_gdvalue)
                .collect();
            versions = values;
        }

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    mod engineinfo {
        use gdsettings_parser::GdSettings;
        use mockall::predicate;
        use std::{
            path::{Path, PathBuf},
            str::FromStr,
        };

        use crate::engine::EngineInfo;
        use gdpm_io::MockIoAdapter;

        #[test]
        fn test_new() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_path_is_file()
                .with(predicate::eq(Path::new("/")))
                .times(1)
                .return_const(true);

            assert!(
                EngineInfo::new(&adapter, "1.0.0".into(), PathBuf::from("/")).is_ok(),
                "engine info retrieving should work if file exists"
            );

            adapter
                .expect_path_is_file()
                .with(predicate::eq(Path::new("/")))
                .times(1)
                .return_const(false);

            assert!(
                EngineInfo::new(&adapter, "1.0.0".into(), PathBuf::from("/")).is_err(),
                "engine info retrieving should NOT work if file does not exist"
            );
        }

        #[test]
        fn test_from_settings() {
            let settings = indoc::indoc! {r#"
                [engines]
                1-0-0 = { "path": "/hello", "version": "1.0.0" }
                2-0-0 = { "path": "/hi", "version": "2.0.0" }
            "#};

            let settings = GdSettings::from_str(settings).unwrap();
            let engine_list = EngineInfo::from_settings(settings);

            assert_eq!(
                engine_list,
                vec![
                    EngineInfo {
                        path: PathBuf::from("/hello"),
                        version: "1.0.0".to_string()
                    },
                    EngineInfo {
                        path: PathBuf::from("/hi"),
                        version: "2.0.0".to_string()
                    }
                ]
            )
        }
    }

    #[test]
    fn test_from_settings() {}
}
