use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::EngineHandler,
    io::IoAdapter,
    scaffolder::{ProjectInfo, ProjectRenderer, Scaffolder},
    types::version::GodotVersion,
};

use crate::{
    common::{print_missing_default_engine_message, validate_engine_version_or_exit},
    context::Context,
};

use super::Execute;

#[derive(Parser)]
pub struct New {
    /// Game name
    game_name: String,

    /// Project path
    path: PathBuf,

    /// Renderer
    #[clap(short, long, default_value = "forward+")]
    renderer: ProjectRenderer,

    /// Engine version
    #[clap(short, long)]
    engine: Option<GodotVersion>,
}

impl Execute for New {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let engine = if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?
        } else if let Some(v) = ehandler.get_default()? {
            validate_engine_version_or_exit(context.io(), &v)?
        } else {
            print_missing_default_engine_message();
            return Ok(());
        };

        if !engine.is_version_4() {
            println!(
                "{}",
                "Project scaffolding is only supported for Godot 4".color("yellow")
            );
        } else {
            let scaffolder = Scaffolder::new(context.io());
            let project_info = ProjectInfo::new(self.game_name, self.renderer);
            scaffolder.scaffold(engine.version, project_info, &self.path)?;
        }

        Ok(())
    }
}
