use crate::{config, logger};
use std::{process::Command, str, sync::mpsc::Receiver, thread, time::Duration};

struct Pending {
    command: bool,
    loop_break: bool
}

pub fn spawn(index: usize, shared_rx: Receiver<usize>) {
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

            // if `init` is true, run the command first thing in the loop
            let mut pending = Pending {
                command: config::OPTS.init,
                loop_break: false
            };

            if config::OPTS.verbose && pending.command {
                info!(
                    thread_log, "INIT";
                    "sync" => true
                );
            }

            // thread loop
            loop {
                if !pending.command {
                    // watch for events on `shared_rx`
                    loop {
                        // note that either `recv` or `recv_timeout` can update the value of
                        // `pending.command`
                        pending = if config::OPTS.entries[index].delay == 0.0 {
                            // handle null `delay`
                            self::recv(&thread_log, &shared_rx)
                        }
                        else {
                            self::recv_timeout(&thread_log, &shared_rx, pending.command, index)
                        };

                        if pending.loop_break {
                            // break out of the watch loop:
                            // if a command execution is pending it will be consumed, otherwise the
                            // watch loop will resume
                            break;
                        }
                    }
                }

                if pending.command {
                    // log the commands
                    if config::OPTS.dry_run {
                        info!(
                            thread_log, "RUN";
                            "mode" => "dry",
                            "commands" => format!("{:?}", config::OPTS.entries[index].commands)
                        );
                    }
                    // execute the commands with `sh -c ...`
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
                                .expect("Error while executing command");

                            if config::OPTS.verbose {
                                info!(
                                    thread_log, "OUTPUT";
                                    "stdout" => match str::from_utf8(&output.stdout) {
                                        Ok(value) => value,
                                        Err(err) => panic!("Could not parse stdout: {}", err)
                                    },
                                    "stderr" => match str::from_utf8(&output.stderr) {
                                        Ok(value) => value,
                                        Err(err) => panic!("Could not parse stderr: {}", err)
                                    }
                                );
                            }

                            if !output.status.success() {
                                break;
                            }
                        }
                    }

                    // notify that a command was executed
                    pending.command = false;
                }
            }
        })
        .expect("Could not spawn handler thread");
}

fn recv(thread_log: &slog::Logger, shared_rx: &Receiver<usize>) -> Pending {
    // received an event
    match shared_rx.recv() {
        Ok(_) => {
            info!(thread_log, "EVENT");

            // notify that a command execution is pending
            Pending {
                command: true,
                loop_break: true
            }
        }
        Err(err) => {
            crit!(
                thread_log, "EVENT";
                "message" => err.to_string(),
                "error" => true
            );

            panic!("Error while receiving event: {}", err)
        }
    }
}

fn recv_timeout(
    thread_log: &slog::Logger,
    shared_rx: &Receiver<usize>,
    pending_command: bool,
    index: usize
) -> Pending {
    // received an event before timeout elapsed
    if shared_rx
        .recv_timeout(Duration::from_millis(
            (config::OPTS.entries[index].delay * 1000_f64) as u64
        ))
        .is_ok()
    {
        info!(thread_log, "EVENT");

        // notify that a command execution is pending
        Pending {
            command: true,
            loop_break: false
        }
    }
    // no event received before timeout
    else {
        Pending {
            command: pending_command,
            loop_break: true
        }
    }
}
