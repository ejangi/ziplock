//! Core types and constants for the ZipLock unified architecture.
//!
//! This module defines shared data structures and constants used throughout
//! the shared library, providing a common foundation for all operations.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repository metadata containing version and structural information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryMetadata {
    /// Repository format version
    pub version: String,

    /// Repository format identifier
    pub format: String,

    /// Timestamp when repository was created
    pub created_at: i64,

    /// Timestamp when repository was last modified
    pub last_modified: i64,

    /// Number of credentials in repository
    pub credential_count: usize,

    /// Structure version for compatibility
    pub structure_version: String,

    /// Generator identifier
    pub generator: String,
}

impl Default for RepositoryMetadata {
    fn default() -> Self {
        let now = Utc::now().timestamp();
        Self {
            version: "1.0".to_string(),
            format: "memory-v1".to_string(),
            created_at: now,
            last_modified: now,
            credential_count: 0,
            structure_version: "1.0".to_string(),
            generator: "ziplock-unified".to_string(),
        }
    }
}

/// File map type for representing extracted archive contents
/// Maps file paths to their byte content
pub type FileMap = HashMap<String, Vec<u8>>;

/// Repository statistics for monitoring and display
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RepositoryStats {
    /// Number of credentials stored
    pub credential_count: usize,

    /// Repository metadata
    pub metadata: RepositoryMetadata,

    /// Whether repository is initialized
    pub initialized: bool,

    /// Whether repository has unsaved changes
    pub modified: bool,
}

/// Constants for repository structure
pub const METADATA_FILE: &str = "metadata.yml";
pub const CREDENTIALS_INDEX_FILE: &str = "credentials/index.yml";
pub const CREDENTIALS_DIR: &str = "credentials";
pub const ATTACHMENTS_DIR: &str = "attachments";

/// Repository format constants
pub const CURRENT_VERSION: &str = "1.0";
pub const CURRENT_FORMAT: &str = "memory-v1";
pub const CURRENT_STRUCTURE_VERSION: &str = "1.0";
pub const GENERATOR_NAME: &str = "ziplock-unified";

/// Maximum field value length to prevent memory issues
pub const MAX_FIELD_VALUE_LENGTH: usize = 10_000;

/// Maximum number of fields per credential
pub const MAX_FIELDS_PER_CREDENTIAL: usize = 50;

/// Maximum credential title length
pub const MAX_TITLE_LENGTH: usize = 200;

/// Maximum notes length
pub const MAX_NOTES_LENGTH: usize = 10_000;

/// Maximum tag length
pub const MAX_TAG_LENGTH: usize = 50;

/// Maximum number of tags per credential
pub const MAX_TAGS_PER_CREDENTIAL: usize = 10;

/// TOTP constants
pub const DEFAULT_TOTP_PERIOD: u32 = 30;
pub const DEFAULT_TOTP_DIGITS: usize = 6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_metadata_default() {
        let metadata = RepositoryMetadata::default();
        assert_eq!(metadata.version, "1.0");
        assert_eq!(metadata.format, "memory-v1");
        assert_eq!(metadata.credential_count, 0);
        assert!(metadata.created_at > 0);
        assert!(metadata.last_modified > 0);
    }

    #[test]
    fn test_repository_metadata_serialization() {
        let metadata = RepositoryMetadata::default();
        let yaml = serde_yaml::to_string(&metadata).unwrap();
        let deserialized: RepositoryMetadata = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(metadata, deserialized);
    }

    #[test]
    fn test_repository_stats() {
        let metadata = RepositoryMetadata::default();
        let stats = RepositoryStats {
            credential_count: 5,
            metadata,
            initialized: true,
            modified: false,
        };

        assert_eq!(stats.credential_count, 5);
        assert!(stats.initialized);
        assert!(!stats.modified);
    }

    #[test]
    fn test_constants() {
        assert_eq!(METADATA_FILE, "metadata.yml");
        assert_eq!(CREDENTIALS_DIR, "credentials");
        assert_eq!(CURRENT_VERSION, "1.0");
        assert_eq!(DEFAULT_TOTP_PERIOD, 30);
        assert_eq!(DEFAULT_TOTP_DIGITS, 6);
    }
}
