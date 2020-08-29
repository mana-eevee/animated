use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};
use whoami;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Quality {
    Res360,
    Res480,
    Res720,
    Res1080,
    Res4k,
}

impl FromStr for Quality {
    type Err = ();

    fn from_str(s: &str) -> Result<Quality, ()> {
        match s {
            "360p" => Ok(Quality::Res360),
            "480p" => Ok(Quality::Res480),
            "720p" => Ok(Quality::Res720),
            "1080p" => Ok(Quality::Res1080),
            "4k" => Ok(Quality::Res4k),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Quality::Res360 => write!(f, "{}", "360p".to_string()),
            Quality::Res480 => write!(f, "{}", "480p".to_string()),
            Quality::Res720 => write!(f, "{}", "720p".to_string()),
            Quality::Res1080 => write!(f, "{}", "1080p".to_string()),
            Quality::Res4k => write!(f, "{}", "4k".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Anime {
    pub title: String,
    pub quality: Quality,
    pub subgroup: String,
    pub last_seen_episode: i32,
    pub tombstone: bool,
}

impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.to_lowercase().hash(state);
        self.quality.to_string().hash(state);
        self.subgroup.to_lowercase().hash(state);
    }
}

impl Anime {
    pub fn to_hash(&self) -> String {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        return s.finish().to_string();
    }
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "[{}] {} @ {}", self.subgroup, self.title, self.quality);
    }
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct Config {
    #[serde(default = "default_download_path")]
    pub download_path: String,
}

pub fn default_download_path() -> String {
    let username = whoami::username();
    return format!("/home/{}/Downloads", username);
}
