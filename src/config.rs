use crate::{cli, logger};
use regex::Regex;
use std::{collections::HashMap, path::PathBuf, sync::RwLock};

lazy_static! {
    // configuration options
    pub static ref OPTS: RwLock<Config> = RwLock::new(Config::from(cli::Options::load()));
}

pub struct Config {
    pub log_file: Option<PathBuf>,
    pub dry_run: bool,
    pub init: bool,
    pub verbose: bool,
    pub entries: HashMap<PathBuf, Entry>,
    options: cli::Options
}

impl Config {
    // convert ConfigFromToml to Config
    pub fn from(options: cli::Options) -> Self {
        // parse configuration file and command line options
        let config_toml = ConfigFromToml::from(&options);

        // override file configuration with command line options
        Self {
            log_file: options.log_file.to_owned().or(config_toml.log_file),
            dry_run: options.dry_run || config_toml.dry_run.unwrap_or_default(),
            init: options.init || config_toml.init.unwrap_or_default(),
            verbose: options.verbose || config_toml.verbose.unwrap_or_default(),
            entries: config_toml
                .entries
                .iter()
                .map(|entry_toml| {
                    // map EntryFromToml to (PathBuf, Entry)
                    (
                        // ensure `path` exists
                        if entry_toml.path.exists() {
                            entry_toml.path.to_owned()
                        }
                        else {
                            panic!("No such file or directory {}", entry_toml.path.display());
                        },
                        Entry::from(entry_toml)
                    )
                })
                .collect(),
            options
        }
    }

    pub fn reload(&mut self) -> Result<(), ()> {
        // parse configuration file and command line options
        match ConfigFromToml::reload_from(&self.options) {
            Ok(config_toml) => {
                let mut entries: HashMap<PathBuf, Entry> =
                    HashMap::with_capacity(config_toml.entries.len());

                for entry_toml in config_toml.entries {
                    // map EntryFromToml to (PathBuf, Entry)
                    entries.insert(
                        // ensure `path` exists
                        if entry_toml.path.exists() {
                            entry_toml.path.to_owned()
                        }
                        else {
                            error!(
                                logger::ROOT, "RELOAD";
                                "reason" => "No such file or directory",
                                "path" => entry_toml.path.display()
                            );

                            return Err(());
                        },
                        match Entry::reload_from(&entry_toml) {
                            Ok(entry) => entry,
                            Err(_) => return Err(())
                        }
                    );
                }

                self.log_file = self.options.log_file.to_owned().or(config_toml.log_file);
                self.dry_run = self.options.dry_run || config_toml.dry_run.unwrap_or_default();
                self.init = self.options.init || config_toml.init.unwrap_or_default();
                self.verbose = self.options.verbose || config_toml.verbose.unwrap_or_default();
                self.entries = entries;

                // TODO: update logger::ROOT log_file

                Ok(())
            }
            Err(_) => Err(())
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub recursive: bool,
    pub delay: f64,
    pub excludes: Vec<Regex>,
    pub commands: Vec<String>
}

impl Entry {
    // convert EntryFromToml to Entry
    fn from(entry_toml: &EntryFromToml) -> Self {
        Self {
            recursive: entry_toml.recursive.unwrap_or_default(),
            // ensure `delay` is not negative
            delay: match entry_toml.delay {
                Some(value) if value.is_sign_positive() => value,
                Some(value) => panic!("Delay shall not be negative: {}", value),
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

    fn reload_from(entry_toml: &EntryFromToml) -> Result<Self, ()> {
        let mut excludes = vec![];

        for exclude in entry_toml.excludes.to_owned().unwrap_or_default() {
            // compile each exclude string
            match Regex::new(&exclude) {
                Ok(expr) => excludes.push(expr),
                Err(err) => {
                    error!(
                        logger::ROOT, "RELOAD";
                        "reason" => "Could not parse expression",
                        "message" => err.to_string()
                    );

                    return Err(());
                }
            }
        }

        Ok(Self {
            recursive: entry_toml.recursive.unwrap_or_default(),
            // ensure `delay` is not negative
            delay: match entry_toml.delay {
                Some(value) if value.is_sign_positive() => value,
                Some(value) => {
                    error!(
                        logger::ROOT, "RELOAD";
                        "reason" => "Delay shall not be negative",
                        "value" => value
                    );

                    return Err(());
                }
                None => f64::default()
            },
            excludes,
            commands: entry_toml.commands.to_owned()
        })
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

    fn reload_from(options: &cli::Options) -> Result<Self, toml::de::Error> {
        // parse configuration from file
        toml::from_str(
            &std::fs::read_to_string(&options.config_file).unwrap_or_else(|err| {
                error!(
                    logger::ROOT, "RELOAD";
                    "reason" => "Could not open configuration file",
                    "path" => options.config_file.display(),
                    "message" => err.to_string()
                );

                "".to_string()
            })
        )
    }
}

// configuration entry struct for TOML parsing (optional values and
// rename/alias)
#[derive(Deserialize)]
struct EntryFromToml {
    path: PathBuf,
    recursive: Option<bool>,
    delay: Option<f64>,
    #[serde(alias = "exclude")]
    excludes: Option<Vec<String>>,
    #[serde(alias = "command")]
    commands: Vec<String>
}
