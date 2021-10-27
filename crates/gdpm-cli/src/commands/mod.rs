use argh::FromArgs;
use color_eyre::Result;

mod dependency;
mod engine;
mod project;

/// manage Godot versions and project dependencies
#[derive(FromArgs)]
pub struct Args {
    /// verbose mode
    #[argh(switch)]
    verbose: bool,
    #[argh(subcommand)]
    command: Command
}

trait Execute {
    /// Execute!
    fn execute(self) -> Result<()>;
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    Info(project::Info),
    Edit(project::Edit),
    SetEngine(project::SetEngine),
    UnsetEngine(project::UnsetEngine),
    Add(dependency::Add),
    Fork(dependency::Fork),
    Remove(dependency::Remove),
    List(dependency::List),
    Sync(dependency::Sync),
    Desync(dependency::Desync),
    Engine(engine::Engine)
}

pub fn parse_args(args: Args) -> Result<()> {
    let verbose = args.verbose;

    match args.command {
        Command::Info(c) => c.execute(),
        Command::Edit(c) => c.execute(),
        Command::SetEngine(c) => c.execute(),
        Command::UnsetEngine(c) => c.execute(),
        Command::Add(c) => c.execute(),
        Command::Fork(c) => c.execute(),
        Command::Remove(c) => c.execute(),
        Command::List(c) => c.execute(),
        Command::Sync(c) => c.execute(),
        Command::Desync(c) => c.execute(),
        Command::Engine(c) => c.execute()
    }
}
