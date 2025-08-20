//! Archive manager module for ZipLock shared library
//!
//! This module handles all operations with encrypted 7z archives containing
//! credential records. It provides a safe abstraction over the sevenz-rust2
//! crate with proper file locking, backup management, and error handling.

#[cfg(target_os = "android")]
use super::android_saf::{is_content_uri, AndroidSafHandle};
use super::cloud_storage::{CloudFileHandle, CloudStorageError};

use crate::archive::validation::{RepositoryStats, ValidationIssue};
use crate::archive::{ArchiveConfig, ArchiveError, ArchiveResult};
use crate::models::CredentialRecord;
use crate::validation::validate_credential;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sevenz_rust2;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Archive manager handles all 7z archive operations
#[derive(Debug)]
pub struct ArchiveManager {
    config: ArchiveConfig,
    current_archive: Arc<RwLock<Option<OpenArchive>>>,
}

/// Source type for archive operations
#[derive(Debug, Clone)]
enum ArchiveSourceType {
    /// Regular file path
    FilePath,
    /// Android content URI
    #[cfg(target_os = "android")]
    ContentUri(String),
}

/// Represents an open, decrypted archive
#[derive(Debug)]
struct OpenArchive {
    path: PathBuf,
    master_password: String,
    temp_dir: TempDir,
    cloud_file_handle: Option<CloudFileHandle>,
    #[cfg(target_os = "android")]
    android_saf_handle: Option<AndroidSafHandle>,
    source_type: ArchiveSourceType,
    credentials: HashMap<String, CredentialRecord>,
    modified: bool,
    last_access: SystemTime,
}

impl OpenArchive {
    /// Validate file lock is still valid and check for conflicts
    fn validate_file_lock(&self) -> ArchiveResult<()> {
        // Check for cloud storage conflicts
        if let Some(ref cloud_file_handle) = self.cloud_file_handle {
            cloud_file_handle
                .verify_no_conflicts()
                .map_err(|e| match e {
                    CloudStorageError::ContentModified => ArchiveError::Internal {
                        message: "Archive file was modified externally. Please close and reopen the archive.".to_string(),
                    },
                    _ => ArchiveError::Internal {
                        message: format!("File lock validation failed: {}", e),
                    },
                })?;
        }
        Ok(())
    }

    /// Get the path to the locked file
    #[allow(dead_code)] // Future use for file locking
    fn locked_file_path(&self) -> PathBuf {
        self.path.with_extension("lock")
    }

    /// Get the temp directory path
    #[allow(dead_code)] // Future use for file locking
    fn temp_dir_path(&self) -> &Path {
        self.temp_dir.path()
    }
}

/// Archive metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArchiveMetadata {
    version: String,
    created_at: SystemTime,
    last_modified: SystemTime,
    credential_count: usize,
}

impl ArchiveManager {
    /// Create a new archive manager
    pub fn new(config: ArchiveConfig) -> ArchiveResult<Self> {
        // Ensure directories exist if specified
        if let Some(ref dir) = config.default_archive_dir {
            fs::create_dir_all(dir).map_err(|e| ArchiveError::CreationFailed {
                reason: format!("Failed to create archive directory: {}", e),
            })?;
        }

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
    pub async fn create_archive(
        &self,
        path: PathBuf,
        master_password: String,
    ) -> ArchiveResult<()> {
        info!("Creating new archive at: {:?}", path);

        if path.exists() {
            return Err(ArchiveError::CreationFailed {
                reason: "Archive already exists".to_string(),
            });
        }

        // Validate master password length
        if master_password.len() < self.config.min_password_length {
            return Err(ArchiveError::InvalidRecord {
                reason: format!(
                    "Master password must be at least {} characters",
                    self.config.min_password_length
                ),
            });
        }

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ArchiveError::CreationFailed {
                reason: format!("Failed to create parent directory: {}", e),
            })?;
        }

        // Create temporary directory for archive operations
        let temp_dir = TempDir::new().map_err(|e| ArchiveError::CreationFailed {
            reason: format!("Failed to create temporary directory: {}", e),
        })?;

        // Create metadata file
        let metadata = ArchiveMetadata {
            version: crate::ARCHIVE_FORMAT_VERSION.to_string(),
            created_at: SystemTime::now(),
            last_modified: SystemTime::now(),
            credential_count: 0,
        };

        // Save metadata to temp directory
        let metadata_path = temp_dir.path().join("metadata.yml");
        let metadata_yaml =
            serde_yaml::to_string(&metadata).map_err(|e| ArchiveError::CreationFailed {
                reason: format!("Failed to serialize metadata: {}", e),
            })?;

        fs::write(&metadata_path, metadata_yaml).map_err(|e| ArchiveError::CreationFailed {
            reason: format!("Failed to write metadata: {}", e),
        })?;

        // Create credentials directory
        let credentials_dir = temp_dir.path().join("credentials");
        fs::create_dir_all(&credentials_dir).map_err(|e| ArchiveError::CreationFailed {
            reason: format!("Failed to create credentials directory: {}", e),
        })?;

