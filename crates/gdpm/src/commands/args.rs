use clap::{Parser, Subcommand};
use color_eyre::Result;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter};
use tracing_subscriber::EnvFilter;

use crate::context::Context;

use super::dependencies;
use super::engine;
use super::project;

/// Manage Godot versions and project dependencies
#[derive(Parser)]
#[clap(author, version, about, long_about = None, name = "gdpm")]
#[clap(propagate_version = true, infer_subcommands = true)]
pub struct Args {
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage engine versions
    Engine {
        #[clap(subcommand)]
        command: EngineCommand,
    },
    /// Manage dependencies (WIP)
    Deps {
        #[clap(subcommand)]
        command: DependenciesCommand,
    },
    /// Manage project
    Project {
        #[clap(subcommand)]
        command: ProjectCommand,
    },
}

#[derive(Subcommand)]
#[clap(disable_version_flag = true)]
enum EngineCommand {
    Add(engine::add::Add),
    Remove(engine::remove::Remove),
    Default(engine::default::Default),
    List(engine::list::List),
    ListRemote(engine::list_remote::ListRemote),
    Run(engine::run::Run),
}

#[derive(Subcommand)]
#[clap(disable_version_flag = true)]
enum DependenciesCommand {
    /// Add dependency
    Add(dependencies::add::Add),
    /// Remove dependency
    Remove(dependencies::remove::Remove),
    /// Sync installed dependencies
    Sync(dependencies::sync::Sync),
}

#[derive(Subcommand)]
#[clap(disable_version_flag = true)]
enum ProjectCommand {
    /// Create a new project
    New(project::new::New),
    /// Edit project using associated engine version
    Edit(project::edit::Edit),
    /// Show project info
    Info(project::info::Info),
    /// Run project using associated engine version
    Run(project::run::Run),
    /// Set associated engine version
    SetEngine(project::set_engine::SetEngine),
    /// Unset associated engine version
    UnsetEngine(project::unset_engine::UnsetEngine),
}

pub fn parse_args<I: IoAdapter, D: DownloadAdapter>(
    context: Context<I, D>,
    args: Args,
) -> Result<()> {
    // Set RUST_LOG depending on "verbose" arg
    if std::env::var("RUST_LOG").unwrap_or_default().is_empty() {
        if args.verbose {
            std::env::set_var("RUST_LOG", "warn,gdpm=debug");
        } else {
            std::env::set_var("RUST_LOG", "warn,gdpm=info");
        }
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .with_target(false)
        .compact()
        .init();

    match args.command {
        Command::Engine { command } => match command {
            EngineCommand::Add(c) => c.execute(&context),
            EngineCommand::Remove(c) => c.execute(&context),
            EngineCommand::Default(c) => c.execute(&context),
            EngineCommand::List(c) => c.execute(&context),
            EngineCommand::Run(c) => c.execute(&context),
            EngineCommand::ListRemote(c) => c.execute(&context),
        },
        Command::Deps { command } => match command {
            DependenciesCommand::Add(c) => c.execute(&context),
            DependenciesCommand::Remove(c) => c.execute(&context),
            DependenciesCommand::Sync(c) => c.execute(&context),
        },
        Command::Project { command } => match command {
            ProjectCommand::Edit(c) => c.execute(&context),
            ProjectCommand::Info(c) => c.execute(&context),
            ProjectCommand::New(c) => c.execute(&context),
            ProjectCommand::Run(c) => c.execute(&context),
            ProjectCommand::SetEngine(c) => c.execute(&context),
            ProjectCommand::UnsetEngine(c) => c.execute(&context),
        },
    }
}
