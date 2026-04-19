//! Identity disc — the player's weapon.
//!
//! The disc bounces off walls, multi-hits enemies, and *returns* to the
//! player before the next throw is permitted. That return-flight is what
//! gates combat pacing: you can't spam throws. All of the physics (launch,
//! bounce, homing-return, collision registration) lands in Issue #10 — this
//! module only defines the type.

/// Life-cycle of the disc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscState {
    /// Held by the player, ready to throw.
    Idle,
    /// In flight toward / past enemies. Bouncing.
    Flying,
    /// Boomerang-ing back to the player. Not yet catchable.
    Returning,
}

/// Position, velocity and hit list of the current disc throw.
///
/// `hit_ids` records indices into the enemy list for every enemy the disc
/// has passed through on this throw — it's the selection set that the
/// operation menu consumes when the disc finally returns.
#[derive(Debug, Clone)]
pub struct Disc {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub state: DiscState,
    pub hit_ids: Vec<usize>,
}

impl Disc {
    /// A fresh disc in the player's hand, ready to throw.
    pub fn new_idle() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            state: DiscState::Idle,
            hit_ids: Vec::new(),
        }
    }

    /// `true` if the player is allowed to throw right now.
    pub fn is_ready(&self) -> bool {
        matches!(self.state, DiscState::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_disc_is_ready() {
        let d = Disc::new_idle();
        assert_eq!(d.state, DiscState::Idle);
        assert!(d.is_ready());
        assert!(d.hit_ids.is_empty());
    }

    #[test]
    fn flying_or_returning_disc_is_not_ready() {
        let mut d = Disc::new_idle();
        d.state = DiscState::Flying;
        assert!(!d.is_ready());
        d.state = DiscState::Returning;
        assert!(!d.is_ready());
    }
}
