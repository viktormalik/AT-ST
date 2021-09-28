use atst::run;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "AT-ST", about = "Tool for automatic testing of student tasks.")]
struct Project {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    #[structopt(parse(from_os_str))]
    config_file: PathBuf,
}

fn main() {
    let project = Project::from_args();
    run(&project.path, &project.config_file)
}
