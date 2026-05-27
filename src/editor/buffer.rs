use std::path::PathBuf;

/// Holds the content and file path of a single open text buffer.
///
/// Uses a `ropey::Rope` as the source of truth for efficient large-file editing,
/// and a `ui_cache: String` as a temporary sync layer for egui's `TextEdit`.
#[derive(Default)]
pub struct Buffer {
    /// Source of truth — efficient for large files.
    pub rope: ropey::Rope,
    /// Sync layer for egui — mutated by TextEdit, flushed to rope on save.
    ui_cache: String,
    path: Option<PathBuf>,
    is_dirty: bool,
}

impl Buffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self {
            rope: ropey::Rope::new(),
            ui_cache: String::new(),
            path: None,
            is_dirty: false,
        }
    }

    /// Load content into the buffer and associate it with a file path.
    /// Populates `ui_cache` from the rope so the UI has text to render immediately.
    pub fn load(&mut self, rope: ropey::Rope, path: PathBuf) {
        self.ui_cache = String::from(&rope);
        self.rope = rope;
        self.path = Some(path);
        self.is_dirty = false;
    }

    /// Clear the buffer content and path.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.rope = ropey::Rope::new();
        self.ui_cache.clear();
        self.path = None;
        self.is_dirty = false;
    }

    /// The buffer's current text content (from the UI cache).
    #[allow(dead_code)]
    pub fn content(&self) -> &str {
        &self.ui_cache
    }

    /// Mutable reference to the UI cache (e.g. for egui TextEdit).
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.ui_cache
    }

    /// Flush the UI cache back into the Rope source of truth.
    /// Call this before saving to disk.
    pub fn sync_to_rope(&mut self) {
        self.rope = ropey::Rope::from_str(&self.ui_cache);
    }

    /// The path associated with this buffer, if any.
    #[allow(dead_code)]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// Returns true if the buffer holds content (has a path loaded).
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.path.is_some()
    }

    /// Mark the buffer as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Mark the buffer as clean (e.g. after saving).
    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    /// Returns true if the buffer has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}