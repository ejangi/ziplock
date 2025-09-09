//! Repository service for Linux desktop app using UnifiedRepositoryManager
//!
//! This module provides a high-level service interface for repository operations
//! using the shared library's UnifiedRepositoryManager with DesktopFileProvider.
//! It bridges between the async UI layer and the sync repository operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::sync::{Arc, RwLock};
use tokio::task;
use tracing::{debug, error, info, warn};

use ziplock_shared::{CoreError, CredentialRecord, DesktopFileProvider, UnifiedRepositoryManager};

/// Repository service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    pub credential_count: usize,
    pub is_open: bool,
    pub is_modified: bool,
    pub current_path: Option<String>,
}

/// Repository service that provides async interface to UnifiedRepositoryManager
pub struct RepositoryService {
    manager: Arc<RwLock<Option<UnifiedRepositoryManager<DesktopFileProvider>>>>,
    current_stats: Arc<RwLock<RepositoryStats>>,
}

impl RepositoryService {
    /// Create a new repository service
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(None)),
            current_stats: Arc::new(RwLock::new(RepositoryStats {
                credential_count: 0,
                is_open: false,
                is_modified: false,
                current_path: None,
            })),
        }
    }

    /// Create a new repository at the specified path
    #[allow(dead_code)]
    pub async fn create_repository(&self, path: String, password: String) -> Result<()> {
        let manager_clone = Arc::clone(&self.manager);
        let stats_clone = Arc::clone(&self.current_stats);

        task::spawn_blocking(move || {
            info!("Creating new repository at: {}", path);

            let file_provider = DesktopFileProvider::new();
            let mut manager = UnifiedRepositoryManager::new(file_provider);

            match manager.create_repository(&path, &password) {
                Ok(()) => {
                    info!("Repository created successfully: {}", path);

                    // Update stats
                    {
                        let mut stats = stats_clone.write().unwrap();
                        stats.is_open = true;
                        stats.current_path = Some(path.clone());
                        stats.credential_count = 0;
                        stats.is_modified = false;
                    }

                    // Store manager
                    {
                        let mut mgr_guard = manager_clone.write().unwrap();
                        *mgr_guard = Some(manager);
                    }

                    Ok(())
                }
                Err(e) => {
                    error!("Failed to create repository {}: {}", path, e);
                    Err(anyhow::anyhow!("Failed to create repository: {}", e))
                }
            }
        })
        .await?
    }

    /// Open an existing repository
    pub async fn open_repository(&self, path: String, password: String) -> Result<()> {
        let manager_clone = Arc::clone(&self.manager);
        let stats_clone = Arc::clone(&self.current_stats);

        task::spawn_blocking(move || {
            info!("Opening repository: {}", path);

            let file_provider = DesktopFileProvider::new();
            let mut manager = UnifiedRepositoryManager::new(file_provider);

            match manager.open_repository(&path, &password) {
                Ok(()) => {
                    info!("Repository opened successfully: {}", path);

                    // Get credential count
                    let credential_count = manager
                        .list_credentials()
                        .map(|creds| creds.len())
                        .unwrap_or(0);

                    // Update stats
                    {
                        let mut stats = stats_clone.write().unwrap();
                        stats.is_open = true;
                        stats.current_path = Some(path.clone());
                        stats.credential_count = credential_count;
                        stats.is_modified = false;
                    }

                    // Store manager
                    {
                        let mut mgr_guard = manager_clone.write().unwrap();
                        *mgr_guard = Some(manager);
                    }

                    Ok(())
                }
                Err(CoreError::FileOperation(ziplock_shared::FileError::InvalidPassword)) => {
                    warn!("Invalid password for repository: {}", path);
                    Err(anyhow::anyhow!("Invalid password"))
                }
                Err(CoreError::FileOperation(ziplock_shared::FileError::NotFound { .. })) => {
                    warn!("Repository file not found: {}", path);
                    Err(anyhow::anyhow!("Repository file not found"))
                }
                Err(e) => {
                    error!("Failed to open repository {}: {}", path, e);
                    Err(anyhow::anyhow!("Failed to open repository: {}", e))
                }
            }
        })
        .await?
    }

    /// Close the current repository
    #[allow(dead_code)]
    pub async fn close_repository(&self) -> Result<()> {
        let manager_clone = Arc::clone(&self.manager);
        let stats_clone = Arc::clone(&self.current_stats);

        task::spawn_blocking(move || {
            info!("Closing repository");

            // Save any pending changes before closing
            if let Some(ref mut manager) = manager_clone.write().unwrap().as_mut() {
                if let Err(e) = manager.save_repository() {
                    warn!("Failed to save repository before closing: {}", e);
                }
            }

            // Clear manager
            *manager_clone.write().unwrap() = None;

            // Reset stats
            {
                let mut stats = stats_clone.write().unwrap();
                stats.is_open = false;
                stats.current_path = None;
                stats.credential_count = 0;
                stats.is_modified = false;
            }

            info!("Repository closed");
            Ok(())
        })
        .await?
    }

    /// Add a new credential
    pub async fn add_credential(&self, credential: CredentialRecord) -> Result<String> {
        let manager_clone = Arc::clone(&self.manager);
        let stats_clone = Arc::clone(&self.current_stats);
        let credential_id = credential.id.clone();

        task::spawn_blocking(move || {
            let mut mgr_guard = manager_clone.write().unwrap();
            match mgr_guard.as_mut() {
                Some(manager) => {
                    match manager.add_credential(credential) {
                        Ok(()) => {
                            debug!("Added credential: {}", credential_id);

                            // Update stats
                            {
                                let mut stats = stats_clone.write().unwrap();
                                stats.credential_count += 1;
                                stats.is_modified = true;
                            }

                            // Auto-save
                            if let Err(e) = manager.save_repository() {
                                error!("Failed to auto-save after adding credential: {}", e);
                                return Err(anyhow::anyhow!("Failed to save: {}", e));
                            }

                            Ok(credential_id)
                        }
                        Err(e) => {
                            error!("Failed to add credential: {}", e);
                            Err(anyhow::anyhow!("Failed to add credential: {}", e))
                        }
                    }
                }
                None => {
                    error!("No repository is open");
                    Err(anyhow::anyhow!("No repository is open"))
                }
            }
        })
        .await?
    }

    /// Get a credential by ID
    pub async fn get_credential(&self, id: String) -> Result<Option<CredentialRecord>> {
        let manager_clone = Arc::clone(&self.manager);

        task::spawn_blocking(move || {
            let mgr_guard = manager_clone.read().unwrap();
            match mgr_guard.as_ref() {
                Some(manager) => match manager.get_credential_readonly(&id) {
                    Ok(credential) => Ok(Some(credential.clone())),
                    Err(CoreError::CredentialNotFound { .. }) => Ok(None),
                    Err(e) => {
                        error!("Failed to get credential {}: {}", id, e);
                        Err(anyhow::anyhow!("Failed to get credential: {}", e))
                    }
                },
                None => {
                    error!("No repository is open");
                    Err(anyhow::anyhow!("No repository is open"))
                }
            }
        })
        .await?
    }

    /// Update an existing credential
    pub async fn update_credential(&self, credential: CredentialRecord) -> Result<()> {
        let manager_clone = Arc::clone(&self.manager);
        let stats_clone = Arc::clone(&self.current_stats);
        let credential_id = credential.id.clone();

        task::spawn_blocking(move || {
            let mut mgr_guard = manager_clone.write().unwrap();
            match mgr_guard.as_mut() {
                Some(manager) => {
                    match manager.update_credential(credential) {
                        Ok(()) => {
                            debug!("Updated credential: {}", credential_id);

                            // Update stats
                            {
                                let mut stats = stats_clone.write().unwrap();
                                stats.is_modified = true;
                            }

                            // Auto-save
                            if let Err(e) = manager.save_repository() {
                                error!("Failed to auto-save after updating credential: {}", e);
                                return Err(anyhow::anyhow!("Failed to save: {}", e));
                            }

                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to update credential {}: {}", credential_id, e);
                            Err(anyhow::anyhow!("Failed to update credential: {}", e))
                        }
                    }
                }
                None => {
                    error!("No repository is open");
                    Err(anyhow::anyhow!("No repository is open"))
                }
            }
        })
        .await?
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: String) -> Result<()> {
        let manager_clone = Arc::clone(&self.manager);
        let stats_clone = Arc::clone(&self.current_stats);

        task::spawn_blocking(move || {
            let mut mgr_guard = manager_clone.write().unwrap();
            match mgr_guard.as_mut() {
                Some(manager) => {
                    match manager.delete_credential(&id) {
                        Ok(_) => {
                            debug!("Deleted credential: {}", id);

                            // Update stats
                            {
                                let mut stats = stats_clone.write().unwrap();
                                stats.credential_count = stats.credential_count.saturating_sub(1);
                                stats.is_modified = true;
                            }

                            // Auto-save
                            if let Err(e) = manager.save_repository() {
                                error!("Failed to auto-save after deleting credential: {}", e);
                                return Err(anyhow::anyhow!("Failed to save: {}", e));
                            }

                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to delete credential {}: {}", id, e);
                            Err(anyhow::anyhow!("Failed to delete credential: {}", e))
                        }
                    }
                }
                None => {
                    error!("No repository is open");
                    Err(anyhow::anyhow!("No repository is open"))
                }
            }
        })
        .await?
    }

    /// List all credentials
    pub async fn list_credentials(&self) -> Result<Vec<CredentialRecord>> {
        let manager_clone = Arc::clone(&self.manager);

        task::spawn_blocking(move || {
            let mgr_guard = manager_clone.read().unwrap();
            match mgr_guard.as_ref() {
                Some(manager) => match manager.list_credentials() {
                    Ok(credentials) => {
                        debug!("Listed {} credentials", credentials.len());
                        Ok(credentials)
                    }
                    Err(e) => {
                        error!("Failed to list credentials: {}", e);
                        Err(anyhow::anyhow!("Failed to list credentials: {}", e))
                    }
                },
                None => {
                    error!("No repository is open");
                    Err(anyhow::anyhow!("No repository is open"))
                }
            }
        })
        .await?
    }

    /// Search credentials
    #[allow(dead_code)]
    pub async fn search_credentials(&self, query: String) -> Result<Vec<CredentialRecord>> {
        let manager_clone = Arc::clone(&self.manager);

        task::spawn_blocking(move || {
            let mgr_guard = manager_clone.read().unwrap();
            match mgr_guard.as_ref() {
                Some(manager) => match manager.list_credentials() {
                    Ok(credentials) => {
                        let query_lower = query.to_lowercase();
                        let filtered: Vec<CredentialRecord> = credentials
                            .into_iter()
                            .filter(|cred| {
                                cred.title.to_lowercase().contains(&query_lower)
                                    || cred.fields.iter().any(|(_, field)| {
                                        field.display_value().to_lowercase().contains(&query_lower)
                                    })
                            })
                            .collect();

                        debug!("Search '{}' returned {} results", query, filtered.len());
                        Ok(filtered)
                    }
                    Err(e) => {
                        error!("Failed to search credentials: {}", e);
                        Err(anyhow::anyhow!("Failed to search credentials: {}", e))
                    }
                },
                None => {
                    error!("No repository is open");
                    Err(anyhow::anyhow!("No repository is open"))
                }
            }
        })
        .await?
    }

    /// Get repository statistics
    #[allow(dead_code)]
    pub async fn get_stats(&self) -> Result<RepositoryStats> {
        let stats = self.current_stats.read().unwrap().clone();
        Ok(stats)
    }

    /// Check if repository is open
    pub async fn is_open(&self) -> bool {
        self.current_stats.read().unwrap().is_open
    }

    /// Get current repository path
    #[allow(dead_code)]
    pub async fn current_path(&self) -> Option<String> {
        self.current_stats.read().unwrap().current_path.clone()
    }
}

