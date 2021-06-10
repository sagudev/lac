mod bin;
mod log;
mod processor;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log::Log;
use processor::Processor;

/// Get header
fn get_header(bin: &Path) -> Result<String, Box<dyn Error>> {
    let out = std::process::Command::new(bin).output()?;
    let output = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    Ok(output.lines().next().unwrap().to_owned())
}

/// Place bin from ram to temp folder
pub fn make_bin(jobs: usize) -> Result<PathBuf, Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::write(tmp.clone(), bin::BIN_FILE)?;
    if cfg!(not(target_os = "windows")) {
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(&tmp)
            .output()
            .expect("failed to execute chmod +x");
    }
    println!("Using builtin {}\nthat should be able to span on {} thread(s)", get_header(&tmp)?, jobs);
    Ok(tmp)
}

/// Remove bin from tmp folder
pub fn remove_bin() -> Result<(), Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::remove_file(tmp)?;
    Ok(())
}

pub fn mach(dir: PathBuf, jobs: usize, bin: &Path) -> Result<(), Box<dyn Error>> {
    let mut procesor = Processor::new(bin.to_owned());
    looper(&mut procesor, &get_header(bin)?, dir)?;
    Ok(())
}

/// do recursive loop in path for FLacs and WAVs and do logging
// firstly we need to read logs if they exist
// then recalc hashes
//
fn looper(procesor: &mut Processor, header: &str, dir: PathBuf) -> Result<(), Box<dyn Error>> {
    if let Ok(ff) = fs::read(dir.join("LAC.log")) {
        procesor.append_old(Log::from(&ff)?)
    }
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
            looper(procesor, header, path.path())?;
        }
    }
    fs::write(
        dir.join("LAC.log"),
        format!(
            "{}\n\n{}",
            header,
            procesor.log.relevant(dir.to_str().unwrap())
        ),
    )?;
    Ok(())
}
