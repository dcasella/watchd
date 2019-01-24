use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use std::{
    borrow::Cow,
    path::{Path, PathBuf}
};

pub struct Options {
    pub config: PathBuf,
    pub init: bool,
    pub verbose: bool,
    pub dry_run: bool
}

impl Options {
    pub fn load() -> Self {
        let mut options = Options::default();

        let matches = app_from_crate!()
            .arg(
                Arg::with_name("config-file")
                    .short("f")
                    .long("config-file")
                    .help("Specify configuration file")
                    .empty_values(false)
                    .value_name("FILE")
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
            .arg(
                Arg::with_name("dry-run")
                    .short("d")
                    .long("dry-run")
                    .help("Print commands instead of executing them")
            )
            .get_matches();

        if let Some(path) = matches.value_of("config-file") {
            options.config = PathBuf::from(path.to_string());
        }

        if matches.is_present("init") {
            options.init = true;
        }

        if matches.is_present("verbose") {
            options.verbose = true;
        }

        if matches.is_present("dry-run") {
            options.dry_run = true;
        }

        options
    }

    pub fn config_path(&self) -> Cow<Path> {
        Cow::Borrowed(self.config.as_path())
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            config: PathBuf::from("/etc/watchd/config.toml"),
            init: false,
            verbose: false,
            dry_run: false
        }
    }
}
