use std::path::Path;

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

/// Run command on engine
#[derive(Parser)]
pub struct Run {
    /// Engine version
    #[clap(short, long)]
    engine: Option<GodotVersion>,
    /// Arguments
    args: Vec<String>,
}

impl Run {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?;

            if self.args.is_empty() {
                println!("Running Godot Engine v{} ...", v.to_string().color("green"));
                ehandler.run_version_for_project(&v, Path::new("."))?;
            } else {
                println!(
                    "Executing command {} Godot Engine v{} ...",
                    self.args.join(" ").color("blue"),
                    v.to_string().color("green")
                );
                ehandler.exec_version_for_project(&v, &self.args, Path::new("."))?;
            }
        } else if let Some(e) = ehandler.get_default()? {
            if self.args.is_empty() {
                println!(
                    "Executing command {} on Godot Engine v{} ...",
                    self.args.join(" ").color("blue"),
                    e.to_string().color("green")
                );
                ehandler.exec_version_for_project(&e, &self.args, Path::new("."))?;
            } else {
                println!("Running Godot Engine v{} ...", e.to_string().color("green"));
                ehandler.run_version_for_project(&e, Path::new("."))?;
            }
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}
