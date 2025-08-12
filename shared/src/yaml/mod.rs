//! YAML serialization and deserialization for ZipLock credentials
//!
//! This module provides utilities for converting credential records to and from
//! YAML format, which is used for storing individual credential files within
//! the encrypted 7z archive. It handles proper serialization of all field types
//! and maintains data integrity.

use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{SharedError, SharedResult};
use crate::models::{CredentialField, CredentialRecord, FieldType};

/// YAML serialization utilities
pub struct YamlUtils;

impl YamlUtils {
    /// Serialize a credential record to YAML string
    pub fn serialize_credential(credential: &CredentialRecord) -> SharedResult<String> {
        // Create a serializable version with custom formatting
        let yaml_cred = YamlCredential::from_credential(credential);

        serde_yaml::to_string(&yaml_cred).map_err(|e| SharedError::Serialization {
            message: format!("Failed to serialize credential to YAML: {e}"),
        })
    }

    /// Deserialize a credential record from YAML string
    pub fn deserialize_credential(yaml_content: &str) -> SharedResult<CredentialRecord> {
        let yaml_cred: YamlCredential =
            serde_yaml::from_str(yaml_content).map_err(|e| SharedError::Serialization {
                message: format!("Failed to deserialize credential from YAML: {e}"),
            })?;

        yaml_cred.into_credential()
    }

    /// Write a credential to a YAML file
    pub fn write_credential_file<P: AsRef<Path>>(
        path: P,
        credential: &CredentialRecord,
    ) -> SharedResult<()> {
        let yaml_content = Self::serialize_credential(credential)?;

        fs::write(&path, yaml_content).map_err(|e| SharedError::Internal {
            message: format!("Failed to write credential file {:?}: {}", path.as_ref(), e),
        })?;

        Ok(())
    }

    /// Read a credential from a YAML file
    pub fn read_credential_file<P: AsRef<Path>>(path: P) -> SharedResult<CredentialRecord> {
        let yaml_content = fs::read_to_string(&path).map_err(|e| SharedError::Internal {
            message: format!("Failed to read credential file {:?}: {}", path.as_ref(), e),
        })?;

        Self::deserialize_credential(&yaml_content)
    }

    /// Serialize multiple credentials to a single YAML document
    pub fn serialize_credentials(credentials: &[CredentialRecord]) -> SharedResult<String> {
        let yaml_creds: Vec<YamlCredential> = credentials
            .iter()
            .map(YamlCredential::from_credential)
            .collect();

        serde_yaml::to_string(&yaml_creds).map_err(|e| SharedError::Serialization {
            message: format!("Failed to serialize credentials to YAML: {e}"),
        })
    }

    /// Deserialize multiple credentials from a single YAML document
    pub fn deserialize_credentials(yaml_content: &str) -> SharedResult<Vec<CredentialRecord>> {
        let yaml_creds: Vec<YamlCredential> =
            serde_yaml::from_str(yaml_content).map_err(|e| SharedError::Serialization {
                message: format!("Failed to deserialize credentials from YAML: {e}"),
            })?;

        yaml_creds
            .into_iter()
            .map(|yaml_cred| yaml_cred.into_credential())
            .collect()
    }

    /// Validate YAML syntax without deserializing
    pub fn validate_yaml_syntax(yaml_content: &str) -> SharedResult<()> {
        serde_yaml::from_str::<serde_yaml::Value>(yaml_content).map_err(|e| {
            SharedError::InvalidFormat {
                message: format!("Invalid YAML syntax: {e}"),
            }
        })?;

        Ok(())
    }

    /// Pretty-format YAML content
    pub fn format_yaml(yaml_content: &str) -> SharedResult<String> {
        let value: serde_yaml::Value =
            serde_yaml::from_str(yaml_content).map_err(|e| SharedError::InvalidFormat {
                message: format!("Invalid YAML for formatting: {e}"),
            })?;

        serde_yaml::to_string(&value).map_err(|e| SharedError::Serialization {
            message: format!("Failed to format YAML: {e}"),
        })
    }
}

/// YAML-friendly representation of a credential record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct YamlCredential {
    /// Credential metadata
    id: String,
    title: String,
    #[serde(rename = "type")]
    credential_type: String,

    /// Timestamps (as Unix timestamps for consistency)
    created_at: u64,
    updated_at: u64,

    /// Credential data
    fields: HashMap<String, YamlField>,
    tags: Vec<String>,
    notes: Option<String>,
}

/// YAML-friendly representation of a credential field
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct YamlField {
    #[serde(rename = "type")]
    field_type: String,
    value: String,
    #[serde(default)]
    sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    metadata: HashMap<String, String>,
}

impl YamlCredential {
    /// Convert from CredentialRecord to YamlCredential
    fn from_credential(credential: &CredentialRecord) -> Self {
        use crate::utils::TimeUtils;

        let fields = credential
            .fields
            .iter()
            .map(|(name, field)| (name.clone(), YamlField::from_field(field)))
            .collect();

        Self {
            id: credential.id.clone(),
            title: credential.title.clone(),
            credential_type: credential.credential_type.clone(),
            created_at: TimeUtils::system_time_to_timestamp(credential.created_at),
            updated_at: TimeUtils::system_time_to_timestamp(credential.updated_at),
            fields,
            tags: credential.tags.clone(),
            notes: credential.notes.clone(),
        }
    }

