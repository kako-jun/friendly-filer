# Changelog

All notable changes to friendly-filer are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project skeleton on top of termray 0.3 ([#2](https://github.com/kako-jun/friendly-filer/issues/2)).
  `cargo run` blanks the framebuffer for a brief moment and exits cleanly — Phase 1+ will
  turn this into an interactive file navigator.
- MIT `LICENSE`, `README.md`, `CLAUDE.md`, and `docs/architecture.md` describing the new
  termray-based design and the Phase 0 → 4 roadmap.
- GitHub Actions workflows: `ci.yml` (fmt / clippy / test / release build on every push to
  `main` and every PR) and `release.yml` (five target binaries — linux x86_64 / aarch64,
  macOS x86_64 / aarch64, windows x86_64 — on `v*` tags, uploaded via
  `softprops/action-gh-release@v2`).

### Changed
- Rebuilt the repository from scratch on termray 0.3. The former bevy-based `felipe`
  implementation has been removed; see git history prior to #2 for reference.
- Crate renamed from `felipe` to `friendly-filer`, edition bumped to 2024, MSRV set to
  1.85.0. Dependencies reduced to `termray = "0.3"`, `crossterm = "0.29"`, `anyhow = "1"`.
