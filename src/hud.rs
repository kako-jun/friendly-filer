//! Heads-up display state.
//!
//! The HUD owns the always-on overlay: crosshair, HP gauge, score, crash
//! counter, mode indicator, minimap and breadcrumb. Rendering — all blue
//! line-art per the TRON theme — arrives in Issue #13. This module just
//! carries the state.

/// Runtime mode. FPS features can be toggled off (F1) for a calm file-manager
/// experience, and `/` puts the world into [`Mode::Frozen`] (time-stop) for
/// rename / new-file / fuzzy-search input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal FPS gameplay.
    Fps,
    /// Enemies static, cross-hair selection only.
    FpsOff,
    /// Time stopped for text input / fuzzy search.
    Frozen,
    /// HP hit zero. "SYSTEM ERROR" glitch is playing. Respawn imminent.
    Crashed,
}

/// HUD state. Mirrors the relevant player + scoring values so the renderer
/// doesn't need to reach back into [`crate::player::Player`] on every frame.
#[derive(Debug, Clone, Copy)]
pub struct Hud {
    pub hp: u32,
    pub score: u64,
    pub crash_count: u32,
    pub mode: Mode,
}
