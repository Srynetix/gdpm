use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::{download::Downloader, error::DownloadError, DownloadAdapter},
    engine::{EngineHandler, EngineInfo},
    error::EngineError,
    io::IoAdapter,
    types::version::{GodotVersion, SystemVersion},
};
use tracing::info;

use super::Execute;
use crate::{
    common::{print_missing_default_engine_message, validate_engine_version_or_exit},
    context::Context,
};

const MIRROR_URL: &str = "https://downloads.tuxfamily.org/godotengine/";

#[derive(Parser)]
pub struct Engine {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    List(List),
    Run(Run),
    Default(Default),
    Add(Add),
    Remove(Remove),
}

/// List engines
#[derive(Parser)]
#[clap(name = "list")]
pub struct List;

/// Run command on engine
#[derive(Parser)]
pub struct Run {
    /// Engine version
    #[clap(short, long)]
    engine: Option<String>,
    /// Arguments
    args: Vec<String>,
}

/// Show or set default engine
#[derive(Parser)]
pub struct Default {
    /// Engine verion
    engine: Option<String>,
}

/// Download and install engine from official mirror or specific URL / path (e.g. 3.3.4, 3.3.4.mono, 3.5.rc1, 3.5.rc1.mono)
#[derive(Parser)]
pub struct Add {
    /// Engine version
    engine: String,
    /// Headless?
    #[clap(long)]
    headless: bool,
    /// Server?
    #[clap(long)]
    server: bool,
    /// Target URL
    #[clap(long)]
    target_url: Option<String>,
    /// Target path
    #[clap(long)]
    target_path: Option<PathBuf>,
    /// Allow overwrite
    #[clap(long)]
    overwrite: bool,
}

/// Uninstall engine
#[derive(Parser)]
pub struct Remove {
    /// Engine version
    version: String,
    /// Headless?
    #[clap(long)]
    headless: bool,
    /// Server?
    #[clap(long)]
    server: bool,
}

impl Execute for Engine {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        self.cmd.execute(context)
    }
}

impl Execute for Command {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        match self {
            Self::List(c) => c.execute(context),
            Self::Run(c) => c.execute(context),
            Self::Default(c) => c.execute(context),
            Self::Add(c) => c.execute(context),
            Self::Remove(c) => c.execute(context),
        }
    }
}

impl Execute for List {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
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

impl Execute for Run {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?;

            if self.args.is_empty() {
                println!("Running Godot Engine v{} ...", v.color("green"));
                ehandler.run_version_for_project(&v, Path::new("."))?;
            } else {
                println!(
                    "Executing command {} Godot Engine v{} ...",
                    self.args.join(" ").color("blue"),
                    v.color("green")
                );
                ehandler.exec_version_for_project(&v, &self.args, Path::new("."))?;
            }
        } else if let Some(e) = ehandler.get_default()? {
            if self.args.is_empty() {
                println!(
                    "Executing command {} on Godot Engine v{} ...",
                    self.args.join(" ").color("blue"),
                    e.color("green")
                );
                ehandler.exec_version_for_project(&e, &self.args, Path::new("."))?;
            } else {
                println!("Running Godot Engine v{} ...", e.color("green"));
                ehandler.run_version_for_project(&e, Path::new("."))?;
            }
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for Default {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        if let Some(version) = self.engine {
            validate_engine_version_or_exit(context.io(), &version)?;
            let ehandler = EngineHandler::new(context.io());
            ehandler.set_as_default(&version)?;
            println!("Godot Engine v{} set as default.", version.color("green"));
        } else {
            let ehandler = EngineHandler::new(context.io());
            if let Some(e) = ehandler.get_default()? {
                println!("{} {}", "*".color("green"), e.color("green"));
            } else {
                print_missing_default_engine_message();
            }
        }

        Ok(())
    }
}

impl Add {
    async fn download_file_at_url<I: IoAdapter, D: DownloadAdapter>(
        context: &Context<I, D>,
        url: &str,
        version: GodotVersion,
        system: SystemVersion,
    ) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());

        match Downloader::download_file_at_url(context.download(), url).await {
            Ok(c) => {
                let path =
                    ehandler.install_from_official_zip(c, version.clone(), system.clone())?;
                println!(
                    "{}",
                    format!(
                        "Version '{}' installed for system '{}' at path '{}'",
                        version,
                        system,
                        path.display()
                    )
                    .color("green")
                );
            }
            Err(DownloadError::NotFound(u)) => {
                println!(
                    "{}",
                    format!(
                        "Version '{}' does not exist for system '{}' (or wrong url: {})",
                        version, system, u
                    )
                    .color("red")
                );
            }
            Err(e) => {
                println!(
                    "{}",
                    format!(
                        "Unexpected error while trying to download file at url '{}'\n    | {}",
                        url, e
                    )
                    .color("red")
                )
            }
        }

        Ok(())
    }
}

impl Execute for Add {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let (version, system) = parse_godot_version_args(&self.engine, self.headless, self.server);

        let version_name = format!("{}", version);
        let existing_version = ehandler.has_version(&version_name)?;
        if existing_version.is_some() {
            if !self.overwrite {
                println!("{}",
                    format!("Engine version '{}' is already installed. Use '--overwrite' to force installation.", version_name).color("yellow")
                );
                std::process::exit(1);
            } else {
                info!(
                    "Will overwrite existing engine version '{}'.",
                    version_name.color("green")
                );
            }
        }

        if let Some(path) = self.target_path {
            let engine_info = EngineInfo::new(context.io(), self.engine, path)?;
            let verbose_name = engine_info.get_verbose_name();
            let ehandler = EngineHandler::new(context.io());
            ehandler.register(engine_info)?;

            println!("{} is registered.", verbose_name);
            return Ok(());
        }

        let url = self.target_url.unwrap_or_else(|| {
            Downloader::get_official_url_for_version(version.clone(), system.clone(), MIRROR_URL)
        });

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(Self::download_file_at_url(&context, &url, version, system))?;

        Ok(())
    }
}

impl Execute for Remove {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let (version, _system) =
            parse_godot_version_args(&self.version, self.headless, self.server);

        let ehandler = EngineHandler::new(context.io());
        let version_name = format!("{}", version);
        match ehandler.uninstall(version) {
            Ok(()) => println!(
                "{}",
                format!(
                    "Engine version '{}' was successfully uninstalled.",
                    version_name
                )
                .color("green")
            ),
            Err(e) => match e {
                EngineError::EngineNotFound(_) => {
                    println!(
                        "{}",
                        format!("Unknown engine version '{}'.", version_name).color("red")
                    );
                    std::process::exit(1);
                }
                EngineError::EngineNotInstalled(_) => {
                    ehandler.unregister(&version_name)?;
                }
                e => return Err(e.into()),
            },
        }

        Ok(())
    }
}

pub fn parse_godot_version_args(
    version: &str,
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

    (GodotVersion::from_str(version).unwrap(), system)
}
