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
pub struct Info {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl Info {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context, &self.path)?;
        info.write_repr(context.io())?;

        let dhandler = DependencyHandler::new(context.io());
        let dependencies = dhandler.list_project_dependencies(&self.path)?;
        if dependencies.is_empty() {
            write_stdout!(
                context.io(),
                "Project '{}' has no dependency.\n",
                info.get_versioned_name().color("green")
            )?;
        } else {
            write_stdout!(
                context.io(),
                "Dependencies from project '{}':\n",
                info.get_versioned_name().color("green")
            )?;

            for dep in dependencies {
                write_stdout!(context.io(), "- {}\n", dep.get_verbose_name())?;
            }
        }

        Ok(())
    }
}
