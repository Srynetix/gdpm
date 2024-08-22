use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    io::{write_stdout, IoAdapter},
    plugins::DependencyHandler,
};

use crate::{common::get_project_info_or_exit, context::Context};

#[derive(Parser)]
pub struct Sync {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// Name
    name: Option<String>,
}

impl Sync {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context, &self.path)?;
        let dhandler = DependencyHandler::new(context.io());

        if let Some(n) = self.name {
            dhandler.sync_project_plugin(&self.path, &n)?;

            write_stdout!(
                context.io(),
                "Dependency {} is now synchronized for project {}.\n",
                n.color("green"),
                info.get_versioned_name().color("green")
            )?;
        } else {
            dhandler.sync_project_plugins(&self.path)?;

            write_stdout!(
                context.io(),
                "Dependencies are now synchronized for project {}.\n",
                info.get_versioned_name().color("green")
            )?;
        }

        Ok(())
    }
}
