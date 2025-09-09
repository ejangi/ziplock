//! YAML serialization utilities for ZipLock
//!
//! This module provides utilities for serializing and deserializing
//! credentials and other data structures to/from YAML format.

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::errors::{CoreError, CoreResult};
use crate::core::types::{FileMap, RepositoryMetadata};
use crate::models::CredentialRecord;

/// Serialize a credential record to YAML string
pub fn serialize_credential(credential: &CredentialRecord) -> CoreResult<String> {
    serde_yaml::to_string(credential).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to serialize credential: {}", e),
    })
}

/// Deserialize a credential record from YAML string
pub fn deserialize_credential(yaml: &str) -> CoreResult<CredentialRecord> {
    serde_yaml::from_str(yaml).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to deserialize credential: {}", e),
    })
}

/// Serialize repository metadata to YAML string
pub fn serialize_metadata(metadata: &RepositoryMetadata) -> CoreResult<String> {
    serde_yaml::to_string(metadata).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to serialize metadata: {}", e),
    })
}

/// Deserialize repository metadata from YAML string
pub fn deserialize_metadata(yaml: &str) -> CoreResult<RepositoryMetadata> {
    serde_yaml::from_str(yaml).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to deserialize metadata: {}", e),
    })
}

/// Serialize a file map to JSON string for mobile platform exchange
pub fn serialize_file_map(file_map: &FileMap) -> CoreResult<String> {
    // Convert Vec<u8> to base64 strings for JSON serialization
    let json_map: HashMap<String, String> = file_map
        .iter()
        .map(|(path, data)| (path.clone(), BASE64_STANDARD.encode(data)))
        .collect();

    serde_json::to_string(&json_map).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to serialize file map: {}", e),
    })
}

/// Deserialize a file map from JSON string for mobile platform exchange
pub fn deserialize_file_map(json: &str) -> CoreResult<FileMap> {
    let json_map: HashMap<String, String> =
        serde_json::from_str(json).map_err(|e| CoreError::SerializationError {
            message: format!("Failed to deserialize file map JSON: {}", e),
        })?;

    let mut file_map = HashMap::new();
    for (path, base64_data) in json_map {
        let data =
            BASE64_STANDARD
                .decode(&base64_data)
                .map_err(|e| CoreError::SerializationError {
                    message: format!("Failed to decode base64 data for {}: {}", path, e),
                })?;
        file_map.insert(path, data);
    }

    Ok(file_map)
}

/// Serialize any serializable type to YAML with pretty formatting
pub fn serialize_pretty<T: Serialize>(value: &T) -> CoreResult<String> {
    serde_yaml::to_string(value).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to serialize to YAML: {}", e),
    })
}

/// Deserialize any deserializable type from YAML
pub fn deserialize_from_yaml<T: for<'de> Deserialize<'de>>(yaml: &str) -> CoreResult<T> {
    serde_yaml::from_str(yaml).map_err(|e| CoreError::SerializationError {
        message: format!("Failed to deserialize from YAML: {}", e),
    })
}

/// Convert a credential to a sanitized version for logging/display
/// (removes sensitive field values)
pub fn sanitize_credential_for_log(credential: &CredentialRecord) -> CredentialRecord {
    let mut sanitized = credential.clone();

    // Replace sensitive field values with placeholder
    for (_field_name, field) in sanitized.fields.iter_mut() {
        if field.sensitive {
            field.value = "[REDACTED]".to_string();
        }
    }

    sanitized
}

/// Validate YAML structure for credentials
pub fn validate_credential_yaml(yaml: &str) -> CoreResult<()> {
    // First try to parse as generic YAML to check syntax
    let _: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| CoreError::ValidationError {
            message: format!("Invalid YAML syntax: {}", e),
        })?;

    // Then try to parse as credential to check structure
    let _: CredentialRecord = deserialize_credential(yaml)?;

    Ok(())
}

/// Validate YAML structure for metadata
pub fn validate_metadata_yaml(yaml: &str) -> CoreResult<()> {
    // First try to parse as generic YAML to check syntax
    let _: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| CoreError::ValidationError {
            message: format!("Invalid YAML syntax: {}", e),
        })?;

    // Then try to parse as metadata to check structure
    let _: RepositoryMetadata = deserialize_metadata(yaml)?;

    Ok(())
}

/// Extract field names from a credential YAML without fully deserializing
pub fn extract_field_names_from_yaml(yaml: &str) -> CoreResult<Vec<String>> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| CoreError::SerializationError {
            message: format!("Failed to parse YAML: {}", e),
        })?;

    let fields = value
        .get("fields")
        .and_then(|f| f.as_mapping())
        .ok_or_else(|| CoreError::ValidationError {
            message: "No fields found in credential YAML".to_string(),
        })?;

    Ok(fields
        .keys()
        .filter_map(|k| k.as_str().map(|s| s.to_string()))
        .collect())
}

