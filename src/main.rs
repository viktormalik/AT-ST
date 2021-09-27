use crate::analyses::analyses_from_yaml;
use crate::config::Config;
use crate::modules::*;
use crate::test_case::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod analyses;
mod config;
mod modules;
mod test_case;

#[derive(StructOpt)]
#[structopt(name = "AT-ST", about = "Tool for automatic testing of student tasks.")]
struct Project {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    #[structopt(parse(from_os_str))]
    config_file: PathBuf,
}

/// One student task that is to be evaluated
pub struct Solution {
    path: PathBuf,
    src_file: PathBuf,
    obj_file: PathBuf,
    bin_file: PathBuf,

    included: Vec<String>,
    source: String,

    score: f64,
}

impl Solution {
    pub fn new(path: &Path, config: &Config) -> Self {
        let src_file = Path::new(config.src_file.as_ref().unwrap());
        Self {
            path: path.to_path_buf(),
            src_file: src_file.to_path_buf(),
            bin_file: PathBuf::from(src_file.file_stem().unwrap()),
            obj_file: src_file.with_extension("o"),
            included: vec![],
            source: String::new(),
            score: 0.0,
        }
    }
}

fn main() {
    let project = Project::from_args();
    let config = Config::from_yaml(&project.config_file, &project.path);
    let test_cases = tests_from_yaml(&project.path.join(&project.config_file));
    let analyses = analyses_from_yaml(&project.path.join(&project.config_file));

    // Solutions are sub-directories of the student directory starting with 'x'
    let solutions = project
        .path
        .read_dir()
        .expect("Could not read project directory")
        .filter_map(|res| res.ok())
        .filter(|entry| {
            entry.path().is_dir() && entry.file_name().into_string().unwrap().starts_with('x')
        })
        .map(|entry| Solution::new(&entry.path(), &config));

    // Create modules that will be run on each solution
    // Currently used modules:
    //  - compilation
    //  - source parsing
    //  - test cases execution
    //  - source analyses
    //  - custom scripts
    let mut modules: Vec<Box<dyn Module>> = vec![];
    modules.push(Box::new(Compiler::new(&config)));
    modules.push(Box::new(Parser {}));
    modules.push(Box::new(TestExec::new(test_cases)));
    modules.push(Box::new(AnalysesExec::new(analyses)));
    for script in &config.scripts {
        modules.push(Box::new(ScriptExec::new(script)));
    }

    // Evaluation - run all modules on each solution
    for mut solution in solutions {
        print!("{}: ", solution.path.file_name().unwrap().to_str().unwrap());
        for m in &modules {
            m.execute(&mut solution);
        }
        println!("{}", (solution.score * 100.0).round() / 100.0);
    }
}
