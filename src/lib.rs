mod bin;
mod log;
mod processor;
use async_std::fs;
use async_std::path::Path;
use async_std::path::PathBuf;
use std::error::Error;
use async_std::prelude::*;

use log::Log;
use processor::Processor;

/// Get header
async fn get_header(bin: &Path) -> Result<String, Box<dyn Error>> {
    let out = async_std::process::Command::new(bin).output().await?;
    let output = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    Ok(output.lines().next().unwrap().to_owned())
}

/// Place bin from ram to temp folder
pub async fn make_bin(jobs: usize) -> Result<PathBuf, Box<dyn Error>> {
    let tmp = PathBuf::from(std::env::temp_dir().join(bin::BIN_EXE).to_str().unwrap());
    fs::write(tmp.clone(), bin::BIN_FILE).await?;
    if cfg!(not(target_os = "windows")) {
        async_std::process::Command::new("chmod")
            .arg("+x")
            .arg(&tmp)
            .output()
            .await
            .expect("failed to execute chmod +x");
    }
    println!(
        "Using builtin {}\nthat should be able to span on {} thread(s)",
        get_header(&tmp).await?,
        jobs
    );
    Ok(tmp)
}

/// Remove bin from tmp folder
pub async fn remove_bin() -> Result<(), Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::remove_file(tmp).await?;
    Ok(())
}

pub async fn mach(dir: PathBuf, bin: &Path) -> Result<(), Box<dyn Error>> {
    let mut procesor = Processor::new(bin.to_owned());
    looper(&mut procesor, &get_header(&bin).await?, dir).await?;
    Ok(())
}

/// do recursive loop in path for FLacs and WAVs and do logging
// firstly we need to read logs if they exist
// then recalc hashes
#[async_recursion::async_recursion]
async fn looper(
    procesor: &mut Processor,
    header: &str,
    dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    if let Ok(ff) = fs::read(dir.join("LAC.log")).await {
        procesor.append_old(Log::from(&ff)?)
    }

    let mut entries = fs::read_dir(&dir).await?;
    while let Some(path) = entries.next().await {
        let path = path?;
        if path.metadata().await?.is_file() {
            if let Some(ext) = path.path().extension() {
                let ext = ext.to_str().unwrap().to_ascii_lowercase();
                match ext.as_str() {
                    "flac" => {
                        procesor.process_flac(path.path()).await?;
                    }
                    "wav" => {
                        procesor.process_wav(path.path()).await?;
                    }
                    _ => { /* Do nothing */ }
                }
            }
        } else {
            looper(procesor, header, path.path()).await?;
        }
    }
    fs::write(
        dir.join("LAC.log"),
        format!(
            "{}\n\n{}",
            header,
            procesor.log.relevant(dir.to_str().unwrap())
        ),
    )
    .await?;
    Ok(())
}
