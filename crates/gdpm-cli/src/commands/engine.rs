use std::path::{Path, PathBuf};

use argh::FromArgs;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::engine::{
    exec_engine_version_command_for_project, get_default_engine, list_engines_info,
    register_engine_entry, run_engine_version_for_project, set_default_engine,
    unregister_engine_entry, EngineInfo,
};

use super::Execute;
use crate::common::{print_missing_default_engine_message, validate_engine_version_or_exit};

/// engine management
#[derive(FromArgs)]
#[argh(subcommand, name = "engine")]
pub struct Engine {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    List(List),
    Register(Register),
    Unregister(Unregister),
    Start(Start),
    Cmd(Cmd),
    SetDefault(SetDefault),
    GetDefault(GetDefault),
}

/// list engines
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {}

/// register engine
#[derive(FromArgs)]
#[argh(subcommand, name = "register")]
pub struct Register {
    /// version
    #[argh(positional)]
    version: String,
    /// engine path
    #[argh(positional)]
    path: PathBuf,
    /// mono edition?
    #[argh(switch)]
    mono: bool,
    /// built from source?
    #[argh(switch)]
    source: bool,
}

/// unregister engine
#[derive(FromArgs)]
#[argh(subcommand, name = "unregister")]
pub struct Unregister {
    /// version
    #[argh(positional)]
    version: String,
}

/// start engine
#[derive(FromArgs)]
#[argh(subcommand, name = "start")]
pub struct Start {
    /// version
    #[argh(option, short = 'v')]
    version: Option<String>,
}

/// execute command on engine
#[derive(FromArgs)]
#[argh(subcommand, name = "cmd")]
pub struct Cmd {
    /// version
    #[argh(option, short = 'v')]
    version: Option<String>,
    /// arguments
    #[argh(positional)]
    args: Vec<String>,
}

/// set engine as default
#[derive(FromArgs)]
#[argh(subcommand, name = "set-default")]
pub struct SetDefault {
    /// version
    #[argh(positional)]
    version: String,
}

/// get default engine
#[derive(FromArgs)]
#[argh(subcommand, name = "get-default")]
pub struct GetDefault {}

impl Execute for Engine {
    fn execute(self) -> Result<()> {
        self.cmd.execute()
    }
}

impl Execute for Command {
    fn execute(self) -> Result<()> {
        match self {
            Self::List(c) => c.execute(),
            Self::Register(c) => c.execute(),
            Self::Unregister(c) => c.execute(),
            Self::Start(c) => c.execute(),
            Self::Cmd(c) => c.execute(),
            Self::SetDefault(c) => c.execute(),
            Self::GetDefault(c) => c.execute(),
        }
    }
}

impl Execute for List {
    fn execute(self) -> Result<()> {
        let entries = list_engines_info()?;
        let default_entry = get_default_engine()?;

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

impl Execute for Register {
    fn execute(self) -> Result<()> {
        let engine_info = EngineInfo::new(self.version, self.path, self.mono, self.source)?;
        let verbose_name = engine_info.get_verbose_name();
        register_engine_entry(engine_info)?;

        println!("{} is registered.", verbose_name);
        Ok(())
    }
}

impl Execute for Unregister {
    fn execute(self) -> Result<()> {
        validate_engine_version_or_exit(&self.version)?;
        unregister_engine_entry(&self.version)?;

        println!(
            "Godot Engine v{} unregistered.",
            self.version.color("green")
        );
        Ok(())
    }
}

impl Execute for Start {
    fn execute(self) -> Result<()> {
        if let Some(v) = self.version {
            validate_engine_version_or_exit(&v)?;
            println!("Running Godot Engine v{} ...", v.color("green"));
            run_engine_version_for_project(&v, Path::new("."))?;
        } else if let Some(e) = get_default_engine()? {
            println!("Running Godot Engine v{} ...", e.color("green"));
            run_engine_version_for_project(&e, Path::new("."))?;
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for Cmd {
    fn execute(self) -> Result<()> {
        if self.args.is_empty() {
            println!("{}", "You need to pass arguments. If you only want to start the engine, use `engine start`.".color("yellow"));
        } else if let Some(v) = self.version {
            validate_engine_version_or_exit(&v)?;
            println!(
                "Executing command {} Godot Engine v{} ...",
                self.args.join(" ").color("blue"),
                v.color("green")
            );
            exec_engine_version_command_for_project(&v, &self.args, Path::new("."))?;
        } else if let Some(e) = get_default_engine()? {
            println!(
                "Executing command {} on Godot Engine v{} ...",
                self.args.join(" ").color("blue"),
                e.color("green")
            );
            exec_engine_version_command_for_project(&e, &self.args, Path::new("."))?;
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for SetDefault {
    fn execute(self) -> Result<()> {
        validate_engine_version_or_exit(&self.version)?;
        set_default_engine(&self.version)?;
        println!(
            "Godot Engine v{} set as default.",
            self.version.color("green")
        );

        Ok(())
    }
}

impl Execute for GetDefault {
    fn execute(self) -> Result<()> {
        if let Some(e) = get_default_engine()? {
            println!("{} {}", "*".color("green"), e.color("green"));
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}
