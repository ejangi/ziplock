//! Cloud storage file handling utilities
//!
//! This module provides enhanced file handling for cloud storage scenarios,
//! including detection of cloud-synced files and copy-to-local strategies
//! to prevent sync conflicts during archive operations.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use tracing::{debug, info, warn};

use super::file_lock::{FileLockError, LockFile};

#[derive(Error, Debug)]
pub enum CloudStorageError {
    #[error("Failed to copy cloud file to local storage: {reason}")]
    CopyFailed { reason: String },

    #[error("Content hash mismatch - file was modified externally")]
    ContentModified,

    #[error("Failed to create temporary directory: {0}")]
    TempDirFailed(#[from] std::io::Error),

    #[error("File lock error: {0}")]
    LockError(#[from] FileLockError),

    #[error("Hash calculation failed: {reason}")]
    HashFailed { reason: String },
}

/// A handle for working with files that may be stored in cloud storage
#[derive(Debug)]
pub struct CloudFileHandle {
    /// Original file path (may be cloud storage)
    original_path: PathBuf,
    /// Local working copy path (always local)
    local_path: PathBuf,
    /// File lock on the local copy
    _lock_file: LockFile,
    /// Hash of original content for conflict detection
    original_hash: String,
    /// Whether this is a cloud storage file that needs sync back
    is_cloud_file: bool,
    /// Whether the file was modified and needs sync back
    needs_sync_back: bool,
}

impl CloudFileHandle {
    /// Create a new cloud file handle
    /// Automatically detects cloud storage and copies to local storage if needed
    pub fn new<P: AsRef<Path>>(
        path: P,
        timeout_seconds: Option<u64>,
    ) -> Result<Self, CloudStorageError> {
        let original_path = path.as_ref().to_path_buf();
        let timeout = timeout_seconds.unwrap_or(60); // Default 60 seconds for cloud operations

        let is_cloud = is_cloud_storage_path(&original_path);

        if is_cloud {
            Self::create_from_cloud_file(original_path, timeout)
        } else {
            Self::create_from_local_file(original_path, timeout)
        }
    }

    /// Create handle from a cloud storage file (copies to local)
    fn create_from_cloud_file(
        cloud_path: PathBuf,
        timeout: u64,
    ) -> Result<Self, CloudStorageError> {
        info!("Creating cloud file handle for: {:?}", cloud_path);

        // Calculate original hash before copying
        let original_hash = calculate_file_hash(&cloud_path)?;

        // Create local working directory
        let local_dir = create_temp_working_dir()?;
        let filename = cloud_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive.7z");
        let local_path = local_dir.join(filename);

        // Copy cloud file to local storage
        fs::copy(&cloud_path, &local_path).map_err(|e| CloudStorageError::CopyFailed {
            reason: format!("Failed to copy {:?} to {:?}: {}", cloud_path, local_path, e),
        })?;

        // Create lock on local copy
        let lock_file = LockFile::create(&local_path, timeout)?;

        info!(
            "Copied cloud file to local storage: {:?} -> {:?}",
            cloud_path, local_path
        );

        Ok(Self {
            original_path: cloud_path,
            local_path,
            _lock_file: lock_file,
            original_hash,
            is_cloud_file: true,
            needs_sync_back: false,
        })
    }

    /// Create handle from a local file (no copying needed)
    fn create_from_local_file(
        local_path: PathBuf,
        timeout: u64,
    ) -> Result<Self, CloudStorageError> {
        debug!("Creating local file handle for: {:?}", local_path);

        let original_hash = calculate_file_hash(&local_path)?;
        let lock_file = LockFile::create(&local_path, timeout)?;

        Ok(Self {
            original_path: local_path.clone(),
            local_path,
            _lock_file: lock_file,
            original_hash,
            is_cloud_file: false,
            needs_sync_back: false,
        })
    }

    /// Get the local working path (always safe to use for file operations)
    pub fn local_path(&self) -> &Path {
        &self.local_path
    }

    /// Get the original path (may be cloud storage)
    pub fn original_path(&self) -> &Path {
        &self.original_path
    }

    /// Check if this is a cloud storage file
    pub fn is_cloud_file(&self) -> bool {
        self.is_cloud_file
    }

    /// Mark that the file has been modified and needs sync back
    pub fn mark_modified(&mut self) {
        self.needs_sync_back = true;
    }

    /// Verify the original file hasn't been modified by external sync
    pub fn verify_no_conflicts(&self) -> Result<(), CloudStorageError> {
        if !self.is_cloud_file {
            return Ok(());
        }

        if !self.original_path.exists() {
            warn!(
                "Original cloud file no longer exists: {:?}",
                self.original_path
            );
            return Ok(()); // File was deleted, no conflict
        }

        let current_hash = calculate_file_hash(&self.original_path)?;
        if current_hash != self.original_hash {
            return Err(CloudStorageError::ContentModified);
        }

        Ok(())
    }

    /// Sync changes back to the original location if needed
    pub fn sync_back(&mut self) -> Result<(), CloudStorageError> {
        if !self.needs_sync_back || !self.is_cloud_file {
            debug!("No sync back needed for: {:?}", self.original_path);
            return Ok(());
        }

        info!(
            "Syncing changes back to cloud storage: {:?}",
            self.original_path
        );

        // Copy local changes back to original location
        fs::copy(&self.local_path, &self.original_path).map_err(|e| {
            CloudStorageError::CopyFailed {
                reason: format!("Failed to sync back to {:?}: {}", self.original_path, e),
            }
        })?;

        self.needs_sync_back = false;
        info!(
            "Successfully synced changes back to: {:?}",
            self.original_path
        );

        Ok(())
    }
}

impl Drop for CloudFileHandle {
    fn drop(&mut self) {
        // Attempt to sync back if needed
        if let Err(e) = self.sync_back() {
            warn!("Failed to sync changes back on drop: {}", e);
        }

        // Clean up local working copy if it's a cloud file
        if self.is_cloud_file {
            if let Some(parent) = self.local_path.parent() {
                if let Err(e) = fs::remove_dir_all(parent) {
                    debug!("Failed to clean up temp directory {:?}: {}", parent, e);
                }
            }
        }
    }
}

/// Detect if a file path appears to be from cloud storage
pub fn is_cloud_storage_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Android cloud storage patterns
    path_str.contains("/android/data/com.google.android.apps.docs") ||  // Google Drive
    path_str.contains("/android/data/com.dropbox.android") ||           // Dropbox
    path_str.contains("/android/data/com.microsoft.skydrive") ||        // OneDrive
    path_str.contains("/android/data/com.box.android") ||               // Box
    path_str.contains("/android/data/com.amazon.clouddrive") ||         // Amazon Drive
    path_str.contains("/android/data/com.nextcloud.client") ||          // Nextcloud

    // Storage Access Framework patterns
    path_str.starts_with("content://") ||

    // Generic cloud storage indicators in common locations
    (path_str.contains("/cloud/") ||
     path_str.contains("/sync/") ||
     path_str.contains("/googledrive/") ||
     path_str.contains("/dropbox/") ||
     path_str.contains("/onedrive/") ||
     path_str.contains("/box sync/") ||
     path_str.contains("/nextcloud/")) ||

    // Temporary cache directories that might be cloud-synced
    (path_str.contains("/cache/") && (
        path_str.contains("drive") ||
        path_str.contains("dropbox") ||
        path_str.contains("onedrive") ||
        path_str.contains("cloud") ||
        path_str.contains("sync")
    )) ||

    // Windows cloud storage patterns
    path_str.contains("\\onedrive\\") ||
    path_str.contains("\\google drive\\") ||
    path_str.contains("\\dropbox\\") ||
    path_str.contains("\\box\\") ||

    // macOS cloud storage patterns
    path_str.contains("/google drive/") ||
    path_str.contains("/dropbox/") ||
    path_str.contains("/onedrive/") ||
    path_str.contains("/icloud/")
}

/// Calculate a simple hash of file content for conflict detection
fn calculate_file_hash(path: &Path) -> Result<String, CloudStorageError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let metadata = fs::metadata(path).map_err(|e| CloudStorageError::HashFailed {
        reason: format!("Failed to read metadata for {:?}: {}", path, e),
    })?;

