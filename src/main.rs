#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate lazy_static;

mod cli;
mod config;
mod logger;
mod signal;
mod watcher;

use std::io::Error;
use watcher::Watcher;

fn main() -> Result<(), Error> {
    info!(
        logger::ROOT, "PROGRAM";
        "status" => "started"
    );

    // for each entry, instantiate a Watcher
    let watchers: Vec<Watcher> = config::OPTS
        .entries
        .keys()
        .map(|entry_path| Watcher::new(entry_path.clone()))
        .collect();

    if config::OPTS.verbose {
        info!(
            logger::ROOT, "BOOT";
            "watchers" => "started",
            "handlers" => "started"
        );
    }

    // main loop signal handling
    signal::Handler::new(watchers).handle()
}
