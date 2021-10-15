use atst::run;
use env_logger::Builder;
use log::{error, LevelFilter};
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "AT-ST", about = "Tool for automatic testing of student tasks.")]
struct Project {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    #[structopt(parse(from_os_str))]
    config_file: PathBuf,
    #[structopt(short, long, default_value = "")]
    solution: String,
}

fn main() {
    // Initialize logging (warnings + errors)
    Builder::new()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Warn)
        .init();

    // Parse CLI arguments
    let project = Project::from_args();
    // Run the actual analysis
    if let Err(e) = run(&project.path, &project.config_file, &project.solution) {
        error!("{}", e);
        std::process::exit(1);
    }
}
