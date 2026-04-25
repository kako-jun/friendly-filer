//! Enemies = files.
//!
//! Each file in the current directory spawns one [`Enemy`]. The file's
//! extension decides its [`EnemyKind`], which drives future AI behaviour
//! (jump frequency, lunge speed, floatiness). HP is derived from file size
//! on a log scale so a 5-megabyte `.log` needs more hits than a 200-byte
//! `.rs`, but nothing takes more than five disc hits to down.
//!
//! The AI itself — update / jump / lunge / disc-throw — lands in Issue #9.

/// Behavioural archetype. Determined from a file's extension at spawn time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    /// `.rs` and other source files — quick, frequent jumps.
    Nimble,
    /// `.log`, `.bin` and other bulk files — slow, heavy, rare hops.
    Heavy,
    /// `.png`, `.jpg` and other media — floaty, hovers above the floor.
    Floaty,
    /// Everything else.
    Default,
}

/// An enemy derived from a single file entry.
///
/// `identified` flips to `true` the moment the disc confirms the kill; the
/// renderer uses it to switch the wireframe from [`crate::palette::ENEMY_RED`]
/// to [`crate::palette::GRID_BLUE`] and reveal the file name (Issue #12).
#[derive(Debug, Clone)]
pub struct Enemy {
    pub file_name: String,
    pub size: u64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vz: f64,
    pub on_ground: bool,
    pub jump_timer: f64,
    pub hp: u32,
    pub kind: EnemyKind,
    pub identified: bool,
}

impl Enemy {
    /// Build an enemy from file metadata.
    ///
    /// - `kind` is picked from the extension (see [`EnemyKind`]).
    /// - `hp` is `ceil(log2(size_kb))`, clamped to `[1, 5]`. Empty files and
    ///   files under 1 KB all get 1 HP.
    /// - `x` / `y` are scene-space coordinates (not screen). `z` starts at 0
    ///   for ground-walkers and is offset to `0.8` for [`EnemyKind::Floaty`]
    ///   on the first [`Self::step_jump`] cycle (see #18 for the long-form
    ///   hover-bob curve).
    /// - `jump_timer` is seeded with `jump_interval(kind) × jitter`, where
    ///   `jitter ∈ [0.8, 1.2]` is derived from a [`DefaultHasher`] of the
    ///   filename. Same-length filenames therefore desynchronise too,
    ///   which a length-only hash would not achieve.
    pub fn from_metadata(file_name: String, size: u64, x: f64, y: f64) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let kind = classify(&file_name);
        let hp = hp_from_size(size);
        let mut hasher = DefaultHasher::new();
        file_name.hash(&mut hasher);
        let jitter = 0.8 + (hasher.finish() % 41) as f64 / 100.0;
        let jump_timer = jump_interval(kind) * jitter;
        Self {
            file_name,
            size,
            x,
            y,
            z: 0.0,
            vz: 0.0,
            on_ground: true,
            jump_timer,
            hp,
            kind,
            identified: false,
        }
    }

    /// Compute new position toward player without wall collision check.
    /// The caller is responsible for calling `blocked_at` and updating position.
    pub fn compute_next_pos(&self, player_x: f64, player_y: f64, dt: f64) -> (f64, f64) {
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 0.1 {
            return (self.x, self.y); // Already at player
        }

        let speed = match self.kind {
            EnemyKind::Nimble => 1.5,
            EnemyKind::Heavy => 0.7,
            EnemyKind::Floaty => 1.0,
            EnemyKind::Default => 1.0,
        };

        let vx = (dx / dist) * speed * dt;
        let vy = (dy / dist) * speed * dt;

        (self.x + vx, self.y + vy)
    }

    /// Apply gravity and handle jumping with kind-based frequencies.
    pub fn step_jump(&mut self, dt: f64) {
        let gravity = if self.kind == EnemyKind::Floaty { 5.0 } else { 12.0 };
        let ground_z = if self.kind == EnemyKind::Floaty { 0.8 } else { 0.0 };

        if self.on_ground && self.vz == 0.0 {
            self.jump_timer -= dt;
            if self.jump_timer <= 0.0 {
                self.vz = 4.5;
                self.on_ground = false;
                self.jump_timer = jump_interval(self.kind);
            }
            return;
        }

        self.on_ground = false;
        self.vz -= gravity * dt;
        self.z += self.vz * dt;

        if self.z <= ground_z {
            self.z = ground_z;
            self.vz = 0.0;
            self.on_ground = true;
        }
    }
}

fn classify(file_name: &str) -> EnemyKind {
    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("rs") => EnemyKind::Nimble,
        Some("log") => EnemyKind::Heavy,
        Some("png") => EnemyKind::Floaty,
        _ => EnemyKind::Default,
    }
}

/// HP mapping from file byte size.
///
/// Empty files, files under 1 KB, and files that are **exactly 1 KB
/// (1024 bytes)** all resolve to 1 HP — the `kb <= 1.0` cutoff is
/// inclusive. From just above 1 KB, HP scales as `ceil(log2(size_kb))`,
/// capped at 5 so that no file takes more than five disc hits to down.
fn hp_from_size(size: u64) -> u32 {
    if size == 0 {
        return 1;
    }
    let kb = size as f64 / 1024.0;
    if kb <= 1.0 {
        return 1;
    }
    let raw = kb.log2().ceil() as i64;
    raw.clamp(1, 5) as u32
}

fn jump_interval(kind: EnemyKind) -> f64 {
    match kind {
        EnemyKind::Nimble => 1.5,
        EnemyKind::Heavy => 5.0,
        EnemyKind::Floaty => 2.0,
        EnemyKind::Default => 3.0,
    }
}

/// A swarm is an aggregate spawned by the LOD system when a directory has
/// more than `Config::lod_individual_max` files. One swarm visually replaces
/// many individual enemies of the same `kind_hint` / size band.
///
/// Full aggregation rules and visuals land in Issues #9 / #13.
#[derive(Debug, Clone)]
pub struct Swarm {
    pub kind_hint: EnemyKind,
    pub member_count: usize,
    pub total_size: u64,
    pub x: f64,
    pub y: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_by_extension() {
        assert_eq!(classify("main.rs"), EnemyKind::Nimble);
        assert_eq!(classify("path/to/lib.RS"), EnemyKind::Nimble);
        assert_eq!(classify("server.log"), EnemyKind::Heavy);
        assert_eq!(classify("screenshot.png"), EnemyKind::Floaty);
        assert_eq!(classify("README"), EnemyKind::Default);
        assert_eq!(classify("data.tar.gz"), EnemyKind::Default);
    }

    #[test]
    fn hp_is_in_range_and_grows_with_size() {
        assert_eq!(hp_from_size(0), 1);
        assert_eq!(hp_from_size(100), 1);
        assert_eq!(hp_from_size(1024), 1);
        // ~8 KB -> log2(8)=3
        assert_eq!(hp_from_size(8 * 1024), 3);
        // Huge files clamp to 5.
        assert_eq!(hp_from_size(10 * 1024 * 1024 * 1024), 5);
    }

    #[test]
    fn from_metadata_populates_fields() {
        let e = Enemy::from_metadata("game.rs".into(), 4096, 2.0, 3.0);
        assert_eq!(e.file_name, "game.rs");
        assert_eq!(e.size, 4096);
        assert_eq!(e.kind, EnemyKind::Nimble);
        assert_eq!(e.x, 2.0);
        assert_eq!(e.y, 3.0);
        assert!(!e.identified);
        assert!(e.hp >= 1 && e.hp <= 5);
    }
}
