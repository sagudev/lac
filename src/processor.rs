use std::error::Error;
use std::fs;
use std::path::PathBuf;

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

/// Runs LAC and parse result
fn process(path: &PathBuf, bin: &PathBuf) -> Result<Result<LAC, String>, Box<dyn Error>> {
    let out = std::process::Command::new(bin).arg(path).output()?;
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

/// Process WAV file
pub fn process_wav(path: PathBuf, bin: &PathBuf) -> Result<File, Box<dyn Error>> {
    let f = fs::read(&path)?;
    let hash = hash(&f);
    let result = process(&path, bin)?;
    Ok(File { path, hash, result })
}

/// Process FLAC file
pub fn process_flac(path: PathBuf, bin: &PathBuf) -> Result<File, Box<dyn Error>> {
    let f = fs::read(&path)?;
    let hash = hash(&f);
    let waw = decode_file(&path);
    let result = process(&waw, bin)?;
    fs::remove_file(waw)?;
    Ok(File { path, hash, result })
}
