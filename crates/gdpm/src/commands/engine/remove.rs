use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::EngineHandler,
    error::EngineError,
    io::{write_stdout, IoAdapter},
    types::version::GodotVersion,
};

use crate::context::Context;

/// Uninstall engine
#[derive(Parser)]
#[clap(name = "remove", alias = "rm")]
pub struct Remove {
    /// Engine version
    engine: GodotVersion,
    /// Headless?
    #[clap(long)]
    headless: bool,
    /// Server?
    #[clap(long)]
    server: bool,
}

impl Remove {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        match ehandler.uninstall(&self.engine) {
            Ok(()) => write_stdout!(
                context.io(),
                "{}\n",
                format!(
                    "Engine version '{}' was successfully uninstalled.",
                    self.engine
                )
                .color("green")
            )?,
            Err(e) => match e {
                EngineError::EngineNotFound(_) => {
                    write_stdout!(
                        context.io(),
                        "{}\n",
                        format!("Unknown engine version '{}'.", self.engine).color("red")
                    )?;
                    std::process::exit(1);
                }
                EngineError::EngineNotInstalled(_) => {
                    ehandler.unregister(&self.engine)?;
                }
                e => return Err(e.into()),
            },
        }

        Ok(())
    }
}
