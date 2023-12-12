//! Plugins module.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use colored::Colorize;
use gdpm_io::{Error, IoAdapter};
use gdsettings_parser::{parse_gdsettings_file, GdValue};
use slugify::slugify;
use tracing::{info, warn};

use super::{config::ProjectConfig, project::ProjectHandler};
use crate::error::{PluginError, ProjectError};

const DEPS_SECTION: &str = "dependencies";
const ADDONS_FOLDER: &str = "addons";
const PLUGIN_CFG: &str = "plugin.cfg";

/// Dependency source
#[derive(Debug, PartialEq)]
pub enum DependencySource {
    /// Git HTTP/HTTPS
    GitHttp(String),
    /// Git SSH
    GitSsh(String),
    /// Path
    Path(PathBuf),
    /// Current
    Current,
}

impl DependencySource {
    /// Create from string
    pub fn from_value(source: &str) -> Self {
        if source == "." {
            Self::Current
        } else if source.starts_with("http") {
            Self::GitHttp(source.to_string())
        } else if source.starts_with("git@") {
            Self::GitSsh(source.to_string())
        } else {
            Self::Path(Path::new(source).to_path_buf())
        }
    }

    /// Get path
    pub fn path(&self) -> String {
        match &self {
            Self::Current => ".".to_string(),
            Self::GitHttp(x) => x.to_string(),
            Self::GitSsh(x) => x.to_string(),
            Self::Path(x) => x.to_string_lossy().to_string(),
        }
    }
}

impl std::string::ToString for DependencySource {
    fn to_string(&self) -> String {
        match &self {
            Self::Current => "Current".to_string(),
            Self::GitHttp(x) => format!("Git (HTTP): {}", x),
            Self::GitSsh(x) => format!("Git (SSH): {}", x),
            Self::Path(x) => x.to_string_lossy().to_string(),
        }
    }
}

/// Plugin info
#[derive(Debug)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// Version
    pub version: String,
    /// Script
    pub script: String,
    /// Folder name
    pub folder_name: String,
}

/// Dependency
#[derive(Debug)]
pub struct Dependency {
    /// Name
    pub name: String,
    /// Version
    pub version: String,
    /// Checksum
    pub checksum: String,
    /// Source
    pub source: DependencySource,
}

impl Dependency {
    /// From GdValue
    pub fn from_gdvalue(name: &str, value: &GdValue) -> Result<Self, PluginError> {
        let value = value
            .to_object()
            .ok_or_else(|| PluginError::MalformedDependency(name.to_string()))?;

        let name = value
            .get("name")
            .and_then(|x| x.to_str())
            .ok_or_else(|| PluginError::MalformedDependency(name.to_string()))?;
        let version = value
            .get("version")
            .and_then(|x| x.to_str())
            .ok_or_else(|| PluginError::MalformedDependency(name.to_string()))?;
        let source = value
            .get("source")
            .and_then(|x| x.to_str())
            .ok_or_else(|| PluginError::MalformedDependency(name.to_string()))?;

        Ok(Dependency {
            name,
            checksum: "".to_string(),
            source: DependencySource::from_value(&source),
            version,
        })
    }

    /// Create from PluginInfo
    pub fn from_plugin_info(plugin: &PluginInfo) -> Self {
        Self {
            name: plugin.folder_name.clone(),
            version: plugin.version.clone(),
            checksum: "".to_string(),
            source: DependencySource::Current,
        }
    }

    /// To GdValue
    pub fn to_gdvalue(&self) -> GdValue {
        GdValue::Object(vec![
            ("name".into(), GdValue::String(self.name.clone())),
            ("version".into(), GdValue::String(self.version.clone())),
            ("source".into(), GdValue::String(self.source.path())),
        ])
    }

    /// Get verbose name
    pub fn get_verbose_name(&self) -> String {
        format!(
            "{} (v{}) (source: {})",
            self.name.color("green"),
            self.version.color("green"),
            self.source.to_string().color("blue")
        )
    }
}

