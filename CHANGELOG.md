<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Fixed

- Updated dependencies to fix security vulnerabilities
- Fix clippy needless_borrow warning

## [0.2.2] - 2019-02-01

### Fixed

- `init` does not apply anymore when configuration reload fails

## [0.2.1] - 2019-02-01

### Changed

- Documented the inability to reload `log_path`

## [0.2.0] - 2019-02-01

### Added

- Configuration reload via SIGHUP

### Changed

- Handle and index watchers with keys instead of indices
- General code structure "improvements"

## [0.1.4] - 2019-01-29

### Changed

- Rename `interval` option to `delay` to better convey its meaning

## [0.1.3] - 2019-01-28

### Changed

- Notify about the unimplemented SIGHUP handler

### Fixed

- Fix RPM %config directive

## [0.1.2] - 2019-01-28

### Changed

- Switch from asynchronous to synchronous command execution
- Increase handler verbosity

## [0.1.1] - 2019-01-28

### Fixed

- Fix CentOS 7 RPM systemd unit path
