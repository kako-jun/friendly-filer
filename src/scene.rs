//! Directory → scene conversion (skeleton).
//!
//! Real `read_dir` → [`DirScene`] conversion (LOD, swarm aggregation, portal
//! seal checks, watch integration) lives in Issue #3 (rebooted under the FPS
//! layer). This module exists now only to own the struct shape and provide
//! a placeholder scene so `main.rs` has something to render for the #8
//! demo frame.

use crate::enemy::Enemy;
use crate::portal::{Monolith, ParentGate, Portal};

/// The renderable state of a single directory.
#[derive(Debug, Clone)]
pub struct DirScene {
    pub player_spawn: (f64, f64),
    pub enemies: Vec<Enemy>,
    pub portals: Vec<Portal>,
    pub monolith: Monolith,
    pub parent_gate: Option<ParentGate>,
}

impl DirScene {
    /// A hard-coded demo scene used by the #8 skeleton: the player spawns at
    /// the origin, one enemy stands in front of them, there are no portals,
    /// and the monolith sits behind the player. Swap this out for real
    /// `read_dir` work in Issue #3.
    pub fn placeholder() -> Self {
        let enemy = Enemy::from_metadata("demo.rs".to_string(), 2 * 1024, (0.0, 4.0));
        Self {
            player_spawn: (0.0, 0.0),
            enemies: vec![enemy],
            portals: Vec::new(),
            monolith: Monolith { x: 0.0, y: -3.0 },
            parent_gate: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_has_one_enemy_and_no_portals() {
        let s = DirScene::placeholder();
        assert_eq!(s.enemies.len(), 1);
        assert!(s.portals.is_empty());
        assert!(s.parent_gate.is_none());
    }
}
