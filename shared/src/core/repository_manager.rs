//! Repository manager for coordinating memory and file operations
//!
//! This module provides the unified repository manager that coordinates
//! between the pure memory repository and file operation providers,
//! implementing the complete repository lifecycle with proper separation
//! of concerns.


use crate::core::errors::{CoreError, CoreResult};
use crate::core::file_provider::FileOperationProvider;
use crate::core::memory_repository::UnifiedMemoryRepository;
use crate::core::types::{FileMap, RepositoryStats};
use crate::models::CredentialRecord;

/// Repository manager that coordinates memory operations with file I/O
pub struct UnifiedRepositoryManager<F: FileOperationProvider> {
    /// Pure memory repository for credential operations
    memory_repo: UnifiedMemoryRepository,

    /// File operation provider for platform-specific file handling
    file_provider: F,

    /// Current archive file path (if any)
    current_path: Option<String>,

    /// Current master password (kept in memory for save operations)
    master_password: Option<String>,

    /// Whether a repository is currently open
    is_open: bool,
}

impl<F: FileOperationProvider> UnifiedRepositoryManager<F> {
    /// Create a new repository manager with the given file provider
    pub fn new(file_provider: F) -> Self {
        Self {
            memory_repo: UnifiedMemoryRepository::new(),
            file_provider,
            current_path: None,
            master_password: None,
            is_open: false,
        }
    }

    /// Create a new repository at the specified path
    ///
    /// This creates an empty repository and saves it to the given path.
    ///
    /// # Arguments
    /// * `path` - Path where to create the new repository
    /// * `master_password` - Password for encrypting the repository
    ///
    /// # Returns
    /// * `Ok(())` - If repository was created successfully
    /// * `Err(CoreError)` - If creation fails
    pub fn create_repository(&mut self, path: &str, master_password: &str) -> CoreResult<()> {
        if self.is_open {
            return Err(CoreError::AlreadyInitialized);
        }

        // Initialize empty memory repository
        self.memory_repo = UnifiedMemoryRepository::new();
        self.memory_repo.initialize()?;

        // Set up manager state
        self.current_path = Some(path.to_string());
        self.master_password = Some(master_password.to_string());
        self.is_open = true;

        // Save the empty repository
        self.save_repository()?;

        Ok(())
    }

    /// Open an existing repository from the specified path
    ///
    /// # Arguments
    /// * `path` - Path to the repository archive
    /// * `master_password` - Password for decrypting the repository
    ///
    /// # Returns
    /// * `Ok(())` - If repository was opened successfully
    /// * `Err(CoreError)` - If opening fails
    pub fn open_repository(&mut self, path: &str, master_password: &str) -> CoreResult<()> {
        if self.is_open {
            return Err(CoreError::AlreadyInitialized);
        }

        // Read archive file
        let archive_data = self.file_provider.read_archive(path)?;

        // Extract archive contents
        let file_map = self
            .file_provider
            .extract_archive(&archive_data, master_password)?;

        // Load into memory repository
        self.memory_repo = UnifiedMemoryRepository::new();
        self.memory_repo.load_from_files(file_map)?;

        // Set up manager state
        self.current_path = Some(path.to_string());
        self.master_password = Some(master_password.to_string());
        self.is_open = true;

        Ok(())
    }

    /// Save the repository to its current path
    ///
    /// # Returns
    /// * `Ok(())` - If save was successful
    /// * `Err(CoreError)` - If save fails
    pub fn save_repository(&mut self) -> CoreResult<()> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        let path = self
            .current_path
            .as_ref()
            .ok_or_else(|| CoreError::StructureError {
                message: "No current path set for repository".to_string(),
            })?
            .clone();

        let password = self
            .master_password
            .as_ref()
            .ok_or_else(|| CoreError::StructureError {
                message: "No master password set for repository".to_string(),
            })?
            .clone();

