use std::{process::Command, sync::mpsc, time::Duration};

use notify::{DebouncedEvent, RecursiveMode, Watcher};

use crate::config::Entry;

pub fn spawn(entry: Entry, init: bool, dry_run: bool, verbose: bool) {
    std::thread::Builder::new()
        .name("notify-monitor".to_string())
        .spawn(move || {
            // event channel
            let (tx, rx) = mpsc::channel();

            // debounced (1s) events watcher
            let mut watcher = notify::watcher(tx, Duration::from_millis(10)).unwrap();

            // add entry path to the watcher
            watcher
                .watch(
                    &entry.path,
                    if entry.recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    },
                )
                .unwrap();

            let mut synced = !init;

            // event loop
            loop {
                if synced {
                    'match_loop: loop {
                        match rx
                            .recv_timeout(Duration::from_millis((entry.interval * 1000_f64) as u64))
                        {
                            Ok(DebouncedEvent::Create(path))
                            | Ok(DebouncedEvent::Write(path))
                            | Ok(DebouncedEvent::Chmod(path))
                            | Ok(DebouncedEvent::Remove(path)) => {
                                for exclude in &entry.excludes {
                                    if let Some(path) = path.to_str() {
                                        if exclude.is_match(path) {
                                            if verbose {
                                                println!("Event for {:?} ignored with exclude \"{:?}\" for {:?}", &entry.path, &exclude, &path);
                                            }

                                            continue 'match_loop;
                                        }
                                    }
                                }

                                if verbose {
                                    println!("Event for {:?} for {:?}", &entry.path, &path);
                                }

                                synced = false;
                            }
                            Ok(DebouncedEvent::Rename(path_from, path_to)) => {
                                for exclude in &entry.excludes {
                                    if let Some(path_from) = path_from.to_str() {
                                        if let Some(path_to) = path_to.to_str() {
                                            if exclude.is_match(path_from)
                                                || exclude.is_match(path_to)
                                            {
                                                if verbose {
                                                    println!("Event for {:?} ignored with exclude \"{:?}\" on Rename from {:?} to {:?}", &entry.path, &exclude, &path_from, &path_to);
                                                }

                                                continue 'match_loop;
                                            }
                                        }
                                    }
                                }

                                if verbose {
                                    println!("Event for {:?} from {:?} to {:?}", &entry.path, &path_from, &path_to);
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
                    if dry_run {
                        println!("Run command {:#?} for {:?}", &entry.commands, &entry.path);
                    } else {
                        // TODO: run entry.commands
                        for command in &entry.commands {
                            if verbose {
                                println!("Run {:?}", &command);
                            }

                            let output = Command::new("sh").arg("-c").arg(&command).output().unwrap_or_else(|_| {
                                panic!("Run {:?} failed", &command);
                            });

                            if verbose {
                                match String::from_utf8(output.stdout) {
                                    Ok(stdout) => println!("Stdout: {}", stdout),
                                    Err(err) => eprintln!("Stdout error: {}", err)
                                }
                            }
                        }
                    }

                    synced = true;
                }
            }
        })
        .expect("Could not spawn watcher thread");
}
