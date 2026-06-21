# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-21

### Added

- Initial release.
- `gron` / `gron_with_root` — flatten a `serde_json::Value` into assignment lines.
- `ungron` — reconstruct a `Value` from gron lines (infers missing containers).
- `UngronError` for malformed lines, invalid JSON values, and path conflicts.
- `gron` CLI binary: flatten (stdin/file), `-u`/`--ungron`, `--root`.

[0.1.0]: https://github.com/trananhtung/gron-rs/releases/tag/v0.1.0
