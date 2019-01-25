use crate::cli;
use regex::Regex;
use std::path::PathBuf;

lazy_static! {
    // configuration options
    pub static ref OPTS: Config = Config::from(cli::Options::load());
}

#[derive(Debug)]
pub struct Config {
    pub log_file: Option<PathBuf>,
    pub dry_run: bool,
    pub init: bool,
    pub verbose: bool,
    pub entries: Vec<Entry>
}

impl Config {
    // convert ConfigFromToml to Config
    pub fn from(options: cli::Options) -> Self {
        // parse configuration file and command line options
        let config_toml = ConfigFromToml::from(&options);

        // override file configuration with command line options
        Self {
            log_file: options.log_file.or(config_toml.log_file),
            dry_run: options.dry_run || config_toml.dry_run.unwrap_or_default(),
            init: options.init || config_toml.init.unwrap_or_default(),
            verbose: options.verbose || config_toml.verbose.unwrap_or_default(),
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
            // ensure `path` exists
            path: if entry_toml.path.exists() {
                entry_toml.path.to_owned()
            }
            else {
                panic!("No such file or directory {:#?}", entry_toml.path);
            },
            recursive: entry_toml.recursive.unwrap_or_default(),
            // ensure `interval` is not negative
            interval: match entry_toml.interval {
                Some(value) if value.is_sign_positive() => value,
                Some(value) => panic!("Interval can't be negative: {}", value),
                None => f64::default()
            },
            excludes: entry_toml
                .excludes
                .to_owned()
                .unwrap_or_default()
                .iter()
                .map(|x| {
                    // compile each exclude string
                    Regex::new(&x).unwrap_or_else(|_| panic!("Could not parse expression {:#?}", x))
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
    fn from(options: &cli::Options) -> Self {
        // parse configuration from file
        toml::from_str(
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
        })
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
