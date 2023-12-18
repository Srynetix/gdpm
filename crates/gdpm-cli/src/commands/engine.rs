use std::path::{Path, PathBuf};

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
    common::{
        parse_godot_version_args, print_missing_default_engine_message,
        validate_engine_version_or_exit,
    },
    context::Context,
};

pub(crate) const MIRROR_URL: &str = "https://downloads.tuxfamily.org/godotengine/";

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
    engine: Option<GodotVersion>,
    /// Arguments
    args: Vec<String>,
}

/// Show or set default engine
#[derive(Parser)]
pub struct Default {
    /// Engine verion
    engine: Option<GodotVersion>,
}

/// Download and install engine from official mirror or specific URL / path (e.g. 3.3.4, 3.3.4.mono, 3.5.rc1, 3.5.rc1.mono)
#[derive(Parser)]
pub(crate) struct Add {
    /// Engine version
    pub(crate) engine: GodotVersion,
    /// Headless?
    #[clap(long)]
    pub(crate) headless: bool,
    /// Server?
    #[clap(long)]
    pub(crate) server: bool,
    /// Target URL
    #[clap(long)]
    pub(crate) target_url: Option<String>,
    /// Target path
    #[clap(long)]
    pub(crate) target_path: Option<PathBuf>,
    /// Allow overwrite
    #[clap(long)]
    pub(crate) overwrite: bool,
}

/// Uninstall engine
#[derive(Parser)]
pub struct Remove {
    /// Engine version
    engine: GodotVersion,
    /// Headless?
    #[clap(long)]
    headless: bool,
    /// Server?
    #[clap(long)]
    server: bool,
}

impl Execute for Engine {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        self.cmd.execute(context)
    }
}

impl Execute for Command {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
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
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
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
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if let Some(v) = self.engine {
            validate_engine_version_or_exit(context.io(), &v)?;

            if self.args.is_empty() {
                println!("Running Godot Engine v{} ...", v.to_string().color("green"));
                ehandler.run_version_for_project(&v, Path::new("."))?;
            } else {
                println!(
                    "Executing command {} Godot Engine v{} ...",
                    self.args.join(" ").color("blue"),
                    v.to_string().color("green")
                );
                ehandler.exec_version_for_project(&v, &self.args, Path::new("."))?;
            }
        } else if let Some(e) = ehandler.get_default()? {
            if self.args.is_empty() {
                println!(
                    "Executing command {} on Godot Engine v{} ...",
                    self.args.join(" ").color("blue"),
                    e.to_string().color("green")
                );
                ehandler.exec_version_for_project(&e, &self.args, Path::new("."))?;
            } else {
                println!("Running Godot Engine v{} ...", e.to_string().color("green"));
                ehandler.run_version_for_project(&e, Path::new("."))?;
            }
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for Default {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        if let Some(version) = self.engine {
            validate_engine_version_or_exit(context.io(), &version)?;
            let ehandler = EngineHandler::new(context.io());
            ehandler.set_as_default(&version)?;
            println!(
                "Godot Engine v{} set as default.",
                version.to_string().color("green")
            );
        } else {
            let ehandler = EngineHandler::new(context.io());
            if let Some(e) = ehandler.get_default()? {
                println!("{} {}", "*".color("green"), e.to_string().color("green"));
            } else {
                print_missing_default_engine_message();
            }
        }

        Ok(())
    }
}

impl Add {
    pub(crate) async fn download_file_at_url<I: IoAdapter, D: DownloadAdapter>(
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

    pub(crate) async fn download_and_install_export_templates<I: IoAdapter, D: DownloadAdapter>(
        context: &Context<I, D>,
        url: &str,
        version: GodotVersion,
    ) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());

        match Downloader::download_file_at_url(context.download(), url).await {
            Ok(c) => {
                let path = ehandler.install_export_templates(c, version.clone())?;
                println!(
                    "{}",
                    format!(
                        "Export templates for version '{}' installed at path '{}'",
                        version,
                        path.display()
                    )
                    .color("green")
                );
            }
            Err(DownloadError::NotFound(u)) => {
                println!(
                    "{}",
                    format!(
                        "Export templates for version '{}' does not exist (or wrong url: {})",
                        version, u
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
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let (version, system) = parse_godot_version_args(&self.engine, self.headless, self.server);

        let existing_version = ehandler.has_version(&version)?;
        if existing_version.is_some() {
            if !self.overwrite {
                println!("{}",
                    format!("Engine version '{}' is already installed. Use '--overwrite' to force installation.", version).color("yellow")
                );
                std::process::exit(1);
            } else {
                info!(
                    "Will overwrite existing engine version '{}'.",
                    version.to_string().color("green")
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

        if let Some(url) = self.target_url {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(Self::download_file_at_url(context, &url, version, system))?;

            println!("Cannot fetch export templates, missing URL.");
        } else {
            let editor_url = Downloader::get_official_editor_url_for_version(
                version.clone(),
                system.clone(),
                MIRROR_URL,
            );
            let templates_url = Downloader::get_official_export_templates_url_for_version(
                version.clone(),
                MIRROR_URL,
            );

            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(Self::download_file_at_url(
                context,
                &editor_url,
                version.clone(),
                system,
            ))?;
            rt.block_on(Self::download_and_install_export_templates(
                context,
                &templates_url,
                version,
            ))?;
        }

        Ok(())
    }
}

impl Execute for Remove {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let (version, _system) = parse_godot_version_args(&self.engine, self.headless, self.server);

        let ehandler = EngineHandler::new(context.io());
        match ehandler.uninstall(&version) {
            Ok(()) => println!(
                "{}",
                format!("Engine version '{}' was successfully uninstalled.", version)
                    .color("green")
            ),
            Err(e) => match e {
                EngineError::EngineNotFound(_) => {
                    println!(
                        "{}",
                        format!("Unknown engine version '{}'.", version).color("red")
                    );
                    std::process::exit(1);
                }
                EngineError::EngineNotInstalled(_) => {
                    ehandler.unregister(&version)?;
                }
                e => return Err(e.into()),
            },
        }

        Ok(())
    }
}
