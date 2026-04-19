//! Operation menu — the UI that opens the moment a file/folder is identified.
//!
//! Every enemy downed by a disc throw, and every portal/monolith shot, flows
//! through the same menu. The *context* differs (individual file vs. swarm
//! bulk vs. folder vs. the current-folder monolith), but the action set is
//! consistent enough to share one enum.
//!
//! The menu UI, bulk handling, Undo, `.trash` move, per-operation animation
//! (derezz for delete, copy-split for copy, ...) all land in Issue #12.

/// A single menu action.
///
/// `Info` shows metadata without altering the file. `Cancel` closes the menu
/// and lets the disc reset without performing any operation — the enemy
/// reverts to hostile ([`crate::palette::ENEMY_RED`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Open,
    Rename,
    Move,
    Copy,
    Delete,
    Info,
    Cancel,
}

/// Which kind of target the menu is currently acting on. Determines which
/// entries are shown / disabled, and which animation plays afterward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuContext {
    /// A single identified file.
    File,
    /// A swarm bulk selection (many files aggregated by extension / size).
    Swarm,
    /// A subfolder portal.
    Folder,
    /// The current folder's monolith (`new file` / `new folder` live here).
    Monolith,
}
