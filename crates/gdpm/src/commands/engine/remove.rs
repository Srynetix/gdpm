use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::EngineHandler,
    error::EngineError,
    io::{write_stdout, IoAdapter},
    types::version::{GodotVersion, SystemVersion},
};

use crate::{common::parse_godot_version_args, context::Context};

/// Uninstall engine
#[derive(Parser)]
#[clap(name = "remove", alias = "rm")]
pub struct Remove {
    /// Engine version
    engine: GodotVersion,
    /// System version
    #[clap(long)]
    system_version: Option<SystemVersion>,
    /// Headless?
    #[clap(long)]
    headless: bool,
    /// Server?
    #[clap(long)]
    server: bool,
}

impl Remove {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let (version, _system) = parse_godot_version_args(&self.engine, self.system_version)?;

        let ehandler = EngineHandler::new(context.io());
        match ehandler.uninstall(&version) {
            Ok(()) => write_stdout!(
                context.io(),
                "{}\n",
                format!("Engine version '{}' was successfully uninstalled.", version)
                    .color("green")
            )?,
            Err(e) => match e {
                EngineError::EngineNotFound(_) => {
                    write_stdout!(
                        context.io(),
                        "{}\n",
                        format!("Unknown engine version '{}'.", version).color("red")
                    )?;
                    std::process::exit(1);
                }
                EngineError::EngineNotInstalled(_) => {
                    ehandler.unregister(&version)?;
                }
                e => return Err(e.into()),
            },
        }

        Ok(())
    }
}
