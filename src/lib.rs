//! friendly-filer — a TRON-styled first-person file manager on top of termray.
//!
//! Each file in the current directory is rendered as a hostile wireframe
//! enemy; each subfolder becomes a portal. The player walks the 3D grid,
//! throws an identity disc to "identify" a target, and an operation menu
//! opens on return. Beneath the game layer it is a real, reversible file
//! manager — `delete` moves to `.trash`, `u` undoes, and `Game Over` does
//! not exist.
//!
//! This crate currently exposes the FPS-layer **skeleton** (Issue #8): the
//! module shapes and palette are in place, but movement, combat and real
//! filesystem reads arrive in the `#9–#18` sub-issues. See
//! `docs/fps-spec.md` for the full design.

pub mod config;
pub mod disc;
pub mod enemy;
pub mod hud;
pub mod menu;
pub mod palette;
pub mod physics;
pub mod player;
pub mod portal;
pub mod render;
pub mod scene;
