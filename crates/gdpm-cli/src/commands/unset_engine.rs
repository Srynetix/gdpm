use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter, project::ProjectHandler};

use crate::common::get_project_info_or_exit;

use super::Execute;

#[derive(Parser)]
pub struct UnsetEngine {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl Execute for UnsetEngine {
    fn execute<I: IoAdapter, D: DownloadAdapter>(
        self,
        context: crate::context::Context<I, D>,
    ) -> Result<()> {
        let phandler = ProjectHandler::new(context.io());
        phandler.unset_project_engine(&self.path)?;
        let info = get_project_info_or_exit(context.io(), &self.path);

        println!(
            "Engine deassociated from project {}.",
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}
