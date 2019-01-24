use crate::config;
use lazy_static::lazy_static;
use slog::{Drain, Duplicate, Fuse, Level, LevelFilter, Logger};
use slog_async::Async;
use slog_term::{FullFormat, PlainDecorator};
use std::{fs::OpenOptions, path::PathBuf};

lazy_static! {
    // root logger
    pub static ref ROOT: Logger = self::new(&config::OPTS.log_file);
}

// create an asynchronous terminal drain
fn term_drain() -> Fuse<Async> {
    let decorator = PlainDecorator::new(std::io::stdout());
    let drain = FullFormat::new(decorator).build().fuse();

    Async::new(drain).build().fuse()
}

// create an asynchronous file drain for `log_path`
fn file_drain(log_path: &PathBuf) -> Fuse<Async> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();

    let file_decorator = PlainDecorator::new(file);
    let file_drain = FullFormat::new(file_decorator).build().fuse();

    Async::new(file_drain).build().fuse()
}

fn new(log_path: &Option<PathBuf>) -> Logger {
    // always log to stdout
    let term_filter = LevelFilter::new(self::term_drain(), Level::Info);
    let options = o!();

    if let Some(log_path) = log_path {
        // duplicate logs to file in `log_path`
        Logger::root(
            Duplicate::new(
                term_filter,
                LevelFilter::new(self::file_drain(log_path), Level::Info)
            )
            .fuse(),
            options
        )
    }
    else {
        Logger::root(term_filter.fuse(), options)
    }
}
