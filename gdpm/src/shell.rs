use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Get project info
    Info {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
    },
}

/// Run gdpm shell
pub fn run_shell() -> Result<(), Error> {
    let args = Opt::from_args();
    match args.cmd {
        Command::Info { path } => {
            use crate::actions::get_project_info;
            println!("Reading project info from path {:?} ...", path);
            let info = get_project_info(&path)?;
            info.show();
        }
    }

    Ok(())
}
