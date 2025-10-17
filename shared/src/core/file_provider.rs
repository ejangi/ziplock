//! File operation provider interface for ZipLock
//!
//! This module provides the trait and implementations for file operations,
//! allowing the memory repository to delegate file I/O to platform-specific
//! providers while maintaining clean separation of concerns.

use std::collections::HashMap;
use tracing::{debug, error, warn};

use crate::core::errors::{FileError, FileResult};
use crate::core::types::FileMap;

/// Trait for providing file operations to the repository manager
///
/// This trait abstracts all file I/O operations, allowing different platforms
/// to implement file handling in their preferred way while keeping the
/// memory repository pure.
pub trait FileOperationProvider: Send + Sync {
    /// Read an archive file from the filesystem
    ///
    /// # Arguments
    /// * `path` - Path to the archive file
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Archive file contents as bytes
    /// * `Err(FileError)` - If file cannot be read
    fn read_archive(&self, path: &str) -> FileResult<Vec<u8>>;

    /// Write archive data to the filesystem
    ///
    /// # Arguments
    /// * `path` - Path where to write the archive
    /// * `data` - Archive data as bytes
    ///
    /// # Returns
    /// * `Ok(())` - If write was successful
    /// * `Err(FileError)` - If file cannot be written
    fn write_archive(&self, path: &str, data: &[u8]) -> FileResult<()>;

    /// Extract archive contents to a file map
    ///
    /// This method should use platform-appropriate 7z libraries to extract
    /// the archive contents into memory without creating temporary files.
    /// Desktop platforms use sevenz-rust2, mobile platforms use native libraries.
    ///
    /// # Arguments
    /// * `data` - Encrypted archive data
    /// * `password` - Archive password for decryption
    ///
    /// # Returns
    /// * `Ok(FileMap)` - Extracted files as path->content map
    /// * `Err(FileError)` - If extraction fails
    fn extract_archive(&self, data: &[u8], password: &str) -> FileResult<FileMap>;

    /// Create an encrypted archive from a file map
    ///
    /// This method should use platform-appropriate 7z libraries to create
    /// an AES-256 encrypted archive from the file map contents.
    ///
    /// # Arguments
    /// * `files` - File map with path->content mappings
    /// * `password` - Password for AES-256 encryption
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Created archive as bytes
    /// * `Err(FileError)` - If archive creation fails
    fn create_archive(&self, files: FileMap, password: &str) -> FileResult<Vec<u8>>;
}

/// Desktop file provider using sevenz-rust2 for direct archive operations
#[derive(Debug, Default)]
pub struct DesktopFileProvider;

impl DesktopFileProvider {
    /// Create a new desktop file provider
    pub fn new() -> Self {
        Self
    }
}

