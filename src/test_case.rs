extern crate yaml_rust;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use yaml_rust::YamlLoader;

/// Single test case for the project
/// Contains test name, test input (args and stdin), expected output, and test score
pub struct TestCase {
    pub name: String,
    pub score: f64,
    pub args: Vec<String>,
    pub stdin: String,
    pub stdout: String,
}

pub fn tests_from_yaml(yaml_file: &Path) -> Vec<TestCase> {
    let mut yaml_str = String::new();
    File::open(yaml_file)
        .expect("Error opening file with test specifications")
        .read_to_string(&mut yaml_str)
        .expect("Could not read file with test specifications");
    let yaml = YamlLoader::load_from_str(&yaml_str)
        .expect("Error parsing file with test specifications: not a YAML");

    match yaml[0]["tests"].as_vec() {
        Some(v) => v
            .iter()
            .map(|test| TestCase {
                name: test["name"].as_str().unwrap().to_string(),
                score: test["score"].as_f64().unwrap(),
                args: match test["args"].as_str() {
                    Some(args) => args.split_whitespace().map(String::from).collect(),
                    None => vec![],
                },
                stdin: test["stdin"].as_str().unwrap().to_string(),
                stdout: test["stdout"].as_str().unwrap().to_string(),
            })
            .collect(),
        None => vec![],
    }
}
