use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::buffer::Buffer;

/// Manages a collection of open text buffers keyed by their file path.
/// This replaces the single `Buffer` field in `RiideApp`.
#[derive(Default)]
pub struct BufferManager {
    buffers: HashMap<PathBuf, Buffer>,
}

impl BufferManager {
    /// Create a new empty buffer manager.
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    /// Get an immutable reference to a buffer by path. Returns `None` if not loaded.
    pub fn get(&self, path: &Path) -> Option<&Buffer> {
        self.buffers.get(path)
    }

    /// Get a mutable reference to a buffer by path. Returns `None` if not loaded.
    pub fn get_mut(&mut self, path: &Path) -> Option<&mut Buffer> {
        self.buffers.get_mut(path)
    }

    /// Load content into a buffer, creating it if it doesn't exist.
    pub fn load(&mut self, path: PathBuf, content: String) {
        let mut buffer = Buffer::new();
        buffer.load(content, path.clone());
        self.buffers.insert(path, buffer);
    }

    /// Remove a buffer by path. Returns `true` if it existed.
    pub fn remove(&mut self, path: &Path) -> bool {
        self.buffers.remove(path).is_some()
    }

    /// Returns true if a buffer for the given path is already loaded.
    pub fn contains(&self, path: &Path) -> bool {
        self.buffers.contains_key(path)
    }
}