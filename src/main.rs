#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate lazy_static;

mod cli;
mod config;
mod handler;
mod logger;
mod signal;
mod watcher;

use std::{io::Error, sync::mpsc};

fn main() -> Result<(), Error> {
    info!(
        logger::ROOT, "PROGRAM";
        "status" => "started"
    );

    // for each entry, instantiate a shared channel such that the watcher thread
    // writes to `shared_tx` while the handler thread read from `shared_rx`
    for (i, _) in config::OPTS.entries.iter().enumerate() {
        let (shared_tx, shared_rx) = mpsc::channel();

        watcher::spawn(i, shared_tx.clone());
        handler::spawn(i, shared_rx);
    }

    if config::OPTS.verbose {
        info!(
            logger::ROOT, "BOOT";
            "watchers" => "started",
            "handlers" => "started"
        );
    }

    // signal handling
    signal::handle()
}
