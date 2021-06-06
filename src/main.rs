use llv::{mach, make_bin, remove_bin};
use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let path = {
        if args.len() == 1 {
            env::current_dir()?
        } else {
            PathBuf::from(&args[1])
        }
    };
    let bin = make_bin()?;
    mach(path, &bin)?;
    remove_bin()?;
    Ok(())
}
