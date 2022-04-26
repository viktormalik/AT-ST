mod analyses;
mod config;
mod modules;

use config::Config;
use log::warn;
use modules::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// One student task that is to be evaluated
#[derive(Default)]
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
        let src_file = Path::new(&config.src_file);
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

/// Single test case for the project
/// Contains test input (args and stdin) and expected output
#[derive(Default)]
pub struct TestCase {
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
}

pub enum TestCasesRequirement {
    ALL,
    ANY,
}

impl Default for TestCasesRequirement {
    fn default() -> Self {
        TestCasesRequirement::ALL
    }
}

/// A scored test for the project
/// Contains test `name`, `score`, and a list of test `cases`.
/// The `requirement` field specifies when the score is awarded. Current possible values are:
///   - `ALL`: all test cases must pass
///   - `ANY`: at least one test case must pass
#[derive(Default)]
pub struct Test {
    pub name: String,
    pub score: f64,
    pub test_cases: Vec<TestCase>,
    pub requirement: TestCasesRequirement,
}

pub const DEFAULT_TEST_TIMEOUT: u64 = 5000;

#[derive(Error, Debug)]
pub enum AtstError {
    #[error("Configuration error: {source}")]
    ConfigError {
        #[from]
        source: config::ConfigError,
    },
    #[error("error executing '{0}' (not installed?)")]
    ExecError(String),
    #[error("Internal error: {msg}")]
    InternalError { msg: String },
    #[error("solution execution error: {source}")]
    SolutionExecErr {
        #[from]
        source: std::io::Error,
    },
}

/// Main entry point of the program
/// Runs evaluation of all tests in `path` as defined in `config_file`
/// If `solution` is set, only evaluate that solution
pub fn run(
    path: &PathBuf,
    config_file: &PathBuf,
    only_solution: &str,
) -> Result<HashMap<String, f64>, AtstError> {
    let config = Config::from_yaml(&config_file, &path)?;

    let mut solutions = vec![];

    if !only_solution.is_empty() {
        // Single solution
        let s = Solution::new(&path.join(only_solution), &config);
        if s.path.exists() {
            solutions.push(s);
        } else {
            warn!("Selected solution does not exist");
        }
    } else {
        // Solutions are sub-dirs of the project directory except those explicitly excluded
        solutions = path
            .read_dir()
            .map_err(|_| AtstError::InternalError {
                msg: "could not read project directory".to_string(),
            })?
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry.path().is_dir()
                    && !config
                        .excluded_dirs
                        .contains(&entry.file_name().into_string().unwrap())
            })
            .map(|entry| Solution::new(&entry.path(), &config))
            .collect();
    }

    if solutions.is_empty() {
        warn!("No solutions to analyse");
        return Ok(HashMap::new());
    }

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
    modules.push(Box::new(TestExec::new(&config.tests, config.timeout)));
    modules.push(Box::new(AnalysesExec::new(&config.analyses)));
    for script in &config.scripts {
        modules.push(Box::new(ScriptExec::new(script)));
    }

    let mut result = HashMap::new();
    // Evaluation - run all modules on each solution
    for mut solution in solutions {
        let name = solution
            .path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        print!("{}: ", name);

        let src_file = &solution.path.join(&solution.src_file);
        if !src_file.exists() {
            println!("no source found");
            continue;
        }

        for m in &modules {
            m.execute(&mut solution)?;
        }
        println!("{}", (solution.score * 100.0).round() / 100.0);
        result.insert(name.to_string(), solution.score);
    }

    Ok(result)
}

#[cfg(test)]
mod test_utils {
    use super::Solution;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    /// Create a new solution from the given source for testing purposes
    ///
    /// Creates a temporary solution directory DIR and writes `src` to DIR/test.c.
    /// If `compile` is set, builds the source into an object file (DIR/test.o)
    /// and an executable (DIR/test).
    pub fn get_solution(src: &str, compile: bool) -> Solution {
        let dir = tempfile::tempdir().unwrap();

        let src_file_name = PathBuf::from("test.c");
        let obj_file_name = PathBuf::from("test.o");
        let bin_file_name = PathBuf::from("test");

        let mut src_file = File::create(dir.path().join(&src_file_name)).unwrap();
        let _ = src_file.write(src.as_bytes());

        if compile {
            let _ = Command::new("gcc")
                .arg("-c")
                .arg(&src_file_name)
                .arg("-o")
                .arg(&obj_file_name)
                .current_dir(&dir)
                .stderr(Stdio::null())
                .status();

            let _ = Command::new("gcc")
                .arg(&obj_file_name)
                .arg("-o")
                .arg(&bin_file_name)
                .current_dir(&dir)
                .stderr(Stdio::null())
                .status();
        }

        Solution {
            path: dir.into_path(),
            src_file: src_file_name.clone(),
            obj_file: obj_file_name.clone(),
            bin_file: bin_file_name.clone(),
            source: src.to_string(),
            ..Default::default()
        }
    }
}
