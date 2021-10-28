//! Godot project file parser

use std::{
    collections::BTreeMap,
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::GdValue;

#[derive(Parser)]
#[grammar = "gdsettings.pest"]
struct GdSettingsParser;

/// Parser error
#[derive(Debug, Error)]
pub enum ParserError {
    /// Parse error
    #[error("parse error")]
    ParseError,

    /// Pest error
    #[error("pest error: {0}")]
    PestError(String),

    /// Type conversion error
    #[error("type conversion error: {0}")]
    TypeConversionError(String),
}

impl<E> From<pest::error::Error<E>> for ParserError
where
    E: std::fmt::Debug,
{
    fn from(error: pest::error::Error<E>) -> Self {
        Self::PestError(format!("{:?}", error))
    }
}

impl From<ParseIntError> for ParserError {
    fn from(error: ParseIntError) -> Self {
        Self::TypeConversionError(format!("{:?}", error))
    }
}

impl From<ParseFloatError> for ParserError {
    fn from(error: ParseFloatError) -> Self {
        Self::TypeConversionError(format!("{:?}", error))
    }
}

impl From<ParseBoolError> for ParserError {
    fn from(error: ParseBoolError) -> Self {
        Self::TypeConversionError(format!("{:?}", error))
    }
}

/// GdSettings error
#[derive(Debug, Error)]
pub enum GdSettingsError {
    /// Missing section
    #[error("missing section: {}", section)]
    MissingSection {
        /// Section
        section: String,
    },
    /// Missing property
    #[error("missing property: {}", property)]
    MissingProperty {
        /// Property
        property: String,
    },
}

/// Godot settings map
pub type GdSettingsMap = BTreeMap<String, GdValue>;
/// Godot settings type
pub type GdSettingsType = BTreeMap<String, GdSettingsMap>;

/// GdSettings
#[derive(PartialEq, Debug, Clone)]
pub struct GdSettings(GdSettingsType);

impl GdSettings {
    /// Create a new wrapper
    ///
    /// # Arguments
    ///
    /// * `map` - Map instance
    ///
    pub fn new(map: GdSettingsType) -> Self {
        Self(map)
    }

    /// Get property.
    ///
    /// # Arguments
    ///
    /// * `section` - Section name
    /// * `property` - Property name
    ///
    pub fn get_property(&self, section: &str, property: &str) -> Option<GdValue> {
        self.0.get(section)?.get(property).cloned()
    }

    /// Get section
    ///
    /// # Arguments
    ///
    /// * `section` - Section name
    ///
    pub fn get_section(&self, section: &str) -> Option<GdSettingsMap> {
        self.0.get(section).cloned()
    }

    /// Set property
    ///
    /// # Arguments
    ///
    /// * `section` - Section name
    /// * `property` - Property name
    ///
    pub fn set_property(&mut self, section: &str, property: &str, value: GdValue) {
        let section_entry = self
            .0
            .entry(section.to_string())
            .or_insert_with(GdSettingsMap::new);
        section_entry.insert(property.to_string(), value);
    }

    /// Remove property
    ///
    /// # Arguments
    ///
    /// * `section` - Section name
    /// * `property` - Property name
    ///
    pub fn remove_property(
        &mut self,
        section: &str,
        property: &str,
    ) -> Result<(), GdSettingsError> {
        let section_entry = self
            .0
            .get_mut(section)
            .ok_or(GdSettingsError::MissingSection {
                section: section.to_string(),
            })?;
        if section_entry.get(property).is_none() {
            return Err(GdSettingsError::MissingProperty {
                property: property.to_string(),
            });
        }

        section_entry.remove(property);

        Ok(())
    }

    /// Get map
    pub fn get_map(&self) -> &GdSettingsType {
        &self.0
    }
}

impl ToString for GdSettings {
    fn to_string(&self) -> String {
        serialize_gdsettings(&self)
    }
}

/// Serialize a GdSettings object to String
///
/// # Arguments
///
/// * `settings` - GdSettingsType object
///
pub fn serialize_gdsettings(settings: &GdSettings) -> String {
    let mut output = String::new();

    fn write_props(hmap: &BTreeMap<String, GdValue>, output: &mut String) {
        for (k, v) in hmap.iter() {
            output.push_str(&k);
            output.push_str(" = ");
            output.push_str(&v.to_string());
            output.push_str("\n");
        }
    }

    let map = settings.get_map();

    // First, get global settings
    let globs = map.get("");
    if let Some(hmap) = globs {
        write_props(hmap, &mut output);
        output.push_str("\n");
    }

    // Then
    for (k, v) in map.iter() {
        if k != "" {
            output.push_str("[");
            output.push_str(k);
            output.push_str("]");
            output.push_str("\n");
            write_props(v, &mut output);
            output.push_str("\n");
        }
    }

    output
}

/// Parse Godot settings file
///
/// # Arguments
///
/// * `contents` - File contents
///
pub fn parse_gdsettings_file(contents: &str) -> Result<GdSettings, ParserError> {
    use pest::iterators::Pair;

    let data = GdSettingsParser::parse(Rule::file, contents)?
        .next()
        .ok_or(ParserError::ParseError)?;
    let mut properties: GdSettingsType = BTreeMap::new();
    let mut current_section = "";

    fn parse_gdvalue(pair: Pair<Rule>) -> Result<GdValue, ParserError> {
        let value = match pair.as_rule() {
            Rule::object => GdValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let name = inner_rules
                            .next()
                            .ok_or(ParserError::ParseError)?
                            .as_str()
                            .trim_matches('"');
                        let value =
                            parse_gdvalue(inner_rules.next().ok_or(ParserError::ParseError)?)?;
                        Ok((name.to_string(), value))
                    })
                    .collect::<Result<Vec<(String, GdValue)>, ParserError>>()?,
            ),
            Rule::array => GdValue::Array(
                pair.into_inner()
                    .map(parse_gdvalue)
                    .collect::<Result<Vec<GdValue>, ParserError>>()?,
            ),
            Rule::string => GdValue::String(pair.as_str().trim_matches('"').to_string()),
            Rule::class_name => GdValue::ClassName(pair.as_str().to_string()),
            Rule::class_instance => {
                let mut inner_rules = pair.into_inner();
                let class_name = inner_rules
                    .next()
                    .ok_or(ParserError::ParseError)?
                    .as_str()
                    .to_string();
                let mut args = vec![];
                let mut kwargs = vec![];

                for pair in inner_rules {
                    match pair.as_rule() {
                        // Check for kwarg
                        Rule::pair => {
                            let mut inner_rules = pair.into_inner();
                            let name = inner_rules.next().ok_or(ParserError::ParseError)?.as_str();
                            let value =
                                parse_gdvalue(inner_rules.next().ok_or(ParserError::ParseError)?)?;
                            kwargs.push((name.to_string(), value))
                        }
                        // Else convert
                        _ => args.push(parse_gdvalue(pair)?),
                    }
                }

                GdValue::ClassInstance(class_name, args, kwargs)
            }
            Rule::int => GdValue::Int(pair.as_str().parse()?),
            Rule::float => GdValue::Float(pair.as_str().parse()?),
            Rule::boolean => GdValue::Boolean(pair.as_str().parse()?),
            Rule::null => GdValue::Null,
            _ => unreachable!(),
        };

        Ok(value)
    }

    for line in data.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner();
                current_section = inner_rules.next().ok_or(ParserError::ParseError)?.as_str();
                properties.insert(current_section.to_string(), BTreeMap::new());
            }
            Rule::property => {
                let mut inner_rules = line.into_inner();

                let name = inner_rules
                    .next()
                    .ok_or(ParserError::ParseError)?
                    .as_str()
                    .to_string();
                let value = inner_rules.next().ok_or(ParserError::ParseError)?;
                let value = parse_gdvalue(value)?;

                let section = properties.entry(current_section.to_string()).or_default();
                section.insert(name, value);
            }
            Rule::comment | Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    Ok(GdSettings::new(properties))
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::Path};

    use super::*;

    #[test]
    fn full_serializer_test() {
        // Get first project file in tests/samples/project_files
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let project_dirs = manifest_dir
            .join("tests")
            .join("samples")
            .join("project_files");
        let project_file = project_dirs.join("project2.godot");

        let mut content = String::new();
        let mut input = File::open(project_file).unwrap();
        input.read_to_string(&mut content).unwrap();

        let data = parse_gdsettings_file(&content).unwrap();
        let ser = serialize_gdsettings(&data);

        let serdata = parse_gdsettings_file(&ser).unwrap();
        assert_eq!(data, serdata);
    }

    #[test]
    fn full_parser_test() {
        // Get first project file in tests/samples/project_files
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let project_dirs = manifest_dir
            .join("tests")
            .join("samples")
            .join("project_files");
        let project_file = project_dirs.join("project2.godot");

        let mut content = String::new();
        let mut input = File::open(project_file).unwrap();
        input.read_to_string(&mut content).unwrap();

        let data = parse_gdsettings_file(&content).unwrap();
        serialize_gdsettings(&data);
    }

    #[test]
    fn parser_test() {
        // Get first project file in tests/samples/project_files
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let project_dirs = manifest_dir
            .join("tests")
            .join("samples")
            .join("project_files");
        let project_file = project_dirs.join("project1.godot");

        let mut content = String::new();
        let mut input = File::open(project_file).unwrap();
        input.read_to_string(&mut content).unwrap();

        GdSettingsParser::parse(Rule::file, &content).unwrap();
    }

    #[test]
    fn comments_parser_test() {
        let content = "\
; Engine configuration file.
; It's best edited using the editor UI and not directly,
; since the parameters that go here are not all obvious.
;
; Format:
;   [section] ; section goes between []
;   param=value ; assign values to parameters";

        GdSettingsParser::parse(Rule::file, &content).expect("failed to parse");
    }

    #[test]
    fn globals_parser_test() {
        let content = r###"
config_version=4

_global_script_classes=[  ]
_global_script_class_icons={}

[application]

config/name="Empty Project"
config/icon="res://icon.png"

[rendering]

environment/default_environment="res://default_env.tres""###;

        GdSettingsParser::parse(Rule::file, &content).expect("failed to parse");
    }
}
