use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::log::LOG;
use crate::log::{File, LAC};
use sha2::Digest;

/// Crates hash of file
fn hash(v: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    // write input message
    hasher.update(v);
    // read hash digest and consume hasher
    hex::encode(hasher.finalize().iter().map(|x| *x).collect::<Vec<u8>>())
}

/// https://github.com/ruuda/claxon/blob/master/examples/decode_simple.rs#L18
fn decode_file(fname: &PathBuf) -> PathBuf {
    let mut reader = claxon::FlacReader::open(fname).expect("failed to open FLAC stream");

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
        let sample = opt_sample.expect("failed to decode FLAC stream");
        wav_writer
            .write_sample(sample)
            .expect("failed to write wav file");
    }

    fname_wav
}

#[derive(Debug)]
pub struct Processor {
    old_log: Option<LOG>,
    pub log: LOG,
    bin: PathBuf,
}

impl Processor {
    pub fn new(old_log: Option<LOG>, log: LOG, bin: PathBuf) -> Self {
        Self { old_log, log, bin }
    }

    /// checks if recalc is neede based on hash
    /// make dupe of old in new
    fn recalc_dupe(&mut self, path: &PathBuf, hash: &str) -> bool {
        if let Some(old) = &self.old_log {
            let k = path.parent().unwrap();
            println!("{:#?}", old);
            if old.data.contains_key(k) {
                println!("2");
                for f in old.data.get(k).unwrap() {
                    println!("{}", f);
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

    /// Runs LAC and parse result
    fn process(&self, path: &PathBuf) -> Result<Result<LAC, String>, Box<dyn Error>> {
        let out = std::process::Command::new(&self.bin).arg(path).output()?;
        let output = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
        if output.contains("clean") {
            Ok(Ok(LAC::Clean))
        } else if output.contains("transcoded") {
            Ok(Ok(LAC::Transcoded))
        } else if output.contains("upscaled") {
            Ok(Ok(LAC::Upscaled))
        } else if output.contains("upsampled") {
            Ok(Ok(LAC::Upsampled))
        } else {
            Ok(Err(format!("{:#?}", out)))
        }
    }

    /// Process WAV file
    pub fn process_wav(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let f = fs::read(&path)?;
        let hash = hash(&f);
        if self.recalc_dupe(&path, &hash) {
            let result = self.process(&path)?;
            self.log.insert_or_update(File { path, hash, result })
        }
        Ok(())
    }

    /// Process FLAC file
    pub fn process_flac(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let f = fs::read(&path)?;
        let hash = hash(&f);
        if self.recalc_dupe(&path, &hash) {
            let waw = decode_file(&path);
            let result = self.process(&waw)?;
            fs::remove_file(waw)?;
            self.log.insert_or_update(File { path, hash, result })
        }
        Ok(())
    }
}