impl FileOperationProvider for DesktopFileProvider {
    fn read_archive(&self, path: &str) -> FileResult<Vec<u8>> {
        std::fs::read(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound {
                path: path.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string(),
            },
            _ => FileError::IoError {
                message: format!("Failed to read archive '{}': {}", path, e),
            },
        })
    }

    fn write_archive(&self, path: &str, data: &[u8]) -> FileResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(FileError::IoError {
                        message: format!("Failed to create directory for '{}': {}", path, e),
                    });
                }
            }
        }

        std::fs::write(path, data).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                path: path.to_string(),
            },
            _ => FileError::IoError {
                message: format!("Failed to write archive '{}': {}", path, e),
            },
        })
    }

    fn extract_archive(&self, data: &[u8], password: &str) -> FileResult<FileMap> {
        debug!("Starting archive extraction: {} bytes", data.len());
        debug!(
            "Archive encryption: {}",
            if password.is_empty() {
                "disabled"
            } else {
                "enabled"
            }
        );

        // Write buffer to temporary file first since sevenz-rust2 API requires file paths
        let temp_archive =
            std::env::temp_dir().join(format!("ziplock_archive_{}.7z", uuid::Uuid::new_v4()));
        let temp_dir =
            std::env::temp_dir().join(format!("ziplock_extract_{}", uuid::Uuid::new_v4()));

        debug!("Temp archive path: {:?}", temp_archive);
        debug!("Temp extract dir: {:?}", temp_dir);

        // Write archive data to temp file
        std::fs::write(&temp_archive, data).map_err(|e| {
            error!(
                "Failed to write temp archive file {:?}: {}",
                temp_archive, e
            );
            FileError::ExtractionFailed {
                message: format!("Failed to write temp archive file: {}", e),
            }
        })?;

        debug!("Archive data written to temp file: {} bytes", data.len());

        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            error!("Failed to create temp directory {:?}: {}", temp_dir, e);
            FileError::ExtractionFailed {
                message: format!("Failed to create temp directory: {}", e),
            }
        })?;

        debug!("Temp extraction directory created: {:?}", temp_dir);

        // Use sevenz-rust2 helper functions
        let result = if password.is_empty() {
            debug!("Calling sevenz_rust2::decompress_file without password");
            sevenz_rust2::decompress_file(&temp_archive, &temp_dir)
        } else {
            debug!("Calling sevenz_rust2::decompress_file_with_password with password");
            sevenz_rust2::decompress_file_with_password(&temp_archive, &temp_dir, password.into())
        };

        match result {
            Ok(()) => {
                debug!("Archive extraction successful, reading extracted files");

                // Read extracted files into memory
                let mut file_map = HashMap::new();

                fn read_dir_recursive(
                    dir: &std::path::Path,
                    base_path: &std::path::Path,
                    file_map: &mut HashMap<String, Vec<u8>>,
                ) -> std::io::Result<()> {
                    for entry in std::fs::read_dir(dir)? {
                        let entry = entry?;
                        let path = entry.path();

                        if path.is_file() {
                            let relative_path = path
                                .strip_prefix(base_path)
                                .map_err(|_| std::io::Error::other("Path error"))?;
                            let content = std::fs::read(&path)?;
                            let relative_path_str = relative_path.to_string_lossy().to_string();
                            debug!(
                                "Extracted file: {} ({} bytes)",
                                relative_path_str,
                                content.len()
                            );
                            file_map.insert(relative_path_str, content);
                        } else if path.is_dir() {
                            debug!("Recursing into directory: {:?}", path);
                            read_dir_recursive(&path, base_path, file_map)?;
                        }
                    }
                    Ok(())
                }

                read_dir_recursive(&temp_dir, &temp_dir, &mut file_map).map_err(|e| {
                    error!("Failed to read extracted files from {:?}: {}", temp_dir, e);
                    FileError::ExtractionFailed {
                        message: format!("Failed to read extracted files: {}", e),
                    }
                })?;

                debug!(
                    "Successfully extracted {} files from archive",
                    file_map.len()
                );
                for (path, content) in &file_map {
                    debug!("  - {}: {} bytes", path, content.len());
                }

                // Clean up temp files
                debug!("Cleaning up temporary files");
                if let Err(e) = std::fs::remove_file(&temp_archive) {
                    warn!("Failed to remove temp archive {:?}: {}", temp_archive, e);
                }
                if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                    warn!("Failed to remove temp directory {:?}: {}", temp_dir, e);
                }

                debug!("Archive extraction completed successfully");
                Ok(file_map)
            }
            Err(e) => {
                error!("Archive extraction failed: {}", e);

                // Log directory contents for debugging
                debug!("Checking temp directory contents after extraction failure:");
                if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                    let mut file_count = 0;
                    for entry in entries {
                        if let Ok(entry) = entry {
                            debug!("  - {:?}", entry.path());
                            file_count += 1;
                        }
                    }
                    debug!(
                        "Found {} items in temp directory after failed extraction",
                        file_count
                    );
                } else {
                    debug!("Could not read temp directory contents for debugging");
                }

                // Clean up temp files on error
                let _ = std::fs::remove_file(&temp_archive);
                let _ = std::fs::remove_dir_all(&temp_dir);

                // Check for password-related errors
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("password")
                    || error_str.contains("wrong")
                    || error_str.contains("decrypt")
                {
                    error!("Archive extraction failed due to password issue");
                    Err(FileError::InvalidPassword)
                } else {
                    error!("Archive extraction failed due to format/corruption issue");
                    Err(FileError::ExtractionFailed {
                        message: format!("Failed to extract 7z archive: {}", e),
                    })
                }
            }
        }
    }

    fn create_archive(&self, files: FileMap, password: &str) -> FileResult<Vec<u8>> {
        // Create temporary directory to write files
        let temp_dir =
            std::env::temp_dir().join(format!("ziplock_create_{}", uuid::Uuid::new_v4()));

        debug!(
            "Creating temp directory for archive creation: {:?}",
            temp_dir
        );
        debug!("Files to be archived: {} files", files.len());
        for (path, content) in &files {
            debug!("  - File: {} ({} bytes)", path, content.len());
        }

        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            error!("Failed to create temp directory {:?}: {}", temp_dir, e);
            FileError::CreationFailed {
                message: format!("Failed to create temp directory: {}", e),
            }
        })?;

        debug!("Temp directory created successfully: {:?}", temp_dir);

        // Write files to temporary directory with Windows path separator fix
        let mut files_written = 0;
        for (path, content) in files {
            // Simple Windows path fix: convert forward slashes to backslashes on Windows
            let normalized_path = if cfg!(windows) {
                path.replace('/', "\\")
            } else {
                path.clone()
            };

            let file_path = temp_dir.join(&normalized_path);
            debug!(
                "Writing file: {} -> {:?} ({} bytes)",
                path,
                file_path,
                content.len()
            );

            // Create parent directory
            if let Some(parent) = file_path.parent() {
                debug!("Creating parent directory: {:?}", parent);
                std::fs::create_dir_all(parent).map_err(|e| FileError::CreationFailed {
                    message: format!("Failed to create directory structure: {}", e),
                })?;
                debug!("Parent directory created successfully: {:?}", parent);
            }

            // Write file
            std::fs::write(&file_path, &content).map_err(|e| FileError::CreationFailed {
                message: format!("Failed to write file '{}': {}", path, e),
            })?;

            // Verify file was written correctly
            let written_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
            debug!(
                "File written successfully: {} ({} bytes on disk)",
                path, written_size
            );

            if written_size != content.len() as u64 {
                warn!(
                    "Size mismatch for file {}: expected {} bytes, found {} bytes",
                    path,
                    content.len(),
                    written_size
                );
            }

            files_written += 1;
        }

        debug!(
            "Successfully wrote {} files to temp directory",
            files_written
        );

        // Verify directory contents before archiving
        debug!("Verifying temp directory contents before archiving:");
        match std::fs::read_dir(&temp_dir) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_dir() {
                                debug!("  DIR:  {:?}", path);
                            } else {
                                debug!("  FILE: {:?} ({} bytes)", path, metadata.len());
                            }
                        } else {
                            warn!("Could not read metadata for {:?}", path);
                            debug!("  UNKNOWN: {:?}", path);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read temp directory contents: {}", e);
            }
        }

        // Create archive from temporary directory
        let temp_archive = temp_dir.with_extension("7z");
        debug!(
            "Creating archive: {:?} from directory: {:?}",
            temp_archive, temp_dir
        );

        let has_password = !password.is_empty();
        debug!(
            "Archive encryption: {}",
            if has_password { "enabled" } else { "disabled" }
        );

        let result = if password.is_empty() {
            debug!("Calling sevenz_rust2::compress_to_path without password");
            sevenz_rust2::compress_to_path(&temp_dir, &temp_archive)
        } else {
            debug!("Calling sevenz_rust2::compress_to_path_encrypted with password");
            sevenz_rust2::compress_to_path_encrypted(&temp_dir, &temp_archive, password.into())
        };

        match result {
            Ok(()) => {
                debug!("Archive creation successful, verifying archive file");

                // Verify archive was created
                let archive_metadata = std::fs::metadata(&temp_archive).map_err(|e| {
                    error!("Archive file not found after creation: {}", e);
                    FileError::CreationFailed {
                        message: format!("Archive file not found after creation: {}", e),
                    }
                })?;

                debug!(
                    "Archive created successfully: {:?} ({} bytes)",
                    temp_archive,
                    archive_metadata.len()
                );

                // Read the created archive into memory
                let archive_data = std::fs::read(&temp_archive).map_err(|e| {
                    error!("Failed to read created archive {:?}: {}", temp_archive, e);
                    FileError::CreationFailed {
                        message: format!("Failed to read created archive: {}", e),
                    }
                })?;

                debug!(
                    "Archive data read into memory: {} bytes",
                    archive_data.len()
                );

                // Clean up temporary files
                debug!("Cleaning up temporary files");
                if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                    warn!("Failed to remove temp directory {:?}: {}", temp_dir, e);
                }
                if let Err(e) = std::fs::remove_file(&temp_archive) {
                    warn!("Failed to remove temp archive {:?}: {}", temp_archive, e);
                }

                debug!("Archive creation completed successfully");
                Ok(archive_data)
            }
            Err(e) => {
                error!("Archive creation failed: {}", e);

                // Log directory contents for debugging
                debug!("Directory contents at failure:");
                if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            debug!("  - {:?}", entry.path());
                        }
                    }
                } else {
                    debug!("Could not read temp directory for debugging");
                }

                // Clean up temporary files on error
                let _ = std::fs::remove_dir_all(&temp_dir);
                let _ = std::fs::remove_file(&temp_archive);

                Err(FileError::CreationFailed {
                    message: format!("Failed to create 7z archive: {}", e),
                })
            }
        }
    }
}

