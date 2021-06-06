use std::{collections::HashMap, error::Error, fmt::Display, path::PathBuf};

#[derive(Clone, Debug)]
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

impl core::str::FromStr for LAC {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        if s.contains("clean") {
            Ok(LAC::Clean)
        } else if s.contains("transcoded") {
            Ok(LAC::Transcoded)
        } else if s.contains("upscaled") {
            Ok(LAC::Upscaled)
        } else if s.contains("upsampled") {
            Ok(LAC::Upsampled)
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct File {
    pub path: PathBuf,
    pub hash: String,
    pub result: Result<LAC, String>,
}

impl File {
    fn from(s1: &str, s2: &str, s3: &str) -> Self {
        File {
            path: s1
                .split(": ")
                .nth(1)
                .expect("Wrong format of LAC.log")
                .trim()
                .into(),
            hash: s2
                .split(": ")
                .nth(1)
                .expect("Wrong format of LAC.log")
                .trim()
                .into(),
            result: Ok(s3
                .split(": ")
                .nth(1)
                .expect("Wrong format of LAC.log")
                .trim()
                .parse()
                .unwrap()),
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File:   {}", self.path.to_str().unwrap())?;
        writeln!(f, "Hash:   {}", self.hash)?;
        writeln!(f, "Result: {}", self.result.as_ref().unwrap())
    }
}

/// Get header
pub fn get_header(bin: &PathBuf) -> Result<String, Box<dyn Error>> {
    let out = std::process::Command::new(bin).output()?;
    let output = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    Ok(output.lines().next().unwrap().to_owned())
}

#[derive(Clone, Debug)]
pub struct LOG {
    pub header: String,
    /// folder: vec of FILEs
    pub data: HashMap<PathBuf, Vec<File>>,
}

impl Display for LOG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.header)?;
        writeln!(f, "")?;
        for (p, v) in &self.data {
            for file in v {
                writeln!(f, "{}", file)?;
            }
        }
        Ok(())
    }
}

fn insert_or_update(m: &mut HashMap<PathBuf, Vec<File>>, f: File) {
    let k = f.path.parent().unwrap();
    if m.contains_key(k) {
        let v = m.get_mut(k).unwrap();
        v.push(f);
    } else {
        m.insert(k.to_owned(), vec![f]);
    }
}

impl LOG {
    pub fn new(bin: &PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            header: get_header(bin).unwrap(),
            data: HashMap::new(),
        })
    }
    pub fn insert_or_update(&mut self, f: File) {
        insert_or_update(&mut self.data, f)
    }
    pub fn from(bin: &PathBuf, v: &[u8]) -> Result<Self, Box<dyn Error>> {
        let raw_data = &String::from_utf8_lossy(v)
            .lines()
            .filter(|&x| !x.is_empty())
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()[1..];
        assert_eq!(raw_data.len() % 3, 0);
        let mut data = HashMap::new();
        for i in 0..raw_data.len() / 3 {
            insert_or_update(
                &mut data,
                File::from(&raw_data[i * 3], &raw_data[i * 3 + 1], &raw_data[i * 3 + 2]),
            );
        }
        Ok(Self {
            header: get_header(bin).unwrap(),
            data,
        })
    }
}
