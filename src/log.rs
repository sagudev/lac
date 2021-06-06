use std::{fmt::Display, path::PathBuf};

pub enum LAC {
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

pub struct File {
    pub path: PathBuf,
    pub hash: String,
    pub result: Result<LAC, String>,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File:   {:?}", self.path)?;
        writeln!(f, "Hash:   {}", self.hash)?;
        writeln!(f, "Result: {}", self.result.as_ref().unwrap())
    }
}
