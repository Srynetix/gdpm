use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    io::{write_stdout, IoAdapter},
    project::ProjectHandler,
};

use crate::{common::get_project_info_or_exit, context::Context};

#[derive(Parser)]
pub struct UnsetEngine {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl UnsetEngine {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let phandler = ProjectHandler::new(context.io());
        phandler.unset_project_engine(&self.path)?;
        let info = get_project_info_or_exit(context, &self.path)?;

        write_stdout!(
            context.io(),
            "Engine deassociated from project {}.\n",
            info.get_versioned_name().color("green")
        )?;

        Ok(())
    }
}
