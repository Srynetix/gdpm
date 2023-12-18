use std::path::Path;

use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    engine::{EngineHandler, EngineInfo},
    io::IoAdapter,
    project::{GdProjectInfo, ProjectHandler},
    types::version::{GodotVersion, SystemVersion},
};
use question::{Answer, Question};

pub enum CheckEngineResponse {
    Found(EngineInfo),
    UseDefault(EngineInfo),
    Download(GodotVersion),
    Abort,
}

pub(crate) fn print_missing_default_engine_message() {
    println!(
        "{}",
        "No default engine registered. Use `engine default <version>` to register one."
            .color("yellow")
    );
}

pub(crate) fn print_missing_project_engine_message() {
    println!(
        "{}",
        "You have no engine version associated to your project.".color("yellow")
    );
}

pub(crate) fn get_project_info_or_exit<I: IoAdapter>(io_adapter: &I, p: &Path) -> GdProjectInfo {
    let phandler = ProjectHandler::new(io_adapter);
    match phandler.get_project_info(p) {
        Ok(info) => info,
        Err(_) => {
            if p.to_str() == Some(".") {
                println!(
                    "{}",
                    "Godot project not found at current path.".color("yellow"),
                );
            } else {
                println!(
                    "{} `{}`.",
                    "Godot project not found at path".color("yellow"),
                    p.display(),
                );
            }
            std::process::exit(1);
        }
    }
}

pub(crate) fn check_engine_version_or_ask_default<I: IoAdapter>(
    io_adapter: &I,
    version: &GodotVersion,
) -> Result<CheckEngineResponse> {
    let ehandler = EngineHandler::new(io_adapter);
    match ehandler.get_version(version) {
        Ok(v) => Ok(CheckEngineResponse::Found(v)),
        Err(_) => {
            println!("Your project is associated with engine '{version}', which is not installed.");
            if let Answer::YES = Question::new(&format!("Do you want to download and install engine version '{version}' using official repositories? [y/n]")).confirm() {
                return Ok(CheckEngineResponse::Download(version.to_owned()))
            }

            if let Some(v) = ehandler.get_default()? {
                if let Answer::YES = Question::new(&format!(
                    "Do you want to use the default engine version instead? (version '{v}') [y/n]"
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

pub(crate) fn validate_engine_version_or_exit<I: IoAdapter>(
    io_adapter: &I,
    version: &GodotVersion,
) -> Result<EngineInfo> {
    let ehandler = EngineHandler::new(io_adapter);
    match ehandler.get_version(version) {
        Ok(v) => Ok(v),
        Err(_) => {
            let available_engines = ehandler.list()?;
            let available_engine_names: Vec<String> = available_engines
                .into_iter()
                .map(|x| format!("- {}", x.get_verbose_name().color("green")))
                .collect();

            println!("{}", format!("Unknown engine with version `{}`. You need to `engine register` this version before using it.", version.to_string().color("green")).color("yellow"));

            if available_engine_names.is_empty() {
                println!("{}", "No engine registered.".color("yellow"));
            } else {
                let list = format!("Available engines:\n{}", available_engine_names.join("\n"))
                    .color("yellow");
                println!("{}", list);
            }

            std::process::exit(1);
        }
    }
}

pub(crate) fn parse_godot_version_args(
    version: &GodotVersion,
    headless: bool,
    server: bool,
) -> (GodotVersion, SystemVersion) {
    let system = SystemVersion::determine_system_kind();

    if !system.is_linux() && headless {
        println!(
            "{}",
            "You can not install an headless version of Godot Engine on a non-Linux platform."
                .color("red")
        );
        std::process::exit(1);
    } else if !system.is_linux() && server {
        println!(
            "{}",
            "You can not install an server version of Godot Engine on a non-Linux platform."
                .color("red")
        );
        std::process::exit(1);
    }

    (version.clone(), system)
}
