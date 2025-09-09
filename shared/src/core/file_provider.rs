//! File operation provider interface for ZipLock
//!
//! This module provides the trait and implementations for file operations,
//! allowing the memory repository to delegate file I/O to platform-specific
//! providers while maintaining clean separation of concerns.

use std::collections::HashMap;

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
        // Write buffer to temporary file first since sevenz-rust2 API requires file paths
        let temp_archive =
            std::env::temp_dir().join(format!("ziplock_archive_{}.7z", uuid::Uuid::new_v4()));
        let temp_dir =
            std::env::temp_dir().join(format!("ziplock_extract_{}", uuid::Uuid::new_v4()));

        // Write archive data to temp file
        std::fs::write(&temp_archive, data).map_err(|e| FileError::ExtractionFailed {
            message: format!("Failed to write temp archive file: {}", e),
        })?;

        std::fs::create_dir_all(&temp_dir).map_err(|e| FileError::ExtractionFailed {
            message: format!("Failed to create temp directory: {}", e),
        })?;

        // Use sevenz-rust2 helper functions
        let result = if password.is_empty() {
            sevenz_rust2::decompress_file(&temp_archive, &temp_dir)
        } else {
            sevenz_rust2::decompress_file_with_password(&temp_archive, &temp_dir, password.into())
        };

        match result {
            Ok(()) => {
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
                            let relative_path = path.strip_prefix(base_path).map_err(|_| {
                                std::io::Error::new(std::io::ErrorKind::Other, "Path error")
                            })?;
                            let content = std::fs::read(&path)?;
                            file_map.insert(relative_path.to_string_lossy().to_string(), content);
                        } else if path.is_dir() {
                            read_dir_recursive(&path, base_path, file_map)?;
                        }
                    }
                    Ok(())
                }

                read_dir_recursive(&temp_dir, &temp_dir, &mut file_map).map_err(|e| {
                    FileError::ExtractionFailed {
                        message: format!("Failed to read extracted files: {}", e),
                    }
                })?;

                // Clean up temp files
                let _ = std::fs::remove_file(&temp_archive);
                let _ = std::fs::remove_dir_all(&temp_dir);

                Ok(file_map)
            }
            Err(e) => {
                // Clean up temp files on error
                let _ = std::fs::remove_file(&temp_archive);
                let _ = std::fs::remove_dir_all(&temp_dir);

                // Check for password-related errors
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("password")
                    || error_str.contains("wrong")
                    || error_str.contains("decrypt")
                {
                    Err(FileError::InvalidPassword)
                } else {
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
        std::fs::create_dir_all(&temp_dir).map_err(|e| FileError::CreationFailed {
            message: format!("Failed to create temp directory: {}", e),
        })?;

        // Write files to temporary directory
        for (path, content) in files {
            let file_path = temp_dir.join(&path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| FileError::CreationFailed {
                    message: format!("Failed to create directory structure: {}", e),
                })?;
            }
            std::fs::write(&file_path, content).map_err(|e| FileError::CreationFailed {
                message: format!("Failed to write file '{}': {}", path, e),
            })?;
        }

        // Create archive from temporary directory
        let temp_archive = temp_dir.with_extension("7z");

        let result = if password.is_empty() {
            sevenz_rust2::compress_to_path(&temp_dir, &temp_archive)
        } else {
            sevenz_rust2::compress_to_path_encrypted(&temp_dir, &temp_archive, password.into())
        };

        match result {
            Ok(()) => {
                // Read the created archive into memory
                let archive_data =
                    std::fs::read(&temp_archive).map_err(|e| FileError::CreationFailed {
                        message: format!("Failed to read created archive: {}", e),
                    })?;

                // Clean up temporary files
                let _ = std::fs::remove_dir_all(&temp_dir);
                let _ = std::fs::remove_file(&temp_archive);

                Ok(archive_data)
            }
            Err(e) => {
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
