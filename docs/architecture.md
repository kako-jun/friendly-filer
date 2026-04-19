# friendly-filer — Architecture

> Phase 0 document. Expect this file to grow as each phase lands.

## Three-layer model

friendly-filer is a thin orchestrator over the external `termray` crate. The runtime splits cleanly into three layers.

```
+-------------------------------------------------------------+
|  Input loop layer          (src/main.rs, later src/nav.rs)  |
|  - crossterm event read                                     |
|  - Wizardry / free navigation state                         |
|  - dispatch to ops / scene                                  |
+-------------------------------------------------------------+
|  Filesystem I/O layer      (src/scene.rs, src/ops.rs)       |
|  - read_dir / metadata / du-style aggregation               |
|  - sprite / label assembly                                  |
|  - OS-delegated open, rename, delete, copy, move            |
+-------------------------------------------------------------+
|  termray render layer      (external crate `termray` 0.3)   |
|  - DDA wall raycasting                                      |
|  - floor / ceiling per-column ray intersection              |
|  - sprite & label projection                                |
|  - Framebuffer → crossterm half-block presentation          |
+-------------------------------------------------------------+
```

The render layer is a dependency, not part of this crate. friendly-filer only *injects* file-manager semantics into termray through trait impls (`WallTexturer`, `FloorTexturer`, `SpriteArt`, `GlyphRenderer`).

## Phase dependency graph

```
Phase 0 (#2)  skeleton            ─┐
                                   ├─► Phase 1 (#3) scene build
Phase 1 (#3)  DirScene ───────────┬┘
                                  │
                                  ├─► Phase 2 (#4) navigation ──┐
                                  │                              │
                                  └─► Phase 3 (#5) operations ───┤
                                                                 ▼
                                                       Phase 4 (#6) polish
```

- **Phase 0** lays down the build, CI, release, and render pipeline plumbing. No filesystem reads yet.
- **Phase 1** introduces `DirScene` — a builder that turns a directory path into a `TileMap`, `HeightMap`, sprite list, and label set that termray can render.
- **Phase 2** adds the camera state machine (Wizardry 90° steps vs. free smooth movement) and the input event loop.
- **Phase 3** wires keybindings to file operations (open / delete / rename / copy / move), delegating to the OS where that is the right answer.
- **Phase 4** is the atmosphere pass: depth-aware color, fog, themes, configuration.

## Current state (Phase 0)

`src/main.rs` currently:

1. Reads terminal size via `crossterm::terminal::size`.
2. Allocates a `termray::Framebuffer` at full terminal width × 2×(rows−2) pixels (half-block rendering doubles vertical resolution).
3. Clears it to a dark blue-grey.
4. Enters the alternate screen, hides the cursor, enables raw mode.
5. Writes one frame using half-block characters (`▀`): top pixel → foreground RGB, bottom pixel → background RGB.
6. Sleeps ~800 ms so the user can see the blank frame.
7. Restores the terminal and exits.

There is no scene, no input handling, no filesystem interaction yet — those arrive with the phases above.

## External dependencies

| Crate | Version | Role |
|---|---|---|
| [`termray`](https://crates.io/crates/termray) | 0.3 | Raycasting, framebuffer, sprite/label projection |
| [`crossterm`](https://crates.io/crates/crossterm) | 0.29 | Cross-platform terminal I/O and input events |
| [`anyhow`](https://crates.io/crates/anyhow) | 1 | Error propagation in the binary |

The dependency list is deliberately minimal. File-system work in later phases will use `std::fs` wherever possible; `open` (or a platform-specific equivalent) will be added when Phase 3 needs OS delegation.
