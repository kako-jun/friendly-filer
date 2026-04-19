//! Input loop: crossterm key events → high-level frame intents.
//!
//! The input layer keeps a small [`InputState`] that remembers **when each
//! motion key was last seen** (pressed or auto-repeated). A key is treated
//! as "held" if its last-seen timestamp is within [`KEY_HOLD_TIMEOUT`] of
//! the current frame. This gives us OS-key-repeat-rate-independent motion
//! (W held for 1 s advances exactly `MOVE_SPEED × 1 s` regardless of
//! whether the terminal fires 20 or 120 repeat events per second) and also
//! lets us correctly handle kitty / win32 `Release` events when they are
//! delivered — a release instantly drops the key from the tracker.
//!
//! One-shot actions (F1 toggle, Esc / q quit, space jump) are handled on
//! the `Press` event itself and do not go through the held-key table.
//!
//! Mouse aim is deliberately deferred to #16; arrow keys are the canonical
//! #18 aim binding.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// How long a key is considered "still held" after the most recent
/// `Press` / `Repeat` event. Must be longer than a typical terminal's
/// key-repeat interval (usually 30–50 ms) but short enough that releasing
/// the key stops motion before the player drifts visibly. 120 ms is a
/// comfortable middle ground and matches the "feel" of the pre-refactor
/// event-count model at default repeat rates.
pub const KEY_HOLD_TIMEOUT: Duration = Duration::from_millis(120);

/// Hard cap on the number of crossterm events drained in a single
/// [`poll_frame_input`] call. Without this, a user hammering keys (or a
/// terminal flooding us with bracketed-paste input) could monopolise the
/// frame. 128 is well above any realistic held-key burst at 60 FPS and
/// still prevents pathological stalls.
pub const MAX_EVENTS_PER_FRAME: usize = 128;

/// The collected set of actions triggered in one frame poll.
///
/// Fields are populated from [`InputState`] on each
/// [`poll_frame_input`] call. `forward` / `strafe` / `yaw_delta` /
/// `pitch_delta` are always in `[-1.0, 1.0]` (clamped before return) and
/// are continuous while the relevant key is held — OS key-repeat rate no
/// longer factors in.
#[derive(Debug, Default, Clone, Copy)]
pub struct FrameInput {
    /// Net forward intent. W held = +1, S held = −1, both / neither = 0.
    pub forward: f64,
    /// Net strafe intent. D held = +1, A held = −1.
    pub strafe: f64,
    /// Shift held while any motion key is held — apply
    /// [`crate::physics::RUN_MULTIPLIER`].
    pub run: bool,
    /// A jump was requested this frame (Space pressed).
    pub jump: bool,
    /// Net yaw delta. Right held = +1, Left held = −1. Caller scales by
    /// [`crate::physics::AIM_YAW_RATE`] × dt.
    pub yaw_delta: f64,
    /// Net pitch delta. Up held = +1, Down held = −1.
    pub pitch_delta: f64,
    /// F1 was pressed — toggle FPS OFF (handled by the caller).
    pub toggle_fps_off: bool,
    /// Esc or q was pressed — request a clean shutdown.
    pub quit: bool,
}

/// Persistent input state carried across frames. Remembers the last time
/// each motion key was pressed / auto-repeated, plus the running
/// mode-toggle / quit flags.
#[derive(Debug, Default)]
pub struct InputState {
    /// Last-seen-pressed timestamp for every motion key currently
    /// considered "live". Pruned on `Release` events and by time-based
    /// held-check during [`poll_frame_input`].
    key_last_seen: HashMap<KeyCode, Instant>,
    /// Whether the most recent press involved the Shift modifier. Used to
    /// derive [`FrameInput::run`] as long as any motion key is held.
    shift_held: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Is `code` currently considered held at time `now`?
    fn is_held(&self, code: KeyCode, now: Instant) -> bool {
        match self.key_last_seen.get(&code) {
            Some(&t) => now.duration_since(t) <= KEY_HOLD_TIMEOUT,
            None => false,
        }
    }

    /// Held check as a signed contribution to an axis (returns 1.0 / 0.0).
    fn axis(&self, code: KeyCode, now: Instant) -> f64 {
        if self.is_held(code, now) { 1.0 } else { 0.0 }
    }
}

