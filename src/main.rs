use async_std::path::PathBuf;
use async_std::task;
use lac::Error;
use lac::{mach, make_bin, remove_bin};
use std::env;

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Lossless Audio Checker Wrapper
struct Args {
    /// file path
    #[argh(
        positional,
        default = "PathBuf::from(env::current_dir().unwrap().to_str().unwrap())"
    )]
    path: PathBuf,

    /// number of jobs
    #[argh(option, short = 'j', default = "(num_cpus::get() + 3) / 4")]
    jobs: usize,

    /// force recheck
    #[argh(switch, short = 'f')]
    force: bool,

    /// do not print nonclean files
    #[argh(switch, short = 'n')]
    no_print: bool,
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
        mach(args.path, args.force, args.no_print, &bin).await?;
    }
    remove_bin().await?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();
    // SECRET EGG
    // if jobs==0 then jobs=cpu_cores_count
    // if !jobs then jobs=(num_cpus::get() + 3) / 4
    let jobs = if args.jobs == 0 {
        num_cpus::get()
    } else {
        args.jobs
    }
    // cores needs to be set before async
    std::env::set_var("ASYNC_STD_THREAD_COUNT", jobs.to_string());
    task::block_on(amain(args))
}