    /// Convert from YamlCredential to CredentialRecord
    fn into_credential(self) -> SharedResult<CredentialRecord> {
        use crate::utils::TimeUtils;

        let fields = self
            .fields
            .into_iter()
            .map(|(name, yaml_field)| {
                let name_clone = name.clone();
                yaml_field
                    .into_field()
                    .map(|field| (name, field))
                    .map_err(|e| SharedError::Field {
                        field: name_clone,
                        message: e.to_string(),
                    })
            })
            .collect::<SharedResult<HashMap<String, CredentialField>>>()?;

        let credential = CredentialRecord {
            id: self.id,
            title: self.title,
            credential_type: self.credential_type,
            created_at: TimeUtils::timestamp_to_system_time(self.created_at),
            updated_at: TimeUtils::timestamp_to_system_time(self.updated_at),
            fields,
            tags: self.tags,
            notes: self.notes,
        };

        Ok(credential)
    }
}

impl YamlField {
    /// Convert from CredentialField to YamlField
    fn from_field(field: &CredentialField) -> Self {
        Self {
            field_type: Self::field_type_to_string(&field.field_type),
            value: field.value.clone(),
            sensitive: field.sensitive,
            label: field.label.clone(),
            metadata: field.metadata.clone(),
        }
    }

    /// Convert from YamlField to CredentialField
    fn into_field(self) -> SharedResult<CredentialField> {
        let field_type = Self::string_to_field_type(&self.field_type)?;

        Ok(CredentialField {
            field_type,
            value: self.value,
            sensitive: self.sensitive,
            label: self.label,
            metadata: self.metadata,
        })
    }

    /// Convert FieldType to string representation
    fn field_type_to_string(field_type: &FieldType) -> String {
        match field_type {
            FieldType::Text => "text".to_string(),
            FieldType::Password => "password".to_string(),
            FieldType::Email => "email".to_string(),
            FieldType::Url => "url".to_string(),
            FieldType::Username => "username".to_string(),
            FieldType::Phone => "phone".to_string(),
            FieldType::CreditCardNumber => "credit_card_number".to_string(),
            FieldType::ExpiryDate => "expiry_date".to_string(),
            FieldType::Cvv => "cvv".to_string(),
            FieldType::TotpSecret => "totp_secret".to_string(),
            FieldType::TextArea => "text_area".to_string(),
            FieldType::Number => "number".to_string(),
            FieldType::Date => "date".to_string(),
            FieldType::Custom(name) => format!("custom:{name}"),
        }
    }

    /// Convert string representation to FieldType
    fn string_to_field_type(type_str: &str) -> SharedResult<FieldType> {
        match type_str {
            "text" => Ok(FieldType::Text),
            "password" => Ok(FieldType::Password),
            "email" => Ok(FieldType::Email),
            "url" => Ok(FieldType::Url),
            "username" => Ok(FieldType::Username),
            "phone" => Ok(FieldType::Phone),
            "credit_card_number" => Ok(FieldType::CreditCardNumber),
            "expiry_date" => Ok(FieldType::ExpiryDate),
            "cvv" => Ok(FieldType::Cvv),
            "totp_secret" => Ok(FieldType::TotpSecret),
            "text_area" => Ok(FieldType::TextArea),
            "number" => Ok(FieldType::Number),
            "date" => Ok(FieldType::Date),
            custom if custom.starts_with("custom:") => {
                let name = custom.strip_prefix("custom:").unwrap();
                Ok(FieldType::Custom(name.to_string()))
            }
            _ => Err(SharedError::InvalidFormat {
                message: format!("Unknown field type: {type_str}"),
            }),
        }
    }
}

