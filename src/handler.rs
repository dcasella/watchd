use crate::{config, logger};
use std::{process::Command, sync::mpsc, thread, time::Duration};

pub fn spawn(index: usize, shared_rx: mpsc::Receiver<usize>) {
    // generate thread name for logging purposes
    let thread_name = format!("handler-{}", index);

    thread::Builder::new()
        .name(thread_name.clone())
        .spawn(move || {
            let thread_log = logger::ROOT.new(o!(
                "thread" => thread_name,
                "id" => index
            ));

            if config::OPTS.verbose {
                if let Some(path) = &config::OPTS.entries[index].path.to_str() {
                    info!(
                        thread_log, "SPAWN";
                        "entry-path" => path
                    );
                }
            }

            let mut synced = !config::OPTS.init;

            if config::OPTS.verbose && !synced {
                info!(
                    thread_log, "INIT";
                    "sync" => true
                );
            }

            // event loop
            loop {
                if synced {
                    // watch for events on `shared_rx`
                    loop {
                        if shared_rx
                            .recv_timeout(Duration::from_millis(
                                (config::OPTS.entries[index].interval * 1000_f64) as u64
                            ))
                            .is_ok()
                        {
                            if config::OPTS.verbose {
                                info!(thread_log, "EVENT");
                            }

                            synced = false;
                        }
                        else {
                            break;
                        }
                    }
                }

                if !synced {
                    if config::OPTS.dry_run {
                        info!(
                            thread_log, "RUN";
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
                                    thread_log, "RUN";
                                    "command" => command,
                                    "output" => stdout.trim_end()
                                ),
                                Err(err) => warn!(
                                    thread_log, "RUN";
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
        .expect("Could not spawn handler thread");
}
