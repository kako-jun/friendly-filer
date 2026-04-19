//! Input loop: crossterm key events → high-level frame intents.
//!
//! crossterm's raw-mode `KeyEvent` stream only delivers `Press` and `Repeat`
//! kinds on most terminals (there is no `KeyUp`), so continuous-motion
//! controls are implemented by treating every observed event as
//! "apply one frame's worth of motion". Holding W produces repeat events
//! at the terminal's key-repeat rate and the player glides forward; releasing
//! simply stops the events. This matches how vintage roguelikes handle
//! hold-to-move and keeps us away from platform-specific raw keyboard APIs.
//!
//! Mouse aim is deliberately deferred to #16; arrow keys are the canonical
//! #18 aim binding.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

/// The collected set of actions triggered in one frame poll.
///
/// All fields are zero / `false` in the "nothing happened" baseline;
/// the frame loop adds them into the player's state via
/// [`crate::physics`]. `forward` / `strafe` are *event counts*: each
/// `Press` or `Repeat` of the key bumps them by 1, and the caller
/// multiplies by [`crate::physics::MOVE_SPEED`] × frame_dt so the motion
/// per second ends up proportional to the terminal's key-repeat rate
/// (which the user can tune in their OS settings if they want snappier
/// controls).
#[derive(Debug, Default, Clone, Copy)]
pub struct FrameInput {
    /// Net forward intent. W adds +1, S subtracts 1.
    pub forward: f64,
    /// Net strafe intent. D adds +1, A subtracts 1.
    pub strafe: f64,
    /// Shift held while moving — apply [`crate::physics::RUN_MULTIPLIER`].
    pub run: bool,
    /// A jump was requested this frame.
    pub jump: bool,
    /// Net yaw delta in "event units" (Left = -1, Right = +1). The caller
    /// scales by [`crate::physics::AIM_YAW_RATE`] × dt.
    pub yaw_delta: f64,
    /// Net pitch delta. Up = +1 (look up), Down = -1.
    pub pitch_delta: f64,
    /// F1 was pressed — toggle FPS OFF (handled by the caller).
    pub toggle_fps_off: bool,
    /// Esc or q was pressed — request a clean shutdown.
    pub quit: bool,
}

/// Drain all currently-pending crossterm events and fold them into a single
/// [`FrameInput`]. Returns `Ok(FrameInput::default())` if nothing was
/// pending after `poll_timeout`.
///
/// `poll_timeout` is forwarded to [`event::poll`]; pass `Duration::ZERO`
/// for a non-blocking read (the common case inside a 60 FPS frame loop).
pub fn poll_frame_input(poll_timeout: Duration) -> anyhow::Result<FrameInput> {
    let mut fi = FrameInput::default();

    // First poll with the caller's timeout; subsequent polls must be
    // non-blocking so we don't wait out the full timeout for each event
    // in a burst (e.g. a held key).
    let mut first = true;
    loop {
        let wait = if first { poll_timeout } else { Duration::ZERO };
        first = false;
        if !event::poll(wait)? {
            break;
        }
        match event::read()? {
            Event::Key(k) => {
                // Ignore Release events; we only care about Press / Repeat.
                // (crossterm raw mode emits Press on most terminals and
                // Press+Repeat on kitty / win32; Release is rare but we're
                // future-proof against it.)
                if matches!(k.kind, KeyEventKind::Release) {
                    continue;
                }
                apply_key(&mut fi, k.code, k.modifiers);
            }
            // Non-key events are ignored in #18; mouse aim lands in #16.
            _ => continue,
        }
    }

    Ok(fi)
}

fn apply_key(fi: &mut FrameInput, code: KeyCode, mods: KeyModifiers) {
    // Track Shift as the "run" modifier any time a motion key fires.
    let shift = mods.contains(KeyModifiers::SHIFT);

    match code {
        // --- Movement ---
        KeyCode::Char('w') | KeyCode::Char('W') => {
            fi.forward += 1.0;
            fi.run |= shift;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            fi.forward -= 1.0;
            fi.run |= shift;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            fi.strafe -= 1.0;
            fi.run |= shift;
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            fi.strafe += 1.0;
            fi.run |= shift;
        }

        // --- Jump ---
        KeyCode::Char(' ') => fi.jump = true,

        // --- Aim (arrow keys) ---
        KeyCode::Up => fi.pitch_delta += 1.0,
        KeyCode::Down => fi.pitch_delta -= 1.0,
        KeyCode::Left => fi.yaw_delta -= 1.0,
        KeyCode::Right => fi.yaw_delta += 1.0,

        // --- Modes ---
        KeyCode::F(1) => fi.toggle_fps_off = true,

        // --- Quit ---
        KeyCode::Esc => fi.quit = true,
        KeyCode::Char('q') | KeyCode::Char('Q') => fi.quit = true,

        _ => {}
    }
}
