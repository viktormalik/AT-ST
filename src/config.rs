extern crate yaml_rust;

use self::yaml_rust::Yaml;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use yaml_rust::YamlLoader;

/// Project configuration
/// Currently contains only a list of scripts that are evaluated during execution
/// Typically parsed from a YAML file
pub struct Config {
    pub scripts: Vec<PathBuf>,
}

impl Config {
    pub fn from_yaml(yaml_file: &Path, project_path: &Path) -> Self {
        let mut yaml_str = String::new();
        File::open(project_path.join(yaml_file))
            .unwrap()
            .read_to_string(&mut yaml_str)
            .expect("Could not read config file");
        let yaml = YamlLoader::load_from_str(&yaml_str).unwrap();

        Config {
            scripts: yaml[0]
                .as_hash()
                .unwrap()
                .get(&Yaml::from_str("scripts"))
                .unwrap()
                .as_vec()
                .unwrap()
                .iter()
                .map(|scr| project_path.join(scr.as_str().unwrap().to_string()))
                .collect(),
        }
    }
}
