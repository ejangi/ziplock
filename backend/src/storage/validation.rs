//! Repository validation and repair utilities for ZipLock
//!
//! This module provides comprehensive validation and repair functionality for
//! ZipLock repository archives, ensuring they conform to the repository format
//! specification version 1.0.

use crate::error::{BackendResult, StorageError};
use anyhow::Context;
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::Path;
use std::time::SystemTime;
use tracing::{debug, info, warn};
use ziplock_shared::models::CredentialRecord;

/// Repository format version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
    /// Optional pre-release identifier
    pub pre_release: Option<String>,
}

impl RepositoryVersion {
    /// Current repository format version (1.0.0)
    pub const CURRENT: RepositoryVersion = RepositoryVersion {
        major: 1,
        minor: 0,
        patch: 0,
        pre_release: None,
    };

    /// Parse version from string (e.g., "1.0.0" or "1.0.0-beta")
    pub fn parse(version_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version_str.split('-').collect();
        let version_part = parts[0];
        let pre_release = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };

        let version_numbers: Vec<&str> = version_part.split('.').collect();
        if version_numbers.len() != 3 {
            return Err(format!("Invalid version format: {}", version_str));
        }

        let major = version_numbers[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", version_numbers[0]))?;
        let minor = version_numbers[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", version_numbers[1]))?;
        let patch = version_numbers[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid patch version: {}", version_numbers[2]))?;

        Ok(RepositoryVersion {
            major,
            minor,
            patch,
            pre_release,
        })
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &RepositoryVersion) -> bool {
        // Major version must match for compatibility
        self.major == other.major
    }

    /// Check if this version is newer than another version
    pub fn is_newer_than(&self, other: &RepositoryVersion) -> bool {
        if self.major != other.major {
            return self.major > other.major;
        }
        if self.minor != other.minor {
            return self.minor > other.minor;
        }
        self.patch > other.patch
    }
}

impl std::fmt::Display for RepositoryVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref pre) = self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Repository validation issues that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssue {
    /// Missing required file or directory
    MissingRequired { path: String, description: String },
    /// Invalid file format or content
    InvalidFormat { path: String, reason: String },
    /// Version mismatch or incompatibility
    VersionIssue {
        found: String,
        expected: String,
        severity: ValidationSeverity,
    },
    /// Corrupted credential data
    CorruptedCredential {
        credential_id: String,
        reason: String,
    },
    /// Legacy format detected
    LegacyFormat {
        description: String,
        migration_needed: bool,
    },
    /// Structural inconsistency
    StructuralIssue {
        description: String,
        auto_fixable: bool,
    },
}

/// Severity level of validation issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical issues that prevent repository usage
    Critical,
    /// Warning issues that should be addressed
    Warning,
    /// Informational issues that can be ignored
    Info,
}

/// Result of repository validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Repository format version found
    pub version: Option<RepositoryVersion>,
    /// List of validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Whether the repository is valid for use
    pub is_valid: bool,
    /// Whether auto-repair is possible
    pub can_auto_repair: bool,
    /// Statistics about the repository
    pub stats: RepositoryStats,
}

/// Statistics about repository contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    /// Total number of credentials
    pub credential_count: usize,
    /// Number of custom types defined
    pub custom_type_count: usize,
    /// Total size of repository in bytes
    pub total_size_bytes: u64,
    /// Last modification time
    pub last_modified: Option<SystemTime>,
    /// Repository creation time
    pub created_at: Option<SystemTime>,
}

/// Repository validator
pub struct RepositoryValidator {
    /// Whether to perform deep validation of credential data
    pub deep_validation: bool,
    /// Whether to check for legacy formats
    pub check_legacy_formats: bool,
    /// Whether to validate credential schemas
    pub validate_schemas: bool,
}

impl Default for RepositoryValidator {
    fn default() -> Self {
        Self {
            deep_validation: true,
            check_legacy_formats: true,
            validate_schemas: true,
        }
    }
}

