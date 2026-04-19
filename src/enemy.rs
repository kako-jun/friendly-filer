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
    pub fn from_metadata(file_name: String, size: u64, pos: (f64, f64)) -> Self {
        let kind = classify(&file_name);
        let hp = hp_from_size(size);
        Self {
            file_name,
            size,
            x: pos.0,
            y: pos.1,
            z: 0.0,
            hp,
            kind,
            identified: false,
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
        let e = Enemy::from_metadata("game.rs".into(), 4096, (2.0, 3.0));
        assert_eq!(e.file_name, "game.rs");
        assert_eq!(e.size, 4096);
        assert_eq!(e.kind, EnemyKind::Nimble);
        assert_eq!(e.x, 2.0);
        assert_eq!(e.y, 3.0);
        assert!(!e.identified);
        assert!(e.hp >= 1 && e.hp <= 5);
    }
}