/// Create a YAML index file for multiple credentials
pub fn create_credentials_index(
    credentials: &HashMap<String, CredentialRecord>,
) -> CoreResult<String> {
    #[derive(Serialize)]
    struct CredentialIndexEntry {
        id: String,
        title: String,
        credential_type: String,
        tags: Vec<String>,
        created_at: i64,
        updated_at: i64,
    }

    #[derive(Serialize)]
    struct CredentialIndex {
        version: String,
        credential_count: usize,
        credentials: Vec<CredentialIndexEntry>,
    }

    let mut index_entries = Vec::new();
    for credential in credentials.values() {
        index_entries.push(CredentialIndexEntry {
            id: credential.id.clone(),
            title: credential.title.clone(),
            credential_type: credential.credential_type.clone(),
            tags: credential.tags.clone(),
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        });
    }

    // Sort by title for consistent ordering
    index_entries.sort_by(|a, b| a.title.cmp(&b.title));

    let index = CredentialIndex {
        version: "1.0".to_string(),
        credential_count: credentials.len(),
        credentials: index_entries,
    };

    serialize_pretty(&index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, CredentialRecord};

    #[test]
    fn test_credential_serialization() {
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));

        let yaml = serialize_credential(&credential).unwrap();
        assert!(yaml.contains("Test Login"));
        assert!(yaml.contains("testuser"));
        assert!(yaml.contains("testpass"));

        let deserialized = deserialize_credential(&yaml).unwrap();
        assert_eq!(credential.id, deserialized.id);
        assert_eq!(credential.title, deserialized.title);
        assert_eq!(credential.fields.len(), deserialized.fields.len());
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = RepositoryMetadata::default();
        let yaml = serialize_metadata(&metadata).unwrap();
        assert!(yaml.contains("version"));
        assert!(yaml.contains("1.0"));

        let deserialized = deserialize_metadata(&yaml).unwrap();
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.format, deserialized.format);
    }

    #[test]
    fn test_file_map_serialization() {
        let mut file_map = HashMap::new();
        file_map.insert("test.txt".to_string(), b"hello world".to_vec());
        file_map.insert("data.bin".to_string(), vec![0, 1, 2, 3, 4]);

        let json = serialize_file_map(&file_map).unwrap();
        let deserialized = deserialize_file_map(&json).unwrap();

        assert_eq!(file_map.len(), deserialized.len());
        assert_eq!(file_map.get("test.txt"), deserialized.get("test.txt"));
        assert_eq!(file_map.get("data.bin"), deserialized.get("data.bin"));
    }

    #[test]
    fn test_credential_sanitization() {
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("secret123"));

        let sanitized = sanitize_credential_for_log(&credential);

        // Username should remain (not sensitive)
        assert_eq!(sanitized.get_field("username").unwrap().value, "testuser");

        // Password should be redacted (sensitive)
        assert_eq!(sanitized.get_field("password").unwrap().value, "[REDACTED]");
    }

    #[test]
    fn test_yaml_validation() {
        let valid_yaml = r#"
id: "test-id"
title: "Test Credential"
credential_type: "login"
fields: {}
tags: []
notes: ""
created_at: 1234567890
updated_at: 1234567890
accessed_at: 1234567890
favorite: false
folder_path: ""
"#;

        assert!(validate_credential_yaml(valid_yaml).is_ok());

        let invalid_yaml = r#"
invalid: yaml: structure: [
"#;

        assert!(validate_credential_yaml(invalid_yaml).is_err());
    }

    #[test]
    fn test_field_names_extraction() {
        let yaml = r#"
id: "test-id"
title: "Test"
credential_type: "login"
fields:
  username:
    field_type: "Username"
    value: "testuser"
    sensitive: false
    label: null
    metadata: {}
  password:
    field_type: "Password"
    value: "testpass"
    sensitive: true
    label: null
    metadata: {}
tags: []
notes: ""
created_at: 1234567890
updated_at: 1234567890
accessed_at: 1234567890
favorite: false
folder_path: ""
"#;

        let field_names = extract_field_names_from_yaml(yaml).unwrap();
        assert_eq!(field_names.len(), 2);
        assert!(field_names.contains(&"username".to_string()));
        assert!(field_names.contains(&"password".to_string()));
    }

    #[test]
    fn test_credentials_index_creation() {
        let mut credentials = HashMap::new();

        let credential1 = CredentialRecord::new("Gmail".to_string(), "login".to_string());
        let credential2 = CredentialRecord::new("Bank Account".to_string(), "login".to_string());

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);

        let index_yaml = create_credentials_index(&credentials).unwrap();
        assert!(index_yaml.contains("Gmail"));
        assert!(index_yaml.contains("Bank Account"));
        assert!(index_yaml.contains("credential_count: 2"));
    }

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let original = RepositoryMetadata::default();
        let yaml = serialize_pretty(&original).unwrap();
        let deserialized: RepositoryMetadata = deserialize_from_yaml(&yaml).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_invalid_base64_in_file_map() {
        let invalid_json = r#"{"test.txt": "invalid base64!!!"}"#;
        let result = deserialize_file_map(invalid_json);
        assert!(result.is_err());
    }
}
