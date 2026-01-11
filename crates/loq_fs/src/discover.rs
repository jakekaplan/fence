//! Configuration file discovery.
//!
//! Finds `loq.toml` by walking up the directory tree from a starting point.

use std::path::{Path, PathBuf};

/// Finds a config file in or above the given directory.
///
/// Searches upward from `start_dir` looking for `loq.toml`.
/// Returns the path to the config file if found, or `None` if not found.
#[must_use]
pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
    let mut current = Some(start_dir);

    while let Some(dir) = current {
        let candidate = dir.join("loq.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        current = dir.parent();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn finds_config_in_current_dir() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("loq.toml"), "default_max_lines = 10").unwrap();

        let found = find_config(root);
        assert_eq!(found.unwrap(), root.join("loq.toml"));
    }

    #[test]
    fn finds_config_in_parent_dir() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let sub = root.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(root.join("loq.toml"), "default_max_lines = 10").unwrap();

        let found = find_config(&sub);
        assert_eq!(found.unwrap(), root.join("loq.toml"));
    }

    #[test]
    fn no_config_returns_none() {
        let temp = TempDir::new().unwrap();
        let found = find_config(temp.path());
        assert!(found.is_none());
    }

    #[test]
    fn directory_named_loq_toml_is_ignored() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create a DIRECTORY named loq.toml (unusual but possible)
        std::fs::create_dir(root.join("loq.toml")).unwrap();

        let found = find_config(root);

        // Should NOT find the directory, returns None
        assert!(found.is_none());
    }
}
