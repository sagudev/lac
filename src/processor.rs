use async_std::fs;
use async_std::path::Path;
use async_std::path::PathBuf;
use std::error::Error;

use crate::log::Log;
use crate::log::{File, Lac};
use sha2::Digest;

/// Crates hash of file
fn hash(v: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    // write input message
    hasher.update(v);
    // read hash digest and consume hasher
    hex::encode(hasher.finalize().iter().copied().collect::<Vec<u8>>())
}

/// https://github.com/ruuda/claxon/blob/master/examples/decode_simple.rs#L18
fn decode_file(fname: &Path) -> PathBuf {
    let mut reader = claxon::FlacReader::open(fname).expect("failed to open FLac stream");

    let spec = hound::WavSpec {
        channels: reader.streaminfo().channels as u16,
        sample_rate: reader.streaminfo().sample_rate,
        bits_per_sample: reader.streaminfo().bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    let fname_wav = fname.with_extension("wav");
    let opt_wav_writer = hound::WavWriter::create(&fname_wav, spec);
    let mut wav_writer = opt_wav_writer.expect("failed to create wav file");

    for opt_sample in reader.samples() {
        let sample = opt_sample.expect("failed to decode FLac stream");
        wav_writer
            .write_sample(sample)
            .expect("failed to write wav file");
    }

    fname_wav
}

#[derive(Debug)]
pub struct Processor {
    old_log: Option<Log>,
    pub log: Log,
    bin: PathBuf,
}

impl Processor {
    pub fn new(bin: PathBuf) -> Self {
        Self {
            old_log: None,
            log: Log::new(),
            bin,
        }
    }

    pub fn append_old(&mut self, log: Log) {
        if let Some(old_log) = &mut self.old_log {
            old_log.append(log)
        } else {
            self.old_log = Some(log)
        }
    }

    /// checks if recalc is neede based on hash
    /// make dupe of old in new
    fn recalc_dupe(&mut self, path: &Path, hash: &str) -> bool {
        if let Some(old) = &self.old_log {
            let k = path.parent().unwrap();
            if old.data.contains_key(k) {
                for f in old.data.get(k).unwrap() {
                    if f.path == *path && f.hash == *hash {
                        // data is the same, copy
                        self.log.insert_or_update(f.clone());
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Runs Lac and parse result
    fn process(&self, path: &Path) -> Result<Result<Lac, String>, Box<dyn Error>> {
        let out = std::process::Command::new(&self.bin).arg(path).output()?;
        let output = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
        if output.contains("clean") {
            Ok(Ok(Lac::Clean))
        } else if output.contains("transcoded") {
            Ok(Ok(Lac::Transcoded))
        } else if output.contains("upscaled") {
            Ok(Ok(Lac::Upscaled))
        } else if output.contains("upsampled") {
            Ok(Ok(Lac::Upsampled))
        } else {
            Ok(Err(format!("{:#?}", out)))
        }
    }

    /// Process WAV file
    pub async fn process_wav(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let f = fs::read(&path).await?;
        let hash = hash(&f);
        if self.recalc_dupe(&path, &hash) {
            let result = self.process(&path)?;
            self.log.insert_or_update(File { path, hash, result })
        }
        Ok(())
    }

    /// Process FLac file
    pub async fn process_flac(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let f = fs::read(&path).await?;
        let hash = hash(&f);
        if self.recalc_dupe(&path, &hash) {
            let waw = decode_file(&path);
            let result = self.process(&waw)?;
            fs::remove_file(waw).await?;
            self.log.insert_or_update(File { path, hash, result })
        }
        Ok(())
    }
}
