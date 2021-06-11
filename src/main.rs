use async_std::path::PathBuf;
use lac::{mach, make_bin, remove_bin};
use std::env;
use std::error::Error;

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// A command with positional arguments.
struct Args {
    /// file path
    #[argh(
        positional,
        default = "PathBuf::from(env::current_dir().unwrap().to_str().unwrap())"
    )]
    path: PathBuf,

    /// number of jobs
    #[argh(option, short = 'j', default = "num_cpus::get() - 2")]
    jobs: usize,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();
    let bin = make_bin(args.jobs).await?;
    // if path is file run real Lac
    if args.path.is_file().await {
        async_std::process::Command::new(bin)
            .arg(args.path)
            .spawn()?;
    } else {
        mach(args.path, &bin).await?;
    }
    remove_bin().await?;
    Ok(())
}
