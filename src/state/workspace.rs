use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Tracks the current workspace directory, which folders are expanded in the tree,
/// which tabs are open, which tab is active, and any pending close confirmation.
/// Pure data — no I/O operations here.
#[derive(Default)]
pub struct WorkspaceState {
    pub current_dir: PathBuf,
    pub active_file_path: Option<PathBuf>,
    pub expanded_dirs: HashSet<PathBuf>,
    pub open_tabs: Vec<PathBuf>,
    pub pending_close_tab: Option<PathBuf>,
}

impl WorkspaceState {
    /// Create a new workspace state rooted at `dir`.
    pub fn new(dir: PathBuf) -> Self {
        Self {
            current_dir: dir,
            active_file_path: None,
            expanded_dirs: HashSet::new(),
            open_tabs: Vec::new(),
            pending_close_tab: None,
        }
    }

    /// Navigate into a directory and clear expanded sub-directories.
    pub fn navigate_to(&mut self, dir: PathBuf) {
        self.current_dir = dir;
        self.expanded_dirs.clear();
    }

    /// Set the active (focused) file.
    #[allow(dead_code)]
    pub fn set_active_file(&mut self, path: PathBuf) {
        self.active_file_path = Some(path);
    }

    /// The current directory being browsed in the file explorer.
    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// The currently focused file path, if any.
    pub fn active_file_path(&self) -> Option<&PathBuf> {
        self.active_file_path.as_ref()
    }

    /// Toggle whether a directory is expanded in the file tree.
    pub fn toggle_expanded(&mut self, path: &Path) {
        if self.expanded_dirs.contains(path) {
            self.expanded_dirs.remove(path);
        } else {
            self.expanded_dirs.insert(path.to_path_buf());
        }
    }

    /// Returns true if a directory is currently expanded.
    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded_dirs.contains(path)
    }

    /// Open a tab for the given file (no-op if already open).
    pub fn open_tab(&mut self, path: PathBuf) {
        if !self.open_tabs.contains(&path) {
            self.open_tabs.push(path.clone());
        }
        self.active_file_path = Some(path);
    }

    /// Close a tab. If it was the active tab, switches to a neighbour.
    pub fn close_tab(&mut self, path: &Path) {
        let idx = self.open_tabs.iter().position(|p| p == path);
        if let Some(i) = idx {
            self.open_tabs.remove(i);
            if self.active_file_path.as_deref() == Some(path) {
                if self.open_tabs.is_empty() {
                    self.active_file_path = None;
                } else if i < self.open_tabs.len() {
                    self.active_file_path = Some(self.open_tabs[i].clone());
                } else {
                    self.active_file_path = Some(self.open_tabs[i - 1].clone());
                }
            }
        }
    }

    /// Switch the active tab to the given path (must be in open_tabs).
    pub fn switch_tab(&mut self, path: &Path) {
        if self.open_tabs.contains(&path.to_path_buf()) {
            self.active_file_path = Some(path.to_path_buf());
        }
    }

    /// Remove all entries from `expanded_dirs` that are not children of `root`.
    #[allow(dead_code)]
    pub fn prune_expanded(&mut self, root: &Path) {
        self.expanded_dirs.retain(|p| p.starts_with(root));
    }
}