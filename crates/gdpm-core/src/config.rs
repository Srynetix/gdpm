//! User .gdpm config module.

use std::path::{Path, PathBuf};

use gdpm_io::{Error, IoAdapter};
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
    pub fn get_or_create_global_directory(&self) -> Result<PathBuf, Error> {
        let config_directory = self
            .io_adapter
            .get_user_configuration_directory()?
            .join(ROOT_CONFIG_FOLDER_NAME);
        if !self.io_adapter.path_exists(&config_directory) {
            self.io_adapter.create_dir(&config_directory)?;
        }

        Ok(config_directory)
    }

    /// Get or create directory in global directory.
    pub fn get_or_create_directory(&self, path: &Path) -> Result<PathBuf, Error> {
        let path = self.get_or_create_global_directory()?.join(path);
        if !self.io_adapter.path_exists(&path) {
            self.io_adapter.create_dir(&path)?;
        }

        Ok(path)
    }

    /// Get or create file in global directory.
    pub fn get_or_create_file(&self, path: &Path) -> Result<PathBuf, Error> {
        let path = self.get_or_create_global_directory()?.join(path);
        if !self.io_adapter.path_exists(&path) {
            self.io_adapter.create_file(&path)?;
        }

        Ok(path)
    }

    /// Get file in global directory.
    pub fn get_file(&self, path: &Path) -> Result<PathBuf, Error> {
        Ok(self.get_or_create_global_directory()?.join(path))
    }

    /// Read file to string from global directory.
    pub fn read_file_to_string(&self, path: &Path) -> Result<String, Error> {
        self.io_adapter.read_file_to_string(&self.get_file(path)?)
    }

    /// Write string to file in global directory.
    pub fn write_string_to_file(&self, path: &Path, contents: &str) -> Result<(), Error> {
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

        let path = self.get_global_config_path();
        udir.get_or_create_file(path)?;

        let contents = udir.read_file_to_string(path)?;
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
        if !self.io_adapter.path_exists(&project) {
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

#[cfg(test)]
mod tests {
    mod userdir {
        use std::path::{Path, PathBuf};

        use gdpm_io::MockIoAdapter;

        use crate::config::UserDir;
        use mockall::predicate;

        #[test]
        fn test_get_or_create_global_directory() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .return_once(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| false);

            adapter
                .expect_create_dir()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .return_once(|_| Ok(()));

            let udir = UserDir::new(&adapter);
            assert_eq!(
                udir.get_or_create_global_directory().unwrap(),
                PathBuf::from("/home/user/.config/gdpm")
            );
        }

        #[test]
        fn test_get_or_create_directory() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .return_once(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| false);
            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/foo")))
                .times(1)
                .returning(|_| false);

            adapter
                .expect_create_dir()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| Ok(()));
            adapter
                .expect_create_dir()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/foo")))
                .times(1)
                .returning(|_| Ok(()));

            let udir = UserDir::new(&adapter);
            assert_eq!(
                udir.get_or_create_directory(&PathBuf::from("foo")).unwrap(),
                PathBuf::from("/home/user/.config/gdpm/foo")
            );
        }

        #[test]
        fn test_get_file() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .return_once(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| true);

            let udir = UserDir::new(&adapter);
            assert_eq!(
                udir.get_file(&PathBuf::from("foo")).unwrap(),
                PathBuf::from("/home/user/.config/gdpm/foo")
            );
        }

        #[test]
        fn test_get_or_create_file() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .return_once(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| true);
            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/foo")))
                .times(1)
                .returning(|_| false);

            adapter
                .expect_create_file()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/foo")))
                .times(1)
                .returning(|_| Ok(()));

            let udir = UserDir::new(&adapter);
            udir.get_or_create_file(&PathBuf::from("foo")).unwrap();
        }

        #[test]
        fn test_read_file_to_string() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .return_once(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| true);

            adapter
                .expect_read_file_to_string()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/foo")))
                .times(1)
                .returning(|_| Ok("OK".into()));

            let udir = UserDir::new(&adapter);
            assert_eq!(
                udir.read_file_to_string(&PathBuf::from("foo")).unwrap(),
                "OK".to_string()
            );
        }

        #[test]
        fn test_write_string_to_file() {
            let mut adapter = MockIoAdapter::new();
            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .return_once(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| true);

            adapter
                .expect_write_string_to_file()
                .with(
                    predicate::eq(Path::new("/home/user/.config/gdpm/foo")),
                    predicate::eq("OK"),
                )
                .times(1)
                .returning(|_, _| Ok(()));

            let udir = UserDir::new(&adapter);
            udir.write_string_to_file(&PathBuf::from("foo"), "OK")
                .unwrap();
        }
    }

    mod globalconfig {
        use mockall::predicate;
        use std::path::{Path, PathBuf};

        use gdpm_io::MockIoAdapter;
        use gdsettings_parser::{GdSettings, GdSettingsType};

        use crate::config::{GlobalConfig, GLOBAL_CONFIG_FILENAME};

        #[test]
        fn test_get_global_config_path() {
            let adapter = MockIoAdapter::new();

            let gconf = GlobalConfig::new(&adapter);
            assert_eq!(
                gconf.get_global_config_path(),
                Path::new(GLOBAL_CONFIG_FILENAME)
            );
        }

        #[test]
        fn test_load() {
            let mut adapter = MockIoAdapter::new();
            let empty_settings = GdSettings::new(GdSettingsType::new());
            let empty_settings_str = empty_settings.to_string();

            adapter
                .expect_get_user_configuration_directory()
                .times(2)
                .returning(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(2)
                .returning(|_| true);

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/gdpm.cfg")))
                .times(1)
                .returning(|_| false);

            adapter
                .expect_create_file()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/gdpm.cfg")))
                .times(1)
                .returning(|_| Ok(()));

            adapter
                .expect_read_file_to_string()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm/gdpm.cfg")))
                .times(1)
                .return_once(move |_| Ok(empty_settings_str));

            let gconf = GlobalConfig::new(&adapter);
            assert_eq!(gconf.load().unwrap(), empty_settings);
        }

        #[test]
        fn test_save() {
            let mut adapter = MockIoAdapter::new();
            let empty_settings = GdSettings::new(GdSettingsType::new());
            let empty_settings_str = empty_settings.to_string();

            adapter
                .expect_get_user_configuration_directory()
                .times(1)
                .returning(|| Ok(PathBuf::from("/home/user/.config")));

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/home/user/.config/gdpm")))
                .times(1)
                .returning(|_| true);

            adapter
                .expect_write_string_to_file()
                .withf(move |path, contents| {
                    path == Path::new("/home/user/.config/gdpm/gdpm.cfg")
                        && contents == empty_settings_str
                })
                .times(1)
                .return_once(|_, _| Ok(()));

            let gconf = GlobalConfig::new(&adapter);
            gconf.save(empty_settings).unwrap();
        }
    }

    mod projectconfig {
        use mockall::predicate;
        use std::path::Path;

        use gdpm_io::MockIoAdapter;
        use gdsettings_parser::{GdSettings, GdSettingsType};

        use crate::config::{ProjectConfig, PROJECT_CONFIG_FILENAME};

        #[test]
        fn test_get_project_config_path() {
            let adapter = MockIoAdapter::new();

            let pconf = ProjectConfig::new(&adapter);
            assert_eq!(
                pconf.get_project_config_path(Path::new("/")),
                Path::new("/").join(PROJECT_CONFIG_FILENAME)
            );
        }

        #[test]
        fn test_ensure_project_exists() {
            let mut adapter = MockIoAdapter::new();

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/project.godot")))
                .times(1)
                .returning(|_| true);

            let pconf = ProjectConfig::new(&adapter);
            assert_eq!(
                pconf.ensure_project_exists(Path::new("/")).unwrap(),
                Path::new("/").join(PROJECT_CONFIG_FILENAME)
            );
        }

        #[test]
        fn test_load() {
            let mut adapter = MockIoAdapter::new();
            let empty_settings = GdSettings::new(GdSettingsType::new());
            let empty_settings_str = empty_settings.to_string();

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/project.godot")))
                .times(1)
                .returning(|_| true);

            adapter
                .expect_read_file_to_string()
                .with(predicate::eq(Path::new("/project.godot")))
                .times(1)
                .returning(move |_| Ok(empty_settings_str.clone()));

            let pconf = ProjectConfig::new(&adapter);
            assert_eq!(pconf.load(Path::new("/")).unwrap(), empty_settings);
        }

        #[test]
        fn test_save() {
            let mut adapter = MockIoAdapter::new();
            let empty_settings = GdSettings::new(GdSettingsType::new());
            let empty_settings_str = empty_settings.to_string();

            adapter
                .expect_path_exists()
                .with(predicate::eq(Path::new("/project.godot")))
                .times(1)
                .returning(|_| true);

            adapter
                .expect_write_string_to_file()
                .withf(move |path, contents| {
                    path == Path::new("/project.godot") && contents == empty_settings_str
                })
                .times(1)
                .returning(|_, _| Ok(()));

            let pconf = ProjectConfig::new(&adapter);
            pconf.save(Path::new("/"), empty_settings).unwrap();
        }
    }
}