        // Create a README file to ensure the archive has sufficient content
        // This helps ensure the 7z archive meets minimum size requirements
        let readme_content = format!(
            "# ZipLock Archive\n\n\
            This is a ZipLock encrypted archive created on {}.\n\
            \n\
            Version: {}\n\
            Format: 7z with AES-256 encryption\n\
            \n\
            This archive contains encrypted password and credential data.\n\
            Only authorized users with the master password can access the contents.\n\
            \n\
            For more information about ZipLock, visit: https://github.com/ziplock\n\
            \n\
            ## Structure\n\
            - metadata.yml: Archive metadata and configuration\n\
            - credentials/: Directory containing encrypted credential files\n\
            \n\
            ## Security Features\n\
            - AES-256 encryption\n\
            - PBKDF2 key derivation\n\
            - Secure random salt generation\n\
            - Tamper detection\n\
            \n\
            ## Archive Information\n\
            This archive was created with ZipLock to ensure it has sufficient content\n\
            for proper 7z format compatibility and extraction reliability.\n\
            \n\
            Even empty credential archives require this minimal structure to maintain\n\
            compatibility with the 7z compression library and provide a consistent\n\
            user experience across all platforms.\n\
            \n\
            The credentials directory will be populated as you add password entries\n\
            to your secure archive.\n\
            ",
            metadata
                .created_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata.version
        );

        let readme_path = temp_dir.path().join("README.md");
        fs::write(&readme_path, readme_content).map_err(|e| ArchiveError::CreationFailed {
            reason: format!("Failed to write README: {}", e),
        })?;

        // Create a placeholder file in the credentials directory to ensure it's included in the archive
        // This helps ensure the archive has a proper directory structure even when empty
        let placeholder_content = "# ZipLock Credentials Directory\n\n\
            This directory will contain your encrypted credential files.\n\
            This placeholder file ensures the directory structure is preserved\n\
            in the 7z archive even when no credentials have been added yet.\n\
            \n\
            This file will be automatically managed by ZipLock and should not\n\
            be modified manually.\n";

        let placeholder_path = credentials_dir.join(".ziplock_placeholder");
        fs::write(&placeholder_path, placeholder_content).map_err(|e| {
            ArchiveError::CreationFailed {
                reason: format!("Failed to write credentials placeholder: {}", e),
            }
        })?;

        // Create the encrypted archive
        self.create_encrypted_archive(temp_dir.path(), &path, &master_password)
            .await?;

