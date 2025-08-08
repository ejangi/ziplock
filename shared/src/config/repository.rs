//! Repository detection and validation utilities
//!
//! This module provides functionality to discover, validate, and manage
//! ZipLock repositories on the filesystem.

use crate::error::{SharedError, SharedResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, warn};

use super::RepositoryInfo;

/// Find all potential ZipLock repositories in a directory
///
/// Searches for .7z files that could be ZipLock repositories.
/// Does not perform deep validation - just identifies candidates.
pub fn find_repositories_in_directory<P: AsRef<Path>>(dir: P) -> SharedResult<Vec<RepositoryInfo>> {
    let dir = dir.as_ref();

    if !dir.exists() || !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut repositories = Vec::new();

    debug!("Searching for repositories in: {:?}", dir);

    let entries = fs::read_dir(dir).map_err(|e| SharedError::Internal {
        message: format!("Failed to read directory {dir:?}: {e}"),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| SharedError::Internal {
            message: format!("Failed to read directory entry: {e}"),
        })?;

        let path = entry.path();

        // Skip directories and non-.7z files
        if !path.is_file() {
            continue;
        }

        if !has_ziplock_extension(&path) {
            continue;
        }

        // Try to create repository info
        match RepositoryInfo::from_path(&path) {
            Ok(mut repo_info) => {
                // Perform basic validation
                repo_info.is_valid_format = is_potentially_valid_repository(&path);
                repositories.push(repo_info);
            }
            Err(e) => {
                warn!(
                    "Failed to get info for potential repository {:?}: {}",
                    path, e
                );
            }
        }
    }

    debug!(
        "Found {} potential repositories in {:?}",
        repositories.len(),
        dir
    );
    Ok(repositories)
}

/// Check if a file has a valid ZipLock repository extension
pub fn has_ziplock_extension<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("7z"))
        .unwrap_or(false)
}

/// Perform basic validation to check if a file could be a ZipLock repository
///
/// This performs lightweight checks without requiring the master password:
/// - File exists and is readable
/// - Has correct extension
/// - Has reasonable file size
/// - Basic file format checks
pub fn is_potentially_valid_repository<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Check if file exists and is readable
    if !path.exists() || !path.is_file() {
        return false;
    }

    // Check extension
    if !has_ziplock_extension(path) {
        return false;
    }

    // Check file size (must be > 0 and < 100MB for reasonable repositories)
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len();
        if size == 0 || size > 100 * 1024 * 1024 {
            return false;
        }
    } else {
        return false;
    }

    // Try to open as 7z archive (basic format validation)
    // This is a lightweight check without decryption
    is_valid_7z_format(path)
}

/// Check if a file appears to be a valid 7z archive
///
/// Performs basic format validation without attempting decryption
fn is_valid_7z_format<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Check 7z file signature
    match fs::File::open(path) {
        Ok(mut file) => {
            use std::io::Read;
            let mut header = [0u8; 6];
            if file.read_exact(&mut header).is_ok() {
                // 7z signature: '7', 'z', 0xBC, 0xAF, 0x27, 0x1C
                header == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Recursively search for repositories in a directory tree
///
/// Searches up to a specified depth to avoid infinite recursion
pub fn find_repositories_recursive<P: AsRef<Path>>(
    dir: P,
    max_depth: usize,
) -> SharedResult<Vec<RepositoryInfo>> {
    let dir = dir.as_ref();
    let mut all_repositories = Vec::new();

    fn search_recursive(
        dir: &Path,
        current_depth: usize,
        max_depth: usize,
        repositories: &mut Vec<RepositoryInfo>,
    ) -> SharedResult<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        // Find repositories in current directory
        match find_repositories_in_directory(dir) {
            Ok(mut repos) => repositories.append(&mut repos),
            Err(e) => {
                warn!("Failed to search directory {:?}: {}", dir, e);
            }
        }

        // Search subdirectories if we haven't reached max depth
        if current_depth < max_depth {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip hidden directories and common non-repository directories
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with('.')
                                || matches!(
                                    name,
                                    "node_modules" | "target" | ".git" | ".svn" | "build" | "dist"
                                )
                            {
                                continue;
                            }
                        }

                        search_recursive(&path, current_depth + 1, max_depth, repositories)?;
                    }
                }
            }
        }

        Ok(())
    }

    search_recursive(dir, 0, max_depth, &mut all_repositories)?;

    // Remove duplicates and sort by last modified
    all_repositories.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    all_repositories.dedup_by(|a, b| a.path == b.path);

    Ok(all_repositories)
}

/// Get metadata for a repository file
pub fn get_repository_metadata<P: AsRef<Path>>(path: P) -> SharedResult<RepositoryMetadata> {
    let path = path.as_ref();

    let metadata = fs::metadata(path).map_err(|e| SharedError::Internal {
        message: format!("Failed to read file metadata for {path:?}: {e}"),
    })?;

    let display_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    Ok(RepositoryMetadata {
        path: path.to_path_buf(),
        display_name,
        size: metadata.len(),
        last_modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
        is_valid_format: is_potentially_valid_repository(path),
        is_accessible: path.exists() && path.is_file(),
    })
}