    let mut hasher = DefaultHasher::new();

    // Hash file size and modification time for lightweight conflict detection
    metadata.len().hash(&mut hasher);
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
            duration.as_secs().hash(&mut hasher);
        }
    }

    // For small files, also hash first and last 1KB of content
    if metadata.len() < 1024 * 1024 {
        // Files smaller than 1MB
        if let Ok(content) = fs::read(path) {
            content.hash(&mut hasher);
        }
    } else {
        // For larger files, hash first and last 1KB
        if let Ok(mut file) = fs::File::open(path) {
            use std::io::{Read, Seek, SeekFrom};

            let mut buffer = [0u8; 1024];
            if file.read(&mut buffer).is_ok() {
                buffer.hash(&mut hasher);
            }

            if file.seek(SeekFrom::End(-1024)).is_ok() && file.read(&mut buffer).is_ok() {
                buffer.hash(&mut hasher);
            }
        }
    }

    Ok(format!("{:x}", hasher.finish()))
}

/// Create a temporary working directory for cloud file operations
fn create_temp_working_dir() -> Result<PathBuf, CloudStorageError> {
    let temp_dir = std::env::temp_dir().join("ziplock_cloud_working");

    // Create unique subdirectory
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let working_dir = temp_dir.join(format!("session_{}", timestamp));

    fs::create_dir_all(&working_dir)?;

    debug!("Created cloud working directory: {:?}", working_dir);
    Ok(working_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cloud_storage_detection() {
        // Android patterns
        assert!(is_cloud_storage_path(Path::new(
            "/Android/data/com.google.android.apps.docs/files/test.7z"
        )));
        assert!(is_cloud_storage_path(Path::new(
            "/Android/data/com.dropbox.android/cache/test.7z"
        )));

        // Storage Access Framework
        assert!(is_cloud_storage_path(Path::new(
            "content://com.android.providers.media.documents/document/test"
        )));

        // Windows patterns
        assert!(is_cloud_storage_path(Path::new(
            "C:\\Users\\test\\OneDrive\\test.7z"
        )));
        assert!(is_cloud_storage_path(Path::new(
            "C:\\Users\\test\\Google Drive\\test.7z"
        )));

        // Generic patterns
        assert!(is_cloud_storage_path(Path::new("/home/user/cloud/test.7z")));
        assert!(is_cloud_storage_path(Path::new(
            "/tmp/cache/drive_temp/test.7z"
        )));

        // Should not detect normal local paths
        assert!(!is_cloud_storage_path(Path::new(
            "/home/user/documents/test.7z"
        )));
        assert!(!is_cloud_storage_path(Path::new("/tmp/test.7z")));
        assert!(!is_cloud_storage_path(Path::new(
            "C:\\Users\\test\\Documents\\test.7z"
        )));
    }

    #[test]
    fn test_local_file_handle() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write some content
        std::fs::write(path, b"test content").unwrap();

        let handle = CloudFileHandle::new(path, Some(5)).unwrap();

        assert!(!handle.is_cloud_file());
        assert_eq!(handle.local_path(), handle.original_path());
        assert_eq!(handle.local_path(), path);
    }

    #[test]
    fn test_file_hash_calculation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write some content
        std::fs::write(path, b"test content").unwrap();

        let hash1 = calculate_file_hash(path).unwrap();
        let hash2 = calculate_file_hash(path).unwrap();

        // Same file should produce same hash
        assert_eq!(hash1, hash2);

        // Modify content
        std::fs::write(path, b"different content").unwrap();
        let hash3 = calculate_file_hash(path).unwrap();

        // Different content should produce different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_conflict_detection() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        std::fs::write(path, b"original content").unwrap();

        // Create a cloud file handle by using a cloud storage path pattern
        let _cloud_path = Path::new("/Android/data/com.google.android.apps.docs/files/test.7z");

        // Copy temp file to a location that looks like cloud storage for testing
        let temp_dir = std::env::temp_dir().join("ziplock_test_cloud");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let test_cloud_file = temp_dir.join("test.7z");
        std::fs::copy(path, &test_cloud_file).unwrap();

        // For this test, we'll test the hash calculation directly since
        // CloudFileHandle would copy cloud files to local storage
        let original_hash = calculate_file_hash(&test_cloud_file).unwrap();

        // Modify the file externally
        std::fs::write(&test_cloud_file, b"externally modified content").unwrap();
        let new_hash = calculate_file_hash(&test_cloud_file).unwrap();

        // Hashes should be different
        assert_ne!(original_hash, new_hash);

        // Clean up
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
