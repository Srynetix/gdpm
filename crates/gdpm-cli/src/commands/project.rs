use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{engine::EngineHandler, project::ProjectHandler};
use question::{Answer, Question};

use super::Execute;
use crate::common::{
    get_project_info_or_exit, print_missing_default_engine_message,
    print_missing_project_engine_message, validate_engine_version_or_exit,
};

/// project management
#[derive(Parser)]
#[clap(name = "project")]
pub struct Project {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Info(Info),
    Edit(Edit),
    SetEngine(SetEngine),
    UnsetEngine(UnsetEngine),
}

/// get project info
#[derive(Parser)]
#[clap(name = "info")]
pub struct Info {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

/// edit project
#[derive(Parser)]
#[clap(name = "edit")]
pub struct Edit {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// version
    #[clap(short, long)]
    version: Option<String>,
}

/// set project engine
#[derive(Parser)]
#[clap(name = "set-engine")]
pub struct SetEngine {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// version
    version: String,
}

/// unset project engine
#[derive(Parser)]
#[clap(name = "unset-engine")]
pub struct UnsetEngine {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl Execute for Project {
    fn execute(self) -> Result<()> {
        self.cmd.execute()
    }
}

impl Execute for Command {
    fn execute(self) -> Result<()> {
        match self {
            Self::Info(c) => c.execute(),
            Self::Edit(c) => c.execute(),
            Self::SetEngine(c) => c.execute(),
            Self::UnsetEngine(c) => c.execute(),
        }
    }
}

impl Execute for Info {
    fn execute(self) -> Result<()> {
        let info = get_project_info_or_exit(&self.path);
        info.show();

        Ok(())
    }
}

impl Execute for Edit {
    fn execute(self) -> Result<()> {
        let info = get_project_info_or_exit(&self.path);

        if let Some(v) = self.version {
            validate_engine_version_or_exit(&v)?;
            println!(
                "Running Godot Engine v{} for project {} ...",
                v.color("green"),
                info.get_versioned_name().color("green")
            );
            EngineHandler::run_version_for_project(&v, &self.path)?;
        } else if let Some(e) = info.get_engine_version() {
            println!(
                "Running Godot Engine v{} for project {} ...",
                e.color("green"),
                info.get_versioned_name().color("green")
            );
            EngineHandler::run_version_for_project(e, &self.path)?;
        } else if let Some(e) = EngineHandler::get_default()? {
            print_missing_project_engine_message();
            match Question::new(&format!(
                "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                e.color("green"),
                info.get_versioned_name().color("green")
            ))
            .confirm()
            {
                Answer::YES => ProjectHandler::set_project_engine(&self.path, &e)?,
                Answer::NO => println!("Okay. You will be asked again next time."),
                _ => unreachable!(),
            }

            println!(
                "Running Godot Engine v{} for project {} ...",
                e.color("green"),
                info.get_versioned_name().color("green")
            );
            EngineHandler::run_version_for_project(&e, &self.path)?;
        } else {
            print_missing_project_engine_message();
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for SetEngine {
    fn execute(self) -> Result<()> {
        let info = get_project_info_or_exit(&self.path);
        validate_engine_version_or_exit(&self.version)?;
        ProjectHandler::set_project_engine(&self.path, &self.version)?;
        println!(
            "Godot Engine v{} set for project {}.",
            self.version.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}

impl Execute for UnsetEngine {
    fn execute(self) -> Result<()> {
        ProjectHandler::unset_project_engine(&self.path)?;
        let info = get_project_info_or_exit(&self.path);

        println!(
            "Engine deassociated from project {}.",
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}