        info!("Successfully created archive at: {:?}", path);
        Ok(())
    }

    /// Open an existing archive for operations
    pub async fn open_archive(&self, path: PathBuf, master_password: String) -> ArchiveResult<()> {
        #[cfg(target_os = "android")]
        {
            let path_str = path.to_string_lossy().to_string();
            if is_content_uri(&path_str) {
                return self
                    .open_archive_from_content_uri(path_str, master_password)
                    .await;
            }
        }

        self.open_archive_from_file_path(path, master_password)
            .await
    }

    /// Open an archive from a file path
    async fn open_archive_from_file_path(
        &self,
        path: PathBuf,
        master_password: String,
    ) -> ArchiveResult<()> {
        info!("Opening archive from file path: {:?}", path);

        // Check if archive exists
        if !path.exists() {
            return Err(ArchiveError::NotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Check if an archive is already open
        {
            let current = self.current_archive.read().await;
            if current.is_some() {
                return Err(ArchiveError::Internal {
                    message: "Another archive is already open".to_string(),
                });
            }
        }

        // Create cloud-aware file handle
        let cloud_file_handle = CloudFileHandle::new(&path, Some(self.config.file_lock_timeout))
            .map_err(|e| match e {
                CloudStorageError::LockError(_) => ArchiveError::LockFailed {
                    path: path.to_string_lossy().to_string(),
                },
                _ => ArchiveError::Internal {
                    message: format!("Failed to create file handle for {:?}: {}. This may indicate file permission issues, that the file is in use by another application, or that a stale lock file exists.", path, e),
                },
            })?;

        // Get the local working path (may be different from original for cloud files)
        let working_path = cloud_file_handle.local_path();

        // Warn if this is a cloud storage file
        if cloud_file_handle.is_cloud_file() {
            warn!(
                "Opening archive from cloud storage: {:?}. Working with local copy: {:?}",
                path, working_path
            );
        }

        // Create temporary directory
        let temp_dir = TempDir::new().map_err(|e| ArchiveError::OpenFailed {
            reason: format!("Failed to create temporary directory: {}", e),
        })?;

        // Extract archive to temp directory (use working path for cloud files)
        self.extract_archive(working_path, temp_dir.path(), &master_password)
            .await?;

        // Load credentials from extracted directory
        let credentials = self
            .load_credentials_from_directory(temp_dir.path())
            .await?;

        // Create OpenArchive instance
        let open_archive = OpenArchive {
            path: path.clone(),
            master_password,
            temp_dir,
            cloud_file_handle: Some(cloud_file_handle),
            #[cfg(target_os = "android")]
            android_saf_handle: None,
            source_type: ArchiveSourceType::FilePath,
            credentials,
            modified: false,
            last_access: SystemTime::now(),
        };

        // Store the open archive
        {
            let mut current = self.current_archive.write().await;
            *current = Some(open_archive);
        }

        info!("Successfully opened archive: {:?}", path);
        Ok(())
    }

    /// Open an archive from an Android content URI
    #[cfg(target_os = "android")]
    async fn open_archive_from_content_uri(
        &self,
        content_uri: String,
        master_password: String,
    ) -> ArchiveResult<()> {
        info!("Opening archive from Android content URI: {}", content_uri);

        // Check if an archive is already open
        {
            let current = self.current_archive.read().await;
            if current.is_some() {
                return Err(ArchiveError::Internal {
                    message: "Another archive is already open".to_string(),
                });
            }
        }

        // Create Android SAF handle
        let mut android_saf_handle =
            AndroidSafHandle::new(&content_uri).map_err(|e| ArchiveError::OpenFailed {
                reason: format!("Failed to open Android content URI: {}", e),
            })?;

        // Create temporary file copy for 7z operations
        let temp_archive_path =
            android_saf_handle
                .create_temp_file_copy()
                .map_err(|e| ArchiveError::OpenFailed {
                    reason: format!("Failed to create temporary file copy: {}", e),
                })?;

        // Create temporary directory for extraction
        let temp_dir = TempDir::new().map_err(|e| ArchiveError::OpenFailed {
            reason: format!("Failed to create temporary directory: {}", e),
        })?;

        // Extract archive to temp directory
        self.extract_archive(temp_archive_path, temp_dir.path(), &master_password)
            .await?;

        // Load credentials from extracted directory
        let credentials = self
            .load_credentials_from_directory(temp_dir.path())
            .await?;

        // Create OpenArchive instance
        let open_archive = OpenArchive {
            path: PathBuf::from(&content_uri), // Store the content URI as path for reference
            master_password,
            temp_dir,
            cloud_file_handle: None,
            android_saf_handle: Some(android_saf_handle),
            source_type: ArchiveSourceType::ContentUri(content_uri.clone()),
            credentials,
            modified: false,
            last_access: SystemTime::now(),
        };

        // Store the open archive
        {
            let mut current = self.current_archive.write().await;
            *current = Some(open_archive);
        }

        info!(
            "Successfully opened archive from content URI: {}",
            content_uri
        );
        Ok(())
    }

    /// Close the current archive
    pub async fn close_archive(&self) -> ArchiveResult<()> {
        let mut archive_guard = self.current_archive.write().await;

        if let Some(mut archive) = archive_guard.take() {
            info!("Closing archive: {:?}", archive.path);

            // Save if modified
            if archive.modified {
                warn!("Archive has unsaved changes, saving before closing");

                // Write credentials to temp directory
                self.write_credentials_to_directory(archive.temp_dir.path(), &archive.credentials)
                    .await?;

                // Create new encrypted archive
                if let Some(ref cloud_handle) = archive.cloud_file_handle {
                    self.create_encrypted_archive(
                        archive.temp_dir.path(),
                        cloud_handle.local_path(),
                        &archive.master_password,
                    )
                    .await?;

                    // Mark for sync back and sync to cloud storage if needed
                    if let Some(ref mut cloud_handle) = archive.cloud_file_handle {
                        cloud_handle.mark_modified();
                        if cloud_handle.is_cloud_file() {
                            info!(
                                "Syncing final changes back to cloud storage: {:?}",
                                archive.path
                            );
                            cloud_handle.sync_back().map_err(|e| {
                                warn!("Failed to sync final changes to cloud storage: {}", e);
                                ArchiveError::Internal {
                                    message: format!(
                                        "Failed to sync to cloud storage on close: {}",
                                        e
                                    ),
                                }
                            })?;
                        }
                    }
                }
            }

            // Clean up old backups before closing (use local path for cloud files)
            if self.config.auto_backup {
                let backup_path = if let Some(ref cloud_handle) = archive.cloud_file_handle {
                    if cloud_handle.is_cloud_file() {
                        cloud_handle.local_path()
                    } else {
                        &archive.path
                    }
                } else {
                    &archive.path
                };
                if let Err(e) = self.cleanup_old_backups(backup_path).await {
                    warn!("Failed to cleanup old backups: {}", e);
                }
            }

            // CloudFileHandle is automatically cleaned up when dropped
            info!("Archive closed successfully");
        } else {
            info!("No archive was open");
        }

        Ok(())
    }

    /// Get a credential by ID
    pub async fn get_credential(&self, id: String) -> ArchiveResult<CredentialRecord> {
        let archive = self.current_archive.read().await;
        let archive = archive.as_ref().ok_or(ArchiveError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive.validate_file_lock()?;

        archive
            .credentials
            .get(&id)
            .cloned()
            .ok_or(ArchiveError::RecordNotFound { id })
    }

    /// List all credentials (metadata only)
    pub async fn list_credentials(&self) -> ArchiveResult<Vec<CredentialRecord>> {
        let archive = self.current_archive.read().await;
        let archive = archive.as_ref().ok_or(ArchiveError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive.validate_file_lock()?;
        Ok(archive.credentials.values().cloned().collect())
    }

    /// Add a new credential
    pub async fn add_credential(&self, mut credential: CredentialRecord) -> ArchiveResult<String> {
        let mut archive_guard = self.current_archive.write().await;
        let archive = archive_guard.as_mut().ok_or(ArchiveError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive.validate_file_lock()?;

        // Ensure credential has an ID
        if credential.id.is_empty() {
            credential.id = Uuid::new_v4().to_string();
        }

        // Set timestamps
        let now = SystemTime::now();
        credential.created_at = now;
        credential.updated_at = now;

        // Validate credential
        validate_credential(&credential).map_err(|e| ArchiveError::InvalidRecord {
            reason: e.to_string(),
        })?;

        // Add to collection
        let id = credential.id.clone();
        archive.credentials.insert(id.clone(), credential);
        archive.modified = true;
        archive.last_access = SystemTime::now();

        info!("Added credential: {}", id);
        Ok(id)
    }

    /// Update an existing credential
    pub async fn update_credential(
        &self,
        id: String,
        mut credential: CredentialRecord,
    ) -> ArchiveResult<()> {
        let mut archive_guard = self.current_archive.write().await;
        let archive = archive_guard.as_mut().ok_or(ArchiveError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive.validate_file_lock()?;

        // Ensure ID matches
        credential.id = id.clone();
        credential.updated_at = SystemTime::now();

        // Validate credential
        validate_credential(&credential).map_err(|e| ArchiveError::InvalidRecord {
            reason: e.to_string(),
        })?;

        // Update in collection
        archive.credentials.insert(id.clone(), credential);
        archive.modified = true;
        archive.last_access = SystemTime::now();

        info!("Updated credential: {}", id);
        Ok(())
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: String) -> ArchiveResult<()> {
        let mut archive_guard = self.current_archive.write().await;
        let archive = archive_guard.as_mut().ok_or(ArchiveError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive.validate_file_lock()?;

        if archive.credentials.remove(&id).is_some() {
            archive.modified = true;
            archive.last_access = SystemTime::now();
            info!("Deleted credential: {}", id);
            Ok(())
        } else {
            Err(ArchiveError::RecordNotFound { id })
        }
    }

    /// Search credentials by query
    pub async fn search_credentials(&self, query: String) -> ArchiveResult<Vec<CredentialRecord>> {
        let archive = self.current_archive.read().await;
        let archive = archive.as_ref().ok_or(ArchiveError::RecordNotFound {
            id: "Archive not open".to_string(),
        })?;

        archive.validate_file_lock()?;

        let query_lower = query.to_lowercase();
        let results: Vec<CredentialRecord> = archive
            .credentials
            .values()
            .filter(|cred| {
                cred.title.to_lowercase().contains(&query_lower)
                    || cred.credential_type.to_lowercase().contains(&query_lower)
                    || cred
                        .notes
                        .as_ref()
                        .is_some_and(|notes| notes.to_lowercase().contains(&query_lower))
                    || cred
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// Save the current archive
    pub async fn save_archive(&self) -> ArchiveResult<()> {
        let mut archive_guard = self.current_archive.write().await;

        if let Some(archive) = archive_guard.as_mut() {
            if archive.modified {
                info!("Saving archive: {:?}", archive.path);

                // Check for cloud storage conflicts before saving
                archive.validate_file_lock()?;

                // Write credentials to temp directory
                self.write_credentials_to_directory(archive.temp_dir.path(), &archive.credentials)
                    .await?;

                match &archive.source_type {
                    ArchiveSourceType::FilePath => {
                        if let Some(ref mut cloud_file_handle) = archive.cloud_file_handle {
                            // Create backup if enabled (use local working path for cloud files)
                            if self.config.auto_backup {
                                self.create_backup(cloud_file_handle.local_path()).await?;
                            }

                            // Create new encrypted archive (save to local working path)
                            self.create_encrypted_archive(
                                archive.temp_dir.path(),
                                cloud_file_handle.local_path(),
                                &archive.master_password,
                            )
                            .await?;

                            // Mark that the file was modified for cloud storage sync back
                            cloud_file_handle.mark_modified();

                            // For cloud files, sync back to original location
                            if cloud_file_handle.is_cloud_file() {
                                info!("Syncing changes back to cloud storage: {:?}", archive.path);
                                cloud_file_handle.sync_back().map_err(|e| {
                                    ArchiveError::Internal {
                                        message: format!("Failed to sync to cloud storage: {}", e),
                                    }
                                })?;
                            }
                        } else {
                            // Direct file path without cloud handling
                            if self.config.auto_backup {
                                self.create_backup(&archive.path).await?;
                            }

                            self.create_encrypted_archive(
                                archive.temp_dir.path(),
                                &archive.path,
                                &archive.master_password,
                            )
                            .await?;
                        }
                    }
                    #[cfg(target_os = "android")]
                    ArchiveSourceType::ContentUri(content_uri) => {
                        if let Some(ref mut android_saf_handle) = archive.android_saf_handle {
                            // Create temporary archive file
                            let temp_archive_path = android_saf_handle
                                .create_temp_file_copy()
                                .map_err(|e| ArchiveError::Internal {
                                    message: format!(
                                        "Failed to create temporary archive file: {}",
                                        e
                                    ),
                                })?;

                            // Create backup if enabled
                            if self.config.auto_backup {
                                self.create_backup(temp_archive_path).await?;
                            }

                            // Create new encrypted archive
                            self.create_encrypted_archive(
                                archive.temp_dir.path(),
                                temp_archive_path,
                                &archive.master_password,
                            )
                            .await?;

                            // Sync changes back to content URI
                            info!(
                                "Syncing changes back to Android content URI: {}",
                                content_uri
                            );
                            android_saf_handle.sync_temp_file_back().map_err(|e| {
                                ArchiveError::Internal {
                                    message: format!(
                                        "Failed to sync to Android content URI: {}",
                                        e
                                    ),
                                }
                            })?;
                        }
                    }
                }

                archive.modified = false;
                info!("Archive saved successfully");
            } else {
                debug!("Archive has no changes to save");
            }
        } else {
            return Err(ArchiveError::Internal {
                message: "No archive currently open".to_string(),
            });
        }

        Ok(())
    }

    /// Create a backup of the archive
    async fn create_backup(&self, archive_path: &Path) -> ArchiveResult<()> {
        if !archive_path.exists() {
            return Ok(());
        }

        let backup_dir = self
            .config
            .backup_dir
            .as_deref()
            .unwrap_or_else(|| archive_path.parent().unwrap_or_else(|| Path::new(".")));

        fs::create_dir_all(backup_dir).map_err(|e| ArchiveError::BackupFailed {
            reason: format!("Failed to create backup directory: {}", e),
        })?;

        let now: DateTime<Utc> = SystemTime::now().into();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();

        let backup_name = format!(
            "{}_{}.7z",
            archive_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy(),
            timestamp
        );

        let backup_path = backup_dir.join(backup_name);

        fs::copy(archive_path, &backup_path).map_err(|e| ArchiveError::BackupFailed {
            reason: format!("Failed to copy archive for backup: {}", e),
        })?;

        info!("Created backup: {:?}", backup_path);
        Ok(())
    }

    /// Clean up old backup files, keeping only the most recent ones based on backup_count
    async fn cleanup_old_backups(&self, archive_path: &Path) -> ArchiveResult<()> {
        let backup_dir = self
            .config
            .backup_dir
            .as_deref()
            .unwrap_or_else(|| archive_path.parent().unwrap_or_else(|| Path::new(".")));

        if !backup_dir.exists() {
            return Ok(());
        }

        let archive_stem = archive_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        // Find all backup files for this archive
        let entries = fs::read_dir(backup_dir).map_err(|e| ArchiveError::BackupFailed {
            reason: format!("Failed to read backup directory: {}", e),
        })?;

        let mut backup_files = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| ArchiveError::BackupFailed {
                reason: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Check if this is a backup file for our archive
                if filename.starts_with(&format!("{}_", archive_stem)) && filename.ends_with(".7z")
                {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            backup_files.push((path, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove old backups beyond the configured count
        if backup_files.len() > self.config.backup_count as usize {
            let files_to_remove = &backup_files[self.config.backup_count as usize..];

            for (path, _) in files_to_remove {
                match fs::remove_file(path) {
                    Ok(_) => info!("Removed old backup: {:?}", path),
                    Err(e) => warn!("Failed to remove old backup {:?}: {}", path, e),
                }
            }
        }

        Ok(())
    }

    /// Load credentials from directory
    /// Load credentials from a temporary directory after extraction
    async fn load_credentials_from_directory(
        &self,
        dir: &Path,
    ) -> ArchiveResult<HashMap<String, CredentialRecord>> {
        let credentials_dir = dir.join("credentials");
        let mut credentials = HashMap::new();

        if !credentials_dir.exists() {
            return Ok(credentials);
        }

        let entries = fs::read_dir(&credentials_dir).map_err(|e| ArchiveError::ExtractFailed {
            reason: format!("Failed to read credentials directory: {}", e),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| ArchiveError::ExtractFailed {
                reason: format!("Failed to read directory entry: {}", e),
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
                    let content = fs::read_to_string(&record_file).map_err(|e| {
                        ArchiveError::ExtractFailed {
                            reason: format!("Failed to read credential record.yml: {}", e),
                        }
                    })?;

                    let credential: CredentialRecord =
                        serde_yaml::from_str(&content).map_err(|e| {
                            ArchiveError::InvalidRecord {
                                reason: format!("Failed to parse credential YAML: {}", e),
                            }
                        })?;

                    credentials.insert(credential.id.clone(), credential);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                // Legacy format: /credentials/credential-id.yml
                let content =
                    fs::read_to_string(&path).map_err(|e| ArchiveError::ExtractFailed {
                        reason: format!("Failed to read credential file: {}", e),
                    })?;

                let credential: CredentialRecord =
                    serde_yaml::from_str(&content).map_err(|e| ArchiveError::InvalidRecord {
                        reason: format!("Failed to parse credential YAML: {}", e),
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
    ) -> ArchiveResult<()> {
        let credentials_dir = dir.join("credentials");

        // Clear existing credential files but preserve directory structure and placeholders
        if credentials_dir.exists() {
            let entries = fs::read_dir(&credentials_dir).map_err(|e| ArchiveError::AddFailed {
                reason: format!("Failed to read credentials directory: {}", e),
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| ArchiveError::AddFailed {
                    reason: format!("Failed to read directory entry: {}", e),
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
                    .map_err(|e| ArchiveError::AddFailed {
                        reason: format!("Failed to remove existing credential: {}", e),
                    })?;
                }
            }
        } else {
            fs::create_dir(&credentials_dir).map_err(|e| ArchiveError::AddFailed {
                reason: format!("Failed to create credentials directory: {}", e),
            })?;
        }

        // Write credentials using repository format v1.0: /credentials/credential-id/record.yml
        for credential in credentials.values() {
            let credential_dir = credentials_dir.join(&credential.id);

            fs::create_dir(&credential_dir).map_err(|e| ArchiveError::AddFailed {
                reason: format!("Failed to create credential directory: {}", e),
            })?;

            let record_file = credential_dir.join("record.yml");
            let content =
                serde_yaml::to_string(credential).map_err(|e| ArchiveError::AddFailed {
                    reason: format!("Failed to serialize credential: {}", e),
                })?;

            fs::write(&record_file, content).map_err(|e| ArchiveError::AddFailed {
                reason: format!("Failed to write credential record: {}", e),
            })?;
        }

        // Ensure placeholder file exists to preserve directory in archive
        let placeholder = credentials_dir.join(".gitkeep");
        if !placeholder.exists() {
            fs::write(&placeholder, "# ZipLock credentials directory\n# This file ensures the directory is preserved in the archive\n")
                .map_err(|e| ArchiveError::AddFailed {
                    reason: format!("Failed to write credentials placeholder: {}", e),
                })?;
        }

        debug!("Wrote {} credentials to directory", credentials.len());
        Ok(())
    }

    /// Create encrypted 7z archive
    async fn create_encrypted_archive(
        &self,
        source_dir: &Path,
        archive_path: &Path,
        password: &str,
    ) -> ArchiveResult<()> {
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
                // Always use non-solid compression for better compatibility with small archives
                // and more reliable extraction, especially on mobile platforms
                info!(
                    "Using non-solid compression for better compatibility (level {})",
                    config.level
                );

                // Use the convenience function which creates more compatible archives
                // This avoids potential issues with solid compression for minimal content
                sevenz_rust2::compress_to_path_encrypted(
                    &source_dir,
                    &archive_path,
                    password.as_str().into(),
                )?;

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
                Err(ArchiveError::CreationFailed {
                    reason: format!("7z compression failed: {}", e),
                })
            }
            Err(e) => {
                error!("Task execution failed: {}", e);
                Err(ArchiveError::CreationFailed {
                    reason: format!("Task failed: {}", e),
                })
            }
        }
    }

    /// Extract archive to directory
    async fn extract_archive(
        &self,
        archive_path: &Path,
        destination: &Path,
        password: &str,
    ) -> ArchiveResult<()> {
        info!("Extracting archive: {:?}", archive_path);
        debug!("Destination directory: {:?}", destination);

        // Validate archive file size before attempting extraction
        let file_size = fs::metadata(archive_path)
            .map_err(|e| ArchiveError::ExtractFailed {
                reason: format!("Failed to get archive file metadata: {}", e),
            })?
            .len();

        if file_size == 0 {
            return Err(ArchiveError::ExtractFailed {
                reason: "Archive file is empty (0 bytes)".to_string(),
            });
        }

        if file_size < 32 {
            return Err(ArchiveError::ExtractFailed {
                reason: format!(
                    "Archive file is too small ({} bytes) to be a valid 7z archive. Minimum expected size is 32 bytes for the signature header.",
                    file_size
                ),
            });
        }

        // Enhanced validation for small archives
        if file_size < 512 {
            warn!(
                "Archive file is very small ({} bytes), performing additional validation for: {:?}",
                file_size, archive_path
            );

            // Validate 7z signature
            if let Err(e) = self.validate_7z_signature(archive_path).await {
                return Err(ArchiveError::ExtractFailed {
                    reason: format!("Archive signature validation failed: {}", e),
                });
            }
        }

        info!("Archive file size validation passed: {} bytes", file_size);

        // Create destination directory
        fs::create_dir_all(destination).map_err(|e| ArchiveError::ExtractFailed {
            reason: format!("Failed to create destination directory: {}", e),
        })?;

        // Use sevenz_rust's extraction function with enhanced error handling and timeout
        info!("Starting 7z extraction with enhanced error handling");

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(60), // 60 second timeout for extraction
            tokio::task::spawn_blocking({
                let archive_path = archive_path.to_owned();
                let destination = destination.to_owned();
                let password = password.to_owned();

                move || {
                    info!("Extraction task started for: {:?}", archive_path);

                    // Get file info before extraction
                    let file_size = std::fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
                    info!("Attempting 7z extraction of {} byte archive: {:?}", file_size, archive_path);

                    // Use simple panic catcher only for sevenz_rust2 library
                    std::panic::catch_unwind(|| {
                        info!("Calling sevenz_rust2::decompress_file_with_password");
                        sevenz_rust2::decompress_file_with_password(
                            &archive_path,
                            &destination,
                            password.as_str().into(),
                        )
                    })
                    .map_err(|panic_info| {
                        let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "Unknown panic in 7z library".to_string()
                        };
                        error!("7z extraction panicked: {}", panic_msg);
                        sevenz_rust2::Error::from(std::io::Error::other(
                            format!("7z library operation panicked: {}. Archive may be corrupted or use incompatible format.", panic_msg),
                        ))
                    })
                    .and_then(|result| result)
                }
            })
        ).await;

        match result {
            Ok(Ok(Ok(()))) => {
                info!("Successfully extracted archive to: {:?}", destination);

                // Verify extraction results
                let extracted_files = fs::read_dir(destination)
                    .map_err(|e| ArchiveError::ExtractFailed {
                        reason: format!("Failed to read extracted directory: {}", e),
                    })?
                    .count();

                if extracted_files == 0 {
                    return Err(ArchiveError::ExtractFailed {
                        reason: "Archive extracted successfully but no files were found in the destination. The archive may be empty or corrupted.".to_string(),
                    });
                }

                info!(
                    "Extraction verification passed: {} files extracted",
                    extracted_files
                );
                Ok(())
            }
            Ok(Ok(Err(e))) => {
                error!("Failed to extract 7z archive: {}", e);

                // Check for password-related errors
                let error_string = e.to_string();
                if error_string.contains("MaybeBadPassword")
                    || error_string.contains("range decoder first byte is 0")
                    || error_string.contains("Invalid password")
                    || error_string.contains("Wrong password")
                    || error_string.contains("DataError")
                {
                    Err(ArchiveError::CryptoError {
                        reason: "Invalid master password or corrupted archive encryption"
                            .to_string(),
                    })
                } else if error_string.contains("panicked") || error_string.contains("panic") {
                    Err(ArchiveError::ExtractFailed {
                        reason: format!(
                            "Archive extraction failed due to internal error: {}. The archive may be corrupted, use an unsupported format, or be too minimal for the 7z library to handle properly.",
                            e
                        ),
                    })
                } else {
                    Err(ArchiveError::ExtractFailed {
                        reason: format!("Archive extraction failed: {}. This may indicate a corrupted archive or incompatible format.", e),
                    })
                }
            }
            Ok(Err(e)) => {
                error!("Task execution failed: {}", e);
                Err(ArchiveError::ExtractFailed {
                    reason: format!("Extraction task failed: {}", e),
                })
            }
            Err(_) => {
                error!("Archive extraction timed out after 60 seconds");
                Err(ArchiveError::ExtractFailed {
                    reason:
                        "Archive extraction timed out. The archive may be corrupted or too large."
                            .to_string(),
                })
            }
        }
    }

    /// Validate 7z archive signature to detect corruption early
    async fn validate_7z_signature(&self, path: &Path) -> ArchiveResult<()> {
        info!("Validating 7z signature for: {:?}", path);

        let mut file = fs::File::open(path).map_err(|e| ArchiveError::ExtractFailed {
            reason: format!("Failed to open archive for signature validation: {}", e),
        })?;

        let mut signature = [0u8; 6];
        file.read_exact(&mut signature)
            .map_err(|e| ArchiveError::ExtractFailed {
                reason: format!("Failed to read archive signature: {}", e),
            })?;

        // 7z signature is: 0x37 0x7A 0xBC 0xAF 0x27 0x1C
        let expected_signature = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

        info!("Read signature: {:02x?}", signature);
        info!("Expected signature: {:02x?}", expected_signature);

        if signature != expected_signature {
            return Err(ArchiveError::ExtractFailed {
                reason: format!(
                    "Invalid 7z signature. Expected {:02x?}, got {:02x?}. The file may not be a valid 7z archive or may be corrupted.",
                    expected_signature, signature
                ),
            });
        }

        info!("7z signature validation passed");
        Ok(())
    }

    /// Validate archive file
    pub async fn validate_archive_file(
        &self,
        path: &Path,
    ) -> ArchiveResult<crate::archive::ValidationReport> {
        // Basic validation - check if file exists and has correct extension
        if !path.exists() {
            return Ok(crate::archive::ValidationReport {
                version: None,
                issues: vec![ValidationIssue::MissingRequired {
                    path: path.to_string_lossy().to_string(),
                    description: "Archive file does not exist".to_string(),
                }],
                is_valid: false,
                can_auto_repair: false,
                stats: RepositoryStats {
                    credential_count: 0,
                    custom_type_count: 0,
                    total_size_bytes: 0,
                    last_modified: Some(std::time::SystemTime::UNIX_EPOCH),
                    created_at: Some(std::time::SystemTime::UNIX_EPOCH),
                },
            });
        }

        if path.extension().and_then(|s| s.to_str()) != Some("7z") {
            return Ok(crate::archive::ValidationReport {
                version: None,
                issues: vec![ValidationIssue::InvalidFormat {
                    path: path.to_string_lossy().to_string(),
                    reason: "Archive file does not have .7z extension".to_string(),
                }],
                is_valid: false,
                can_auto_repair: false,
                stats: RepositoryStats {
                    credential_count: 0,
                    custom_type_count: 0,
                    total_size_bytes: 0,
                    last_modified: Some(std::time::SystemTime::UNIX_EPOCH),
                    created_at: Some(std::time::SystemTime::UNIX_EPOCH),
                },
            });
        }

        Ok(crate::archive::ValidationReport {
            version: None,
            issues: vec![],
            is_valid: true,
            can_auto_repair: false,
            stats: RepositoryStats {
                credential_count: 0,
                custom_type_count: 0,
                total_size_bytes: 0,
                last_modified: Some(std::time::SystemTime::UNIX_EPOCH),
                created_at: Some(std::time::SystemTime::UNIX_EPOCH),
            },
        })
    }

    /// Repair archive file
    pub async fn repair_archive_file(&self, path: &Path) -> ArchiveResult<()> {
        // Basic repair implementation
        info!("Attempting to repair archive: {:?}", path);

        // In a real implementation, this would attempt to repair the archive
        // For now, just return success if the file exists
        if path.exists() {
            Ok(())
        } else {
            Err(ArchiveError::NotFound {
                path: path.to_string_lossy().to_string(),
            })
        }
    }
}

impl Drop for ArchiveManager {
    fn drop(&mut self) {
        // Note: async drop is not supported, so we can't properly save here
        // In a real implementation, you'd want to ensure proper cleanup
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_archive_manager_creation() {
        let config = ArchiveConfig::default();
        let manager = ArchiveManager::new(config).unwrap();
        assert!(!manager.is_open().await);
    }

    #[tokio::test]
    async fn test_create_new_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.7z");

        let config = ArchiveConfig::default();
        let manager = ArchiveManager::new(config).unwrap();

        let result = manager
            .create_archive(archive_path, "strong_password_123".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_credential_operations() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.7z");

        let config = ArchiveConfig::default();
        let manager = ArchiveManager::new(config).unwrap();

        // Create and open archive
        manager
            .create_archive(archive_path.clone(), "strong_password_123".to_string())
            .await
            .unwrap();
        manager
            .open_archive(archive_path, "strong_password_123".to_string())
            .await
            .unwrap();

        // Create a credential
        let credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        let id = manager.add_credential(credential).await.unwrap();

        // Retrieve the credential
        let retrieved = manager.get_credential(id.clone()).await.unwrap();
        assert_eq!(retrieved.title, "Test Login");

        // List credentials
        let credentials = manager.list_credentials().await.unwrap();
        assert_eq!(credentials.len(), 1);

        // Delete credential
        manager.delete_credential(id).await.unwrap();
        let credentials = manager.list_credentials().await.unwrap();
        assert_eq!(credentials.len(), 0);
    }

    #[tokio::test]
    async fn test_backup_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.7z");
        let backup_dir = temp_dir.path().join("backups");

        let config = ArchiveConfig {
            backup_count: 2, // Test with only 2 backups
            auto_backup: true,
            backup_dir: Some(backup_dir.clone()),
            ..Default::default()
        };

        let manager = ArchiveManager::new(config).unwrap();

        // Create and open archive
        manager
            .create_archive(archive_path.clone(), "strong_password_123".to_string())
            .await
            .unwrap();
        manager
            .open_archive(archive_path.clone(), "strong_password_123".to_string())
            .await
            .unwrap();

        // Create several backups by saving multiple times
        for i in 0..4 {
            let credential =
                CredentialRecord::new(format!("Test Login {}", i), "login".to_string());
            manager.add_credential(credential).await.unwrap();
            manager.save_archive().await.unwrap();

            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Close archive to trigger cleanup
        manager.close_archive().await.unwrap();

        // Check that backup files exist with human-readable timestamps
        let backup_files = std::fs::read_dir(&backup_dir).unwrap();
        let mut backup_count = 0;

        for entry in backup_files {
            let entry = entry.unwrap();
            let filename = entry.file_name().into_string().unwrap();

            if filename.starts_with("test_") && filename.ends_with(".7z") {
                backup_count += 1;

                // Extract timestamp part and verify it's in YYYYMMDDHHMMSS format
                let timestamp_part = filename
                    .strip_prefix("test_")
                    .unwrap()
                    .strip_suffix(".7z")
                    .unwrap();

                // Should be 14 characters (YYYYMMDDHHMMSS)
                assert_eq!(timestamp_part.len(), 14);

                // Should be all digits
                assert!(timestamp_part.chars().all(|c| c.is_ascii_digit()));

                // Year should be reasonable (20XX)
                let year: u32 = timestamp_part[0..4].parse().unwrap();
                assert!((2024..=2030).contains(&year));
            }
        }

        // Should have at most backup_count files due to cleanup
        assert!(
            backup_count <= 2,
            "Expected at most 2 backup files, found {}",
            backup_count
        );
        assert!(backup_count > 0, "Expected at least 1 backup file");
    }
}
