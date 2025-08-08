//! Cross-platform path utilities for ZipLock configuration
//!
//! This module provides consistent path handling across different platforms
//! for configuration directories, data directories, and other application paths.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Get the user's configuration directory for ZipLock
///
/// This follows platform conventions:
/// - Linux: ~/.config/ziplock
/// - Windows: %APPDATA%/ZipLock
/// - macOS: ~/Library/Application Support/ZipLock
pub fn get_config_directory() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join(".config")))
        .context("Could not determine config directory")?
        .join("ziplock");

    Ok(config_dir)
}

/// Get the user's data directory for ZipLock
///
/// This is where repositories and other user data should be stored by default:
/// - Linux: ~/.local/share/ziplock
/// - Windows: %APPDATA%/ZipLock
/// - macOS: ~/Library/Application Support/ZipLock
pub fn get_data_directory() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
        .context("Could not determine data directory")?
        .join("ziplock");

    Ok(data_dir)
}

/// Get the user's cache directory for ZipLock
///
/// This is for temporary files, thumbnails, etc.:
/// - Linux: ~/.cache/ziplock
/// - Windows: %LOCALAPPDATA%/ZipLock/cache
/// - macOS: ~/Library/Caches/ZipLock
pub fn get_cache_directory() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join(".cache")))
        .context("Could not determine cache directory")?
        .join("ziplock");

    Ok(cache_dir)
}

/// Get the default directory for storing repositories
///
/// This provides a user-friendly default location:
/// - All platforms: ~/Documents/ZipLock or ~/ZipLock if Documents doesn't exist
pub fn get_default_repositories_directory() -> Result<PathBuf> {
    let default_dir = dirs::document_dir()
        .or_else(dirs::home_dir)
        .context("Could not determine default repositories directory")?
        .join("ZipLock");

    Ok(default_dir)
}

/// Get common search directories where repositories might be found
///
/// Returns a list of directories to search for existing repositories
pub fn get_common_repository_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Default repositories directory
    if let Ok(default_dir) = get_default_repositories_directory() {
        paths.push(default_dir);
    }

    // Documents directory
    if let Some(docs_dir) = dirs::document_dir() {
        paths.push(docs_dir);
    }

    // Desktop directory
    if let Some(desktop_dir) = dirs::desktop_dir() {
        paths.push(desktop_dir);
    }

    // Downloads directory
    if let Some(downloads_dir) = dirs::download_dir() {
        paths.push(downloads_dir);
    }

    // User data directory
    if let Ok(data_dir) = get_data_directory() {
        paths.push(data_dir);
    }

    // Home directory
    if let Some(home_dir) = dirs::home_dir() {
        paths.push(home_dir);
    }

    paths
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {path:?}"))?;
    }
    Ok(())
}

/// Check if a path is within the user's home directory
pub fn is_in_user_directory(path: &Path) -> bool {
    if let Some(home_dir) = dirs::home_dir() {
        path.starts_with(&home_dir)
    } else {
        false
    }
}

/// Get a relative path from the user's home directory if possible
pub fn get_relative_to_home(path: &Path) -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        path.strip_prefix(&home_dir).ok().map(|p| p.to_path_buf())
    } else {
        None
    }
}

/// Expand a path that starts with ~ to the full home directory path
pub fn expand_home_path(path: &str) -> Result<PathBuf> {
    if let Some(relative_path) = path.strip_prefix('~') {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;
        if relative_path.starts_with('/') || relative_path.starts_with('\\') {
            Ok(home_dir.join(&relative_path[1..]))
        } else if relative_path.is_empty() {
            Ok(home_dir)
        } else {
            Ok(home_dir.join(relative_path))
        }
    } else {
        Ok(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_directory() {
        let config_dir = get_config_directory().unwrap();
        assert!(config_dir.to_string_lossy().contains("ziplock"));
    }

    #[test]
    fn test_data_directory() {
        let data_dir = get_data_directory().unwrap();
        assert!(data_dir.to_string_lossy().contains("ziplock"));
    }

    #[test]
    fn test_cache_directory() {
        let cache_dir = get_cache_directory().unwrap();
        assert!(cache_dir.to_string_lossy().contains("ziplock"));
    }

    #[test]
    fn test_default_repositories_directory() {
        let repo_dir = get_default_repositories_directory().unwrap();
        assert!(repo_dir.to_string_lossy().contains("ZipLock"));
    }

    #[test]
    fn test_common_search_paths() {
        let paths = get_common_repository_search_paths();
        assert!(!paths.is_empty());

        // All paths should be absolute
        for path in &paths {
            assert!(path.is_absolute());
        }
    }

    #[test]
    fn test_expand_home_path() {
        let expanded = expand_home_path("~/Documents/test.7z").unwrap();
        assert!(expanded.is_absolute());
        assert!(expanded.to_string_lossy().contains("Documents"));
        assert!(expanded.to_string_lossy().contains("test.7z"));

        let expanded_root = expand_home_path("~").unwrap();
        assert!(expanded_root.is_absolute());

        let non_home = expand_home_path("/absolute/path").unwrap();
        assert_eq!(non_home, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_is_in_user_directory() {
        if let Some(home_dir) = dirs::home_dir() {
            let user_path = home_dir.join("Documents");
            assert!(is_in_user_directory(&user_path));

            let non_user_path = PathBuf::from("/etc/passwd");
            assert!(!is_in_user_directory(&non_user_path));
        }
    }

    #[test]
    fn test_relative_to_home() {
        if let Some(home_dir) = dirs::home_dir() {
            let user_path = home_dir.join("Documents").join("test.txt");
            let relative = get_relative_to_home(&user_path).unwrap();
            assert_eq!(relative, PathBuf::from("Documents/test.txt"));

            let non_user_path = PathBuf::from("/etc/passwd");
            assert!(get_relative_to_home(&non_user_path).is_none());
        }
    }
}
