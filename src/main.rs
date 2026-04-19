//! friendly-filer — Phase 0 skeleton.
//!
//! Blanks the termray framebuffer with a dark background, presents a single
//! frame via crossterm half-block rendering, pauses briefly, and exits. Phase 1
//! will replace this with a real directory walk and scene build.

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
use termray::{Color, Framebuffer};

fn main() -> anyhow::Result<()> {
    let (cols, rows) = size()?;
    let fb_w = cols as usize;
    // Reserve two rows for the terminal prompt, render at 2x vertical resolution
    // via half-block characters (one cell = top + bottom pixel).
    let fb_h = (rows as usize).saturating_sub(2).max(1) * 2;

    let mut fb = Framebuffer::new(fb_w, fb_h);
    fb.clear(Color::rgb(15, 15, 25));

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    let result = render_frame(&fb);

    std::thread::sleep(std::time::Duration::from_millis(800));

    execute!(stdout(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

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
