mod log;
mod processor;
use async_std::fs;
use async_std::path::Path;
use async_std::path::PathBuf;
use async_std::prelude::*;
use async_std::sync::RwLock;
use lac::Lac;
pub use log::Error;
use std::sync::Arc;

use log::Log;
use processor::Processor;

use crate::runner::log::report;
use crate::runner::log::FnF;
use async_std::task;
use futures::stream::FuturesUnordered;

use lac::bin::BIN_EXE;
use lac::bin::BIN_FILE;

/// Get header
async fn get_header(bin: &Path) -> Result<String, Error> {
    let out = async_std::process::Command::new(bin).output().await?;
    let output = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    Ok(output.lines().next().unwrap().to_owned())
}

/// Place bin from ram to temp folder
pub async fn make_bin(jobs: usize) -> Result<PathBuf, Error> {
    let tmp = PathBuf::from(std::env::temp_dir().join(BIN_EXE).to_str().unwrap());
    fs::write(tmp.clone(), BIN_FILE).await?;
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
pub async fn remove_bin() -> Result<(), Error> {
    let tmp = std::env::temp_dir().join(BIN_EXE);
    fs::remove_file(tmp).await?;
    Ok(())
}

pub async fn mach(dir: PathBuf, force: bool, no_print: bool, bin: &Path) -> Result<(), Error> {
    let procesor = Arc::new(RwLock::new(Processor::new(
        bin.to_owned(),
        get_header(bin).await?,
    )));
    let log = looper(procesor, force, dir).await?;
    if !no_print {
        for file in log.vectorize() {
            if let Ok(f) = &file.result {
                if *f != Lac::Clean {
                    println!("{}", file);
                }
            }
        }
    }
    Ok(())
}

/// do recursive loop in path for FLacs and WAVs and do logging
// firstly we need to read logs if they exist
// then recalc hashes
#[async_recursion::async_recursion]
async fn looper(procesor: Arc<RwLock<Processor>>, force: bool, dir: PathBuf) -> Result<Log, Error> {
    if !force {
        if let Ok(ff) = fs::read(dir.join("LAC.log")).await {
            procesor.write().await.append_old(Log::from(&ff)?)
        }
    }
    let mut log = Log::new();
    let mut tasks = FuturesUnordered::new();
    let mut entries = fs::read_dir(&dir).await?;
    while let Some(path) = entries.next().await {
        let procesor = procesor.clone();
        tasks.push(task::spawn(async move {
            let path = path?;
            if path.metadata().await?.is_file() {
                if let Some(ext) = path.path().extension() {
                    let ext = ext.to_str().unwrap().to_ascii_lowercase();
                    match ext.as_str() {
                        "flac" => {
                            return report(
                                procesor.read().await.process_flac(path.path()).await,
                                path.path(),
                            );
                        }
                        "wav" => {
                            return report(
                                procesor.read().await.process_wav(path.path()).await,
                                path.path(),
                            );
                        }
                        _ => { /* Do nothing */ }
                    }
                }
            } else {
                return Ok(FnF::Folder(looper(procesor, force, path.path()).await?));
            }
            Ok::<FnF, Error>(FnF::None)
        }));
    }
    while let Some(item) = tasks.next().await {
        match item {
            Ok(FnF::File(f)) => log.insert(f),
            Ok(FnF::Folder(f)) => log.append(f),
            Ok(FnF::None) => { /* Do nothing */ }
            Err(err) => {
                println!("{}", err)
            }
        }
    }
    fs::write(
        dir.join("LAC.log"),
        format!(
            "{}\n\n{:?}",
            procesor.read().await.header,
            log.relevant(dir.to_str().unwrap())
        ),
    )
    .await?;
    Ok(log)
}
