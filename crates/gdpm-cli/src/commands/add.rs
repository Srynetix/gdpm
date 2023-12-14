use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter, plugins::DependencyHandler};

use crate::common::get_project_info_or_exit;

use super::Execute;

#[derive(Parser)]
pub struct Add {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: String,
    /// version
    version: String,
    /// source
    source: String,
    /// do not sync
    #[clap(long)]
    no_sync: bool,
}

impl Execute for Add {
    fn execute<I: IoAdapter, D: DownloadAdapter>(
        self,
        context: crate::context::Context<I, D>,
    ) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let dhandler = DependencyHandler::new(context.io());
        dhandler.add_dependency(
            &self.path,
            &self.name,
            &self.version,
            &self.source,
            self.no_sync,
        )?;

        if self.no_sync {
            println!(
                "Dependency {} (v{}) from {} added to project {}.",
                self.name.color("green"),
                self.version.color("green"),
                self.source.color("blue"),
                info.get_versioned_name().color("green")
            );
        } else {
            println!(
                "Dependency {} (v{}) from {} added and installed to project {}.",
                self.name.color("green"),
                self.version.color("green"),
                self.source.color("blue"),
                info.get_versioned_name().color("green")
            );
        }

        Ok(())
    }
}
