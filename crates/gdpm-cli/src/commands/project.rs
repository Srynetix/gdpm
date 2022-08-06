use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter, engine::EngineHandler, io::IoAdapter, project::ProjectHandler,
};
use gdscript_parser::GdScriptParser;
use question::{Answer, Question};

use super::Execute;
use crate::{
    common::{
        get_project_info_or_exit, print_missing_default_engine_message,
        print_missing_project_engine_message, validate_engine_version_or_exit,
    },
    context::Context,
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
    Lint(Lint),
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

/// lint GDScript code
#[derive(Parser)]
#[clap(name = "lint")]
pub struct Lint {
    /// file/dir path
    path: PathBuf,
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
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        self.cmd.execute(context)
    }
}

impl Execute for Command {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        match self {
            Self::Info(c) => c.execute(context),
            Self::Edit(c) => c.execute(context),
            Self::Lint(c) => c.execute(context),
            Self::SetEngine(c) => c.execute(context),
            Self::UnsetEngine(c) => c.execute(context),
        }
    }
}

impl Execute for Info {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        info.show();

        Ok(())
    }
}

impl Execute for Lint {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        match GdScriptParser::parse_path(context.io(), self.path) {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("{}", e);
                Ok(())
            }
        }
    }
}

impl Execute for Edit {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let ehandler = EngineHandler::new(context.io());
        let phandler = ProjectHandler::new(context.io());

        if let Some(v) = self.version {
            validate_engine_version_or_exit(context.io(), &v)?;
            println!(
                "Running Godot Engine v{} for project {} ...",
                v.color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(&v, &self.path)?;
        } else if let Some(e) = info.get_engine_version() {
            println!(
                "Running Godot Engine v{} for project {} ...",
                e.color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(e, &self.path)?;
        } else if let Some(e) = ehandler.get_default()? {
            print_missing_project_engine_message();
            match Question::new(&format!(
                "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                e.color("green"),
                info.get_versioned_name().color("green")
            ))
            .confirm()
            {
                Answer::YES => phandler.set_project_engine(&self.path, &e)?,
                Answer::NO => println!("Okay. You will be asked again next time."),
                _ => unreachable!(),
            }

            println!(
                "Running Godot Engine v{} for project {} ...",
                e.color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(&e, &self.path)?;
        } else {
            print_missing_project_engine_message();
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for SetEngine {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let phandler = ProjectHandler::new(context.io());
        validate_engine_version_or_exit(context.io(), &self.version)?;
        phandler.set_project_engine(&self.path, &self.version)?;
        println!(
            "Godot Engine v{} set for project {}.",
            self.version.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}

impl Execute for UnsetEngine {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let phandler = ProjectHandler::new(context.io());
        phandler.unset_project_engine(&self.path)?;
        let info = get_project_info_or_exit(context.io(), &self.path);

        println!(
            "Engine deassociated from project {}.",
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}
