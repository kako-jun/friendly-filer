//! Player physics: movement + gravity + jump + camera aim.
//!
//! Pure functions operating on [`crate::player::Player`] state. No rendering,
//! no input: the frame loop reads the current input vector, calls
//! [`step_movement`] / [`step_gravity`] / [`try_jump`] / [`add_yaw`] /
//! [`add_pitch`], and pushes the resulting pose into the termray camera.
//!
//! Every tunable is a module-level `pub const`. Config-file overrides land
//! with Issue #17; until then tweaking a constant here is the intended
//! workflow.

use std::f64::consts::{FRAC_PI_2, TAU};

use termray::TileMap;

use crate::player::Player;

/// Baseline walking speed, in world units per second. Combined with
/// [`RUN_MULTIPLIER`] when the player holds Shift.
pub const MOVE_SPEED: f64 = 3.0;

/// Multiplier applied to [`MOVE_SPEED`] while the run modifier (Shift) is
/// held. Tuned so sprinting feels distinct from walking but does not skip
/// past collision checks in a single frame at 60 FPS (3.0 × 1.8 / 60 ≈
/// 0.09 units / frame, much less than [`PLAYER_RADIUS`]).
pub const RUN_MULTIPLIER: f64 = 1.8;

/// Initial upward velocity applied when the player triggers a jump, in
/// world units per second. Combined with [`GRAVITY`] this yields a peak
/// height of `JUMP_INITIAL_VZ² / (2·GRAVITY)` ≈ 0.84 units — enough to
/// clear a low portal step (see #11) without leaving the room's 1-unit
/// ceiling.
pub const JUMP_INITIAL_VZ: f64 = 4.5;

/// Downward gravitational acceleration, world units / second². Larger
/// value = snappier, arcade-ier jumps. The current tuning gives a total
/// airtime of `2·JUMP_INITIAL_VZ / GRAVITY` ≈ 0.75 s.
pub const GRAVITY: f64 = 12.0;

/// Eye height when standing on the floor. Matches
/// [`termray::Camera::new`]'s default `z = 0.5`, so the camera sits in
/// the vertical middle of a 1-unit-tall cell.
pub const GROUND_Z: f64 = 0.5;

/// Yaw rotation speed from the arrow keys, radians / second. Roughly
/// equivalent to one full 360° turn per 3 seconds when the key is held.
pub const AIM_YAW_RATE: f64 = 2.0;

/// Pitch rotation speed from the arrow keys, radians / second.
pub const AIM_PITCH_RATE: f64 = 1.5;

/// Upper bound (in radians) on `|pitch|`. termray's pitch uses a
/// `tan(pitch)` horizon shift which diverges at ±π/2; a 0.05-rad margin
/// keeps the math well-conditioned without visibly clipping extreme looks.
pub const PITCH_MAX: f64 = FRAC_PI_2 - 0.05;

/// Collision radius used by [`step_movement`]. The player is modelled as
/// a circle of this radius on the floor plane; movement is rejected if
/// the circle would overlap a solid cell. 0.25 leaves a visible gap
/// between the camera and the walls without feeling spongy.
pub const PLAYER_RADIUS: f64 = 0.25;

/// Advance the player's horizontal position along the camera's facing
/// direction using an axis-aligned Doom-style slide.
///
/// - `forward` is the signed scalar along the view direction
///   (+1 = W, −1 = S, after scaling by [`MOVE_SPEED`] × `dt` × optional
///   [`RUN_MULTIPLIER`] by the caller).
/// - `strafe` is the same convention but along the right-hand strafe
///   direction (+1 = D, −1 = A).
///
/// Each axis (x, y) is tested independently: if moving in x alone would
/// collide with a wall (as decided by [`TileMap::is_solid`] considering
/// [`PLAYER_RADIUS`]), the x component is dropped but y may still
/// succeed — giving the classic "slide along the wall" feel without
/// requiring a full circle / rectangle sweep.
pub fn step_movement(player: &mut Player, forward: f64, strafe: f64, dt: f64, map: &dyn TileMap) {
    let (sin, cos) = player.yaw.sin_cos();
    // forward: (cos, sin), right: (-sin, cos). Same convention as
    // termray::Camera::{forward, right}.
    let dx = (cos * forward + -sin * strafe) * dt;
    let dy = (sin * forward + cos * strafe) * dt;

    if !blocked_at(map, player.x + dx, player.y) {
        player.x += dx;
    }
    if !blocked_at(map, player.x, player.y + dy) {
        player.y += dy;
    }
}