impl PluginInfo {
    /// Load plugin info from project addon
    pub fn from_project_addon<I: IoAdapter>(
        io_adapter: &I,
        project_path: &Path,
        addon_folder: &str,
    ) -> Result<Self, PluginError> {
        let addon_path = project_path.join(PLUGIN_CFG);
        if let Ok(cfg_contents) = io_adapter.read_file_to_string(&addon_path) {
            let addon_cfg =
                parse_gdsettings_file(&cfg_contents).map_err(ProjectError::MalformedProject)?;

            let name = addon_cfg
                .get_property("plugin", "name")
                .and_then(|x| x.to_str())
                .ok_or_else(|| PluginError::MissingProperty("name".to_string()))?;
            let description = addon_cfg
                .get_property("plugin", "description")
                .and_then(|x| x.to_str())
                .ok_or_else(|| PluginError::MissingProperty("description".to_string()))?;
            let author = addon_cfg
                .get_property("plugin", "author")
                .and_then(|x| x.to_str())
                .ok_or_else(|| PluginError::MissingProperty("author".to_string()))?;
            let version = addon_cfg
                .get_property("plugin", "version")
                .and_then(|x| x.to_str())
                .ok_or_else(|| PluginError::MissingProperty("version".to_string()))?;
            let script = addon_cfg
                .get_property("plugin", "script")
                .and_then(|x| x.to_str())
                .ok_or_else(|| PluginError::MissingProperty("script".to_string()))?;

            Ok(Self {
                name,
                description,
                author,
                version,
                script,
                folder_name: addon_folder.to_string(),
            })
        } else {
            warn!(
                "No 'plugin.cfg' found in addon '{}'.",
                addon_folder.color("green")
            );

            Ok(Self {
                name: addon_folder.into(),
                description: String::new(),
                author: String::new(),
                version: String::new(),
                script: String::new(),
                folder_name: addon_folder.into(),
            })
        }
    }

    /// Show
    pub fn show(&self) {
        println!(
            "- {} (v{}) ({})",
            self.name.color("green"),
            self.version.color("green"),
            self.script.color("yellow")
        );
        println!(
            "  from {}: {}",
            self.author.color("green"),
            self.description.color("yellow")
        );
        println!("  in folder {}", self.folder_name.color("blue"));
    }
}

/// Dependency handler.
pub struct DependencyHandler<'a, I: IoAdapter> {
    io_adapter: &'a I,
}

impl<'a, I: IoAdapter> DependencyHandler<'a, I> {
    /// Creates a new dependency handler.
    pub fn new(io_adapter: &'a I) -> Self {
        Self { io_adapter }
    }

    /// Check if the dependency is installed
    pub fn is_installed(&self, dependency: &Dependency, project_path: &Path) -> bool {
        let path = project_path.join(ADDONS_FOLDER).join(&dependency.name);
        self.io_adapter.path_exists(&path)
    }

    /// Uninstall dependency
    pub fn uninstall(
        &self,
        dependency: &Dependency,
        project_path: &Path,
    ) -> Result<(), PluginError> {
        if self.is_installed(dependency, project_path) {
            self.io_adapter
                .remove_dir_all(&project_path.join(ADDONS_FOLDER).join(&dependency.name))?;
        }

        Ok(())
    }

    /// Install dependency
    pub fn install(
        &self,
        dependency: &Dependency,
        project_path: &Path,
    ) -> Result<PluginInfo, PluginError> {
        let addon_path = project_path.join(ADDONS_FOLDER).join(&dependency.name);
        if self.io_adapter.path_exists(&addon_path) {
            return Err(PluginError::AlreadyInstalled(dependency.name.clone()));
        }

        match &dependency.source {
            DependencySource::Current => {
                // Current project
                PluginInfo::from_project_addon(self.io_adapter, project_path, &dependency.name)
            }
            DependencySource::Path(p) => {
                // Another project
                let full_path = if p.is_relative() {
                    project_path.join(p)
                } else {
                    p.to_path_buf()
                };

                let project_addons = project_path.join(ADDONS_FOLDER);
                if !self.io_adapter.path_exists(&project_addons) {
                    self.io_adapter.create_dir(&project_addons)?;
                }

                // Copy folder to project
                self.io_adapter.copy_dir(&full_path, &project_addons)?;

                PluginInfo::from_project_addon(self.io_adapter, &full_path, &dependency.name)
            }
            DependencySource::GitSsh(p) => {
                // Clone in the project .gdpm folder
                let gdpm_path = project_path.join(".gdpm");
                if !self.io_adapter.path_exists(&gdpm_path) {
                    self.io_adapter.create_dir(&gdpm_path)?;
                }
                let plugin_path = gdpm_path.join(&dependency.name);
                if !self.io_adapter.path_exists(&plugin_path) {
                    info!(
                        "Cloning plugin in '{}' from repository '{}' ...",
                        plugin_path.display(),
                        p
                    );
                    Command::new("git")
                        .arg("clone")
                        .arg(p)
                        .arg(&dependency.name)
                        .current_dir(&gdpm_path)
                        .output()
                        .map_err(|e| Error::CommandExecutionError(e.to_string()))?;
                }

                let project_addons = project_path.join(ADDONS_FOLDER);
                if !self.io_adapter.path_exists(&project_addons) {
                    self.io_adapter.create_dir(&project_addons)?;
                }

                // Copy folder to project
                self.io_adapter.copy_dir(&plugin_path, &project_addons)?;

                // Remove .gdpm
                self.io_adapter.remove_dir_all(&gdpm_path)?;

                PluginInfo::from_project_addon(self.io_adapter, &plugin_path, &dependency.name)
            }
            _ => unimplemented!(),
        }
    }

