mod daemon;

extern crate bincode;
extern crate common;
#[macro_use]
extern crate log;
extern crate stderrlog;

use clap::{load_yaml, App};
use std::process;

use common::config;

fn main() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(4)
        .color(stderrlog::ColorChoice::Always)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    let yaml = load_yaml!("server.yaml");
    App::from(yaml).get_matches();

    let config_result = config::read();

    match config_result {
        Ok(config) => {
            daemon::run(config);
        }
        Err(e) => {
            error!(
                "An error occurred while reading animated config. \
                You may need to create one with `animated -e`.\nError: {}",
                e.to_string()
            );
            process::exit(1);
        }
    }
}
