use crate::config::Config;
use crate::modules::{Module, ScriptExec};
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod modules;

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
    score: f64,
}

fn main() {
    let project = Project::from_args();
    let config = Config::from_yaml(&project.config_file, &project.path);

    // Solutions are sub-directories of the student directory starting with 'x'
    let solutions = project
        .path
        .read_dir()
        .expect("Could not read project directory")
        .filter_map(|res| res.ok())
        .filter(|entry| {
            entry.path().is_dir() && entry.file_name().into_string().unwrap().starts_with('x')
        })
        .map(|entry| Solution {
            path: entry.path(),
            score: 0.0,
        });

    // Create modules that will be run on each solution
    // For now, only custom scripts are supported
    let mut modules: Vec<Box<dyn Module>> = vec![];
    for script in config.scripts {
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
