use clap::Parser;
use color_eyre::Result;
use commands::{parse_args, Args};
use context::Context;
use gdpm_core::{downloader::DownloadImpl, io::IoImpl};

mod commands;
mod common;
mod context;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Force env if not present
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    let args: Args = Args::parse();
    let ctx = Context::new(IoImpl, DownloadImpl);
    parse_args(ctx, args)
}
