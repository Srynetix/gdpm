use std::path::PathBuf;

use argh::FromArgs;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    engine::{get_default_engine, get_engine_version, run_engine_version_for_project},
    project::{get_project_info, set_project_engine, unset_project_engine},
};
use question::{Answer, Question};

use super::{print_missing_default_engine_message, print_missing_project_engine_message, Execute};

/// get project info
#[derive(FromArgs)]
#[argh(subcommand, name = "info")]
pub struct Info {
    /// project path
    #[argh(option, short = 'p', default = "PathBuf::from(\".\")")]
    path: PathBuf,
}

/// edit project
#[derive(FromArgs)]
#[argh(subcommand, name = "edit")]
pub struct Edit {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,

    /// version
    #[argh(option)]
    version: Option<String>,
}

/// set project engine
#[derive(FromArgs)]
#[argh(subcommand, name = "set-engine")]
pub struct SetEngine {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,

    /// version
    #[argh(option)]
    version: Option<String>,
}

/// unset project engine
#[derive(FromArgs)]
#[argh(subcommand, name = "unset-engine")]
pub struct UnsetEngine {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
}

impl Execute for Info {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;
        info.show();

        Ok(())
    }
}

impl Execute for Edit {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;

        if let Some(v) = self.version {
            // Use specific version (override)
            println!(
                "Running Godot Engine v{} for project {} ...",
                v.color("green"),
                info.get_versioned_name().color("green")
            );
            run_engine_version_for_project(&v, &self.path)?;
        } else {
            if let Some(e) = info.get_engine_version() {
                println!(
                    "Running Godot Engine v{} for project {} ...",
                    e.color("green"),
                    info.get_versioned_name().color("green")
                );
                run_engine_version_for_project(&e, &self.path)?;
            } else {
                if let Some(e) = get_default_engine()? {
                    print_missing_project_engine_message();
                    match Question::new(&format!(
                        "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                        e.color("green"),
                        info.get_versioned_name().color("green")
                    ))
                    .confirm()
                    {
                        Answer::YES => set_project_engine(&self.path, &e)?,
                        Answer::NO => println!("Okay. You will be asked again next time."),
                        _ => unreachable!(),
                    }

                    println!(
                        "Running Godot Engine v{} for project {} ...",
                        e.color("green"),
                        info.get_versioned_name().color("green")
                    );
                    run_engine_version_for_project(&e, &self.path)?;
                } else {
                    print_missing_project_engine_message();
                    print_missing_default_engine_message();
                }
            }
        }

        Ok(())
    }
}

impl Execute for SetEngine {
    fn execute(self) -> Result<()> {
        let info = get_project_info(&self.path)?;
        if let Some(v) = self.version {
            get_engine_version(&v)?;
            set_project_engine(&self.path, &v)?;
            println!(
                "Godot Engine v{} set for project {}.",
                v.color("green"),
                info.get_versioned_name().color("green")
            );
        } else {
            if let Some(e) = get_default_engine()? {
                set_project_engine(&self.path, &e)?;
                println!(
                    "Godot Engine v{} set for project {}.",
                    e.color("green"),
                    info.get_versioned_name().color("green")
                );
            } else {
                print_missing_default_engine_message();
            }
        }

        Ok(())
    }
}

impl Execute for UnsetEngine {
    fn execute(self) -> Result<()> {
        unset_project_engine(&self.path)?;
        let info = get_project_info(&self.path)?;

        println!(
            "Engine deassociated from project {}.",
            info.get_versioned_name().color("green")
        );

        Ok(())
    }
}
