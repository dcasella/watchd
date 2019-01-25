use crate::logger;
use signal_hook::{iterator::Signals, SIGINT};
use std::{error::Error, sync::mpsc, thread};

fn signal_channel() -> Result<mpsc::Receiver<i32>, Box<Error>> {
    let signals = Signals::new(&[SIGINT])?;
    let (tx, rx) = mpsc::channel();

    // generate thread name for logging purposes
    let thread_name = "signal-handler";

    thread::Builder::new()
        .name(thread_name.to_owned())
        .spawn(move || {
            // instantiate thread-local logger
            let thread_log = logger::ROOT.new(o!(
                "thread" => thread_name
            ));

            for signal in signals.forever() {
                info!(
                    thread_log, "PROGRAM";
                    "status" => "exited",
                    "signal" => signal
                );

                let _ = tx.send(signal);
            }
        })
        .expect("Could not spawn signal-handler thread");

    Ok(rx)
}

pub fn select() -> Result<(), Box<Error>> {
    let signal_events = signal_channel()?;

    // signal handling loop
    loop {
        if signal_events.recv().is_ok() {
            break;
        }
    }

    Ok(())
}
