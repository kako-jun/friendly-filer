//! friendly-filer — FPS skeleton demo frame (Issue #8).
//!
//! Renders a single TRON-styled frame for ~0.8 seconds and exits:
//! black background, blue floor grid fading toward the horizon, a red
//! enemy placeholder in the middle, and a small blue label at the bottom.
//! Real input loop, enemy AI, disc physics and filesystem reads land in
//! the #9–#18 sub-issues.

use std::io::{Write, stdout};

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::style::{
    Color as CtColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};
use termray::Framebuffer;
use termray::label::{Font8x8, GlyphRenderer};

use friendly_filer::palette::{BG_BLACK, ENEMY_RED, GEOMETRY_GRAY, GRID_BLUE, UI_BLUE};

/// RAII guard that enters the alternate screen in raw mode on construction
/// and restores the terminal on drop. Ensures the terminal is cleaned up
/// even if the program panics or returns early.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, Hide, Clear(ClearType::All))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort cleanup. Errors here can't propagate but can't be
        // meaningfully recovered from either — we've already lost control
        // of the terminal.
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn main() -> anyhow::Result<()> {
    let (cols, rows) = size()?;
    let fb_w = cols as usize;
    // Reserve two rows for the terminal prompt and render at 2× vertical
    // resolution via half-block characters (one cell = top + bottom pixel).
    let fb_h = (rows as usize).saturating_sub(2) * 2;
    if fb_w == 0 || fb_h == 0 {
        return Ok(());
    }

    let mut fb = Framebuffer::new(fb_w, fb_h);
    draw_tron_demo(&mut fb);

    let _guard = TerminalGuard::new()?;

    render_frame(&fb)?;

    // Fixed display time so the demo frame is visible before exit. Replaced
    // by the real input loop + frame pacing in the #18 / #13 sub-issues.
    std::thread::sleep(std::time::Duration::from_millis(800));

    Ok(())
}

/// Paint the TRON skeleton frame directly into the framebuffer. Order:
/// clear → floor grid → horizon bar → enemy placeholder → HUD banner.
fn draw_tron_demo(fb: &mut Framebuffer) {
    fb.clear(BG_BLACK);

    let w = fb.width();
    let h = fb.height();
    if w == 0 || h == 0 {
        return;
    }

    // Horizon sits slightly above centre — leaves more floor area for the
    // grid, mimicking the termray camera tilt we'll adopt in #18.
    let horizon = h / 2;

    // ---------- Floor grid ----------
    //
    // A fake-perspective grid: lines spaced ~8 px on the near rows, spacing
    // contracting toward the horizon. Cheap stand-in for real ray-floor
    // intersection, which termray gives us in #3 / #9.
    draw_floor_grid(fb, horizon);

    // ---------- Horizon strip ----------
    //
    // A single bright blue line delineating ground from void.
    for x in 0..w {
        fb.set_pixel(x, horizon, GRID_BLUE);
    }

    // ---------- Enemy placeholder ----------
    //
    // Centred red rectangle. Stand-in for the wireframe hostile that the
    // real enemy module will render via termray sprites in #9.
    draw_enemy_placeholder(fb, horizon);

    // ---------- HUD banner ----------
    let banner = "FRIENDLY-FILER v0.2.0-dev - TRON MODE - #8";
    draw_banner(fb, banner);
}

