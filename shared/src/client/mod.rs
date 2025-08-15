//! Unified Client Interface
//!
//! This module provides a unified client interface that works across all platforms
//! by using the new shared API directly. This replaces the IPC-based client for
//! desktop platforms and provides FFI bindings for mobile platforms.

use crate::api::{ApiSession, ZipLockApi};
use crate::archive::ArchiveConfig;
use crate::error::{SharedError, SharedResult};
use crate::models::CredentialRecord;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::RwLock;

/// Global API instance for shared state
static GLOBAL_API: OnceLock<Mutex<Option<Arc<ZipLockApi>>>> = OnceLock::new();

/// Unified client for ZipLock operations across all platforms
pub struct ZipLockClient {
    session: Arc<RwLock<Option<ApiSession>>>,
    is_connected: bool,
}

impl ZipLockClient {
    /// Create a new client instance
    pub fn new() -> SharedResult<Self> {
        // Ensure global API is initialized
        Self::ensure_global_api_initialized()?;

        Ok(Self {
            session: Arc::new(RwLock::new(None)),
            is_connected: false,
        })
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: ArchiveConfig) -> SharedResult<Self> {
        // Initialize global API with custom config
        Self::ensure_global_api_initialized_with_config(config)?;

        Ok(Self {
            session: Arc::new(RwLock::new(None)),
            is_connected: false,
        })
    }

    /// Ensure global API is initialized with default config
    fn ensure_global_api_initialized() -> SharedResult<()> {
        let config = ArchiveConfig::default();
        Self::ensure_global_api_initialized_with_config(config)
    }

    /// Ensure global API is initialized with specific config
    fn ensure_global_api_initialized_with_config(config: ArchiveConfig) -> SharedResult<()> {
        let global_api = GLOBAL_API.get_or_init(|| Mutex::new(None));
        let mut api_guard = global_api.lock().map_err(|e| SharedError::Internal {
            message: format!("Failed to lock global API: {}", e),
        })?;

        if api_guard.is_none() {
            let api = ZipLockApi::new(config).map_err(|e| SharedError::Api {
                message: e.to_string(),
            })?;
            *api_guard = Some(Arc::new(api));
        }

        Ok(())
    }