/// One-shot action produced by [`apply_key`] — press events that do not
/// map onto the held-key table. Continuous motion events return `None`
/// because they are handled via `key_last_seen` updates as a side-effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputAction {
    /// Space was pressed this frame.
    Jump,
    /// F1 was pressed this frame.
    ToggleFpsOff,
    /// Esc or q was pressed this frame.
    Quit,
}

/// Fold a single crossterm [`KeyEvent`] into the persistent input state,
/// returning any one-shot action triggered by it. Motion keys update the
/// `key_last_seen` table as a side-effect and return `None`.
///
/// Release events prune the released key from `key_last_seen` immediately
/// (for terminals that deliver releases, e.g. kitty with the keyboard
/// enhancement flags enabled).
pub(crate) fn apply_key(state: &mut InputState, ev: KeyEvent, now: Instant) -> Option<InputAction> {
    // Normalise letter keys to their lowercase form so we don't track
    // `'W'` and `'w'` as two separate held keys when Shift is held.
    let code = normalise(ev.code);

    if matches!(ev.kind, KeyEventKind::Release) {
        state.key_last_seen.remove(&code);
        if matches!(
            code,
            KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')
        ) && !state.key_last_seen.keys().any(|k| {
            matches!(
                k,
                KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')
            )
        }) {
            state.shift_held = false;
        }
        return None;
    }

    // Press or Repeat.
    match code {
        // Motion keys → refresh held timestamp.
        KeyCode::Char('w')
        | KeyCode::Char('a')
        | KeyCode::Char('s')
        | KeyCode::Char('d')
        | KeyCode::Up
        | KeyCode::Down
        | KeyCode::Left
        | KeyCode::Right => {
            state.key_last_seen.insert(code, now);
            // Shift is sampled from the most recent motion press. The
            // uppercase-letter path (some terminals deliver `'W'` without
            // a SHIFT modifier bit) also counts as a run signal.
            let shift_bit = ev.modifiers.contains(KeyModifiers::SHIFT);
            let uppercase = matches!(
                ev.code,
                KeyCode::Char('W') | KeyCode::Char('A') | KeyCode::Char('S') | KeyCode::Char('D')
            );
            if matches!(
                code,
                KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d')
            ) {
                state.shift_held = shift_bit || uppercase;
            }
            None
        }

        // One-shot actions.
        KeyCode::Char(' ') => Some(InputAction::Jump),
        KeyCode::F(1) => Some(InputAction::ToggleFpsOff),
        KeyCode::Esc => Some(InputAction::Quit),
        KeyCode::Char('q') => Some(InputAction::Quit),

        _ => None,
    }
}

/// Collapse letter casing so `'W'` and `'w'` are treated as the same
/// tracked key. All other codes pass through unchanged.
fn normalise(code: KeyCode) -> KeyCode {
    match code {
        KeyCode::Char(c) if c.is_ascii_uppercase() => KeyCode::Char(c.to_ascii_lowercase()),
        other => other,
    }
}

