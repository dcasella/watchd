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

fn main() -> Result<(), Error> {
    info!(
        logger::ROOT, "PROGRAM";
        "status" => "started"
    );

    // main loop signal handling
    signal::Handler::new().handle()
}
