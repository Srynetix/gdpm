use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

use gdpm::parsers::{parse_gdproject_file, serialize_gdproperties};

#[test]
fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_dirs = manifest_dir
        .join("tests")
        .join("samples")
        .join("project_files");

    // Read each file
    for entry in fs::read_dir(project_dirs).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let mut string = String::new();
        let mut input = fs::File::open(path).unwrap();
        input.read_to_string(&mut string).unwrap();

        // Start test
        test_project_file(&string);
    }
}

fn test_project_file(file_contents: &str) {
    let data = parse_gdproject_file(file_contents).unwrap();

    assert_eq!(
        parse_gdproject_file(&serialize_gdproperties(&data)).unwrap(),
        data
    );
}
