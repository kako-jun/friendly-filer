//! friendly-filer — FPS frame loop (Issue #18).
//!
//! Enters an alternate screen, drives a 60 FPS termray raycaster render
//! loop, and wires crossterm key events through the physics module into
//! the player pose. Esc / q quit cleanly. Enemy AI, disc physics, sprites
//! and the real HUD arrive with #9 / #10 / #13; the HUD line at the bottom
//! of the screen is a minimal debug readout.

use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
};
use termray::label::{Font8x8, GlyphRenderer};
use termray::{Framebuffer, render_floor_ceiling, render_walls};

use friendly_filer::input::{InputState, poll_frame_input};
use friendly_filer::palette::{BG_BLACK, UI_BLUE};
use friendly_filer::physics::{
    AIM_PITCH_RATE, AIM_YAW_RATE, GROUND_Z, MOVE_SPEED, RUN_MULTIPLIER, add_pitch, add_yaw,
    step_gravity, step_movement, try_jump,
};
use friendly_filer::player::Player;
use friendly_filer::render::{FloorTextureGrid, WallTextureFlat, present};
use friendly_filer::scene::DirScene;

/// Target frame time in milliseconds. 16 ms ≈ 62.5 FPS — the terminal
/// half-block renderer isn't pixel-perfect enough for higher rates to
/// make a visible difference, and staying here keeps CPU usage polite.
const FRAME_MS: u64 = 16;

/// Maximum raycaster depth in world units. The 8×8 placeholder arena is
/// bounded by 7.something units so 32 is comfortably beyond the far wall
/// without wasting DDA steps on empty space.
const RAY_MAX_DEPTH: f64 = 32.0;

/// RAII guard that enters the alternate screen in raw mode on construction
/// and restores the terminal on drop. Ensures the terminal is cleaned up
/// even if the program panics or returns early.
struct TerminalGuard;

impl TerminalGuard {
    /// Enter raw mode and then the alternate screen, returning a guard
    /// that restores both on drop.
    ///
    /// Ordering rationale:
    /// - If [`enable_raw_mode`] fails, `Self` is never constructed, so
    ///   `Drop` does not run. That is correct: we never transitioned
    ///   into raw mode and the terminal is still in its original state,
    ///   so there is nothing to undo.
    /// - After [`enable_raw_mode`] succeeds we construct `Self` first,
    ///   *then* call `execute!(EnterAlternateScreen, Hide)`. If those
    ///   subsequent calls fail, the guard still exists and its `Drop`
    ///   will run — disabling raw mode (and best-effort restoring the
    ///   screen / cursor) so the user's shell is not left in raw mode.
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let guard = Self;
        execute!(stdout(), EnterAlternateScreen, Hide)?;
        Ok(guard)
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

const ENEMY_SPRITE_TYPE: u8 = 3;

struct EnemyArt {
    enemy: termray::SpriteDef,
}

impl EnemyArt {
    fn new() -> Self {
        const ENEMY_PATTERN: &[&str] = &[
            ".#.",
            "###",
            "#.#",
            "#.#",
            "###",
            "#.#",
            "#.#",
        ];
        Self {
            enemy: termray::SpriteDef {
                pattern: ENEMY_PATTERN,
                height_scale: 1.2,
                float_offset_scale: 0.0,
            },
        }
    }
}

impl termray::SpriteArt for EnemyArt {
    fn art(&self, sprite_type: u8) -> Option<&termray::SpriteDef> {
        match sprite_type {
            ENEMY_SPRITE_TYPE => Some(&self.enemy),
            _ => None,
        }
    }

