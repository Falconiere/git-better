# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/Falconiere/git-better/releases/tag/v1.0.0) - 2026-08-06

### Added

- add gb conventions and gb skill install, plus a tagged release pipeline

### CI

- adopt the comemory release pattern (release-plz + dist, shared tap)
- code review via toolu-ghactions/code-review@main

### Documentation

- *(release)* fold the release-plz.toml header into the workspace section
- scope to Apple Silicon (aarch64-apple-darwin) only
- clarify CI runs on ubuntu even though project targets macOS

### Fixed

- *(ci)* drop the dead BSD stat fallback and trim release-plz.toml comments
- address code-review findings on conventions, skill install, and release
- correct read-op bugs found in code review

### M0+M1

- skeleton + read ops with --better and syntax-highlighted diffs

### Maintenance

- drop unused tokio dependency and bump to 1.0.0
- align toolchain + lint/format/nextest configs with yamless

### Testing

- cover the cache invalidation paths and the with-remote flag
