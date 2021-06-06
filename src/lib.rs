mod bin;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

pub fn make_bin() -> Result<PathBuf, Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::write(tmp.clone(), bin::BIN_FILE)?;
    if cfg!(not(target_os = "windows")) {
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(tmp.clone())
            .output()
            .expect("failed to execute chmod +x");
    }
    Ok(tmp)
}

pub fn remove_bin() -> Result<(), Box<dyn Error>> {
    let tmp = std::env::temp_dir().join(bin::BIN_EXE);
    fs::remove_file(tmp)?;
    Ok(())
}

use sha2::Digest;
fn hash(v: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    // write input message
    hasher.update(v);
    // read hash digest and consume hasher
    hex::encode(hasher.finalize().iter().map(|x| *x).collect::<Vec<u8>>())
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

enum LAC {
    Clean,
    Transcoded,
    Upscaled,
    Upsampled,
}

impl Display for LAC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LAC::Clean => write!(f, "Clean"),
            LAC::Transcoded => write!(f, "Transcoded"),
            LAC::Upscaled => write!(f, "Upscaled"),
            LAC::Upsampled => write!(f, "Upsampled"),
        }
    }
}

struct File {
    path: PathBuf,
    hash: String,
    result: Result<LAC, String>,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File:   {:?}", self.path)?;
        writeln!(f, "Hash:   {}", self.hash)?;
        writeln!(f, "Result: {}", self.result.as_ref().unwrap())
    }
}

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

fn process_wav(path: PathBuf, bin: &PathBuf) -> Result<File, Box<dyn Error>> {
    let f = fs::read(&path)?;
    let hash = hash(&f);
    let result = process(&path, bin)?;
    Ok(File { path, hash, result })
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

fn process_flac(path: PathBuf, bin: &PathBuf) -> Result<File, Box<dyn Error>> {
    let f = fs::read(&path)?;
    let hash = hash(&f);
    let waw = decode_file(&path);
    let result = process(&waw, bin)?;
    fs::remove_file(waw)?;
    Ok(File { path, hash, result })
}
