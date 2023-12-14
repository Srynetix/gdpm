use std::{path::Path, str::FromStr};

use gdpm_io::IoAdapter;
use gdsettings_parser::{GdSettings, GdSettingsMap, GdSettingsType, GdValue};

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unknown project version: {0}")]
    UnknownProjectVersion(String),
    #[error("Unknown project renderer: {0}")]
    UnknownProjectRenderer(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectRenderer {
    ForwardPlus,
    Mobile,
    Compatibility,
}

impl FromStr for ProjectRenderer {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "forward+" => Ok(Self::ForwardPlus),
            "mobile" => Ok(Self::Mobile),
            "compatibility" => Ok(Self::Compatibility),
            other => Err(ParseError::UnknownProjectRenderer(other.into())),
        }
    }
}

impl ProjectRenderer {
    pub fn to_full_name(&self) -> &'static str {
        match self {
            Self::ForwardPlus => "Forward+",
            Self::Compatibility => "GL Compatibility",
            Self::Mobile => "Mobile",
        }
    }

    pub fn to_technical_name(&self) -> &'static str {
        match self {
            Self::ForwardPlus => "forward+",
            Self::Mobile => "mobile",
            Self::Compatibility => "gl_compatibility",
        }
    }
}

pub struct ProjectInfo {
    game_name: String,
    renderer: ProjectRenderer,
}

impl ProjectInfo {
    pub fn new(game_name: String, renderer: ProjectRenderer) -> Self {
        Self {
            game_name,
            renderer,
        }
    }
}

pub struct ProjectScaffoldV4 {
    engine_version: String,
    project_info: ProjectInfo,
}

impl ProjectScaffoldV4 {
    pub fn new(engine_version: String, project_info: ProjectInfo) -> Self {
        Self {
            engine_version,
            project_info,
        }
    }

    pub fn scaffold(&self, io: &impl IoAdapter, path: impl AsRef<Path>) -> Result<(), Error> {
        // Create folder
        if !io.path_exists(path.as_ref()) {
            io.create_dir(path.as_ref())?;
        }

        self.scaffold_project_godot(io, path.as_ref())?;
        self.scaffold_icon(io, path.as_ref())?;
        self.scaffold_gitattributes(io, path.as_ref())?;
        self.scaffold_gitignore(io, path.as_ref())?;

        Ok(())
    }

    fn scaffold_project_godot(&self, io: &impl IoAdapter, path: &Path) -> Result<(), Error> {
        let mut map = GdSettingsType::new();

        let mut global_map = GdSettingsMap::new();
        global_map.insert("config_version".into(), GdValue::Int(5));

        let mut app_map = GdSettingsMap::new();
        app_map.insert(
            "config/name".into(),
            GdValue::String(self.project_info.game_name.clone()),
        );
        app_map.insert(
            "config/features".into(),
            GdValue::ClassInstance(
                "PackedStringArray".into(),
                vec![
                    GdValue::String(self.engine_version.clone()),
                    GdValue::String(self.project_info.renderer.to_full_name().to_owned()),
                ],
                vec![],
            ),
        );
        app_map.insert(
            "config/icon".into(),
            GdValue::String("res://icon.svg".into()),
        );

        let mut rendering_map = GdSettingsMap::new();
        rendering_map.insert(
            "renderer/rendering_method".into(),
            GdValue::String(self.project_info.renderer.to_technical_name().to_owned()),
        );
        rendering_map.insert(
            "renderer/rendering_method.mobile".into(),
            GdValue::String(self.project_info.renderer.to_technical_name().to_owned()),
        );

        let mut engine_map = GdSettingsMap::new();
        engine_map.insert(
            "version".into(),
            GdValue::String(self.engine_version.clone()),
        );

        map.insert("".into(), global_map);
        map.insert("application".into(), app_map);
        map.insert("rendering".into(), rendering_map);
        map.insert("engine".into(), engine_map);

        let gdsettings = GdSettings::new(map);
        io.write_string_to_file(&path.join("project.godot"), &gdsettings.to_string())?;
        Ok(())
    }

    fn scaffold_icon(&self, io: &impl IoAdapter, path: &Path) -> Result<(), Error> {
        static ICON: &str = r###"<svg height="128" width="128" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="2" width="124" height="124" rx="14" fill="#363d52" stroke="#212532" stroke-width="4"/><g transform="scale(.101) translate(122 122)"><g fill="#fff"><path d="M105 673v33q407 354 814 0v-33z"/><path fill="#478cbf" d="m105 673 152 14q12 1 15 14l4 67 132 10 8-61q2-11 15-15h162q13 4 15 15l8 61 132-10 4-67q3-13 15-14l152-14V427q30-39 56-81-35-59-83-108-43 20-82 47-40-37-88-64 7-51 8-102-59-28-123-42-26 43-46 89-49-7-98 0-20-46-46-89-64 14-123 42 1 51 8 102-48 27-88 64-39-27-82-47-48 49-83 108 26 42 56 81zm0 33v39c0 276 813 276 813 0v-39l-134 12-5 69q-2 10-14 13l-162 11q-12 0-16-11l-10-65H447l-10 65q-4 11-16 11l-162-11q-12-3-14-13l-5-69z"/><path d="M483 600c3 34 55 34 58 0v-86c-3-34-55-34-58 0z"/><circle cx="725" cy="526" r="90"/><circle cx="299" cy="526" r="90"/></g><g fill="#414042"><circle cx="307" cy="532" r="60"/><circle cx="717" cy="532" r="60"/></g></g></svg>"###;

        io.write_string_to_file(&path.join("icon.svg"), ICON)?;
        Ok(())
    }

    fn scaffold_gitignore(&self, io: &impl IoAdapter, path: &Path) -> Result<(), Error> {
        static DATA: &str = "# Godot 4+ specific ignores\n.godot/";

        io.write_string_to_file(&path.join(".gitignore"), DATA)?;
        Ok(())
    }

    fn scaffold_gitattributes(&self, io: &impl IoAdapter, path: &Path) -> Result<(), Error> {
        static DATA: &str =
            "# Normalize EOL for all files that Git considers text files.\n* text=auto eol=lf";

        io.write_string_to_file(&path.join(".gitattributes"), DATA)?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] gdpm_io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub struct Scaffolder<'a, I: IoAdapter> {
    io: &'a I,
}

impl<'a, I: IoAdapter> Scaffolder<'a, I> {
    pub fn new(io: &'a I) -> Self {
        Self { io }
    }

    pub fn scaffold(
        &self,
        engine_version: String,
        project_info: ProjectInfo,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        ProjectScaffoldV4::new(engine_version, project_info).scaffold(self.io, path)
    }
}
