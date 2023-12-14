use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter, plugins::DependencyHandler};

use crate::common::get_project_info_or_exit;

use super::Execute;

#[derive(Parser)]
pub struct Info {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl Execute for Info {
    fn execute<I: IoAdapter, D: DownloadAdapter>(
        self,
        context: crate::context::Context<I, D>,
    ) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        info.show();

        let dhandler = DependencyHandler::new(context.io());
        let dependencies = dhandler.list_project_dependencies(&self.path)?;
        if dependencies.is_empty() {
            println!(
                "Project '{}' has no dependency.",
                info.get_versioned_name().color("green")
            );
        } else {
            println!(
                "Dependencies from project '{}':",
                info.get_versioned_name().color("green")
            );

            for dep in dependencies {
                println!("- {}", dep.get_verbose_name());
            }
        }

        Ok(())
    }
}
