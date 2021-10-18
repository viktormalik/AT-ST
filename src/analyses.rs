use crate::{AtstError, Solution};
use regex::{Regex, RegexSet};
use std::process::Command;

/// List of all supported analysers
pub enum AnalyserKind {
    NoCall,
    NoHeader,
    NoGlobals,

    Unsupported,
}

impl AnalyserKind {
    pub fn from(str: &str) -> Self {
        match str {
            "no-call" => AnalyserKind::NoCall,
            "no-header" => AnalyserKind::NoHeader,
            "no-globals" => AnalyserKind::NoGlobals,
            _ => AnalyserKind::Unsupported,
        }
    }
}

/// Source file analysis
/// If analyse() returns true, penalty() will be added to the solution score
pub trait Analyser {
    fn analyse(&self, solution: &Solution) -> Result<bool, AtstError>;
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
    fn analyse(&self, solution: &Solution) -> Result<bool, AtstError> {
        let re = RegexSet::new(self.funs.iter().map(|f| format!(r"{}\s*\(", f))).map_err(|_| {
            AtstError::InternalError {
                msg: "no-call analyser regex error".to_string(),
            }
        })?;
        Ok(re.is_match(&solution.source))
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
    fn analyse(&self, solution: &Solution) -> Result<bool, AtstError> {
        Ok(solution.included.contains(&self.header))
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
    fn analyse(&self, solution: &Solution) -> Result<bool, AtstError> {
        let nm_output = Command::new("nm")
            .arg(&solution.obj_file)
            .current_dir(&solution.path)
            .output()
            .map_err(|_| AtstError::ExecError("nm".to_string()))?;

        let symbols =
            std::str::from_utf8(&nm_output.stdout).map_err(|_| AtstError::InternalError {
                msg: "invalid output of nm".to_string(),
            })?;

        let re = Regex::new(r"\d*\s* [BCD] (.*)").map_err(|_| AtstError::InternalError {
            msg: "no-globals analyser regex error".to_string(),
        })?;
        Ok(re.is_match(symbols))
    }

    fn penalty(&self) -> f64 {
        self.penalty
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::test_utils::get_solution;

    fn test_on(analyser: &dyn Analyser, src: &str, included: &Vec<String>, expected: bool) {
        let mut solution = get_solution(src, true);
        solution.included = included.clone();
        let res = analyser.analyse(&solution);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
    }

    fn test_on_default(analyser: &dyn Analyser, expected: bool) {
        let src = r#"#include <stdio.h>
                     int x;
                     int main() {
                         printf("foo");
                     }
                  "#;
        test_on(analyser, src, &vec!["stdio.h".to_string()], expected);
    }

    #[test]
    fn no_call_analyser_match() {
        let analyser = NoCallAnalyser {
            funs: vec!["printf".to_string()],
            penalty: -1.0,
        };
        test_on_default(&analyser, true);
    }

    #[test]
    fn no_call_analyser_nomatch() {
        let analyser = NoCallAnalyser {
            funs: vec!["foo".to_string()],
            penalty: -1.0,
        };
        test_on_default(&analyser, false);
    }

    #[test]
    fn no_header_analyser_match() {
        let analyser = NoHeaderAnalyser {
            header: "stdio.h".to_string(),
            penalty: -1.0,
        };
        test_on_default(&analyser, true);
    }

    #[test]
    fn no_header_analyser_nomatch() {
        let analyser = NoHeaderAnalyser {
            header: "foo.h".to_string(),
            penalty: -1.0,
        };
        test_on_default(&analyser, false);
    }

    #[test]
    fn no_globals_analyser_match() {
        let analyser = NoGlobalsAnalyser { penalty: -1.0 };
        test_on_default(&analyser, true);
    }

    #[test]
    fn no_globals_analyser_nomatch() {
        let analyser = NoGlobalsAnalyser { penalty: -1.0 };
        test_on(&analyser, "int main() {}", &vec![], false);
    }
}
