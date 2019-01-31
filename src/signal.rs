use crate::{config, logger, watcher::Watcher};
use signal_hook::{iterator::Signals, SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use std::io::Error;

pub struct Handler {
    watchers: Vec<Watcher>
}

impl Handler {
    pub fn new(watchers: Vec<Watcher>) -> Self {
        Self { watchers }
    }

    pub fn handle(&self) -> Result<(), Error> {
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
                    if config::OPTS.verbose {
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
                    warn!(
                        logger::ROOT, "PROGRAM (UNIMPLEMENTED)";
                        "status" => "reloading",
                        "signal" => signal
                    );

                    // handle watchers
                    // self.watchers.iter().for_each(|w| w.terminate());
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
}
