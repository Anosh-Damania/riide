use std::path::{Path, PathBuf};

/// Trait abstracting all file-system operations.
/// Enables swapping in mock implementations for testing.
pub trait FileSystemOps {
    /// List all entries in `path`, sorted with directories first.
    fn read_dir_entries(&self, path: &Path) -> Result<Vec<PathBuf>, String>;
    /// Read the entire contents of a text file.
    fn read_file(&self, path: &Path) -> Result<String, String>;
    /// Write `content` to `path`, creating/overwriting the file.
    fn write_file(&self, path: &Path, content: &str) -> Result<(), String>;
}

/// The real filesystem implementation using `std::fs`.
#[derive(Default)]
pub struct RealFileSystem;

impl FileSystemOps for RealFileSystem {
    fn read_dir_entries(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let mut paths: Vec<PathBuf> = Vec::new();
        let entries =
            std::fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            paths.push(entry.path());
        }
        // Sort: directories first, then files; each group alphabetically
        paths.sort_by(|a, b| {
            let a_dir = a.is_dir();
            let b_dir = b.is_dir();
            match (a_dir, b_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });
        Ok(paths)
    }

    fn read_file(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read file '{}': {}", path.display(), e))
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        std::fs::write(path, content)
            .map_err(|e| format!("Could not save file '{}': {}", path.display(), e))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_directory_identifies_file() {
        let tmp = std::env::temp_dir().join("riide_test_list");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let file_path = tmp.join("test_file.txt");
        std::fs::write(&file_path, "hello").expect("failed to write test file");

        let fs = RealFileSystem;
        let entries = fs.read_dir_entries(&tmp).expect("read_dir_entries failed");

        assert!(entries.contains(&file_path), "test file should be in the listing");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_read_file_success() {
        let tmp = std::env::temp_dir().join("riide_test_read");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let file_path = tmp.join("hello.txt");
        std::fs::write(&file_path, "Hello, Riide!").expect("failed to write test file");

        let fs = RealFileSystem;
        let content = fs.read_file(&file_path).expect("read_file should succeed");

        assert_eq!(content, "Hello, Riide!");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_read_nonexistent_file_returns_err() {
        let tmp = std::env::temp_dir().join("riide_test_nonexistent");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let missing = tmp.join("does_not_exist.txt");

        let fs = RealFileSystem;
        let result = fs.read_file(&missing);

        assert!(result.is_err(), "reading a non-existent file should return Err");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}