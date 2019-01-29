# watchd

A fearlessly concurrent filesystem watcher daemon.  
Watch for file or directory changes (content, permissions) and execute commands.

## Build

Install a Rust toolchain with Cargo.  
[rustup](https://rustup.rs/) may help with that.

### Build on Debian

Install `cargo-deb` and use the Makefile:

```
cargo install cargo-deb
make deb
```

The `.deb` file will be in the `./releases` directory.

### Build on RHEL

Install `cargo-rpm` and use the Makefile:

```
cargo install cargo-rpm
make rpm
```

The `.rpm` file will be in the `./releases` directory.

## Install

### Install on Debian

[Build](#build-on-debian) it first, then:

```
dpkg -i ./target/debian/watchd....deb
```

### Install on RHEL

[Build](#build-on-rhel) it first, then:

```
rpm -i ./target/release/rpmbuild/RPMS/x86_64/watchd....rpm
```

## Configuration

The default configuration does a decent job at explaining things; this is just a Markdownification of it.

- Default [`config.toml`](package/etc/config.toml).
- Example [`config.toml`](examples/config.toml).

### Global

#### `log-file`

Duplicate stdout logs to the specified file.

**Default:** None

**Example:**

```toml
log-file = "/var/log/watchd.log"
```

#### `dry-run`

Log commands without executing.

**Default:** `false`

**Example:**

```toml
dry-run = true
```

#### `init`

Execute commands on program start.

**Default:** `false`

**Example:**

```toml
init = true
```

#### `verbose`

Increment log verbosity.

**Default:** `false`

**Example:**

```toml
verbose = true
```

#### `entry` sections

File or directory to watch.  
Multiple entries allowed.

##### `path`

Path to watch.

- String
- Required
- Must be a valid filesystem path

##### `recursive`

Watch `path` and its subdirectories.

- Boolean
- Default: `false`

##### `delay`

Time between the last received event and the command to execute.

- Float64
- Default: `0.0`
- Must be non-negative

##### `exclude`

List of patterns to ignore.  
Syntax: [`docs.rs`](https://docs.rs/regex/1.1.0/regex/#syntax).

- []String
- Default: `[]`
- Must be valid regular expressions
- Backslashes must be escaped

##### `command`

- []String
- Required
- Commands are executed via `$(sh -c "${command}")`
