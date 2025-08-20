//! Memory-based repository management for ZipLock
//!
//! This module provides a centralized, in-memory repository manager that handles
//! file structure design and content management consistently across all platforms.
//! The Android app and other platforms can use this to get instructions about
//! which files/folders to create and their content, eliminating platform-specific
//! file structure logic.

use crate::models::{CredentialField, CredentialRecord};
use crate::validation::validate_credential;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors that can occur during memory repository operations
#[derive(Error, Debug, Clone)]
pub enum MemoryRepositoryError {
    #[error("Credential not found: {id}")]
    CredentialNotFound { id: String },

    #[error("Repository not initialized")]
    NotInitialized,

    #[error("Repository already initialized")]
    AlreadyInitialized,

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Invalid credential data: {message}")]
    InvalidCredential { message: String },

    #[error("File operation error: {message}")]
    FileOperationError { message: String },

    #[error("Repository structure error: {message}")]
    StructureError { message: String },
}

/// Result type for memory repository operations
pub type MemoryRepositoryResult<T> = Result<T, MemoryRepositoryError>;

/// Represents a file operation that the platform should perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    /// Operation type: "create", "update", "delete"
    pub operation: String,
    /// Relative path within the repository
    pub path: String,
    /// File content (None for directories or delete operations)
    pub content: Option<Vec<u8>>,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File metadata
    pub metadata: HashMap<String, String>,
}

/// Information about a file in the repository structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryFileInfo {
    /// Relative path within repository
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Last modified timestamp
    pub modified: u64,
    /// File permissions (platform-specific)
    pub permissions: Option<String>,
    /// Content hash for integrity checking
    pub content_hash: Option<String>,
}

/// Complete repository structure definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStructure {
    /// Repository format version
    pub version: String,
    /// All files and directories in the repository
    pub files: Vec<RepositoryFileInfo>,
    /// Repository metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modification timestamp
    pub modified_at: u64,
}

/// Repository metadata that goes in metadata.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub version: String,
    pub format: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub credential_count: u32,
    pub structure_version: String,
    pub generator: String,
}

/// Serialized credential data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCredential {
    pub id: String,
    pub title: String,
    pub credential_type: String,
    pub fields: HashMap<String, SerializedField>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Serialized field data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedField {
    pub value: String,
    pub field_type: String,
    pub sensitive: bool,
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub validation: Option<String>,
}

