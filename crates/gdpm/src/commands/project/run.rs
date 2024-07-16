use std::path::PathBuf;

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

#[derive(Parser)]
pub struct Run {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// Engine version
    engine: Option<GodotVersion>,
}

impl Run {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?;
            println!(
                "Running project using Godot Engine v{} ...",
                v.to_string().color("green")
            );
            ehandler.run_version_for_project_no_editor(&v, &self.path)?;
        } else if let Some(e) = ehandler.get_default()? {
            println!(
                "Running project using Godot Engine v{} ...",
                e.to_string().color("green")
            );
            ehandler.run_version_for_project_no_editor(&e, &self.path)?;
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}
