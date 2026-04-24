# Changelog

All notable changes to `mouseless` are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/).

## [Unreleased]

## [0.1.1] - 2026-04-24

### Added
- GitHub Actions CI workflow (`fmt` on Blacksmith ubuntu, `clippy` + `build` + `test` matrix on `macos-26` for aarch64/x86_64).
- GitHub Actions release workflow: on `v*` tag push, builds both mac targets, uploads tarballs + SHA256SUMS to GitHub Releases, runs `cargo publish` to crates.io.
- MIT `LICENSE` file and crates.io publish metadata in `Cargo.toml`.

### Changed
- Renamed crate and binary from `computerbase` to `mouseless`.
- README install instructions now lead with `cargo install mouseless`.

## [0.1.0] - 2026-04-24

### Added
- Initial crates.io publish as `mouseless`.
- 21 MCP tools for macOS desktop control: screenshot, zoom, click/drag/scroll, keyboard input, clipboard read/write, app launch, batch dispatch.
- Streamable HTTP and stdio transports.
- Dedicated enigo input thread, 3-tier coordinate type system, ease-out-cubic drag animation.

[Unreleased]: https://github.com/smithery-ai/mouseless/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/smithery-ai/mouseless/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/smithery-ai/mouseless/releases/tag/v0.1.0
