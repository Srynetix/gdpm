use clap::Parser;
use color_eyre::Result;
use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter};

use crate::context::Context;

/// List engines from remote
#[derive(Parser)]
#[clap(name = "list-remote")]
pub struct ListRemote;

impl ListRemote {
    pub fn execute<I: IoAdapter, D: DownloadAdapter>(self, context: &Context<I, D>) -> Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(Self::lookup_remote_versions(context))?;

        Ok(())
    }

    pub(crate) async fn lookup_remote_versions<I: IoAdapter, D: DownloadAdapter>(
        context: &Context<I, D>,
    ) -> Result<()> {
        let versions = context.download().lookup_remote_versions().await?;

        for version in versions {
            println!("{version}");
        }

        Ok(())
    }
}
