use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::EngineHandler,
    io::{write_stderr, write_stdout, IoAdapter},
    scaffolder::{ProjectInfo, ProjectRenderer, Scaffolder},
    types::version::GodotVersion,
};

use crate::{
    common::{print_missing_default_engine_message, validate_engine_version_or_exit},
    context::Context,
};

#[derive(Parser)]
pub struct New {
    /// Game name
    game_name: String,

    /// Project path
    path: PathBuf,

    /// Renderer
    #[clap(short, long, default_value = "forward_plus")]
    renderer: ProjectRenderer,

    /// Engine version
    #[clap(short, long)]
    engine: Option<GodotVersion>,
}

impl New {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let engine = if let Some(v) = self.engine {
            validate_engine_version_or_exit(context, &v)?
        } else if let Some(v) = ehandler.get_default()? {
            validate_engine_version_or_exit(context, &v)?
        } else {
            print_missing_default_engine_message(context)?;
            return Ok(());
        };

        if !engine.is_version_4() {
            write_stderr!(
                context.io(),
                "{}\n",
                "Project scaffolding is only supported for Godot 4".color("yellow")
            )?;
        } else {
            let scaffolder = Scaffolder::new(context.io());
            let project_info = ProjectInfo::new(self.game_name, self.renderer);
            scaffolder.scaffold(engine.version, project_info, &self.path)?;

            write_stdout!(
                context.io(),
                "{}\n",
                "Project successfully generated".color("green")
            )?;
        }

        Ok(())
    }
}
