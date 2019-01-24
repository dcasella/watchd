use lazy_static::lazy_static;
use slog::{Drain, Duplicate, Level, LevelFilter, Logger};
use slog_async::Async;
use slog_term::{FullFormat, PlainDecorator};
use std::fs::OpenOptions;

use crate::config::Config;

lazy_static! {
    pub static ref ROOT: Logger = self::create("watchd.log");
}

fn create(path: &'static str) -> Logger {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    let stdout_decorator = PlainDecorator::new(std::io::stdout());
    let term_drain = FullFormat::new(stdout_decorator).build().fuse();
    let term_drain = Async::new(term_drain).build().fuse();

    let file_decorator = PlainDecorator::new(file);
    let file_drain = FullFormat::new(file_decorator).build().fuse();
    let file_drain = Async::new(file_drain).build().fuse();

    Logger::root(
        Duplicate::new(
            LevelFilter::new(term_drain, Level::Info),
            LevelFilter::new(file_drain, Level::Info)
        )
        .fuse(),
        o!("program" => env!("CARGO_PKG_NAME"))
    )
}
