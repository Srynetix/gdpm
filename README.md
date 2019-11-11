# gdpm - Godot Project Manager

A command line utility to manage a Godot 3.x project.  
Toy project (for now) written in Rust.

## Roadmap

- [x] Parse the Godot project file
- [x] Get project information using the project.godot file
- [x] Write custom configuration in the project.godot file
- [x] Manage Godot instances
- [ ] Manage dependencies (in the addons folder)
  - [x] Sync from another project
  - [x] Sync from external source (git)
  - [ ] Sync to another project
  - [x] Desync dependencies
  - [ ] Fork dependency in project (inclusion)
- [ ] Execute custom actions
- [ ] Proxy commands to engine instance (like export)

## Workflow

Using gdpm is quite simple.  
You can use the following workflow.

### 1. Register engine instances

Before we start to manage your projects, you have to register engine instances.  
For example, imagine you have two Godot instances on your disk:
- Godot Engine **3.1**, stored in `C:\Godot\3.1\godot.exe`
- Godot Engine **3.2beta1**, stored in `C:\Godot\3.2beta1\godot.exe`

You have to use the `gdpm engine register` command to register each version in a gdpm configuration file.  
Following these examples, you have to execute:

```bash
gdpm engine register 3.1 C:\Godot\3.1\godot.exe
gdpm engine register 3.2beta1 C:\Godot\3.2beta1\godot.exe
```

The first engine entry will became default.  
If you want to set the **3.2beta1** version as default, just execute:

```bash
gdpm engine default 3.2beta1
```

### 2. Assign your engine version

Until project creation is supported from the tool, you just have to create a project with Godot.  
A quick shortcut from gdpm is to execute the following command anywhere:

```bash
gdpm engine run
```

It will start the default engine.

When your project is ready, you can assign an engine version to it using this command (in the project folder):

```bash
gdpm set-engine 3.1
# or gpdm set-engine 3.2beta1
```

You can see the changes using the `gdpm info` command, which should show you the associated engine version.  
Now, to run the engine editor associated to your project, you can just execute:

```bash
gdpm edit
```

Your project will be opened in the right engine version.

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
gdpm add gitplugin 1.0.0 git@github.com:example/example-project`
# Now sync everything to register `plugin1` and install the other plugins
gdpm sync
```

## Details

Dependencies will be added to the `project.godot` file, so we don't have to manage two project files.  
Godot recognize each entry in the file so if we add a `[dependencies]` section, it will show up in the project settings editor, so it can be manipulated from inside the engine.

gdpm configuration will be in a `.gdpm` folder in the user home.  
It will contain paths to different Godot instances (with unique names).  
These names will be used in project.godot, with an error if the path is not found.

## Commands

### `add`

Add a dependency to the current project folder.  
You can pass another folder using the `--path` argument.  
The required arguments are: [name], [version] and [source].

If a dependency with the same name is already registered to the project, it will be updated.

*Note*: added dependencies are not automatically resolved. To install the dependencies, use the `gdpm sync` command.

*Examples:*

```bash
gdpm add plugin1 1.0.0 ../plugin1
# Dependency plugin1 (v1.0.0) from ../plugin1 added to project ..

gdpm add scenerunner 1.0.0 git@github.com:Srynetix/godot-plugin-scenerunner
# Dependency scenerunner (v1.0.0) from git@github.com:Srynetix/godot-plugin-scenerunner added to project ..

gdpm add plugin1 1.0.0 ../plugin1 --path ./my/project
# Dependency plugin1 (v1.0.0) from ../plugin1 added to project ./my/project.
```

### `desync`

Desynchronize dependencies for the current project.  
You can pass another folder using the `--path` argument.

Il will remove installed plugins which are not from the project.

*Examples:*

```bash
gdpm desync
# Dependencies are desynchronized for project ..

gdpm desync --path ./my/project
# Dependencies are desynchronized for project ./my/project.
```

### `edit`

Open the engine in editor mode for the current folder.  
You can pass another folder using the `--path` argument.

If the project has no engine associated, it will ask you if you want to use the default engine for this project.
You can also specify to open the project in a specific editor version.

*Examples:*

```bash
gdpm edit
# > Running Godot Engine v3.1.1 for project . ...

gdpm edit --path ./my/project
# > Running Godot Engine v3.1.1 for project ./my/project ...

gdpm edit --path ./my/project 3.2alpha3
# > Running Godot Engine v3.2alpha3 for project ./my/project ...
```

