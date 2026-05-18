use std::path::{Path, PathBuf};

/// Tracks the current workspace directory and the actively opened file.
/// Pure data — no I/O operations here.
#[derive(Default)]
pub struct WorkspaceState {
    current_dir: PathBuf,
    active_file_path: Option<PathBuf>,
}

impl WorkspaceState {
    /// Create a new workspace state rooted at `dir`.
    pub fn new(dir: PathBuf) -> Self {
        Self {
            current_dir: dir,
            active_file_path: None,
        }
    }

    /// Navigate into a directory and clear the active file.
    pub fn navigate_to(&mut self, dir: PathBuf) {
        self.current_dir = dir;
        self.active_file_path = None;
    }

    /// Set the active (open) file.
    pub fn set_active_file(&mut self, path: PathBuf) {
        self.active_file_path = Some(path);
    }

    /// The current directory being browsed in the file explorer.
    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// The currently open file path, if any.
    #[allow(dead_code)]
    pub fn active_file_path(&self) -> Option<&PathBuf> {
        self.active_file_path.as_ref()
    }
}