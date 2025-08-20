//! Linux Platform File Operations Handler
//!
//! This module provides Linux-specific file operations for handling 7z archives
//! when the adaptive hybrid FFI detects runtime conflicts and requires external
//! file operations to prevent nested runtime panics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur during Linux file operations
#[derive(Error, Debug)]
pub enum LinuxFileOperationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("7z command failed: {message}")]
    SevenZipError { message: String },

    #[error("Invalid file operation: {message}")]
    InvalidOperation { message: String },

    #[error("Archive operation failed: {message}")]
    ArchiveError { message: String },

    #[error("Path error: {message}")]
    PathError { message: String },
}

/// Result type for Linux file operations
pub type LinuxFileOperationResult<T> = Result<T, LinuxFileOperationError>;

/// Represents a file operation instruction from the hybrid FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationInstruction {
    #[serde(rename = "type")]
    pub operation_type: String,
    pub path: String,
    pub password: Option<String>,
    pub format: Option<String>,
    pub content: Option<String>,
    pub target_directory: Option<String>,
    pub files: Option<HashMap<String, String>>,
}

/// Container for multiple file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperations {
    pub operations: Vec<FileOperationInstruction>,
}

/// Linux-specific file operations handler
pub struct LinuxFileOperationsHandler {
    temp_dir: Option<PathBuf>,
}

impl LinuxFileOperationsHandler {
    /// Create a new Linux file operations handler
    pub fn new() -> Self {
        Self { temp_dir: None }
    }

    /// Execute file operations from JSON instructions
    pub async fn execute_file_operations(
        &mut self,
        operations_json: &str,
    ) -> LinuxFileOperationResult<()> {
        debug!("Executing file operations: {}", operations_json);

        let operations: FileOperations = serde_json::from_str(operations_json)?;

        for operation in operations.operations {
            match operation.operation_type.as_str() {
                "create_archive" => {
                    self.create_archive_operation(&operation).await?;
                }
                "extract_archive" => {
                    self.extract_archive_operation(&operation).await?;
                }
                "update_archive" => {
                    self.update_archive_operation(&operation).await?;
                }
                "create_file" => {
                    self.create_file_operation(&operation).await?;
                }
                "create_directory" => {
                    self.create_directory_operation(&operation).await?;
                }
                "delete_file" => {
                    self.delete_file_operation(&operation).await?;
                }
                _ => {
                    warn!("Unknown operation type: {}", operation.operation_type);
                    return Err(LinuxFileOperationError::InvalidOperation {
                        message: format!("Unknown operation: {}", operation.operation_type),
                    });
                }
            }
        }

        Ok(())
    }

    /// Create a 7z archive
    async fn create_archive_operation(
        &mut self,
        operation: &FileOperationInstruction,
    ) -> LinuxFileOperationResult<()> {
        let archive_path = Path::new(&operation.path);
        let password = operation.password.as_ref().ok_or_else(|| {
            LinuxFileOperationError::InvalidOperation {
                message: "Password required for archive creation".to_string(),
            }
        })?;

        info!("Creating 7z archive at: {}", archive_path.display());

        // Create parent directory if it doesn't exist
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create temporary directory for archive content
        let temp_dir = self.get_or_create_temp_dir()?;

        // If files are specified, create them in temp directory
        if let Some(files) = &operation.files {
            for (file_path, content) in files {
                let full_path = temp_dir.join(file_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&full_path, content)?;
                debug!("Created file: {}", full_path.display());
            }
        } else {
            // Create basic repository structure
            let repo_info_path = temp_dir.join("repository.info");
            let repo_info = r#"version: "1.0"
format: "ziplock"
created: "1970-01-01T00:00:00Z"
"#;
            fs::write(&repo_info_path, repo_info)?;
            debug!("Created basic repository.info");
        }

        // Use 7z command to create archive
        let mut cmd = Command::new("7z");
        cmd.arg("a")
            .arg("-t7z")
            .arg(format!("-p{}", password))
            .arg("-mhe=on") // Encrypt headers
            .arg("-mx=9") // Maximum compression
            .arg(archive_path)
            .arg(format!("{}/*", temp_dir.display()));

        debug!("Running 7z command: {:?}", cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("7z command failed: {}", error_msg);
            return Err(LinuxFileOperationError::SevenZipError {
                message: format!("Failed to create archive: {}", error_msg),
            });
        }

        info!(
            "Successfully created 7z archive: {}",
            archive_path.display()
        );
        Ok(())
    }

    /// Extract a 7z archive
    async fn extract_archive_operation(
        &mut self,
        operation: &FileOperationInstruction,
    ) -> LinuxFileOperationResult<()> {
        let archive_path = Path::new(&operation.path);
        let password = operation.password.as_ref().ok_or_else(|| {
            LinuxFileOperationError::InvalidOperation {
                message: "Password required for archive extraction".to_string(),
            }
        })?;

        info!("Extracting 7z archive from: {}", archive_path.display());

        // Create temporary directory for extraction
        let temp_dir = self.get_or_create_temp_dir()?;

        // Use 7z command to extract archive
        let mut cmd = Command::new("7z");
        cmd.arg("x")
            .arg(format!("-p{}", password))
            .arg("-y") // Yes to all prompts
            .arg(format!("-o{}", temp_dir.display()))
            .arg(archive_path);

        debug!("Running 7z extract command: {:?}", cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("7z extract command failed: {}", error_msg);
            return Err(LinuxFileOperationError::SevenZipError {
                message: format!("Failed to extract archive: {}", error_msg),
            });
        }

        info!("Successfully extracted archive to: {}", temp_dir.display());
        Ok(())
    }

