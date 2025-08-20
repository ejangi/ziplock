//! API module for ZipLock shared library
//!
//! This module provides the core business logic API that was previously
//! in the backend. It handles all credential operations, archive management,
//! and validation in a way that can be called directly via FFI or through
//! the legacy IPC interface.

pub mod handlers;

// Re-export commonly used types
pub use handlers::{ApiHandlers, ArchiveInfo, BackendStatus, OperationStats, ValidationError};

use crate::archive::{ArchiveConfig, ArchiveError, ArchiveManager};
use crate::models::CredentialRecord;

use std::sync::Arc;
use thiserror::Error;

/// Core API client for direct FFI access
pub struct ZipLockApi {
    handlers: ApiHandlers,
    archive_manager: Arc<ArchiveManager>,
}

/// Session management for the API
#[derive(Debug, Clone)]
pub struct ApiSession {
    pub session_id: String,
    pub authenticated: bool,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
}

/// API operation results
pub type ApiResult<T> = Result<T, ApiError>;

/// Comprehensive error type for API operations
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Archive error: {0}")]
    Archive(#[from] ArchiveError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Shared library error: {message}")]
    Shared { message: String },

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: String },

    #[error("Operation not allowed in current state")]
    InvalidState,

    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("Archive not open")]
    ArchiveNotOpen,

    #[error("Internal API error: {message}")]
    Internal { message: String },
}

impl ZipLockApi {
    /// Create a new API instance
    pub fn new(config: ArchiveConfig) -> ApiResult<Self> {
        let archive_manager = Arc::new(ArchiveManager::new(config)?);
        let handlers = ApiHandlers::new(archive_manager.clone(), Default::default());

        Ok(Self {
            handlers,
            archive_manager,
        })
    }

    /// Create a new session
    pub fn create_session(&self) -> ApiResult<ApiSession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now();

        Ok(ApiSession {
            session_id,
            authenticated: false,
            created_at: now,
            last_activity: now,
        })
    }

    /// Authenticate a session by opening an archive
    pub async fn authenticate_session(
        &self,
        session: &mut ApiSession,
        archive_path: std::path::PathBuf,
        master_password: String,
    ) -> ApiResult<()> {
        self.handlers
            .open_archive(archive_path, master_password)
            .await
            .map_err(ApiError::from)?;

        session.authenticated = true;
        session.last_activity = std::time::SystemTime::now();

        Ok(())
    }

    /// Create a new archive
    pub async fn create_archive(
        &self,
        path: std::path::PathBuf,
        master_password: String,
    ) -> ApiResult<()> {
        self.handlers
            .create_archive(path, master_password)
            .await
            .map_err(ApiError::from)
    }

    /// Open an existing archive
    pub async fn open_archive(
        &self,
        path: std::path::PathBuf,
        master_password: String,
    ) -> ApiResult<()> {
        self.handlers
            .open_archive(path, master_password)
            .await
            .map_err(ApiError::from)
    }

    /// Close the current archive
    pub async fn close_archive(&self) -> ApiResult<()> {
        self.handlers.close_archive().await.map_err(ApiError::from)
    }

    /// List all credentials
    pub async fn list_credentials(&self) -> ApiResult<Vec<CredentialRecord>> {
        self.handlers
            .list_credentials(false)
            .await
            .map_err(ApiError::from)
    }

    /// Get a specific credential by ID
    pub async fn get_credential(&self, id: &str) -> ApiResult<CredentialRecord> {
        self.handlers
            .get_credential(id.to_string())
            .await
            .map_err(ApiError::from)
    }

    /// Create a new credential
    pub async fn create_credential(&self, credential: CredentialRecord) -> ApiResult<String> {
        self.handlers
            .create_credential(credential)
            .await
            .map_err(ApiError::from)
    }

    /// Update an existing credential
    pub async fn update_credential(
        &self,
        id: String,
        credential: CredentialRecord,
    ) -> ApiResult<()> {
        self.handlers
            .update_credential(id, credential)
            .await
            .map_err(ApiError::from)
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: String) -> ApiResult<()> {
        self.handlers
            .delete_credential(id)
            .await
            .map_err(ApiError::from)
    }

    /// Search credentials
    pub async fn search_credentials(&self, query: String) -> ApiResult<Vec<CredentialRecord>> {
        self.handlers
            .search_credentials(query)
            .await
            .map_err(ApiError::from)
    }

    /// Save the current archive
    pub async fn save_archive(&self) -> ApiResult<()> {
        self.handlers.save_archive().await.map_err(ApiError::from)
    }

    /// Get archive information
    pub async fn get_archive_info(&self) -> ApiResult<ArchiveInfo> {
        self.handlers
            .get_archive_info()
            .await
            .map_err(ApiError::from)
    }

    /// Get API status
    pub async fn get_status(&self) -> ApiResult<BackendStatus> {
        Ok(self.handlers.get_status().await)
    }

    /// Validate the repository
    pub async fn validate_repository(
        &self,
        path: std::path::PathBuf,
    ) -> ApiResult<crate::archive::ValidationReport> {
        self.handlers
            .validate_repository(path)
            .await
            .map_err(ApiError::from)
    }

    /// Repair archive
    pub async fn repair_archive(
        &self,
        path: std::path::PathBuf,
        master_password: String,
    ) -> ApiResult<crate::archive::ValidationReport> {
        self.handlers
            .repair_archive(path, master_password)
            .await
            .map_err(|e| ApiError::Shared {
                message: e.to_string(),
            })
    }

    /// Check if an archive is currently open
    pub async fn is_archive_open(&self) -> bool {
        self.archive_manager.is_open().await
    }
}

impl From<crate::error::SharedError> for ApiError {
    fn from(error: crate::error::SharedError) -> Self {
        ApiError::Shared {
            message: error.to_string(),
        }
    }
}

/// Convert API errors to user-friendly messages
impl ApiError {
    pub fn user_message(&self) -> String {
        match self {
            ApiError::Archive(ArchiveError::NotFound { .. }) => {
                "Archive file not found. Please check the file path.".to_string()
            }
            ApiError::Archive(ArchiveError::Corrupted { .. }) => {
                "Archive appears to be corrupted. Please restore from backup.".to_string()
            }
            ApiError::Archive(ArchiveError::LockFailed { .. }) => {
                "Could not lock archive file. It may be in use by another application.".to_string()
            }
            ApiError::AuthenticationRequired => "Please unlock the archive first.".to_string(),
            ApiError::InvalidCredentials => "Invalid password. Please try again.".to_string(),
            ApiError::ArchiveNotOpen => "No archive is currently open.".to_string(),
            ApiError::Validation(ValidationError::InvalidCredentialId { .. }) => {
                "Invalid credential ID format.".to_string()
            }
            ApiError::Validation(ValidationError::EmptySearchQuery) => {
                "Search query cannot be empty.".to_string()
            }
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::ArchiveConfig;

    #[tokio::test]
    async fn test_api_creation() {
        let config = ArchiveConfig::default();
        let api = ZipLockApi::new(config).unwrap();

        let status = api.get_status().await.unwrap();
        assert!(!status.is_archive_open);
    }

    #[tokio::test]
    async fn test_session_creation() {
        let config = ArchiveConfig::default();
        let api = ZipLockApi::new(config).unwrap();

        let session = api.create_session().unwrap();
        assert!(!session.authenticated);
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_api_error_user_messages() {
        let error = ApiError::AuthenticationRequired;
        assert!(error.user_message().contains("unlock"));

        let error = ApiError::InvalidCredentials;
        assert!(error.user_message().contains("password"));
    }
}
