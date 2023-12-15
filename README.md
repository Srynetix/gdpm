# gdpm - Godot Project Manager

[![Coverage Status](https://coveralls.io/repos/github/Srynetix/gdpm/badge.svg?branch=main)](https://coveralls.io/github/Srynetix/gdpm?branch=main)

A command line utility to manage a Godot project.

[CHANGELOG](./CHANGELOG.md)

## Roadmap

- [x] Parse the Godot project file
- [x] Get project information using the project.godot file
- [x] Write custom configuration in the project.godot file
- [x] Manage Godot executables
  - [x] Manual registering and unregistering
  - [x] Scan versions and download from official mirror 
- [x] Manage dependencies (in the addons folder)
  - [x] Sync from another project
  - [x] Sync from external source (git)
  - [x] Desync dependencies
  - [x] Fork dependency in project (inclusion)
- [x] Proxy commands to engine instance (like export)

## Installation

The easiest way (you need to have Rust and Cargo):

```bash
cargo install --git https://github.com/Srynetix/gdpm#v2.0.0 gdpm
```

You can also get the latest release on the [GitHub release page](https://github.com/Srynetix/gdpm/releases), and put the executable in your PATH. 

## Workflow

Using gdpm is quite simple.  
You can use the following workflow.

### 1. Register or install engine instances

Before you start to manage your projects, you have to register engine instances.  
For example, imagine you have two Godot instances on your disk:
- Godot Engine **4.1**, stored in `C:\Godot\4.1\godot.exe`
- Godot Engine **4.2.beta1**, stored in `C:\Godot\4.2beta1\godot.exe`

You have to use the `gdpm engine add` command to register each version in a gdpm configuration file.  
Following these examples, you have to execute:

```bash
gdpm engine add 4.1 --target-path C:\Godot\4.1\godot.exe
gdpm engine add 4.2.beta1 --target-path C:\Godot\3.2beta1\godot.exe
```

The first engine entry will became default.  
If you want to set the **4.2beta1** version as default, just execute:

```bash
gdpm engine set-default 4.2.beta1
```

If you do not have already installed engine versions, you can also use the `engine add` command to let `gdpm` install them for you.

```bash
# If you want the 4.2.1 stable, mono edition
gdpm engine add 4.2.1.mono

# If you want the 4.2 beta 2, gdscript edition
gdpm engine add 4.2.beta2
```

### 2. Create a project / Assign your engine version

You can create a project using the `new` subcommand.

```bash
gdpm new my_game ./my_project_path
```

It will use the default engine.

You can use the `gdpm info` command, which should show you the associated engine version.  
Now, to run the engine editor associated to your project, you can just execute:

```bash
gdpm edit
```

Your project will be opened in the right engine version.

If you are editing an existing project, you can assign an engine version to it using this command (in the project folder):

```bash
gdpm set-engine 3.1
# or gpdm set-engine 3.2.beta1
```

### 3. Manage dependencies

In Godot, the root `addons` folder is special, and contains plugins, with a `plugin.cfg` definition file.  
Using gdpm, addons will be identified using their folder name. So if you have a `addons/plugin1` folder with a `plugin.cfg` file,
your plugin will be identified as `plugin1`, and it can be managed.

Plugins can be fetched from (for now) 3 different location types:
- Current project: when the plugin is integrated to the project,
- Filesystem path: when the plugin is present in another project located in the filesystem,
- Git URL: when the plugin is located on a remote repository.

Project plugins can be auto-registered as "current project" dependencies using the `gdpm sync` command.

As an example, let's say that your project contains a `plugin1` plugin in its `addons` folder.  
You know that you will reuse `plugin2` from your precedent project, and you may want to use a plugin contained in a remote Git repository.  
How do we specify this? How can we retrieve the plugin code in our project? It's quite simple:

```bash
# Let's add plugin2 from the ../other-project project
gdpm add plugin2 1.0.0 ../other-project
# Then add `gitplugin` from the `git@github.com:example/example-project` project
gdpm add gitplugin 1.0.0 git@github.com:example/example-project
```

Your plugins will be copied in your project.

## Details

Dependencies will be added to the `project.godot` file, so we don't have to manage two project files.  
Godot recognize each entry in the file so if we add a `[dependencies]` section, it will show up in the project settings editor, so it can be manipulated from inside the engine.

gdpm configuration will be in a `.gdpm` folder in the user home.  
It will contain paths to different Godot instances (with unique names).  
These names will be used in `project.godot`, with an error if the path is not found.

## Commands

[Command documentation](./COMMANDS.md)
