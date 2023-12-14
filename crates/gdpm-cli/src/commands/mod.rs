use clap::{Parser, Subcommand};
use color_eyre::Result;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter};
use tracing_subscriber::EnvFilter;

use crate::context::Context;

mod add;
mod edit;
mod engine;
mod info;
mod new;
mod remove;
mod run;
mod set_engine;
mod sync;
mod unset_engine;

/// Manage Godot versions and project dependencies
#[derive(Parser)]
#[clap(author, version, about, long_about = None, name = "gdpm")]
#[clap(propagate_version = true)]
pub struct Args {
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,

    #[clap(subcommand)]
    command: Command,
}

trait Execute {
    /// Execute!
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()>;
}

#[derive(Subcommand)]
enum Command {
    /// Manage engine versions
    Engine(engine::Engine),
    /// Add dependency
    Add(add::Add),
    /// Edit project using associated engine version
    Edit(edit::Edit),
    /// Show project info
    Info(info::Info),
    /// Create a new project
    New(new::New),
    /// Remove dependency
    Remove(remove::Remove),
    /// Run project using associated engine version
    Run(run::Run),
    /// Set associated engine version
    SetEngine(set_engine::SetEngine),
    /// Sync installed dependencies
    Sync(sync::Sync),
    /// Unset associated engine version
    UnsetEngine(unset_engine::UnsetEngine),
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
        Command::Add(c) => c.execute(context),
        Command::Edit(c) => c.execute(context),
        Command::Engine(c) => c.execute(context),
        Command::Info(c) => c.execute(context),
        Command::New(c) => c.execute(context),
        Command::Remove(c) => c.execute(context),
        Command::Run(c) => c.execute(context),
        Command::SetEngine(c) => c.execute(context),
        Command::Sync(c) => c.execute(context),
        Command::UnsetEngine(c) => c.execute(context),
    }
}
