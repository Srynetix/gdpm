use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter, engine::EngineHandler, io::IoAdapter, project::ProjectHandler,
    types::version::GodotVersion,
};
use question::{Answer, Question};

use crate::{
    commands::engine,
    common::{
        check_engine_version_or_ask_default, get_project_info_or_exit,
        print_missing_default_engine_message, print_missing_project_engine_message,
        validate_engine_version_or_exit, CheckEngineResponse,
    },
    context::Context,
};

use super::Execute;

#[derive(Parser)]
pub struct Edit {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// Engine version
    #[clap(short, long)]
    engine: Option<GodotVersion>,
}

impl Execute for Edit {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context.io(), &self.path);
        let ehandler = EngineHandler::new(context.io());
        let phandler = ProjectHandler::new(context.io());

        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?;
            println!(
                "Running Godot Engine v{} for project {} ...",
                v.to_string().color("green"),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(&v, &self.path)?;
        } else if let Some(e) = info.get_engine_version() {
            let engine_response = check_engine_version_or_ask_default(context.io(), e)?;
            let engine_version = match engine_response {
                CheckEngineResponse::Found(v) => v,
                CheckEngineResponse::UseDefault(v) => v,
                CheckEngineResponse::Abort => {
                    println!("Aborting.");
                    std::process::exit(1);
                }
                CheckEngineResponse::Download(v) => {
                    let cmd = engine::Add {
                        engine: v.clone(),
                        headless: false,
                        overwrite: false,
                        server: false,
                        target_path: None,
                        target_url: None,
                    };

                    cmd.execute(context)?;
                    ehandler.get_version(&v)?
                }
            };

            println!(
                "Running Godot Engine v{} for project {} ...",
                engine_version.get_name(),
                info.get_versioned_name().color("green")
            );
            ehandler.run_version_for_project(&engine_version.version, &self.path)?;
        } else if let Some(e) = ehandler.get_default()? {
            print_missing_project_engine_message();
            match Question::new(&format!(
                "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                e.to_string().color("green"),
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
                e.to_string().color("green"),
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
