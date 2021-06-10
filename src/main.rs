use lac::{mach, make_bin, remove_bin};
use std::env;
use std::error::Error;
use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// A command with positional arguments.
struct Args {
    /// file path
    #[argh(positional, default = "env::current_dir().unwrap()")]
    path: PathBuf,

    /// number of jobs
    #[argh(option, short = 'j', default = "num_cpus::get() - 2")]
    jobs: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();
    let bin = make_bin(args.jobs)?;
    // if path is file run real Lac
    if args.path.is_file() {
        std::process::Command::new(bin).arg(args.path).spawn()?;
    } else {
        mach(args.path, args.jobs, &bin)?;
    }
    remove_bin()?;
    Ok(())
}
