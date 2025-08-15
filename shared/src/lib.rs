//! ZipLock Shared Library
//!
//! This crate contains shared data models, utilities, and common functionality
//! used across the ZipLock password manager application. It provides a consistent
//! interface for credential management, field validation, and data serialization.
//!
//! # Features
//!
//! - **Data Models**: Core structures for credentials, fields, and templates
//! - **Field Types**: Comprehensive field type system with validation
//! - **YAML Support**: Serialization and deserialization for credential records
//! - **Utilities**: Common functions for validation, search, and manipulation
//! - **Template System**: Pre-defined templates for common credential types
//!
//! # Usage
//!
//! ```rust
//! use ziplock_shared::models::{CredentialRecord, CredentialField, FieldType};
//!
//! // Create a new credential
//! let mut credential = CredentialRecord::new(
//!     "My Login".to_string(),
//!     "login".to_string(),
//! );
//!
//! // Add fields
//! credential.set_field("username", CredentialField::username("user@example.com"));
//! credential.set_field("password", CredentialField::password("secure_password"));
//!
//! // Validate the credential
//! assert!(credential.validate().is_ok());
//! ```

pub mod api;
pub mod archive;
pub mod client;
pub mod config;

pub mod models;
pub mod utils;
pub mod validation;
pub mod yaml;

// C FFI module for mobile platform integration
#[cfg(feature = "c-api")]
pub mod ffi;

// Re-export commonly used types for convenience
pub use models::{
    CommonTemplates, CredentialField, CredentialRecord, CredentialTemplate, FieldTemplate,
    FieldType, FieldValidation,
};

// Re-export config functionality
pub use config::{
    AppConfig, ConfigManager, FrontendConfig, RecentRepository, RepositoryConfig, RepositoryInfo,
    UiConfig,
};

// Re-export client functionality
pub use client::ZipLockClient;

// Re-export utilities
pub use utils::*;

// Re-export validation functionality
pub use validation::{
    is_valid_credential_id, sanitize_identifier, validate_credential, validate_master_passphrase,
    validate_master_passphrase_strict, CommonPatterns, EnhancedPassphraseValidator,
    PassphraseRequirements, PassphraseStrength, PassphraseValidationResult, PassphraseValidator,
    StrengthLevel, ValidationPresets, ValidationUtils,
};

// Re-export YAML functionality
pub use yaml::*;

// Re-export API functionality
pub use api::{ApiError, ApiResult, ApiSession, ZipLockApi};

// Re-export archive functionality
pub use archive::{ArchiveConfig, ArchiveError, ArchiveManager, ArchiveResult};

/// Current library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported archive format version
pub const ARCHIVE_FORMAT_VERSION: &str = "1.0";

/// Error types used throughout the library
pub mod error {
    use thiserror::Error;

    /// Common error type for shared library operations
    #[derive(Error, Debug)]
    pub enum SharedError {
        #[error("Validation error: {message}")]
        Validation { message: String },

        #[error("Serialization error: {message}")]
        Serialization { message: String },

        #[error("Field error: {field} - {message}")]
        Field { field: String, message: String },

        #[error("Template error: {template} - {message}")]
        Template { template: String, message: String },

        #[error("Invalid data format: {message}")]
        InvalidFormat { message: String },

        #[error("Missing required field: {field}")]
        MissingField { field: String },

        #[error("Internal error: {message}")]
        Internal { message: String },

        #[error("Archive error: {0}")]
        Archive(#[from] crate::archive::ArchiveError),

        #[error("API error: {message}")]
        Api { message: String },

        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Configuration error: {message}")]
        Config { message: String },

        #[error("Authentication error: {message}")]
        Auth { message: String },
    }

    impl From<anyhow::Error> for SharedError {
        fn from(error: anyhow::Error) -> Self {
            SharedError::Internal {
                message: error.to_string(),
            }
        }
    }

    /// Result type alias for shared library operations
    pub type SharedResult<T> = Result<T, SharedError>;
}

pub use error::{SharedError, SharedResult};

/// Library configuration and constants
pub mod constants {
    /// Maximum field value length (1MB)
    pub const MAX_FIELD_VALUE_LENGTH: usize = 1024 * 1024;

    /// Maximum number of fields per credential
    pub const MAX_FIELDS_PER_CREDENTIAL: usize = 100;

