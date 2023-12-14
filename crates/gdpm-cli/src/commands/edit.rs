use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter, engine::EngineHandler, io::IoAdapter, project::ProjectHandler,
};
use question::{Answer, Question};

use crate::common::{
    get_project_info_or_exit, print_missing_default_engine_message,
    print_missing_project_engine_message, validate_engine_version_or_exit,
};

use super::Execute;

#[derive(Parser)]
pub struct Edit {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// Engine version
    #[clap(short, long)]
    engine: Option<String>,
}

impl Execute for Edit {
    fn execute<I: IoAdapter, D: DownloadAdapter>(
        self,
        context: crate::context::Context<I, D>,
    ) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let ehandler = EngineHandler::new(context.io());
        let phandler = ProjectHandler::new(context.io());

        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?;
            println!(
                "Running Godot Engine v{} for project {} ...",
                v.color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(&v, &self.path)?;
        } else if let Some(e) = info.get_engine_version() {
            println!(
                "Running Godot Engine v{} for project {} ...",
                e.color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(e, &self.path)?;
        } else if let Some(e) = ehandler.get_default()? {
            print_missing_project_engine_message();
            match Question::new(&format!(
                "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                e.color("green"),
                info.get_versioned_name().color("green")
            ))
            .confirm()
            {
                Answer::YES => phandler.set_project_engine(&self.path, &e)?,
                Answer::NO => println!("Okay. You will be asked again next time."),
                _ => unreachable!(),
            }

            println!(
                "Running Godot Engine v{} for project {} ...",
                e.color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(&e, &self.path)?;
        } else {
            print_missing_project_engine_message();
            print_missing_default_engine_message();
        }

        Ok(())
    }
}
