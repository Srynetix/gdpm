# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2022-05-23

### Added

- New `list-remote` command which scans official mirror to fetch all available Godot versions (starting from 3.0)

### Fixed

- Fix reading engine global configuration file when it's not created

### Changed

- New adapter system to ease testing
- The `install` and `uninstall` commands now accept a "qualified" version name, like "3.2.rc1" or "3.2.rc1.mono" (instead of individual flags)

## [1.1.1] - 2022-05-16

### Fixed

- Fix default paths on commands

## [1.1.0] - 2022-05-16

### Added

- You can now download and install versions from official Godot mirrors.

### Changed

- Project reorganization in multiple crates
- Dependencies update

## [1.0.1] - 2019-11-28

### Fixed

- Create addons folder before adding dependencies ([#18](https://github.com/Srynetix/gdpm/pull/18))

## [1.0.0] - 2019-11-27

- Initial version

[Unreleased]: https://github.com/Srynetix/gdpm/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/Srynetix/gdpm/releases/tag/v1.2.0
[1.1.1]: https://github.com/Srynetix/gdpm/releases/tag/v1.1.1
[1.1.0]: https://github.com/Srynetix/gdpm/releases/tag/v1.1.0
[1.0.1]: https://github.com/Srynetix/gdpm/releases/tag/v1.0.1
[1.0.0]: https://github.com/Srynetix/gdpm/releases/tag/v1.0.0