/// Detailed metadata about a repository file
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryMetadata {
    /// Path to the repository file
    pub path: PathBuf,

    /// Display name (filename without extension)
    pub display_name: String,

    /// File size in bytes
    pub size: u64,

    /// Last modified timestamp
    pub last_modified: SystemTime,

    /// Created timestamp
    pub created: SystemTime,

    /// Whether this appears to be a valid repository format
    pub is_valid_format: bool,

    /// Whether the file is accessible for reading
    pub is_accessible: bool,
}

/// Search for repositories in common user directories
pub fn find_repositories_in_common_locations() -> Vec<RepositoryInfo> {
    let mut all_repositories = Vec::new();

    // Get common search paths
    let search_paths = super::paths::get_common_repository_search_paths();

    for search_path in search_paths {
        if let Ok(mut repos) = find_repositories_in_directory(&search_path) {
            all_repositories.append(&mut repos);
        }
    }

    // Remove duplicates and sort
    all_repositories.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    all_repositories.dedup_by(|a, b| a.path == b.path);

    // Limit results to prevent overwhelming the user
    all_repositories.truncate(50);

    all_repositories
}

/// Validate that a user has permission to access a repository file
pub fn validate_user_access<P: AsRef<Path>>(path: P) -> SharedResult<bool> {
    let path = path.as_ref();

    // Check if file exists
    if !path.exists() {
        return Ok(false);
    }

    // Check if file is readable
    match fs::File::open(path) {
        Ok(_) => Ok(true),
        Err(e) => {
            debug!("Failed to open repository file {:?}: {}", path, e);
            Ok(false)
        }
    }
}

/// Check if a path looks like it could be a repository filename
pub fn is_likely_repository_filename(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();

    // Must end with .7z
    if !filename_lower.ends_with(".7z") {
        return false;
    }

    // Common repository naming patterns
    let repository_patterns = [
        "password",
        "credential",
        "vault",
        "safe",
        "keychain",
        "ziplock",
        "secret",
        "login",
        "key",
        "wallet",
    ];

    repository_patterns
        .iter()
        .any(|&pattern| filename_lower.contains(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_has_ziplock_extension() {
        assert!(has_ziplock_extension("test.7z"));
        assert!(has_ziplock_extension("TEST.7Z"));
        assert!(has_ziplock_extension("/path/to/repo.7z"));

        assert!(!has_ziplock_extension("test.zip"));
        assert!(!has_ziplock_extension("test.txt"));
        assert!(!has_ziplock_extension("test"));
    }

    #[test]
    fn test_find_repositories_in_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        let repo1 = temp_dir.path().join("passwords.7z");
        let repo2 = temp_dir.path().join("vault.7z");
        let not_repo = temp_dir.path().join("document.pdf");

        File::create(&repo1).unwrap();
        File::create(&repo2).unwrap();
        File::create(&not_repo).unwrap();

        let repositories = find_repositories_in_directory(temp_dir.path()).unwrap();

        assert_eq!(repositories.len(), 2);
        assert!(repositories.iter().any(|r| r.path == repo1));
        assert!(repositories.iter().any(|r| r.path == repo2));
    }

    #[test]
    fn test_is_valid_7z_format() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file with 7z signature
        let valid_7z = temp_dir.path().join("valid.7z");
        let mut file = File::create(&valid_7z).unwrap();
        use std::io::Write;
        file.write_all(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C])
            .unwrap();

        // Create a file without 7z signature
        let invalid_7z = temp_dir.path().join("invalid.7z");
        let mut file = File::create(&invalid_7z).unwrap();
        file.write_all(b"not a 7z file").unwrap();

        assert!(is_valid_7z_format(&valid_7z));
        assert!(!is_valid_7z_format(&invalid_7z));
    }

    #[test]
    fn test_repository_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo.7z");

        File::create(&repo_path).unwrap();

        let metadata = get_repository_metadata(&repo_path).unwrap();
        assert_eq!(metadata.path, repo_path);
        assert_eq!(metadata.display_name, "test_repo");
        assert!(metadata.is_accessible);
    }

    #[test]
    fn test_is_likely_repository_filename() {
        assert!(is_likely_repository_filename("passwords.7z"));
        assert!(is_likely_repository_filename("my_vault.7z"));
        assert!(is_likely_repository_filename("credentials.7z"));
        assert!(is_likely_repository_filename("ziplock_backup.7z"));

        assert!(!is_likely_repository_filename("document.7z"));
        assert!(!is_likely_repository_filename("backup.zip"));
        assert!(!is_likely_repository_filename("test.txt"));
    }

    #[test]
    fn test_validate_user_access() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.7z");
        let non_existing_file = temp_dir.path().join("not_exists.7z");

        File::create(&existing_file).unwrap();

        assert!(validate_user_access(&existing_file).unwrap());
        assert!(!validate_user_access(&non_existing_file).unwrap());
    }

    #[test]
    fn test_find_repositories_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let repo1 = temp_dir.path().join("repo1.7z");
        let repo2 = subdir.join("repo2.7z");

        File::create(&repo1).unwrap();
        File::create(&repo2).unwrap();

        let repositories = find_repositories_recursive(temp_dir.path(), 2).unwrap();
        assert_eq!(repositories.len(), 2);
    }
}
