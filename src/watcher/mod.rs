pub mod handler;

use crate::{config, logger};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

pub fn spawn(entry_path: PathBuf, shared_tx: mpsc::Sender<String>) {
    // generate thread name for logging purposes
    let thread_name = format!("watcher-{}", &entry_path.to_string_lossy());

    thread::Builder::new()
        .name(thread_name.clone())
        .spawn(move || {
            // thread event channels
            let (tx, rx) = mpsc::channel();

            // debounced (10ms) events watcher
            let mut watcher = notify::watcher(tx, Duration::from_millis(10)).unwrap();

            // add entry `path` to the watcher
            watcher
                .watch(
                    &entry_path,
                    if config::OPTS.entries[&entry_path].recursive {
                        RecursiveMode::Recursive
                    }
                    else {
                        RecursiveMode::NonRecursive
                    }
                )
                .unwrap();

            // instantiate thread-local logger
            let thread_log = logger::ROOT.new(o!(
                "thread" => thread_name.clone(),
                "id" => format!("{}", &entry_path.to_string_lossy())
            ));

            if config::OPTS.verbose {
                info!(thread_log, "SPAWN");
            }

            // watch for events on `rx`
            'event_loop: loop {
                match rx.recv() {
                    // single file operation
                    Ok(DebouncedEvent::Create(path))
                    | Ok(DebouncedEvent::Write(path))
                    | Ok(DebouncedEvent::Chmod(path))
                    | Ok(DebouncedEvent::Remove(path)) => {
                        let path = path.to_str().expect("Could not parse path");

                        // test path against excludes
                        for exclude in &config::OPTS.entries[&entry_path].excludes {
                            if exclude.is_match(path) {
                                if config::OPTS.verbose {
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

                        if config::OPTS.verbose {
                            info!(
                                thread_log, "EVENT";
                                "path" => path
                            );
                        }

                        // forward event to the shared channel
                        let _ = shared_tx.send(path.to_owned());
                    }
                    // multiple file operation
                    Ok(DebouncedEvent::Rename(path_from, path_to)) => {
                        let path_from = path_from.to_str().expect("Could not parse path_from");
                        let path_to = path_to.to_str().expect("Could not parse path_to");

                        // test both paths against excludes
                        for exclude in &config::OPTS.entries[&entry_path].excludes {
                            if exclude.is_match(path_from) || exclude.is_match(path_to) {
                                if config::OPTS.verbose {
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
                        let _ = shared_tx.send(path_from.to_owned());
                        let _ = shared_tx.send(path_to.to_owned());
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
        .expect("Could not spawn watcher thread");
}
