# friendly-filer

## Overview

TRON-styled FPS disc shooter that is also a real file manager. Rust +
crossterm, termray-powered raycasting. The current directory is the
arena, files are red wireframe enemies, subfolders are blue portals.
Throwing the identity disc identifies the target, opens the operation
menu, and the player picks `open` / `rename` / `move` / `copy` /
`delete` / `info`. `delete` goes to `.trash`, `u` undoes. No Game
Over — `crash` respawns at the folder entrance with full HP. FPS
semantics can be switched off (`F1`) so the program degrades to a
quiet 3D file browser.

(Formerly `felipe`, renamed 2026-04-16. The bevy predecessor was
deleted in Phase 0 / #2 and lives in git history only. The earlier
three-phase "3D filer" plan has been folded into the FPS layer — see
`docs/fps-spec.md` and Issue #8.)

## Architecture

Single-crate binary depending on the external
[`termray`](https://github.com/kako-jun/termray) crate (0.3+). The
simulation layer is pure Rust; rendering is outsourced to termray.

### Module map (`src/`)

- `main.rs` — entry point, terminal guard, current demo frame
- `lib.rs` — public re-exports of the modules below
- `palette.rs` — TRON 3-color constants (`BG_BLACK`, `GRID_BLUE`,
  `ENEMY_RED`, `GEOMETRY_GRAY`, `UI_BLUE`, `WARN_RED`)
- `player.rs` — `Player`, `new`, `is_crashed` (movement & jump in #18)
- `enemy.rs` — `EnemyKind`, `Enemy`, `Enemy::from_metadata`, `Swarm`
  (AI in #9)
- `disc.rs` — `DiscState`, `Disc`, `is_ready` (physics in #10)
- `portal.rs` — `Portal`, `Monolith`, `ParentGate`,
  `is_dangerous_path` (sealed doors in #11)
- `menu.rs` — `Operation`, `MenuContext` (effects & undo in #12)
- `hud.rs` — `Hud`, `Mode` (full HUD in #13)
- `config.rs` — `Config` + `Default`, `AimStyle` (TOML in #17)
- `scene.rs` — `DirScene` placeholder (real directory → scene in the
  absorbed #3 scope)

All modules ship with unit tests at the skeleton stage (10 tests green).

## Operation model (planned)

- **FPS controls** — WASD / hjkl move, space jumps (single, gravity),
  keyboard or mouse aim (config toggle, keyboard default).
- **Identity disc** — throw, bounce off walls, multi-hit, returns.
  Can't throw again until it comes back; damage accumulated while out =
  lock strength; confirm with Enter / right click. `delete`-style
  operations always prompt.
- **Targets**
  - File enemy → per-file op menu
  - Portal (subfolder) → enter = `cd`, shoot = folder-level op menu
  - Monolith (current folder) → shoot for new-file / new-folder / copy
    / move / delete / rename of the current dir
  - Parent gate (`..`) → can't be shot, only walked into
- **Search** — `/` freezes time, fuzzy search, warp to hit.
- **FPS OFF** — `F1` disables combat; enemies stand still, selection is
  cursor-only.

## Visual style (TRON)

3-color palette: bright-cyan blue for grid / walls / friendly UI /
identified files, bright red for enemies / warnings / lock-ons / HP,
gray for geometry faces, on a black background. High-intensity ANSI +
bold stands in for terminal glow. No bloom, no ligatures (consistent
with the neo-env rules in `MEMORY.md`).

Atmosphere work (fog, per-depth color shift, theming) is pulled into
the TRON discipline rather than the old "Made in Abyss / Evangelion"
mood — a single, consistent TRON look beats a mixed-metaphor one.

## Build & run

```bash
cargo run --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo test
```

`cargo run` currently paints one TRON frame (black + blue grid + red
enemy box + blue banner) for ~0.8 s and exits cleanly.

## Current stage

Skeleton of the FPS layer (#8). Module shapes, palette, one demo frame,
`docs/fps-spec.md`. Movement, enemies, disc, portals, menus, HUD,
search, input mode, FPS-OFF, config and real filesystem reads all land
in #9 – #18.

## Roadmap

| Issue | Scope |
|---|---|
| #8  | TRON skeleton + FPS spec (this) |
| #9  | Enemy spawn & AI, LOD / Swarm, `notify` watch |
| #10 | Identity disc: throw, bounce, multi-hit, return |
| #11 | Portals, monolith, parent gate, sealed doors |
| #12 | Operation menu, bulk ops, per-op effect, `.trash`, Undo |
| #13 | HUD, minimap, crash / respawn |
| #14 | Search mode (`/`, fuzzy warp) |
| #15 | Input mode (rename, new file, new folder) |
| #16 | FPS OFF, preview, shell integration (`--cd-on-exit`) |
| #17 | Config (palette, keys, LOD thresholds) |
| #18 | Player movement (WASD / hjkl, jump, gravity, aim) |

The earlier Phase 1–4 issues (#3 – #6) remain open for reference but
are superseded by #9 – #18.

## Historical notes

Before the 2026-04-16 rename, this repo was `felipe` — a bevy-based 3D
cyberpunk file manager (`src/main.rs` 654 lines, bevy 0.14 + PBR +
winit + x11). The 2026-04-16 reboot dropped bevy for termray's TUI
raycasting. The 2026-04 FPS reshape (#8) then folded the linear
three-phase "3D filer" roadmap into a TRON disc-shooter with reversible
filesystem semantics. All earlier code lives in git history only.
