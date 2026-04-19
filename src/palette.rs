//! TRON-themed colour palette.
//!
//! Only three semantic colours make up the visible world: **black** (sky/void),
//! **blue** (grid, ally UI, identified files) and **red** (enemies, warnings,
//! lock-on). A muted **gray** is used for the faces of geometric props so that
//! silhouettes read cleanly against the black background.
//!
//! The palette is currently hard-coded. Issue #17 will introduce a `Config`
//! loader that lets users override any of these constants from a TOML file;
//! the aliases defined at the bottom of this module (`UI_BLUE`, `WARN_RED`,
//! `LOCK_RED`, ...) exist so that callers refer to semantic roles rather than
//! raw colour values, which makes that future override trivial.

use termray::Color;

/// Deep-space background. Slightly warmer than pure black so that the
/// anti-aliased blue grid lines stay crisp on terminals that dither dark
/// shades.
pub const BG_BLACK: Color = Color::rgb(8, 8, 12);

/// The TRON signature cyan-blue. Used for every piece of benign geometry:
/// floor grid, wall rims, portal frames, HUD chrome and — once a file has been
/// identified — the post-identification name plate.
pub const GRID_BLUE: Color = Color::rgb(79, 195, 247);

/// Hostile red. Enemy wireframes, lock-on brackets, HP-low warnings, and the
/// brief "SYSTEM ERROR" flash that plays after a crash.
pub const ENEMY_RED: Color = Color::rgb(255, 61, 61);

/// Neutral gray used for the shaded faces of props (walls, monolith,
/// portals). Never used on its own — it always sits behind a [`GRID_BLUE`]
/// outline.
pub const GEOMETRY_GRAY: Color = Color::rgb(58, 58, 58);

// ---------- Semantic aliases ----------
//
// Prefer these when calling code. They document *why* a colour is chosen at
// the call site, not just *what* RGB values are being pushed to termray.

/// HUD text, ally markers, identified-file name plates.
pub const UI_BLUE: Color = GRID_BLUE;

/// Warnings, low-HP flashes, "dangerous path" seal indicators.
pub const WARN_RED: Color = ENEMY_RED;

/// The lock-on brackets that frame an enemy while the disc is returning.
pub const LOCK_RED: Color = ENEMY_RED;

/// Secondary UI grey — same as [`GEOMETRY_GRAY`], aliased for readability in
/// HUD rendering code.
///
/// Primary use sites: HUD sub-text (#13, crash counter / breadcrumb body
/// text) and the monolith / portal panel faces. Like the other semantic
/// aliases, this is one of the constants the future `Config` loader (#17)
/// will let users override from TOML.
pub const UI_GRAY: Color = GEOMETRY_GRAY;