    /// List project dependencies
    pub fn list_project_dependencies(&self, path: &Path) -> Result<Vec<Dependency>, PluginError> {
        let pconf = ProjectConfig::new(self.io_adapter);
        let conf = pconf.load(path)?;
        let mut deps = vec![];
        if let Some(dependencies) = conf.get_section(DEPS_SECTION) {
            for (name, value) in dependencies {
                deps.push(Dependency::from_gdvalue(&name, &value)?);
            }
        }

        Ok(deps)
    }

    /// Get dependency
    pub fn get_dependency(
        &self,
        project_path: &Path,
        name: &str,
    ) -> Result<Dependency, PluginError> {
        let deps = self.list_project_dependencies(project_path)?;
        let slug = slugify!(name);
        for dep in deps {
            let dep_slug = slugify!(&dep.name);
            if dep_slug == slug {
                return Ok(dep);
            }
        }

        Err(PluginError::MissingDependency(name.to_string()))
    }

    /// List plugins from project
    pub fn list_plugins_from_project(
        &self,
        project_path: &Path,
    ) -> Result<Vec<PluginInfo>, PluginError> {
        let addons_path = project_path.join(ADDONS_FOLDER);
        let mut addons = vec![];

        if self.io_adapter.path_exists(&addons_path) {
            for entry in self.io_adapter.read_dir(&addons_path)? {
                let entry = entry
                    .map_err(|e| Error::ReadDirEntryError(addons_path.to_owned(), e.to_string()))?;
                let path = entry.path();

                // Check for plugin.cfg
                let plugin_path = path.join(PLUGIN_CFG);
                if !self.io_adapter.path_exists(&plugin_path) {
                    // Ignore plugin
                    continue;
                }

                // Read configuration
                addons.push(PluginInfo::from_project_addon(
                    self.io_adapter,
                    project_path,
                    entry.file_name().to_str().unwrap(),
                )?);
            }
        }

        Ok(addons)
    }

    /// Add dependency to project
    pub fn add_dependency(
        &self,
        project_path: &Path,
        name: &str,
        version: &str,
        source: &str,
        no_install: bool,
    ) -> Result<(), PluginError> {
        let dependency = Dependency {
            name: name.to_string(),
            checksum: "".to_string(),
            version: version.to_string(),
            source: DependencySource::from_value(source),
        };

        let pconf = ProjectConfig::new(self.io_adapter);
        let mut data = pconf.load(project_path)?;
        let slug = slugify!(name);
        data.set_property(DEPS_SECTION, &slug, dependency.to_gdvalue());

        if !no_install {
            self.install(&dependency, project_path)?;
        }

        pconf.save(project_path, data).map_err(Into::into)
    }

    /// Remove dependency from project
    pub fn remove_dependency(&self, project_path: &Path, name: &str) -> Result<(), PluginError> {
        let phandler = ProjectHandler::new(self.io_adapter);
        let project_info = phandler.get_project_info(project_path)?;
        let pconf = ProjectConfig::new(self.io_adapter);
        let mut data = pconf.load(project_path)?;
        let slug = slugify!(name);

        // Check if dependency is present in project
        if let Some(value) = data.get_property(DEPS_SECTION, &slug) {
            let dep = Dependency::from_gdvalue(&slug, &value)?;
            // Check if dependency is installed
            if self.is_installed(&dep, project_path) {
                self.uninstall(&dep, project_path)?;
                println!(
                    "Addon folder {} removed from project {}.",
                    dep.name.color("green"),
                    project_info.get_versioned_name().color("green")
                );
            }
        }

        if data.remove_property(DEPS_SECTION, &slug).is_err() {
            return Err(PluginError::MissingDependency(slug));
        }

        pconf.save(project_path, data).map_err(Into::into)
    }

