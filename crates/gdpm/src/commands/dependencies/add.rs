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
pub struct Add {
    /// project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    /// name
    name: String,
    /// source
    source: String,
    /// do not sync
    #[clap(long)]
    no_sync: bool,
}

impl Add {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context, &self.path)?;
        let dhandler = DependencyHandler::new(context.io());
        dhandler.add_dependency(&self.path, &self.name, &self.source, self.no_sync)?;

        if self.no_sync {
            write_stdout!(
                context.io(),
                "Dependency {} from {} added to project {}.\n",
                self.name.color("green"),
                self.source.color("blue"),
                info.get_versioned_name().color("green")
            )?;
        } else {
            write_stdout!(
                context.io(),
                "Dependency {} from {} added and installed to project {}.\n",
                self.name.color("green"),
                self.source.color("blue"),
                info.get_versioned_name().color("green")
            )?;
        }

        Ok(())
    }
}
