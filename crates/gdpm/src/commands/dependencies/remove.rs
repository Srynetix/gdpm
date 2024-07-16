use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter, plugins::DependencyHandler};

use crate::{common::get_project_info_or_exit, context::Context};

#[derive(Parser)]
pub struct Remove {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// Name
    name: String,
}

impl Remove {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());
        dhandler.remove_dependency(&self.path, &self.name)?;

        println!(
            "Dependency {} removed from project {}.",
            self.name.color("green"),
            info.get_versioned_name().color("green")
        );

        dhandler.desync_project_plugin(&self.path, &self.name)?;
        println!(
            "Dependency {} is desynchronized for project {}.",
            self.name.color("green"),
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}