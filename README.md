# friendly-filer

A TRON-styled FPS disc shooter that is also a real file manager.

Walk the current directory as a 3D arena. Files appear as red wireframe
enemies, subfolders as blue portals. Throw the identity disc, watch it
bounce and return, then pick `open` / `rename` / `move` / `copy` /
`delete` from the operation menu. `delete` drops into `.trash` and `u`
undoes. There is no Game Over — `crash` just respawns you at the folder
entrance with full HP.

> **Status**: Pre-alpha skeleton. The TRON palette, module layout and a
> one-frame demo are in place (see
> [#8](https://github.com/kako-jun/friendly-filer/issues/8) and
> [`docs/fps-spec.md`](./docs/fps-spec.md)). Movement, combat, real
> enemies, portals and menus land in sub-issues #9 – #18.

## Stack

- Rust 2024 (MSRV 1.85)
- [termray](https://crates.io/crates/termray) 0.3 — TUI raycasting,
  framebuffer, sprite & label projection
- [crossterm](https://crates.io/crates/crossterm) — input and terminal I/O

## Build

```bash
cargo build --release
./target/release/friendly-filer
```

At the current skeleton stage the binary paints one TRON-palette frame
(black background, blue converging floor grid, a gray-filled red enemy
placeholder in the middle, a blue `FRIENDLY-FILER v0.2.0-dev — TRON
MODE — #8` banner at the bottom) for about 0.8 seconds, then restores
the terminal and exits. Enough to prove the render pipeline and palette
are wired correctly before the real FPS layer lands.

## Roadmap (FPS layer)

| Issue | Scope |
|---|---|
| [#8](https://github.com/kako-jun/friendly-filer/issues/8) | TRON skeleton + FPS spec (this) |
| #9 | Enemy spawn & AI, LOD / Swarm, `notify` watch |
| #10 | Identity disc: throw, bounce, multi-hit, return |
| #11 | Portals, current-folder monolith, parent gate, sealed doors |
| #12 | Operation menu, bulk ops, per-op effect, `.trash`, Undo |
| #13 | HUD, minimap, crash / respawn animation |
| #14 | Search mode (`/`, fuzzy warp, freeze-time) |
| #15 | Input mode (rename / new file / new folder) |
| #16 | FPS OFF mode, preview, shell integration (`--cd-on-exit`) |
| #17 | Config (palette, keys, LOD thresholds) |
| #18 | Player movement (WASD / hjkl, jump, gravity, aim) |

The earlier Phase 1–4 issues (#3 – #6) are folded into the FPS layer.

## License

MIT. See [LICENSE](LICENSE).

## History

Previously known as `felipe` — a bevy-based cyberpunk file manager.
Rebooted on termray 0.3 in April 2026 as a TUI-native tool, then
reshaped again into the current TRON-styled FPS direction. The bevy
code lives in git history only.