/// Archive metadata for YAML serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct YamlArchiveMetadata {
    /// Archive format version
    pub version: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modification timestamp
    pub last_modified: u64,
    /// Number of credentials in archive
    pub credential_count: usize,
    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl YamlArchiveMetadata {
    /// Create new archive metadata
    pub fn new(credential_count: usize) -> Self {
        use crate::utils::TimeUtils;
        let now = TimeUtils::current_timestamp();

        Self {
            version: crate::ARCHIVE_FORMAT_VERSION.to_string(),
            created_at: now,
            last_modified: now,
            credential_count,
            metadata: HashMap::new(),
        }
    }

    /// Update the last modified timestamp
    pub fn touch(&mut self) {
        use crate::utils::TimeUtils;
        self.last_modified = TimeUtils::current_timestamp();
    }

    /// Serialize to YAML string
    pub fn to_yaml(&self) -> SharedResult<String> {
        serde_yaml::to_string(self).map_err(|e| SharedError::Serialization {
            message: format!("Failed to serialize archive metadata: {e}"),
        })
    }

    /// Deserialize from YAML string
    pub fn from_yaml(yaml_content: &str) -> SharedResult<Self> {
        serde_yaml::from_str(yaml_content).map_err(|e| SharedError::Serialization {
            message: format!("Failed to deserialize archive metadata: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, CredentialRecord};
    use std::collections::HashMap;

    #[test]
    fn test_credential_yaml_roundtrip() {
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));
        credential.set_field("website", CredentialField::url("https://example.com"));
        credential.add_tag("test");
        credential.notes = Some("Test notes".to_string());

        // Serialize to YAML
        let yaml = YamlUtils::serialize_credential(&credential).unwrap();
        assert!(yaml.contains("title: Test Login"));
        assert!(yaml.contains("type: login"));
        assert!(yaml.contains("username:"));
        assert!(yaml.contains("password:"));

        // Deserialize back
        let deserialized = YamlUtils::deserialize_credential(&yaml).unwrap();
        assert_eq!(deserialized.title, credential.title);
        assert_eq!(deserialized.credential_type, credential.credential_type);
        assert_eq!(deserialized.fields.len(), credential.fields.len());
        assert_eq!(deserialized.tags, credential.tags);
        assert_eq!(deserialized.notes, credential.notes);

        // Check specific fields
        let username_field = deserialized.get_field("username").unwrap();
        assert_eq!(username_field.value, "testuser");
        assert_eq!(username_field.field_type, FieldType::Username);

        let password_field = deserialized.get_field("password").unwrap();
        assert_eq!(password_field.value, "testpass");
        assert!(password_field.sensitive);
    }

    #[test]
    fn test_multiple_credentials_serialization() {
        let cred1 = CredentialRecord::new("Cred 1".to_string(), "login".to_string());
        let cred2 = CredentialRecord::new("Cred 2".to_string(), "note".to_string());
        let credentials = vec![cred1, cred2];

        let yaml = YamlUtils::serialize_credentials(&credentials).unwrap();
        let deserialized = YamlUtils::deserialize_credentials(&yaml).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].title, "Cred 1");
        assert_eq!(deserialized[1].title, "Cred 2");
    }

    #[test]
    fn test_field_type_conversion() {
        let test_cases = vec![
            (FieldType::Text, "text"),
            (FieldType::Password, "password"),
            (FieldType::Email, "email"),
            (FieldType::TotpSecret, "totp_secret"),
            (FieldType::Custom("special".to_string()), "custom:special"),
        ];

        for (field_type, expected_string) in test_cases {
            let string_repr = YamlField::field_type_to_string(&field_type);
            assert_eq!(string_repr, expected_string);

            let converted_back = YamlField::string_to_field_type(&string_repr).unwrap();
            assert_eq!(converted_back, field_type);
        }
    }

    #[test]
    fn test_archive_metadata() {
        let mut metadata = YamlArchiveMetadata::new(5);
        metadata
            .metadata
            .insert("source".to_string(), "test".to_string());

        let yaml = metadata.to_yaml().unwrap();
        assert!(yaml.contains("credential_count: 5"));
        assert!(yaml.contains("version:"));

        let deserialized = YamlArchiveMetadata::from_yaml(&yaml).unwrap();
        assert_eq!(deserialized.credential_count, 5);
        assert_eq!(
            deserialized.metadata.get("source"),
            Some(&"test".to_string())
        );
    }

    #[test]
    fn test_yaml_validation() {
        let valid_yaml = "title: Test\ntype: login\nfields: {}";
        assert!(YamlUtils::validate_yaml_syntax(valid_yaml).is_ok());

        let invalid_yaml = "title: Test\n  invalid: [unclosed";
        assert!(YamlUtils::validate_yaml_syntax(invalid_yaml).is_err());
    }

    #[test]
    fn test_sensitive_field_handling() {
        let mut credential = CredentialRecord::new("Test".to_string(), "login".to_string());
        credential.set_field("password", CredentialField::password("secret123"));
        credential.set_field("username", CredentialField::username("user"));

        let yaml = YamlUtils::serialize_credential(&credential).unwrap();
        let deserialized = YamlUtils::deserialize_credential(&yaml).unwrap();

        let password_field = deserialized.get_field("password").unwrap();
        assert!(password_field.sensitive);

        let username_field = deserialized.get_field("username").unwrap();
        assert!(!username_field.sensitive);
    }

    #[test]
    fn test_custom_field_type() {
        let mut credential = CredentialRecord::new("Test".to_string(), "custom".to_string());
        let custom_field = CredentialField {
            field_type: FieldType::Custom("api_key".to_string()),
            value: "abc123".to_string(),
            sensitive: true,
            label: Some("API Key".to_string()),
            metadata: HashMap::new(),
        };
        credential.set_field("api_key", custom_field);

        let yaml = YamlUtils::serialize_credential(&credential).unwrap();
        let deserialized = YamlUtils::deserialize_credential(&yaml).unwrap();

        let api_key_field = deserialized.get_field("api_key").unwrap();
        assert_eq!(
            api_key_field.field_type,
            FieldType::Custom("api_key".to_string())
        );
        assert_eq!(api_key_field.value, "abc123");
        assert!(api_key_field.sensitive);
        assert_eq!(api_key_field.label, Some("API Key".to_string()));
    }
}