/// Mock file provider for testing
#[derive(Debug, Clone)]
pub struct MockFileProvider {
    /// Simulated archive files (path -> data)
    pub archives: HashMap<String, Vec<u8>>,
    /// Whether operations should fail
    pub should_fail: bool,
    /// Simulated file maps for extraction
    pub file_maps: HashMap<String, FileMap>,
}

impl MockFileProvider {
    /// Create a new mock file provider
    pub fn new() -> Self {
        Self {
            archives: HashMap::new(),
            should_fail: false,
            file_maps: HashMap::new(),
        }
    }

    /// Create a mock provider that fails operations
    pub fn with_failure() -> Self {
        Self {
            archives: HashMap::new(),
            should_fail: true,
            file_maps: HashMap::new(),
        }
    }

    /// Add a mock archive file
    pub fn add_archive<P: Into<String>>(&mut self, path: P, data: Vec<u8>) {
        self.archives.insert(path.into(), data);
    }

    /// Add a mock file map for extraction
    pub fn add_file_map<P: Into<String>>(&mut self, path: P, file_map: FileMap) {
        self.file_maps.insert(path.into(), file_map);
    }
}

impl Default for MockFileProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperationProvider for MockFileProvider {
    fn read_archive(&self, path: &str) -> FileResult<Vec<u8>> {
        if self.should_fail {
            return Err(FileError::NotFound {
                path: path.to_string(),
            });
        }

        self.archives
            .get(path)
            .cloned()
            .ok_or_else(|| FileError::NotFound {
                path: path.to_string(),
            })
    }

