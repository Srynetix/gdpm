use argh::FromArgs;
use std::path::PathBuf;
use color_eyre::Result;

use super::Execute;

/// add dependency
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
pub struct Add {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(positional)]
    name: String,
    /// version
    #[argh(positional)]
    version: String,
    /// source
    #[argh(positional)]
    source: String,
    /// do not install
    #[argh(option)]
    no_install: bool,
}

/// fork dependency: include in code
#[derive(FromArgs)]
#[argh(subcommand, name = "fork")]
pub struct Fork {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(positional)]
    name: String,
}

/// remove dependency
#[derive(FromArgs)]
#[argh(subcommand, name = "remove")]
pub struct Remove {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(positional)]
    name: String,
}

/// list dependencies
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
}

/// sync project dependencies
#[derive(FromArgs)]
#[argh(subcommand, name = "sync")]
pub struct Sync {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(option)]
    name: Option<String>
}

/// desync project dependencies
#[derive(FromArgs)]
#[argh(subcommand, name = "desync")]
pub struct Desync {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
    /// name
    #[argh(option)]
    name: Option<String>
}

impl Execute for Add {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Fork {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Remove {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for List {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Sync {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Desync {
    fn execute(self) -> Result<()> {
        todo!()
    }
}
