use crate::Solution;
use std::path::PathBuf;
use std::process::Command;

/// Modules are used to prepare or evaluate individual project solutions
/// This trait is used to execute each module on a solution
pub trait Module {
    fn execute(&self, solution: &Solution);
}

/// Execute custom script provided by the user
/// This can be used if built-in modules are not sufficient
pub struct ScriptExec {
    script_path: PathBuf,
}

impl ScriptExec {
    pub fn new(script_path: PathBuf) -> Self {
        Self { script_path }
    }
}

impl Module for ScriptExec {
    /// Just run the script inside the solution directory
    fn execute(&self, solution: &Solution) {
        println!(
            "  {}",
            self.script_path.file_name().unwrap().to_str().unwrap()
        );
        let script = self.script_path.canonicalize().unwrap();
        Command::new(script)
            .current_dir(&solution.path)
            .status()
            .expect("Failed to execute script");
    }
}