    fn write_archive(&self, path: &str, _data: &[u8]) -> FileResult<()> {
        if self.should_fail {
            return Err(FileError::PermissionDenied {
                path: path.to_string(),
            });
        }

        // In a real implementation, we'd store this, but for mock we just succeed
        Ok(())
    }

    fn extract_archive(&self, _data: &[u8], _password: &str) -> FileResult<FileMap> {
        if self.should_fail {
            return Err(FileError::InvalidPassword);
        }

        // Return a simple mock file map
        let mut file_map = HashMap::new();
        file_map.insert("metadata.yml".to_string(), b"version: 1.0".to_vec());
        file_map.insert(
            "credentials/test-id/record.yml".to_string(),
            b"id: test-id\ntitle: Test".to_vec(),
        );

        Ok(file_map)
    }

    fn create_archive(&self, _files: FileMap, _password: &str) -> FileResult<Vec<u8>> {
        if self.should_fail {
            return Err(FileError::CreationFailed {
                message: "Mock failure".to_string(),
            });
        }

        // Return some mock archive data
        Ok(vec![0x50, 0x4b, 0x03, 0x04]) // Mock zip signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_file_provider() {
        let mut provider = MockFileProvider::new();
        provider.add_archive("/test.7z", vec![1, 2, 3, 4]);

        // Test read
        let data = provider.read_archive("/test.7z").unwrap();
        assert_eq!(data, vec![1, 2, 3, 4]);

        // Test read non-existent
        assert!(provider.read_archive("/missing.7z").is_err());

        // Test write
        assert!(provider.write_archive("/output.7z", &[5, 6, 7]).is_ok());

        // Test extract
        let file_map = provider.extract_archive(&[1, 2, 3], "password").unwrap();
        assert!(file_map.contains_key("metadata.yml"));

        // Test create
        let archive_data = provider.create_archive(HashMap::new(), "password").unwrap();
        assert!(!archive_data.is_empty());
    }

    #[test]
    fn test_mock_failure_mode() {
        let provider = MockFileProvider::with_failure();

        assert!(provider.read_archive("/test.7z").is_err());
        assert!(provider.write_archive("/test.7z", &[1, 2, 3]).is_err());
        assert!(provider.extract_archive(&[1, 2, 3], "password").is_err());
        assert!(provider.create_archive(HashMap::new(), "password").is_err());
    }

    #[test]
    fn test_desktop_file_provider_creation() {
        let provider = DesktopFileProvider::new();

        // Test that we can create the provider (actual file operations would need real files)
        assert!(std::mem::size_of_val(&provider) == 0); // Zero-sized type
    }

    // Note: Full desktop provider tests would require setting up test files
    // and would be integration tests rather than unit tests
}