impl Default for RepositoryService {
    fn default() -> Self {
        Self::new()
    }
}

// Global repository service instance
static REPOSITORY_SERVICE: std::sync::OnceLock<RepositoryService> = std::sync::OnceLock::new();

/// Get the global repository service instance
pub fn get_repository_service() -> &'static RepositoryService {
    REPOSITORY_SERVICE.get_or_init(RepositoryService::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use ziplock_shared::models::{CredentialField, CredentialRecord};

    fn create_test_credential() -> CredentialRecord {
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));
        credential
    }

    #[tokio::test]
    async fn test_repository_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.7z");
        let repo_path_str = repo_path.to_string_lossy().to_string();

        let service = RepositoryService::new();

        // Initially not open
        assert!(!service.is_open().await);

        // Create repository
        service
            .create_repository(repo_path_str.clone(), "testpass".to_string())
            .await
            .unwrap();
        assert!(service.is_open().await);

        // Add credential
        let credential = create_test_credential();
        let id = service.add_credential(credential.clone()).await.unwrap();
        assert_eq!(id, credential.id);

        // Get credential
        let retrieved = service.get_credential(id.clone()).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Login");

        // List credentials
        let credentials = service.list_credentials().await.unwrap();
        assert_eq!(credentials.len(), 1);

        // Close and reopen
        service.close_repository().await.unwrap();
        assert!(!service.is_open().await);

        service
            .open_repository(repo_path_str, "testpass".to_string())
            .await
            .unwrap();
        assert!(service.is_open().await);

        // Verify persistence
        let credentials = service.list_credentials().await.unwrap();
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].title, "Test Login");
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("search_test.7z");
        let repo_path_str = repo_path.to_string_lossy().to_string();

        let service = RepositoryService::new();
        service
            .create_repository(repo_path_str, "testpass".to_string())
            .await
            .unwrap();

        // Add multiple credentials
        let mut cred1 = CredentialRecord::new("Gmail Account".to_string(), "login".to_string());
        cred1.set_field("username", CredentialField::username("user@gmail.com"));
        service.add_credential(cred1).await.unwrap();

        let mut cred2 = CredentialRecord::new("Work Login".to_string(), "login".to_string());
        cred2.set_field(
            "username",
            CredentialField::username("employee@company.com"),
        );
        service.add_credential(cred2).await.unwrap();

        // Search tests
        let gmail_results = service
            .search_credentials("Gmail".to_string())
            .await
            .unwrap();
        assert_eq!(gmail_results.len(), 1);
        assert_eq!(gmail_results[0].title, "Gmail Account");

        let email_results = service
            .search_credentials("gmail.com".to_string())
            .await
            .unwrap();
        assert_eq!(email_results.len(), 1);

        let all_results = service
            .search_credentials("Login".to_string())
            .await
            .unwrap();
        assert_eq!(all_results.len(), 2);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let service = RepositoryService::new();

        // Try operations without open repository
        let result = service.list_credentials().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No repository is open"));

        // Try to open non-existent repository
        let result = service
            .open_repository("/nonexistent/path.7z".to_string(), "pass".to_string())
            .await;
        assert!(result.is_err());

        // Try wrong password
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.7z");
        let repo_path_str = repo_path.to_string_lossy().to_string();

        service
            .create_repository(repo_path_str.clone(), "correctpass".to_string())
            .await
            .unwrap();
        service.close_repository().await.unwrap();

        let result = service
            .open_repository(repo_path_str, "wrongpass".to_string())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid password"));
    }
}
