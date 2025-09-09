//! Core error types for the ZipLock unified architecture.
//!
//! This module defines the error types used throughout the shared library,
//! providing clear separation between memory operations and file operations.

use std::fmt;

/// Core errors for memory repository operations
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// Repository has not been initialized
    NotInitialized,

    /// Repository is already initialized
    AlreadyInitialized,

    /// Credential with the given ID was not found
    CredentialNotFound { id: String },

    /// Data validation failed
    ValidationError { message: String },

    /// Serialization/deserialization failed
    SerializationError { message: String },

    /// Invalid credential data
    InvalidCredential { message: String },

    /// Repository structure error
    StructureError { message: String },

    /// Internal error (unexpected conditions)
    InternalError { message: String },

    /// File operation error (wrapped)
    FileOperation(FileError),
}

/// File operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum FileError {
    /// File or archive not found
    NotFound { path: String },

    /// Permission denied accessing file
    PermissionDenied { path: String },

    /// Failed to extract archive
    ExtractionFailed { message: String },

    /// Failed to create archive
    CreationFailed { message: String },

    /// Invalid or incorrect password
    InvalidPassword,

    /// Archive is corrupted or invalid
    CorruptedArchive { message: String },

    /// General I/O error
    IoError { message: String },
}

/// Result type for core operations
pub type CoreResult<T> = Result<T, CoreError>;

/// Result type for file operations
pub type FileResult<T> = Result<T, FileError>;

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::NotInitialized => write!(f, "Repository not initialized"),
            CoreError::AlreadyInitialized => write!(f, "Repository already initialized"),
            CoreError::CredentialNotFound { id } => write!(f, "Credential not found: {id}"),
            CoreError::ValidationError { message } => write!(f, "Validation error: {message}"),
            CoreError::SerializationError { message } => {
                write!(f, "Serialization error: {message}")
            }
            CoreError::InvalidCredential { message } => {
                write!(f, "Invalid credential: {message}")
            }
            CoreError::StructureError { message } => write!(f, "Structure error: {message}"),
            CoreError::InternalError { message } => write!(f, "Internal error: {message}"),
            CoreError::FileOperation(err) => write!(f, "File operation error: {err}"),
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::NotFound { path } => write!(f, "File not found: {path}"),
            FileError::PermissionDenied { path } => write!(f, "Permission denied: {path}"),
            FileError::ExtractionFailed { message } => write!(f, "Extraction failed: {message}"),
            FileError::CreationFailed { message } => write!(f, "Creation failed: {message}"),
            FileError::InvalidPassword => write!(f, "Invalid password"),
            FileError::CorruptedArchive { message } => write!(f, "Corrupted archive: {message}"),
            FileError::IoError { message } => write!(f, "I/O error: {message}"),
        }
    }
}

impl std::error::Error for CoreError {}
impl std::error::Error for FileError {}

impl From<FileError> for CoreError {
    fn from(err: FileError) -> Self {
        CoreError::FileOperation(err)
    }
}

impl From<serde_yaml::Error> for CoreError {
    fn from(err: serde_yaml::Error) -> Self {
        CoreError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::IoError {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let core_err = CoreError::CredentialNotFound {
            id: "test-id".to_string(),
        };
        assert_eq!(core_err.to_string(), "Credential not found: test-id");

        let file_err = FileError::NotFound {
            path: "/test/path".to_string(),
        };
        assert_eq!(file_err.to_string(), "File not found: /test/path");
    }

    #[test]
    fn test_error_conversion() {
        let file_err = FileError::InvalidPassword;
        let core_err: CoreError = file_err.into();

        match core_err {
            CoreError::FileOperation(FileError::InvalidPassword) => (),
            _ => panic!("Unexpected error conversion"),
        }
    }

    #[test]
    fn test_yaml_error_conversion() {
        let yaml_content = "invalid: yaml: content: [";
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let core_err: CoreError = yaml_err.into();

        match core_err {
            CoreError::SerializationError { .. } => (),
            _ => panic!("Unexpected error conversion"),
        }
    }
}