    fn color(&self, sprite_type: u8) -> termray::Color {
        match sprite_type {
            ENEMY_SPRITE_TYPE => termray::Color::rgb(220, 30, 30),
            _ => termray::Color::rgb(255, 255, 255),
        }
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

    // --- Scene + per-frame state ---
    let mut scene = DirScene::from_dir(&std::env::current_dir()?)?;
    let mut player = Player::new(scene.player_spawn.0, scene.player_spawn.1, scene.spawn_yaw);
    player.z = GROUND_Z;

    let mut camera = scene.camera();
    camera.set_z(GROUND_Z);

    let wall_tx = WallTextureFlat;
    let floor_tx = FloorTextureGrid;
    let enemy_art = EnemyArt::new();

    let mut fb = Framebuffer::new(fb_w, fb_h);
    let mut fps_off = false;

    // --- Enter the alternate screen, then run the frame loop ---
    let _guard = TerminalGuard::new()?;

    let frame_target = Duration::from_millis(FRAME_MS);
    let mut last_tick = Instant::now();
    let mut input_state = InputState::new();

    loop {
        // FIXME: ターミナルリサイズに未対応 (#13/#16 で対応予定)
        // Mark the start of this frame up-front so the pacing sleep at
        // the bottom accounts for the time spent polling input, running
        // physics, casting rays and writing to stdout — not just the
        // physics integration window.
        let frame_start = Instant::now();

        // Poll with a 0-duration timeout so the loop runs at the target
        // frame rate instead of stalling on slow keyboard input.
        let input = poll_frame_input(&mut input_state, Duration::ZERO)?;
        if input.quit {
            break;
        }
        if input.toggle_fps_off {
            fps_off = !fps_off;
        }

        // --- dt ---
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f64().min(0.1);
        last_tick = now;

        // --- Motion ---
        // Each event is one frame's worth; scale by MOVE_SPEED × dt so the
        // speed stays consistent if the terminal changes key-repeat rate.
        let speed = if input.run {
            MOVE_SPEED * RUN_MULTIPLIER
        } else {
            MOVE_SPEED
        };
        step_movement(
            &mut player,
            input.forward * speed,
            input.strafe * speed,
            dt,
            scene.map(),
        );

        // --- Jump + gravity ---
        if input.jump {
            try_jump(&mut player);
        }
        step_gravity(&mut player, dt);

        // --- Enemy AI ---
        use friendly_filer::physics::blocked_at;
        let is_blocked: Vec<bool> = {
            let map = scene.map();
            scene.enemies.iter()
                .map(|e| {
                    let (nx, ny) = e.compute_next_pos(player.x, player.y, dt);
                    blocked_at(map, nx, ny)
                })
                .collect()
        };
        for (enemy, blocked) in scene.enemies.iter_mut().zip(is_blocked) {
            if !blocked {
                let (next_x, next_y) = enemy.compute_next_pos(player.x, player.y, dt);
                enemy.x = next_x;
                enemy.y = next_y;
            }
            enemy.step_jump(dt);
        }

        // --- Aim ---
        if input.yaw_delta != 0.0 {
            add_yaw(&mut player, input.yaw_delta * AIM_YAW_RATE * dt);
        }
        if input.pitch_delta != 0.0 {
            add_pitch(&mut player, input.pitch_delta * AIM_PITCH_RATE * dt);
        }

        // --- Sync camera from player pose ---
        camera.set_pose(player.x, player.y, player.yaw);
        camera.set_z(player.z);
        camera.set_pitch(player.pitch);

        // --- Render ---
        fb.clear(BG_BLACK);
        let rays = camera.cast_all_rays(scene.map(), fb.width(), RAY_MAX_DEPTH);
        render_floor_ceiling(
            &mut fb,
            &rays,
            &floor_tx,
            scene.heights(),
            &camera,
            RAY_MAX_DEPTH,
        );
        render_walls(
            &mut fb,
            &rays,
            &wall_tx,
            scene.heights(),
            &camera,
            RAY_MAX_DEPTH,
        );

        // --- Enemy sprites ---
        let sprites: Vec<termray::Sprite> = scene.enemies.iter()
            .map(|e| termray::Sprite {
                x: e.x,
                y: e.y,
                sprite_type: ENEMY_SPRITE_TYPE,
            })
            .collect();
        let projected = termray::project_sprites(&sprites, &camera, scene.heights(), fb.width(), fb.height());
        termray::render_sprites(&mut fb, &projected, &rays, &enemy_art, RAY_MAX_DEPTH);

        draw_hud(&mut fb, &player, fps_off);

        present(&fb)?;

        // --- Frame pacing ---
        // Measure from `frame_start` so the sleep compensates for the
        // full frame (input poll + physics + render + present). Using
        // `last_tick` here instead would underestimate the elapsed time
        // because it was re-bound right after the input poll.
        let elapsed = frame_start.elapsed();
        if elapsed < frame_target {
            std::thread::sleep(frame_target - elapsed);
        }
    }

    Ok(())
}

/// Tiny debug HUD shown at the bottom-left of the frame. Full HUD with HP
/// bars, minimap and mode ornaments lands with Issue #13; for now we just
/// need to see that movement / jumps / aim are affecting state.
fn draw_hud(fb: &mut Framebuffer, player: &Player, fps_off: bool) {
    let font = Font8x8;
    let glyph_h = font.glyph_height() as i32;
    let glyph_w = font.glyph_width() as i32;

    let mode = if fps_off { "FPS-OFF" } else { "FPS" };
    let full = format!(
        "pos=({:.1},{:.1},{:.1}) yaw={:.2} pitch={:.2} vz={:.1} hp={} MODE={}",
        player.x, player.y, player.z, player.yaw, player.pitch, player.vz, player.hp, mode
    );

    let fb_h = fb.height() as i32;
    let fb_w = fb.width() as i32;

    // Prefer the verbose form; fall back to a compact single-line read-out
    // when the framebuffer isn't wide enough to fit it. The short form
    // drops the vz / mode labels and uses single-letter prefixes.
    let full_px = 2 + full.chars().count() as i32 * glyph_w;
    let text = if full_px <= fb_w {
        full
    } else {
        format!(
            "P{:.1},{:.1} Y{:.2} P{:.0} V{:.0} H{} {}",
            player.x, player.y, player.yaw, player.pitch, player.vz, player.hp, mode
        )
    };

    let y = (fb_h - glyph_h - 2).max(0);

    for (i, ch) in text.chars().enumerate() {
        let x = 2 + i as i32 * glyph_w;
        if x + glyph_w > fb_w {
            break;
        }
        font.draw_glyph(fb, x, y, ch, UI_BLUE);
    }
}
