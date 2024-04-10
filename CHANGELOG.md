# Changelog
<!-- We follow the Keep a Changelog standard https://keepachangelog.com/en/1.0.0/ -->

## [Unreleased]
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
