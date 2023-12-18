use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter, io::IoAdapter, project::ProjectHandler,
    types::version::GodotVersion,
};

use crate::{
    common::{get_project_info_or_exit, validate_engine_version_or_exit},
    context::Context,
};

use super::Execute;

#[derive(Parser)]
pub struct SetEngine {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// Engine version
    engine: GodotVersion,
}

impl Execute for SetEngine {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let phandler = ProjectHandler::new(context.io());
        validate_engine_version_or_exit(context.io(), &self.engine)?;
        phandler.set_project_engine(&self.path, &self.engine)?;
        println!(
            "Godot Engine v{} set for project {}.",
            self.engine.to_string().color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}
