use clap::{Parser, Subcommand};
use color_eyre::Result;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter};
use tracing_subscriber::EnvFilter;

use crate::context::Context;

mod dependency;
mod engine;
mod project;

/// manage Godot versions and project dependencies
#[derive(Parser)]
#[clap(author, version, about, long_about = None, name = "gdpm")]
#[clap(propagate_version = true)]
pub struct Args {
    /// verbose mode
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
    Project(project::Project),
    Dependencies(dependency::Dependencies),
    Engine(engine::Engine),
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
        Command::Project(c) => c.execute(context),
        Command::Dependencies(c) => c.execute(context),
        Command::Engine(c) => c.execute(context),
    }
}
