//! Godot project file parser

use std::collections::BTreeMap;

use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "gdsettings.pest"]
struct GdSettingsParser;

/// Godot value
#[derive(PartialEq, Debug)]
pub enum GdValue {
    /// Object
    Object(Vec<(String, GdValue)>),
    /// Array
    Array(Vec<GdValue>),
    /// String
    String(String),
    /// Int
    Int(i32),
    /// Float
    Float(f64),
    /// Boolean
    Boolean(bool),
    /// Class name
    ClassName(String),
    /// Class instance: a name, arguments and keyword arguments
    ClassInstance(String, Vec<GdValue>, Vec<(String, GdValue)>),
    /// Null
    Null,
}

/// Godot settings map
pub type GdSettings = BTreeMap<String, BTreeMap<String, GdValue>>;

/// Serialize a GdValue to string
///
/// # Arguments
///
/// * `val` - GdValue object
///
pub fn serialize_gdvalue(val: &GdValue) -> String {
    match val {
        GdValue::Object(o) => {
            let contents: Vec<_> = o
                .iter()
                .map(|(name, value)| format!("{}: {}", name, serialize_gdvalue(value)))
                .collect();
            format!("{{{}}}", contents.join(", "))
        }
        GdValue::Array(a) => {
            let contents: Vec<_> = a.iter().map(serialize_gdvalue).collect();
            format!("[{}]", contents.join(", "))
        }
        GdValue::ClassInstance(cls, args, kwargs) => {
            let args_content = args
                .iter()
                .map(serialize_gdvalue)
                .collect::<Vec<_>>()
                .join(", ");
            let kwargs_content = kwargs
                .iter()
                .map(|(name, value)| format!("{}: {}", name, serialize_gdvalue(value)))
                .collect::<Vec<_>>()
                .join(", ");

            if kwargs.is_empty() {
                format!("{}({})", cls, args_content)
            } else {
                format!("{}({}, {})", cls, args_content, kwargs_content)
            }
        }
        GdValue::ClassName(n) => n.to_string(),
        GdValue::String(s) => s.to_string(),
        GdValue::Int(n) => n.to_string(),
        GdValue::Float(n) => format!("{:.9}", n),
        GdValue::Boolean(b) => b.to_string(),
        GdValue::Null => "null".to_string(),
    }
}

/// Serialize a GdSettings object to String
///
/// # Arguments
///
/// * `settings` - GdSettings object
///
pub fn serialize_gdsettings(settings: &GdSettings) -> String {
    let mut output = String::new();

    fn write_props(hmap: &BTreeMap<String, GdValue>, output: &mut String) {
        for (k, v) in hmap.iter() {
            output.push_str(&k);
            output.push_str(" = ");
            output.push_str(&serialize_gdvalue(v));
            output.push_str("\n");
        }
    }

    // First, get global settings
    let globs = settings.get("");
    if let Some(hmap) = globs {
        write_props(hmap, &mut output);
        output.push_str("\n");
    }

    // Then
    for (k, v) in settings.iter() {
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
pub fn parse_gdsettings_file(contents: &str) -> Result<GdSettings, Error<Rule>> {
    use pest::iterators::Pair;

    let data = GdSettingsParser::parse(Rule::file, contents)?
        .next()
        .unwrap();
    let mut properties: GdSettings = BTreeMap::new();
    let mut current_section = "";

    fn parse_gdvalue(pair: Pair<Rule>) -> GdValue {
        match pair.as_rule() {
            Rule::object => GdValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let name = inner_rules.next().unwrap().as_str();
                        let value = parse_gdvalue(inner_rules.next().unwrap());
                        (name.to_string(), value)
                    })
                    .collect(),
            ),
            Rule::array => GdValue::Array(pair.into_inner().map(parse_gdvalue).collect()),
            Rule::string => GdValue::String(pair.as_str().to_string()),
            Rule::class_name => GdValue::ClassName(pair.as_str().to_string()),
            Rule::class_instance => {
                let mut inner_rules = pair.into_inner();
                let class_name = inner_rules.next().unwrap().as_str().to_string();
                let mut args = vec![];
                let mut kwargs = vec![];

                for pair in inner_rules {
                    match pair.as_rule() {
                        // Check for kwarg
                        Rule::pair => {
                            let mut inner_rules = pair.into_inner();
                            let name = inner_rules.next().unwrap().as_str();
                            let value = parse_gdvalue(inner_rules.next().unwrap());
                            kwargs.push((name.to_string(), value))
                        }
                        // Else convert
                        _ => args.push(parse_gdvalue(pair)),
                    }
                }

                GdValue::ClassInstance(class_name, args, kwargs)
            }
            Rule::int => GdValue::Int(pair.as_str().parse().unwrap()),
            Rule::float => GdValue::Float(pair.as_str().parse().unwrap()),
            Rule::boolean => GdValue::Boolean(pair.as_str().parse().unwrap()),
            Rule::null => GdValue::Null,
            _ => unreachable!(),
        }
    }

    for line in data.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner();
                current_section = inner_rules.next().unwrap().as_str();
            }
            Rule::property => {
                let mut inner_rules = line.into_inner();

                let name = inner_rules.next().unwrap().as_str().to_string();
                let value = inner_rules.next().unwrap();
                let value = parse_gdvalue(value);

                let section = properties.entry(current_section.to_string()).or_default();
                section.insert(name, value);
            }
            Rule::comment | Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    Ok(properties)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

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
