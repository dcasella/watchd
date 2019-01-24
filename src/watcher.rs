use crate::{config, logger};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::{process::Command, sync::mpsc, thread, time::Duration};

pub fn spawn(index: usize) {
    // generate thread name for logging purposes
    let thread_name = format!("watcher-{}", index);

    thread::Builder::new()
        .name(thread_name.clone())
        .spawn(move || {
            // event channel
            let (tx, rx) = mpsc::channel();

            // debounced (10ms) events watcher
            let mut watcher = notify::watcher(tx, Duration::from_millis(10)).unwrap();

            // add entry path to the watcher
            watcher
                .watch(
                    &config::OPTS.entries[index].path,
                    if config::OPTS.entries[index].recursive {
                        RecursiveMode::Recursive
                    }
                    else {
                        RecursiveMode::NonRecursive
                    }
                )
                .unwrap();

            let thread_log = logger::ROOT.new(o!("thread" => thread_name));

            if config::OPTS.verbose {
                if let Some(path) = &config::OPTS.entries[index].path.to_str() {
                    info!(
                        thread_log,
                        "SPAWN";
                        "entry-path" => path
                    );
                }
            }

            let mut synced = !config::OPTS.init;

            // event loop
            loop {
                if synced {
                    'match_loop: loop {
                        match rx.recv_timeout(Duration::from_millis(
                            (config::OPTS.entries[index].interval * 1000_f64) as u64
                        )) {
                            Ok(DebouncedEvent::Create(path))
                            | Ok(DebouncedEvent::Write(path))
                            | Ok(DebouncedEvent::Chmod(path))
                            | Ok(DebouncedEvent::Remove(path)) => {
                                for exclude in &config::OPTS.entries[index].excludes {
                                    if let Some(path) = path.to_str() {
                                        if exclude.is_match(path) {
                                            if config::OPTS.verbose {
                                                info!(
                                                    thread_log,
                                                    "EVENT";
                                                    "exclude" => true,
                                                    "pattern" => exclude.as_str(),
                                                    "path" => path
                                                );
                                            }

                                            continue 'match_loop;
                                        }
                                    }
                                }

                                if config::OPTS.verbose {
                                    if let Some(path) = path.to_str() {
                                        info!(
                                            thread_log,
                                            "EVENT";
                                            "path" => path
                                        );
                                    }
                                }

                                synced = false;
                            }
                            Ok(DebouncedEvent::Rename(path_from, path_to)) => {
                                for exclude in &config::OPTS.entries[index].excludes {
                                    if let Some(path_from) = path_from.to_str() {
                                        if let Some(path_to) = path_to.to_str() {
                                            if exclude.is_match(path_from)
                                                || exclude.is_match(path_to)
                                            {
                                                if config::OPTS.verbose {
                                                    info!(
                                                        thread_log,
                                                        "EVENT";
                                                        "exclude" => true,
                                                        "pattern" => exclude.as_str(),
                                                        "path-from" => path_from,
                                                        "path-to" => path_to
                                                    );
                                                }

                                                continue 'match_loop;
                                            }
                                        }
                                    }
                                }

                                if config::OPTS.verbose {
                                    if let Some(path_from) = path_from.to_str() {
                                        if let Some(path_to) = path_to.to_str() {
                                            info!(
                                                thread_log,
                                                "EVENT";
                                                "path-from" => path_from,
                                                "path-to" => path_to
                                            );
                                        }
                                    }
                                }

                                synced = false;
                            }
                            Err(_) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }

                if !synced {
                    if config::OPTS.dry_run {
                        info!(
                            thread_log,
                            "RUN";
                            "mode" => "dry",
                            "commands" => format!("{:?}", config::OPTS.entries[index].commands)
                        );
                    }
                    else {
                        for command in &config::OPTS.entries[index].commands {
                            info!(
                                thread_log, "RUN";
                                "command" => command
                            );

                            let output = Command::new("sh")
                                .arg("-c")
                                .arg(&command)
                                .output()
                                .unwrap_or_else(|_| {
                                    crit!(
                                        thread_log, "RUN";
                                        "command" => command,
                                        "error" => true
                                    );
                                    panic!("error: {}", command);
                                });

                            match String::from_utf8(output.stdout) {
                                Ok(stdout) => info!(
                                    thread_log,
                                    "RUN";
                                    "command" => command,
                                    "output" => stdout.trim_end()
                                ),
                                Err(err) => warn!(
                                    thread_log,
                                    "RUN";
                                    "error" => true,
                                    "command" => command,
                                    "output" => err.to_string()
                                )
                            }
                        }
                    }

                    synced = true;
                }
            }
        })
        .expect("could not spawn watcher thread");
}