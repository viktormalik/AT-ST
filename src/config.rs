extern crate yaml_rust;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use yaml_rust::YamlLoader;

/// Project configuration
/// Currently contains only a list of scripts that are evaluated during execution
/// Typically parsed from a YAML file
pub struct Config {
    // Basic information
    pub src_file: Option<String>,

    // Compiler information
    pub compiler: Option<String>,
    pub c_flags: Option<String>,
    pub ld_flags: Option<String>,

    // Additional scripts to be run
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
