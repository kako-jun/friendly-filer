//! Runtime configuration.
//!
//! TOML loading / writing and CLI flag wiring land in Issue #17. This module
//! only defines the shape and the [`Default`] values so the rest of the FPS
//! skeleton can consume them.

/// How the player aims. Defaults to keyboard so that SSH / tmux sessions —
/// where mouse capture is unreliable — stay usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AimStyle {
    Keyboard,
    Mouse,
}

/// Top-level config.
///
/// - `lod_individual_max`: above this enemy count, the renderer switches to
///   "individual + faded background" mode (see [`crate::enemy::Swarm`]).
/// - `lod_faded_max`: above this, it switches to full swarm aggregation.
/// - `aim`: input style, see [`AimStyle`].
/// - `startup_fps_on`: whether FPS gameplay is enabled on launch. Set to
///   `false` to land straight in calm file-manager mode (F1-toggleable).
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub lod_individual_max: usize,
    pub lod_faded_max: usize,
    pub aim: AimStyle,
    pub startup_fps_on: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lod_individual_max: 20,
            lod_faded_max: 100,
            aim: AimStyle::Keyboard,
            startup_fps_on: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = Config::default();
        assert_eq!(c.lod_individual_max, 20);
        assert_eq!(c.lod_faded_max, 100);
        assert_eq!(c.aim, AimStyle::Keyboard);
        assert!(c.startup_fps_on);
    }
}
