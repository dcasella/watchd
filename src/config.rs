use crate::cli;
use regex::Regex;
use std::path::PathBuf;

lazy_static! {
    pub static ref OPTS: Config = Config::from(&cli::Options::load());
}

#[derive(Debug)]
pub struct Config {
    pub init: bool,
    pub verbose: bool,
    pub dry_run: bool,
    pub entries: Vec<Entry>
}

impl Config {
    pub fn from(options: &crate::cli::Options) -> Self {
        let mut config = Self::default();

        // parse configuration file and command line options
        let config_toml = ConfigFromToml::from(options);

        // convert ConfigFromToml to Config

        if let Some(value) = config_toml.init {
            config.init = value;
        }

        if let Some(value) = config_toml.verbose {
            config.verbose = value;
        }

        if let Some(value) = config_toml.dry_run {
            config.dry_run = value;
        }

        for entry_toml in config_toml.entries {
            config.entries.push(Entry::from(&entry_toml));
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init: false,
            verbose: false,
            dry_run: false,
            entries: Vec::new()
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
    fn from(entry_toml: &EntryFromToml) -> Self {
        let mut entry = Self::default();

        // retrieve path from entry_toml
        if entry_toml.path.exists() {
            entry.path = entry_toml.path.clone();
        }
        else {
            panic!("No such file or directory {:#?}", entry_toml.path);
        }

        // if entry_toml.recursive is set, retrieve it
        if let Some(value) = entry_toml.recursive {
            entry.recursive = value;
        }

        // retrieve commands from entry_toml
        entry.commands = entry_toml.commands.clone();

        // if entry_toml.interval is set, retrieve it
        if let Some(value) = entry_toml.interval {
            entry.interval = value;
        }

        // if entry_toml.exclude is set, retrieve it
        if let Some(expressions) = entry_toml.excludes.clone() {
            // convert each string expression in a regex
            for expr in expressions {
                entry.excludes.push(
                    Regex::new(&expr)
                        .unwrap_or_else(|_| panic!("Could not parse RegExp {:#?}", expr))
                );
            }
        }

        entry
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            path: PathBuf::default(),
            recursive: false,
            interval: 0.0,
            excludes: Vec::new(),
            commands: Vec::new()
        }
    }
}

// configuration struct for TOML parsing (optional values and rename/alias)
#[derive(Deserialize)]
struct ConfigFromToml {
    init: Option<bool>,
    verbose: Option<bool>,
    #[serde(rename = "dry-run")]
    dry_run: Option<bool>,
    #[serde(rename = "entry")]
    entries: Vec<EntryFromToml>
}

impl ConfigFromToml {
    fn from(options: &crate::cli::Options) -> Self {
        // configuration file path
        let config_path = options.config_path();

        // parse configuration from file
        let mut config: Self = toml::from_str(
            &std::fs::read_to_string(&config_path).unwrap_or_else(|err| {
                panic!(
                    "Could not open configuration file {:?}: {}",
                    &config_path, err
                )
            })
        )
        .unwrap_or_else(|err| {
            panic!(
                "Could not parse configuration file {:#?}: {}",
                &config_path, err
            )
        });

        // override file configuration with command line options

        if options.init {
            config.init = Some(true);
        }

        if options.verbose {
            config.verbose = Some(true);
        }

        if options.dry_run {
            config.dry_run = Some(true);
        }

        config
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
