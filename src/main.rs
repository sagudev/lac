use async_std::path::PathBuf;
use async_std::task;
use lac::Error;
use lac::{mach, make_bin, remove_bin};
use std::env;

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
    #[argh(option, short = 'j', default = "num_cpus::get() / 4")]
    jobs: usize,

    /// number of jobs
    #[argh(switch, short = 'f')]
    force: bool,
}

//#[async_std::main]
async fn amain(args: Args) -> Result<(), Error> {
    let bin = make_bin(args.jobs).await?;
    // if path is file run real Lac
    if args.path.is_file().await {
        async_std::process::Command::new(bin)
            .arg(args.path)
            .spawn()?;
    } else {
        mach(args.path, args.force, &bin).await?;
    }
    remove_bin().await?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();
    // cores needs to be set before
    std::env::set_var("ASYNC_STD_THREAD_COUNT", args.jobs.to_string());
    task::block_on(amain(args))
}
