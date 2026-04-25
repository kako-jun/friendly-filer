//! Directory → scene conversion (skeleton).
//!
//! Real `read_dir` → [`DirScene`] conversion (LOD, swarm aggregation, portal
//! seal checks, watch integration) lives in Issue #3 (rebooted under the FPS
//! layer). This module currently ships a **placeholder** `DirScene` that
//! backs the #18 playable demo: a small closed room with a single enemy and
//! a central monolith, walkable on the termray raycaster.

use termray::{Camera, FlatHeightMap, GridMap, TILE_EMPTY};

use crate::enemy::Enemy;
use crate::portal::{Monolith, ParentGate, Portal};

/// Edge length (in cells) of the placeholder arena. Small enough that the
/// full room fits inside a 80×24 terminal's camera frustum at FOV 80°, big
/// enough that WASD motion is clearly visible.
const PLACEHOLDER_MAP_SIZE: usize = 8;

/// Default horizontal field of view in radians. 80° feels natural in the
/// terminal half-block half-resolution; bumping higher amplifies the
/// fish-eye distortion at the edges.
const DEFAULT_FOV_RAD: f64 = 80.0 * std::f64::consts::PI / 180.0;

/// The renderable state of a single directory.
///
/// Holds both the simulation-layer data (enemies, portals, monolith) and
/// the termray-side world geometry (tile map + flat height map) that the
/// raycaster walks. Real `read_dir`-driven construction lands in #3.
pub struct DirScene {
    /// Tile grid consumed by [`termray::cast_ray`] for wall hits and by
    /// [`crate::physics::step_movement`] for collision.
    tile_map: GridMap,
    /// Per-corner floor / ceiling heights. The placeholder uses a flat
    /// world (floor=0, ceil=1); #11 will introduce portal steps.
    height_map: FlatHeightMap,
    /// Player spawn point in world coordinates (cell-center offsets).
    pub player_spawn: (f64, f64),
    /// Yaw the camera faces at spawn, in radians. `0.0` looks +x.
    pub spawn_yaw: f64,
    // The following fields are populated by `placeholder()` but not yet
    // consumed by the render / combat path. They are `pub` so the frame
    // loop can iterate them once the features land:
    //   - `enemies`  — wired up in #9 (enemy rendering + AI)
    //   - `portals`  — wired up in #11 (portal geometry + traversal)
    //   - `monolith` / `parent_gate` — HUD overlays and navigation in #13
    pub enemies: Vec<Enemy>,
    pub portals: Vec<Portal>,
    pub monolith: Monolith,
    pub parent_gate: Option<ParentGate>,
}

impl DirScene {
    /// An 8×8 closed room used by the #18 playable demo. The outer ring is
    /// [`termray::TILE_WALL`], the 6×6 interior is [`TILE_EMPTY`] and the
    /// player spawns one cell in from the NW corner looking east. A single
    /// demo enemy stands near the far wall; the monolith sits in the middle
    /// of the room. Swap this out for real `read_dir` work in Issue #3.
    pub fn placeholder() -> Self {
        let size = PLACEHOLDER_MAP_SIZE;
        let mut tile_map = GridMap::new(size, size);
        // Carve a 6×6 open interior inside the all-wall default grid.
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                tile_map.set(x, y, TILE_EMPTY);
            }
        }

        // One demo enemy near the far (east) wall so it's visible from
        // spawn without being pressed against the player's face.
        let enemy = Enemy::from_metadata("placeholder.txt".to_string(), 8 * 1024, 4.0, 6.0);

        Self {
            tile_map,
            height_map: FlatHeightMap,
            player_spawn: (1.5, 1.5),
            spawn_yaw: 0.0,
            enemies: vec![enemy],
            portals: Vec::new(),
            monolith: Monolith { x: 4.0, y: 4.0 },
            parent_gate: None,
        }
    }

    /// The termray tile grid (walls + empty cells). Movement collision and
    /// wall raycasting both consult this same map so the player and the
    /// rays agree on what counts as a wall.
    pub fn map(&self) -> &GridMap {
        &self.tile_map
    }

    /// The termray height map. Flat in the placeholder; per-cell steps will
    /// arrive with portal geometry in #11.
    pub fn heights(&self) -> &FlatHeightMap {
        &self.height_map
    }

    /// A fresh [`Camera`] positioned at the spawn pose with the default FOV.
    /// The caller owns the camera and updates it from the player's pose each
    /// frame via [`Camera::set_pose`] / `set_z` / `set_pitch`.
    pub fn camera(&self) -> Camera {
        Camera::new(
            self.player_spawn.0,
            self.player_spawn.1,
            self.spawn_yaw,
            DEFAULT_FOV_RAD,
        )
    }

    /// Load a scene from a real directory. Files become enemies with deterministic
    /// positions based on a hash of the filename. Directories are ignored for now.
    /// Up to 20 files are loaded; extras are discarded (LOD comes later).
    pub fn from_dir(path: &std::path::Path) -> std::io::Result<Self> {
        let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().ok().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        let size = PLACEHOLDER_MAP_SIZE;
        let mut tile_map = GridMap::new(size, size);
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                tile_map.set(x, y, TILE_EMPTY);
            }
        }

        let mut enemies = Vec::new();
        for (i, entry) in entries.iter().take(20).enumerate() {
            if let Ok(metadata) = entry.metadata() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_size = metadata.len();

                // Deterministic position from file index: spread across interior 6x6.
                let x = 1.5 + (i % 6) as f64 + 0.5;
                let y = 1.5 + (i / 6) as f64 + 0.5;
                let x = x.min(6.5);
                let y = y.min(6.5);

                enemies.push(Enemy::from_metadata(file_name, file_size, x, y));
            }
        }

        Ok(Self {
            tile_map,
            height_map: FlatHeightMap,
            player_spawn: (1.5, 1.5),
            spawn_yaw: 0.0,
            enemies,
            portals: Vec::new(),
            monolith: Monolith { x: 4.0, y: 4.0 },
            parent_gate: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termray::TileMap;

    #[test]
    fn placeholder_has_one_enemy_and_no_portals() {
        let s = DirScene::placeholder();
        assert_eq!(s.enemies.len(), 1);
        assert!(s.portals.is_empty());
        assert!(s.parent_gate.is_none());
    }

    #[test]
    fn placeholder_outer_ring_is_solid_interior_is_walkable() {
        let s = DirScene::placeholder();
        let m = s.map();
        // NW corner is wall.
        assert!(m.is_solid(0, 0));
        // A middle interior cell is walkable.
        assert!(!m.is_solid(3, 3));
        // East outer wall.
        assert!(m.is_solid(PLACEHOLDER_MAP_SIZE as i32 - 1, 3));
    }

    #[test]
    fn camera_is_at_spawn_with_default_fov() {
        let s = DirScene::placeholder();
        let cam = s.camera();
        assert!((cam.x - s.player_spawn.0).abs() < 1e-12);
        assert!((cam.y - s.player_spawn.1).abs() < 1e-12);
        assert!((cam.angle - s.spawn_yaw).abs() < 1e-12);
        assert!((cam.fov - DEFAULT_FOV_RAD).abs() < 1e-12);
    }
}
