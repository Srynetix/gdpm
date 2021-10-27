use argh::FromArgs;
use std::path::PathBuf;
use color_eyre::Result;

use super::Execute;

/// get project info
#[derive(FromArgs)]
#[argh(subcommand, name = "info")]
pub struct Info {
    /// project path
    #[argh(option, short = 'p', default = "PathBuf::from(\".\")")]
    path: PathBuf
}

/// edit project
#[derive(FromArgs)]
#[argh(subcommand, name = "edit")]
pub struct Edit {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,

    /// version
    #[argh(positional, default = "String::from(\"\")")]
    version: String
}

/// set project engine
#[derive(FromArgs)]
#[argh(subcommand, name = "set-engine")]
pub struct SetEngine {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,

    /// version
    #[argh(option)]
    version: Option<String>
}

/// unset project engine
#[derive(FromArgs)]
#[argh(subcommand, name = "unset-engine")]
pub struct UnsetEngine {
    /// project path
    #[argh(option, default = "PathBuf::from(\".\")")]
    path: PathBuf,
}

impl Execute for Info {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Edit {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for SetEngine {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for UnsetEngine {
    fn execute(self) -> Result<()> {
        todo!()
    }
}
