use std::{sync::mpsc, time::Duration};

use notify::{DebouncedEvent, RecursiveMode, Watcher};

use crate::config::Entry;

pub struct Monitor {
    _thread: std::thread::JoinHandle<()>,
    rx: mpsc::Receiver<DebouncedEvent>,
    entry: &'static Entry
}

pub trait OnEvent {
    fn on_event(&mut self);
}

impl Monitor {
    pub fn new<H>(entry: &'static Entry, handler: &'static mut H) -> Self
    where
        H: OnEvent + Send + 'static
    {
        // shared event channel
        let (config_tx, config_rx) = mpsc::channel();

        Self {
            _thread: std::thread::Builder::new()
                .name("notify-monitor".to_string())
                .spawn(move || {
                    // event channel
                    let (tx, rx) = mpsc::channel();

                    // debounced (1s) events watcher
                    let mut watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();

                    // add entry path to the watcher
                    watcher
                        .watch(
                            entry.path.to_owned(),
                            if entry.recursive {
                                RecursiveMode::Recursive
                            }
                            else {
                                RecursiveMode::NonRecursive
                            }
                        )
                        .unwrap();

                    // event loop
                    loop {
                        match rx.recv().expect("Could not receive watcher event") {
                            event => {
                                let _ = config_tx.send(event);
                                handler.on_event();
                            }
                        }
                    }
                })
                .expect("Could not spawn watcher thread"),
            rx: config_rx,
            entry
        }
    }
}
