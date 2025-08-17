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
pub mod update_checker;
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

// Re-export update checker functionality
pub use update_checker::{
    InstallationMethod, ReleaseAsset, ReleaseInfo, UpdateCheckResult, UpdateChecker,
};

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

        let identity_template = CommonTemplates::identity();
        assert_eq!(identity_template.name, "identity");
        assert!(!identity_template.fields.is_empty());

        let password_template = CommonTemplates::password();
        assert_eq!(password_template.name, "password");
        assert!(!password_template.fields.is_empty());

        let document_template = CommonTemplates::document();
        assert_eq!(document_template.name, "document");
        assert!(!document_template.fields.is_empty());

        let ssh_key_template = CommonTemplates::ssh_key();
        assert_eq!(ssh_key_template.name, "ssh_key");
        assert!(!ssh_key_template.fields.is_empty());

        let bank_account_template = CommonTemplates::bank_account();
        assert_eq!(bank_account_template.name, "bank_account");
        assert!(!bank_account_template.fields.is_empty());

        let api_credentials_template = CommonTemplates::api_credentials();
        assert_eq!(api_credentials_template.name, "api_credentials");
        assert!(!api_credentials_template.fields.is_empty());

        let crypto_wallet_template = CommonTemplates::crypto_wallet();
        assert_eq!(crypto_wallet_template.name, "crypto_wallet");
        assert!(!crypto_wallet_template.fields.is_empty());

        let database_template = CommonTemplates::database();
        assert_eq!(database_template.name, "database");
        assert!(!database_template.fields.is_empty());

        let software_license_template = CommonTemplates::software_license();
        assert_eq!(software_license_template.name, "software_license");
        assert!(!software_license_template.fields.is_empty());
    }

    #[test]
    #[cfg(feature = "c-api")]
    fn test_common_templates_ffi_integration() {
        // Test that CommonTemplates can be accessed through FFI
        use std::ffi::CString;
        use std::ptr;

        // Test getting all templates
        let mut templates_ptr: *mut crate::ffi::CCredentialTemplate = ptr::null_mut();
        let mut count: std::os::raw::c_int = 0;

        let result =
            unsafe { crate::ffi::ziplock_templates_get_all(&mut templates_ptr, &mut count) };
        assert_eq!(result, 0); // Success
        assert!(!templates_ptr.is_null());
        assert_eq!(count, 12); // We have 12 built-in templates

        // Clean up
        unsafe { crate::ffi::ziplock_templates_free(templates_ptr, count) };

        // Test getting specific template
        let mut template = crate::ffi::CCredentialTemplate {
            name: ptr::null_mut(),
            description: ptr::null_mut(),
            field_count: 0,
            fields: ptr::null_mut(),
            tag_count: 0,
            tags: ptr::null_mut(),
        };

        let template_name = CString::new("login").unwrap();
        let result = unsafe {
            crate::ffi::ziplock_template_get_by_name(template_name.as_ptr(), &mut template)
        };
        assert_eq!(result, 0); // Success

        assert!(!template.name.is_null());
        let name = unsafe { std::ffi::CStr::from_ptr(template.name).to_str().unwrap() };
        assert_eq!(name, "login");

        // Clean up
        unsafe { crate::ffi::ziplock_template_free(&mut template) };
    }

    #[test]
    fn test_all_specification_credential_types_implemented() {
        // According to specification section 3.3, these are all the required credential types
        let all_templates = vec![
            ("Login", CommonTemplates::login()),
            ("Secure Note", CommonTemplates::secure_note()),
            ("Credit Card", CommonTemplates::credit_card()),
            ("Identity", CommonTemplates::identity()),
            ("Password", CommonTemplates::password()),
            ("Document", CommonTemplates::document()),
            ("SSH Key", CommonTemplates::ssh_key()),
            ("Bank Account", CommonTemplates::bank_account()),
            ("API Credentials", CommonTemplates::api_credentials()),
            ("Crypto Wallet", CommonTemplates::crypto_wallet()),
            ("Database", CommonTemplates::database()),
            ("Software License", CommonTemplates::software_license()),
        ];

        // Verify we have all 12 credential types from the specification
        assert_eq!(
            all_templates.len(),
            12,
            "Missing credential types from specification"
        );

        // Verify each template is properly configured
        for (description, template) in all_templates {
            assert!(
                !template.name.is_empty(),
                "{} template missing name",
                description
            );
            assert!(
                !template.description.is_empty(),
                "{} template missing description",
                description
            );
            assert!(
                !template.fields.is_empty(),
                "{} template has no fields",
                description
            );
            assert!(
                !template.default_tags.is_empty(),
                "{} template has no default tags",
                description
            );

            // Verify each field has required properties
            for field in &template.fields {
                assert!(
                    !field.name.is_empty(),
                    "{} template has field with empty name",
                    description
                );
                assert!(
                    !field.label.is_empty(),
                    "{} template has field with empty label",
                    description
                );
            }
        }
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
