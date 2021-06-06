mod bin;
mod log;
mod processor;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::processor::get_header;
use crate::processor::process_flac;
use crate::processor::process_wav;

pub fn make_bin() -> Result<PathBuf, Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::write(tmp.clone(), bin::BIN_FILE)?;
    if cfg!(not(target_os = "windows")) {
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(&tmp)
            .output()
            .expect("failed to execute chmod +x");
    }
    println!("Using builtin {}", get_header(&tmp)?);
    Ok(tmp)
}

pub fn remove_bin() -> Result<(), Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::remove_file(tmp)?;
    Ok(())
}

// do recursive loop in path for FLACs and WAVs and log in files
// firstly we need to read logs if they exist
// then recalc hashes
//
pub fn mach(dir: PathBuf, bin: &PathBuf) -> Result<(), Box<dyn Error>> {
    if let Ok(ff) = fs::read(dir.join("LAC.log")) {
        // parse
    } else {
        println!("INFO: LAC.log does not exist in {:?}", dir);
    }
    for path in fs::read_dir(dir)? {
        let path = path?;
        if path.metadata()?.is_file() {
            if let Some(ext) = path.path().extension() {
                let ext = ext.to_str().unwrap().to_ascii_lowercase();
                match ext.as_str() {
                    "flac" => {
                        println!("{}", process_flac(path.path(), bin)?);
                    }
                    "wav" => {
                        println!("{}", process_wav(path.path(), bin)?);
                    }
                    _ => { /* Do nothing */ }
                }
            }
        } else {
            mach(path.path(), &bin)?;
        }
    }
    Ok(())
}