/// Test whether a circle of [`PLAYER_RADIUS`] centred at `(x, y)` would
/// overlap any solid tile in `map`. Samples the four cardinal edges of
/// the bounding box; that's enough for axis-aligned grid walls and
/// avoids the cost of a full per-corner sweep.
fn blocked_at(map: &dyn TileMap, x: f64, y: f64) -> bool {
    let r = PLAYER_RADIUS;
    for (tx, ty) in [
        (x - r, y),
        (x + r, y),
        (x, y - r),
        (x, y + r),
        // Diagonal corners as well, so the player can't squeeze through
        // a 45° gap between two walls.
        (x - r, y - r),
        (x + r, y - r),
        (x - r, y + r),
        (x + r, y + r),
    ] {
        let cx = tx.floor() as i32;
        let cy = ty.floor() as i32;
        if map.is_solid(cx, cy) {
            return true;
        }
    }
    false
}

/// Integrate gravity over `dt`, clamping the player onto the floor at
/// [`GROUND_Z`]. Leaves `player.z` unchanged when the player is grounded
/// and not moving vertically.
pub fn step_gravity(player: &mut Player, dt: f64) {
    if player.z <= GROUND_Z && player.vz == 0.0 {
        // Already at rest on the floor. Skip the integration so
        // floating-point drift doesn't lift the player off the ground.
        return;
    }
    player.vz -= GRAVITY * dt;
    player.z += player.vz * dt;
    if player.z <= GROUND_Z {
        player.z = GROUND_Z;
        player.vz = 0.0;
    }
}

/// Apply a jump impulse if the player is standing on the floor. Returns
/// `true` if a jump was actually issued (so the HUD / sfx can react), or
/// `false` if the request was ignored (already airborne).
pub fn try_jump(player: &mut Player) -> bool {
    if player.z == GROUND_Z && player.vz == 0.0 {
        player.vz = JUMP_INITIAL_VZ;
        true
    } else {
        false
    }
}

/// Add `delta` radians to the player's yaw, wrapping into `[0, 2π)`.
/// Using the full positive range matches
/// [`termray::math::normalize_angle`]'s convention.
pub fn add_yaw(player: &mut Player, delta: f64) {
    player.yaw = (player.yaw + delta).rem_euclid(TAU);
}

