use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter, plugins::DependencyHandler};

use super::Execute;
use crate::{common::get_project_info_or_exit, context::Context};

/// dependency management
#[derive(Parser)]
#[clap(name = "deps")]
pub struct Dependencies {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Add(Add),
    Fork(Fork),
    Remove(Remove),
    List(List),
    Sync(Sync),
    Desync(Desync),
}

/// add dependency
#[derive(Parser)]
#[clap(name = "add")]
pub struct Add {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: String,
    /// version
    version: String,
    /// source
    source: String,
    /// do not install
    #[clap(long)]
    no_install: bool,
}

/// fork dependency: include in code
#[derive(Parser)]
#[clap(name = "fork")]
pub struct Fork {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: String,
}

/// remove dependency
#[derive(Parser)]
#[clap(name = "remove")]
pub struct Remove {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: String,
}

/// list dependencies
#[derive(Parser)]
#[clap(name = "list")]
pub struct List {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

/// sync project dependencies
#[derive(Parser)]
#[clap(name = "sync")]
pub struct Sync {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: Option<String>,
}

/// desync project dependencies
#[derive(Parser)]
#[clap(name = "desync")]
pub struct Desync {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: Option<String>,
}

impl Execute for Dependencies {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        self.cmd.execute(context)
    }
}

impl Execute for Command {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        match self {
            Self::List(c) => c.execute(context),
            Self::Add(c) => c.execute(context),
            Self::Fork(c) => c.execute(context),
            Self::Remove(c) => c.execute(context),
            Self::Sync(c) => c.execute(context),
            Self::Desync(c) => c.execute(context),
        }
    }
}

impl Execute for Add {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());
        dhandler.add_dependency(
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
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());
        dhandler.fork_dependency(&self.path, &self.name)?;

        println!(
            "Plugin {} forked in project {}.",
            self.name.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}

impl Execute for Remove {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());
        dhandler.remove_dependency(&self.path, &self.name)?;

        println!(
            "Dependency {} removed from project {}.",
            self.name.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}

impl Execute for List {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());
        let dependencies = dhandler.list_project_dependencies(&self.path)?;
        if dependencies.is_empty() {
            println!(
                "Project '{}' has no dependency.",
                info.get_versioned_name().color("green")
            );
        } else {
            println!(
                "Dependencies from project '{}':",
                info.get_versioned_name().color("green")
            );

            for dep in dependencies {
                println!("- {}", dep.get_verbose_name());
            }
        }

        Ok(())
    }
}

impl Execute for Sync {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());

        if let Some(n) = self.name {
            dhandler.sync_project_plugin(&self.path, &n)?;

            println!(
                "Dependency {} is now synchronized for project {}.",
                n.color("green"),
                info.get_versioned_name().color("green")
            )
        } else {
            dhandler.sync_project_plugins(&self.path)?;

            println!(
                "Dependencies are now synchronized for project {}.",
                info.get_versioned_name().color("green")
            )
        }

        Ok(())
    }
}

impl Execute for Desync {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());

        if let Some(n) = self.name {
            dhandler.desync_project_plugin(&self.path, &n)?;

            println!(
                "Dependency {} is desynchronized for project {}.",
                n.color("green"),
                info.get_versioned_name().color("green")
            );
        } else {
            dhandler.desync_project_plugins(&self.path)?;

            println!(
                "Dependencies are desynchronized for project {}.",
                info.get_versioned_name().color("green")
            )
        }

        Ok(())
    }
}
