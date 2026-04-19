# friendly-filer — Architecture

> This document describes the **FPS layer** design introduced by
> [#8](https://github.com/kako-jun/friendly-filer/issues/8). The older
> three-phase "3D filer" roadmap from #2 is folded into the FPS layer and
> is no longer pursued as a standalone track. See
> [`docs/fps-spec.md`](./fps-spec.md) for the canonical design spec.

## Concept

friendly-filer is a **TRON-styled FPS that happens to be a real file
manager**. The current directory is the arena, files are wireframe
enemies, subfolders are portals, and the player throws an identity disc
to "identify" a target — at which point an operation menu opens and the
usual file operations (open / rename / move / copy / delete / info) are
available. `delete` moves to `.trash` and is reversible via `u`. Game
Over does not exist.

FPS semantics can be switched off (`F1`) so the program degrades to a
quiet 3D file browser for days when that is all that is wanted.

## Layered model

```
+-------------------------------------------------------------+
|  Game loop & input         (src/main.rs, later src/app.rs)  |
|  - frame pacing, crossterm event read                       |
|  - Player state, Disc state, Menu state                     |
|  - dispatches file operations once a target is confirmed    |
+-------------------------------------------------------------+
|  Simulation layer          (player, enemy, disc, portal,    |
|                             hud, menu, scene, config)       |
|  - Pure Rust data: no rendering, minimal IO                 |
|  - Unit-testable (10 tests green at the skeleton stage)     |
+-------------------------------------------------------------+
|  Filesystem I/O layer      (scene::DirScene, future ops.rs) |
|  - read_dir / metadata / du-style aggregation               |
|  - sprite / label assembly, trash + undo journal            |
|  - OS-delegated open, rename, delete, copy, move            |
+-------------------------------------------------------------+
|  termray render layer      (external crate `termray` 0.3)   |
|  - DDA wall raycasting                                      |
|  - floor / ceiling per-column ray intersection              |
|  - sprite & label projection, 8×8 glyph font                |
|  - Framebuffer → crossterm half-block presentation          |
+-------------------------------------------------------------+
```

termray is a dependency, not part of this crate. friendly-filer injects
file-manager and FPS semantics into termray through trait impls
(`WallTexturer`, `FloorTexturer`, `SpriteArt`, `GlyphRenderer`) and by
feeding it the scene data produced by `DirScene`.

## Module map (`src/`)

| Module | Status | Responsibility |
|---|---|---|
| `palette.rs` | skeleton (#8) | TRON 3-color constants: `BG_BLACK`, `GRID_BLUE`, `ENEMY_RED`, `GEOMETRY_GRAY`, `UI_BLUE`, `WARN_RED`. Single source of truth for all drawing. |
| `player.rs` | skeleton (#8) | `Player { pos, yaw, hp, .. }`, `new`, `is_crashed`. Movement / jump land in #18. |
| `enemy.rs` | skeleton (#8) | `EnemyKind`, `Enemy`, `Enemy::from_metadata` (extension-based classification, `log(size)` HP), `Swarm` for LOD aggregation. AI lands in #9. |
| `disc.rs` | skeleton (#8) | `DiscState { Idle, Flying, Returning }`, `Disc`, `is_ready`. Physics + multi-hit land in #10. |
| `portal.rs` | skeleton (#8) | `Portal` (subfolder), `Monolith` (current folder ops), `ParentGate` (`..`), `is_dangerous_path` guard. Sealed-door logic lands in #11. |
| `menu.rs` | skeleton (#8) | `Operation { Open, Rename, Move, Copy, Delete, Info, Cancel }` and `MenuContext { Single, Bulk, Folder }`. Real menu + effects land in #12. |
| `hud.rs` | skeleton (#8) | `Hud { hp, score, .. }`, `Mode { Fps, Calm, Search, Input }`. Full HUD lands in #13. |
| `config.rs` | skeleton (#8) | `Config` with `Default`, `AimStyle { Keyboard, Mouse }`. Real TOML loader lands in #17. |
| `scene.rs` | skeleton (#8) | `DirScene` placeholder returning a single dummy enemy. Real directory → scene conversion lands in #3 (absorbed into the FPS layer). |
| `main.rs` | demo (#8) | Renders one TRON-palette frame (black bg, blue grid, red enemy placeholder, blue banner) for ~0.8 s and exits. |

## Current state (#8 skeleton)

What `cargo run` does today:

1. Reads terminal size via `crossterm::terminal::size`.
2. Allocates a `termray::Framebuffer` sized to the terminal (half-block
   vertical doubling applied).
3. Paints a single TRON-palette frame:
   - black background,
   - converging blue floor grid with a blue horizon line,
   - a gray-filled, red-outlined rectangle in the middle as an enemy
     placeholder,
   - a blue `Font8x8` banner near the bottom.
4. Enters the alternate screen, hides the cursor, enables raw mode via
   a RAII `TerminalGuard`.
5. Sleeps 800 ms so the frame is visible.
6. Drops the guard, restoring the terminal, and exits cleanly.

No input loop, no filesystem reads, no real enemies, no disc, no menu —
those arrive with issues #9–#18. All 10 simulation-layer unit tests are
green.

## Sub-issue graph (FPS layer)

```
#8 FPS parent (skeleton + spec)
 ├─► #9  enemy spawn & AI (needs #3 scene, LOD, watch)
 ├─► #10 identity disc (needs #9 for targets)
 ├─► #11 portals + monolith + parent gate + sealed doors
 ├─► #12 operation menu + bulk + per-op animation + undo + .trash
 ├─► #13 HUD + crash + respawn + minimap
 ├─► #14 search mode (/, fuzzy warp, freeze-time)
 ├─► #15 input mode (rename, new file/folder, freeze-time)
 ├─► #16 FPS OFF + preview panel + shell integration (--cd-on-exit)
 ├─► #17 config (TOML, palette, keys, LOD thresholds)
 └─► #18 player movement (WASD / hjkl, jump, gravity, aim)
```

The old Phase 1–4 issues (#3 scene build, #4 navigation, #5 operations,
#6 polish) are **absorbed** into this graph. They remain open for
context but new work happens on #9–#18.

## External dependencies

| Crate | Version | Role |
|---|---|---|
| [`termray`](https://crates.io/crates/termray) | 0.3 | Raycasting, framebuffer, sprite & label projection, `Font8x8` |
| [`crossterm`](https://crates.io/crates/crossterm) | 0.29 | Cross-platform terminal I/O and input events |
| [`anyhow`](https://crates.io/crates/anyhow) | 1 | Error propagation in the binary |

The dependency list is deliberately minimal. Filesystem work will lean
on `std::fs`; `open`-style OS delegation and `notify` (external change
watch) will be added when their issues land.
