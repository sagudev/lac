pub mod bin;

#[derive(Clone, PartialEq)]
pub enum Lac {
    Clean,
    Transcoded,
    Upscaled,
    Upsampled,
}

/// Print to Display
impl std::fmt::Display for Lac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lac::Clean => write!(f, "Clean"),
            Lac::Transcoded => write!(f, "Transcoded"),
            Lac::Upscaled => write!(f, "Upscaled"),
            Lac::Upsampled => write!(f, "Upsampled"),
        }
    }
}

/// Print to log
impl std::fmt::Debug for Lac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lac::Clean => write!(f, "Clean"),
            Lac::Transcoded => write!(f, "Transcoded"),
            Lac::Upscaled => write!(f, "Upscaled"),
            Lac::Upsampled => write!(f, "Upsampled"),
        }
    }
}

impl core::str::FromStr for Lac {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        if s.contains("clean") {
            Ok(Lac::Clean)
        } else if s.contains("transcoded") {
            Ok(Lac::Transcoded)
        } else if s.contains("upscaled") {
            Ok(Lac::Upscaled)
        } else if s.contains("upsampled") {
            Ok(Lac::Upsampled)
        } else {
            Err(())
        }
    }
}
