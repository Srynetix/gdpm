use std::path::Path;

use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::DownloadAdapter,
    engine::{EngineHandler, EngineInfo},
    io::{write_stdout, IoAdapter},
    project::{GdProjectInfo, ProjectHandler},
    types::version::GodotVersion,
};
use question::{Answer, Question};

use crate::context::Context;

pub enum CheckEngineResponse {
    Found(EngineInfo),
    UseDefault(EngineInfo),
    Download(GodotVersion),
    Abort,
}

pub(crate) fn print_missing_default_engine_message<I: IoAdapter, D: DownloadAdapter>(
    context: &Context<I, D>,
) -> Result<()> {
    write_stdout!(
        context.io(),
        "{}",
        "No default engine registered. Use `engine default <version>` to register one.\n"
            .color("yellow")
    )?;
    Ok(())
}

pub(crate) fn print_missing_project_engine_message<I: IoAdapter, D: DownloadAdapter>(
    context: &Context<I, D>,
) -> Result<()> {
    write_stdout!(
        context.io(),
        "{}",
        "You have no engine version associated to your project.\n".color("yellow")
    )?;
    Ok(())
}

pub(crate) fn get_project_info_or_exit<I: IoAdapter, D: DownloadAdapter>(
    context: &Context<I, D>,
    project_path: &Path,
) -> Result<GdProjectInfo> {
    let phandler = ProjectHandler::new(context.io());
    match phandler.get_project_info(project_path) {
        Ok(info) => Ok(info),
        Err(_) => {
            if project_path.to_str() == Some(".") {
                write_stdout!(
                    context.io(),
                    "{}",
                    "Godot project not found at current path.\n".color("yellow")
                )?;
            } else {
                write_stdout!(
                    context.io(),
                    "{} `{}`.",
                    "Godot project not found at path\n".color("yellow"),
                    project_path.display()
                )?;
            }
            std::process::exit(1);
        }
    }
}

pub(crate) fn check_engine_version_or_ask_default<I: IoAdapter, D: DownloadAdapter>(
    context: &Context<I, D>,
    version: &GodotVersion,
) -> Result<CheckEngineResponse> {
    let ehandler = EngineHandler::new(context.io());
    match ehandler.get_version(version) {
        Ok(v) => Ok(CheckEngineResponse::Found(v)),
        Err(_) => {
            write_stdout!(
                context.io(),
                "Your project is associated with engine '{}', which is not installed.\n",
                version.to_string().green()
            )?;
            if let Answer::YES = Question::new(&format!("Do you want to download and install engine version '{}' using official repositories? [y/n]", version.to_string().green())).confirm() {
                return Ok(CheckEngineResponse::Download(version.to_owned()))
            }

            if let Some(v) = ehandler.get_default()? {
                if let Answer::YES = Question::new(&format!(
                    "Do you want to use the default engine version instead? (version '{}') [y/n]",
                    v.to_string().green()
                ))
                .confirm()
                {
                    return Ok(CheckEngineResponse::UseDefault(
                        ehandler.get_version(&v).unwrap(),
                    ));
                }
            }

            Ok(CheckEngineResponse::Abort)
        }
    }
}

pub(crate) fn validate_engine_version_or_exit<I: IoAdapter, D: DownloadAdapter>(
    context: &Context<I, D>,
    version: &GodotVersion,
) -> Result<EngineInfo> {
    let ehandler = EngineHandler::new(context.io());
    match ehandler.get_version(version) {
        Ok(v) => Ok(v),
        Err(_) => {
            let available_engines = ehandler.list()?;
            let available_engine_names: Vec<String> = available_engines
                .into_iter()
                .map(|x| format!("- {}", x.get_verbose_name().color("green")))
                .collect();

            write_stdout!(context.io(), "{}", format!("Unknown engine with version `{}`. You need to `engine add` or `engine register` this version before using it.\n", version.to_string().color("green")).color("yellow"))?;

            if available_engine_names.is_empty() {
                write_stdout!(
                    context.io(),
                    "{}",
                    "No engine registered.\n".color("yellow")
                )?;
            } else {
                let list = format!("Available engines:\n{}", available_engine_names.join("\n"))
                    .color("yellow");
                write_stdout!(context.io(), "{}\n", list)?;
            }

            std::process::exit(1);
        }
    }
}