/// In-memory repository manager that handles all file structure logic
pub struct MemoryRepository {
    /// Whether the repository is initialized
    initialized: bool,
    /// All credentials stored in memory
    credentials: HashMap<String, CredentialRecord>,
    /// Repository metadata
    metadata: RepositoryMetadata,
    /// Pending file operations
    #[allow(dead_code)]
    pending_operations: Vec<FileOperation>,
    /// Repository structure
    structure: RepositoryStructure,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRepository {
    /// Create a new memory repository
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            initialized: false,
            credentials: HashMap::new(),
            metadata: RepositoryMetadata {
                version: "1.0".to_string(),
                format: "memory-v1".to_string(),
                created_at: now,
                last_modified: now,
                credential_count: 0,
                structure_version: "1.0".to_string(),
                generator: "ziplock-shared".to_string(),
            },
            pending_operations: Vec::new(),
            structure: RepositoryStructure {
                version: "1.0".to_string(),
                files: Vec::new(),
                metadata: HashMap::new(),
                created_at: now,
                modified_at: now,
            },
        }
    }

    /// Initialize the repository
    pub fn initialize(&mut self) -> MemoryRepositoryResult<()> {
        if self.initialized {
            return Err(MemoryRepositoryError::AlreadyInitialized);
        }

        self.initialized = true;
        self.generate_base_structure()?;

        Ok(())
    }

    /// Check if repository is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Load repository from extracted archive content
    pub fn load_from_content(
        &mut self,
        files: HashMap<String, Vec<u8>>,
    ) -> MemoryRepositoryResult<()> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        // Load metadata
        if let Some(metadata_content) = files.get("metadata.yml") {
            let metadata_str = String::from_utf8(metadata_content.clone()).map_err(|e| {
                MemoryRepositoryError::SerializationError {
                    message: format!("Invalid UTF-8 in metadata.yml: {}", e),
                }
            })?;

            self.metadata = serde_yaml::from_str(&metadata_str).map_err(|e| {
                MemoryRepositoryError::SerializationError {
                    message: format!("Failed to parse metadata.yml: {}", e),
                }
            })?;
        }

        // Load credentials
        if let Some(credentials_content) = files.get("credentials/index.yml") {
            let credentials_str = String::from_utf8(credentials_content.clone()).map_err(|e| {
                MemoryRepositoryError::SerializationError {
                    message: format!("Invalid UTF-8 in credentials/index.yml: {}", e),
                }
            })?;

            let serialized_credentials: Vec<SerializedCredential> =
                serde_yaml::from_str(&credentials_str).map_err(|e| {
                    MemoryRepositoryError::SerializationError {
                        message: format!("Failed to parse credentials/index.yml: {}", e),
                    }
                })?;

            // Convert serialized credentials to CredentialRecord
            for serialized in serialized_credentials {
                let credential = self.deserialize_credential(serialized)?;
                self.credentials.insert(credential.id.clone(), credential);
            }
        }

        // Load individual credential files (repository format v1.0: /credentials/credential-id/record.yml)
        for (path, content) in files.iter() {
            if path.starts_with("credentials/") && path.ends_with("/record.yml") {
                let content_str = String::from_utf8(content.clone()).map_err(|e| {
                    MemoryRepositoryError::SerializationError {
                        message: format!("Invalid UTF-8 in {}: {}", path, e),
                    }
                })?;

                // Deserialize directly as CredentialRecord (not SerializedCredential)
                let credential: CredentialRecord =
                    serde_yaml::from_str(&content_str).map_err(|e| {
                        MemoryRepositoryError::SerializationError {
                            message: format!("Failed to parse {}: {}", path, e),
                        }
                    })?;

                self.credentials.insert(credential.id.clone(), credential);
            }
        }

        self.update_metadata();
        Ok(())
    }

    /// Get all file operations needed to persist the repository
    pub fn get_file_operations(&mut self) -> MemoryRepositoryResult<Vec<FileOperation>> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        let mut operations = Vec::new();

        // Create base directory structure
        operations.push(FileOperation {
            operation: "create".to_string(),
            path: "credentials".to_string(),
            content: None,
            is_directory: true,
            metadata: HashMap::new(),
        });

        operations.push(FileOperation {
            operation: "create".to_string(),
            path: "attachments".to_string(),
            content: None,
            is_directory: true,
            metadata: HashMap::new(),
        });

        // Create metadata.yml
        let metadata_yaml = serde_yaml::to_string(&self.metadata).map_err(|e| {
            MemoryRepositoryError::SerializationError {
                message: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        operations.push(FileOperation {
            operation: "create".to_string(),
            path: "metadata.yml".to_string(),
            content: Some(metadata_yaml.into_bytes()),
            is_directory: false,
            metadata: HashMap::from([(
                "content-type".to_string(),
                "application/x-yaml".to_string(),
            )]),
        });

        // Create credentials index
        let serialized_credentials: Vec<SerializedCredential> = self
            .credentials
            .values()
            .map(|cred| self.serialize_credential(cred))
            .collect::<Result<Vec<_>, _>>()?;

        let credentials_yaml = serde_yaml::to_string(&serialized_credentials).map_err(|e| {
            MemoryRepositoryError::SerializationError {
                message: format!("Failed to serialize credentials: {}", e),
            }
        })?;

        operations.push(FileOperation {
            operation: "create".to_string(),
            path: "credentials/index.yml".to_string(),
            content: Some(credentials_yaml.into_bytes()),
            is_directory: false,
            metadata: HashMap::from([(
                "content-type".to_string(),
                "application/x-yaml".to_string(),
            )]),
        });

        // Create individual credential files using repository format v1.0: /credentials/credential-id/record.yml
        for credential in self.credentials.values() {
            // Create directory for each credential
            operations.push(FileOperation {
                operation: "create".to_string(),
                path: format!("credentials/{}", credential.id),
                content: None,
                is_directory: true,
                metadata: HashMap::new(),
            });

            let credential_yaml = serde_yaml::to_string(credential).map_err(|e| {
                MemoryRepositoryError::SerializationError {
                    message: format!("Failed to serialize credential {}: {}", credential.id, e),
                }
            })?;

            operations.push(FileOperation {
                operation: "create".to_string(),
                path: format!("credentials/{}/record.yml", credential.id),
                content: Some(credential_yaml.into_bytes()),
                is_directory: false,
                metadata: HashMap::from([
                    ("content-type".to_string(), "application/x-yaml".to_string()),
                    ("credential-id".to_string(), credential.id.clone()),
                ]),
            });
        }

        Ok(operations)
    }

    /// Add a new credential
    pub fn add_credential(
        &mut self,
        credential: CredentialRecord,
    ) -> MemoryRepositoryResult<String> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        // Validate the credential
        validate_credential(&credential).map_err(|e| MemoryRepositoryError::ValidationError {
            message: e.to_string(),
        })?;

        let id = credential.id.clone();
        self.credentials.insert(id.clone(), credential);
        self.update_metadata();

        Ok(id)
    }

    /// Get a credential by ID
    pub fn get_credential(&self, id: &str) -> MemoryRepositoryResult<&CredentialRecord> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        self.credentials
            .get(id)
            .ok_or_else(|| MemoryRepositoryError::CredentialNotFound { id: id.to_string() })
    }

    /// Update an existing credential
    pub fn update_credential(
        &mut self,
        credential: CredentialRecord,
    ) -> MemoryRepositoryResult<()> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        // Validate the credential
        validate_credential(&credential).map_err(|e| MemoryRepositoryError::ValidationError {
            message: e.to_string(),
        })?;

        let id = credential.id.clone();
        if !self.credentials.contains_key(&id) {
            return Err(MemoryRepositoryError::CredentialNotFound { id });
        }

        self.credentials.insert(id, credential);
        self.update_metadata();

        Ok(())
    }

    /// Delete a credential
    pub fn delete_credential(&mut self, id: &str) -> MemoryRepositoryResult<()> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        if !self.credentials.contains_key(id) {
            return Err(MemoryRepositoryError::CredentialNotFound { id: id.to_string() });
        }

        self.credentials.remove(id);
        self.update_metadata();

        Ok(())
    }

    /// List all credentials
    pub fn list_credentials(&self) -> MemoryRepositoryResult<Vec<&CredentialRecord>> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        Ok(self.credentials.values().collect())
    }

    /// Get repository metadata
    pub fn get_metadata(&self) -> &RepositoryMetadata {
        &self.metadata
    }

    /// Get repository structure
    pub fn get_structure(&self) -> &RepositoryStructure {
        &self.structure
    }

    /// Search credentials by title or content
    pub fn search_credentials(
        &self,
        query: &str,
    ) -> MemoryRepositoryResult<Vec<&CredentialRecord>> {
        if !self.initialized {
            return Err(MemoryRepositoryError::NotInitialized);
        }

        let query_lower = query.to_lowercase();
        let results: Vec<&CredentialRecord> = self
            .credentials
            .values()
            .filter(|credential| {
                // Search in title
                credential.title.to_lowercase().contains(&query_lower) ||
                // Search in credential type
                credential.credential_type.to_lowercase().contains(&query_lower) ||
                // Search in field values (non-sensitive only for security)
                credential.fields.values().any(|field| {
                    !field.sensitive && field.value.to_lowercase().contains(&query_lower)
                })
            })
            .collect();

        Ok(results)
    }

    /// Get credential count
    pub fn get_credential_count(&self) -> u32 {
        self.credentials.len() as u32
    }

    /// Private helper methods
    fn generate_base_structure(&mut self) -> MemoryRepositoryResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.structure.files = vec![
            RepositoryFileInfo {
                path: "metadata.yml".to_string(),
                size: 0, // Will be calculated when serialized
                is_directory: false,
                modified: now,
                permissions: Some("644".to_string()),
                content_hash: None,
            },
            RepositoryFileInfo {
                path: "credentials".to_string(),
                size: 0,
                is_directory: true,
                modified: now,
                permissions: Some("755".to_string()),
                content_hash: None,
            },
            RepositoryFileInfo {
                path: "credentials/index.yml".to_string(),
                size: 0,
                is_directory: false,
                modified: now,
                permissions: Some("644".to_string()),
                content_hash: None,
            },
            RepositoryFileInfo {
                path: "attachments".to_string(),
                size: 0,
                is_directory: true,
                modified: now,
                permissions: Some("755".to_string()),
                content_hash: None,
            },
        ];

        self.structure.modified_at = now;
        Ok(())
    }

    fn update_metadata(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.metadata.last_modified = now;
        self.metadata.credential_count = self.credentials.len() as u32;
        self.structure.modified_at = now;
    }

    fn serialize_credential(
        &self,
        credential: &CredentialRecord,
    ) -> MemoryRepositoryResult<SerializedCredential> {
        let mut serialized_fields = HashMap::new();

        for (key, field) in &credential.fields {
            serialized_fields.insert(
                key.clone(),
                SerializedField {
                    value: field.value.clone(),
                    field_type: format!("{:?}", field.field_type),
                    sensitive: field.sensitive,
                    label: field.label.clone(),
                    placeholder: field.metadata.get("placeholder").cloned(),
                    validation: field.metadata.get("validation").cloned(),
                },
            );
        }

        Ok(SerializedCredential {
            id: credential.id.clone(),
            title: credential.title.clone(),
            credential_type: credential.credential_type.clone(),
            fields: serialized_fields,
            tags: credential.tags.to_vec(),
            notes: credential.notes.clone(),
            created_at: credential
                .created_at
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: credential
                .updated_at
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    fn deserialize_credential(
        &self,
        serialized: SerializedCredential,
    ) -> MemoryRepositoryResult<CredentialRecord> {
        let mut fields = HashMap::new();

        for (key, field) in serialized.fields {
            // Parse field type from string representation
            let field_type = match field.field_type.as_str() {
                "Text" => crate::models::FieldType::Text,
                "Password" => crate::models::FieldType::Password,
                "Email" => crate::models::FieldType::Email,
                "Url" => crate::models::FieldType::Url,
                "Username" => crate::models::FieldType::Username,
                "Phone" => crate::models::FieldType::Phone,
                "CreditCardNumber" => crate::models::FieldType::CreditCardNumber,
                "ExpiryDate" => crate::models::FieldType::ExpiryDate,
                "Cvv" => crate::models::FieldType::Cvv,
                "TotpSecret" => crate::models::FieldType::TotpSecret,
                "TextArea" => crate::models::FieldType::TextArea,
                "Number" => crate::models::FieldType::Number,
                "Date" => crate::models::FieldType::Date,
                _ => crate::models::FieldType::Text,
            };

            let mut metadata = HashMap::new();
            if let Some(placeholder) = field.placeholder {
                metadata.insert("placeholder".to_string(), placeholder);
            }
            if let Some(validation) = field.validation {
                metadata.insert("validation".to_string(), validation);
            }

            fields.insert(
                key,
                CredentialField {
                    value: field.value,
                    field_type,
                    sensitive: field.sensitive,
                    label: field.label,
                    metadata,
                },
            );
        }

        Ok(CredentialRecord {
            id: serialized.id,
            title: serialized.title,
            credential_type: serialized.credential_type,
            fields,
            tags: serialized.tags.into_iter().collect(),
            notes: serialized.notes,
            created_at: UNIX_EPOCH + std::time::Duration::from_secs(serialized.created_at),
            updated_at: UNIX_EPOCH + std::time::Duration::from_secs(serialized.updated_at),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, CredentialRecord};

    #[test]
    fn test_memory_repository_lifecycle() {
        let mut repo = MemoryRepository::new();

        // Test initialization
        assert!(!repo.is_initialized());
        repo.initialize().unwrap();
        assert!(repo.is_initialized());

        // Test adding credential
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::email("user@example.com"));
        credential.set_field("password", CredentialField::password("secret123"));

        let id = repo.add_credential(credential.clone()).unwrap();
        assert_eq!(id, credential.id);
        assert_eq!(repo.get_credential_count(), 1);

        // Test getting credential
        let retrieved = repo.get_credential(&id).unwrap();
        assert_eq!(retrieved.title, "Test Login");

        // Test updating credential
        let mut updated_credential = credential.clone();
        updated_credential.title = "Updated Login".to_string();
        repo.update_credential(updated_credential).unwrap();

        let retrieved = repo.get_credential(&id).unwrap();
        assert_eq!(retrieved.title, "Updated Login");

        // Test file operations
        let operations = repo.get_file_operations().unwrap();
        assert!(!operations.is_empty());

        // Should have base structure + credential files
        let dir_ops: Vec<_> = operations.iter().filter(|op| op.is_directory).collect();
        let file_ops: Vec<_> = operations.iter().filter(|op| !op.is_directory).collect();

        assert!(dir_ops.len() >= 2); // credentials/, attachments/
        assert!(file_ops.len() >= 3); // metadata.yml, credentials/index.yml, credentials/{id}.yml

        // Test search
        let results = repo.search_credentials("Updated").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Updated Login");

        // Test deletion
        repo.delete_credential(&id).unwrap();
        assert_eq!(repo.get_credential_count(), 0);
        assert!(repo.get_credential(&id).is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut repo = MemoryRepository::new();
        repo.initialize().unwrap();

        // Create test credential
        let mut credential = CredentialRecord::new("Test".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::email("user@test.com"));
        credential.set_field("password", CredentialField::password("secret"));
        credential.add_tag("test");

        // Serialize and deserialize
        let serialized = repo.serialize_credential(&credential).unwrap();
        let deserialized = repo.deserialize_credential(serialized).unwrap();

        // Verify data integrity
        assert_eq!(credential.id, deserialized.id);
        assert_eq!(credential.title, deserialized.title);
        assert_eq!(credential.credential_type, deserialized.credential_type);
        assert_eq!(credential.fields.len(), deserialized.fields.len());
        assert!(credential.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_repository_structure() {
        let mut repo = MemoryRepository::new();
        repo.initialize().unwrap();

        let structure = repo.get_structure();
        assert_eq!(structure.version, "1.0");
        assert!(!structure.files.is_empty());

        // Should have basic structure files
        let paths: Vec<_> = structure.files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&"metadata.yml".to_string()));
        assert!(paths.contains(&&"credentials".to_string()));
        assert!(paths.contains(&&"credentials/index.yml".to_string()));
        assert!(paths.contains(&&"attachments".to_string()));
    }
}
