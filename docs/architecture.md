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
| `menu.rs` | skeleton (#8) | `Operation { Open, Rename, Move, Copy, Delete, Info, Cancel }` and `MenuContext { File, Swarm, Folder, Monolith }`. Real menu + effects land in #12. |
| `hud.rs` | skeleton (#8) | `Hud { hp, score, .. }`, `Mode { Fps, FpsOff, Frozen, Crashed }`. Full HUD lands in #13. |
| `config.rs` | skeleton (#8) | `Config` with `Default`, `AimStyle { Keyboard, Mouse }`. Real TOML loader lands in #17. |
| `scene.rs` | extended (#18) | `DirScene` now owns an 8×8 `GridMap` + `FlatHeightMap` + `spawn_yaw`, with `map()` / `heights()` / `camera()` accessors that feed termray's raycaster. Real `read_dir` → scene work still lands in #3. |
| `input.rs` | done (#18) | Crossterm `event::poll` → `FrameInput { forward, strafe, run, jump, yaw_delta, pitch_delta, toggle_fps_off, quit }`. Press / Repeat are both treated as one frame of input; Release is ignored. Mouse aim is deferred to #16. |
| `physics.rs` | done (#18) | Pure functions on `Player`: `step_movement` (axis-aligned Doom slide), `step_gravity`, `try_jump`, `add_yaw`, `add_pitch`. Tunables live as module `pub const`s; TOML config lands in #17. |
| `render.rs` | done (#18) | Framebuffer → stdout `present()` (half-block), plus `WallTextureFlat` and `FloorTextureGrid` TRON adapters for termray's `WallTexturer` / `FloorTexturer` traits. |
| `main.rs` | FPS loop (#18) | 60 FPS frame loop: input → physics → camera sync → `cast_all_rays` → `render_floor_ceiling` + `render_walls` → debug HUD → `present`. Esc / q quit; F1 toggles FPS-OFF mode. The #8 static demo frame has been removed. |

## Current state (#18 playable walk-around)

What `cargo run` does today:

1. Reads terminal size via `crossterm::terminal::size` and allocates a
   half-block-doubled `termray::Framebuffer`.
2. Builds an 8×8 `DirScene::placeholder()` — solid outer ring, 6×6
   walkable interior, one demo enemy, one central monolith.
3. Enters the alternate screen + raw mode via `TerminalGuard`.
4. Runs a 60 FPS frame loop driving the **real** termray raycaster:
   - `input::poll_frame_input` drains crossterm events into a
     `FrameInput`;
   - `physics::step_movement` / `try_jump` / `step_gravity` /
     `add_yaw` / `add_pitch` update the `Player`;
   - `Camera::set_pose` / `set_z` / `set_pitch` sync the camera;
   - `render_floor_ceiling` + `render_walls` rasterize the frame into
     the framebuffer using `FloorTextureGrid` + `WallTextureFlat`;
   - a single `Font8x8` debug line prints pos / yaw / pitch / vz /
     HP / MODE at the bottom of the screen;
   - `render::present` flushes the framebuffer with half-block
     characters, then the loop sleeps to hit the 16 ms frame budget.
5. Esc or `q` terminates; `TerminalGuard::drop` restores the terminal.

The #8 static demo frame (hand-drawn floor fan, red enemy rectangle,
bottom-centre banner) has been removed. The draw_floor_grid /
draw_enemy_placeholder / draw_banner helpers are gone, replaced by the
termray pipeline above.

### Physics tunables (`src/physics.rs`)

| Constant | Value | Meaning |
|---|---|---|
| `MOVE_SPEED` | `3.0` | World units / second, walking. |
| `RUN_MULTIPLIER` | `1.8` | Held-Shift sprint multiplier. |
| `JUMP_INITIAL_VZ` | `4.5` | Upward velocity on jump. |
| `GRAVITY` | `12.0` | Downward acceleration. |
| `GROUND_Z` | `0.5` | Eye height on the floor (matches termray default). |
| `AIM_YAW_RATE` | `2.0` | rad / s for arrow-key yaw. |
| `AIM_PITCH_RATE` | `1.5` | rad / s for arrow-key pitch. |
| `PITCH_MAX` | `π/2 − 0.05` | Pitch clamp (avoids tan() singularity). |
| `PLAYER_RADIUS` | `0.25` | Collision circle radius. |

All of these become TOML-overridable in #17.

Enemy AI, disc physics and sealed-portal geometry still arrive with
issues #9 – #17. The simulation-layer unit tests plus the new physics
and render tests total 24 green.

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
