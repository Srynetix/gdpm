use std::path::Path;

use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    engine::{get_engine_version, list_engines_info, EngineInfo},
    project::{get_project_info, GdProjectInfo},
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

pub(crate) fn get_project_info_or_exit(p: &Path) -> GdProjectInfo {
    match get_project_info(p) {
        Ok(info) => info,
        Err(_) => {
            println!(
                "{} `{}`.",
                "Godot project not found at path".color("yellow"),
                p.display(),
            );
            std::process::exit(1);
        }
    }
}

pub(crate) fn validate_engine_version_or_exit(version: &str) -> Result<EngineInfo> {
    match get_engine_version(version) {
        Ok(v) => Ok(v),
        Err(_) => {
            let available_engines = list_engines_info()?;
            let available_engine_names: Vec<String> = available_engines
                .into_iter()
                .map(|x| format!("- {}", x.get_verbose_name().color("green")))
                .collect();

            let msg = format!("Unknown engine with version `{}`. You need to `engine register` this version before using it.", version.color("green")).color("yellow");
            println!("{}", msg);

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
