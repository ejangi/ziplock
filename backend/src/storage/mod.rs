//! Storage module for ZipLock backend
//!
//! This module handles all operations with encrypted 7z archives containing
//! credential records. It provides a safe abstraction over the sevenz-rust2
//! crate with proper file locking, backup management, and error handling.
//!
//! Key features:
//! - Thread-safe archive operations
//! - Automatic backup creation before modifications
//! - File locking to prevent concurrent access
//! - YAML credential record management
//! - Archive integrity verification
//! - Advanced compression configuration (solid compression, compression levels)

pub mod file_lock;
pub mod validation;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sevenz_rust2::{encoder_options, ArchiveWriter};
use std::collections::HashMap;
use std::fs;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::StorageConfig;
use crate::error::{BackendResult, CryptoError, StorageError};
use file_lock::FileLock;
use ziplock_shared::models::CredentialRecord;

/// Archive manager handles all 7z archive operations
#[derive(Debug)]
pub struct ArchiveManager {
    config: StorageConfig,
    current_archive: Arc<RwLock<Option<OpenArchive>>>,
}

/// Represents an open, decrypted archive
#[derive(Debug)]
struct OpenArchive {
    path: PathBuf,
    master_password: String,
    temp_dir: TempDir,
    file_lock: FileLock,
    credentials: HashMap<String, CredentialRecord>,
    modified: bool,
    last_access: SystemTime,
}

/// Archive metadata stored in the archive
#[derive(Debug, Serialize, Deserialize)]
struct ArchiveMetadata {
    version: String,
    created_at: SystemTime,
    last_modified: SystemTime,
    credential_count: usize,
}

impl ArchiveManager {
    /// Create a new archive manager
    pub fn new(config: StorageConfig) -> BackendResult<Self> {
        // Ensure directories exist
        fs::create_dir_all(&config.default_archive_dir)
            .with_context(|| {
                format!(
                    "Failed to create archive directory: {:?}",
                    config.default_archive_dir
                )
            })
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        Ok(Self {
            config,
            current_archive: Arc::new(RwLock::new(None)),
        })
    }

    /// Check if an archive is currently open
    pub async fn is_open(&self) -> bool {
        self.current_archive.read().await.is_some()
    }

    /// Create a new encrypted archive
    pub async fn create_archive<P: AsRef<Path>>(
        &self,
        path: P,
        master_password: &str,
    ) -> BackendResult<()> {
        let path = path.as_ref().to_path_buf();
        info!("Creating new archive at: {:?}", path);

        if path.exists() {
            return Err(StorageError::ArchiveCreation {
                reason: format!("Archive already exists at {:?}", path),
            }
            .into());
        }

        // Validate master password
        if master_password.is_empty() {
            return Err(StorageError::InvalidRecord {
                reason: "Master password cannot be empty".to_string(),
            }
            .into());
        }

        if master_password.len() < self.config.min_password_length.unwrap_or(8) {
            return Err(StorageError::InvalidRecord {
                reason: format!(
                    "Master password must be at least {} characters",
                    self.config.min_password_length.unwrap_or(8)
                ),
            }
            .into());
        }

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {:?}", parent))
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;
        }

        // Create temporary directory for archive operations
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        // Create metadata file
        let metadata = ArchiveMetadata {
            version: "1.0.0".to_string(), // Repository format version
            created_at: SystemTime::now(),
            last_modified: SystemTime::now(),
            credential_count: 0,
        };

        let metadata_path = temp_dir.path().join("metadata.yml");
        let metadata_content = serde_yaml::to_string(&metadata)
            .context("Failed to serialize metadata")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        fs::write(&metadata_path, metadata_content)
            .context("Failed to write metadata file")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        // Create repository structure per specification v1.0
        let credentials_dir = temp_dir.path().join("credentials");
        let types_dir = temp_dir.path().join("types");

        fs::create_dir(&credentials_dir)
            .context("Failed to create credentials directory")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        fs::create_dir(&types_dir)
            .context("Failed to create types directory")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        // Add placeholder files to ensure directories are preserved in archive
        let credentials_placeholder = credentials_dir.join(".gitkeep");
        fs::write(&credentials_placeholder, "# ZipLock credentials directory\n# This file ensures the directory is preserved in the archive\n")
            .context("Failed to write credentials placeholder")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        let types_placeholder = types_dir.join(".gitkeep");
        fs::write(&types_placeholder, "# ZipLock custom types directory\n# This file ensures the directory is preserved in the archive\n")
            .context("Failed to write types placeholder")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        // Create the encrypted 7z archive
        self.create_encrypted_archive(&path, temp_dir.path(), master_password)
            .await
            .context("Failed to create encrypted archive")?;

