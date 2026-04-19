# friendly-filer

## Overview

TUI 3D first-person file manager. Rust + crossterm. termray-powered raycasting — walk through your filesystem as if it were a city, with file names floating as labels above sprites.

(Formerly `felipe`, renamed on 2026-04-16. The bevy-based predecessor was deleted in Phase 0 / #2; it lives in git history only.)

## Architecture

Single-crate binary depending on the external [`termray`](https://github.com/kako-jun/termray) crate (0.3+).

- `src/main.rs` — entry point, input loop, frame presentation (crossterm half-block output)
- `src/lib.rs` — public surface. Empty at Phase 0, will grow with each phase
- `src/scene.rs` (Phase 1) — `DirScene`: `TileMap` + `HeightMap` + sprites + labels from a directory
- `src/nav.rs` (Phase 2) — camera + Wizardry/free navigation modes
- `src/ops.rs` (Phase 3) — file operations (open, delete, rename, copy, move)
- `src/atmosphere.rs` (Phase 4) — theme, fog, per-depth color shift

termray (external) owns raycasting: DDA walls, per-column ray-floor intersection, sprite and label projection. friendly-filer injects file-manager semantics (directory → scene, file → sprite, name → label) through `WallTexturer` / `FloorTexturer` / `SpriteArt` / `GlyphRenderer` trait impls.

## Operation model (planned)

- **Wizardry-style grid navigation** — turn in 90° steps, step cell-by-cell. Primary mode for comfortable browsing on narrow terminals.
- **Free navigation** — smooth movement for when you actually want to look around.
- **Label-first selection** — sprites are files; selecting a sprite invokes an OS-delegated action (open / reveal) via `open` or the platform equivalent.

## Atmosphere (planned)

Design notes inherited from the `felipe` era:

- Folder depth = terrain height. Big trees stand tall, empty subtrees are flat.
- Made in Abyss / Evangelion mood — fog, cool palettes, subdued saturation, a sense of descending into something deep.
- Theme is configurable per directory; per-depth color shift makes nesting visually obvious.

All of this lands in Phase 1 (scene build) and Phase 4 (polish).

## Build & run

```bash
cargo run --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

## Current phase

Phase 0 — skeleton. `cargo run` blanks the framebuffer to dark blue-grey for ~0.8 s then exits cleanly. The Cargo manifest, CI/release workflows, and documentation are in place; Phase 1+ will make it interactive.

## Roadmap

| Phase | Issue | Scope |
|---|---|---|
| 0 | #2 | Rebuild on termray 0.3 (this) |
| 1 | #3 | Filesystem → termray scene conversion |
| 2 | #4 | First-person navigation (Wizardry + free) |
| 3 | #5 | File operations (open / delete / rename / copy / move) |
| 4 | #6 | UX polish: atmosphere, themes, config |

## Historical notes

Before the 2026-04-16 rename, this repo was `felipe` — a bevy-based 3D cyberpunk file manager (`src/main.rs` at 654 lines, depending on bevy 0.14 with PBR + winit + x11). The binary name candidate "フィリップ" was a design-document persona for that era. The current design drops bevy entirely in favor of termray's TUI raycasting; the bevy code was removed in the commit that opened Phase 0.
