use argh::FromArgs;
use color_eyre::Result;
use tracing_subscriber::EnvFilter;

mod dependency;
mod engine;
mod project;

/// manage Godot versions and project dependencies
#[derive(FromArgs)]
pub struct Args {
    /// verbose mode
    #[argh(switch, short = 'v')]
    verbose: bool,

    #[argh(subcommand)]
    command: Command,
}

trait Execute {
    /// Execute!
    fn execute(self) -> Result<()>;
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    Project(project::Project),
    Dependencies(dependency::Dependencies),
    Engine(engine::Engine),
    Version(Version),
}

/// show version
#[derive(FromArgs)]
#[argh(subcommand, name = "version")]
struct Version {}

pub fn parse_args(args: Args) -> Result<()> {
    // Set RUST_LOG depending on "verbose" arg
    if std::env::var("RUST_LOG").unwrap_or_default().is_empty() {
        if args.verbose {
            std::env::set_var("RUST_LOG", "warn,gdpm=debug");
        } else {
            std::env::set_var("RUST_LOG", "warn,gdpm=info");
        }
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .with_target(false)
        .compact()
        .init();

    match args.command {
        Command::Project(c) => c.execute(),
        Command::Dependencies(c) => c.execute(),
        Command::Engine(c) => c.execute(),
        Command::Version(_) => {
            let cmd_name = std::env::current_exe()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            println!("{} {}", cmd_name, env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