        info!("Successfully created new archive at: {:?}", path);
        Ok(())
    }

    /// Open an existing encrypted archive
    pub async fn open_archive<P: AsRef<Path>>(
        &self,
        path: P,
        master_password: &str,
    ) -> BackendResult<()> {
        let path = path.as_ref().to_path_buf();
        info!("Opening archive: {:?}", path);

        // Check if archive exists
        if !path.exists() {
            return Err(StorageError::ArchiveNotFound {
                path: path.to_string_lossy().to_string(),
            }
            .into());
        }

        // Check if an archive is already open
        if self.is_open().await {
            warn!("Closing existing archive before opening new one");
            self.close_archive().await?;
        }

        // Acquire file lock
        let file_lock = FileLock::new(&path, self.config.file_lock_timeout).map_err(|_e| {
            StorageError::FileLock {
                path: path.to_string_lossy().to_string(),
            }
        })?;

        // Create temporary directory for extraction
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory")
            .map_err(|e| StorageError::ArchiveOpen {
                reason: e.to_string(),
            })?;

        // Extract the archive
        self.extract_archive(&path, temp_dir.path(), master_password)
            .await
            .context("Failed to extract archive")?;

        // Validate repository format and auto-repair if needed
        let was_repaired = self
            .validate_repository_format(temp_dir.path())
            .context("Repository format validation failed")?;

        // Verify archive integrity after potential repairs
        if self.config.verify_integrity {
            self.verify_archive_integrity(temp_dir.path())
                .context("Archive integrity check failed")?;
        }

        // If repairs were made, save the repaired archive
        if was_repaired {
            info!("Repository format was repaired, saving updated archive");
            self.create_encrypted_archive(&path, temp_dir.path(), master_password)
                .await
                .context("Failed to save repaired archive")?;
        }

        // Load credentials from extracted files
        let credentials = self
            .load_credentials_from_directory(temp_dir.path())
            .await
            .context("Failed to load credentials")?;

        // Create and store the open archive
        let open_archive = OpenArchive {
            path: path.clone(),
            master_password: master_password.to_string(),
            temp_dir,
            file_lock,
            credentials,
            modified: false,
            last_access: SystemTime::now(),
        };

        *self.current_archive.write().await = Some(open_archive);
        info!("Successfully opened archive: {:?}", path);
        Ok(())
    }

    /// Close the currently open archive
    pub async fn close_archive(&self) -> BackendResult<()> {
        let mut archive_guard = self.current_archive.write().await;

        if let Some(archive) = archive_guard.take() {
            info!("Closing archive: {:?}", archive.path);

            // Save changes if modified
            if archive.modified {
                info!("Archive was modified, saving changes");
                drop(archive_guard); // Release lock before async operation
                self.save_archive_internal(archive).await?;
            } else {
                info!("Archive not modified, closing without save");
                drop(archive_guard); // Release lock to avoid Send trait issues
                                     // File lock will be released when archive is dropped
            }
        } else {
            drop(archive_guard); // Release lock even when no archive is open
        }

        Ok(())
    }

    /// Get a credential by ID
    pub async fn get_credential(&self, id: &str) -> BackendResult<CredentialRecord> {
        let archive = self.current_archive.read().await;
        let archive = archive.as_ref().ok_or(StorageError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive
            .credentials
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::RecordNotFound { id: id.to_string() }.into())
    }

    /// List all credentials (metadata only)
    pub async fn list_credentials(&self) -> BackendResult<Vec<CredentialRecord>> {
        let archive = self.current_archive.read().await;
        let archive = archive.as_ref().ok_or(StorageError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        Ok(archive.credentials.values().cloned().collect())
    }

    /// Add a new credential
    pub async fn add_credential(&self, mut credential: CredentialRecord) -> BackendResult<String> {
        let mut archive_guard = self.current_archive.write().await;
        let archive = archive_guard.as_mut().ok_or(StorageError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        // Generate ID if not provided
        if credential.id.is_empty() {
            credential.id = Uuid::new_v4().to_string();
        }

        // Check if credential already exists
        if archive.credentials.contains_key(&credential.id) {
            return Err(StorageError::InvalidRecord {
                reason: format!("Credential with ID {} already exists", credential.id),
            }
            .into());
        }

        // Update timestamps
        let now = SystemTime::now();
        credential.created_at = now;
        credential.updated_at = now;

        // Validate credential
        self.validate_credential(&credential)?;

        // Add to collection
        let id = credential.id.clone();
        archive.credentials.insert(id.clone(), credential);
        archive.modified = true;
        archive.last_access = now;

        info!("Added credential with ID: {}", id);
        Ok(id)
    }

    /// Update an existing credential
    pub async fn update_credential(
        &self,
        id: &str,
        mut credential: CredentialRecord,
    ) -> BackendResult<()> {
        let mut archive_guard = self.current_archive.write().await;
        let archive = archive_guard.as_mut().ok_or(StorageError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        // Check if credential exists
        if !archive.credentials.contains_key(id) {
            return Err(StorageError::RecordNotFound { id: id.to_string() }.into());
        }

        // Preserve ID and creation time
        let existing = &archive.credentials[id];
        credential.id = id.to_string();
        credential.created_at = existing.created_at;
        credential.updated_at = SystemTime::now();

        // Validate credential
        self.validate_credential(&credential)?;

        // Update in collection
        archive.credentials.insert(id.to_string(), credential);
        archive.modified = true;
        archive.last_access = SystemTime::now();

        info!("Updated credential with ID: {}", id);
        Ok(())
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: &str) -> BackendResult<()> {
        let mut archive_guard = self.current_archive.write().await;
        let archive = archive_guard.as_mut().ok_or(StorageError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        if archive.credentials.remove(id).is_none() {
            return Err(StorageError::RecordNotFound { id: id.to_string() }.into());
        }

        archive.modified = true;
        archive.last_access = SystemTime::now();

        info!("Deleted credential with ID: {}", id);
        Ok(())
    }

    /// Search credentials by query
    pub async fn search_credentials(&self, query: &str) -> BackendResult<Vec<CredentialRecord>> {
        let archive = self.current_archive.read().await;
        let archive = archive.as_ref().ok_or(StorageError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        let query = query.to_lowercase();
        let results = archive
            .credentials
            .values()
            .filter(|cred| {
                cred.title.to_lowercase().contains(&query)
                    || cred
                        .notes
                        .as_ref()
                        .map_or(false, |notes| notes.to_lowercase().contains(&query))
                    || cred
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query))
                    || cred
                        .fields
                        .values()
                        .any(|field| field.value.to_lowercase().contains(&query))
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// Save the current archive
    pub async fn save_archive(&self) -> BackendResult<()> {
        let archive_guard = self.current_archive.write().await;

        if let Some(archive) = archive_guard.as_ref() {
            if archive.modified {
                info!("Saving modified archive: {:?}", archive.path);

                // Clone the archive data for async operation
                let archive_data = OpenArchive {
                    path: archive.path.clone(),
                    master_password: archive.master_password.clone(),
                    temp_dir: TempDir::new()
                        .context("Failed to create temp dir")
                        .map_err(|e| StorageError::ArchiveCreation {
                            reason: e.to_string(),
                        })?,
                    file_lock: FileLock::new(&archive.path, self.config.file_lock_timeout)
                        .map_err(|_e| StorageError::FileLock {
                            path: archive.path.to_string_lossy().to_string(),
                        })?,
                    credentials: archive.credentials.clone(),
                    modified: archive.modified,
                    last_access: archive.last_access,
                };

                drop(archive_guard); // Release lock before async operation
                self.save_archive_internal(archive_data).await?;

                // Mark as not modified
                if let Some(archive) = self.current_archive.write().await.as_mut() {
                    archive.modified = false;
                }
            }
        }

        Ok(())
    }

    /// Internal method to save archive
    async fn save_archive_internal(&self, archive: OpenArchive) -> BackendResult<()> {
        // Create backup if enabled
        if self.config.auto_backup {
            self.create_backup(&archive.path).await?;
        }

        // Write credentials to temporary directory
        self.write_credentials_to_directory(&archive.temp_dir.path(), &archive.credentials)
            .await?;

        // Update metadata
        let metadata = ArchiveMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: SystemTime::now(), // TODO: preserve original created_at
            last_modified: SystemTime::now(),
            credential_count: archive.credentials.len(),
        };

        let metadata_path = archive.temp_dir.path().join("metadata.yml");
        let metadata_content = serde_yaml::to_string(&metadata)
            .context("Failed to serialize metadata")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        fs::write(&metadata_path, metadata_content)
            .context("Failed to write metadata")
            .map_err(|e| StorageError::ArchiveCreation {
                reason: e.to_string(),
            })?;

        // Create the encrypted archive
        self.create_encrypted_archive(
            &archive.path,
            archive.temp_dir.path(),
            &archive.master_password,
        )
        .await?;

        info!("Successfully saved archive: {:?}", archive.path);
        Ok(())
    }

    /// Create a backup of the archive
    async fn create_backup(&self, archive_path: &Path) -> BackendResult<()> {
        if !archive_path.exists() {
            return Ok(());
        }

        let backup_dir = self
            .config
            .backup_dir
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.config.default_archive_dir.join("backups"));

        fs::create_dir_all(&backup_dir)
            .context("Failed to create backup directory")
            .map_err(|e| StorageError::BackupFailed {
                reason: e.to_string(),
            })?;

        // Generate backup filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let filename = archive_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        let backup_filename = format!("{}_backup_{}.7z", filename, timestamp);
        let backup_path = backup_dir.join(backup_filename);

        // Copy the archive
        fs::copy(archive_path, &backup_path)
            .context("Failed to create backup")
            .map_err(|e| StorageError::BackupFailed {
                reason: e.to_string(),
            })?;

        // Clean up old backups if needed
        self.cleanup_old_backups(&backup_dir, &filename).await?;

        debug!("Created backup: {:?}", backup_path);
        Ok(())
    }

    /// Clean up old backup files
    async fn cleanup_old_backups(
        &self,
        backup_dir: &Path,
        filename_prefix: &str,
    ) -> BackendResult<()> {
        if self.config.backup_count == 0 {
            return Ok(());
        }

        let mut backups = Vec::new();

        let entries = fs::read_dir(backup_dir)
            .context("Failed to read backup directory")
            .map_err(|e| StorageError::BackupFailed {
                reason: e.to_string(),
            })?;

        for entry in entries {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(|e| StorageError::BackupFailed {
                    reason: e.to_string(),
                })?;

            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            if filename_str.starts_with(filename_prefix) && filename_str.contains("_backup_") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((entry.path(), modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess backups
        for (path, _) in backups.into_iter().skip(self.config.backup_count) {
            if let Err(e) = fs::remove_file(&path) {
                warn!("Failed to remove old backup {:?}: {}", path, e);
            } else {
                debug!("Removed old backup: {:?}", path);
            }
        }

        Ok(())
    }

    /// Validate a credential record
    fn validate_credential(&self, credential: &CredentialRecord) -> BackendResult<()> {
        if credential.title.trim().is_empty() {
            return Err(StorageError::InvalidRecord {
                reason: "Credential title cannot be empty".to_string(),
            }
            .into());
        }

        if credential.id.is_empty() {
            return Err(StorageError::InvalidRecord {
                reason: "Credential ID cannot be empty".to_string(),
            }
            .into());
        }

        // Additional validation can be added here
        Ok(())
    }

    /// Load credentials from directory
    async fn load_credentials_from_directory(
        &self,
        dir: &Path,
    ) -> BackendResult<HashMap<String, CredentialRecord>> {
        let credentials_dir = dir.join("credentials");
        let mut credentials = HashMap::new();

        if !credentials_dir.exists() {
            return Ok(credentials);
        }

        let entries = fs::read_dir(&credentials_dir)
            .context("Failed to read credentials directory")
            .map_err(|e| StorageError::ArchiveExtract {
                reason: e.to_string(),
            })?;

        for entry in entries {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(|e| StorageError::ArchiveExtract {
                    reason: e.to_string(),
                })?;

            let path = entry.path();

            // Skip placeholder files
            if path.file_name().and_then(|s| s.to_str()) == Some(".gitkeep") {
                continue;
            }

            // Handle both legacy format (direct .yml files) and spec format (folders with record.yml)
            if path.is_dir() {
                // Repository format v1.0: /credentials/credential-name/record.yml
                let record_file = path.join("record.yml");
                if record_file.exists() {
                    let content = fs::read_to_string(&record_file)
                        .context("Failed to read credential record.yml")
                        .map_err(|e| StorageError::ArchiveExtract {
                            reason: e.to_string(),
                        })?;

                    let credential: CredentialRecord = serde_yaml::from_str(&content)
                        .context("Failed to parse credential YAML")
                        .map_err(|e| StorageError::InvalidRecord {
                            reason: e.to_string(),
                        })?;

                    credentials.insert(credential.id.clone(), credential);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                // Legacy format: /credentials/credential-id.yml
                let content = fs::read_to_string(&path)
                    .context("Failed to read credential file")
                    .map_err(|e| StorageError::ArchiveExtract {
                        reason: e.to_string(),
                    })?;

                let credential: CredentialRecord = serde_yaml::from_str(&content)
                    .context("Failed to parse credential YAML")
                    .map_err(|e| StorageError::InvalidRecord {
                        reason: e.to_string(),
                    })?;

                credentials.insert(credential.id.clone(), credential);
            }
        }

        debug!("Loaded {} credentials from directory", credentials.len());
        Ok(credentials)
    }

    /// Write credentials to directory
    async fn write_credentials_to_directory(
        &self,
        dir: &Path,
        credentials: &HashMap<String, CredentialRecord>,
    ) -> BackendResult<()> {
        let credentials_dir = dir.join("credentials");

        // Clear existing credential files but preserve directory structure and placeholders
        if credentials_dir.exists() {
            let entries = fs::read_dir(&credentials_dir)
                .context("Failed to read credentials directory")
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;

            for entry in entries {
                let entry = entry
                    .context("Failed to read directory entry")
                    .map_err(|e| StorageError::ArchiveCreation {
                        reason: e.to_string(),
                    })?;

                let path = entry.path();

                // Skip placeholder files
                if path.file_name().and_then(|s| s.to_str()) == Some(".gitkeep") {
                    continue;
                }

                // Remove credential directories and files
                if path.is_dir() || path.extension().and_then(|s| s.to_str()) == Some("yml") {
                    if path.is_dir() {
                        fs::remove_dir_all(&path)
                    } else {
                        fs::remove_file(&path)
                    }
                    .with_context(|| format!("Failed to remove existing credential: {:?}", path))
                    .map_err(|e| StorageError::ArchiveCreation {
                        reason: e.to_string(),
                    })?;
                }
            }
        } else {
            fs::create_dir(&credentials_dir)
                .context("Failed to create credentials directory")
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;
        }

        // Write credentials using repository format v1.0: /credentials/credential-id/record.yml
        for credential in credentials.values() {
            let credential_dir = credentials_dir.join(&credential.id);

            fs::create_dir(&credential_dir)
                .with_context(|| {
                    format!(
                        "Failed to create credential directory: {:?}",
                        credential_dir
                    )
                })
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;

            let record_file = credential_dir.join("record.yml");
            let content = serde_yaml::to_string(credential)
                .context("Failed to serialize credential")
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;

            fs::write(&record_file, content)
                .with_context(|| format!("Failed to write credential record: {:?}", record_file))
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;
        }

        // Ensure placeholder file exists to preserve directory in archive
        let placeholder = credentials_dir.join(".gitkeep");
        if !placeholder.exists() {
            fs::write(&placeholder, "# ZipLock credentials directory\n# This file ensures the directory is preserved in the archive\n")
                .context("Failed to write credentials placeholder")
                .map_err(|e| StorageError::ArchiveCreation {
                    reason: e.to_string(),
                })?;
        }

        debug!("Wrote {} credentials to directory", credentials.len());
        Ok(())
    }

    /// Create encrypted 7z archive with advanced compression settings
    async fn create_encrypted_archive(
        &self,
        archive_path: &Path,
        source_dir: &Path,
        password: &str,
    ) -> BackendResult<()> {
        info!("Creating encrypted archive: {:?}", archive_path);
        debug!("Source directory: {:?}", source_dir);

        let compression_config = &self.config.compression;

        // Use advanced sevenz_rust2 API for better control
        let result = tokio::task::spawn_blocking({
            let archive_path = archive_path.to_owned();
            let source_dir = source_dir.to_owned();
            let password = password.to_owned();
            let config = compression_config.clone();

            move || -> Result<(), sevenz_rust2::Error> {
                if config.solid {
                    // Use solid compression for better compression ratio
                    info!("Using solid compression with level {}", config.level);
                    let mut writer = ArchiveWriter::create(&archive_path)?;

                    // Configure compression methods with encryption and LZMA2
                    let mut methods = Vec::new();
                    methods.push(
                        encoder_options::AesEncoderOptions::new(password.as_str().into()).into(),
                    );

                    // Configure LZMA2 with dictionary size and multi-threading
                    let lzma2_opts = encoder_options::LZMA2Options::from_level(config.level.into());
                    // Note: LZMA2Options in sevenz-rust2 doesn't expose dictionary_size and num_threads
                    // These are internal optimizations handled by the library
                    methods.push(lzma2_opts.into());

                    writer.set_content_methods(methods);

                    // Note: Block size configuration not exposed in current sevenz-rust2 API

                    writer.push_source_path(&source_dir, |_| true)?;
                    writer.finish()?;
                } else {
                    // Use convenience function for faster random access
                    info!("Using non-solid compression for better random access");
                    sevenz_rust2::compress_to_path_encrypted(
                        &source_dir,
                        &archive_path,
                        password.as_str().into(),
                    )?;
                }
                Ok(())
            }
        })
        .await;

        match result {
            Ok(Ok(())) => {
                info!(
                    "Successfully created encrypted 7z archive: {:?}",
                    archive_path
                );
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Failed to create 7z archive: {}", e);
                Err(StorageError::ArchiveCreation {
                    reason: format!("7z compression failed: {}", e),
                }
                .into())
            }
            Err(e) => {
                error!("Task execution failed: {}", e);
                Err(StorageError::ArchiveCreation {
                    reason: format!("Task failed: {}", e),
                }
                .into())
            }
        }
    }

    /// Extract encrypted 7z archive
    async fn extract_archive(
        &self,
        archive_path: &Path,
        dest_dir: &Path,
        password: &str,
    ) -> BackendResult<()> {
        info!("Extracting archive: {:?}", archive_path);
        debug!("Destination directory: {:?}", dest_dir);

        // Create destination directory
        fs::create_dir_all(dest_dir)
            .context("Failed to create destination directory")
            .map_err(|e| StorageError::ArchiveExtract {
                reason: e.to_string(),
            })?;

        // Use sevenz_rust's extraction function
        let result = tokio::task::spawn_blocking({
            let archive_path = archive_path.to_owned();
            let dest_dir = dest_dir.to_owned();
            let password = password.to_owned();

            move || {
                sevenz_rust2::decompress_file_with_password(
                    &archive_path,
                    &dest_dir,
                    password.as_str().into(),
                )
            }
        })
        .await;

        match result {
            Ok(Ok(())) => {
                info!("Successfully extracted archive to: {:?}", dest_dir);
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Failed to extract 7z archive: {}", e);

                // Check for password-related errors
                let error_string = e.to_string();
                if error_string.contains("MaybeBadPassword")
                    || error_string.contains("range decoder first byte is 0")
                    || error_string.contains("Invalid password")
                    || error_string.contains("Wrong password")
                {
                    Err(crate::error::CryptoError::InvalidMasterKey.into())
                } else {
                    Err(StorageError::ArchiveExtract {
                        reason: format!("Archive extraction failed: {}", e),
                    }
                    .into())
                }
            }
            Err(e) => {
                error!("Task execution failed: {}", e);
                Err(StorageError::ArchiveExtract {
                    reason: format!("Task failed: {}", e),
                }
                .into())
            }
        }
    }

    /// Verify archive integrity according to repository format v1.0
    fn verify_archive_integrity(&self, dir: &Path) -> BackendResult<()> {
        // Check that required files exist per repository format v1.0
        let metadata_path = dir.join("metadata.yml");
        let credentials_dir = dir.join("credentials");
        let types_dir = dir.join("types");

        if !metadata_path.exists() {
            return Err(StorageError::CorruptedArchive {
                reason: "Missing metadata.yml file - repository format v1.0 requires this file"
                    .to_string(),
            }
            .into());
        }

        if !credentials_dir.exists() {
            return Err(StorageError::CorruptedArchive {
                reason: "Missing /credentials directory - repository format v1.0 requires this directory".to_string(),
            }
            .into());
        }

        if !credentials_dir.is_dir() {
            return Err(StorageError::CorruptedArchive {
                reason: "/credentials exists but is not a directory".to_string(),
            }
            .into());
        }

        if !types_dir.exists() {
            return Err(StorageError::CorruptedArchive {
                reason: "Missing /types directory - repository format v1.0 requires this directory"
                    .to_string(),
            }
            .into());
        }

        if !types_dir.is_dir() {
            return Err(StorageError::CorruptedArchive {
                reason: "/types exists but is not a directory".to_string(),
            }
            .into());
        }

        // Validate metadata format
        let metadata_content = fs::read_to_string(&metadata_path)
            .context("Failed to read metadata")
            .map_err(|e| StorageError::CorruptedArchive {
                reason: e.to_string(),
            })?;

        let metadata: ArchiveMetadata = serde_yaml::from_str(&metadata_content)
            .context("Invalid metadata format")
            .map_err(|e| StorageError::CorruptedArchive {
                reason: e.to_string(),
            })?;

        // Validate repository format version
        if metadata.version != "1.0.0" {
            info!(
                "Repository format version mismatch: archive={}, current=1.0.0",
                metadata.version
            );
            // Note: For now we're permissive, but this is where we'd handle version migration
        }

        debug!("Archive integrity verification passed for repository format v1.0");
        Ok(())
    }

    /// Validate and potentially repair repository format issues
    fn validate_repository_format(&self, dir: &Path) -> BackendResult<bool> {
        let mut repaired = false;
        let types_dir = dir.join("types");
        let credentials_dir = dir.join("credentials");

        // Ensure types directory exists (auto-repair)
        if !types_dir.exists() {
            info!("Auto-repairing: Creating missing /types directory");
            fs::create_dir(&types_dir)
                .context("Failed to create types directory during repair")
                .map_err(|e| StorageError::CorruptedArchive {
                    reason: e.to_string(),
                })?;

            // Add placeholder to ensure it's preserved
            let types_placeholder = types_dir.join(".gitkeep");
            fs::write(&types_placeholder, "# ZipLock custom types directory\n# This file ensures the directory is preserved in the archive\n")
                .context("Failed to write types placeholder during repair")
                .map_err(|e| StorageError::CorruptedArchive {
                    reason: e.to_string(),
                })?;
            repaired = true;
        }

        // Ensure credentials directory has proper structure
        if !credentials_dir.exists() {
            info!("Auto-repairing: Creating missing /credentials directory");
            fs::create_dir(&credentials_dir)
                .context("Failed to create credentials directory during repair")
                .map_err(|e| StorageError::CorruptedArchive {
                    reason: e.to_string(),
                })?;

            // Add placeholder to ensure it's preserved
            let credentials_placeholder = credentials_dir.join(".gitkeep");
            fs::write(&credentials_placeholder, "# ZipLock credentials directory\n# This file ensures the directory is preserved in the archive\n")
                .context("Failed to write credentials placeholder during repair")
                .map_err(|e| StorageError::CorruptedArchive {
                    reason: e.to_string(),
                })?;
            repaired = true;
        }

        if repaired {
            info!("Repository format validation completed with repairs");
        } else {
            debug!("Repository format validation passed without repairs needed");
        }

        Ok(repaired)
    }

    /// Validate repository format of an extracted archive directory
    pub fn validate_repository_format_detailed(
        &self,
        dir: &Path,
    ) -> BackendResult<validation::ValidationReport> {
        let validator = validation::RepositoryValidator::new();
        validator.validate(dir)
    }

    /// Auto-repair repository format issues in an extracted archive directory
    pub fn auto_repair_repository_format(
        &self,
        dir: &Path,
    ) -> BackendResult<validation::ValidationReport> {
        let validator = validation::RepositoryValidator::new();
        validator.auto_repair(dir)
    }

    /// Validate an archive file by extracting and checking its format
    pub async fn validate_archive_file<P: AsRef<Path>>(
        &self,
        archive_path: P,
        password: &str,
    ) -> BackendResult<validation::ValidationReport> {
        let archive_path = archive_path.as_ref();

        // Create temporary directory for extraction
        let temp_dir = tempfile::TempDir::new()
            .context("Failed to create temporary directory")
            .map_err(|e| StorageError::ArchiveExtract {
                reason: e.to_string(),
            })?;

        // Extract the archive
        self.extract_archive(archive_path, temp_dir.path(), password)
            .await
            .context("Failed to extract archive for validation")?;

        // Validate the extracted contents
        self.validate_repository_format_detailed(temp_dir.path())
    }

    /// Auto-repair an archive file by extracting, repairing, and re-creating the archive
    pub async fn repair_archive_file<P: AsRef<Path>>(
        &self,
        archive_path: P,
        password: &str,
    ) -> BackendResult<validation::ValidationReport> {
        let archive_path = archive_path.as_ref();

        // Create temporary directory for extraction
        let temp_dir = tempfile::TempDir::new()
            .context("Failed to create temporary directory")
            .map_err(|e| StorageError::ArchiveExtract {
                reason: e.to_string(),
            })?;

        // Extract the archive
        self.extract_archive(archive_path, temp_dir.path(), password)
            .await
            .context("Failed to extract archive for repair")?;

        // Auto-repair the extracted contents
        let report = self.auto_repair_repository_format(temp_dir.path())?;

        // If repairs were made, recreate the archive
        if report.can_auto_repair && !report.is_valid {
            info!("Recreating archive with repaired repository format");
            self.create_encrypted_archive(archive_path, temp_dir.path(), password)
                .await
                .context("Failed to recreate repaired archive")?;
        }

        Ok(report)
    }
}

impl Drop for ArchiveManager {
    fn drop(&mut self) {
        // Note: We can't use async in Drop, so we just log a warning
        // The archive will be cleaned up when the RwLock is dropped
        warn!("ArchiveManager dropped - any open archive will be closed automatically");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_storage_config() -> StorageConfig {
        let temp_dir = tempdir().unwrap();
        StorageConfig {
            default_archive_dir: temp_dir.path().to_path_buf(),
            max_archive_size_mb: 10,
            backup_count: 3,
            auto_backup: true,
            backup_dir: None,
            file_lock_timeout: 5,
            temp_dir: None,
            verify_integrity: true,
            min_password_length: Some(8),
            compression: crate::config::CompressionConfig {
                level: 6,
                solid: false,
                multi_threaded: true,
                dictionary_size_mb: 32,
                block_size_mb: 0,
            },
        }
    }

    #[tokio::test]
    async fn test_archive_manager_creation() {
        let config = test_storage_config();
        let manager = ArchiveManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_create_new_archive() {
        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let archive_path = manager.config.default_archive_dir.join("test.7z");

        let result = manager
            .create_archive(&archive_path, "test_password_123")
            .await;
        assert!(result.is_ok());
        assert!(archive_path.exists());
    }

    #[tokio::test]
    async fn test_weak_password_rejection() {
        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let archive_path = manager.config.default_archive_dir.join("test.7z");

        let result = manager.create_archive(&archive_path, "weak").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_production_password_length_validation() {
        // Test with production config (12 character minimum)
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            default_archive_dir: temp_dir.path().to_path_buf(),
            max_archive_size_mb: 10,
            backup_count: 3,
            auto_backup: true,
            backup_dir: None,
            file_lock_timeout: 5,
            temp_dir: None,
            verify_integrity: true,
            min_password_length: Some(12), // Production setting
            compression: crate::config::CompressionConfig {
                level: 6,
                solid: false,
                multi_threaded: true,
                dictionary_size_mb: 32,
                block_size_mb: 0,
            },
        };
        let manager = ArchiveManager::new(config).unwrap();
        let archive_path = manager.config.default_archive_dir.join("test.7z");

        // Test exact frontend scenario: "ZipLock123!" (11 characters) should fail
        let result = manager.create_archive(&archive_path, "ZipLock123!").await;
        assert!(result.is_err());
        if let Err(e) = result {
            let error_message = e.to_string();
            assert!(error_message.contains("Master password must be at least 12 characters"));
            println!(
                "✅ Correctly rejected 11-character password: {}",
                error_message
            );
        }

        // Test that 12 characters works
        let result = manager.create_archive(&archive_path, "ZipLock123!!").await;
        assert!(result.is_ok(), "12-character password should be accepted");
        println!("✅ Correctly accepted 12-character password");
    }

    #[tokio::test]
    async fn test_password_validation_edge_cases() {
        // Test with production config
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            default_archive_dir: temp_dir.path().to_path_buf(),
            max_archive_size_mb: 10,
            backup_count: 3,
            auto_backup: true,
            backup_dir: None,
            file_lock_timeout: 5,
            temp_dir: None,
            verify_integrity: true,
            min_password_length: Some(12), // Production setting
            compression: crate::config::CompressionConfig {
                level: 6,
                solid: false,
                multi_threaded: true,
                dictionary_size_mb: 32,
                block_size_mb: 0,
            },
        };
        let manager = ArchiveManager::new(config).unwrap();

        // Test empty password
        let archive_path1 = manager.config.default_archive_dir.join("test1.7z");
        let result = manager.create_archive(&archive_path1, "").await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Master password cannot be empty"));
            println!("✅ Correctly rejected empty password");
        }

        // Test exactly minimum length (12 chars)
        let archive_path2 = manager.config.default_archive_dir.join("test2.7z");
        let result = manager.create_archive(&archive_path2, "123456789012").await;
        assert!(result.is_ok(), "Exact minimum length should work");
        println!("✅ Correctly accepted exact minimum length password");

        // Test one character less than minimum (11 chars)
        let archive_path3 = manager.config.default_archive_dir.join("test3.7z");
        let result = manager.create_archive(&archive_path3, "12345678901").await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e
                .to_string()
                .contains("Master password must be at least 12 characters"));
            println!("✅ Correctly rejected 11-character password");
        }
    }

    #[tokio::test]
    async fn test_credential_operations() {
        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let archive_path = manager.config.default_archive_dir.join("test.7z");

        // Create archive
        manager
            .create_archive(&archive_path, "test_password_123")
            .await
            .unwrap();

        // Open archive
        manager
            .open_archive(&archive_path, "test_password_123")
            .await
            .unwrap();

        // Create test credential
        let credential = CredentialRecord {
            id: String::new(), // Will be auto-generated
            title: "Test Credential".to_string(),
            credential_type: "login".to_string(),
            fields: HashMap::new(),
            tags: vec!["test".to_string()],
            notes: Some("Test notes".to_string()),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        // Add credential
        let id = manager.add_credential(credential).await.unwrap();
        assert!(!id.is_empty());

        // Get credential
        let retrieved = manager.get_credential(&id).await.unwrap();
        assert_eq!(retrieved.title, "Test Credential");

        // Update credential
        let mut updated = retrieved.clone();
        updated.title = "Updated Credential".to_string();
        manager.update_credential(&id, updated).await.unwrap();

        // Verify update
        let retrieved = manager.get_credential(&id).await.unwrap();
        assert_eq!(retrieved.title, "Updated Credential");

        // Delete credential
        manager.delete_credential(&id).await.unwrap();

        // Verify deletion
        let result = manager.get_credential(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_credentials() {
        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let archive_path = manager.config.default_archive_dir.join("test.7z");

        // Create and open archive
        manager
            .create_archive(&archive_path, "test_password_123")
            .await
            .unwrap();
        manager
            .open_archive(&archive_path, "test_password_123")
            .await
            .unwrap();

        // Add test credentials
        let mut cred1 = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        cred1.set_field(
            "username",
            ziplock_shared::models::CredentialField {
                value: "testuser".to_string(),
                field_type: ziplock_shared::models::FieldType::Text,
                sensitive: false,
                label: Some("Username".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        );
        manager.add_credential(cred1).await.unwrap();

        let mut cred2 =
            CredentialRecord::new("Test Email Account".to_string(), "login".to_string());
        cred2.set_field(
            "username",
            ziplock_shared::models::CredentialField {
                value: "user@email.com".to_string(),
                field_type: ziplock_shared::models::FieldType::Text,
                sensitive: false,
                label: Some("Username".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        );
        manager.add_credential(cred2).await.unwrap();

        // Search credentials
        let results = manager.search_credentials("test").await.unwrap();
        assert_eq!(results.len(), 2);

        let results = manager.search_credentials("email").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Email Account");

        manager.close_archive().await.unwrap();
    }

    #[tokio::test]
    async fn test_archive_format_validation() {
        use std::process::Command;
        use tempfile::TempDir;

        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("validation_test.7z");
        let extract_dir = temp_dir.path().join("extract_test");

        // Create archive
        let password = "TestPassword123!";
        let result = manager.create_archive(&archive_path, password).await;
        assert!(result.is_ok(), "Failed to create archive: {:?}", result);
        assert!(archive_path.exists(), "Archive file was not created");

        // Verify archive can be extracted by our own extraction method
        let extract_result = manager
            .extract_archive(&archive_path, &extract_dir, password)
            .await;
        assert!(
            extract_result.is_ok(),
            "Failed to extract archive: {:?}",
            extract_result
        );

        // Debug: List all files in our extracted directory
        if let Ok(entries) = std::fs::read_dir(&extract_dir) {
            println!("Files in our extracted directory:");
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let file_type = if path.is_dir() { "DIR" } else { "FILE" };
                    println!("  {} - {}", file_type, path.display());
                }
            }
        }

        // Verify required structure exists
        assert!(extract_dir.exists(), "Extract directory not created");
        assert!(
            extract_dir.join("metadata.yml").exists(),
            "metadata.yml not found"
        );
        assert!(
            extract_dir.join("credentials").exists(),
            "credentials directory not found"
        );
        assert!(
            extract_dir.join("credentials").is_dir(),
            "credentials is not a directory"
        );

        // Verify metadata content
        let metadata_content = std::fs::read_to_string(extract_dir.join("metadata.yml")).unwrap();
        let metadata: ArchiveMetadata = serde_yaml::from_str(&metadata_content).unwrap();
        assert_eq!(metadata.credential_count, 0);
        assert!(!metadata.version.is_empty());

        // Try to extract with system 7z command if available (for Linux compatibility test)
        if Command::new("7z").arg("--help").output().is_ok() {
            let system_extract_dir = temp_dir.path().join("system_extract");
            std::fs::create_dir_all(&system_extract_dir).unwrap();

            let output = Command::new("7z")
                .arg("x")
                .arg(format!("-p{}", password))
                .arg(&archive_path)
                .arg(format!("-o{}", system_extract_dir.display()))
                .arg("-y") // Assume yes for all queries
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    println!("System 7z extraction successful");

                    // Debug: List all files in the extracted directory
                    if let Ok(entries) = std::fs::read_dir(&system_extract_dir) {
                        println!("Files in system extracted directory:");
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                let file_type = if path.is_dir() { "DIR" } else { "FILE" };
                                println!("  {} - {}", file_type, path.display());
                            }
                        }
                    }

                    assert!(
                        system_extract_dir.join("metadata.yml").exists(),
                        "System 7z extraction failed - metadata.yml not found"
                    );
                    assert!(
                        system_extract_dir.join("credentials").exists(),
                        "System 7z extraction failed - credentials directory not found"
                    );
                } else {
                    println!(
                        "System 7z extraction failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    // Don't fail the test if system 7z has issues, but log it
                }
            }
        }

        // Test archive integrity verification
        let integrity_result = manager.verify_archive_integrity(&extract_dir);
        assert!(
            integrity_result.is_ok(),
            "Archive integrity check failed: {:?}",
            integrity_result
        );
    }

    #[tokio::test]
    async fn test_repository_format_v1_0() {
        use tempfile::TempDir;

        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("repo_v1_0.7z");

        // Create archive with version 1.0 format
        let password = "SecurePassword456!";
        manager
            .create_archive(&archive_path, password)
            .await
            .unwrap();

        // Extract and verify structure matches spec v1.0 directly
        let extract_dir = temp_dir.path().join("format_check");
        manager
            .extract_archive(&archive_path, &extract_dir, password)
            .await
            .unwrap();

        // Verify repository format v1.0 structure
        assert!(extract_dir.exists(), "Extract directory not created");

        // Check metadata.yml exists and is valid
        let metadata_file = extract_dir.join("metadata.yml");
        assert!(metadata_file.exists(), "Missing metadata.yml file");
        let metadata_content = std::fs::read_to_string(&metadata_file).unwrap();
        let metadata: ArchiveMetadata = serde_yaml::from_str(&metadata_content).unwrap();
        assert!(!metadata.version.is_empty(), "Version field is empty");
        assert_eq!(
            metadata.credential_count, 0,
            "Initial credential count should be 0"
        );

        // Verify /credentials folder structure per spec
        let credentials_dir = extract_dir.join("credentials");
        assert!(credentials_dir.exists(), "Missing /credentials directory");
        assert!(credentials_dir.is_dir(), "/credentials is not a directory");

        // Verify placeholder file exists to ensure directory preservation
        let credentials_placeholder = credentials_dir.join(".gitkeep");
        assert!(
            credentials_placeholder.exists(),
            "Missing .gitkeep in /credentials"
        );

        // Verify /types folder exists per spec
        let types_dir = extract_dir.join("types");
        assert!(
            types_dir.exists(),
            "Missing /types directory per repository format v1.0"
        );
        assert!(types_dir.is_dir(), "/types is not a directory");

        // Verify placeholder file exists to ensure directory preservation
        let types_placeholder = types_dir.join(".gitkeep");
        assert!(types_placeholder.exists(), "Missing .gitkeep in /types");

        // Test repository format validation
        let validation_result = manager.verify_archive_integrity(&extract_dir);
        assert!(
            validation_result.is_ok(),
            "Repository format v1.0 validation failed: {:?}",
            validation_result
        );

        // Test that system 7z can extract the same structure
        if std::process::Command::new("7z")
            .arg("--help")
            .output()
            .is_ok()
        {
            let system_extract_dir = temp_dir.path().join("system_extract");
            std::fs::create_dir_all(&system_extract_dir).unwrap();

            let output = std::process::Command::new("7z")
                .arg("x")
                .arg(format!("-p{}", password))
                .arg(&archive_path)
                .arg(format!("-o{}", system_extract_dir.display()))
                .arg("-y")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    assert!(
                        system_extract_dir.join("metadata.yml").exists(),
                        "System 7z failed to extract metadata.yml"
                    );
                    assert!(
                        system_extract_dir.join("credentials").exists(),
                        "System 7z failed to extract /credentials"
                    );
                    assert!(
                        system_extract_dir.join("types").exists(),
                        "System 7z failed to extract /types"
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_repository_validation_functionality() {
        use tempfile::TempDir;

        let config = test_storage_config();
        let manager = ArchiveManager::new(config).unwrap();
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("validation_test.7z");

        // Create archive with proper v1.0 format
        let password = "ValidationTest123!";
        manager
            .create_archive(&archive_path, password)
            .await
            .unwrap();

        // Test archive validation
        let validation_report = manager
            .validate_archive_file(&archive_path, password)
            .await
            .unwrap();

        println!("Validation report: {:?}", validation_report);
        assert!(validation_report.is_valid, "Archive should be valid");
        assert_eq!(
            validation_report.issues.len(),
            0,
            "No validation issues expected"
        );
        assert_eq!(
            validation_report.stats.credential_count, 0,
            "No credentials expected in new archive"
        );

        // Test detailed validation on extracted directory
        let extract_dir = temp_dir.path().join("validation_extract");
        manager
            .extract_archive(&archive_path, &extract_dir, password)
            .await
            .unwrap();

        let detailed_report = manager
            .validate_repository_format_detailed(&extract_dir)
            .unwrap();

        assert!(
            detailed_report.is_valid,
            "Extracted archive should be valid"
        );
        assert!(
            detailed_report.version.is_some(),
            "Version should be detected"
        );

        println!(
            "Repository validation test passed - format v{} detected",
            detailed_report
                .version
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }
}
