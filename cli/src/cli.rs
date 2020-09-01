extern crate bincode;
extern crate common;
#[macro_use]
extern crate prettytable;

use clap::{load_yaml, App};
use prettytable::Table;
use std::{
    env, fs,
    io::{Error, ErrorKind},
    process,
    str::FromStr,
};

use common::{
    config,
    constants::CONFIG_PATH,
    rocksdb,
    structs::{default_download_path, Anime, Config, Quality},
};

fn create_default() {
    let default_config = Config {
        download_path: default_download_path(),
    };
    let serialized = serde_json::to_string(&default_config).unwrap();
    let write_result = config::write(&serialized);

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

    process::Command::new(editor)
        .arg(&temp_file_path)
        .status()
        .expect("Failed to open $EDITOR for editing animated config.");

    let edited_config = fs::read_to_string(&temp_file_path)
        .expect("Failed to read newly edited animated config. Did something happen to it?");
    let write_result = config::write(&edited_config);

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
        Ok(_) => {
            println!("{}", format!("Config was set to: {}", edited_config).trim());
        }
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

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let app = App::from(yaml);
    let matches = app.get_matches();

    let config_result = config::read();

    if matches.is_present("edit") {
        edit(&config_result);
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
                    let watch_id = rocksdb::upsert_anime(&anime);
                    println!("Now watching {} / Watch ID: {}", anime, watch_id);
                }
                ("unwatch", Some(_unwatch_matches)) => {}
                ("list", Some(_list_matches)) => {
                    let all_anime = rocksdb::list_anime();
                    let mut table = Table::new();

                    table.add_row(row!["Title", "Sub Group", "Quality", "Watch ID"]);

                    for (watch_id, anime) in all_anime {
                        table.add_row(row![anime.title, anime.subgroup, anime.quality, watch_id]);
                    }

                    table.printstd();
                }
                ("", None) => println!(
                    "Let's get started! Run `animated watch` to start watching for new episodes."
                ),
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
