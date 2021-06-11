use async_std::path::PathBuf;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum Lac {
    Clean,
    Transcoded,
    Upscaled,
    Upsampled,
}

impl Display for Lac {
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

#[derive(Clone, Debug)]
pub struct File {
    pub path: PathBuf,
    pub hash: String,
    pub result: Result<Lac, String>,
}

impl File {
    fn from(s1: &str, s2: &str, s3: &str) -> Self {
        File {
            path: s1
                .split(": ")
                .nth(1)
                .expect("Wrong format of Lac.log")
                .trim()
                .into(),
            hash: s2
                .split(": ")
                .nth(1)
                .expect("Wrong format of Lac.log")
                .trim()
                .into(),
            result: Ok(s3
                .split(": ")
                .nth(1)
                .expect("Wrong format of Lac.log")
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

#[derive(Clone, Debug)]
pub struct Log {
    /// folder: vec of FILEs
    pub data: HashMap<PathBuf, Vec<File>>,
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in self.data.values() {
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

impl Log {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn append(&mut self, log: Log) {
        self.data.extend(log.data)
    }
    pub fn insert_or_update(&mut self, f: File) {
        insert_or_update(&mut self.data, f)
    }
    pub fn relevant(&self, dir: &str) -> Log {
        let mut log = Log::new();
        for (k, v) in &self.data {
            if k.to_str().unwrap().contains(dir) {
                log.data.insert(k.clone(), v.clone());
            }
        }
        log
    }
    pub fn from(v: &[u8]) -> Result<Self, Box<dyn Error>> {
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
        Ok(Self { data })
    }
}
