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

            // if `init` is true, run the command first thing in the loop
            let mut pending_command = config::OPTS.init;

            if config::OPTS.verbose && pending_command {
                info!(
                    thread_log, "INIT";
                    "sync" => true
                );
            }

            // thread loop
            loop {
                if !pending_command {
                    // watch for events on `shared_rx`
                    loop {
                        // received an event before timeout elapsed
                        if shared_rx
                            .recv_timeout(Duration::from_millis(
                                (config::OPTS.entries[index].interval * 1000_f64) as u64
                            ))
                            .is_ok()
                        {
                            if config::OPTS.verbose {
                                info!(thread_log, "EVENT");
                            }

                            // notify that a command execution is pending
                            pending_command = true;
                        }
                        // no event received before timeout
                        else {
                            // break out of the watch loop:
                            // if a command execution is pending it will be consumed, otherwise the
                            // watch loop will resume
                            break;
                        }
                    }
                }

                if pending_command {
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

                            let _ = Command::new("sh").arg("-c").arg(&command).spawn();
                        }
                    }

                    // notify that a command was executed
                    pending_command = false;
                }
            }
        })
        .expect("Could not spawn handler thread");
}
