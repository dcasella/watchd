#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;

mod cli;
mod config;
mod monitor;

use std::thread::sleep;
use std::time::Duration;

use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;

use config::Config;

fn main() {
    let config = Config::from(&cli::Options::load());

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(if config.verbose {
        Severity::Trace
    } else {
        Severity::Warning
    });
    builder.destination(Destination::Stdout);

    let _guard = slog_scope::set_global_logger(builder.build().unwrap());

    for entry in config.entries {
        monitor::spawn(entry, config.init, config.dry_run, config.verbose);
    }

    if config.verbose {
        info!("watchd started");
    }

    // main loop
    loop {
        sleep(Duration::from_secs(60));
    }
}
