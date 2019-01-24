use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use std::path::PathBuf;

static DEFAULT_CONFIG_FILE: &str = "/etc/watchd/config.toml";

pub struct Options {
    pub config_file: PathBuf,
    pub log_file: Option<PathBuf>,
    pub init: bool,
    pub verbose: bool,
    pub dry_run: bool
}

impl Options {
    pub fn load() -> Self {
        let matches = app_from_crate!()
            .arg(
                Arg::with_name("config-file")
                    .short("f")
                    .long("config-file")
                    .help("Specify configuration file")
                    .empty_values(false)
                    .value_name("FILE")
                    .default_value(self::DEFAULT_CONFIG_FILE)
            )
            .arg(
                Arg::with_name("log-file")
                    .long("log-file")
                    .help("Specify log file")
                    .empty_values(false)
                    .value_name("FILE")
            )
            .arg(
                Arg::with_name("dry-run")
                    .short("d")
                    .long("dry-run")
                    .help("Print commands instead of executing them")
            )
            .arg(
                Arg::with_name("init")
                    .short("i")
                    .long("init")
                    .help("Initial synchronization process")
            )
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .help("Enable verbosity")
            )
            .get_matches();

        Self {
            config_file: PathBuf::from(matches.value_of("config-file").unwrap()),
            log_file: matches.value_of("log-file").map(PathBuf::from),
            dry_run: matches.is_present("dry-run"),
            init: matches.is_present("init"),
            verbose: matches.is_present("verbose")
        }
    }
}