    /// Get the global API instance
    fn get_api() -> SharedResult<Arc<ZipLockApi>> {
        let global_api = GLOBAL_API.get().ok_or_else(|| SharedError::Internal {
            message: "Global API not initialized".to_string(),
        })?;

        let api_guard = global_api.lock().map_err(|e| SharedError::Internal {
            message: format!("Failed to lock global API: {}", e),
        })?;

        api_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| SharedError::Internal {
                message: "API not initialized".to_string(),
            })
    }

    /// Initialize the client connection
    pub async fn connect(&mut self) -> SharedResult<()> {
        let api = Self::get_api()?;

        // Create a new session
        let new_session = api.create_session().map_err(|e| SharedError::Api {
            message: e.to_string(),
        })?;

        let mut session_guard = self.session.write().await;
        *session_guard = Some(new_session);

        self.is_connected = true;
        Ok(())
    }

    /// Test connectivity with a ping
    pub async fn ping(&mut self) -> SharedResult<(String, u64)> {
        if !self.is_connected {
            return Err(SharedError::Internal {
                message: "Client not connected".to_string(),
            });
        }

        let api = Self::get_api()?;

        // Get API status to verify connectivity
        let status = api.get_status().await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })?;
        Ok((status.version, status.uptime_seconds))
    }

    /// Create a new session
    pub async fn create_session(&self) -> SharedResult<String> {
        let api = Self::get_api()?;

        let new_session = api.create_session().map_err(|e| SharedError::Api {
            message: e.to_string(),
        })?;
        let session_id = new_session.session_id.clone();

        let mut session_guard = self.session.write().await;
        *session_guard = Some(new_session);

        Ok(session_id)
    }

    /// Create an archive
    pub async fn create_archive(&self, path: PathBuf, master_password: String) -> SharedResult<()> {
        let api = Self::get_api()?;

        api.create_archive(path, master_password)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })
    }

    /// Open an existing archive
    pub async fn open_archive(&self, path: PathBuf, master_password: String) -> SharedResult<()> {
        let api = Self::get_api()?;

        // Open the archive
        api.open_archive(path, master_password)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })?;

        // Authenticate the current session
        let mut session_guard = self.session.write().await;
        if let Some(session) = session_guard.as_mut() {
            session.authenticated = true;
            session.last_activity = std::time::SystemTime::now();
        }

        Ok(())
    }

    /// Close the current archive
    pub async fn close_archive(&self) -> SharedResult<()> {
        let api = Self::get_api()?;

        api.close_archive().await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// List all credentials
    /// List credentials in the current archive
    pub async fn list_credentials(&self) -> SharedResult<Vec<CredentialRecord>> {
        let api = Self::get_api()?;

        api.list_credentials().await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// Get a specific credential by ID
    /// Get all credentials
    pub async fn get_credentials(
        &self,
        _session_id: Option<String>,
    ) -> SharedResult<Vec<CredentialRecord>> {
        let api = Self::get_api()?;

        api.list_credentials().await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// Get a specific credential by ID (internal method)
    pub async fn get_credential_internal(&self, id: String) -> SharedResult<CredentialRecord> {
        let api = Self::get_api()?;
        api.get_credential(&id).await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// Create a new credential from a CredentialRecord
    pub async fn create_credential_record(
        &self,
        credential: CredentialRecord,
    ) -> SharedResult<String> {
        let api = Self::get_api()?;

        api.create_credential(credential)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })
    }

    /// Create a new credential with individual parameters (compatibility method)
    pub async fn create_credential(
        &self,
        _session_id: Option<String>,
        title: String,
        credential_type: String,
        fields: std::collections::HashMap<String, crate::models::CredentialField>,
        tags: Vec<String>,
        notes: Option<String>,
    ) -> SharedResult<String> {
        use crate::models::CredentialRecord;

        let mut credential = CredentialRecord::new(title, credential_type);
        credential.fields = fields;
        credential.tags = tags;
        if let Some(notes) = notes {
            credential.notes = Some(notes);
        }

        self.create_credential_record(credential).await
    }

    /// Update an existing credential
    pub async fn update_credential(
        &self,
        _session_id: Option<String>,
        credential: CredentialRecord,
    ) -> SharedResult<()> {
        let api = Self::get_api()?;
        let id = credential.id.clone();

        api.update_credential(id, credential)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })
    }

    /// Delete a credential
    pub async fn delete_credential(
        &self,
        _session_id: Option<String>,
        id: String,
    ) -> SharedResult<()> {
        let api = Self::get_api()?;

        api.delete_credential(id)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })
    }

    /// Get a specific credential by ID
    pub async fn get_credential(
        &self,
        _session_id: Option<String>,
        id: String,
    ) -> SharedResult<CredentialRecord> {
        let api = Self::get_api()?;

        api.get_credential(&id).await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// Search credentials
    pub async fn search_credentials(&self, query: String) -> SharedResult<Vec<CredentialRecord>> {
        let api = Self::get_api()?;

        api.search_credentials(query)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })
    }

    /// Save the current archive
    pub async fn save_archive(&self) -> SharedResult<()> {
        let api = Self::get_api()?;

        api.save_archive().await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// Get archive information
    pub async fn get_archive_info(&self) -> SharedResult<crate::api::ArchiveInfo> {
        let api = Self::get_api()?;

        api.get_archive_info().await.map_err(|e| SharedError::Api {
            message: e.to_string(),
        })
    }

    /// Check if an archive is currently open
    pub async fn is_archive_open(&self) -> bool {
        match Self::get_api() {
            Ok(api) => api.is_archive_open().await,
            Err(_) => false, // If we can't get the API, assume no archive is open
        }
    }

    /// Validate a repository
    pub async fn validate_repository(&self, path: PathBuf) -> SharedResult<bool> {
        let api = Self::get_api()?;

        let report = api
            .validate_repository(path)
            .await
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })?;
        Ok(report.is_valid)
    }

    /// Repair an archive (internal method)
    pub async fn repair_archive_internal(
        &self,
        path: PathBuf,
        master_password: String,
    ) -> SharedResult<()> {
        let api = Self::get_api()?;

        api.repair_archive(path, master_password)
            .await
            .map(|_| ()) // Convert ValidationReport to ()
            .map_err(|e| SharedError::Api {
                message: e.to_string(),
            })
    }

    /// Get current session information
    pub async fn get_session(&self) -> Option<ApiSession> {
        self.session.read().await.clone()
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Get the current session ID (async version)
    pub async fn get_session_id_async(&self) -> Option<String> {
        let session_guard = self.session.read().await;
        session_guard.as_ref().map(|s| s.session_id.clone())
    }

    /// Check if the current session is authenticated
    pub async fn is_authenticated(&self) -> bool {
        let session_guard = self.session.read().await;
        session_guard
            .as_ref()
            .map(|s| s.authenticated)
            .unwrap_or(false)
    }

    /// Set session ID (for compatibility with client wrapper)
    pub fn set_session_id(&mut self, _session_id: String) {
        // For FFI client, sessions are managed internally
        // This method is kept for compatibility but doesn't need implementation
        // since the session is handled by the API layer
    }

    /// Get session ID (synchronous version for compatibility)
    pub fn get_session_id(&self) -> Option<String> {
        // Note: This is a synchronous version for compatibility with client wrapper
        // In practice, you should use the async version for better performance
        if let Ok(session_guard) = self.session.try_read() {
            session_guard.as_ref().map(|s| s.session_id.clone())
        } else {
            None
        }
    }

    /// Validate archive comprehensively with master password
    pub async fn validate_archive_comprehensive(
        &self,
        archive_path: PathBuf,
        master_password: String,
    ) -> SharedResult<(bool, usize, bool)> {
        // Open the archive temporarily for validation
        let temp_api = ZipLockApi::new(ArchiveConfig::default()).map_err(|e| SharedError::Api {
            message: e.to_string(),
        })?;

        // Try to open the archive to validate it can be decrypted
        match temp_api
            .open_archive(archive_path.clone(), master_password)
            .await
        {
            Ok(_) => {
                // Get credential count
                let credentials =
                    temp_api
                        .list_credentials()
                        .await
                        .map_err(|e| SharedError::Api {
                            message: e.to_string(),
                        })?;
                let credential_count = credentials.len();

                // Close the temporary archive
                let _ = temp_api.close_archive().await;

                // Return (is_valid, credential_count, can_auto_repair)
                Ok((true, credential_count, false))
            }
            Err(_) => {
                // Archive validation failed
                Ok((false, 0, false))
            }
        }
    }

    /// Repair archive with master password (compatibility method)
    pub async fn repair_archive(
        &self,
        archive_path: PathBuf,
        master_password: String,
    ) -> SharedResult<(bool, usize)> {
        // Use the API repair method with master password
        let api = Self::get_api()?;
        match api
            .repair_archive(archive_path.clone(), master_password.clone())
            .await
        {
            Ok(_) => {
                // Try to get credential count after repair
                let (is_valid, credential_count, _) = self
                    .validate_archive_comprehensive(archive_path, master_password)
                    .await?;
                Ok((is_valid, credential_count))
            }
            Err(_) => {
                // Repair failed
                Ok((false, 0))
            }
        }
    }

    /// Check if error message indicates session timeout
    pub fn is_session_timeout_error(error_message: &str) -> bool {
        error_message.contains("session")
            && (error_message.contains("timeout")
                || error_message.contains("expired")
                || error_message.contains("invalid"))
    }
}

impl Default for ZipLockClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default ZipLockClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = ZipLockClient::new().unwrap();
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_client_connection() {
        let mut client = ZipLockClient::new().unwrap();
        client.connect().await.unwrap();
        assert!(client.is_connected());
    }

    #[tokio::test]
    async fn test_session_creation() {
        let mut client = ZipLockClient::new().unwrap();
        client.connect().await.unwrap();

        let session_id = client.create_session().await.unwrap();
        assert!(!session_id.is_empty());
        assert_eq!(client.get_session_id(), Some(session_id));
    }
}
