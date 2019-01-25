use crate::{config, logger};
use signal_hook::{iterator::Signals, SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use std::io::Error;

pub fn handle() -> Result<(), Error> {
    let signals = Signals::new(&[
        SIGHUP,  // 1
        SIGINT,  // 2
        SIGQUIT, // 3
        SIGTERM, // 15
    ])?;

    for signal in signals.forever() {
        match signal {
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
            SIGHUP => {
                info!(
                    logger::ROOT, "PROGRAM";
                    "status" => "reloading",
                    "signal" => signal
                );
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
