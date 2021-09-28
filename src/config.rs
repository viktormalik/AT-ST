extern crate yaml_rust;

use crate::analyses::*;
use crate::TestCase;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlLoader};

/// Project configuration
/// Contains:
///   - compiler information
///   - list of test cases to evaluate the solutions on
///   - list of source analyses to run on the solutions
///   - list of additional scripts to be run on each solution
/// Typically parsed from a YAML file
pub struct Config {
    // Basic information
    pub src_file: Option<String>,

    // Compiler information
    pub compiler: Option<String>,
    pub c_flags: Option<String>,
    pub ld_flags: Option<String>,

    pub test_cases: Vec<TestCase>,
    pub analyses: Vec<Box<dyn Analyser>>,
    pub scripts: Vec<PathBuf>,
}

impl Config {
    pub fn from_yaml(yaml_file: &Path, project_path: &Path) -> Self {
        let mut yaml_str = String::new();
        File::open(project_path.join(yaml_file))
            .expect("Error opening configuration file")
            .read_to_string(&mut yaml_str)
            .expect("Could not read configuration file");
        let yaml = YamlLoader::load_from_str(&yaml_str)
            .expect("Error parsing configuration file: not a YAML");

        Self {
            src_file: yaml[0]["source"].as_str().map(String::from),
            compiler: yaml[0]["compiler"]["CC"].as_str().map(String::from),
            c_flags: yaml[0]["compiler"]["CFLAGS"].as_str().map(String::from),
            ld_flags: yaml[0]["compiler"]["LDFLAGS"].as_str().map(String::from),
            test_cases: tests_from_yaml(&yaml[0]),
            analyses: analyses_from_yaml(&yaml[0]),
            scripts: match yaml[0]["scripts"].as_vec() {
                Some(v) => v
                    .iter()
                    .filter_map(|s| s.as_str().map(|s| project_path.join(s)))
                    .collect(),
                None => vec![],
            },
        }
    }
}

pub fn tests_from_yaml(yaml: &Yaml) -> Vec<TestCase> {
    match yaml["tests"].as_vec() {
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

pub fn analyses_from_yaml(yaml: &Yaml) -> Vec<Box<dyn Analyser>> {
    match yaml["analyses"].as_vec() {
        Some(v) => v
            .iter()
            .filter_map(|analysis| match analysis["analyser"].as_str() {
                Some("no-call") => Some(Box::new(NoCallAnalyser::new(
                    analysis["funs"]
                        .as_vec()
                        .unwrap()
                        .iter()
                        .map(|f| f.as_str().unwrap())
                        .map(str::to_string)
                        .collect(),
                    analysis["penalty"].as_f64().unwrap(),
                )) as Box<dyn Analyser>),

                Some("no-header") => Some(Box::new(NoHeaderAnalyser::new(
                    analysis["header"].as_str().unwrap().to_string(),
                    analysis["penalty"].as_f64().unwrap(),
                )) as Box<dyn Analyser>),

                Some("no-globals") => Some(Box::new(NoGlobalsAnalyser::new(
                    analysis["penalty"].as_f64().unwrap(),
                )) as Box<dyn Analyser>),

                _ => None,
            })
            .collect(),
        None => vec![],
    }
}
