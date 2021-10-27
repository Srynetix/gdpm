use argh::FromArgs;
use std::path::PathBuf;
use color_eyre::Result;

use super::Execute;

/// engine management
#[derive(FromArgs)]
#[argh(subcommand, name = "engine")]
pub struct Engine {
    #[argh(subcommand)]
    cmd: Command
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    List(List),
    Register(Register),
    Unregister(Unregister),
    Start(Start),
    Cmd(Cmd),
    SetDefault(SetDefault),
    GetDefault(GetDefault)
}

/// list engines
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {
    /// project path
    #[argh(switch, short = 'p')]
    verbose: bool
}

/// register engine
#[derive(FromArgs)]
#[argh(subcommand, name = "register")]
pub struct Register {
    /// version
    #[argh(positional)]
    version: String,
    /// engine path
    #[argh(positional)]
    path: PathBuf,
    /// mono edition?
    #[argh(switch)]
    mono: bool,
    /// built from source?
    #[argh(switch)]
    built_from_source: bool
}

/// unregister engine
#[derive(FromArgs)]
#[argh(subcommand, name = "unregister")]
pub struct Unregister {
    /// version
    #[argh(positional)]
    version: String
}

/// start engine
#[derive(FromArgs)]
#[argh(subcommand, name = "start")]
pub struct Start {
    /// version
    #[argh(option)]
    version: Option<String>
}

/// execute command on engine
#[derive(FromArgs)]
#[argh(subcommand, name = "cmd")]
pub struct Cmd {
    /// version
    #[argh(option)]
    version: Option<String>,
    /// arguments
    #[argh(option)]
    args: Vec<String>
}

/// set engine as default
#[derive(FromArgs)]
#[argh(subcommand, name = "set-default")]
pub struct SetDefault {
    /// version
    #[argh(positional)]
    version: String
}

/// get default engine
#[derive(FromArgs)]
#[argh(subcommand, name = "get-default")]
pub struct GetDefault {}

impl Execute for Engine {
    fn execute(self) -> Result<()> {
        self.cmd.execute()
    }
}

impl Execute for Command {
    fn execute(self) -> Result<()> {
        match self {
            Self::List(c) => c.execute(),
            Self::Register(c) => c.execute(),
            Self::Unregister(c) => c.execute(),
            Self::Start(c) => c.execute(),
            Self::Cmd(c) => c.execute(),
            Self::SetDefault(c) => c.execute(),
            Self::GetDefault(c) => c.execute(),
        }
    }
}

impl Execute for List {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Register {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Unregister {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Start {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for Cmd {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for SetDefault {
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl Execute for GetDefault {
    fn execute(self) -> Result<()> {
        todo!()
    }
}
