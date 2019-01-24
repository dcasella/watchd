#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate lazy_static;

mod cli;
mod config;
mod logger;
mod monitor;

use std::{thread::sleep, time::Duration};

fn main() {
    if config::OPTS.verbose {
        info!(logger::ROOT, "INIT"; "watchers" => "starting");
    }

    for (i, _) in config::OPTS.entries.iter().enumerate() {
        monitor::spawn(i);
    }

    if config::OPTS.verbose {
        info!(logger::ROOT, "INIT"; "watchers" => "ready");
    }

    // main loop
    loop {
        sleep(Duration::from_secs(60));
    }
}
