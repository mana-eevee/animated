use crate::constants::{ANIMATED_DIR, CONFIG_PATH};
use crate::structs::Config;

use std::{fs, io::Error};

pub fn read() -> Result<Config, Box<Error>> {
    let file_contents: String = fs::read_to_string(CONFIG_PATH)?;
    let config: Config = serde_json::from_str(&file_contents).unwrap();
    return Ok(config);
}

pub fn write(config_contents: &str) -> Result<(), Box<Error>> {
    fs::create_dir_all(ANIMATED_DIR)?;
    fs::write(CONFIG_PATH, config_contents)?;
    return Ok(());
}
