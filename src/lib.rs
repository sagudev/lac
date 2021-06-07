mod bin;
mod log;
mod processor;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log::Log;
use processor::Processor;

use crate::log::get_header;

/// Place bin from ram to temp folder
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

/// Remove bin from tmp folder
pub fn remove_bin() -> Result<(), Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::remove_file(tmp)?;
    Ok(())
}

/// do recursive loop in path for FLacs and WAVs and do logging
// firstly we need to read logs if they exist
// then recalc hashes
//
pub fn mach(dir: PathBuf, bin: &Path) -> Result<(), Box<dyn Error>> {
    let log_old = if let Ok(ff) = fs::read(dir.join("Lac.log")) {
        Some(Log::from(&bin, &ff)?)
    } else {
        None
    };
    let mut procesor = Processor::new(log_old, Log::new(&bin)?, bin.to_owned());
    for path in fs::read_dir(&dir)? {
        let path = path?;
        if path.metadata()?.is_file() {
            if let Some(ext) = path.path().extension() {
                let ext = ext.to_str().unwrap().to_ascii_lowercase();
                match ext.as_str() {
                    "flac" => {
                        procesor.process_flac(path.path())?;
                    }
                    "wav" => {
                        procesor.process_wav(path.path())?;
                    }
                    _ => { /* Do nothing */ }
                }
            }
        } else {
            mach(path.path(), &bin)?;
        }
    }
    fs::write(dir.join("Lac.log"), format!("{}", procesor.log))?;
    Ok(())
}