    /// Update an existing archive
    async fn update_archive_operation(
        &mut self,
        operation: &FileOperationInstruction,
    ) -> LinuxFileOperationResult<()> {
        // For update operations, we extract, modify, and recreate
        self.extract_archive_operation(operation).await?;

        // Apply any file modifications from the operation
        if let Some(files) = &operation.files {
            let temp_dir = self.temp_dir.as_ref().ok_or_else(|| {
                LinuxFileOperationError::InvalidOperation {
                    message: "No temp directory available for update".to_string(),
                }
            })?;

            for (file_path, content) in files {
                let full_path = temp_dir.join(file_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&full_path, content)?;
                debug!("Updated file: {}", full_path.display());
            }
        }

        // Recreate the archive
        self.create_archive_operation(operation).await?;

        Ok(())
    }

    /// Create a file in the temporary directory
    async fn create_file_operation(
        &mut self,
        operation: &FileOperationInstruction,
    ) -> LinuxFileOperationResult<()> {
        let temp_dir = self.get_or_create_temp_dir()?;
        let file_path = temp_dir.join(&operation.path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = operation.content.as_deref().unwrap_or("");
        fs::write(&file_path, content)?;

        debug!("Created file: {}", file_path.display());
        Ok(())
    }

    /// Create a directory in the temporary directory
    async fn create_directory_operation(
        &mut self,
        operation: &FileOperationInstruction,
    ) -> LinuxFileOperationResult<()> {
        let temp_dir = self.get_or_create_temp_dir()?;
        let dir_path = temp_dir.join(&operation.path);

        fs::create_dir_all(&dir_path)?;

        debug!("Created directory: {}", dir_path.display());
        Ok(())
    }

    /// Delete a file from the temporary directory
    async fn delete_file_operation(
        &mut self,
        operation: &FileOperationInstruction,
    ) -> LinuxFileOperationResult<()> {
        let temp_dir =
            self.temp_dir
                .as_ref()
                .ok_or_else(|| LinuxFileOperationError::InvalidOperation {
                    message: "No temp directory available for delete".to_string(),
                })?;

        let file_path = temp_dir.join(&operation.path);

        if file_path.exists() {
            if file_path.is_dir() {
                fs::remove_dir_all(&file_path)?;
            } else {
                fs::remove_file(&file_path)?;
            }
            debug!("Deleted: {}", file_path.display());
        }

        Ok(())
    }

    /// Get or create temporary directory for file operations
    fn get_or_create_temp_dir(&mut self) -> LinuxFileOperationResult<&Path> {
        if self.temp_dir.is_none() {
            let temp_dir = std::env::temp_dir().join(format!("ziplock_{}", std::process::id()));
            fs::create_dir_all(&temp_dir)?;
            self.temp_dir = Some(temp_dir);
            debug!("Created temp directory: {:?}", self.temp_dir);
        }

        Ok(self.temp_dir.as_ref().unwrap())
    }

    /// Get the current temporary directory path (if any)
    pub fn get_temp_dir(&self) -> Option<&Path> {
        self.temp_dir.as_deref()
    }

    /// Check if 7z command is available on the system
    pub fn check_7z_availability() -> bool {
        match Command::new("7z").arg("--help").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Get extracted files as a map (for sharing with hybrid FFI)
    pub fn get_extracted_files(&self) -> LinuxFileOperationResult<HashMap<String, Vec<u8>>> {
        let temp_dir =
            self.temp_dir
                .as_ref()
                .ok_or_else(|| LinuxFileOperationError::InvalidOperation {
                    message: "No temp directory available".to_string(),
                })?;

        let mut files = HashMap::new();

        fn collect_files(
            dir: &Path,
            base_dir: &Path,
            files: &mut HashMap<String, Vec<u8>>,
        ) -> LinuxFileOperationResult<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let relative_path = path.strip_prefix(base_dir).map_err(|e| {
                        LinuxFileOperationError::PathError {
                            message: format!("Failed to get relative path: {}", e),
                        }
                    })?;

                    let content = fs::read(&path)?;
                    files.insert(relative_path.to_string_lossy().to_string(), content);
                } else if path.is_dir() {
                    collect_files(&path, base_dir, files)?;
                }
            }
            Ok(())
        }

        collect_files(temp_dir, temp_dir, &mut files)?;
        Ok(files)
    }
}

impl Drop for LinuxFileOperationsHandler {
    fn drop(&mut self) {
        if let Some(temp_dir) = &self.temp_dir {
            if temp_dir.exists() {
                if let Err(e) = fs::remove_dir_all(temp_dir) {
                    warn!("Failed to cleanup temp directory: {}", e);
                } else {
                    debug!("Cleaned up temp directory: {}", temp_dir.display());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_operations_handler_creation() {
        let handler = LinuxFileOperationsHandler::new();
        assert!(handler.temp_dir.is_none());
    }

    #[tokio::test]
    async fn test_7z_availability_check() {
        // This test will pass if 7z is installed, otherwise it will show availability status
        let available = LinuxFileOperationsHandler::check_7z_availability();
        println!("7z available: {}", available);
    }

    #[tokio::test]
    async fn test_file_operations_json_parsing() {
        let json = r#"{
            "operations": [
                {
                    "type": "create_archive",
                    "path": "/tmp/test.7z",
                    "password": "test123",
                    "format": "7z"
                }
            ]
        }"#;

        let operations: FileOperations = serde_json::from_str(json).unwrap();
        assert_eq!(operations.operations.len(), 1);
        assert_eq!(operations.operations[0].operation_type, "create_archive");
    }

    #[tokio::test]
    async fn test_temp_directory_creation() {
        let mut handler = LinuxFileOperationsHandler::new();
        let temp_dir = handler.get_or_create_temp_dir().unwrap();
        assert!(temp_dir.exists());
    }
}
