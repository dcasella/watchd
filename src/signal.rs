use crate::{config, logger, watcher::Watcher};
use signal_hook::{iterator::Signals, SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use std::io::Error;

pub struct Handler {
    watchers: Vec<Watcher>
}

impl Handler {
    pub fn new() -> Self {
        // for each entry, instantiate a Watcher
        let watchers: Vec<Watcher> = watchers();

        if config::OPTS.read().unwrap().verbose {
            info!(
                logger::ROOT, "BOOT";
                "watchers" => "started",
                "handlers" => "started"
            );
        }

        Self { watchers }
    }

    pub fn handle(&mut self) -> Result<(), Error> {
        let signals = Signals::new(&[
            SIGHUP,  // 1
            SIGINT,  // 2
            SIGQUIT, // 3
            SIGTERM, // 15
        ])?;

        // main loop
        // match signals as they come
        for signal in signals.forever() {
            match signal {
                // exit program
                SIGTERM | SIGINT | SIGQUIT => {
                    if config::OPTS.read().unwrap().verbose {
                        info!(
                            logger::ROOT, "PROGRAM";
                            "status" => "exiting",
                            "signal" => signal
                        );
                    }

                    break;
                }
                // reload configuration
                SIGHUP => {
                    info!(
                        logger::ROOT, "RELOAD";
                        "status" => "starting",
                        "signal" => signal
                    );

                    // handle reload
                    self.reload();
                }
                _ => unreachable!()
            }
        }

        info!(
            logger::ROOT, "PROGRAM";
            "status" => "exited"
        );

        Ok(())
    }

    fn reload(&mut self) {
        // terminate watchers; this is required to acquire a WriteLock
        for watcher in &self.watchers {
            watcher.terminate();
        }

        // acquire WriteLock
        if config::OPTS.write().unwrap().reload().is_ok() {
            info!(
                logger::ROOT, "RELOAD";
                "status" => "complete"
            );

            // deploy new watchers
            self.watchers = watchers();
        }
        else {
            error!(
                logger::ROOT, "RELOAD";
                "status" => "failed"
            );

            // redeploy the previous configuration's watchers
            for watcher in self.watchers.iter_mut() {
                watcher.restart();
            }
        }
    }
}

fn watchers() -> Vec<Watcher> {
    config::OPTS
        .read()
        .unwrap()
        .entries
        .keys()
        .map(|entry_path| Watcher::new(entry_path.clone()))
        .collect()
}
