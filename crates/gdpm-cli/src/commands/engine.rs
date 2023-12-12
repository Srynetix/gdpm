use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::{download::Downloader, error::DownloadError, DownloadAdapter, GodotMirrorScanner},
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

/// engine management
#[derive(Parser)]
#[clap(name = "engine")]
pub struct Engine {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    List(List),
    ListRemote(ListRemote),
    Register(Register),
    Unregister(Unregister),
    Start(Start),
    Cmd(Cmd),
    SetDefault(SetDefault),
    GetDefault(GetDefault),
    Install(Install),
    Uninstall(Uninstall),
}

/// list engines
#[derive(Parser)]
#[clap(name = "list")]
pub struct List;

/// list available engines on official mirror
#[derive(Parser)]
#[clap(name = "list-remote")]
pub struct ListRemote {
    /// no cache
    #[clap(long)]
    no_cache: bool,
}

/// register engine
#[derive(Parser)]
#[clap(name = "register")]
pub struct Register {
    /// version
    version: String,
    /// engine path
    path: PathBuf,
    /// mono edition?
    #[clap(long)]
    mono: bool,
    /// built from source?
    #[clap(long)]
    source: bool,
}

/// unregister engine
#[derive(Parser)]
#[clap(name = "unregister")]
pub struct Unregister {
    /// version
    version: String,
}

/// start engine
#[derive(Parser)]
#[clap(name = "start")]
pub struct Start {
    /// version
    version: Option<String>,
}

/// execute command on engine
#[derive(Parser)]
#[clap(name = "cmd")]
pub struct Cmd {
    /// version
    #[clap(short, long)]
    version: Option<String>,
    /// arguments
    args: Vec<String>,
}

/// set engine as default
#[derive(Parser)]
#[clap(name = "set-default")]
pub struct SetDefault {
    /// version
    version: String,
}

/// get default engine
#[derive(Parser)]
#[clap(name = "get-default")]
pub struct GetDefault;

/// download and install engine from official mirror or specific URL (e.g. 3.3.4, 3.3.4.mono, 3.5.rc1, 3.5.rc1.mono)
#[derive(Parser)]
#[clap(name = "install")]
pub struct Install {
    /// version
    version: String,
    /// headless?
    #[clap(long)]
    headless: bool,
    /// server?
    #[clap(long)]
    server: bool,
    /// target URL
    #[clap(long)]
    target_url: Option<String>,
    /// allow overwrite
    #[clap(long)]
    overwrite: bool,
}

/// uninstall engine
#[derive(Parser)]
#[clap(name = "uninstall")]
pub struct Uninstall {
    /// version
    version: String,
    /// headless?
    #[clap(long)]
    headless: bool,
    /// server?
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
            Self::ListRemote(c) => c.execute(context),
            Self::Register(c) => c.execute(context),
            Self::Unregister(c) => c.execute(context),
            Self::Start(c) => c.execute(context),
            Self::Cmd(c) => c.execute(context),
            Self::SetDefault(c) => c.execute(context),
            Self::GetDefault(c) => c.execute(context),
            Self::Install(c) => c.execute(context),
            Self::Uninstall(c) => c.execute(context),
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

impl ListRemote {
    async fn list_remote_engines<I: IoAdapter, D: DownloadAdapter>(
        context: &Context<I, D>,
        no_cache: bool,
    ) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let scanner = GodotMirrorScanner::new(context.download());

        let should_scan = {
            if !no_cache {
                let cached_values = ehandler.read_versions_from_cache()?;
                if !cached_values.is_empty() {
                    info!("Reading versions from cache ...");
                    for version in cached_values {
                        println!("- {}", version);
                    }

                    false
                } else {
                    true
                }
            } else {
                true
            }
        };

        if should_scan {
            info!("Fetching remote versions ...");

            let versions = scanner.scan(MIRROR_URL).await?;
            for version in &versions {
                println!("- {}", version);
            }

            ehandler.write_versions_in_cache(versions)?;
        }

        Ok(())
    }
}

impl Execute for ListRemote {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(Self::list_remote_engines(&context, self.no_cache))?;

        Ok(())
    }
}

impl Execute for Register {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let engine_info = EngineInfo::new(
            context.io(),
            self.version,
            self.path,
            self.mono,
            self.source,
        )?;
        let verbose_name = engine_info.get_verbose_name();
        let ehandler = EngineHandler::new(context.io());
        ehandler.register(engine_info)?;

        println!("{} is registered.", verbose_name);
        Ok(())
    }
}

impl Execute for Unregister {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        validate_engine_version_or_exit(context.io(), &self.version)?;
        let ehandler = EngineHandler::new(context.io());
        ehandler.unregister(&self.version)?;

        println!(
            "Godot Engine v{} unregistered.",
            self.version.color("green")
        );
        Ok(())
    }
}

impl Execute for Start {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if let Some(v) = self.version {
            validate_engine_version_or_exit(context.io(), &v)?;
            println!("Running Godot Engine v{} ...", v.color("green"));
            ehandler.run_version_for_project(&v, Path::new("."))?;
        } else if let Some(e) = ehandler.get_default()? {
            println!("Running Godot Engine v{} ...", e.color("green"));
            ehandler.run_version_for_project(&e, Path::new("."))?;
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for Cmd {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if self.args.is_empty() {
            println!("{}", "You need to pass arguments. If you only want to start the engine, use `engine start`.".color("yellow"));
        } else if let Some(v) = self.version {
            validate_engine_version_or_exit(context.io(), &v)?;
            println!(
                "Executing command {} Godot Engine v{} ...",
                self.args.join(" ").color("blue"),
                v.color("green")
            );
            ehandler.exec_version_for_project(&v, &self.args, Path::new("."))?;
        } else if let Some(e) = ehandler.get_default()? {
            println!(
                "Executing command {} on Godot Engine v{} ...",
                self.args.join(" ").color("blue"),
                e.color("green")
            );
            ehandler.exec_version_for_project(&e, &self.args, Path::new("."))?;
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Execute for SetDefault {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        validate_engine_version_or_exit(context.io(), &self.version)?;
        let ehandler = EngineHandler::new(context.io());
        ehandler.set_as_default(&self.version)?;
        println!(
            "Godot Engine v{} set as default.",
            self.version.color("green")
        );

        Ok(())
    }
}

impl Execute for GetDefault {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        if let Some(e) = ehandler.get_default()? {
            println!("{} {}", "*".color("green"), e.color("green"));
        } else {
            print_missing_default_engine_message();
        }

        Ok(())
    }
}

impl Install {
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

impl Execute for Install {
    fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let (version, system) = parse_godot_version_args(&self.version, self.headless, self.server);

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

impl Execute for Uninstall {
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
                    println!("{}", format!("Engine version '{0}' was not downloaded through gdpm. Use 'unregister {0}' instead.", version_name).color("yellow"));
                    std::process::exit(1);
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