### `info`

Get project info from current folder.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm info
# Project: My project
# - Engine version: v3.2alpha3

gdpm info --path ./my/project
# Project: My project 2
# - Engine version: v3.1.1
```

### `list`

List dependencies from the current project.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm list
# - plugin1 (v1.0.0) (source: Current)
# - plugin2 (v1.0.0) (source ../plugin2)
# - scenerunner (v1.0.0) (source: Git (SSH): git@github.com:Srynetix/godot-plugin-scenerunner)

gdpm list --path ./my/project
# - plugin1 (v1.0.0) (source: Current)
# - plugin2 (v1.0.0) (source ../plugin2)
```

### `remove`

Remove a dependency from the current project.  
You can pass another folder using the `--path` argument.

If the dependency is installed, its folder will also be removed.

*Examples:*

```bash
gdpm remove plugin1
# Dependency plugin1 removed from project ..

gdpm remove plugin1 --path ./my/project
# Dependency plugin1 removed from project ./my/project.
```

### `set-engine`

Associate an engine to a project.  
You can pass another folder using the `--path` argument.

If you have a default engine set and if you are not passing a `version` to the command, il will associate the default engine to your project.

*Examples:*

```bash
gdpm set-engine
# > Godot Engine v3.1.1 set for project: .

gdpm set-engine --path ./my/project
# > Godot Engine v3.1.1 set for project: ./my/project

gdpm set-engine --path ./my/project 3.2alpha3
# > Godot Engine v3.2alpha3 set for project: ./my/project
```

### `sync`

Synchronize/Install registered dependencies from the current project.  
You can pass another folder using the `--path` argument.

It will also scan the `addons` folder and register dependencies as `current` if they are not present in the dependency list.

*Examples:*

```bash
gdpm sync
# Dependencies are now synchronized for project ..

gdpm sync --path ./my/project
# Dependencies are now synchronized for project ./my/project.
```

### `unset-engine`

Deassociate an engine from a project.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm unset-engine
# > Engine deassociated from project: .

gdpm unset-engine --path ./my/project
# > Engine deassociated from project: ./my/project
```

### `engine list`

Get a list of the registered engines for your user.
The default engine will have a star at start.

*Examples:*

```bash
gdpm engine list
# * Godot Engine v3.1.1
#   Godot Engine v3.1.1-mono
#   Godot Engine v3.2alpha3
#   Godot Engine vmaster

gdpm engine list --verbose
# * Godot Engine v3.1.1 (./path/to/3.1.1) [mono: false, source: false]
#   Godot Engine v3.1.1-mono (./path/to/3.1.1-mono) [mono: true, source: false]
#   Godot Engine v3.2alpha3 (./path/to/3.2alpha3) [mono: false, source: false]
#   Godot Engine vmaster (./path/to/master) [mono: false, source: true]
```

### `engine register`

Register/edit an engine for your user.  
You have to specify the `version`, the `path`, and `--mono` and/or `--source` if needed.

The first engine you register will automatically become the default engine.

*Examples:*

```bash
gdpm engine register 3.1.1 C:/.../3.1.1/godot.exe
# > Godot Engine v3.1.1 registered.

gdpm engine register 3.1.1-mono C:/.../3.1.1-mono/godot.exe --mono
# > Godot Engine v3.1.1-mono registered.

gdpm engine register master C:/.../master/bin/godot.exe --source
# > Godot Engine vmaster registered.
```

### `engine unregister`

Unregister an engine for your user.  
If you unregister the default engine, it will remain unset. You will have to call `engine default <version>`.

*Examples:*

```bash
gdpm engine unregister v3.2alpha2
# > Godot Engine v3.2alpha2 unregistered.
```

### `engine default`

Set a registered engine as default, or get the default engine if `version` is not set.

*Examples:*

```bash
gdpm engine default
# > No default engine registered. Use `engine default <version>` to register one.

gdpm engine default 3.1.1
# > Godot Engine v3.1.1 set as default.

gdpm engine default
# * Godot Engine v3.1.1
```

### `engine run`

Run a specific version of an engine, or run the default engine version if `version` is not set.

```bash
gdpm engine run
# > Running Godot Engine v3.1.1 ...

gdpm engine run master
# > Running Godot Engine vmaster ...
```
