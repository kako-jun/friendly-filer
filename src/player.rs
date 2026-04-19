//! Player state.
//!
//! This module only owns the **state** of the first-person camera: position,
//! orientation, vertical velocity (for jumps), and HP. Movement integration,
//! gravity, jump physics and input handling arrive in Issue #18. Weapon /
//! disc state lives in [`crate::disc`].

/// Default starting hit points. Balanced against the enemy HP scale
/// (`log2(size_kb).ceil()`, capped at 5) so that a few unlucky lunges can
/// down the player, but the steady state is survivable.
pub const DEFAULT_MAX_HP: u32 = 10;

/// First-person camera + combat stats.
///
/// `x`, `y` are the floor-plane coordinates (termray world space).
/// `z` is the eye height above the floor; `pitch` tilts the camera up / down.
/// `vz` is vertical velocity used by the jump integrator in #18.
/// `yaw` is the facing direction in radians.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub vz: f64,
    pub hp: u32,
    pub max_hp: u32,
}

impl Player {
    /// Spawn a fresh player at `(x, y)` on the floor with full HP.
    pub fn new(x: f64, y: f64, yaw: f64) -> Self {
        Self {
            x,
            y,
            z: 0.0,
            yaw,
            pitch: 0.0,
            vz: 0.0,
            hp: DEFAULT_MAX_HP,
            max_hp: DEFAULT_MAX_HP,
        }
    }

    /// Whether the player has hit 0 HP and should be in the "crash" state.
    /// The HUD glitch, respawn, and HP restore are handled by the caller
    /// (Issue #13); this is just the predicate.
    pub fn is_crashed(&self) -> bool {
        self.hp == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_spawn_pose_and_full_hp() {
        let p = Player::new(3.5, -2.0, std::f64::consts::FRAC_PI_2);
        assert_eq!(p.x, 3.5);
        assert_eq!(p.y, -2.0);
        assert_eq!(p.z, 0.0);
        assert_eq!(p.yaw, std::f64::consts::FRAC_PI_2);
        assert_eq!(p.pitch, 0.0);
        assert_eq!(p.vz, 0.0);
        assert_eq!(p.hp, DEFAULT_MAX_HP);
        assert_eq!(p.max_hp, DEFAULT_MAX_HP);
        assert!(!p.is_crashed());
    }

    #[test]
    fn is_crashed_only_when_hp_zero() {
        let mut p = Player::new(0.0, 0.0, 0.0);
        assert!(!p.is_crashed());
        p.hp = 1;
        assert!(!p.is_crashed());
        p.hp = 0;
        assert!(p.is_crashed());
    }
}