/// Thin blue horizontal lines at receding vertical spacing above the near
/// edge, plus thin blue verticals converging toward the screen centre. Gives
/// the visual impression of an infinite floor plane without requiring the
/// real raycaster.
fn draw_floor_grid(fb: &mut Framebuffer, horizon: usize) {
    let w = fb.width();
    let h = fb.height();
    if horizon + 1 >= h {
        return;
    }

    let floor_depth = h - horizon - 1;

    // Horizontal grid lines: the `row_below_horizon` value shrinks as we
    // approach the horizon, so we step with a widening stride to get
    // perspective-like compression.
    let mut stride = 2usize;
    let mut y = h - 1;
    while y > horizon {
        for x in 0..w {
            fb.set_pixel(x, y, GRID_BLUE);
        }
        if y < stride + horizon + 1 {
            break;
        }
        y -= stride;
        // Increase stride so farther lines are sparser.
        stride = stride.saturating_add(1).min(floor_depth.max(1));
    }

    // Vertical grid lines converge on a vanishing point at (cx, horizon).
    // Sampling each floor row and projecting x positions outward produces
    // the classic TRON perspective fan.
    let cx = w as f64 / 2.0;
    let line_count = 13i32; // odd -> one line dead centre
    let half = line_count / 2;
    for y_row in (horizon + 1)..h {
        let d = (y_row - horizon) as f64; // distance below horizon, in pixels
        // Scale of the near plane relative to depth.
        let spread = d / (floor_depth.max(1) as f64) * (w as f64 / 1.4);
        for i in -half..=half {
            let offset = i as f64 * spread / half as f64;
            let x = cx + offset;
            if x >= 0.0 && (x as usize) < w {
                fb.set_pixel(x as usize, y_row, GRID_BLUE);
            }
        }
    }
}

/// Red rectangle standing on the floor, roughly where the first hostile
/// wireframe will be rendered by the real enemy pass. Filled in
/// [`GEOMETRY_GRAY`] with an [`ENEMY_RED`] outline so the silhouette reads
/// against the blue grid.
fn draw_enemy_placeholder(fb: &mut Framebuffer, horizon: usize) {
    let w = fb.width();
    let h = fb.height();

    // Size the box as a fraction of screen dims so it scales with the
    // terminal. Minimum dimensions keep it visible on 80×24.
    let box_w = (w / 10).max(6);
    let box_h = (h / 5).max(8);

    let cx = w / 2;
    // Base of box sits a little below the horizon, giving the enemy
    // "feet on the floor" at middle distance.
    let base_y = horizon + (h - horizon) / 4;
    let top_y = base_y.saturating_sub(box_h);
    let x0 = cx.saturating_sub(box_w / 2);
    let x1 = (x0 + box_w).min(w);

    // Gray fill.
    for y in top_y..base_y {
        for x in x0..x1 {
            fb.set_pixel(x, y, GEOMETRY_GRAY);
        }
    }

    // Red outline (top, bottom, sides).
    for x in x0..x1 {
        fb.set_pixel(x, top_y, ENEMY_RED);
        if base_y > 0 && base_y - 1 < h {
            fb.set_pixel(x, base_y - 1, ENEMY_RED);
        }
    }
    for y in top_y..base_y {
        fb.set_pixel(x0, y, ENEMY_RED);
        if x1 > 0 {
            fb.set_pixel(x1 - 1, y, ENEMY_RED);
        }
    }
}

/// Draw a short ASCII banner near the bottom of the framebuffer, using
/// termray's built-in 8×8 font. The text is clipped automatically by
/// [`Font8x8::draw_glyph`], which skips out-of-range characters.
fn draw_banner(fb: &mut Framebuffer, text: &str) {
    let font = Font8x8;
    let glyph_w = font.glyph_width() as i32;
    let glyph_h = font.glyph_height() as i32;

    let total_w = glyph_w * text.len() as i32;
    let fb_w = fb.width() as i32;
    let fb_h = fb.height() as i32;

    // Centre horizontally, sit 2 glyphs above the bottom.
    let start_x = ((fb_w - total_w) / 2).max(0);
    let y = (fb_h - glyph_h * 2).max(0);

    for (i, ch) in text.chars().enumerate() {
        let x = start_x + i as i32 * glyph_w;
        if x >= fb_w {
            break;
        }
        font.draw_glyph(fb, x, y, ch, UI_BLUE);
    }
}

/// Present the framebuffer using half-block characters: one cell = top pixel
/// (foreground) + bottom pixel (background).
fn render_frame(fb: &Framebuffer) -> anyhow::Result<()> {
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
