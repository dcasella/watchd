use crate::logger;
use crossbeam_channel::{bounded, select, tick, Receiver};
use signal_hook::{iterator::Signals, SIGINT};
use std::{error::Error, thread, time::Duration};

fn signal_channel() -> Result<Receiver<i32>, Box<Error>> {
    let signals = Signals::new(&[SIGINT])?;
    let (tx, rx) = bounded(100);

    thread::spawn(move || {
        for signal in signals.forever() {
            info!(
                logger::ROOT, "PROGRAM";
                "status" => "exited",
                "signal" => signal
            );

            let _ = tx.send(signal);
        }
    });

    Ok(rx)
}

pub fn select() -> Result<(), Box<Error>> {
    let signal_events = signal_channel()?;
    let ticks = tick(Duration::from_secs(1));

    // main loop
    loop {
        select! {
            recv(ticks) -> _ => {}
            recv(signal_events) -> _ => {
                break;
            }
        }
    }

    Ok(())
}
