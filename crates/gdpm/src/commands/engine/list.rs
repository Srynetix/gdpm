use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::EngineHandler,
    io::{write_stdout, IoAdapter},
};

use crate::context::Context;

/// List engines
#[derive(Parser)]
#[clap(name = "list", alias = "ls")]
pub struct List;

impl List {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let entries = ehandler.list()?;
        let default_entry = ehandler.get_default()?;

        if entries.is_empty() {
            write_stdout!(
                context.io(),
                "{}\n",
                "No engine registered. Use `engine add` or `engine register` to register an engine."
                    .color("yellow")
            )?;
        } else {
            for entry in entries {
                if let Some(default) = &default_entry {
                    if entry.has_same_slug(default) {
                        write_stdout!(context.io(), "{} ", "*".color("green"))?;
                    } else {
                        write_stdout!(context.io(), "  ")?;
                    }
                } else {
                    write_stdout!(context.io(), "  ")?;
                }

                write_stdout!(context.io(), "{}\n", entry.get_verbose_name())?;
            }
        }

        Ok(())
    }
}
