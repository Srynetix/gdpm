use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use gdpm_core::{
    downloader::{download::Downloader, error::DownloadError, DownloadAdapter},
    engine::{EngineHandler, EngineInfo},
    io::{write_stderr, write_stdout, IoAdapter},
    types::version::{GodotVersion, SystemVersion},
};
use tracing::info;

use crate::{common::parse_godot_version_args, context::Context};

pub(crate) const MIRROR_URL: &str =
    "https://github.com/godotengine/godot-builds/releases/download/";

/// Download and install engine from official mirror or specific URL / path (e.g. 3.3.4, 3.3.4.mono, 3.5.rc1, 3.5.rc1.mono)
#[derive(Parser)]
pub(crate) struct Add {
    /// Engine version
    pub(crate) engine: GodotVersion,
    /// System version
    #[clap(long)]
    pub(crate) system_version: Option<SystemVersion>,
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
                write_stdout!(
                    context.io(),
                    "{}\n",
                    format!(
                        "Version '{}' installed for system '{}' at path '{}'",
                        version,
                        system,
                        path.display()
                    )
                    .color("green")
                )?;
            }
            Err(DownloadError::NotFound(u)) => {
                write_stdout!(
                    context.io(),
                    "{}\n",
                    format!(
                        "Version '{}' does not exist for system '{}' (or wrong url: {})",
                        version, system, u
                    )
                    .color("red")
                )?;
            }
            Err(e) => write_stdout!(
                context.io(),
                "{}\n",
                format!(
                    "Unexpected error while trying to download file at url '{}'\n    | {}",
                    url, e
                )
                .color("red")
            )?,
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
                write_stdout!(
                    context.io(),
                    "{}\n",
                    format!(
                        "Export templates for version '{}' installed at path '{}'",
                        version,
                        path.display()
                    )
                    .color("green")
                )?;
            }
            Err(DownloadError::NotFound(u)) => {
                write_stdout!(
                    context.io(),
                    "{}\n",
                    format!(
                        "Export templates for version '{}' does not exist (or wrong url: {})",
                        version, u
                    )
                    .color("red")
                )?;
            }
            Err(e) => {
                write_stdout!(
                    context.io(),
                    "{}\n",
                    format!(
                        "Unexpected error while trying to download file at url '{}'\n    | {}",
                        url, e
                    )
                    .color("red")
                )?;
            }
        }

        Ok(())
    }

    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let ehandler = EngineHandler::new(context.io());
        let (version, system) = parse_godot_version_args(&self.engine, self.system_version)?;

        let existing_version = ehandler.has_version(&version)?;
        if existing_version.is_some() {
            if !self.overwrite {
                write_stderr!(
                    context.io(),
                    "{}\n",
                    format!("Engine version '{}' is already installed. Use '--overwrite' to force installation.", version).color("yellow")
                )?;
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

            write_stdout!(context.io(), "{} is registered.\n", verbose_name)?;
            return Ok(());
        }

        if let Some(url) = self.target_url {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(Self::download_file_at_url(context, &url, version, system))?;

            write_stderr!(
                context.io(),
                "Cannot fetch export templates, missing URL.\n"
            )?;
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
