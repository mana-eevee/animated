extern crate bincode;
#[macro_use]
extern crate prettytable;

use clap::{load_yaml, App};
use prettytable::Table;
use rocksdb::{IteratorMode, DB};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{
    collections::HashMap,
    env, fmt, fs,
    io::{Error, ErrorKind},
    process,
    process::Command,
    str::FromStr,
};
use whoami;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum Quality {
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
struct Anime {
    title: String,
    quality: Quality,
    subgroup: String,
    last_seen_episode: i32,
    tombstone: bool,
}

impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.to_lowercase().hash(state);
        self.quality.to_string().hash(state);
        self.subgroup.to_lowercase().hash(state);
    }
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "[{}] {} @ {}", self.subgroup, self.title, self.quality);
    }
}

fn gen_anime_hash(anime: &Anime) -> String {
    let mut s = DefaultHasher::new();
    anime.hash(&mut s);
    return s.finish().to_string();
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default = "default_download_path")]
    download_path: String,
}

fn default_download_path() -> String {
    let username = whoami::username();
    return format!("/home/{}/Downloads", username);
}

const ANIMATED_DIR: &str = "/etc/animated/";
const CONFIG_PATH: &str = "/etc/animated/config.json";
const ROCKSDB_PATH: &str = "/etc/animated/animated.rocksdb";

fn read_config() -> Result<Config, Box<Error>> {
    let file_contents: String = fs::read_to_string(CONFIG_PATH)?;
    let config: Config = serde_json::from_str(&file_contents).unwrap();
    return Ok(config);
}

fn write_config(config_contents: &str) -> Result<(), Box<Error>> {
    fs::create_dir_all(ANIMATED_DIR)?;
    fs::write(CONFIG_PATH, config_contents)?;
    return Ok(());
}

fn create_default_config() {
    let default_config = Config {
        download_path: default_download_path(),
    };
    let serialized = serde_json::to_string(&default_config).unwrap();
    let write_result = write_config(&serialized);

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
    let write_result = write_config(&edited_config);

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

fn edit_config(config_result: &Result<Config, Box<Error>>) {
    match config_result {
        Err(e) => match (*e).kind() {
            ErrorKind::NotFound => create_default_config(),
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

fn upsert_anime_in_rocks(anime: &Anime) {
    let db = DB::open_default(ROCKSDB_PATH).expect("Failed to open local RocksDB to upsert anime.");

    db.put(
        gen_anime_hash(anime).as_bytes(),
        bincode::serialize(anime).unwrap(),
    )
    .expect("Failed to upsert anime in RocksDB after opening.");
}

fn add_anime(anime: &Anime) -> String {
    let anime_hash = gen_anime_hash(anime);
    upsert_anime_in_rocks(anime);
    return anime_hash;
}

fn list_anime_in_rocks() -> HashMap<String, Anime> {
    let db = DB::open_default(ROCKSDB_PATH).expect("Failed to open local RocksDB to list anime.");
    let mut all_anime = HashMap::new();
    let iter = db.iterator(IteratorMode::Start);
    for (key, value) in iter {
        // Unbox the Box and then get a pointer.
        let key_byte_array: &[u8] = &*key;
        let anime_byte_array: &[u8] = &*value;
        let anime = bincode::deserialize::<Anime>(anime_byte_array).unwrap();

        if anime.tombstone {
            continue;
        }

        all_anime.insert(String::from_utf8(key_byte_array.to_vec()).unwrap(), anime);
    }
    return all_anime;
}

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let config_result = read_config();

    if matches.is_present("edit") {
        edit_config(&config_result);
        return;
    }

    match config_result {
        Ok(_) => {
            match matches.subcommand() {
                ("watch", Some(watch_matches)) => {
                    let name = watch_matches
                        .value_of("name")
                        .expect("Expected `--name` to have been specified.");
                    // We can directly unwrap this because clap automatically
                    // performs allowed value validation for us.
                    let quality = Quality::from_str(
                        watch_matches
                            .value_of("quality")
                            .expect("Expected `--quality` to have been specified."),
                    )
                    .unwrap();
                    let subgroup = watch_matches
                        .value_of("subgroup")
                        .expect("Expected `--subgroup` to have been specified.");

                    let anime = Anime {
                        title: String::from(name),
                        quality: quality,
                        subgroup: String::from(subgroup),
                        last_seen_episode: -1,
                        tombstone: false,
                    };
                    let watch_id = add_anime(&anime);
                    println!("Now watching {} / Watch ID: {}", anime, watch_id);
                }
                ("unwatch", Some(_unwatch_matches)) => {}
                ("list", Some(_list_matches)) => {
                    let all_anime = list_anime_in_rocks();
                    let mut table = Table::new();

                    table.add_row(row!["Title", "Sub Group", "Quality", "Watch ID"]);

                    for (watch_id, anime) in all_anime {
                        table.add_row(row![anime.title, anime.subgroup, anime.quality, watch_id]);
                    }

                    table.printstd();
                }
                ("", None) => {}
                // If all subcommands are defined above, anything else is unreachabe!
                _ => unreachable!(),
            }
        }
        Err(e) => {
            eprintln!(
                "An error occurred while reading animated config. \
                You may need to create one with `animated -e`.\nError: {}",
                e.to_string()
            );
            process::exit(1);
        }
    }
}
