//! Portals (subfolders), the current-folder monolith, and the parent gate.
//!
//! - [`Portal`]: a door to a subfolder. Walking in = `cd`; shooting it opens
//!   a folder-level operation menu (copy / move / delete / rename / archive).
//!   Its HP is the total recursive size of the subfolder (cached).
//! - [`Monolith`]: a blue-rimmed cube standing in the centre of every folder.
//!   Shooting it opens the *current folder's* operation menu — which is also
//!   where `new file` / `new folder` live.
//! - [`ParentGate`]: a back-ward line to `..`. Can't be shot; walking through
//!   it returns to the parent directory.
//!
//! Wiring up `cd` / folder operations is Issue #11. Here we only define the
//! data types and the dangerous-path guard used to decide which portals ship
//! with a seal.

use std::path::{Path, PathBuf};

/// A subfolder rendered as a door.
///
/// `sealed` is set by [`is_dangerous_path`] at scene-build time and forces
/// the user through an opt-in confirmation before the portal can be
/// destroyed. Walking *into* a sealed portal (i.e. `cd`) is always allowed —
/// the seal only blocks shooting.
#[derive(Debug, Clone)]
pub struct Portal {
    pub path: PathBuf,
    pub x: f64,
    pub y: f64,
    pub total_size: u64,
    pub sealed: bool,
}

/// The current folder's self-operation monolith.
#[derive(Debug, Clone)]
pub struct Monolith {
    pub x: f64,
    pub y: f64,
}

/// The return-to-parent gate. Present on every directory except the
/// filesystem root.
#[derive(Debug, Clone)]
pub struct ParentGate {
    pub x: f64,
    pub y: f64,
}

/// Predicate used at scene-build time to mark irrecoverable portals as
/// `sealed = true`. Currently covers:
///
/// - The filesystem root (`/` on Unix, drive roots on Windows).
/// - `$HOME` itself and any direct child of `$HOME` (e.g. `~/Documents`).
///   These are the folders people actually lose work from.
///
/// Symlink-loop detection and additional seal rules join in Issue #11. The
/// real `cd` / shoot wiring also lives there; this function exists now so
/// its behaviour can be unit-tested in isolation.
pub fn is_dangerous_path(p: &Path) -> bool {
    // Filesystem root, including Windows-style `C:\`.
    if p.parent().is_none() {
        return true;
    }

    if let Some(home) = home_dir() {
        if p == home {
            return true;
        }
        if p.parent() == Some(home.as_path()) {
            return true;
        }
    }

    false
}

fn home_dir() -> Option<PathBuf> {
    // Deliberately avoiding the `home` / `dirs` crates at Phase 0 scope —
    // we only need `$HOME` / `USERPROFILE` and the `std` env lookup is
    // sufficient. Real home-dir resolution lands in #11 alongside the
    // actual filesystem work.
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Ok(h) = std::env::var("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // `is_dangerous_path` reads the process-wide `HOME` env var, and cargo
    // test runs tests in parallel by default. Combining parallelism with
    // `set_var` / `remove_var` would be a data race (and in edition 2024
    // those are `unsafe`), so the whole test surface lives in a single
    // `#[test]` function with one scoped env override.
    #[test]
    fn is_dangerous_path_behaviour() {
        #[cfg(unix)]
        {
            assert!(is_dangerous_path(Path::new("/")));
        }

        // Save originals so we can restore them and leave the env exactly
        // as we found it, no matter which branch we took.
        let original_home = std::env::var_os("HOME");
        let original_userprofile = std::env::var_os("USERPROFILE");

        let fake_home = "/tmp/friendly-filer-home-test";
        // SAFETY: only this test touches `HOME` / `USERPROFILE` during the
        // run; see the comment above on why we can't split this up.
        unsafe {
            std::env::set_var("HOME", fake_home);
        }

        assert!(is_dangerous_path(Path::new(fake_home)));
        assert!(is_dangerous_path(Path::new(&format!(
            "{fake_home}/Documents"
        ))));
        // A grandchild of $HOME is safe.
        assert!(!is_dangerous_path(Path::new(&format!(
            "{fake_home}/projects/freeza"
        ))));

        // With both `HOME` and `USERPROFILE` unset, a deep path under a
        // typical project tree must still be considered safe: we're
        // validating that `home_dir()` returning `None` doesn't accidentally
        // flag arbitrary paths as dangerous.
        unsafe {
            std::env::remove_var("HOME");
            std::env::remove_var("USERPROFILE");
        }
        assert!(!is_dangerous_path(Path::new("/Users/foo/projects/bar")));

        // Restore whatever was there before, preserving absence vs. empty.
        unsafe {
            match original_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match original_userprofile {
                Some(v) => std::env::set_var("USERPROFILE", v),
                None => std::env::remove_var("USERPROFILE"),
            }
        }
    }
}
