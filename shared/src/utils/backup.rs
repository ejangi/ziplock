//! Backup and export utilities for ZipLock
//!
//! This module provides utilities for creating backups, exporting credential
//! data in various formats, and managing repository snapshots for disaster
//! recovery and data portability.

use crate::core::{CoreError, CoreResult, UnifiedMemoryRepository};
use crate::models::CredentialRecord;
use crate::utils::time_utils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format with full credential data
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// YAML format (ZipLock native)
    Yaml,
    /// Encrypted ZipLock backup format
    ZipLockBackup,
}

impl ExportFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Yaml => "yaml",
            ExportFormat::ZipLockBackup => "zlb",
        }
    }

    /// Get MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Json => "application/json",
            ExportFormat::Csv => "text/csv",
            ExportFormat::Yaml => "text/yaml",
            ExportFormat::ZipLockBackup => "application/octet-stream",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON Format",
            ExportFormat::Csv => "CSV (Comma-Separated Values)",
            ExportFormat::Yaml => "YAML Format",
            ExportFormat::ZipLockBackup => "ZipLock Backup",
        }
    }
}

/// Export options and settings
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export format
    pub format: ExportFormat,
    /// Include sensitive data (passwords, etc.)
    pub include_sensitive: bool,
    /// Include metadata fields
    pub include_metadata: bool,
    /// Include tags
    pub include_tags: bool,
    /// Include notes
    pub include_notes: bool,
    /// Filter by credential type
    pub credential_types: Option<Vec<String>>,
    /// Filter by tags
    pub required_tags: Option<Vec<String>>,
    /// Encryption password for backup format
    pub encryption_password: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_sensitive: true,
            include_metadata: true,
            include_tags: true,
            include_notes: true,
            credential_types: None,
            required_tags: None,
            encryption_password: None,
        }
    }
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup creation timestamp
    pub created_at: i64,
    /// ZipLock version that created the backup
    pub ziplock_version: String,
    /// Backup format version
    pub format_version: String,
    /// Number of credentials in backup
    pub credential_count: usize,
    /// Original repository path (if available)
    pub source_path: Option<String>,
    /// Backup description
    pub description: Option<String>,
    /// Checksum for integrity verification
    pub checksum: String,
}

/// Backup container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    /// Backup metadata
    pub metadata: BackupMetadata,
    /// Credential data
    pub credentials: Vec<CredentialRecord>,
    /// Additional repository settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// CSV export record for spreadsheet compatibility
#[derive(Debug, Clone, Serialize)]
struct CsvRecord {
    title: String,
    #[serde(rename = "type")]
    credential_type: String,
    username: String,
    password: String,
    email: String,
    url: String,
    notes: String,
    tags: String,
    created_at: String,
    updated_at: String,
}

/// Backup and export utilities
pub struct BackupManager;

impl BackupManager {
    /// Create a backup of the repository
    pub fn create_backup(
        repository: &UnifiedMemoryRepository,
        options: &ExportOptions,
        description: Option<String>,
    ) -> CoreResult<BackupData> {
        if !repository.is_initialized() {
            return Err(CoreError::NotInitialized);
        }

        let credentials = repository.list_credentials()?;
        let filtered_credentials = Self::filter_credentials(&credentials, options);

        let backup_data = BackupData {
            metadata: BackupMetadata {
                created_at: time_utils::current_timestamp(),
                ziplock_version: env!("CARGO_PKG_VERSION").to_string(),
                format_version: "1.0".to_string(),
                credential_count: filtered_credentials.len(),
                source_path: None,
                description,
                checksum: Self::calculate_checksum(&filtered_credentials),
            },
            credentials: filtered_credentials,
            settings: HashMap::new(),
        };

        Ok(backup_data)
    }

    /// Export repository to specified format
    pub fn export_repository(
        repository: &UnifiedMemoryRepository,
        options: &ExportOptions,
    ) -> CoreResult<Vec<u8>> {
        let backup = Self::create_backup(repository, options, None)?;

        match options.format {
            ExportFormat::Json => Self::export_json(&backup, options),
            ExportFormat::Csv => Self::export_csv(&backup, options),
            ExportFormat::Yaml => Self::export_yaml(&backup, options),
            ExportFormat::ZipLockBackup => Self::export_backup(&backup, options),
        }
    }

    /// Export to JSON format
    fn export_json(backup: &BackupData, _options: &ExportOptions) -> CoreResult<Vec<u8>> {
        serde_json::to_vec_pretty(backup).map_err(|e| CoreError::SerializationError {
            message: format!("JSON export failed: {}", e),
        })
    }

