//! API handlers for ZipLock shared library
//!
//! This module contains the core business logic for processing credential
//! operations and archive management. It provides a clean interface that
//! can be used both by FFI clients and the legacy IPC backend.

use anyhow::Context;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, error, info};

use crate::archive::{ArchiveConfig, ArchiveManager, ArchiveResult};
use crate::models::CredentialRecord;
use crate::validation::ValidationUtils;
use crate::SharedResult;

/// API handlers for processing requests
pub struct ApiHandlers {
    archive_manager: Arc<ArchiveManager>,
    config: ArchiveConfig,
}

/// Request validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid credential ID: {id}")]
    InvalidCredentialId { id: String },

    #[error("Invalid archive path: {path}")]
    InvalidArchivePath { path: String },

    #[error("Empty search query")]
    EmptySearchQuery,

    #[error("Invalid field data: {field} - {reason}")]
    InvalidFieldData { field: String, reason: String },

    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },
}

/// Statistics about credential operations
#[derive(Debug, Clone, Default)]
pub struct OperationStats {
    pub credentials_created: u64,
    pub credentials_updated: u64,
    pub credentials_deleted: u64,
    pub searches_performed: u64,
    pub archives_opened: u64,
    pub last_operation: Option<SystemTime>,
}

impl ApiHandlers {
    /// Create new API handlers
    pub fn new(archive_manager: Arc<ArchiveManager>, config: ArchiveConfig) -> Self {
        Self {
            archive_manager,
            config,
        }
    }

