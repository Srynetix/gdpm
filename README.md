# gdpm - Godot Project Manager

A command line utility to manage a Godot 3.x project.  
Toy project (for now) written in Rust.

## Roadmap

- [x] Parse the Godot project file
- [x] Get project information using the project.godot file
- [ ] Write custom configuration in the project.godot file
- [ ] Manage dependencies (in the addons folder)
  - [ ] Sync from another project (copy)
  - [ ] Sync to another project
  - [ ] Sync from external source (zip)
- [ ] Execute custom actions
- [ ] Proxy commands to engine instance (like export)
- [ ] Manage Godot instances (maybe)

## Details

Dependencies will be added to the `project.godot` file, so we don't have to manage two project files.  
Godot recognize each entry in the file so if we add a `[dependencies]` section, it will show up in the project settings editor, so it can be manipulated from inside the engine.