    /// Export to CSV format
    fn export_csv(backup: &BackupData, options: &ExportOptions) -> CoreResult<Vec<u8>> {
        let mut writer = csv::Writer::from_writer(Vec::new());

        for credential in &backup.credentials {
            let record = CsvRecord {
                title: credential.title.clone(),
                credential_type: credential.credential_type.clone(),
                username: Self::get_field_value(credential, "username", options),
                password: Self::get_field_value(credential, "password", options),
                email: Self::get_field_value(credential, "email", options),
                url: Self::get_field_value(credential, "url", options),
                notes: if options.include_notes {
                    credential.notes.clone().unwrap_or_default()
                } else {
                    String::new()
                },
                tags: if options.include_tags {
                    credential.tags.join(", ")
                } else {
                    String::new()
                },
                created_at: time_utils::format_timestamp(credential.created_at),
                updated_at: time_utils::format_timestamp(credential.updated_at),
            };

            writer
                .serialize(record)
                .map_err(|e| CoreError::SerializationError {
                    message: format!("CSV serialization failed: {}", e),
                })?;
        }

        writer
            .into_inner()
            .map_err(|e| CoreError::SerializationError {
                message: format!("CSV export failed: {}", e),
            })
    }

    /// Export to YAML format
    fn export_yaml(backup: &BackupData, _options: &ExportOptions) -> CoreResult<Vec<u8>> {
        serde_yaml::to_string(backup)
            .map(|s| s.into_bytes())
            .map_err(|e| CoreError::SerializationError {
                message: format!("YAML export failed: {}", e),
            })
    }

    /// Export to encrypted ZipLock backup format
    fn export_backup(backup: &BackupData, options: &ExportOptions) -> CoreResult<Vec<u8>> {
        let json_data = serde_json::to_vec(backup).map_err(|e| CoreError::SerializationError {
            message: format!("Backup serialization failed: {}", e),
        })?;

        if let Some(_password) = &options.encryption_password {
            // In a real implementation, this would use proper encryption
            // For now, just return the JSON data with a header
            let mut encrypted_data = b"ZLBV1.0\n".to_vec();
            encrypted_data.extend_from_slice(&json_data);
            Ok(encrypted_data)
        } else {
            Ok(json_data)
        }
    }

    /// Import backup from data
    pub fn import_backup(data: &[u8], _password: Option<&str>) -> CoreResult<BackupData> {
        // Check for ZipLock backup format
        if data.starts_with(b"ZLBV1.0\n") {
            let json_data = &data[8..]; // Skip header
            serde_json::from_slice(json_data).map_err(|e| CoreError::SerializationError {
                message: format!("Backup import failed: {}", e),
            })
        } else {
            // Try as plain JSON
            serde_json::from_slice(data).map_err(|e| CoreError::SerializationError {
                message: format!("Backup import failed: {}", e),
            })
        }
    }

    /// Verify backup integrity
    pub fn verify_backup(backup: &BackupData) -> bool {
        let calculated_checksum = Self::calculate_checksum(&backup.credentials);
        calculated_checksum == backup.metadata.checksum
    }