/// Drain pending crossterm events into `state`, then snapshot the current
/// held-key set into a [`FrameInput`].
///
/// `poll_timeout` is forwarded to the first [`event::poll`]; subsequent
/// polls are non-blocking. At most [`MAX_EVENTS_PER_FRAME`] events are
/// drained per call to bound the worst-case frame time.
pub fn poll_frame_input(
    state: &mut InputState,
    poll_timeout: Duration,
) -> anyhow::Result<FrameInput> {
    let mut jump = false;
    let mut toggle_fps_off = false;
    let mut quit = false;

    let mut first = true;
    for _ in 0..MAX_EVENTS_PER_FRAME {
        let wait = if first { poll_timeout } else { Duration::ZERO };
        first = false;
        if !event::poll(wait)? {
            break;
        }
        match event::read()? {
            Event::Key(k) => {
                if let Some(action) = apply_key(state, k, Instant::now()) {
                    match action {
                        InputAction::Jump => jump = true,
                        InputAction::ToggleFpsOff => toggle_fps_off = true,
                        InputAction::Quit => quit = true,
                    }
                }
            }
            // Non-key events are ignored in #18; mouse aim lands in #16.
            _ => continue,
        }
    }

    let now = Instant::now();

    // Prune stale entries so the table doesn't grow unbounded on
    // terminals that never emit releases and the user walks through a
    // great many keys over a long session.
    state
        .key_last_seen
        .retain(|_, t| now.duration_since(*t) <= KEY_HOLD_TIMEOUT);

    let any_wasd = [
        KeyCode::Char('w'),
        KeyCode::Char('a'),
        KeyCode::Char('s'),
        KeyCode::Char('d'),
    ]
    .iter()
    .any(|k| state.is_held(*k, now));
    if !any_wasd {
        state.shift_held = false;
    }

    let forward = state.axis(KeyCode::Char('w'), now) - state.axis(KeyCode::Char('s'), now);
    let strafe = state.axis(KeyCode::Char('d'), now) - state.axis(KeyCode::Char('a'), now);
    // Right = yaw +, Left = yaw − (matches the pre-refactor convention
    // so existing physics tests continue to assert the same direction).
    let yaw_delta = state.axis(KeyCode::Right, now) - state.axis(KeyCode::Left, now);
    let pitch_delta = state.axis(KeyCode::Up, now) - state.axis(KeyCode::Down, now);

    Ok(FrameInput {
        forward: forward.clamp(-1.0, 1.0),
        strafe: strafe.clamp(-1.0, 1.0),
        run: state.shift_held && any_wasd,
        jump,
        yaw_delta: yaw_delta.clamp(-1.0, 1.0),
        pitch_delta: pitch_delta.clamp(-1.0, 1.0),
        toggle_fps_off,
        quit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn press(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: mods,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn release(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Release,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn apply_key_w_marks_forward_held() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        let action = apply_key(&mut st, press(KeyCode::Char('w'), KeyModifiers::NONE), t0);
        assert!(action.is_none());
        assert!(st.is_held(KeyCode::Char('w'), t0));
        assert!(!st.shift_held);
    }

    #[test]
    fn apply_key_shift_w_marks_run() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        apply_key(&mut st, press(KeyCode::Char('w'), KeyModifiers::SHIFT), t0);
        assert!(st.shift_held);
    }

    #[test]
    fn apply_key_uppercase_w_also_marks_run() {
        // Some terminals deliver 'W' as Char('W') with no SHIFT bit.
        let mut st = InputState::new();
        let t0 = Instant::now();
        apply_key(&mut st, press(KeyCode::Char('W'), KeyModifiers::NONE), t0);
        assert!(st.is_held(KeyCode::Char('w'), t0));
        assert!(st.shift_held);
    }

    #[test]
    fn apply_key_f1_returns_toggle() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        let action = apply_key(&mut st, press(KeyCode::F(1), KeyModifiers::NONE), t0);
        assert_eq!(action, Some(InputAction::ToggleFpsOff));
    }

    #[test]
    fn apply_key_esc_returns_quit() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        let action = apply_key(&mut st, press(KeyCode::Esc, KeyModifiers::NONE), t0);
        assert_eq!(action, Some(InputAction::Quit));
    }

    #[test]
    fn apply_key_q_returns_quit() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        let action = apply_key(&mut st, press(KeyCode::Char('q'), KeyModifiers::NONE), t0);
        assert_eq!(action, Some(InputAction::Quit));
    }

    #[test]
    fn apply_key_space_returns_jump() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        let action = apply_key(&mut st, press(KeyCode::Char(' '), KeyModifiers::NONE), t0);
        assert_eq!(action, Some(InputAction::Jump));
    }

    #[test]
    fn apply_key_unmapped_does_nothing() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        let action = apply_key(&mut st, press(KeyCode::Char('z'), KeyModifiers::NONE), t0);
        assert!(action.is_none());
        assert!(st.key_last_seen.is_empty());
    }

    #[test]
    fn release_drops_key_from_held_table() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        apply_key(&mut st, press(KeyCode::Char('w'), KeyModifiers::NONE), t0);
        assert!(st.is_held(KeyCode::Char('w'), t0));
        apply_key(&mut st, release(KeyCode::Char('w')), t0);
        assert!(!st.is_held(KeyCode::Char('w'), t0));
    }

    #[test]
    fn held_expires_after_timeout() {
        let mut st = InputState::new();
        let t0 = Instant::now();
        apply_key(&mut st, press(KeyCode::Char('w'), KeyModifiers::NONE), t0);
        let later = t0 + KEY_HOLD_TIMEOUT + Duration::from_millis(1);
        assert!(!st.is_held(KeyCode::Char('w'), later));
    }
}
