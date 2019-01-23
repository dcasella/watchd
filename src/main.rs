#[macro_use]
extern crate serde_derive;

mod cli;
mod config;
mod monitor;

use std::thread::sleep;
use std::time::Duration;

use config::Config;

fn main() {
    let config = Config::from(&cli::Options::load());

    for entry in config.entries {
        monitor::spawn(entry, config.init, config.dry_run, config.verbose);
    }

    // main loop
    loop {
        sleep(Duration::from_secs(60));
    }
}