    /// Fork dependency: integrate plugin inside of project
    pub fn fork_dependency(&self, project_path: &Path, name: &str) -> Result<(), PluginError> {
        let pconf = ProjectConfig::new(self.io_adapter);
        let mut data = pconf.load(project_path)?;
        let slug = slugify!(name);

        // Check if dependency is present in project
        if let Some(value) = data.get_property(DEPS_SECTION, &slug) {
            let mut dep = Dependency::from_gdvalue(&slug, &value)?;
            // Check if dependency is not installed
            if !self.is_installed(&dep, project_path) {
                // Force installl
                self.install(&dep, project_path)?;
            }

            // Set source to current
            dep.source = DependencySource::Current;
            data.set_property(DEPS_SECTION, &slug, dep.to_gdvalue());
            pconf.save(project_path, data)?;
        }

        Ok(())
    }

    /// Sync project dependencies
    ///
    /// * Find and register new dependencies in project
    /// * Install up-to-date dependencies in project
    ///
    pub fn sync_project_plugins(&self, project_path: &Path) -> Result<(), PluginError> {
        // Find and register plugins
        let phandler = ProjectHandler::new(self.io_adapter);
        let pconf = ProjectConfig::new(self.io_adapter);
        let project_info = phandler.get_project_info(project_path)?;
        let mut conf = pconf.load(project_path)?;
        let plugins = self.list_plugins_from_project(project_path)?;
        for plugin in plugins {
            let slug = slugify!(&plugin.name);
            // Check if plugin is absent
            if conf.get_property(DEPS_SECTION, &slug).is_none() {
                let dep = Dependency::from_plugin_info(&plugin);
                conf.set_property(DEPS_SECTION, &slug, dep.to_gdvalue());
                println!(
                    "Plugin {} added as dependency for project {}.",
                    dep.name.color("green"),
                    project_info.get_versioned_name().color("green")
                );
            }
        }
        pconf.save(project_path, conf)?;

        // Install dependencies
        let deps = self.list_project_dependencies(project_path)?;
        for dep in deps {
            match self.install(&dep, project_path) {
                Err(PluginError::AlreadyInstalled(_)) | Ok(_) => Ok(()),
                Err(e) => Err(e),
            }?;

            println!(
                "Plugin {} installed in project {}.",
                dep.name.color("green"),
                project_info.get_versioned_name().color("green")
            );
        }

        Ok(())
    }

    /// Sync one specific project dependency
    pub fn sync_project_plugin(
        &self,
        project_path: &Path,
        plugin_name: &str,
    ) -> Result<(), PluginError> {
        // Find and register plugins
        let phandler = ProjectHandler::new(self.io_adapter);
        let project_info = phandler.get_project_info(project_path)?;
        let plugin_slug = slugify!(plugin_name);
        let pconf = ProjectConfig::new(self.io_adapter);
        let mut conf = pconf.load(project_path)?;
        let plugins = self.list_plugins_from_project(project_path)?;
        for plugin in plugins {
            let slug = slugify!(&plugin.name);
            if slug == plugin_slug {
                // Check if plugin is absent
                if conf.get_property(DEPS_SECTION, &slug).is_none() {
                    let dep = Dependency::from_plugin_info(&plugin);
                    conf.set_property(DEPS_SECTION, &slug, dep.to_gdvalue());
                    println!(
                        "Plugin {} added as dependency for project {}.",
                        dep.name.color("green"),
                        project_info.get_versioned_name().color("green")
                    );
                }
            }
        }
        pconf.save(project_path, conf)?;

        let dep = self.get_dependency(project_path, plugin_name);
        if dep.is_err() {
            return Err(PluginError::MissingDependency(plugin_name.to_owned()));
        }

        let dep = dep?;
        match self.install(&dep, project_path) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                PluginError::AlreadyInstalled(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    /// Desynchronize project dependencies
    ///
    /// Uninstall not-included dependencies.
    pub fn desync_project_plugins(&self, project_path: &Path) -> Result<(), PluginError> {
        let deps = self.list_project_dependencies(project_path)?;
        for dep in deps {
            if dep.source != DependencySource::Current && self.is_installed(&dep, project_path) {
                // Uninstall dependency
                self.uninstall(&dep, project_path)?;
            }
        }

        Ok(())
    }

    /// Desynchronize one specific project dependency
    pub fn desync_project_plugin(
        &self,
        project_path: &Path,
        plugin_name: &str,
    ) -> Result<(), PluginError> {
        let dep = self.get_dependency(project_path, plugin_name);
        if dep.is_err() {
            return Err(PluginError::MissingDependency(plugin_name.to_owned()));
        }

        let dep = dep.unwrap();
        if dep.source == DependencySource::Current {
            return Err(PluginError::CannotDesync(plugin_name.to_owned()));
        }

        self.uninstall(&dep, project_path)?;
        Ok(())
    }
}
