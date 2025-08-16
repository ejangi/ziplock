//! Archive management module for ZipLock shared library
//!
//! This module provides the core archive management functionality that was
//! previously in the backend. It handles creating, opening, reading, and
//! writing to encrypted 7z archives containing credential data.

pub mod cloud_storage;
pub mod file_lock;
pub mod manager;
pub mod validation;

// Re-export commonly used types
pub use cloud_storage::{is_cloud_storage_path, CloudFileHandle, CloudStorageError};
pub use file_lock::{FileLock, FileLockError};
pub use manager::ArchiveManager;
pub use validation::{RepositoryValidator, ValidationIssue, ValidationReport};

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during archive operations
#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("Archive not found: {path}")]
    NotFound { path: String },

    #[error("Archive is corrupted or invalid: {reason}")]
    Corrupted { reason: String },

    #[error("Failed to create archive: {reason}")]
    CreationFailed { reason: String },

    #[error("Failed to open archive: {reason}")]
    OpenFailed { reason: String },

    #[error("Failed to extract from archive: {reason}")]
    ExtractFailed { reason: String },

    #[error("Failed to add to archive: {reason}")]
    AddFailed { reason: String },

    #[error("File lock acquisition failed: {path}")]
    LockFailed { path: String },

    #[error("File lock timeout: {path}")]
    LockTimeout { path: String },

    #[error("Credential record not found: {id}")]
    RecordNotFound { id: String },

    #[error("Record validation failed: {reason}")]
    InvalidRecord { reason: String },

    #[error("Backup creation failed: {reason}")]
    BackupFailed { reason: String },

    #[error("Archive format version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Cryptographic operation failed: {reason}")]
    CryptoError { reason: String },

    #[error("IO operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type for archive operations
pub type ArchiveResult<T> = Result<T, ArchiveError>;

/// Configuration for archive operations
#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    /// Default directory for archives
    pub default_archive_dir: Option<PathBuf>,

    /// Maximum archive size in MB
    pub max_archive_size_mb: u64,

    /// Number of backups to keep
    pub backup_count: u32,

    /// Enable automatic backups
    pub auto_backup: bool,

    /// Backup directory
    pub backup_dir: Option<PathBuf>,

    /// File lock timeout in seconds
    pub file_lock_timeout: u64,

    /// Temporary directory for operations
    pub temp_dir: Option<PathBuf>,

    /// Verify archive integrity on operations
    pub verify_integrity: bool,

    /// Minimum password length
    pub min_password_length: usize,

    /// Compression configuration
    pub compression: CompressionConfig,

    /// Validation configuration
    pub validation: ValidationConfig,
}

/// Compression settings for archives
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression level (0-9)
    pub level: u8,

    /// Use solid compression
    pub solid: bool,

    /// Enable multi-threaded compression
    pub multi_threaded: bool,

    /// Dictionary size in MB
    pub dictionary_size_mb: u32,

    /// Block size in MB
    pub block_size_mb: u32,
}

/// Validation settings for archives
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable comprehensive validation
    pub enable_comprehensive_validation: bool,

    /// Perform deep validation checks
    pub deep_validation: bool,

    /// Check for legacy format compatibility
    pub check_legacy_formats: bool,

    /// Validate data schemas
    pub validate_schemas: bool,

    /// Auto-repair minor issues
    pub auto_repair: bool,

    /// Fail on critical validation issues
    pub fail_on_critical_issues: bool,

    /// Log detailed validation information
    pub log_validation_details: bool,
}

impl From<crate::SharedError> for ArchiveError {
    fn from(error: crate::SharedError) -> Self {
        ArchiveError::Internal {
            message: error.to_string(),
        }
    }
}

impl From<crate::api::ValidationError> for ArchiveError {
    fn from(error: crate::api::ValidationError) -> Self {
        ArchiveError::InvalidRecord {
            reason: error.to_string(),
        }
    }
}

impl From<anyhow::Error> for ArchiveError {
    fn from(error: anyhow::Error) -> Self {
        ArchiveError::Internal {
            message: error.to_string(),
        }
    }
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            default_archive_dir: None,
            max_archive_size_mb: 1000, // 1GB
            backup_count: 3,
            auto_backup: true,
            backup_dir: None,
            file_lock_timeout: 30,
            temp_dir: None,
            verify_integrity: true,
            min_password_length: 12,
            compression: CompressionConfig::default(),
            validation: ValidationConfig::default(),
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 5,
            solid: true,
            multi_threaded: true,
            dictionary_size_mb: 32,
            block_size_mb: 16,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_comprehensive_validation: true,
            deep_validation: false,
            check_legacy_formats: true,
            validate_schemas: true,
            auto_repair: false,
            fail_on_critical_issues: true,
            log_validation_details: false,
        }
    }
}

/// Constants for archive operations
pub mod constants {
    /// Current archive format version
    pub const ARCHIVE_FORMAT_VERSION: &str = "1.0";

    /// Metadata file name in archives
    pub const METADATA_FILE: &str = ".ziplock_metadata.yaml";

    /// Credentials directory in archives
    pub const CREDENTIALS_DIR: &str = "credentials";

    /// Backup file prefix
    pub const BACKUP_PREFIX: &str = "ziplock_backup_";

    /// Temporary file prefix
    pub const TEMP_PREFIX: &str = "ziplock_temp_";

    /// Default file extension for archives
    pub const ARCHIVE_EXTENSION: &str = "7z";

    /// Lock file extension
    pub const LOCK_EXTENSION: &str = "lock";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_config_default() {
        let config = ArchiveConfig::default();
        assert_eq!(config.max_archive_size_mb, 1000);
        assert_eq!(config.backup_count, 3);
        assert!(config.auto_backup);
        assert!(config.verify_integrity);
    }

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.level, 5);
        assert!(config.solid);
        assert!(config.multi_threaded);
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(config.enable_comprehensive_validation);
        assert!(!config.deep_validation);
        assert!(config.validate_schemas);
    }

    #[test]
    fn test_archive_error_display() {
        let error = ArchiveError::NotFound {
            path: "/path/to/archive.7z".to_string(),
        };
        assert!(error.to_string().contains("not found"));
        assert!(error.to_string().contains("/path/to/archive.7z"));
    }
}
