# Changelog
<!-- We follow the Keep a Changelog standard https://keepachangelog.com/en/1.0.0/ -->

## [Unreleased]
### Added
- [#49](https://github.com/tweag/nixtract/pull/49) rewrite describe_derivation to include all found derivations (but actively skip bootstrap packages)

### Fixed
- [#50](https://github.com/tweag/nixtract/pull/50) fix an issue where the root attribute was never assumed to be a derivation

## [0.3.0] - 2024-04-17
### Added
- [#34](https://github.com/tweag/nixtract/pull/34) add option to provide nixtract with a status communication channel
- [#36](https://github.com/tweag/nixtract/pull/36) add option to only extract runtime dependencies
- [#40](https://github.com/tweag/nixtract/pull/40) log warning when narinfo fetching fails

### Fixed
- [#38](https://github.com/tweag/nixtract/pull/38) fixed bug where found derivations were parsed incorrectly
- [#42](https://github.com/tweag/nixtract/pull/42) reintroduced the src field in the derivation description
- [#43](https://github.com/tweag/nixtract/pull/43) enables the `flakes` and `nix-command` features for nix invocations, this avoids users having to have them enabled manually

### Changed
- [#36](https://github.com/tweag/nixtract/pull/36) moved all nixtract configuration options into a single struct passed to the `nixtract` function

[0.3.0]: https://github.com/tweag/nixtract/compare/v0.2.0...v0.3.0