/// Add `delta` radians to the player's pitch, clamping to
/// `±`[`PITCH_MAX`] so the `tan(pitch)` horizon shift never diverges.
pub fn add_pitch(player: &mut Player, delta: f64) {
    player.pitch = (player.pitch + delta).clamp(-PITCH_MAX, PITCH_MAX);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fully-walkable placeholder map: 4×4 with every cell [`TILE_EMPTY`].
    struct OpenMap;
    impl TileMap for OpenMap {
        fn width(&self) -> usize {
            4
        }
        fn height(&self) -> usize {
            4
        }
        fn get(&self, _x: i32, _y: i32) -> Option<termray::TileType> {
            Some(termray::TILE_EMPTY)
        }
        fn is_solid(&self, _x: i32, _y: i32) -> bool {
            false
        }
    }

    /// Grid with a single solid pillar at `(2, 2)`.
    struct PillarMap;
    impl TileMap for PillarMap {
        fn width(&self) -> usize {
            4
        }
        fn height(&self) -> usize {
            4
        }
        fn get(&self, x: i32, y: i32) -> Option<termray::TileType> {
            if (x, y) == (2, 2) {
                Some(termray::TILE_WALL)
            } else {
                Some(termray::TILE_EMPTY)
            }
        }
        fn is_solid(&self, x: i32, y: i32) -> bool {
            (x, y) == (2, 2)
        }
    }

    fn spawn() -> Player {
        let mut p = Player::new(1.5, 1.5, 0.0);
        p.z = GROUND_Z;
        p
    }

    #[test]
    fn step_movement_advances_on_open_map() {
        let mut p = spawn();
        step_movement(&mut p, 1.0, 0.0, 0.1, &OpenMap);
        // forward = +x at yaw 0; so x should have grown, y unchanged.
        assert!(p.x > 1.5);
        assert!((p.y - 1.5).abs() < 1e-12);
    }

    #[test]
    fn step_movement_blocked_by_wall_leaves_x_unchanged() {
        // Stand just west of the pillar at (2, 2) and try to walk into it.
        let mut p = Player::new(1.8, 2.5, 0.0);
        p.z = GROUND_Z;
        let before = (p.x, p.y);
        step_movement(&mut p, 5.0, 0.0, 0.1, &PillarMap);
        // Pushing straight east into the solid cell at (2,2) must not
        // move us into it. The axis-aligned slide rejects the x step
        // and the player stays at the starting x.
        assert!((p.x - before.0).abs() < 1e-12);
        // y was untouched since we didn't supply a strafe.
        assert!((p.y - before.1).abs() < 1e-12);
    }

    #[test]
    fn step_movement_slides_along_wall_when_blocked_on_one_axis() {
        // Stand west of the pillar at (2, 2). Push north-east: x is blocked,
        // y should still move (slide north).
        let mut p = Player::new(1.8, 2.5, 0.0);
        p.z = GROUND_Z;
        // yaw = 0 -> forward is +x. strafe = -1 at yaw 0 means -right = +y
        // reversed... simpler: set yaw so forward = +y (north). Push forward
        // + strafe where strafe enters the pillar on x, but forward slides y.
        p.yaw = std::f64::consts::FRAC_PI_2; // forward = +y
        let y0 = p.y;
        step_movement(&mut p, 1.0, 1.0, 0.1, &PillarMap);
        // y must have moved (forward), x may be blocked by the pillar.
        assert!(
            p.y > y0,
            "expected y to slide forward past the pillar, got {}",
            p.y
        );
    }

    #[test]
    fn gravity_pulls_z_down_when_airborne() {
        let mut p = spawn();
        p.vz = 1.0; // mid-jump, rising
        let z0 = p.z;
        step_gravity(&mut p, 0.1);
        // After 0.1 s the velocity has dropped by 1.2 to -0.2 and z has
        // risen slightly — but a full second later it must be strictly
        // below the start.
        step_gravity(&mut p, 1.0);
        assert!(
            p.z <= z0 + 1e-9,
            "expected z fallen back to ground, got {}",
            p.z
        );
    }

    #[test]
    fn gravity_clamps_to_ground_and_zeroes_vz() {
        let mut p = spawn();
        p.z = GROUND_Z + 0.01;
        p.vz = -10.0;
        step_gravity(&mut p, 0.1);
        assert!((p.z - GROUND_Z).abs() < 1e-12);
        assert_eq!(p.vz, 0.0);
    }

    #[test]
    fn try_jump_only_when_grounded() {
        let mut p = spawn();
        assert!(try_jump(&mut p));
        assert_eq!(p.vz, JUMP_INITIAL_VZ);
        // Second call while airborne must refuse.
        assert!(!try_jump(&mut p));
        // Simulate landing.
        p.z = GROUND_Z;
        p.vz = 0.0;
        assert!(try_jump(&mut p));
    }

    #[test]
    fn add_pitch_clamps_at_limit() {
        let mut p = spawn();
        add_pitch(&mut p, 100.0);
        assert!((p.pitch - PITCH_MAX).abs() < 1e-12);
        add_pitch(&mut p, -100.0);
        assert!((p.pitch + PITCH_MAX).abs() < 1e-12);
    }

    #[test]
    fn add_yaw_wraps_into_zero_tau() {
        let mut p = spawn();
        add_yaw(&mut p, TAU + 0.25);
        assert!(p.yaw >= 0.0 && p.yaw < TAU);
        assert!((p.yaw - 0.25).abs() < 1e-9);
        add_yaw(&mut p, -1.0);
        assert!(p.yaw >= 0.0 && p.yaw < TAU);
    }
}
