# Changelog

All notable changes to friendly-filer are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- FPS layer skeleton ([#8](https://github.com/kako-jun/friendly-filer/issues/8)).
  New simulation-layer modules land in `src/`: `palette` (TRON 3-color constants),
  `player`, `enemy` (with extension-based classification + `log(size)` HP + `Swarm`),
  `disc` (idle / flying / returning state), `portal` (`Portal`, `Monolith`, `ParentGate`,
  `is_dangerous_path`), `menu` (`Operation`, `MenuContext`), `hud` (`Mode`), `config`
  (`Config` + `AimStyle`), and `scene::DirScene` placeholder. 10 unit tests cover the
  shapes, all green.
- TRON-palette demo frame in `src/main.rs`: black background, converging blue floor
  grid + horizon, gray-filled red-outlined enemy placeholder, blue `Font8x8` banner.
  Displays for ~0.8 s and restores the terminal cleanly via a RAII `TerminalGuard`.
- `docs/fps-spec.md` — canonical design doc, copied from Issue #8 (visual language,
  game loop, LOD / Swarm, portals / monolith / parent gate, operation menu + effects,
  undo / `.trash`, search + input + FPS-OFF modes, shell integration, MVP acceptance).
- Project skeleton on top of termray 0.3 ([#2](https://github.com/kako-jun/friendly-filer/issues/2)).
  MIT `LICENSE`, `README.md`, `CLAUDE.md`, and `docs/architecture.md` describing the
  termray-based design and the initial Phase 0 → 4 roadmap.
- GitHub Actions workflows: `ci.yml` (fmt / clippy / test / release build on every push
  to `main` and every PR) and `release.yml` (five target binaries — linux x86_64 /
  aarch64, macOS x86_64 / aarch64, windows x86_64 — on `v*` tags, uploaded via
  `softprops/action-gh-release@v2`).

### Changed
- Reshaped the project into a **TRON-styled FPS disc shooter that is also a file
  manager** ([#8](https://github.com/kako-jun/friendly-filer/issues/8)). The earlier
  linear "Phase 1–4 3D filer" plan is absorbed into the FPS layer; new sub-issues
  #9 – #18 track movement, combat, portals, menus, HUD, search / input modes, FPS-OFF,
  config, and player movement.
- Rewrote `README.md`, `CLAUDE.md`, and `docs/architecture.md` around the FPS layer
  (visual discipline, module map, sub-issue graph). Removed the old Abyss / Evangelion
  atmosphere framing in favor of a single consistent TRON look.
- Rebuilt the repository from scratch on termray 0.3. The former bevy-based `felipe`
  implementation has been removed; see git history prior to #2 for reference.
- Crate renamed from `felipe` to `friendly-filer`, edition bumped to 2024, MSRV set to
  1.85.0. Dependencies reduced to `termray = "0.3"`, `crossterm = "0.29"`, `anyhow = "1"`.
