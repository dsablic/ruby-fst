# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `Map.from_path_mmap` and `Set.from_path_mmap` for memory-mapped loading of large FSTs.
- `Map#range(ge:, le:)` and `Set#range(ge:, le:)` streaming range iteration.
- `Map#starts_with(prefix)` and `Set#starts_with(prefix)` prefix scans.
- `Map#get_le_value` and `Map#get_ge_value` — return only the value (no key allocation) for floor/ceiling lookups.
- Precompiled native gems for Linux (x86_64, aarch64), macOS (x86_64, arm64), and Windows (x64) — installs without a Rust toolchain on those platforms.
- Tests covering binary string encoding, key lifetime after stream drop, builder GC, and 0xFF prefix-scan edge case.
- RuboCop lint configuration with rubocop-minitest and rubocop-rake plugins; runs as part of `rake` and as a CI gate.

### Changed
- Minimum Ruby version raised to 3.2.
- `MapBuilder` and `SetBuilder` now operate on a generic `Storage` backing (heap or mmap) for both `Map` and `Set`.
- Upgraded to magnus 0.8 (drops support for Ruby 2.7 and 3.0 in the underlying bindings; we already require 3.2+).

### Documentation
- README clarifies key encoding, insertion order, mmap vs in-memory loading, and Levenshtein UTF-8 requirement.

## [0.1.0] — 2026-05-13

### Added
- Initial release: `RubyFst::Map`, `RubyFst::Set`, `MapBuilder`, `SetBuilder`.
- Floor/ceiling lookups (`get_le`, `get_ge`).
- Levenshtein automaton search.
- Bytes/file serialization via `to_bytes` / `save` / `from_path`.
