use failure::Error;
use question::{Answer, Question};
use slugify::slugify;
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
    /// Edit project
    Edit {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
        /// Version
        #[structopt(default_value = "")]
        version: String,
    },
    /// Set project engine
    SetEngine {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
        /// Engine version
        #[structopt(default_value = "")]
        version: String,
    },
    /// Unset project engine
    UnsetEngine {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
    },
    /// Engine management
    Engine {
        #[structopt(subcommand)]
        cmd: EngineCommand,
    },
}

#[derive(StructOpt, Debug)]
enum EngineCommand {
    /// List engines
    List {
        /// Verbose
        #[structopt(short, long)]
        verbose: bool,
    },
    /// Register engine entry
    Register {
        /// Version
        version: String,
        /// Get engine path
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        /// Has mono
        #[structopt(long = "mono")]
        has_mono: bool,
        /// Built from source
        #[structopt(long = "source")]
        from_source: bool,
    },
    /// Unregister engine entry
    Unregister {
        /// Version
        version: String,
    },
    /// Run engine
    Run {
        /// Version
        #[structopt(default_value = "")]
        version: String,
    },
    /// Get/Set engine as default
    Default {
        /// Version
        #[structopt(default_value = "")]
        version: String,
    },
}

fn print_missing_default_engine_message() {
    println!("No default engine registered. Use `engine default <version>` to register one.");
}

fn print_missing_project_engine_message() {
    println!("You have no engine version associated to your project.");
}

/// Run gdpm shell
pub fn run_shell() -> Result<(), Error> {
    let args = Opt::from_args();
    match args.cmd {
        Command::Info { path } => {
            use crate::actions::project::get_project_info;
            let info = get_project_info(&path)?;
            info.show();
        }
        Command::SetEngine { path, version } => {
            use crate::actions::engine::{get_default_engine, get_engine_version};
            use crate::actions::project::set_project_engine;
            if version == "" {
                if let Some(e) = get_default_engine()? {
                    set_project_engine(&path, &e)?;
                    println!(
                        "Godot Engine v{} set for project: {}",
                        version,
                        path.to_string_lossy()
                    );
                } else {
                    print_missing_default_engine_message();
                }
            } else {
                get_engine_version(&version)?;
                set_project_engine(&path, &version)?;
                println!(
                    "Godot Engine v{} set for project: {}",
                    version,
                    path.to_string_lossy()
                );
            }
        }
        Command::UnsetEngine { path } => {
            use crate::actions::project::unset_project_engine;
            unset_project_engine(&path)?;
            println!(
                "Engine deassociated from project: {}",
                path.to_string_lossy()
            );
        }
        Command::Edit { path, version } => {
            use crate::actions::engine::{get_default_engine, run_engine_version_for_project};
            use crate::actions::project::{get_project_info, set_project_engine};

            // Use project or default version
            if version == "" {
                // Detect project version
                let project_info = get_project_info(&path)?;
                if let Some(e) = project_info.get_engine_version() {
                    println!(
                        "Running Godot Engine v{} for project {} ...",
                        e,
                        path.to_string_lossy()
                    );
                    run_engine_version_for_project(&e, &path)?;
                } else {
                    // Use default version
                    if let Some(e) = get_default_engine()? {
                        print_missing_project_engine_message();
                        match Question::new(&format!(
                            "Do you want to associate the default engine (v{}) (y/n)?",
                            e
                        ))
                        .confirm()
                        {
                            Answer::YES => set_project_engine(&path, &e)?,
                            Answer::NO => println!("Okay. You will be asked again next time."),
                            _ => unreachable!(),
                        }

                        println!(
                            "Running Godot Engine v{} for project {} ...",
                            e,
                            path.to_string_lossy()
                        );
                        run_engine_version_for_project(&e, &path)?;
                    } else {
                        print_missing_project_engine_message();
                        print_missing_default_engine_message();
                    }
                }
            } else {
                // Use specific version (override)
                println!(
                    "Running Godot Engine v{} for project {} ...",
                    version,
                    path.to_string_lossy()
                );
                run_engine_version_for_project(&version, &path)?;
            }
        }
        Command::Engine { cmd } => match cmd {
            EngineCommand::List { verbose } => {
                use crate::actions::engine::{get_default_engine, list_engines_info};
                let entries = list_engines_info()?;
                let default_entry = get_default_engine()?;
                for entry in entries {
                    if let Some(default) = &default_entry {
                        if entry.get_slug() == slugify!(default) {
                            print!("* ");
                        } else {
                            print!("  ");
                        }
                    } else {
                        print!("  ");
                    }

                    if verbose {
                        entry.show_verbose();
                    } else {
                        entry.show();
                    }
                }
            }
            EngineCommand::Register {
                version,
                path,
                has_mono,
                from_source,
            } => {
                use crate::actions::engine::{register_engine_entry, EngineInfo};
                let engine_info = EngineInfo::new(version.clone(), path, has_mono, from_source)?;

                register_engine_entry(engine_info)?;
                println!("Godot Engine v{} is registered.", version);
            }
            EngineCommand::Unregister { version } => {
                use crate::actions::engine::unregister_engine_entry;
                unregister_engine_entry(&version)?;
                println!("Godot Engine v{} unregistered.", version);
            }
            EngineCommand::Run { version } => {
                use crate::actions::engine::{get_default_engine, run_engine_version};
                if version == "" {
                    if let Some(e) = get_default_engine()? {
                        println!("Running Godot Engine v{} ...", e);
                        run_engine_version(&e)?;
                    } else {
                        print_missing_default_engine_message();
                    }
                } else {
                    println!("Running Godot Engine v{} ...", version);
                    run_engine_version(&version)?;
                }
            }
            EngineCommand::Default { version } => {
                use crate::actions::engine::{get_default_engine, set_default_engine};
                if version.is_empty() {
                    if let Some(e) = get_default_engine()? {
                        println!("* Godot Engine v{}", e);
                    } else {
                        print_missing_default_engine_message();
                    }
                } else {
                    set_default_engine(&version)?;
                    println!("Godot Engine v{} set as default.", version);
                }
            }
        },
    }

    Ok(())
}
