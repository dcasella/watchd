#[macro_use]
extern crate serde_derive;

mod cli;
mod config;
mod monitor;

use config::Config;
use monitor::{Monitor, OnEvent};

struct Watcher {
    monitor: Monitor,
    handler: Handler
}

struct Handler {}

impl Handler {
    fn new() -> Self {
        Self {}
    }
}

impl OnEvent for Handler {
    fn on_event(&mut self) {}
}

fn main() {
    let config = Config::from(&cli::Options::load());

    // monitor entries
    let monitors = Vec::new();

    for entry in config.entries {
        let mut handler = Handler::new();

        monitors.push(Watcher {
            monitor: Monitor::new(&entry, &mut handler),
            handler
        });
    }

    // main loop
    loop {}
}