impl RepositoryValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a validator with custom settings
    pub fn with_options(
        deep_validation: bool,
        check_legacy_formats: bool,
        validate_schemas: bool,
    ) -> Self {
        Self {
            deep_validation,
            check_legacy_formats,
            validate_schemas,
        }
    }

    /// Validate a repository directory
    pub fn validate(&self, repository_path: &Path) -> BackendResult<ValidationReport> {
        info!("Starting repository validation: {:?}", repository_path);

        let mut issues = Vec::new();
        let mut stats = RepositoryStats {
            credential_count: 0,
            custom_type_count: 0,
            total_size_bytes: 0,
            last_modified: None,
            created_at: None,
        };

        // Check basic structure
        self.validate_basic_structure(repository_path, &mut issues)?;

        // Validate metadata
        let version = self.validate_metadata(repository_path, &mut issues, &mut stats)?;

        // Validate credentials directory
        if repository_path.join("credentials").exists() {
            self.validate_credentials_directory(repository_path, &mut issues, &mut stats)?;
        }

        // Validate types directory
        if repository_path.join("types").exists() {
            self.validate_types_directory(repository_path, &mut issues, &mut stats)?;
        }

        // Calculate overall repository size
        if let Ok(metadata) = fs::metadata(repository_path) {
            stats.total_size_bytes = self.calculate_directory_size(repository_path)?;
            stats.last_modified = metadata.modified().ok();
        }

        // Determine if repository is valid
        let critical_issues = issues.iter().any(|issue| {
            matches!(
                issue,
                ValidationIssue::MissingRequired { .. }
                    | ValidationIssue::InvalidFormat { .. }
                    | ValidationIssue::CorruptedCredential { .. }
                    | ValidationIssue::VersionIssue {
                        severity: ValidationSeverity::Critical,
                        ..
                    }
            )
        });

        let is_valid = !critical_issues;
        let can_auto_repair = issues.iter().any(|issue| {
            matches!(
                issue,
                ValidationIssue::MissingRequired { .. }
                    | ValidationIssue::StructuralIssue {
                        auto_fixable: true,
                        ..
                    }
                    | ValidationIssue::LegacyFormat {
                        migration_needed: true,
                        ..
                    }
            )
        });

        let report = ValidationReport {
            version,
            issues,
            is_valid,
            can_auto_repair,
            stats,
        };

        if !report.is_valid {
            warn!(
                "Repository validation completed with {} issues: valid={}, can_repair={}",
                report.issues.len(),
                report.is_valid,
                report.can_auto_repair
            );
            for (i, issue) in report.issues.iter().enumerate() {
                warn!("Validation issue {}: {:?}", i + 1, issue);
            }
        } else {
            info!(
                "Repository validation completed successfully: valid={}, can_repair={}",
                report.is_valid, report.can_auto_repair
            );
        }
        Ok(report)
    }

    /// Attempt to auto-repair repository issues
    pub fn auto_repair(&self, repository_path: &Path) -> BackendResult<ValidationReport> {
        info!("Starting repository auto-repair: {:?}", repository_path);

        // First validate to identify issues
        let initial_report = self.validate(repository_path)?;

        if !initial_report.can_auto_repair {
            return Ok(initial_report);
        }

        let mut repairs_made = 0;

        for issue in &initial_report.issues {
            match issue {
                ValidationIssue::MissingRequired { path, .. } => {
                    if self.repair_missing_required(repository_path, path)? {
                        repairs_made += 1;
                    }
                }
                ValidationIssue::StructuralIssue {
                    auto_fixable: true,
                    description,
                } => {
                    if self.repair_structural_issue(repository_path, description)? {
                        repairs_made += 1;
                    }
                }
                ValidationIssue::LegacyFormat {
                    migration_needed: true,
                    ..
                } => {
                    if self.migrate_legacy_format(repository_path)? {
                        repairs_made += 1;
                    }
                }
                _ => {
                    // Cannot auto-repair this issue
                }
            }
        }

        info!("Auto-repair completed: {} repairs made", repairs_made);

        // Re-validate after repairs
        let final_report = self.validate(repository_path)?;

        if !final_report.is_valid {
            warn!(
                "Auto-repair completed but {} issues remain",
                final_report.issues.len()
            );
            for (i, issue) in final_report.issues.iter().enumerate() {
                warn!("Remaining issue {}: {:?}", i + 1, issue);
            }
        } else {
            info!("Auto-repair successfully resolved all validation issues");
        }

        Ok(final_report)
    }

    /// Validate basic repository structure
    fn validate_basic_structure(
        &self,
        path: &Path,
        issues: &mut Vec<ValidationIssue>,
    ) -> BackendResult<()> {
        // Check for metadata.yml
        let metadata_path = path.join("metadata.yml");
        if !metadata_path.exists() {
            issues.push(ValidationIssue::MissingRequired {
                path: "metadata.yml".to_string(),
                description: "Repository metadata file is required".to_string(),
            });
        }

        // Check for credentials directory
        let credentials_dir = path.join("credentials");
        if !credentials_dir.exists() {
            issues.push(ValidationIssue::MissingRequired {
                path: "credentials/".to_string(),
                description: "Credentials directory is required per repository format v1.0"
                    .to_string(),
            });
        } else if !credentials_dir.is_dir() {
            issues.push(ValidationIssue::StructuralIssue {
                description: "credentials exists but is not a directory".to_string(),
                auto_fixable: false,
            });
        }

        // Check for types directory
        let types_dir = path.join("types");
        if !types_dir.exists() {
            issues.push(ValidationIssue::MissingRequired {
                path: "types/".to_string(),
                description: "Types directory is required per repository format v1.0".to_string(),
            });
        } else if !types_dir.is_dir() {
            issues.push(ValidationIssue::StructuralIssue {
                description: "types exists but is not a directory".to_string(),
                auto_fixable: false,
            });
        }

        Ok(())
    }

    /// Validate metadata.yml file
    fn validate_metadata(
        &self,
        path: &Path,
        issues: &mut Vec<ValidationIssue>,
        stats: &mut RepositoryStats,
    ) -> BackendResult<Option<RepositoryVersion>> {
        let metadata_path = path.join("metadata.yml");
        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&metadata_path)
            .context("Failed to read metadata.yml")
            .map_err(|e| StorageError::InvalidRecord {
                reason: e.to_string(),
            })?;

        let metadata: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
            issues.push(ValidationIssue::InvalidFormat {
                path: "metadata.yml".to_string(),
                reason: format!("Invalid YAML format: {}", e),
            });
            StorageError::InvalidRecord {
                reason: format!("Invalid metadata format: {}", e),
            }
        })?;

        // Extract version information
        let version = if let Some(version_str) = metadata.get("version").and_then(|v| v.as_str()) {
            match RepositoryVersion::parse(version_str) {
                Ok(version) => {
                    // Check version compatibility
                    if version.is_newer_than(&RepositoryVersion::CURRENT) {
                        // Future versions are critical - we don't know how to handle them
                        warn!(
                            "Repository uses future version {}, current version is {}",
                            version,
                            RepositoryVersion::CURRENT
                        );
                        issues.push(ValidationIssue::VersionIssue {
                            found: version.to_string(),
                            expected: RepositoryVersion::CURRENT.to_string(),
                            severity: ValidationSeverity::Critical,
                        });
                    } else if !version.is_compatible_with(&RepositoryVersion::CURRENT) {
                        // Older versions should be auto-upgraded, not blocked
                        info!(
                            "Repository version {} will be upgraded to {}",
                            version,
                            RepositoryVersion::CURRENT
                        );
                        issues.push(ValidationIssue::LegacyFormat {
                            description: format!(
                                "Repository version {} will be upgraded to {}",
                                version,
                                RepositoryVersion::CURRENT
                            ),
                            migration_needed: true,
                        });
                    } else {
                        info!("Repository version {} is current", version);
                    }
                    Some(version)
                }
                Err(e) => {
                    issues.push(ValidationIssue::InvalidFormat {
                        path: "metadata.yml".to_string(),
                        reason: format!("Invalid version format: {}", e),
                    });
                    None
                }
            }
        } else {
            issues.push(ValidationIssue::InvalidFormat {
                path: "metadata.yml".to_string(),
                reason: "Missing version field".to_string(),
            });
            None
        };

        // Extract creation time
        if let Some(_created_str) = metadata.get("created_at").and_then(|v| v.as_str()) {
            // Parse creation time if available (simplified for now)
            stats.created_at = Some(SystemTime::now());
        }

        // Extract credential count
        if let Some(count) = metadata.get("credential_count").and_then(|v| v.as_u64()) {
            stats.credential_count = count as usize;
        }

        Ok(version)
    }

    /// Validate credentials directory structure
    fn validate_credentials_directory(
        &self,
        path: &Path,
        issues: &mut Vec<ValidationIssue>,
        stats: &mut RepositoryStats,
    ) -> BackendResult<()> {
        let credentials_dir = path.join("credentials");
        let entries = fs::read_dir(&credentials_dir)
            .context("Failed to read credentials directory")
            .map_err(|e| StorageError::ArchiveExtract {
                reason: e.to_string(),
            })?;

        let mut actual_credential_count = 0;

        for entry in entries {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(|e| StorageError::ArchiveExtract {
                    reason: e.to_string(),
                })?;

            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            // Skip placeholder files
            if file_name == ".gitkeep" {
                continue;
            }

            if entry_path.is_dir() {
                // Repository format v1.0: check for record.yml
                let record_file = entry_path.join("record.yml");
                if record_file.exists() {
                    if self.deep_validation {
                        match self.validate_credential_record(&record_file) {
                            Ok(_) => actual_credential_count += 1,
                            Err(reason) => {
                                issues.push(ValidationIssue::CorruptedCredential {
                                    credential_id: file_name.to_string(),
                                    reason,
                                });
                            }
                        }
                    } else {
                        actual_credential_count += 1;
                    }
                } else {
                    issues.push(ValidationIssue::StructuralIssue {
                        description: format!(
                            "Credential directory '{}' missing record.yml",
                            file_name
                        ),
                        auto_fixable: false,
                    });
                }
            } else if file_name.ends_with(".yml") {
                // Legacy format detected
                issues.push(ValidationIssue::LegacyFormat {
                    description: format!("Legacy credential file format detected: {}", file_name),
                    migration_needed: true,
                });

                if self.deep_validation {
                    match self.validate_credential_record(&entry_path) {
                        Ok(_) => actual_credential_count += 1,
                        Err(reason) => {
                            issues.push(ValidationIssue::CorruptedCredential {
                                credential_id: file_name.trim_end_matches(".yml").to_string(),
                                reason,
                            });
                        }
                    }
                } else {
                    actual_credential_count += 1;
                }
            }
        }

        // Update stats with actual count
        if stats.credential_count == 0 {
            stats.credential_count = actual_credential_count;
        } else if stats.credential_count != actual_credential_count {
            issues.push(ValidationIssue::StructuralIssue {
                description: format!(
                    "Metadata credential count ({}) doesn't match actual count ({})",
                    stats.credential_count, actual_credential_count
                ),
                auto_fixable: true,
            });
        }

        Ok(())
    }

    /// Validate types directory structure
    fn validate_types_directory(
        &self,
        path: &Path,
        issues: &mut Vec<ValidationIssue>,
        stats: &mut RepositoryStats,
    ) -> BackendResult<()> {
        let types_dir = path.join("types");
        let entries = fs::read_dir(&types_dir)
            .context("Failed to read types directory")
            .map_err(|e| StorageError::ArchiveExtract {
                reason: e.to_string(),
            })?;

        let mut custom_type_count = 0;

        for entry in entries {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(|e| StorageError::ArchiveExtract {
                    reason: e.to_string(),
                })?;

            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            // Skip placeholder files
            if file_name == ".gitkeep" {
                continue;
            }

            if entry_path.is_file() && file_name.ends_with(".yml") {
                if self.validate_schemas {
                    // Validate custom type definition
                    match self.validate_type_definition(&entry_path) {
                        Ok(_) => custom_type_count += 1,
                        Err(reason) => {
                            issues.push(ValidationIssue::InvalidFormat {
                                path: format!("types/{}", file_name),
                                reason,
                            });
                        }
                    }
                } else {
                    custom_type_count += 1;
                }
            }
        }

        stats.custom_type_count = custom_type_count;
        Ok(())
    }

    /// Validate individual credential record
    fn validate_credential_record(&self, record_path: &Path) -> Result<CredentialRecord, String> {
        let content = fs::read_to_string(record_path)
            .map_err(|e| format!("Failed to read credential file: {}", e))?;

        let credential: CredentialRecord = serde_yaml::from_str(&content)
            .map_err(|e| format!("Invalid credential YAML: {}", e))?;

        // Basic validation
        if credential.id.is_empty() {
            return Err("Credential ID cannot be empty".to_string());
        }

        if credential.title.trim().is_empty() {
            return Err("Credential title cannot be empty".to_string());
        }

        Ok(credential)
    }

    /// Validate custom type definition
    fn validate_type_definition(&self, type_path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(type_path)
            .map_err(|e| format!("Failed to read type file: {}", e))?;

        let _type_def: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("Invalid type definition YAML: {}", e))?;

        // Additional type validation could be added here
        Ok(())
    }

    /// Calculate total size of directory recursively
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_directory_size(&self, dir_path: &Path) -> BackendResult<u64> {
        let mut total_size = 0u64;

        if dir_path.is_file() {
            return Ok(fs::metadata(dir_path)?.len());
        }

        let entries = fs::read_dir(dir_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                total_size += self.calculate_directory_size(&path)?;
            } else {
                total_size += fs::metadata(&path)?.len();
            }
        }

        Ok(total_size)
    }

    /// Repair missing required files/directories
    fn repair_missing_required(
        &self,
        repository_path: &Path,
        missing_path: &str,
    ) -> BackendResult<bool> {
        match missing_path {
            "credentials/" => {
                let credentials_dir = repository_path.join("credentials");
                fs::create_dir_all(&credentials_dir)?;

                // Add placeholder file
                let placeholder = credentials_dir.join(".gitkeep");
                fs::write(&placeholder, "# ZipLock credentials directory\n# This file ensures the directory is preserved in the archive\n")?;

                info!("Repaired: Created missing credentials directory");
                Ok(true)
            }
            "types/" => {
                let types_dir = repository_path.join("types");
                fs::create_dir_all(&types_dir)?;

                // Add placeholder file
                let placeholder = types_dir.join(".gitkeep");
                fs::write(&placeholder, "# ZipLock custom types directory\n# This file ensures the directory is preserved in the archive\n")?;

                info!("Repaired: Created missing types directory");
                Ok(true)
            }
            "metadata.yml" => {
                let metadata_path = repository_path.join("metadata.yml");

                // Count existing credentials to set accurate metadata
                let credentials_dir = repository_path.join("credentials");
                let credential_count = if credentials_dir.exists() {
                    fs::read_dir(&credentials_dir)
                        .map(|entries| {
                            entries
                                .filter_map(|entry| entry.ok())
                                .filter(|entry| {
                                    if let Ok(file_type) = entry.file_type() {
                                        file_type.is_dir()
                                            || (file_type.is_file()
                                                && entry
                                                    .path()
                                                    .extension()
                                                    .and_then(|s| s.to_str())
                                                    == Some("yml")
                                                && entry.file_name() != ".gitkeep")
                                    } else {
                                        false
                                    }
                                })
                                .count()
                        })
                        .unwrap_or(0)
                } else {
                    0
                };

                let metadata_content = format!(
                    "version: \"1.0.0\"\ncreated_at: !Timestamp\n  secs_since_epoch: {}\n  nanos_since_epoch: 0\ncredential_count: {}\n",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    credential_count
                );

                fs::write(&metadata_path, metadata_content)?;
                info!("Repaired: Created missing metadata.yml file");
                Ok(true)
            }
            _ => {
                warn!("Cannot auto-repair missing: {}", missing_path);
                Ok(false)
            }
        }
    }

    /// Repair structural issues
    fn repair_structural_issue(
        &self,
        _repository_path: &Path,
        description: &str,
    ) -> BackendResult<bool> {
        // Placeholder for structural repairs
        debug!("Structural issue repair not implemented: {}", description);
        Ok(false)
    }

    /// Migrate legacy format to current format
    fn migrate_legacy_format(&self, repository_path: &Path) -> BackendResult<bool> {
        info!("Performing legacy format migration to repository format v1.0");
        let mut migration_performed = false;

        // Check if we need to update the version in metadata.yml
        let metadata_path = repository_path.join("metadata.yml");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)?;
            let mut metadata: serde_yaml::Value =
                serde_yaml::from_str(&content).map_err(|e| StorageError::InvalidRecord {
                    reason: format!("Failed to parse metadata for migration: {}", e),
                })?;

            // Update version to current
            if let Some(version_field) = metadata.get("version") {
                let current_version = version_field.as_str().unwrap_or("unknown");
                if current_version != RepositoryVersion::CURRENT.to_string() {
                    info!(
                        "Upgrading repository version from {} to {}",
                        current_version,
                        RepositoryVersion::CURRENT
                    );

                    metadata["version"] =
                        serde_yaml::Value::String(RepositoryVersion::CURRENT.to_string());

                    let updated_content = serde_yaml::to_string(&metadata).map_err(|e| {
                        StorageError::InvalidRecord {
                            reason: format!("Failed to serialize updated metadata: {}", e),
                        }
                    })?;

                    fs::write(&metadata_path, updated_content)?;
                    info!(
                        "Repository version successfully upgraded to {}",
                        RepositoryVersion::CURRENT
                    );
                    migration_performed = true;
                }
            }
        }

        let credentials_dir = repository_path.join("credentials");
        if !credentials_dir.exists() {
            return Ok(false);
        }

        let entries = fs::read_dir(&credentials_dir)?;
        let mut migrations_performed = 0;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file()
                && entry_path.extension().and_then(|ext| ext.to_str()) == Some("yml")
            {
                let file_name = entry_path.file_stem().and_then(|name| name.to_str());
                if let Some(credential_id) = file_name {
                    // Create new directory structure
                    let credential_dir = credentials_dir.join(credential_id);
                    fs::create_dir_all(&credential_dir)?;

                    // Move file to record.yml
                    let new_path = credential_dir.join("record.yml");
                    fs::rename(&entry_path, &new_path)?;

                    migrations_performed += 1;
                    info!(
                        "Migrated credential: {} -> {}/record.yml",
                        credential_id, credential_id
                    );
                }
            }
        }

        if migrations_performed > 0 {
            info!(
                "Legacy format migration completed: {} credentials migrated",
                migrations_performed
            );
            migration_performed = true;
        }

        if migration_performed {
            info!("Legacy format migration completed successfully");
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_version_parsing() {
        let version = RepositoryVersion::parse("1.0.0").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre_release, None);

        let version = RepositoryVersion::parse("2.1.3-beta").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, Some("beta".to_string()));

        assert!(RepositoryVersion::parse("invalid").is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = RepositoryVersion::parse("1.0.0").unwrap();
        let v1_1_0 = RepositoryVersion::parse("1.1.0").unwrap();
        let v2_0_0 = RepositoryVersion::parse("2.0.0").unwrap();

        assert!(v1_0_0.is_compatible_with(&v1_1_0));
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));

        assert!(v1_1_0.is_newer_than(&v1_0_0));
        assert!(!v1_0_0.is_newer_than(&v1_1_0));
    }

    #[test]
    fn test_repository_validation() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create minimal valid repository structure
        fs::create_dir(repo_path.join("credentials")).unwrap();
        fs::create_dir(repo_path.join("types")).unwrap();
        fs::write(
            repo_path.join("metadata.yml"),
            "version: 1.0.0\ncredential_count: 0\n",
        )
        .unwrap();
        fs::write(repo_path.join("credentials/.gitkeep"), "").unwrap();
        fs::write(repo_path.join("types/.gitkeep"), "").unwrap();

        let validator = RepositoryValidator::new();
        let report = validator.validate(repo_path).unwrap();

        assert!(report.is_valid);
        assert_eq!(report.issues.len(), 0);
        assert_eq!(report.stats.credential_count, 0);
    }

    #[test]
    fn test_repository_auto_repair() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create repository with missing directories
        fs::write(
            repo_path.join("metadata.yml"),
            "version: 1.0.0\ncredential_count: 0\n",
        )
        .unwrap();

        let validator = RepositoryValidator::new();
        let initial_report = validator.validate(repo_path).unwrap();
        assert!(!initial_report.is_valid);
        assert!(initial_report.can_auto_repair);

        let repaired_report = validator.auto_repair(repo_path).unwrap();
        assert!(repaired_report.is_valid);
        assert!(repo_path.join("credentials").exists());
        assert!(repo_path.join("types").exists());
    }
}