    /// Maximum number of tags per credential
    pub const MAX_TAGS_PER_CREDENTIAL: usize = 20;

    /// Maximum tag length
    pub const MAX_TAG_LENGTH: usize = 50;

    /// Maximum credential title length
    pub const MAX_CREDENTIAL_TITLE_LENGTH: usize = 200;

    /// Maximum notes length
    pub const MAX_NOTES_LENGTH: usize = 10000;

    /// Supported field types for validation
    pub const SUPPORTED_FIELD_TYPES: &[&str] = &[
        "text",
        "password",
        "email",
        "url",
        "username",
        "phone",
        "credit_card_number",
        "expiry_date",
        "cvv",
        "totp_secret",
        "text_area",
        "number",
        "date",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{CredentialField, CredentialRecord};

    #[test]
    fn test_library_version() {
        // VERSION and ARCHIVE_FORMAT_VERSION are compile-time constants
        // Just verify they exist and have expected content
        assert!(VERSION.starts_with(env!("CARGO_PKG_VERSION")));
        assert!(ARCHIVE_FORMAT_VERSION
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.'));
    }

    #[test]
    fn test_credential_creation() {
        let credential = CredentialRecord::new("Test".to_string(), "login".to_string());
        assert_eq!(credential.title, "Test");
        assert_eq!(credential.credential_type, "login");
        assert!(!credential.id.is_empty());
    }

    #[test]
    fn test_field_creation() {
        let field = CredentialField::password("secret").with_label("Password");
        assert_eq!(field.value, "secret");
        assert!(field.sensitive);
        assert_eq!(field.label, Some("Password".to_string()));
    }

    #[test]
    fn test_validation() {
        let credential = CredentialRecord::new("Test".to_string(), "login".to_string());
        assert!(validate_credential(&credential).is_ok());

        // Test invalid ID format
        assert!(!is_valid_credential_id("invalid"));
        assert!(is_valid_credential_id(
            "550e8400-e29b-41d4-a716-446655440000"
        ));
    }

    #[test]
    fn test_identifier_sanitization() {
        assert_eq!(sanitize_identifier("Hello World!"), "Hello_World_");
        assert_eq!(sanitize_identifier("test-file_123"), "test-file_123");
    }

    #[test]
    fn test_config_constants() {
        // Test that constants have expected values for functionality
        let large_value = "a".repeat(constants::MAX_FIELD_VALUE_LENGTH + 1);
        assert!(large_value.len() > constants::MAX_FIELD_VALUE_LENGTH);

        let field_types = constants::SUPPORTED_FIELD_TYPES;
        assert!(field_types.contains(&"password"));
        assert!(field_types.contains(&"username"));
        assert!(field_types.contains(&"email"));
    }

    #[test]
    fn test_common_templates() {
        let login_template = CommonTemplates::login();
        assert_eq!(login_template.name, "login");
        assert!(!login_template.fields.is_empty());

        let cc_template = CommonTemplates::credit_card();
        assert_eq!(cc_template.name, "credit_card");
        assert!(!cc_template.fields.is_empty());

        let note_template = CommonTemplates::secure_note();
        assert_eq!(note_template.name, "secure_note");
        assert!(!note_template.fields.is_empty());
    }

    #[test]
    fn test_secure_note_uses_textarea() {
        let secure_note_template = CommonTemplates::secure_note();

        // Verify it has the correct name
        assert_eq!(secure_note_template.name, "secure_note");

        // Verify it has at least one field
        assert!(!secure_note_template.fields.is_empty());

        // Find the content field and verify it's a TextArea
        let content_field = secure_note_template
            .fields
            .iter()
            .find(|field| field.name == "content")
            .expect("Secure note template should have a 'content' field");

        assert_eq!(
            content_field.field_type,
            FieldType::TextArea,
            "Secure note content field should use TextArea type for multi-line input"
        );
        assert_eq!(content_field.label, "Content");
        assert!(
            content_field.sensitive,
            "Secure note content should be marked as sensitive"
        );
    }

    #[test]
    fn test_passphrase_validation() {
        let validator = PassphraseValidator::default();
        let result = validator.validate("MySecurePassphrase123!");
        assert!(result.meets_requirements);
        assert!(result.level.is_acceptable());
    }
}