        self.save_repository_to_path(&path, &password)
    }

    /// Save the repository to a specific path
    ///
    /// # Arguments
    /// * `path` - Path where to save the repository
    /// * `master_password` - Password for encryption
    ///
    /// # Returns
    /// * `Ok(())` - If save was successful
    /// * `Err(CoreError)` - If save fails
    pub fn save_repository_to_path(&mut self, path: &str, master_password: &str) -> CoreResult<()> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        // Serialize memory repository to file map
        let file_map = self.memory_repo.serialize_to_files()?;

        // Create encrypted archive
        let archive_data = self
            .file_provider
            .create_archive(file_map, master_password)?;

        // Write archive to filesystem
        self.file_provider.write_archive(path, &archive_data)?;

        // Mark repository as saved
        self.memory_repo.mark_saved();

        // Update current path if different
        if self.current_path.as_deref() != Some(path) {
            self.current_path = Some(path.to_string());
        }

        // Update password if different
        if self.master_password.as_deref() != Some(master_password) {
            self.master_password = Some(master_password.to_string());
        }

        Ok(())
    }

    /// Close the current repository
    ///
    /// # Arguments
    /// * `save_if_modified` - Whether to save changes before closing
    ///
    /// # Returns
    /// * `Ok(())` - If close was successful
    /// * `Err(CoreError)` - If close fails (e.g., save fails)
    pub fn close_repository(&mut self, save_if_modified: bool) -> CoreResult<()> {
        if !self.is_open {
            return Ok(()); // Already closed
        }

        if save_if_modified && self.memory_repo.is_modified() {
            self.save_repository()?;
        }

        // Reset state
        self.memory_repo = UnifiedMemoryRepository::new();
        self.current_path = None;
        self.master_password = None;
        self.is_open = false;

        Ok(())
    }

    /// Add a new credential to the repository
    pub fn add_credential(&mut self, credential: CredentialRecord) -> CoreResult<()> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.add_credential(credential)
    }

    /// Get a credential by ID
    pub fn get_credential(&mut self, id: &str) -> CoreResult<&CredentialRecord> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.get_credential(id)
    }

    /// Get a credential by ID without updating access time
    pub fn get_credential_readonly(&self, id: &str) -> CoreResult<&CredentialRecord> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.get_credential_readonly(id)
    }

    /// Update an existing credential
    pub fn update_credential(&mut self, credential: CredentialRecord) -> CoreResult<()> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.update_credential(credential)
    }

    /// Delete a credential by ID
    pub fn delete_credential(&mut self, id: &str) -> CoreResult<CredentialRecord> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.delete_credential(id)
    }

    /// List all credentials
    pub fn list_credentials(&self) -> CoreResult<Vec<CredentialRecord>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.list_credentials()
    }

    /// Get credential summaries (ID and title only)
    pub fn list_credential_summaries(&self) -> CoreResult<Vec<(String, String)>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.list_credential_summaries()
    }

    /// Check if repository is currently open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Check if repository has unsaved changes
    pub fn is_modified(&self) -> bool {
        if !self.is_open {
            return false;
        }

        self.memory_repo.is_modified()
    }

    /// Get current repository path
    pub fn current_path(&self) -> Option<&str> {
        self.current_path.as_deref()
    }

    /// Get repository statistics
    pub fn get_stats(&self) -> CoreResult<RepositoryStats> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.get_stats()
    }

    /// Export repository data for backup or migration
    pub fn export_to_file_map(&self) -> CoreResult<FileMap> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.serialize_to_files()
    }

    /// Import repository data from file map
    pub fn import_from_file_map(&mut self, file_map: FileMap) -> CoreResult<()> {
        if self.is_open {
            return Err(CoreError::AlreadyInitialized);
        }

        self.memory_repo = UnifiedMemoryRepository::new();
        self.memory_repo.load_from_files(file_map)?;
        self.is_open = true;

        Ok(())
    }

    /// Change the master password for the repository
    ///
    /// # Arguments
    /// * `new_password` - New password for encryption
    ///
    /// # Returns
    /// * `Ok(())` - If password change was successful
    /// * `Err(CoreError)` - If password change fails
    pub fn change_master_password(&mut self, new_password: &str) -> CoreResult<()> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        // Update stored password
        self.master_password = Some(new_password.to_string());

        // Save with new password (will re-encrypt)
        self.save_repository()
    }

    /// Get credentials by tag
    pub fn get_credentials_by_tag(&self, tag: &str) -> CoreResult<Vec<CredentialRecord>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.get_credentials_by_tag(tag)
    }

    /// Get credentials by type
    pub fn get_credentials_by_type(
        &self,
        credential_type: &str,
    ) -> CoreResult<Vec<CredentialRecord>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.get_credentials_by_type(credential_type)
    }

    /// Get favorite credentials
    pub fn get_favorite_credentials(&self) -> CoreResult<Vec<CredentialRecord>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.get_favorite_credentials()
    }

    /// Import credentials from another source
    pub fn import_credentials(&mut self, credentials: Vec<CredentialRecord>) -> CoreResult<usize> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.import_credentials(credentials)
    }

    /// Export all credentials
    pub fn export_credentials(&self) -> CoreResult<Vec<CredentialRecord>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.export_credentials()
    }

    /// Clear all credentials from repository
    pub fn clear_credentials(&mut self) -> CoreResult<()> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        self.memory_repo.clear()
    }

    /// Check if a credential exists by ID
    pub fn contains_credential(&self, id: &str) -> bool {
        if !self.is_open {
            return false;
        }

        self.memory_repo.contains_credential(id)
    }

    /// Verify repository integrity
    ///
    /// This performs various checks to ensure the repository is in a valid state.
    pub fn verify_integrity(&self) -> CoreResult<Vec<String>> {
        if !self.is_open {
            return Err(CoreError::NotInitialized);
        }

        let mut issues = Vec::new();
        let stats = self.memory_repo.get_stats()?;

        // Check metadata consistency
        if stats.credential_count != stats.metadata.credential_count {
            issues.push(format!(
                "Metadata credential count mismatch: expected {}, found {}",
                stats.metadata.credential_count, stats.credential_count
            ));
        }

        // Validate each credential
        let credentials = self.memory_repo.list_credentials()?;
        for credential in &credentials {
            let validation_result = crate::utils::validation::validate_credential(credential);
            if !validation_result.is_valid {
                issues.push(format!(
                    "Invalid credential '{}': {}",
                    credential.title,
                    validation_result.errors.join("; ")
                ));
            }
        }

        Ok(issues)
    }

    /// Get a reference to the internal memory repository
    ///
    /// This is primarily for advanced use cases and testing.
    pub fn memory_repository(&self) -> &UnifiedMemoryRepository {
        &self.memory_repo
    }

    /// Get a mutable reference to the internal memory repository
    ///
    /// This is primarily for advanced use cases and testing.
    pub fn memory_repository_mut(&mut self) -> &mut UnifiedMemoryRepository {
        &mut self.memory_repo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::file_provider::MockFileProvider;
    use crate::models::{CredentialField, CredentialRecord};

    fn create_test_credential(title: &str) -> CredentialRecord {
        let mut credential = CredentialRecord::new(title.to_string(), "test".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));
        credential
    }

    #[test]
    fn test_repository_creation() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        assert!(!manager.is_open());
        assert!(manager.create_repository("/test.7z", "password").is_ok());
        assert!(manager.is_open());
        assert!(!manager.is_modified()); // Should be clean after creation and save
    }

    #[test]
    fn test_repository_operations() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        manager.create_repository("/test.7z", "password").unwrap();

        let credential = create_test_credential("Test Credential");
        let credential_id = credential.id.clone();

        // Add credential
        assert!(manager.add_credential(credential).is_ok());
        assert!(manager.is_modified());

        // Get credential
        let retrieved = manager.get_credential_readonly(&credential_id).unwrap();
        assert_eq!(retrieved.title, "Test Credential");

        // Update credential
        let mut updated = retrieved.clone();
        updated.title = "Updated Credential".to_string();
        assert!(manager.update_credential(updated).is_ok());

        // Delete credential
        let deleted = manager.delete_credential(&credential_id).unwrap();
        assert_eq!(deleted.title, "Updated Credential");

        // List credentials
        let credentials = manager.list_credentials().unwrap();
        assert_eq!(credentials.len(), 0);
    }

    #[test]
    fn test_save_and_open_cycle() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        // Create and populate repository
        manager.create_repository("/test.7z", "password").unwrap();
        let credential = create_test_credential("Test Credential");
        manager.add_credential(credential).unwrap();

        assert!(manager.save_repository().is_ok());
        assert!(!manager.is_modified());

        // Close repository
        assert!(manager.close_repository(false).is_ok());
        assert!(!manager.is_open());

        // NOTE: In a real scenario with actual files, we would be able to
        // reopen the repository. With the mock provider, we can't fully
        // test the round-trip, but we can test the interface.
    }

    #[test]
    fn test_repository_not_open_errors() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        assert!(manager
            .add_credential(create_test_credential("Test"))
            .is_err());
        assert!(manager.get_credential("test").is_err());
        assert!(manager.list_credentials().is_err());
        assert!(manager.save_repository().is_err());
        assert!(manager.get_stats().is_err());
    }

    #[test]
    fn test_repository_stats() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        manager.create_repository("/test.7z", "password").unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.credential_count, 0);
        assert!(stats.initialized);

        manager
            .add_credential(create_test_credential("Test"))
            .unwrap();
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.credential_count, 1);
    }

    #[test]
    fn test_change_master_password() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        manager.create_repository("/test.7z", "oldpass").unwrap();
        manager
            .add_credential(create_test_credential("Test"))
            .unwrap();

        assert!(manager.change_master_password("newpass").is_ok());
        assert!(!manager.is_modified()); // Should be saved after password change
    }

    #[test]
    fn test_credential_filtering() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        manager.create_repository("/test.7z", "password").unwrap();

        let mut cred1 = create_test_credential("Login 1");
        cred1.credential_type = "login".to_string();
        cred1.add_tag("work".to_string());
        cred1.favorite = true;

        let mut cred2 = create_test_credential("Note 1");
        cred2.credential_type = "note".to_string();
        cred2.add_tag("personal".to_string());

        manager.add_credential(cred1).unwrap();
        manager.add_credential(cred2).unwrap();

        let logins = manager.get_credentials_by_type("login").unwrap();
        assert_eq!(logins.len(), 1);

        let work_creds = manager.get_credentials_by_tag("work").unwrap();
        assert_eq!(work_creds.len(), 1);

        let favorites = manager.get_favorite_credentials().unwrap();
        assert_eq!(favorites.len(), 1);
    }

    #[test]
    fn test_verify_integrity() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        manager.create_repository("/test.7z", "password").unwrap();
        manager
            .add_credential(create_test_credential("Test"))
            .unwrap();

        let issues = manager.verify_integrity().unwrap();
        assert!(issues.is_empty()); // Should have no integrity issues
    }
}
