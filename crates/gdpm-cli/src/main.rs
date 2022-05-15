use clap::Parser;
use color_eyre::Result;
use commands::{parse_args, Args};

mod commands;
mod common;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Force env if not present
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    let args: Args = Args::parse();
    parse_args(args)
}
