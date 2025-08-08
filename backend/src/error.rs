//! Error types for the ZipLock backend daemon
//!
//! This module defines comprehensive error types used throughout the backend,
//! providing clear error messages and proper error chaining for debugging.

use std::fmt;
use thiserror::Error;

/// Main error type for the ZipLock backend
#[derive(Error, Debug)]
pub enum BackendError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Cryptographic operation errors
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    /// Storage and archive operation errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// IPC communication errors
    #[error("IPC error: {0}")]
    Ipc(#[from] IpcError),

    /// Authentication and authorization errors
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// File system operation errors
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    /// Internal daemon errors
    #[error("Internal error: {message}")]
    Internal { message: String },

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Anyhow errors (for context and chaining)
    #[error("Operation failed: {0}")]
    Anyhow(#[from] anyhow::Error),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    NotFound { path: String },

    #[error("Invalid configuration: {field} - {reason}")]
    Invalid { field: String, reason: String },

    #[error("Configuration parsing failed: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },

    #[error("Configuration file permissions are too permissive: {path}")]
    InsecurePermissions { path: String },
}

/// Cryptographic operation errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key derivation failed: {reason}")]
    KeyDerivation { reason: String },

    #[error("Encryption failed: {reason}")]
    Encryption { reason: String },

    #[error("Decryption failed: {reason}")]
    Decryption { reason: String },

    #[error("Invalid master key")]
    InvalidMasterKey,

    #[error("Master key not available - database is locked")]
    MasterKeyUnavailable,

    #[error("Key generation failed: {reason}")]
    KeyGeneration { reason: String },

    #[error("Cryptographic initialization failed: {reason}")]
    Initialization { reason: String },

    #[error("Random number generation failed")]
    RandomGeneration,

    #[error("Key derivation parameters are invalid")]
    InvalidKeyParams,

    #[error("Cryptographic operation was interrupted")]
    Interrupted,
}

/// Storage and archive operation errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Archive not found: {path}")]
    ArchiveNotFound { path: String },

    #[error("Archive is corrupted or invalid: {reason}")]
    CorruptedArchive { reason: String },

    #[error("Failed to create archive: {reason}")]
    ArchiveCreation { reason: String },

    #[error("Failed to open archive: {reason}")]
    ArchiveOpen { reason: String },

    #[error("Failed to extract from archive: {reason}")]
    ArchiveExtract { reason: String },

    #[error("Failed to add to archive: {reason}")]
    ArchiveAdd { reason: String },

    #[error("File lock acquisition failed: {path}")]
    FileLock { path: String },

    #[error("File lock timeout: {path}")]
    FileLockTimeout { path: String },

    #[error("Credential record not found: {id}")]
    RecordNotFound { id: String },

    #[error("Record validation failed: {reason}")]
    InvalidRecord { reason: String },

    #[error("Backup creation failed: {reason}")]
    BackupFailed { reason: String },

    #[error("Archive format version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
}

/// IPC communication errors
#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Failed to bind to socket: {path}")]
    SocketBind { path: String },

    #[error("Failed to accept connection: {reason}")]
    ConnectionAccept { reason: String },

    #[error("Client connection lost")]
    ConnectionLost,

    #[error("Invalid request format: {reason}")]
    InvalidRequest { reason: String },

    #[error("Request processing failed: {reason}")]
    RequestProcessing { reason: String },

    #[error("Response serialization failed: {reason}")]
    ResponseSerialization { reason: String },

    #[error("Socket permissions error: {reason}")]
    SocketPermissions { reason: String },

    #[error("IPC timeout: operation took too long")]
    Timeout,

    #[error("Maximum concurrent connections reached")]
    TooManyConnections,
}

/// Authentication and authorization errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication required - database is locked")]
    NotAuthenticated,

    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("Authentication timeout - please re-authenticate")]
    AuthTimeout,

    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("Access denied: insufficient permissions")]
    AccessDenied,

    #[error("Session expired - please re-authenticate")]
    SessionExpired,

    #[error("Too many authentication attempts - temporarily locked")]
    TooManyAttempts,
}

