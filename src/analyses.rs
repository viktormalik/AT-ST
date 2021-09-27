use crate::Solution;
use regex::RegexSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use yaml_rust::YamlLoader;

/// Source file analysis
/// If analyse() returns true, penalty() will be added to the solution score
pub trait Analyser {
    fn analyse(&self, solution: &Solution) -> bool;
    fn penalty(&self) -> f64;
}

/// Check that the program does not call one of given functions
pub struct NoCallAnalyser {
    funs: Vec<String>,
    penalty: f64,
}

impl Analyser for NoCallAnalyser {
    fn analyse(&self, solution: &Solution) -> bool {
        let re = RegexSet::new(self.funs.iter().map(|f| format!(r"{}\s*\(", f))).unwrap();
        re.is_match(&solution.source)
    }

    fn penalty(&self) -> f64 {
        self.penalty
    }
}

pub fn analyses_from_yaml(yaml_file: &Path) -> Vec<Box<dyn Analyser>> {
    let mut yaml_str = String::new();
    File::open(yaml_file)
        .expect("Error opening file with test specifications")
        .read_to_string(&mut yaml_str)
        .expect("Could not read file with test specifications");
    let yaml = YamlLoader::load_from_str(&yaml_str)
        .expect("Error parsing file with test specifications: not a YAML");

    match yaml[0]["analyses"].as_vec() {
        Some(v) => v
            .iter()
            .filter_map(|analysis| match analysis["analyser"].as_str() {
                Some("no-call") => Some(Box::new(NoCallAnalyser {
                    funs: analysis["funs"]
                        .as_vec()
                        .unwrap()
                        .iter()
                        .map(|f| f.as_str().unwrap())
                        .map(str::to_string)
                        .collect(),
                    penalty: analysis["penalty"].as_f64().unwrap(),
                }) as Box<dyn Analyser>),
                _ => None,
            })
            .collect(),
        None => vec![],
    }
}
