use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter, plugins::DependencyHandler};

use crate::{common::get_project_info_or_exit, context::Context};

use super::Execute;

#[derive(Parser)]
pub struct Sync {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// Name
    name: Option<String>,
}

impl Execute for Sync {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());

        if let Some(n) = self.name {
            dhandler.sync_project_plugin(&self.path, &n)?;

            println!(
                "Dependency {} is now synchronized for project {}.",
                n.color("green"),
                info.get_versioned_name().color("green")
            )
        } else {
            dhandler.sync_project_plugins(&self.path)?;

            println!(
                "Dependencies are now synchronized for project {}.",
                info.get_versioned_name().color("green")
            )
        }

        Ok(())
    }
}
