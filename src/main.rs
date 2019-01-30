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

    // for each entry, instantiate a shared channel such that the watcher thread
    // writes to `shared_tx` while the handler thread read from `shared_rx`
    for entry_path in config::OPTS.entries.keys() {
        let _ = Watcher::new(entry_path.clone());
    }

    if config::OPTS.verbose {
        info!(
            logger::ROOT, "BOOT";
            "watchers" => "started",
            "handlers" => "started"
        );
    }

    // main loop signal handling
    signal::handle()
}
