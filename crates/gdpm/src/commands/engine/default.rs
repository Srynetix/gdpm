use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter, engine::EngineHandler, io::IoAdapter, types::version::GodotVersion,
};

use crate::{
    common::{print_missing_default_engine_message, validate_engine_version_or_exit},
    context::Context,
};

/// Show or set default engine
#[derive(Parser)]
pub struct Default {
    /// Engine verion
    engine: Option<GodotVersion>,
}

impl Default {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        if let Some(version) = self.engine {
            validate_engine_version_or_exit(context.io(), &version)?;
            let ehandler = EngineHandler::new(context.io());
            ehandler.set_as_default(&version)?;
            println!(
                "Godot Engine v{} set as default.",
                version.to_string().color("green")
            );
        } else {
            let ehandler = EngineHandler::new(context.io());
            if let Some(e) = ehandler.get_default()? {
                println!("{} {}", "*".color("green"), e.to_string().color("green"));
            } else {
                print_missing_default_engine_message();
            }
        }

        Ok(())
    }
}
