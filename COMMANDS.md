# Commands

List of commands and subcommands:

- `project` - project management
  - `info` - get project info
  - `edit` - edit project in Godot Engine
  - `set-engine` - choose a specific engine for a project
  - `unset-engine` - unset the specific engine for a project
- `deps` - dependency management
  - `add` - add a dependency
  - `fork` - integrate a dependency in the project
  - `remove` - remove a dependency
  - `list` - list project dependencies
  - `sync` - synchronize dependencies
  - `desync` - desynchronize dependencies
- `engine` - engine management
  - `list` - list registered engines
  - `list-remote` - list available engines on official mirror
  - `register` - register an engine on your filesystem
  - `unregister` - unregister a known engine
  - `start` - run a specific Godot Engine editor
  - `cmd` - execute a command on a specific Godot Engine editor
  - `set-default` - set the default engine version
  - `get-default` - get the default engine version
  - `install` - download and install a remote Godot Engine version
  - `uninstall` - uninstall a Godot Engine version
- `version` - show gdpm version

List of common flags:

- `-v, --verbose`: verbose mode

## `project` - project management

These are commands used to interact with the project metadata.

### `project info` - get project info

Get project info from current folder.  
You can pass another folder using the `--path` (or `-p`) argument.

*Examples:*

```bash
gdpm project info
# > Project: MyProject
# > - Engine version: v3.2alpha3

gdpm project info --path ./my/project
# > Project: MyProject2
# > - Engine version: v3.1.1
```

### `project edit` - edit project in Godot Engine

Open the engine in editor mode for the current folder.  
You can pass another folder using the `--path` argument.

If the project has no engine associated, it will ask you if you want to use the default engine for this project.
You can also specify to open the project in a specific editor version.

*Examples:*

> *Supposing `3.1.1` is set as default*

```bash
gdpm project edit
# > Running Godot Engine v3.1.1 for project MyProject ...

gdpm project edit --path ./my/project
# > Running Godot Engine v3.1.1 for project MyProject2 ...

gdpm project edit --path ./my/project -v 3.2.alpha3
# > Running Godot Engine v3.2.alpha3 for project MyProject2 ...
```

### `project set-engine` - choose a specific engine for a project

Associate an engine to a project.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm project set-engine 3.2.alpha3
# > Godot Engine v3.2.alpha3 set for project MyProject.

gdpm project set-engine 3.2.alpha3 --path ./my/project
# > Godot Engine v3.2.alpha3 set for project MyProject2.
```

### `project unset-engine` - unset the specific engine for a project

Deassociate an engine from a project.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm project unset-engine
# > Engine deassociated from project MyProject.

gdpm project unset-engine --path ./my/project
# > Engine deassociated from project MyProject2.
```

## `deps` - dependency management

These are commands used to handle dependencies for your project.

### `deps add` - add a dependency

Add a dependency to the current project folder.  
You can pass another folder using the `--path` argument.  
The required arguments are: [name], [version] and [source].

If a dependency with the same name is already registered to the project, it will be updated.

Dependencies are automatically installed, unless to pass the `--no-install` argument.  
If you used `--no-install`, you will have to use the `gdpm deps sync` command to install.

*Examples:*

```bash
gdpm deps add plugin1 1.0.0 ../plugin1
# > Dependency plugin1 (v1.0.0) from ../plugin1 added and installed to project MyProject.

gdpm deps add scenerunner 1.0.0 git@github.com:Srynetix/godot-plugin-scenerunner
# > Dependency scenerunner (v1.0.0) from git@github.com:Srynetix/godot-plugin-scenerunner added and installed to project MyProject.

gdpm deps add plugin1 1.0.0 ../plugin1 --path ./my/project --no-install
# > Dependency plugin1 (v1.0.0) from ../plugin1 added to project MyProject2.
```

### `deps fork` - integrate a dependency in the project

Integrate/Fork a dependency inside of the current folder.  
It will change the source of the dependency (and install it) as from the current project.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm deps add scenerunner 1.0.0 git@github.com:Srynetix/godot-plugin-scenerunner
# > Dependency scenerunner (v1.0.0) from git@github.com:Srynetix/godot-plugin-scenerunner added to project MyProject.

gdpm deps fork scenerunner
# > Dependency scenerunner forked in project MyProject.
```

### `deps remove` - remove a dependency

Remove a dependency from the current project.  
You can pass another folder using the `--path` argument.

If the dependency is installed, its folder will also be removed.

*Examples:*

```bash
gdpm deps remove plugin1
# > Dependency plugin1 removed from project MyProject.

gdpm deps remove plugin1 --path ./my/project
# > Dependency plugin1 removed from project MyProject2.
```

### `deps list` - list project dependencies

List dependencies from the current project.  
You can pass another folder using the `--path` argument.

*Examples:*

```bash
gdpm deps list
# > - plugin1 (v1.0.0) (source: Current)
# > - plugin2 (v1.0.0) (source ../plugin2)
# > - scenerunner (v1.0.0) (source: Git (SSH): git@github.com:Srynetix/godot-plugin-scenerunner)

