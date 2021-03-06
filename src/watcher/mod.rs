pub mod handler;

use crate::{config, logger};
use notify::{DebouncedEvent, Error, RecursiveMode, Watcher as WatcherTrait};
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration
};

pub enum Message {
    Path(String),
    Terminate
}

pub struct Watcher {
    pub entry_path: PathBuf,
    data: WatcherData
}

struct WatcherData {
    _watcher_thread: thread::JoinHandle<()>,
    _handler_thread: thread::JoinHandle<()>,
    shared_tx: Sender<Message>,
    watcher_tx: Sender<DebouncedEvent>
}

impl Watcher {
    pub fn new(entry_path: PathBuf) -> Self {
        Self {
            entry_path: entry_path.to_owned(),
            data: self::WatcherData::new(&entry_path, true)
        }
    }

    pub fn restart(&mut self) {
        self.data = self::WatcherData::new(&self.entry_path, false);
    }

    pub fn terminate(&self) {
        let _ = self.data.shared_tx.send(Message::Terminate);
        let _ = self
            .data
            .watcher_tx
            .send(DebouncedEvent::Error(Error::WatchNotFound, None));
    }
}

impl WatcherData {
    fn new(entry_path: &PathBuf, try_init: bool) -> Self {
        let (shared_tx, shared_rx) = channel();
        let (watcher_tx, watcher_rx) = channel();

        Self {
            _watcher_thread: self::spawn(
                entry_path.to_owned(),
                shared_tx.clone(),
                watcher_tx.clone(),
                watcher_rx
            ),
            _handler_thread: handler::spawn(entry_path.to_owned(), try_init, shared_rx),
            shared_tx,
            watcher_tx
        }
    }
}

fn spawn(
    entry_path: PathBuf,
    shared_tx: Sender<Message>,
    tx: Sender<DebouncedEvent>,
    rx: Receiver<DebouncedEvent>
) -> thread::JoinHandle<()> {
    // generate thread name for logging purposes
    let thread_name = format!("watcher-{}", entry_path.display());

    thread::Builder::new()
        .name(thread_name.to_owned())
        .spawn(move || {
            // debounced (10ms) events watcher
            let mut watcher = notify::watcher(tx, Duration::from_millis(10)).unwrap();

            // add entry `path` to the watcher
            watcher
                .watch(
                    &entry_path,
                    if config::OPTS.read().unwrap().entries[&entry_path].recursive {
                        RecursiveMode::Recursive
                    }
                    else {
                        RecursiveMode::NonRecursive
                    }
                )
                .unwrap();

            // instantiate thread-local logger
            let thread_log = logger::ROOT.new(o!(
                "thread" => thread_name,
                "id" => format!("{}", entry_path.display())
            ));

            if config::OPTS.read().unwrap().verbose {
                info!(thread_log, "SPAWN");
            }

            // watch for events on `rx`
            'event_loop: loop {
                match rx.recv() {
                    // terminate
                    Ok(DebouncedEvent::Error(Error::WatchNotFound, None)) => {
                        info!(thread_log, "TERMINATE");

                        // exit from thread
                        break;
                    }
                    // single file operation
                    Ok(DebouncedEvent::Create(path))
                    | Ok(DebouncedEvent::Write(path))
                    | Ok(DebouncedEvent::Chmod(path))
                    | Ok(DebouncedEvent::Remove(path)) => {
                        let path = path.to_str().expect("Could not parse path");

                        // test path against excludes
                        for exclude in &config::OPTS.read().unwrap().entries[&entry_path].excludes {
                            if exclude.is_match(path) {
                                if config::OPTS.read().unwrap().verbose {
                                    info!(
                                        thread_log, "EVENT";
                                        "exclude" => true,
                                        "pattern" => exclude.as_str(),
                                        "path" => path
                                    );
                                }

                                // ignore; continue to next received event
                                continue 'event_loop;
                            }
                        }

                        if config::OPTS.read().unwrap().verbose {
                            info!(
                                thread_log, "EVENT";
                                "path" => path
                            );
                        }

                        // forward event to the shared channel
                        let _ = shared_tx.send(Message::Path(path.to_owned()));
                    }
                    // multiple file operation
                    Ok(DebouncedEvent::Rename(path_from, path_to)) => {
                        let path_from = path_from.to_str().expect("Could not parse path_from");
                        let path_to = path_to.to_str().expect("Could not parse path_to");

                        // test both paths against excludes
                        for exclude in &config::OPTS.read().unwrap().entries[&entry_path].excludes {
                            if exclude.is_match(path_from) || exclude.is_match(path_to) {
                                if config::OPTS.read().unwrap().verbose {
                                    info!(
                                        thread_log, "EVENT";
                                        "exclude" => true,
                                        "pattern" => exclude.as_str(),
                                        "path-from" => path_from,
                                        "path-to" => path_to
                                    );
                                }

                                // ignore; continue to next received event
                                continue 'event_loop;
                            }
                        }

                        info!(
                            thread_log, "EVENT";
                            "path-from" => path_from,
                            "path-to" => path_to
                        );

                        // forward event to the shared channel
                        let _ = shared_tx.send(Message::Path(path_from.to_owned()));
                        let _ = shared_tx.send(Message::Path(path_to.to_owned()));
                    }
                    // death
                    Err(err) => {
                        error!(
                            thread_log, "EVENT";
                            "error" => true,
                            "message" => err.to_string()
                        );

                        // error; continue to next received event
                        continue;
                    }
                    // ignored operations
                    _ => {}
                }
            }
        })
        .expect("Could not spawn watcher thread")
}
