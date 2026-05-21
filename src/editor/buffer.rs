use std::path::PathBuf;

/// Holds the content and file path of a single open text buffer.
/// This will be replaced by a Rope data structure later.
#[derive(Default)]
pub struct Buffer {
    content: String,
    path: Option<PathBuf>,
    is_dirty: bool,
}

impl Buffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self {
            content: String::new(),
            path: None,
            is_dirty: false,
        }
    }

    /// Load content into the buffer and associate it with a file path.
    pub fn load(&mut self, content: String, path: PathBuf) {
        self.content = content;
        self.path = Some(path);
        self.is_dirty = false;
    }

    /// Clear the buffer content and path.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.content.clear();
        self.path = None;
        self.is_dirty = false;
    }

    /// The buffer's current text content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Mutable reference to the buffer content (e.g. for egui TextEdit).
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
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