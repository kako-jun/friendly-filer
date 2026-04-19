# friendly-filer

TUI 3D first-person file manager — walk through your filesystem as if it were a city.

> **Status**: Pre-alpha. Only the skeleton works right now (Phase 0 — see [#2](https://github.com/kako-jun/friendly-filer/issues/2)). Expect the repo to grow Phase by Phase into a proper 3D filer.

## Stack

- Rust 2024 (MSRV 1.85)
- [termray](https://crates.io/crates/termray) 0.3 — the TUI raycasting engine that powers rendering
- [crossterm](https://crates.io/crates/crossterm) — input and terminal I/O

## Build

```bash
cargo build --release
./target/release/friendly-filer
```

At Phase 0 the binary clears the terminal to a dark background for about 0.8 seconds, then exits — just enough to prove the render pipeline links up. Phase 1 will read the current directory and build a real scene.

## Roadmap

| Phase | Issue | Scope |
|---|---|---|
| 0 | [#2](https://github.com/kako-jun/friendly-filer/issues/2) | Rebuild on termray 0.3 (this) |
| 1 | #3 | Filesystem → termray scene conversion |
| 2 | #4 | First-person navigation (Wizardry + free) |
| 3 | #5 | File operations (open / delete / rename / copy / move) |
| 4 | #6 | UX polish: atmosphere, themes, config |

## License

MIT. See [LICENSE](LICENSE).

## History

Previously known as `felipe` — a bevy-based cyberpunk file manager. Rebooted on termray 0.3 in April 2026 to become a TUI-native tool. The old bevy code lives in git history only.
