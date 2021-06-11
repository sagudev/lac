mod bin;
mod log;
mod processor;
use async_std::fs;
use async_std::path::Path;
use async_std::path::PathBuf;
use async_std::prelude::*;
use async_std::sync::RwLock;
use std::error::Error;
use std::sync::Arc;

use log::Log;
use processor::Processor;

use crate::log::FnF;
use async_std::task;
use futures::stream::FuturesUnordered;

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
    let procesor = Arc::new(RwLock::new(Processor::new(
        bin.to_owned(),
        get_header(&bin).await?,
    )));
    looper(procesor, dir).await?;
    Ok(())
}

/// do recursive loop in path for FLacs and WAVs and do logging
// firstly we need to read logs if they exist
// then recalc hashes
#[async_recursion::async_recursion]
async fn looper(procesor: Arc<RwLock<Processor>>, dir: PathBuf) -> Result<Log, Box<dyn Error>> {
    if let Ok(ff) = fs::read(dir.join("LAC.log")).await {
        procesor.write().await.append_old(Log::from(&ff)?)
    }
    let mut log = Log::new();
    let mut tasks = FuturesUnordered::new();
    let mut entries = fs::read_dir(&dir).await?;
    while let Some(path) = entries.next().await {
        let procesor = procesor.clone();
        tasks.push(task::spawn(async move {
            let path = path.unwrap();
            if path.metadata().await.unwrap().is_file() {
                if let Some(ext) = path.path().extension() {
                    let ext = ext.to_str().unwrap().to_ascii_lowercase();
                    match ext.as_str() {
                        "flac" => {
                            return FnF::File(
                                procesor
                                    .read()
                                    .await
                                    .process_flac(path.path())
                                    .await
                                    .unwrap(),
                            );
                        }
                        "wav" => {
                            return FnF::File(
                                procesor
                                    .read()
                                    .await
                                    .process_wav(path.path())
                                    .await
                                    .unwrap(),
                            );
                        }
                        _ => { /* Do nothing */ }
                    }
                }
            } else {
                return FnF::Folder(looper(procesor, path.path()).await.unwrap());
            }
            FnF::None
        }));
    }
    while let Some(item) = tasks.next().await {
        match item {
            FnF::File(f) => log.insert(f),
            FnF::Folder(f) => log.append(f),
            FnF::None => { /* Do nothing */ }
        }
    }
    fs::write(
        dir.join("LAC.log"),
        format!(
            "{}\n\n{}",
            procesor.read().await.header,
            log.relevant(dir.to_str().unwrap())
        ),
    )
    .await?;
    Ok(log)
}