/// Serialization/deserialization errors
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("YAML serialization failed: {reason}")]
    YamlSerialization { reason: String },

    #[error("YAML deserialization failed: {reason}")]
    YamlDeserialization { reason: String },

    #[error("Binary serialization failed: {reason}")]
    BinarySerialization { reason: String },

    #[error("Binary deserialization failed: {reason}")]
    BinaryDeserialization { reason: String },

    #[error("Invalid data format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    #[error("Data schema validation failed: {reason}")]
    SchemaValidation { reason: String },
}

/// Result type alias for backend operations
pub type BackendResult<T> = Result<T, BackendError>;

/// Trait for converting errors to user-friendly messages
pub trait UserFriendlyError {
    /// Convert the error to a message safe to show to users
    /// (without exposing internal implementation details)
    fn user_message(&self) -> String;

    /// Get the error category for metrics/logging
    fn category(&self) -> ErrorCategory;
}

/// Error categories for metrics and logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Configuration,
    Authentication,
    Storage,
    Crypto,
    Network,
    Validation,
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Configuration => write!(f, "configuration"),
            ErrorCategory::Authentication => write!(f, "authentication"),
            ErrorCategory::Storage => write!(f, "storage"),
            ErrorCategory::Crypto => write!(f, "crypto"),
            ErrorCategory::Network => write!(f, "network"),
            ErrorCategory::Validation => write!(f, "validation"),
            ErrorCategory::Internal => write!(f, "internal"),
        }
    }
}

impl UserFriendlyError for BackendError {
    fn user_message(&self) -> String {
        match self {
            BackendError::Config(_) => {
                "Configuration error. Please check your settings.".to_string()
            }
            BackendError::Crypto(CryptoError::InvalidMasterKey) => {
                "Invalid master key. Please check your password.".to_string()
            }
            BackendError::Crypto(CryptoError::MasterKeyUnavailable) => {
                "Database is locked. Please unlock it first.".to_string()
            }
            BackendError::Crypto(_) => "Encryption operation failed. Please try again.".to_string(),
            BackendError::Storage(StorageError::ArchiveNotFound { .. }) => {
                "Password database not found. Please create a new one or check the file path."
                    .to_string()
            }
            BackendError::Storage(StorageError::CorruptedArchive { .. }) => {
                "Password database appears to be corrupted. Please restore from backup.".to_string()
            }
            BackendError::Storage(StorageError::RecordNotFound { .. }) => {
                "The requested item was not found.".to_string()
            }
            BackendError::Storage(_) => "Storage operation failed. Please try again.".to_string(),
            BackendError::Auth(AuthError::NotAuthenticated) => {
                "Please unlock the database first.".to_string()
            }
            BackendError::Auth(AuthError::InvalidCredentials) => {
                "Invalid password. Please try again.".to_string()
            }
            BackendError::Auth(AuthError::TooManyAttempts) => {
                "Too many failed attempts. Please wait before trying again.".to_string()
            }
            BackendError::Auth(_) => "Authentication failed. Please try again.".to_string(),
            BackendError::Ipc(_) => {
                "Communication error. Please restart the application.".to_string()
            }
            BackendError::Validation(msg) => {
                format!("Validation error: {}", msg)
            }
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }

    fn category(&self) -> ErrorCategory {
        match self {
            BackendError::Config(_) => ErrorCategory::Configuration,
            BackendError::Crypto(_) => ErrorCategory::Crypto,
            BackendError::Storage(_) => ErrorCategory::Storage,
            BackendError::Ipc(_) => ErrorCategory::Network,
            BackendError::Auth(_) => ErrorCategory::Authentication,
            BackendError::Validation(_) => ErrorCategory::Validation,
            _ => ErrorCategory::Internal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = BackendError::Crypto(CryptoError::InvalidMasterKey);
        assert!(error.to_string().contains("Invalid master key"));
    }

    #[test]
    fn test_user_friendly_message() {
        let error = BackendError::Crypto(CryptoError::InvalidMasterKey);
        let message = error.user_message();
        assert!(!message.contains("crypto"));
        assert!(message.contains("password"));
    }

    #[test]
    fn test_error_category() {
        let error = BackendError::Crypto(CryptoError::InvalidMasterKey);
        assert_eq!(error.category(), ErrorCategory::Crypto);
    }

    #[test]
    fn test_error_chaining() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let backend_error = BackendError::FileSystem(io_error);

        assert!(backend_error.to_string().contains("File system error"));
    }
}
