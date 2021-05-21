use crate::config::Config;
use crate::test_case::TestCase;
use crate::Solution;
use std::fs::{read_to_string, remove_file};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Modules are used to prepare or evaluate individual project solutions
/// This trait is used to execute each module on a solution
pub trait Module {
    fn execute(&self, solution: &mut Solution);
}

/// C compiler
pub struct Compiler {
    compiler: String,
    c_flags: String,
    ld_flags: String,
}

impl Compiler {
    pub fn new(config: &Config) -> Self {
        Self {
            compiler: config.compiler.clone().unwrap_or("gcc".to_string()),
            c_flags: config.c_flags.clone().unwrap_or_default(),
            ld_flags: config.ld_flags.clone().unwrap_or_default(),
        }
    }
}

impl Module for Compiler {
    fn execute(&self, solution: &mut Solution) {
        let _ = remove_file(solution.path.join(&solution.bin_file));

        let mut cmd = Command::new(&self.compiler);
        cmd.args(self.c_flags.split_whitespace())
            .args(self.ld_flags.split_whitespace())
            .args(&["-o", &solution.bin_file.to_str().unwrap()])
            .arg(&solution.src_file)
            .current_dir(&solution.path);
        cmd.stderr(Stdio::null());

        if !cmd.status().expect("Error executing GCC").success() {
            return;
        }
        // Compile again with -Werror to see if there are warnings
        cmd.arg("-Werror");
        if !cmd.status().unwrap().success() {
            solution.score -= 0.5;
        }
    }
}

/// Running test cases
pub struct TestExec {
    test_cases: Vec<TestCase>,
}

impl TestExec {
    pub fn new(tests: Vec<TestCase>) -> Self {
        Self { test_cases: tests }
    }
}

impl Module for TestExec {
    fn execute(&self, solution: &mut Solution) {
        // Make sure that the executable exists
        let prog = solution.path.join(&solution.bin_file);
        if !prog.exists() {
            return;
        }

        for test_case in &self.test_cases {
            // Create process with correct arguments
            let mut cmd = Command::new(prog.clone())
                .args(&test_case.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            // Pass stdin to the process and capture its output
            let _ = cmd
                .stdin
                .as_mut()
                .unwrap()
                .write_all(test_case.stdin.as_bytes());
            let output = cmd.wait_with_output().unwrap();
            let stdout = std::str::from_utf8(&output.stdout);

            // Check if stdout matches the expected value
            // TODO: do not ignore whitespace
            if stdout.is_ok() && stdout.unwrap().trim() == test_case.stdout.trim() {
                solution.score += test_case.score;
            }
        }
    }
}

/// Execute custom script provided by the user
/// This can be used if built-in modules are not sufficient
pub struct ScriptExec {
    script_path: PathBuf,
}

impl ScriptExec {
    pub fn new(script_path: &PathBuf) -> Self {
        Self {
            script_path: script_path.clone(),
        }
    }
}

impl Module for ScriptExec {
    /// Just run the script inside the solution directory.
    /// If the script produces a log file (expected format: <script-name>.log), read it and for all
    /// lines starting with <number>:, add <number> to the total score of the solution.
    fn execute(&self, solution: &mut Solution) {
        let script_name = self.script_path.file_name().unwrap().to_str().unwrap();

        let script = self.script_path.canonicalize().unwrap();
        Command::new(script)
            .current_dir(&solution.path)
            .status()
            .expect("Failed to execute script");

        // Read the log file, if one is produced
        let log_file = solution.path.join(format!("{}.log", script_name));
        for line in read_to_string(log_file).unwrap_or_default().lines() {
            match line.split(':').nth(0).unwrap_or_default().parse::<f64>() {
                Ok(n) => solution.score += n,
                _ => {}
            }
        }
    }
}