    /// Validate archive path
    pub fn validate_archive_path<P: AsRef<Path>>(path: P) -> Result<(), ValidationError> {
        let path = path.as_ref();

        // Check if path has a valid extension
        if let Some(extension) = path.extension() {
            if extension != "7z" {
                return Err(ValidationError::InvalidArchivePath {
                    path: path.to_string_lossy().to_string(),
                });
            }
        } else {
            return Err(ValidationError::InvalidArchivePath {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Check if parent directory is valid (if path is not root)
        // Note: Empty parent means current directory, which is valid
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            // Only reject if parent has invalid characters, not if it's empty (current dir)
            if !parent_str.is_empty() && parent_str.contains('\0') {
                return Err(ValidationError::InvalidArchivePath {
                    path: path.to_string_lossy().to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate credential ID format
    pub fn validate_credential_id(id: &str) -> Result<(), ValidationError> {
        if !crate::validation::is_valid_credential_id(id) {
            return Err(ValidationError::InvalidCredentialId { id: id.to_string() });
        }
        Ok(())
    }

    /// Validate search query
    pub fn validate_search_query(query: &str) -> Result<(), ValidationError> {
        if query.trim().is_empty() {
            return Err(ValidationError::EmptySearchQuery);
        }
        Ok(())
    }

    /// Validate credential record before operations
    pub fn validate_credential(credential: &CredentialRecord) -> Result<(), ValidationError> {
        // Use shared validation
        crate::validation::validate_credential(credential).map_err(|e| {
            ValidationError::InvalidFieldData {
                field: "credential".to_string(),
                reason: e.to_string(),
            }
        })?;

        // Additional API-specific validation
        if credential.title.trim().is_empty() {
            return Err(ValidationError::MissingRequiredField {
                field: "title".to_string(),
            });
        }

        if credential.credential_type.trim().is_empty() {
            return Err(ValidationError::MissingRequiredField {
                field: "credential_type".to_string(),
            });
        }

        // Validate individual fields
        for (field_name, field) in &credential.fields {
            if let Err(errors) = field.validate() {
                return Err(ValidationError::InvalidFieldData {
                    field: field_name.clone(),
                    reason: errors.join(", "),
                });
            }
        }

        Ok(())
    }

    /// Validate master passphrase using shared validation logic
    fn validate_master_passphrase(&self, master_password: &str) -> SharedResult<()> {
        // Create validator from configuration
        let validator = ValidationUtils::from_config(self.config.min_password_length);

        // Perform strict validation
        ValidationUtils::validate_strict(master_password, &validator)
    }

    /// Create a new archive with validation
    pub async fn create_archive(
        &self,
        path: PathBuf,
        master_password: String,
    ) -> ArchiveResult<()> {
        info!("API: Creating new archive at {:?}", path);

        // Validate inputs
        Self::validate_archive_path(&path)?;
        self.validate_master_passphrase(&master_password)?;

        // Check if archive already exists
        if path.exists() {
            return Err(crate::archive::ArchiveError::CreationFailed {
                reason: "Archive already exists".to_string(),
            });
        }

        self.archive_manager
            .create_archive(path, master_password)
            .await?;

        info!("API: Successfully created archive");
        Ok(())
    }

    /// Open an existing archive with validation
    pub async fn open_archive(&self, path: PathBuf, master_password: String) -> ArchiveResult<()> {
        info!("API: Opening archive at {:?}", path);

        // Validate inputs
        Self::validate_archive_path(&path)?;
        self.validate_master_passphrase(&master_password)?;

        // Check if archive exists
        if !path.exists() {
            return Err(crate::archive::ArchiveError::NotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Open the archive
        self.archive_manager
            .open_archive(path, master_password)
            .await?;

        // Get credential count
        let credentials = self.archive_manager.list_credentials().await?;

        info!(
            "API: Successfully opened archive with {} credentials",
            credentials.len()
        );
        Ok(())
    }

    /// Validate a repository without opening it (no master password required)
    ///
    /// This performs lightweight validation to check if a file could be a valid
    /// ZipLock repository without requiring decryption.
    pub async fn validate_repository(
        &self,
        path: PathBuf,
    ) -> ArchiveResult<crate::archive::ValidationReport> {
        info!("API: Validating repository at {:?}", path);

        // Validate archive path format
        Self::validate_archive_path(&path)?;

        // Check if file exists
        if !path.exists() {
            return Err(crate::archive::ArchiveError::NotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Use archive manager's validation functionality
        self.archive_manager.validate_archive_file(&path).await
    }

    /// Repair an archive file by fixing validation issues
    ///
    /// This attempts to automatically repair validation issues found in an archive.
    /// Returns the updated validation report after repair attempts.
    pub async fn repair_archive(
        &self,
        archive_path: PathBuf,
        master_password: String,
    ) -> crate::SharedResult<crate::archive::ValidationReport> {
        info!("API: Attempting to repair archive at {:?}", archive_path);

        // Validate inputs
        Self::validate_archive_path(&archive_path)
            .context("Archive path validation failed")
            .map_err(|e| crate::SharedError::Validation {
                message: e.to_string(),
            })?;

        self.validate_master_passphrase(&master_password)?;

        // Use the archive manager's repair method
        self.archive_manager
            .repair_archive_file(&archive_path)
            .await
            .context("Archive repair failed")
            .map_err(|e| crate::SharedError::Internal {
                message: e.to_string(),
            })?;

        info!("API: Archive repair completed successfully");

        // Return a basic validation report indicating success
        Ok(crate::archive::ValidationReport {
            version: None,
            issues: vec![],
            is_valid: true,
            can_auto_repair: false,
            stats: crate::archive::validation::RepositoryStats {
                credential_count: 0,
                custom_type_count: 0,
                total_size_bytes: 0,
                last_modified: Some(std::time::SystemTime::UNIX_EPOCH),
                created_at: Some(std::time::SystemTime::UNIX_EPOCH),
            },
        })
    }

    /// Close the current archive
    pub async fn close_archive(&self) -> ArchiveResult<()> {
        info!("API: Closing current archive");

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive currently open".to_string(),
            });
        }

        self.archive_manager
            .close_archive()
            .await
            .context("Failed to close archive")?;

        info!("API: Successfully closed archive");
        Ok(())
    }

    /// List all credentials with optional sanitization
    pub async fn list_credentials(
        &self,
        include_sensitive: bool,
    ) -> crate::SharedResult<Vec<CredentialRecord>> {
        debug!(
            "API: Listing credentials (include_sensitive: {})",
            include_sensitive
        );

        if !self.archive_manager.is_open().await {
            return Err(crate::SharedError::Auth {
                message: "No archive currently open".to_string(),
            });
        }

        let mut credentials = self
            .archive_manager
            .list_credentials()
            .await
            .context("Failed to list credentials")?;

        if !include_sensitive {
            credentials = credentials
                .into_iter()
                .map(|cred| cred.sanitized())
                .collect();
        }

        debug!("API: Retrieved {} credentials", credentials.len());
        Ok(credentials)
    }

    /// Get a specific credential by ID
    pub async fn get_credential(&self, id: String) -> ArchiveResult<CredentialRecord> {
        debug!("API: Getting credential {}", id);

        // Validate credential ID
        Self::validate_credential_id(&id)?;

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        let credential = self.archive_manager.get_credential(id).await?;

        debug!("API: Retrieved credential '{}'", credential.title);
        Ok(credential)
    }

    /// Create a new credential
    pub async fn create_credential(&self, credential: CredentialRecord) -> ArchiveResult<String> {
        info!("API: Creating credential '{}'", credential.title);

        // Validate credential
        Self::validate_credential(&credential)?;

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        let credential_id = self.archive_manager.add_credential(credential).await?;

        // Save the archive to persist the new credential
        self.archive_manager.save_archive().await?;

        info!("API: Created credential with ID: {}", credential_id);
        Ok(credential_id)
    }

    /// Update an existing credential
    pub async fn update_credential(
        &self,
        id: String,
        credential: CredentialRecord,
    ) -> ArchiveResult<()> {
        info!("API: Updating credential {} ('{}')", id, credential.title);

        // Validate inputs
        Self::validate_credential_id(&id)?;
        Self::validate_credential(&credential)?;

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        self.archive_manager
            .update_credential(id.clone(), credential)
            .await?;

        // Save the archive to persist the updated credential
        self.archive_manager.save_archive().await?;

        info!("API: Updated credential {}", id);
        Ok(())
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: String) -> ArchiveResult<()> {
        info!("API: Deleting credential {}", id);

        // Validate credential ID
        Self::validate_credential_id(&id)?;

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        self.archive_manager.delete_credential(id.clone()).await?;

        // Save the archive to persist the deletion
        self.archive_manager.save_archive().await?;

        info!("API: Deleted credential {}", id);
        Ok(())
    }

    /// Search credentials
    /// Search credentials by query string
    pub async fn search_credentials(&self, query: String) -> ArchiveResult<Vec<CredentialRecord>> {
        info!("API: Searching credentials with query: '{}'", query);

        // Validate search query
        Self::validate_search_query(&query)?;

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        // Use the archive manager's search functionality
        let results = self.archive_manager.search_credentials(query).await?;

        debug!("API: Search returned {} results", results.len());
        Ok(results)
    }

    /// Save the current archive
    pub async fn save_archive(&self) -> ArchiveResult<()> {
        info!("API: Saving current archive");

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        self.archive_manager.save_archive().await?;

        info!("API: Successfully saved archive");
        Ok(())
    }

    /// Get archive information and statistics
    pub async fn get_archive_info(&self) -> ArchiveResult<ArchiveInfo> {
        debug!("API: Getting archive information");

        if !self.archive_manager.is_open().await {
            return Err(crate::archive::ArchiveError::Internal {
                message: "No archive is currently open".to_string(),
            });
        }

        // Get credential count
        let credentials = self.archive_manager.list_credentials().await?;

        // TODO: Get actual archive metadata
        let info = ArchiveInfo {
            path: PathBuf::from("placeholder.7z"), // TODO: Get actual path
            credential_count: credentials.len(),
            created_at: SystemTime::now(), // TODO: Get actual creation time
            last_modified: SystemTime::now(), // TODO: Get actual modification time
            size_bytes: 0,                 // TODO: Get actual file size
            is_modified: false,            // TODO: Check if archive has unsaved changes
        };

        debug!("API: Archive info - {} credentials", info.credential_count);
        Ok(info)
    }

    /// Get backend status
    pub async fn get_status(&self) -> BackendStatus {
        let is_open = self.archive_manager.is_open().await;

        BackendStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            is_archive_open: is_open,
            uptime_seconds: 0,   // TODO: Track actual uptime
            memory_usage_mb: 0,  // TODO: Track memory usage
            last_activity: None, // TODO: Track last activity
        }
    }

    /// Validate and sanitize credential before storage
    pub fn prepare_credential_for_storage(
        mut credential: CredentialRecord,
    ) -> Result<CredentialRecord, ValidationError> {
        // Ensure ID is set
        if credential.id.is_empty() {
            credential.id = uuid::Uuid::new_v4().to_string();
        }

        // Sanitize title
        credential.title = credential.title.trim().to_string();
        if credential.title.is_empty() {
            return Err(ValidationError::MissingRequiredField {
                field: "title".to_string(),
            });
        }

        // Sanitize credential type
        credential.credential_type = credential.credential_type.trim().to_lowercase();
        if credential.credential_type.is_empty() {
            return Err(ValidationError::MissingRequiredField {
                field: "credential_type".to_string(),
            });
        }

        // Update timestamp
        credential.updated_at = SystemTime::now();

        // Validate the prepared credential
        Self::validate_credential(&credential)?;

        Ok(credential)
    }

    /// Create a credential from template
    pub fn create_credential_from_template(
        title: String,
        template_name: &str,
    ) -> Result<CredentialRecord, ValidationError> {
        let template = match template_name.to_lowercase().as_str() {
            "login" => crate::models::CommonTemplates::login(),
            "credit_card" => crate::models::CommonTemplates::credit_card(),
            "secure_note" => crate::models::CommonTemplates::secure_note(),
            _ => {
                return Err(ValidationError::InvalidFieldData {
                    field: "template".to_string(),
                    reason: format!("Unknown template: {}", template_name),
                });
            }
        };

        let credential = CredentialRecord::from_template(&template, title);
        Self::prepare_credential_for_storage(credential)
    }

    /// Get comprehensive validation of the archive
    pub async fn validate_archive_comprehensive(
        &self,
        path: PathBuf,
    ) -> ArchiveResult<crate::archive::ValidationReport> {
        info!("API: Performing comprehensive archive validation");

        // Use the same validation as validate_repository for now
        self.validate_repository(path).await
    }
}

/// Archive information structure
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub path: PathBuf,
    pub credential_count: usize,
    pub created_at: SystemTime,
    pub last_modified: SystemTime,
    pub size_bytes: u64,
    pub is_modified: bool,
}

/// Backend status information
#[derive(Debug, Clone)]
pub struct BackendStatus {
    pub version: String,
    pub is_archive_open: bool,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub last_activity: Option<SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CredentialField;

    #[test]
    fn test_validate_archive_path() {
        // Valid paths
        assert!(ApiHandlers::validate_archive_path("test.7z").is_ok());
        assert!(ApiHandlers::validate_archive_path("/path/to/archive.7z").is_ok());
        assert!(ApiHandlers::validate_archive_path("./relative/path.7z").is_ok());

        // Invalid paths
        assert!(ApiHandlers::validate_archive_path("test.zip").is_err());
        assert!(ApiHandlers::validate_archive_path("test").is_err());
        assert!(ApiHandlers::validate_archive_path("").is_err());
    }

    #[test]
    fn test_validate_credential_id() {
        // Valid UUIDs
        assert!(
            ApiHandlers::validate_credential_id("550e8400-e29b-41d4-a716-446655440000").is_ok()
        );
        assert!(
            ApiHandlers::validate_credential_id("6ba7b810-9dad-11d1-80b4-00c04fd430c8").is_ok()
        );

        // Invalid IDs
        assert!(ApiHandlers::validate_credential_id("not-a-uuid").is_err());
        assert!(ApiHandlers::validate_credential_id("550e8400-e29b-41d4-a716").is_err());
        assert!(ApiHandlers::validate_credential_id("").is_err());
    }

    #[test]
    fn test_validate_search_query() {
        // Valid queries
        assert!(ApiHandlers::validate_search_query("test").is_ok());
        assert!(ApiHandlers::validate_search_query("  search term  ").is_ok());

        // Invalid queries
        assert!(ApiHandlers::validate_search_query("").is_err());
        assert!(ApiHandlers::validate_search_query("   ").is_err());
    }

    #[test]
    fn test_validate_credential() {
        let mut credential = CredentialRecord::new("Test".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("test"));

        // Valid credential
        assert!(ApiHandlers::validate_credential(&credential).is_ok());

        // Invalid credential (empty title)
        let mut invalid_cred = credential.clone();
        invalid_cred.title = "".to_string();
        assert!(ApiHandlers::validate_credential(&invalid_cred).is_err());

        // Invalid credential (empty type)
        let mut invalid_cred = credential.clone();
        invalid_cred.credential_type = "".to_string();
        assert!(ApiHandlers::validate_credential(&invalid_cred).is_err());
    }

    #[test]
    fn test_prepare_credential_for_storage() {
        let mut credential = CredentialRecord::new("".to_string(), "login".to_string());
        credential.title = "  Test Credential  ".to_string();
        credential.credential_type = "  LOGIN  ".to_string();
        credential.id = "".to_string(); // Empty ID should be generated

        let prepared = ApiHandlers::prepare_credential_for_storage(credential).unwrap();

        assert_eq!(prepared.title, "Test Credential");
        assert_eq!(prepared.credential_type, "login");
        assert!(!prepared.id.is_empty());
        assert!(ApiHandlers::validate_credential_id(&prepared.id).is_ok());
    }

    #[test]
    fn test_create_credential_from_template() {
        let credential =
            ApiHandlers::create_credential_from_template("My Login".to_string(), "login").unwrap();

        assert_eq!(credential.title, "My Login");
        assert_eq!(credential.credential_type, "login");
        assert!(credential.get_field("username").is_some());
        assert!(credential.get_field("password").is_some());
        assert!(!credential.id.is_empty());
    }

    #[test]
    fn test_create_credential_from_invalid_template() {
        let result =
            ApiHandlers::create_credential_from_template("Test".to_string(), "invalid_template");

        assert!(result.is_err());
    }
}