gdpm deps list --path ./my/project
# > - plugin1 (v1.0.0) (source: Current)
# > - plugin2 (v1.0.0) (source ../plugin2)
```

### `deps sync` - synchronize dependencies

Synchronize/Install registered dependencies from the current project.  
You can pass another folder using the `--path` argument.

It will also scan the `addons` folder and register dependencies as `current` if they are not present in the dependency list.  
You can specify a plugin name to synchronize only one plugin.

*Examples:*

```bash
gdpm deps sync
# > Dependencies are now synchronized for project MyProject.

gdpm deps sync --path ./my/project
# > Dependencies are now synchronized for project MyProject2.

gdpm deps sync --path ./my/project plugin1
# > Dependency plugin1 is now synchronized for project MyProject2.
```

### `deps desync` - desynchronize dependencies

Desynchronize dependencies for the current project.  
You can pass another folder using the `--path` argument.

Il will remove installed plugins which are not from the project.  
You can specify a plugin name to desynchronize only one plugin.

*Examples:*

```bash
gdpm deps desync
# > Dependencies are desynchronized for project MyProject.

gdpm deps desync --path ./my/project
# > Dependencies are desynchronized for project MyProject2.

gdpm deps desync --path ./my/project plugin1
# > Dependency plugin1 is desynchronized for project MyProject2.
```

## `engine` - engine management

These are commands used to handle engines and versions.

### `engine list` - list registered engines

Get a list of the registered engines for your user.
The default engine will have a star at start.

*Examples:*

```bash
gdpm engine list
# * v3.1.1
#   v3.1.1.mono
#   v3.2.alpha3
#   vmaster

gdpm engine list --verbose
# * v3.1.1 (./path/to/3.1.1) [mono: false, built from source: false]
#   v3.1.1.mono (./path/to/3.1.1-mono) [mono: true, built from source: false]
#   v3.2.alpha3 (./path/to/3.2alpha3) [mono: false, built from source: false]
#   vmaster (./path/to/master) [mono: false, built from source: true]
```

### `engine list-remote` - list available engines on official mirror

Get a list of the available engines on the official Godot Engine mirror.
It will be cached on first run because it is quite slow and make a lot of requests to scan versions.
Use the `--no-cache` flag to reset the cache.

*Examples:*

```bash
gdpm engine list-remote
# - 3.1.1
# - 3.1.1.mono
# - 3.2.rc1.mono
# ...
```

### `engine register` - register an engine on your filesystem

Register/edit an engine for your user.  
You have to specify the `version`, the `path`, and `--mono` and/or `--source` if needed.

The first engine you register will automatically become the default engine.

*Examples:*

```bash
gdpm engine register 3.1.1 C:/.../3.1.1/godot.exe
# > Godot Engine v3.1.1 registered.

gdpm engine register 3.1.1.mono C:/.../3.1.1-mono/godot.exe --mono
# > Godot Engine v3.1.1.mono registered.

gdpm engine register master C:/.../master/bin/godot.exe --source
# > Godot Engine vmaster registered.
```

### `engine unregister` - unregister a known engine

Unregister an engine for your user.  
If you unregister the default engine, it will remain unset. You will have to call `engine default <version>`.

*Examples:*

```bash
gdpm engine unregister 3.2.alpha2
# > Godot Engine v3.2alpha2 unregistered.
```

### `engine start` - run a specific Godot Engine editor

Run a specific version of an engine, or run the default engine version if `version` is not set.

```bash
gdpm engine start
# > Running Godot Engine v3.1.1 ...

gdpm engine start master
# > Running Godot Engine vmaster ...
```

### `engine cmd` - execute a command on a specific Godot Engine editor

Execute a command on a specific version of an engine, or on the default engine version if `version` is not set.  

*Examples:*

```bash
gdpm engine cmd -- --test gui
# > Executing command --test gui on Godot Engine v3.2beta2 ...

gdpm engine cmd -- -h
# > Executing command -h on Godot Engine v3.2beta2 ...
```

### `engine set-default` - set the default engine version

Set a registered engine as default.

*Examples:*

```bash
gdpm engine set-default 3.1.1
# > Godot Engine v3.1.1 set as default.
```

### `engine get-default` - get the default engine version

Get the default engine version.

*Examples:*

```bash
gdpm engine get-default
# * Godot Engine v3.1.1
```

### `engine install` - download and install a remote Godot Engine version

Download and install engine from official mirror or specific URL.

*Examples:*

```bash
# Install 3.1.1
gdpm engine install 3.1.1
# Install 3.1.1.rc1
gdpm engine install 3.1.1.rc1
# Install 3.1.1.beta1
gdpm engine install 3.1.1.beta1
# Install 3.1.1.beta1.mono
gdpm engine install 3.1.1.beta1.mono
# Install 3.1.1 headless version (for linux x64 only)
gdpm engine install 3.1.1 --headless
# Install 3.1.1 server version (for linux x64 only)
gdpm engine install 3.1.1 --server
# Install 3.1.1, even if already present
gdpm engine install 3.1.1 --overwrite
# Install 3.1.1 from a custom URL
gdpm engine install 3.1.1 --target-url https://my.domain/godot.zip
```

### `engine uninstall` - uninstall a Godot Engine version

Uninstall a version installed with `engine install`.  
Use `engine unregister` to remove existing engines installations.

*Examples:*

```bash
# Uninstall 3.1.1
gdpm engine uninstall 3.1.1
# Uninstall 3.1.1.rc1
gdpm engine uninstall 3.1.1.rc1
```