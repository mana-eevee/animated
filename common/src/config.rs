use crate::constants::{ANIMATED_DIR, CONFIG_PATH};
use crate::structs::{Config, default_download_path};

use std::{
    env, fs,
    io::{Error, ErrorKind},
    process,
    process::Command,
};

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

fn create_default() {
    let default_config = Config {
        download_path: default_download_path(),
    };
    let serialized = serde_json::to_string(&default_config).unwrap();
    let write_result = write(&serialized);

    match write_result {
        Err(e) => match (*e).kind() {
            ErrorKind::PermissionDenied => {
                eprintln!(
                    "Failed to generate default animated config due to insufficient permissions. \
                    Make sure that you have write permission to `/etc` then try again."
                );
                process::exit(1);
            }
            _ => {
                eprintln!(
                    "An error occurred while generating default animated config. \
                        \nError: {}",
                    e.to_string()
                );
                process::exit(1);
            }
        },
        Ok(_) => (),
    }
}

fn open_config_editor() {
    let editor = env::var("EDITOR").unwrap();
    let mut temp_file_path = env::temp_dir();
    temp_file_path.push("animated.config.tmp");

    fs::copy(CONFIG_PATH, &temp_file_path)
        .expect("Failed to write temporary animated config for editing.");

    Command::new(editor)
        .arg(&temp_file_path)
        .status()
        .expect("Failed to open $EDITOR for editing animated config.");

    let edited_config = fs::read_to_string(&temp_file_path)
        .expect("Failed to read newly edited animated config. Did something happen to it?");
    let write_result = write(&edited_config);

    match write_result {
        Err(e) => match (*e).kind() {
            ErrorKind::PermissionDenied => {
                eprintln!(
                    "Failed to finalize newly edited animated config due to insufficient permissions. \
                    Make sure that you have write permission to `/etc` then try again."
                );
                process::exit(1);
            }
            _ => {
                eprintln!(
                    "An error occurred while generating default animated config. \
                        \nError: {}",
                    e.to_string()
                );
                process::exit(1);
            }
        },
        Ok(_) => (),
    }
}

pub fn edit(config_result: &Result<Config, Box<Error>>) {
    match config_result {
        Err(e) => match (*e).kind() {
            ErrorKind::NotFound => create_default(),
            _ => {
                eprintln!(
                    "An error occurred while reading animated config. \
                        \nError: {}",
                    e.to_string()
                );
                process::exit(1);
            }
        },
        Ok(_) => (),
    }

    open_config_editor();
}