    /// Filter credentials based on export options
    fn filter_credentials(
        credentials: &[CredentialRecord],
        options: &ExportOptions,
    ) -> Vec<CredentialRecord> {
        credentials
            .iter()
            .filter(|cred| {
                // Filter by credential type
                if let Some(ref types) = options.credential_types {
                    if !types.contains(&cred.credential_type) {
                        return false;
                    }
                }

                // Filter by required tags
                if let Some(ref required_tags) = options.required_tags {
                    if !required_tags.iter().all(|tag| cred.tags.contains(tag)) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .map(|mut cred| {
                // Remove sensitive data if not included
                if !options.include_sensitive {
                    cred.fields.retain(|_, field| !field.sensitive);
                }

                // Remove metadata if not included
                if !options.include_metadata {
                    cred.created_at = 0;
                    cred.updated_at = 0;
                    cred.accessed_at = 0;
                }

                // Remove tags if not included
                if !options.include_tags {
                    cred.tags.clear();
                }

                // Remove notes if not included
                if !options.include_notes {
                    cred.notes = None;
                }

                cred
            })
            .collect()
    }

    /// Get field value for CSV export
    fn get_field_value(
        credential: &CredentialRecord,
        field_name: &str,
        options: &ExportOptions,
    ) -> String {
        if let Some(field) = credential.fields.get(field_name) {
            if field.sensitive && !options.include_sensitive {
                "[HIDDEN]".to_string()
            } else {
                field.value.clone()
            }
        } else {
            String::new()
        }
    }

    /// Calculate checksum for credentials
    fn calculate_checksum(credentials: &[CredentialRecord]) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        for credential in credentials {
            hasher.update(credential.id.as_bytes());
            hasher.update(credential.title.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Save backup to file
    pub fn save_backup_to_file<P: AsRef<Path>>(
        backup: &BackupData,
        path: P,
        options: &ExportOptions,
    ) -> CoreResult<()> {
        let data = match options.format {
            ExportFormat::Json => Self::export_json(backup, options)?,
            ExportFormat::Csv => Self::export_csv(backup, options)?,
            ExportFormat::Yaml => Self::export_yaml(backup, options)?,
            ExportFormat::ZipLockBackup => Self::export_backup(backup, options)?,
        };

        fs::write(path, data).map_err(|e| CoreError::SerializationError {
            message: format!("Failed to save backup: {}", e),
        })
    }

    /// Load backup from file
    pub fn load_backup_from_file<P: AsRef<Path>>(
        path: P,
        password: Option<&str>,
    ) -> CoreResult<BackupData> {
        let data = fs::read(path).map_err(|e| CoreError::SerializationError {
            message: format!("Failed to read backup file: {}", e),
        })?;

        Self::import_backup(&data, password)
    }

    /// Get backup statistics
    pub fn get_backup_stats(backup: &BackupData) -> BackupStats {
        let mut type_counts = HashMap::new();
        let mut tag_counts = HashMap::new();
        let mut total_fields = 0;
        let mut sensitive_fields = 0;

        for credential in &backup.credentials {
            // Count credential types
            *type_counts
                .entry(credential.credential_type.clone())
                .or_insert(0) += 1;

            // Count tags
            for tag in &credential.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }

            // Count fields
            total_fields += credential.fields.len();
            sensitive_fields += credential.fields.values().filter(|f| f.sensitive).count();
        }

        BackupStats {
            credential_count: backup.credentials.len(),
            type_counts,
            tag_counts,
            total_fields,
            sensitive_fields,
            backup_size: 0, // Would need to calculate based on serialized size
            created_at: backup.metadata.created_at,
        }
    }
}

/// Backup statistics
#[derive(Debug, Clone)]
pub struct BackupStats {
    pub credential_count: usize,
    pub type_counts: HashMap<String, usize>,
    pub tag_counts: HashMap<String, usize>,
    pub total_fields: usize,
    pub sensitive_fields: usize,
    pub backup_size: usize,
    pub created_at: i64,
}

/// Migration utilities for upgrading backup formats
pub struct MigrationManager;

impl MigrationManager {
    /// Migrate backup from older format versions
    pub fn migrate_backup(backup: &mut BackupData) -> CoreResult<()> {
        match backup.metadata.format_version.as_str() {
            "1.0" => Ok(()), // Current version, no migration needed
            version => Err(CoreError::SerializationError {
                message: format!("Unsupported backup format version: {}", version),
            }),
        }
    }

    /// Check if backup needs migration
    pub fn needs_migration(backup: &BackupData) -> bool {
        backup.metadata.format_version != "1.0"
    }

    /// Get supported format versions
    pub fn supported_versions() -> &'static [&'static str] {
        &["1.0"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, FieldType};
    use std::path::PathBuf;

    fn get_test_results_dir() -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        // Go up one level from shared/ to project root
        path.pop();
        path.push("tests");
        path.push("results");
        std::fs::create_dir_all(&path).ok();
        path
    }

    fn create_test_repository() -> UnifiedMemoryRepository {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        let mut cred1 = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        cred1.set_field(
            "username",
            CredentialField::new(FieldType::Username, "user1".to_string(), false),
        );
        cred1.set_field("password", CredentialField::password("pass1".to_string()));
        cred1.tags = vec!["work".to_string(), "important".to_string()];

        let mut cred2 = CredentialRecord::new("Test Note".to_string(), "note".to_string());
        cred2.notes = Some("This is a test note".to_string());
        cred2.tags = vec!["personal".to_string()];

        repo.add_credential(cred1).unwrap();
        repo.add_credential(cred2).unwrap();

        repo
    }

    #[test]
    fn test_create_backup() {
        let repo = create_test_repository();
        let options = ExportOptions::default();

        let backup = BackupManager::create_backup(&repo, &options, Some("Test backup".to_string()));
        assert!(backup.is_ok());

        let backup = backup.unwrap();
        assert_eq!(backup.credentials.len(), 2);
        assert_eq!(backup.metadata.credential_count, 2);
        assert_eq!(backup.metadata.description, Some("Test backup".to_string()));
    }

    #[test]
    fn test_export_json() {
        let repo = create_test_repository();
        let options = ExportOptions {
            format: ExportFormat::Json,
            ..Default::default()
        };

        let data = BackupManager::export_repository(&repo, &options).unwrap();
        let json_str = String::from_utf8(data).unwrap();

        assert!(json_str.contains("Test Login"));
        assert!(json_str.contains("metadata"));
    }

    #[test]
    fn test_export_csv() {
        let repo = create_test_repository();
        let options = ExportOptions {
            format: ExportFormat::Csv,
            ..Default::default()
        };

        let data = BackupManager::export_repository(&repo, &options).unwrap();
        let csv_str = String::from_utf8(data).unwrap();

        assert!(csv_str.contains("title,type,username"));
        assert!(csv_str.contains("Test Login"));
    }

    #[test]
    fn test_filtering() {
        let repo = create_test_repository();
        let options = ExportOptions {
            credential_types: Some(vec!["login".to_string()]),
            ..Default::default()
        };

        let backup = BackupManager::create_backup(&repo, &options, None).unwrap();
        assert_eq!(backup.credentials.len(), 1);
        assert_eq!(backup.credentials[0].credential_type, "login");
    }

    #[test]
    fn test_sensitive_data_filtering() {
        let repo = create_test_repository();
        let options = ExportOptions {
            include_sensitive: false,
            ..Default::default()
        };

        let backup = BackupManager::create_backup(&repo, &options, None).unwrap();

        // Check that sensitive fields are removed
        for credential in &backup.credentials {
            for (_, field) in &credential.fields {
                assert!(!field.sensitive);
            }
        }
    }

    #[test]
    fn test_backup_verification() {
        let repo = create_test_repository();
        let options = ExportOptions::default();
        let backup = BackupManager::create_backup(&repo, &options, None).unwrap();

        assert!(BackupManager::verify_backup(&backup));

        // Tamper with the backup
        let mut tampered_backup = backup.clone();
        tampered_backup.credentials.clear();
        assert!(!BackupManager::verify_backup(&tampered_backup));
    }

    #[test]
    fn test_backup_import_export() {
        let repo = create_test_repository();
        let options = ExportOptions {
            format: ExportFormat::ZipLockBackup,
            encryption_password: Some("test_password".to_string()),
            ..Default::default()
        };

        let exported_data = BackupManager::export_repository(&repo, &options).unwrap();
        let imported_backup =
            BackupManager::import_backup(&exported_data, Some("test_password")).unwrap();

        assert_eq!(imported_backup.credentials.len(), 2);
        assert!(BackupManager::verify_backup(&imported_backup));
    }

    #[test]
    fn test_file_operations() {
        let test_dir = get_test_results_dir();
        let backup_path = test_dir.join("test_backup.json");

        let repo = create_test_repository();
        let options = ExportOptions {
            format: ExportFormat::Json,
            ..Default::default()
        };

        let backup = BackupManager::create_backup(&repo, &options, None).unwrap();

        // Save to file
        BackupManager::save_backup_to_file(&backup, &backup_path, &options).unwrap();
        assert!(backup_path.exists());

        // Load from file
        let loaded_backup = BackupManager::load_backup_from_file(&backup_path, None).unwrap();
        assert_eq!(loaded_backup.credentials.len(), backup.credentials.len());
    }

    #[test]
    fn test_backup_stats() {
        let repo = create_test_repository();
        let options = ExportOptions::default();
        let backup = BackupManager::create_backup(&repo, &options, None).unwrap();

        let stats = BackupManager::get_backup_stats(&backup);
        assert_eq!(stats.credential_count, 2);
        assert!(stats.type_counts.contains_key("login"));
        assert!(stats.type_counts.contains_key("note"));
        assert!(stats.tag_counts.contains_key("work"));
    }

    #[test]
    fn test_export_formats() {
        for format in &[
            ExportFormat::Json,
            ExportFormat::Csv,
            ExportFormat::Yaml,
            ExportFormat::ZipLockBackup,
        ] {
            assert!(!format.extension().is_empty());
            assert!(!format.mime_type().is_empty());
            assert!(!format.description().is_empty());
        }
    }

    #[test]
    fn test_migration_manager() {
        let repo = create_test_repository();
        let options = ExportOptions::default();
        let mut backup = BackupManager::create_backup(&repo, &options, None).unwrap();

        assert!(!MigrationManager::needs_migration(&backup));
        assert!(MigrationManager::migrate_backup(&mut backup).is_ok());

        // Test unsupported version
        backup.metadata.format_version = "0.9".to_string();
        assert!(MigrationManager::needs_migration(&backup));
        assert!(MigrationManager::migrate_backup(&mut backup).is_err());
    }
}
