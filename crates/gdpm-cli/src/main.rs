use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use commands::{parse_args, Args};
use context::Context;
use gdpm_core::{downloader::DefaultDownloadAdapter, io::DefaultIoAdapter};

mod commands;
mod common;
mod context;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = Args::parse();
    let ctx = Context::new(DefaultIoAdapter, DefaultDownloadAdapter);

    if let Err(e) = parse_args(ctx, args) {
        eprintln!();
        eprintln!("{}", "/!\\ The application crashed.".red());
        eprintln!("{e:?}");
    }

    Ok(())
}
