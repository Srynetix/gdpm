use std::env;
use std::path::{Path, PathBuf};

use colored::Colorize;
use env_logger;
use failure::Error;
use question::{Answer, Question};
use slugify::slugify;
use structopt::StructOpt;

/// Manage Godot versions and project dependencies
#[derive(StructOpt, Debug)]
#[structopt(name = "gdpm")]
struct Opt {
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
    /// Add dependency
    Add {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
        /// Name
        name: String,
        /// Version
        version: String,
        /// Dependency source
        source: String,
    },
    /// Fork dependency: integrate in code
    Fork {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
        /// Name
        name: String,
    },
    /// Remove dependency
    Remove {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
        /// Name
        name: String,
    },
    /// List dependencies
    List {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
    },
    /// Sync project plugins
    Sync {
        /// Project path
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        path: PathBuf,
    },
    /// Desync project plugins
    Desync {
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
    /// Execute command
    Cmd {
        /// Version
        #[structopt(default_value = "")]
        version: String,
        /// Arguments
        #[structopt(raw(true))]
        args: Vec<String>,
    },
    /// Get/Set engine as default
    Default {
        /// Version
        #[structopt(default_value = "")]
        version: String,
    },
}

fn print_missing_default_engine_message() {
    println!(
        "{}",
        "No default engine registered. Use `engine default <version>` to register one."
            .color("yellow")
    );
}

fn print_missing_project_engine_message() {
    println!(
        "{}",
        "You have no engine version associated to your project.".color("yellow")
    );
}

/// Run gdpm shell
pub fn run_shell() -> Result<(), Error> {
    let args = Opt::from_args();
    if args.verbose > 0 {
        // Enable debug logs
        env::set_var("RUST_LOG", "debug");
    }

    // Initialize logger
    env_logger::init();

    match args.cmd {
        Command::Info { path } => {
            use crate::project::get_project_info;
            let info = get_project_info(&path)?;
            info.show();
        }
        Command::SetEngine { path, version } => {
            use crate::engine::{get_default_engine, get_engine_version};
            use crate::project::{get_project_info, set_project_engine};
            let info = get_project_info(&path)?;

            if version == "" {
                if let Some(e) = get_default_engine()? {
                    set_project_engine(&path, &e)?;
                    println!(
                        "Godot Engine v{} set for project {}.",
                        version.color("green"),
                        info.get_versioned_name().color("green")
                    );
                } else {
                    print_missing_default_engine_message();
                }
            } else {
                get_engine_version(&version)?;
                set_project_engine(&path, &version)?;
                println!(
                    "Godot Engine v{} set for project {}.",
                    version.color("green"),
                    info.get_versioned_name().color("green")
                );
            }
        }
        Command::UnsetEngine { path } => {
            use crate::project::{get_project_info, unset_project_engine};
            unset_project_engine(&path)?;
            let info = get_project_info(&path)?;

            println!(
                "Engine deassociated from project {}.",
                info.get_versioned_name().color("green")
            );
        }
        Command::Edit { path, version } => {
            use crate::engine::{get_default_engine, run_engine_version_for_project};
            use crate::project::{get_project_info, set_project_engine};

            let project_info = get_project_info(&path)?;

            // Use project or default version
            if version == "" {
                // Detect project version
                if let Some(e) = project_info.get_engine_version() {
                    println!(
                        "Running Godot Engine v{} for project {} ...",
                        e.color("green"),
                        project_info.get_versioned_name().color("green")
                    );
                    run_engine_version_for_project(&e, &path)?;
                } else {
                    // Use default version
                    if let Some(e) = get_default_engine()? {
                        print_missing_project_engine_message();
                        match Question::new(&format!(
                            "Do you want to associate the default engine (v{}) to project {} (y/n)?",
                            e.color("green"),
                            project_info.get_versioned_name().color("green")
                        ))
                        .confirm()
                        {
                            Answer::YES => set_project_engine(&path, &e)?,
                            Answer::NO => println!("Okay. You will be asked again next time."),
                            _ => unreachable!(),
                        }

                        println!(
                            "Running Godot Engine v{} for project {} ...",
                            e.color("green"),
                            project_info.get_versioned_name().color("green")
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
                    version.color("green"),
                    project_info.get_versioned_name().color("green")
                );
                run_engine_version_for_project(&version, &path)?;
            }
        }
        Command::List { path } => {
            use crate::plugins::list_project_dependencies;
            use crate::project::get_project_info;
            let project_info = get_project_info(&path)?;
            let dependencies = list_project_dependencies(&path)?;
            println!(
                "Dependencies from project {}:",
                project_info.get_versioned_name().color("green")
            );

            for dep in dependencies {
                dep.show();
            }
        }
        Command::Sync { path } => {
            use crate::plugins::sync_project_plugins;
            use crate::project::get_project_info;
            let project_info = get_project_info(&path)?;
            sync_project_plugins(&path)?;

            println!(
                "Dependencies are now synchronized for project {}.",
                project_info.get_versioned_name().color("green")
            )
        }
        Command::Desync { path } => {
            use crate::plugins::desync_project_plugins;
            use crate::project::get_project_info;
            let project_info = get_project_info(&path)?;
            desync_project_plugins(&path)?;

            println!(
                "Dependencies are desynchronized for project {}.",
                project_info.get_versioned_name().color("green")
            )
        }
        Command::Fork { path, name } => {
            use crate::plugins::fork_dependency;
            use crate::project::get_project_info;
            let project_info = get_project_info(&path)?;
            fork_dependency(&path, &name)?;

            println!(
                "Plugin {} forked in project {}.",
                name.color("green"),
                project_info.get_versioned_name().color("green")
            )
        }
        Command::Add {
            path,
            name,
            version,
            source,
        } => {
            use crate::plugins::add_dependency;
            use crate::project::get_project_info;
            let project_info = get_project_info(&path)?;
            add_dependency(&path, &name, &version, &source)?;

            println!(
                "Dependency {} (v{}) from {} added to project {}.",
                name.color("green"),
                version.color("green"),
                source.color("blue"),
                project_info.get_versioned_name().color("green")
            );
        }
        Command::Remove { path, name } => {
            use crate::plugins::remove_dependency;
            use crate::project::get_project_info;
            let project_info = get_project_info(&path)?;
            remove_dependency(&path, &name)?;

            println!(
                "Dependency {} removed from project {}.",
                name.color("green"),
                project_info.get_versioned_name().color("green")
            );
        }
        Command::Engine { cmd } => match cmd {
            EngineCommand::List { verbose } => {
                use crate::engine::{get_default_engine, list_engines_info};
                let entries = list_engines_info()?;
                let default_entry = get_default_engine()?;
                for entry in entries {
                    if let Some(default) = &default_entry {
                        if entry.get_slug() == slugify!(default) {
                            print!("{} ", "*".color("green"));
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
                use crate::engine::{register_engine_entry, EngineInfo};
                let engine_info = EngineInfo::new(version.clone(), path, has_mono, from_source)?;

                register_engine_entry(engine_info)?;
                println!("Godot Engine v{} is registered.", version.color("green"));
            }
            EngineCommand::Unregister { version } => {
                use crate::engine::unregister_engine_entry;
                unregister_engine_entry(&version)?;
                println!("Godot Engine v{} unregistered.", version.color("green"));
            }
            EngineCommand::Cmd { version, args } => {
                use crate::engine::{exec_engine_version_command_for_project, get_default_engine};
                if version == "" {
                    if let Some(e) = get_default_engine()? {
                        println!(
                            "Executing command {} on Godot Engine v{} ...",
                            args.join(" ").color("blue"),
                            e.color("green")
                        );
                        exec_engine_version_command_for_project(&e, &args, Path::new("."))?;
                    } else {
                        print_missing_default_engine_message();
                    }
                } else {
                    println!(
                        "Executing command {} Godot Engine v{} ...",
                        args.join(" ").color("blue"),
                        version.color("green")
                    );
                    exec_engine_version_command_for_project(&version, &args, Path::new("."))?;
                }
            }
            EngineCommand::Run { version } => {
                use crate::engine::{get_default_engine, run_engine_version_for_project};
                if version == "" {
                    if let Some(e) = get_default_engine()? {
                        println!("Running Godot Engine v{} ...", e.color("green"));
                        run_engine_version_for_project(&e, Path::new("."))?;
                    } else {
                        print_missing_default_engine_message();
                    }
                } else {
                    println!("Running Godot Engine v{} ...", version.color("green"));
                    run_engine_version_for_project(&version, Path::new("."))?;
                }
            }
            EngineCommand::Default { version } => {
                use crate::engine::{get_default_engine, set_default_engine};
                if version.is_empty() {
                    if let Some(e) = get_default_engine()? {
                        println!("{} Godot Engine v{}", "*".color("green"), e.color("green"));
                    } else {
                        print_missing_default_engine_message();
                    }
                } else {
                    set_default_engine(&version)?;
                    println!("Godot Engine v{} set as default.", version.color("green"));
                }
            }
        },
    }

    Ok(())
}
