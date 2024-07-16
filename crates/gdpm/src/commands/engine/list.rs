use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{downloader::DownloadAdapter, engine::EngineHandler, io::IoAdapter};

use crate::context::Context;

/// List engines
#[derive(Parser)]
#[clap(name = "list")]
pub struct List;

impl List {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let entries = ehandler.list()?;
        let default_entry = ehandler.get_default()?;

        if entries.is_empty() {
            println!(
                "{}",
                "No engine registered. Use `engine register` to register an engine."
                    .color("yellow")
            );
        } else {
            for entry in entries {
                if let Some(default) = &default_entry {
                    if entry.has_same_slug(default) {
                        print!("{} ", "*".color("green"));
                    } else {
                        print!("  ");
                    }
                } else {
                    print!("  ");
                }

                println!("{}", entry.get_verbose_name());
            }
        }

        Ok(())
    }
}
