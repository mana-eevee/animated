mod config;
mod constants;
mod daemon;
mod rocksdb;
mod structs;

extern crate bincode;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate log;

use clap::{load_yaml, App};
use prettytable::Table;

use std::{
    process,
    str::FromStr,
};

use structs::{Anime};

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let config_result = config::read();

    if matches.is_present("edit") {
        config::edit(&config_result);
        return;
    }

    match config_result {
        Ok(config) => {
            match matches.subcommand() {
                ("watch", Some(watch_matches)) => {
                    let name = watch_matches
                        .value_of("name")
                        .expect("Expected `--name` to have been specified.");
                    // We can directly unwrap this because clap automatically
                    // performs allowed value validation for us.
                    let quality = structs::Quality::from_str(
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
                ("", None) => {
                    if matches.is_present("daemon") {
                        daemon::run(config);
                    }
                }
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
