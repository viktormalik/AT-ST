use crate::Solution;
use regex::{Regex, RegexSet};
use std::process::Command;

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

impl NoCallAnalyser {
    pub fn new(funs: Vec<String>, penalty: f64) -> Self {
        Self { funs, penalty }
    }
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

/// Check that the program does not include the given header
pub struct NoHeaderAnalyser {
    header: String,
    penalty: f64,
}

impl NoHeaderAnalyser {
    pub fn new(header: String, penalty: f64) -> Self {
        Self { header, penalty }
    }
}

impl Analyser for NoHeaderAnalyser {
    fn analyse(&self, solution: &Solution) -> bool {
        solution.included.contains(&self.header)
    }

    fn penalty(&self) -> f64 {
        self.penalty
    }
}

/// Check that the program does not use global variables
/// Uses 'nm' on the object file.
pub struct NoGlobalsAnalyser {
    penalty: f64,
}

impl NoGlobalsAnalyser {
    pub fn new(penalty: f64) -> Self {
        Self { penalty }
    }
}

impl Analyser for NoGlobalsAnalyser {
    fn analyse(&self, solution: &Solution) -> bool {
        let nm_output = Command::new("nm")
            .arg(&solution.obj_file)
            .current_dir(&solution.path)
            .output()
            .unwrap();

        let symbols = std::str::from_utf8(&nm_output.stdout).unwrap();

        let re = Regex::new(r"\d*\s* [BD] (.*)").unwrap();
        re.is_match(symbols)
    }

    fn penalty(&self) -> f64 {
        self.penalty
    }
}
