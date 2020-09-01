use common::config;
use common::constants::{CONFIG_PATH, DAEMON_LOCK_PATH};
use common::structs::Config;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::thread;
use std::{
    fs::{remove_file, File},
    io::ErrorKind,
    process,
    sync::mpsc::channel,
    sync::Arc,
    sync::Mutex,
    time::Duration,
};

use simple_signal::{self, Signal};

struct DaemonLock;

impl DaemonLock {
    fn grab(&self) {
        info!("Attempting to aqcuire exclusive lock on daemon file.");

        match File::open(DAEMON_LOCK_PATH) {
            Ok(_) => {
                error!(
                    "Failed to grab exclusive lock via lock file `{}`. \
                    Is there another instance of animated running? You can \
                    run `animated kill` to kill any existing instances and \
                    immediately release the lock.",
                    DAEMON_LOCK_PATH
                );
                process::exit(1);
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    File::create(DAEMON_LOCK_PATH).expect("Failed to create daemon lock file.");
                }
                _ => {
                    error!("Failed to open daemon lock file due to: {:?}", e);
                    process::exit(1);
                }
            },
        }
    }

    fn release(&self) {
        info!("Releasing lock file.");

        File::open(DAEMON_LOCK_PATH)
            .expect("Failed to find daemon lock file. Did something else delete it?");
        remove_file(DAEMON_LOCK_PATH).expect(
            format!(
                "Failed to release exlusive lock on lock file. You will\
                need to manually delete the file `{}`.",
                DAEMON_LOCK_PATH
            )
            .as_str(),
        );

        info!("Lock file has been released.");
    }
}

impl Drop for DaemonLock {
    fn drop(&mut self) {
        self.release();
    }
}

fn watch_config_changes(config_mtx: Arc<Mutex<Config>>) -> Box<thread::JoinHandle<()>> {
    let handle = thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(3))
            .expect("Failed to create filesystem watcher to watch config file.");
        watcher
            .watch(CONFIG_PATH, RecursiveMode::NonRecursive)
            .expect(format!("Failed to watch config file at path `{}`.", CONFIG_PATH).as_str());

        info!("Watching for changes in config file.");

        loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Write(_) => {
                        let config_result = config::read();
                        let config =
                            config_result.expect("Failed to read recently written config file.");
                        debug!("Config file was updated. New contents: {:?}", config);
                        let mut current_config = config_mtx.lock().unwrap();
                        *current_config = config;
                    }
                    DebouncedEvent::Remove(_) => {
                        error!("Config file was deleted.");
                        process::exit(1);
                    }
                    DebouncedEvent::Rename(_, _) => {
                        error!("Config file was moved.");
                        process::exit(1);
                    }
                    DebouncedEvent::Error(e, _) => {
                        error!("A watch error occured: {:?}", e);
                        process::exit(1);
                    }
                    _ => (),
                },
                Err(e) => error!("A watch error occured: {:?}", e),
            }
        }
    });

    return Box::new(handle);
}

fn watch_anime(_config_mtx: Arc<Mutex<Config>>) -> Box<thread::JoinHandle<()>> {
    info!("Watching for new anime to download.");

    let handle = thread::spawn(move || loop {});
    return Box::new(handle);
}

pub fn run(config: Config) {
    info!("Starting animated daemon.");
    debug!("Using current config: {:?}", config);

    let _lock = DaemonLock {};
    _lock.grab();

    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_| {
        info!("Daemon received SIGINT/SIGTERM.");
        _lock.release();
        info!("Exiting.");
        process::exit(0);
    });

    let config_mtx = Arc::new(Mutex::new(config));
    let config_watch_handle = *watch_config_changes(Arc::clone(&config_mtx));
    let anime_watch_handle = *watch_anime(Arc::clone(&config_mtx));

    config_watch_handle.join().unwrap();
    anime_watch_handle.join().unwrap();
}
