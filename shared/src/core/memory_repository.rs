//! Pure in-memory repository for ZipLock credentials
//!
//! This module provides the core memory-only repository implementation
//! that handles all credential CRUD operations without any file I/O.
//! File operations are delegated to platform-specific providers.

use chrono::Utc;
use std::collections::HashMap;

use crate::core::errors::{CoreError, CoreResult};
use crate::core::types::{
    FileMap, RepositoryMetadata, RepositoryStats, CREDENTIALS_DIR, METADATA_FILE,
};
use crate::models::CredentialRecord;
use crate::utils::yaml::{
    deserialize_credential, deserialize_metadata, serialize_credential, serialize_metadata,
};

/// Pure in-memory repository for credential operations
#[derive(Debug, Clone)]
pub struct UnifiedMemoryRepository {
    /// Whether the repository has been initialized
    initialized: bool,

    /// All credentials stored in memory, keyed by ID
    credentials: HashMap<String, CredentialRecord>,

    /// Repository metadata
    metadata: RepositoryMetadata,

    /// Whether repository has unsaved changes
    modified: bool,
}

impl Default for UnifiedMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedMemoryRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            initialized: false,
            credentials: HashMap::new(),
            metadata: RepositoryMetadata::default(),
            modified: false,
        }
    }

    /// Initialize the repository (marks it as ready for operations)
    pub fn initialize(&mut self) -> CoreResult<()> {
        if self.initialized {
            return Err(CoreError::AlreadyInitialized);
        }

        self.initialized = true;
        self.modified = true;
        self.update_metadata();

        Ok(())
    }

    /// Check if repository is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Load repository from file map (for mobile platforms)
    pub fn load_from_files(&mut self, file_map: FileMap) -> CoreResult<()> {
        if self.initialized {
            return Err(CoreError::AlreadyInitialized);
        }

        // Load metadata
        let metadata_bytes =
            file_map
                .get(METADATA_FILE)
                .ok_or_else(|| CoreError::StructureError {
                    message: "Missing metadata.yml in archive".to_string(),
                })?;

        let metadata_str = String::from_utf8(metadata_bytes.clone()).map_err(|e| {
            CoreError::SerializationError {
                message: format!("Invalid UTF-8 in metadata: {}", e),
            }
        })?;

        self.metadata = deserialize_metadata(&metadata_str)?;

        // Load credentials
        self.credentials.clear();
        for (file_path, file_data) in &file_map {
            // Normalize path separators for cross-platform compatibility
            let normalized_path = file_path.replace('\\', "/");
            if normalized_path.starts_with(CREDENTIALS_DIR)
                && normalized_path.ends_with("/record.yml")
            {
                let credential_str = String::from_utf8(file_data.clone()).map_err(|e| {
                    CoreError::SerializationError {
                        message: format!("Invalid UTF-8 in credential file {}: {}", file_path, e),
                    }
                })?;

                let credential = deserialize_credential(&credential_str)?;
                self.credentials.insert(credential.id.clone(), credential);
            }
        }

        // Validate loaded data with Windows debugging
        #[cfg(windows)]
        {
            eprintln!("DEBUG [Windows]: load_from_files validation");
            eprintln!(
                "DEBUG [Windows]: Loaded credentials: {}",
                self.credentials.len()
            );
            eprintln!(
                "DEBUG [Windows]: Metadata credential_count: {}",
                self.metadata.credential_count
            );
            eprintln!(
                "DEBUG [Windows]: File map entries processed: {}",
                file_map.len()
            );
            eprintln!("DEBUG [Windows]: File map contents:");
            for (path, content) in &file_map {
                eprintln!("DEBUG [Windows]:   '{}': {} bytes", path, content.len());
            }
        }

        if self.credentials.len() != self.metadata.credential_count {
            #[cfg(windows)]
            eprintln!("DEBUG [Windows]: MISMATCH DETECTED - This is the bug!");

            return Err(CoreError::StructureError {
                message: format!(
                    "Metadata claims {} credentials but found {}",
                    self.metadata.credential_count,
                    self.credentials.len()
                ),
            });
        }

        self.initialized = true;
        self.modified = false;

        // Repair any credentials with missing or empty IDs
        if let Ok(repaired_count) = self.repair_all_credentials() {
            if repaired_count > 0 {
                eprintln!(
                    "DEBUG: Repaired {} credentials after loading from archive",
                    repaired_count
                );
            }
        }

        Ok(())
    }

    /// Serialize repository to file map (for mobile platforms)
    pub fn serialize_to_files(&self) -> CoreResult<FileMap> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        let mut file_map = HashMap::new();

        // Windows-specific debugging
        #[cfg(windows)]
        {
            eprintln!("DEBUG [Windows]: serialize_to_files starting");
            eprintln!(
                "DEBUG [Windows]: Credential count: {}",
                self.credentials.len()
            );
            eprintln!(
                "DEBUG [Windows]: Metadata credential_count: {}",
                self.metadata.credential_count
            );
        }

        // Serialize metadata
        let metadata_yaml = serialize_metadata(&self.metadata)?;
        let metadata_len = metadata_yaml.len();
        file_map.insert(METADATA_FILE.to_string(), metadata_yaml.into_bytes());

        #[cfg(windows)]
        eprintln!(
            "DEBUG [Windows]: Added metadata file: {} ({} bytes)",
            METADATA_FILE, metadata_len
        );

        // Serialize each credential
        for credential in self.credentials.values() {
            let credential_yaml = serialize_credential(credential)?;
            let file_path = format!("{}/{}/record.yml", CREDENTIALS_DIR, credential.id);

            #[cfg(windows)]
            {
                eprintln!(
                    "DEBUG [Windows]: Serializing credential ID: {}",
                    credential.id
                );
                eprintln!("DEBUG [Windows]: File path: '{}'", file_path);
                eprintln!(
                    "DEBUG [Windows]: YAML size: {} bytes",
                    credential_yaml.len()
                );
            }

            file_map.insert(file_path, credential_yaml.into_bytes());
        }

        #[cfg(windows)]
        {
            eprintln!("DEBUG [Windows]: serialize_to_files complete");
            eprintln!("DEBUG [Windows]: Total files in map: {}", file_map.len());
            for (path, content) in &file_map {
                eprintln!("DEBUG [Windows]:   '{}': {} bytes", path, content.len());
            }
        }

        Ok(file_map)
    }

    /// Add a new credential
    pub fn add_credential(&mut self, mut credential: CredentialRecord) -> CoreResult<()> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        // Repair credential ID if missing or empty
        let was_repaired = crate::utils::validation::repair_credential_id(&mut credential);
        if was_repaired {
            eprintln!("DEBUG: Generated new ID for credential: {}", credential.id);
        }

        // Validate the credential
        let validation_result = crate::utils::validation::validate_credential(&credential);
        if !validation_result.is_valid {
            return Err(CoreError::ValidationError {
                message: validation_result.errors.join("; "),
            });
        }

        // Check for duplicate ID
        if self.credentials.contains_key(&credential.id) {
            return Err(CoreError::ValidationError {
                message: format!("Credential with ID '{}' already exists", credential.id),
            });
        }

        // Update timestamps
        let now = Utc::now().timestamp();
        credential.created_at = now;
        credential.updated_at = now;
        credential.accessed_at = now;

        self.credentials.insert(credential.id.clone(), credential);
        self.modified = true;
        self.update_metadata();

        Ok(())
    }

    /// Get a credential by ID
    pub fn get_credential(&mut self, id: &str) -> CoreResult<&CredentialRecord> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        let credential = self
            .credentials
            .get_mut(id)
            .ok_or_else(|| CoreError::CredentialNotFound { id: id.to_string() })?;

        // Update accessed timestamp
        credential.accessed_at = Utc::now().timestamp();
        self.modified = true;

        Ok(credential)
    }

    /// Get a credential by ID without updating access time
    pub fn get_credential_readonly(&self, id: &str) -> CoreResult<&CredentialRecord> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        self.credentials
            .get(id)
            .ok_or_else(|| CoreError::CredentialNotFound { id: id.to_string() })
    }

    /// Update an existing credential
    pub fn update_credential(&mut self, mut credential: CredentialRecord) -> CoreResult<()> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        // Repair credential ID if missing or empty
        let original_id = credential.id.clone();
        let was_repaired = crate::utils::validation::repair_credential_id(&mut credential);
        if was_repaired {
            eprintln!(
                "DEBUG: Repaired credential ID from '{}' to '{}'",
                original_id, credential.id
            );
        }

        // For empty IDs, find credential by matching title since we can't lookup by empty key
        let lookup_id = if original_id.is_empty() {
            // Find credential with matching title
            let matching_credential = self
                .credentials
                .iter()
                .find(|(_, cred)| cred.title == credential.title && cred.id.is_empty());

            if matching_credential.is_none() {
                return Err(CoreError::CredentialNotFound {
                    id: format!("credential with title '{}' and empty ID", credential.title),
                });
            }
            &original_id // Use empty ID for removal
        } else {
            // Normal case - check if credential exists
            if !self.credentials.contains_key(&credential.id) {
                return Err(CoreError::CredentialNotFound {
                    id: credential.id.clone(),
                });
            }
            &credential.id
        };

        // Validate the credential
        let validation_result = crate::utils::validation::validate_credential(&credential);
        if !validation_result.is_valid {
            return Err(CoreError::ValidationError {
                message: validation_result.errors.join("; "),
            });
        }

        // Preserve created_at, update other timestamps
        if let Some(existing) = self.credentials.get(lookup_id) {
            credential.created_at = existing.created_at;
        }
        credential.updated_at = Utc::now().timestamp();
        credential.accessed_at = Utc::now().timestamp();

        // Remove old entry (either empty ID or changed ID)
        self.credentials.remove(lookup_id);

        // Insert with new ID
        self.credentials.insert(credential.id.clone(), credential);
        eprintln!(
            "DEBUG: Updated credential - old key: '{}', new key: '{}'",
            original_id,
            self.credentials
                .keys()
                .last()
                .unwrap_or(&"<none>".to_string())
        );
        self.modified = true;
        self.update_metadata();

        Ok(())
    }

    /// Repair all credentials by ensuring they have valid IDs
    /// This should be called after loading credentials from archives
    pub fn repair_all_credentials(&mut self) -> CoreResult<usize> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        let mut repaired_count = 0;
        let mut credentials_to_update = Vec::new();

        // Collect credentials that need repair
        for (old_id, credential) in &self.credentials {
            if credential.id.is_empty() {
                let mut repaired_credential = credential.clone();
                crate::utils::validation::repair_credential_id(&mut repaired_credential);
                credentials_to_update.push((old_id.clone(), repaired_credential));
            }
        }

        // Apply repairs
        for (old_id, repaired_credential) in credentials_to_update {
            eprintln!(
                "DEBUG: Repairing credential '{}' - changing ID from '{}' to '{}'",
                repaired_credential.title, old_id, repaired_credential.id
            );

            self.credentials.remove(&old_id);
            self.credentials
                .insert(repaired_credential.id.clone(), repaired_credential);
            repaired_count += 1;
        }

        if repaired_count > 0 {
            self.modified = true;
            self.update_metadata();
            eprintln!(
                "DEBUG: Repaired {} credentials with missing IDs",
                repaired_count
            );
        }

        Ok(repaired_count)
    }

    /// Delete a credential by ID
    pub fn delete_credential(&mut self, id: &str) -> CoreResult<CredentialRecord> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        let credential = self
            .credentials
            .remove(id)
            .ok_or_else(|| CoreError::CredentialNotFound { id: id.to_string() })?;

        self.modified = true;
        self.update_metadata();

        Ok(credential)
    }

    /// List all credentials (returns cloned credentials)
    pub fn list_credentials(&self) -> CoreResult<Vec<CredentialRecord>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(self.credentials.values().cloned().collect())
    }

    /// Get credential IDs and titles for listings
    pub fn list_credential_summaries(&self) -> CoreResult<Vec<(String, String)>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(self
            .credentials
            .values()
            .map(|c| (c.id.clone(), c.title.clone()))
            .collect())
    }

    /// Get all credentials as a reference to the internal map
    pub fn get_credentials_ref(&self) -> CoreResult<&HashMap<String, CredentialRecord>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(&self.credentials)
    }

    /// Check if repository has unsaved changes
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark repository as saved (clears modified flag)
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// Get repository statistics
    pub fn get_stats(&self) -> CoreResult<RepositoryStats> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(RepositoryStats {
            credential_count: self.credentials.len(),
            metadata: self.metadata.clone(),
            initialized: self.initialized,
            modified: self.modified,
        })
    }

    /// Get repository metadata
    pub fn get_metadata(&self) -> &RepositoryMetadata {
        &self.metadata
    }

    /// Clear all credentials and reset repository
    pub fn clear(&mut self) -> CoreResult<()> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        self.credentials.clear();
        self.modified = true;
        self.update_metadata();

        Ok(())
    }

    /// Check if a credential exists by ID
    pub fn contains_credential(&self, id: &str) -> bool {
        self.credentials.contains_key(id)
    }

    /// Update repository metadata based on current state
    fn update_metadata(&mut self) {
        self.metadata.credential_count = self.credentials.len();
        self.metadata.last_modified = Utc::now().timestamp();
    }

    /// Import credentials from another repository
    pub fn import_credentials(&mut self, credentials: Vec<CredentialRecord>) -> CoreResult<usize> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        let mut imported_count = 0;
        let mut errors = Vec::new();

        for credential in credentials {
            match self.add_credential(credential.clone()) {
                Ok(()) => imported_count += 1,
                Err(e) => {
                    // For import, we continue on validation errors but collect them
                    errors.push(format!("Failed to import '{}': {}", credential.title, e));

                    // If it's a duplicate ID, try with a new ID
                    if matches!(e, CoreError::ValidationError { .. })
                        && e.to_string().contains("already exists")
                    {
                        let mut new_credential = credential;
                        new_credential.id = uuid::Uuid::new_v4().to_string();

                        if self.add_credential(new_credential).is_ok() {
                            imported_count += 1;
                            errors.pop(); // Remove the error since we recovered
                        }
                    }
                }
            }
        }

        // If we had errors but imported some credentials, log the errors but don't fail
        if !errors.is_empty() && imported_count == 0 {
            return Err(CoreError::ValidationError {
                message: errors.join("; "),
            });
        }

        Ok(imported_count)
    }

    /// Export all credentials
    pub fn export_credentials(&self) -> CoreResult<Vec<CredentialRecord>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(self.credentials.values().cloned().collect())
    }

    /// Get credentials by tag
    pub fn get_credentials_by_tag(&self, tag: &str) -> CoreResult<Vec<CredentialRecord>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(self
            .credentials
            .values()
            .filter(|c| c.has_tag(tag))
            .cloned()
            .collect())
    }

    /// Get credentials by type
    pub fn get_credentials_by_type(
        &self,
        credential_type: &str,
    ) -> CoreResult<Vec<CredentialRecord>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(self
            .credentials
            .values()
            .filter(|c| c.credential_type == credential_type)
            .cloned()
            .collect())
    }

    /// Get favorite credentials
    pub fn get_favorite_credentials(&self) -> CoreResult<Vec<CredentialRecord>> {
        if !self.initialized {
            return Err(CoreError::NotInitialized);
        }

        Ok(self
            .credentials
            .values()
            .filter(|c| c.favorite)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, CredentialRecord};

    fn create_test_credential(title: &str) -> CredentialRecord {
        let mut credential = CredentialRecord::new(title.to_string(), "test".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));
        credential
    }

    #[test]
    fn test_repository_lifecycle() {
        let mut repo = UnifiedMemoryRepository::new();

        // Should not be initialized initially
        assert!(!repo.is_initialized());
        assert!(repo.add_credential(create_test_credential("Test")).is_err());

        // Initialize repository
        assert!(repo.initialize().is_ok());
        assert!(repo.is_initialized());
        assert!(repo.is_modified());

        // Should not be able to initialize twice
        assert!(repo.initialize().is_err());
    }

    #[test]
    fn test_credential_operations() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        let credential = create_test_credential("Test Credential");
        let credential_id = credential.id.clone();

        // Add credential
        assert!(repo.add_credential(credential).is_ok());
        assert_eq!(repo.credentials.len(), 1);

        // Get credential
        let retrieved = repo.get_credential_readonly(&credential_id).unwrap();
        assert_eq!(retrieved.title, "Test Credential");

        // Update credential
        let mut updated = retrieved.clone();
        updated.title = "Updated Credential".to_string();
        assert!(repo.update_credential(updated).is_ok());

        let retrieved = repo.get_credential_readonly(&credential_id).unwrap();
        assert_eq!(retrieved.title, "Updated Credential");

        // Delete credential
        let deleted = repo.delete_credential(&credential_id).unwrap();
        assert_eq!(deleted.title, "Updated Credential");
        assert_eq!(repo.credentials.len(), 0);

        // Should not find deleted credential
        assert!(repo.get_credential_readonly(&credential_id).is_err());
    }

    #[test]
    fn test_file_serialization() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        // Add some test credentials
        let cred1 = create_test_credential("Credential 1");
        let cred2 = create_test_credential("Credential 2");

        repo.add_credential(cred1).unwrap();
        repo.add_credential(cred2).unwrap();

        // Serialize to file map
        let file_map = repo.serialize_to_files().unwrap();
        assert!(file_map.contains_key(METADATA_FILE));
        assert!(file_map.len() > 2); // Metadata + 2 credentials

        // Create new repository and load from file map
        let mut new_repo = UnifiedMemoryRepository::new();
        assert!(new_repo.load_from_files(file_map).is_ok());

        assert!(new_repo.is_initialized());
        assert_eq!(new_repo.credentials.len(), 2);
        assert!(!new_repo.is_modified()); // Should not be modified after load
    }

    #[test]
    fn test_repository_stats() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        let stats = repo.get_stats().unwrap();
        assert_eq!(stats.credential_count, 0);
        assert!(stats.initialized);
        assert!(stats.modified);

        // Add credential and check stats
        repo.add_credential(create_test_credential("Test")).unwrap();
        let stats = repo.get_stats().unwrap();
        assert_eq!(stats.credential_count, 1);
    }

    #[test]
    fn test_duplicate_credential_id() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        let cred1 = create_test_credential("First");
        let mut cred2 = create_test_credential("Second");
        cred2.id = cred1.id.clone(); // Same ID

        assert!(repo.add_credential(cred1).is_ok());
        assert!(repo.add_credential(cred2).is_err()); // Should fail
    }

    #[test]
    fn test_repository_not_initialized_errors() {
        let mut repo = UnifiedMemoryRepository::new();

        assert!(repo.add_credential(create_test_credential("Test")).is_err());
        assert!(repo.get_credential("test").is_err());
        assert!(repo.list_credentials().is_err());
        assert!(repo.delete_credential("test").is_err());
        assert!(repo.get_stats().is_err());
    }

    #[test]
    fn test_credential_filtering() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        let mut cred1 = create_test_credential("Login 1");
        cred1.credential_type = "login".to_string();
        cred1.add_tag("work".to_string());
        cred1.favorite = true;

        let mut cred2 = create_test_credential("Note 1");
        cred2.credential_type = "note".to_string();
        cred2.add_tag("personal".to_string());

        let mut cred3 = create_test_credential("Login 2");
        cred3.credential_type = "login".to_string();
        cred3.add_tag("work".to_string());

        repo.add_credential(cred1).unwrap();
        repo.add_credential(cred2).unwrap();
        repo.add_credential(cred3).unwrap();

        // Test filtering by type
        let logins = repo.get_credentials_by_type("login").unwrap();
        assert_eq!(logins.len(), 2);

        // Test filtering by tag
        let work_creds = repo.get_credentials_by_tag("work").unwrap();
        assert_eq!(work_creds.len(), 2);

        // Test filtering favorites
        let favorites = repo.get_favorite_credentials().unwrap();
        assert_eq!(favorites.len(), 1);
    }

    #[test]
    fn test_import_export() {
        let mut repo1 = UnifiedMemoryRepository::new();
        repo1.initialize().unwrap();

        let cred1 = create_test_credential("Credential 1");
        let cred2 = create_test_credential("Credential 2");

        repo1.add_credential(cred1).unwrap();
        repo1.add_credential(cred2).unwrap();

        // Export credentials
        let exported = repo1.export_credentials().unwrap();
        assert_eq!(exported.len(), 2);

        // Import into new repository
        let mut repo2 = UnifiedMemoryRepository::new();
        repo2.initialize().unwrap();

        let imported_count = repo2.import_credentials(exported).unwrap();
        assert_eq!(imported_count, 2);
        assert_eq!(repo2.credentials.len(), 2);
    }

    #[test]
    fn test_mark_saved() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        assert!(repo.is_modified());
        repo.mark_saved();
        assert!(!repo.is_modified());

        repo.add_credential(create_test_credential("Test")).unwrap();
        assert!(repo.is_modified());
    }
}
