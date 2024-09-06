use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;

use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::EngineHandler,
    io::{write_stderr, write_stdout, IoAdapter},
    project::ProjectHandler,
    types::version::GodotVersion,
};
use question::{Answer, Question};

use crate::{
    commands::engine::add::Add,
    common::{
        check_engine_version_or_ask_default, get_project_info_or_exit,
        print_missing_default_engine_message, print_missing_project_engine_message,
        validate_engine_version_or_exit, CheckEngineResponse,
    },
    context::Context,
};

#[derive(Parser)]
pub struct Edit {
    /// Project path
    #[clap(short, long, default_value = ".")]
    path: PathBuf,

    /// Engine version
    #[clap(short, long)]
    engine: Option<GodotVersion>,
}

impl Edit {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let info = get_project_info_or_exit(context, &self.path)?;
        let ehandler = EngineHandler::new(context.io());
        let phandler = ProjectHandler::new(context.io());

        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context, &v)?;
            write_stdout!(
                context.io(),
                "Running Godot Engine v{} for project {} ...\n",
                v.to_string().color("green"),
                info.get_versioned_name().color("green")
            )?;
            ehandler.run_version_for_project(&v, &self.path)?;
        } else if let Some(e) = info.get_engine_version() {
            let engine_response = check_engine_version_or_ask_default(context, e)?;
            let engine_version = match engine_response {
                CheckEngineResponse::Found(v) => v,
                CheckEngineResponse::UseDefault(v) => v,
                CheckEngineResponse::Abort => {
                    write_stderr!(context.io(), "Aborting.\n")?;
                    std::process::exit(1);
                }
                CheckEngineResponse::Download(v) => {
                    let cmd = Add {
                        engine: v.clone(),
                        overwrite: false,
                        system_version: None,
                        target_path: None,
                        target_url: None,
                    };

                    cmd.execute(context)?;
                    ehandler.get_version(&v)?
                }
            };

            write_stdout!(
                context.io(),
                "Running Godot Engine v{} for project {} ...\n",
                engine_version.get_name(),
                info.get_versioned_name().color("green")
            )?;
            ehandler.run_version_for_project(&engine_version.version, &self.path)?;
        } else if let Some(e) = ehandler.get_default()? {
            print_missing_project_engine_message(context)?;
            match Question::new(&format!(
                "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                e.to_string().color("green"),
                info.get_versioned_name().color("green")
            ))
            .confirm()
            {
                Answer::YES => phandler.set_project_engine(&self.path, &e)?,
                Answer::NO => {
                    write_stdout!(context.io(), "Okay. You will be asked again next time.\n")?
                }
                _ => unreachable!(),
            }

            write_stdout!(
                context.io(),
                "Running Godot Engine v{} for project {} ...\n",
                e.to_string().color("green"),
                info.get_versioned_name().color("green")
            )?;
            ehandler.run_version_for_project(&e, &self.path)?;
        } else {
            print_missing_project_engine_message(context)?;
            print_missing_default_engine_message(context)?;
        }

        Ok(())
    }
}
