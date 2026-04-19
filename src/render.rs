//! Framebuffer → terminal presentation and the TRON texture adapters.
//!
//! This module is the thin shim between termray's generic renderer and
//! friendly-filer's TRON palette. It owns:
//!
//! - [`present`] — writes a [`Framebuffer`] to `stdout` using half-block
//!   characters (one cell = top + bottom pixel). Extracted out of the
//!   #8 demo's `main.rs` so both the frame loop and future offline
//!   renderers can share it.
//! - [`WallTextureFlat`] — a stub [`WallTexturer`] that paints every wall
//!   surface with [`palette::GEOMETRY_GRAY`] darkened by distance, outlined
//!   implicitly by termray's darken-by-distance falloff. Real TRON wall
//!   treatment (blue edges, per-face shading) lands with #12 / #14.
//! - [`FloorTextureGrid`] — a two-tone grid [`FloorTexturer`] that alternates
//!   [`palette::GRID_BLUE`] cell borders against [`palette::BG_BLACK`]
//!   interiors. Matches the TRON "infinite floor grid" look of the #8
//!   demo but now driven by real ray-floor intersection.

use std::io::{Write, stdout};

use crossterm::execute;
use crossterm::style::{
    Color as CtColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use termray::{Color, FloorTexturer, Framebuffer, HitSide, TileType, WallTexturer};

use crate::palette::{BG_BLACK, GEOMETRY_GRAY, GRID_BLUE};

/// Width (in world units) of a floor grid line. Cells are 1×1 world units
/// so any fractional coordinate closer than this to an integer sits inside
/// a grid line. Tuned so the line is visible at spawn distance without
/// pixelating into a solid plane near the camera.
const FLOOR_GRID_LINE_WIDTH: f64 = 0.06;

/// Wall texturer that paints every surface with a single flat gray,
/// darkened by distance via termray's `brightness` argument. Good enough
/// for the #18 movement demo; real TRON wall shading (blue rims, per-face
/// tints) lands with later issues.
pub struct WallTextureFlat;

impl WallTexturer for WallTextureFlat {
    fn sample_wall(
        &self,
        _tile: TileType,
        _wall_x: f64,
        _wall_y: f64,
        side: HitSide,
        brightness: f64,
        _tile_hash: u32,
    ) -> Color {
        // Give the two side-classes a slight brightness offset so opposing
        // walls don't blur into one another when the camera is square-on.
        // termray's `HitSide::Vertical` = the ray hit a N/S face,
        // `Horizontal` = E/W face.
        let side_bias = match side {
            HitSide::Vertical => 1.0,
            HitSide::Horizontal => 0.75,
        };
        GEOMETRY_GRAY.darken(brightness * side_bias)
    }
}

/// Floor / ceiling texturer that draws a blue grid on black. Floor grid
/// lines sit on integer world-coordinate boundaries; the ceiling is always
/// background black (for now — the TRON ceiling grid can be enabled later
/// via config in #17).
pub struct FloorTextureGrid;

impl FloorTexturer for FloorTextureGrid {
    fn sample_floor(&self, world_x: f64, world_y: f64, brightness: f64) -> Color {
        // Distance from the nearest integer in x / y. If either is under
        // the line-width threshold we're on a grid line.
        let dx = (world_x - world_x.round()).abs();
        let dy = (world_y - world_y.round()).abs();
        if dx < FLOOR_GRID_LINE_WIDTH || dy < FLOOR_GRID_LINE_WIDTH {
            GRID_BLUE.darken(brightness.clamp(0.0, 1.0))
        } else {
            BG_BLACK
        }
    }

    fn sample_ceiling(&self, _world_x: f64, _world_y: f64, _brightness: f64) -> Color {
        BG_BLACK
    }
}

/// Write the framebuffer to stdout using the half-block character `▀`:
/// the top half-pixel becomes the foreground color, the bottom becomes
/// the background color. Emits `\r\n` between rows so the terminal cursor
/// resets to column 0 under raw mode (which suppresses the implicit `\r`).
///
/// Returns `Ok(())` for a zero-height framebuffer (nothing to present).
pub fn present(fb: &Framebuffer) -> anyhow::Result<()> {
    let mut out = stdout();
    let height = fb.height();
    if height == 0 {
        return Ok(());
    }
    for y in (0..height).step_by(2) {
        for x in 0..fb.width() {
            let top = fb.get_pixel(x, y);
            let bot = fb.get_pixel(x, (y + 1).min(height - 1));
            execute!(
                out,
                SetForegroundColor(CtColor::Rgb {
                    r: top.r,
                    g: top.g,
                    b: top.b
                }),
                SetBackgroundColor(CtColor::Rgb {
                    r: bot.r,
                    g: bot.g,
                    b: bot.b
                }),
                Print("\u{2580}"),
            )?;
        }
        execute!(out, ResetColor, Print("\r\n"))?;
    }
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_grid_returns_blue_on_integer_coord() {
        let t = FloorTextureGrid;
        let c = t.sample_floor(2.0, 3.5, 1.0);
        // On an integer x boundary the sampler must return something
        // brighter than pure black.
        assert!(u32::from(c.r) + u32::from(c.g) + u32::from(c.b) > 0);
    }

    #[test]
    fn floor_grid_returns_black_interior() {
        let t = FloorTextureGrid;
        // Middle of a cell (0.5, 0.5) is far from any integer boundary.
        let c = t.sample_floor(2.5, 3.5, 1.0);
        assert_eq!(c.r, BG_BLACK.r);
        assert_eq!(c.g, BG_BLACK.g);
        assert_eq!(c.b, BG_BLACK.b);
    }

    #[test]
    fn floor_grid_ceiling_is_always_background() {
        let t = FloorTextureGrid;
        let c = t.sample_ceiling(2.0, 3.0, 1.0);
        assert_eq!(c.r, BG_BLACK.r);
        assert_eq!(c.g, BG_BLACK.g);
        assert_eq!(c.b, BG_BLACK.b);
    }

    #[test]
    fn wall_texture_darkens_with_distance() {
        let t = WallTextureFlat;
        let near = t.sample_wall(1, 0.5, 0.5, HitSide::Vertical, 1.0, 0);
        let far = t.sample_wall(1, 0.5, 0.5, HitSide::Vertical, 0.1, 0);
        // The near color should be brighter than the far color in at least
        // one channel.
        assert!(
            u32::from(near.r) + u32::from(near.g) + u32::from(near.b)
                > u32::from(far.r) + u32::from(far.g) + u32::from(far.b)
        );
    }
}
