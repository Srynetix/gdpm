use std::path::Path;

use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    engine::{EngineHandler, EngineInfo},
    io::IoAdapter,
    project::{GdProjectInfo, ProjectHandler},
};

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

pub(crate) fn validate_engine_version_or_exit<I: IoAdapter>(
    io_adapter: &I,
    version: &str,
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

            println!("{}", format!("Unknown engine with version `{}`. You need to `engine register` this version before using it.", version.color("green")).color("yellow"));

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
