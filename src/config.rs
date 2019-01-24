use crate::cli;
use regex::Regex;
use std::path::PathBuf;

lazy_static! {
    pub static ref OPTS: Config = Config::from(cli::Options::load());
}

#[derive(Debug)]
pub struct Config {
    pub log_file: PathBuf,
    pub dry_run: bool,
    pub init: bool,
    pub verbose: bool,
    pub entries: Vec<Entry>
}

impl Config {
    // convert ConfigFromToml to Config
    pub fn from(options: cli::Options) -> Self {
        // parse configuration file and command line options
        let config_toml = ConfigFromToml::from(options);

        Self {
            log_file: config_toml.log_file.unwrap(),
            dry_run: config_toml.dry_run.unwrap_or_default(),
            init: config_toml.init.unwrap_or_default(),
            verbose: config_toml.verbose.unwrap_or_default(),
            entries: config_toml.entries.iter().map(Entry::from).collect()
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub recursive: bool,
    pub interval: f64,
    pub excludes: Vec<Regex>,
    pub commands: Vec<String>
}

impl Entry {
    // convert EntryFromToml to Entry
    fn from(entry_toml: &EntryFromToml) -> Self {
        Self {
            path: if entry_toml.path.exists() {
                entry_toml.path.to_owned()
            }
            else {
                panic!("No such file or directory {:#?}", entry_toml.path);
            },
            recursive: entry_toml.recursive.unwrap_or_default(),
            interval: entry_toml.interval.unwrap_or_default(),
            excludes: entry_toml
                .excludes
                .to_owned()
                .unwrap_or_default()
                .iter()
                .map(|x| {
                    // compile each exclude string
                    Regex::new(&x).unwrap_or_else(|_| panic!("Could not parse RegExp {:#?}", x))
                })
                .collect(),
            commands: entry_toml.commands.to_owned()
        }
    }
}

// configuration struct for TOML parsing (optional values and rename/alias)
#[derive(Deserialize)]
struct ConfigFromToml {
    #[serde(rename = "log-file")]
    log_file: Option<PathBuf>,
    #[serde(rename = "dry-run")]
    dry_run: Option<bool>,
    init: Option<bool>,
    verbose: Option<bool>,
    #[serde(rename = "entry")]
    entries: Vec<EntryFromToml>
}

impl ConfigFromToml {
    fn from(options: crate::cli::Options) -> Self {
        // parse configuration from file
        let mut config_toml: Self = toml::from_str(
            &std::fs::read_to_string(&options.config_file).unwrap_or_else(|err| {
                panic!(
                    "Could not open configuration file {:?}: {}",
                    &options.config_file, err
                )
            })
        )
        .unwrap_or_else(|err| {
            panic!(
                "Could not parse configuration file {:#?}: {}",
                &options.config_file, err
            )
        });

        // override file configuration with command line options

        if options.log_file != PathBuf::from(crate::DEFAULT_LOG_PATH)
            || config_toml.log_file.is_none()
        {
            config_toml.log_file = Some(options.log_file);
        }

        if options.dry_run {
            config_toml.dry_run = Some(true);
        }

        if options.init {
            config_toml.init = Some(true);
        }

        if options.verbose {
            config_toml.verbose = Some(true);
        }

        config_toml
    }
}

// configuration entry struct for TOML parsing (optional values and
// rename/alias)
#[derive(Deserialize)]
struct EntryFromToml {
    path: PathBuf,
    recursive: Option<bool>,
    interval: Option<f64>,
    #[serde(alias = "exclude")]
    excludes: Option<Vec<String>>,
    #[serde(alias = "command")]
    commands: Vec<String>
}
