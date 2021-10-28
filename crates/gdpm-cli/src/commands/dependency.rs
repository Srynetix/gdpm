use std::path::PathBuf;

use argh::FromArgs;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    plugins::{
        add_dependency, desync_project_plugin, desync_project_plugins, fork_dependency,
        list_project_dependencies, remove_dependency, sync_project_plugin, sync_project_plugins,
    },
    project::get_project_info,
};

use super::Execute;

/// add dependency
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
pub struct Add {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(positional)]
    name: String,
    /// version
    #[argh(positional)]
    version: String,
    /// source
    #[argh(positional)]
    source: String,
    /// do not install
    #[argh(option)]
    no_install: bool,
}

/// fork dependency: include in code
#[derive(FromArgs)]
#[argh(subcommand, name = "fork")]
pub struct Fork {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(positional)]
    name: String,
}

/// remove dependency
#[derive(FromArgs)]
#[argh(subcommand, name = "remove")]
pub struct Remove {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(positional)]
    name: String,
}

/// list dependencies
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
}

/// sync project dependencies
#[derive(FromArgs)]
#[argh(subcommand, name = "sync")]
pub struct Sync {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(option)]
    name: Option<String>,
}

/// desync project dependencies
#[derive(FromArgs)]
#[argh(subcommand, name = "desync")]
pub struct Desync {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(option)]
    name: Option<String>,
}

impl Execute for Add {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;
        add_dependency(
            &self.path,
            &self.name,
            &self.version,
            &self.source,
            self.no_install,
        )?;

        if self.no_install {
            println!(
                "Dependency {} (v{}) from {} added to project {}.",
                self.name.color("green"),
                self.version.color("green"),
                self.source.color("blue"),
                info.get_versioned_name().color("green")
            );
        } else {
            println!(
                "Dependency {} (v{}) from {} added and installed to project {}.",
                self.name.color("green"),
                self.version.color("green"),
                self.source.color("blue"),
                info.get_versioned_name().color("green")
            );
        }

        Ok(())
    }
}

impl Execute for Fork {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;
        fork_dependency(&self.path, &self.name)?;

        println!(
            "Plugin {} forked in project {}.",
            self.name.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}

impl Execute for Remove {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;
        remove_dependency(&self.path, &self.name)?;

        println!(
            "Dependency {} removed from project {}.",
            self.name.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}

impl Execute for List {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;
        let dependencies = list_project_dependencies(&self.path)?;
        println!(
            "Dependencies from project {}:",
            info.get_versioned_name().color("green")
        );

        for dep in dependencies {
            println!("- {}", dep.get_verbose_name());
        }

        Ok(())
    }
}

impl Execute for Sync {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;

        if let Some(n) = self.name {
            sync_project_plugin(&self.path, &n)?;

            println!(
                "Dependency {} is now synchronized for project {}.",
                n.color("green"),
                info.get_versioned_name().color("green")
            )
        } else {
            sync_project_plugins(&self.path)?;

            println!(
                "Dependencies are now synchronized for project {}.",
                info.get_versioned_name().color("green")
            )
        }

        Ok(())
    }
}

impl Execute for Desync {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;

        if let Some(n) = self.name {
            desync_project_plugin(&self.path, &n)?;

            println!(
                "Dependency {} is desynchronized for project {}.",
                n.color("green"),
                info.get_versioned_name().color("green")
            );
        } else {
            desync_project_plugins(&self.path)?;

            println!(
                "Dependencies are desynchronized for project {}.",
                info.get_versioned_name().color("green")
            )
        }

        Ok(())
    }
}
