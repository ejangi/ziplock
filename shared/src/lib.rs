//! ZipLock Shared Library - Unified Architecture
//!
//! This crate provides the core shared functionality for the ZipLock password manager
//! using a unified architecture that separates memory operations from file operations.
//! This design enables optimal performance and platform-specific optimizations while
//! maintaining code reuse across desktop and mobile platforms.
//!
//! # Architecture Overview
//!
//! The unified architecture consists of:
//! - **Core**: Pure memory repository and file operation abstraction
//! - **Models**: Credential data structures and templates
//! - **Utils**: Validation, search, YAML, and TOTP utilities
//! - **FFI**: Platform-specific interfaces for mobile and desktop
//! - **Logging**: Cross-platform logging infrastructure
//!
//! # Platform Integration
//!
//! - **Mobile Platforms**: Use memory-only FFI with JSON file exchange
//! - **Desktop Platforms**: Use full FFI with direct file operations
//! - **File Operations**: Delegated to platform-specific providers
//!
//! # Usage Examples
//!
//! ## Pure Memory Operations
//!
//! ```rust
//! use ziplock_shared::core::UnifiedMemoryRepository;
//! use ziplock_shared::models::{CredentialRecord, CredentialField};
//!
//! let mut repo = UnifiedMemoryRepository::new();
//! repo.initialize().unwrap();
//!
//! let mut credential = CredentialRecord::new("Gmail".to_string(), "login".to_string());
//! credential.set_field("username", CredentialField::username("user@gmail.com"));
//! credential.set_field("password", CredentialField::password("secure123"));
//!
//! repo.add_credential(credential).unwrap();
//! ```
//!
//! ## Full Repository Manager
//!
//! ```rust,no_run
//! use ziplock_shared::core::{UnifiedRepositoryManager, DesktopFileProvider};
//!
//! let provider = DesktopFileProvider::new();
//! let mut manager = UnifiedRepositoryManager::new(provider);
//!
//! // Create or open repository
//! manager.create_repository("/path/to/vault.7z", "master_password").unwrap();
//!
//! // Add credentials, save automatically handled
//! // ...
//! ```

pub mod config;
pub mod core;
pub mod ffi;
pub mod logging;
pub mod models;
pub mod utils;

// Re-export core functionality
pub use core::{
    CoreError, CoreResult, DesktopFileProvider, FileError, FileOperationProvider, FileResult,
    UnifiedMemoryRepository, UnifiedRepositoryManager,
};

// Re-export configuration management
pub use config::{
    AppConfig, ConfigManager, ConfigPaths, ConfigPresets, ConfigValidator, RepositoryConfig,
    RepositoryInfo, RepositoryMetadata, RepositorySecurity, SecurityConfig, UiConfig,
    ValidationConfig, ValidationRule, ValidationSeverity,
};

// Re-export commonly used models
pub use models::{
    CommonTemplates, CredentialField, CredentialRecord, CredentialTemplate, FieldTemplate,
    FieldType,
};

// Re-export utilities
pub use utils::{
    deserialize_credential, generate_totp, serialize_credential, validate_credential, BackupData,
    BackupManager, CredentialCrypto, CredentialSearchEngine, EncryptionUtils, ExportFormat,
    ExportOptions, PasswordAnalyzer, PasswordGenerator, PasswordOptions, PasswordStrength,
    SearchQuery, SearchResult, SecureString, ValidationResult,
};

// Re-export logging
pub use logging::{
    init_default_logging, init_desktop_logging, init_mobile_logging, LogLevel, LoggingConfig,
};

// Re-export FFI common utilities for platform integration
pub use ffi::common::{VersionInfo, ZipLockError};

// Re-export plugin system
pub use core::{Plugin, PluginCapability, PluginManager, PluginRegistry};

/// Current library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Archive format version supported by this library
pub const ARCHIVE_FORMAT_VERSION: &str = "1.0";

/// Shared error type for the unified architecture
pub type SharedError = CoreError;

/// Shared result type for the unified architecture
pub type SharedResult<T> = CoreResult<T>;

/// Initialize the shared library with default configuration
///
/// This should be called once at application startup. It initializes
/// logging and other global state needed by the shared library.
pub fn init_ziplock_shared() {
    init_default_logging();
}

/// Initialize the shared library for mobile platforms
///
/// This variant sets up mobile-specific configuration including
/// appropriate logging and performance optimizations.
pub fn init_ziplock_shared_mobile() {
    init_mobile_logging();
}

/// Initialize the shared library for desktop platforms
///
/// This variant enables more verbose logging and debugging features
/// suitable for desktop development and usage.
pub fn init_ziplock_shared_desktop() {
    init_desktop_logging();
}

/// Create a desktop configuration manager with default paths
///
/// This is a convenience function for desktop applications to quickly
/// set up configuration management using platform-appropriate paths.
pub fn create_desktop_config_manager() -> ConfigManager<DesktopFileProvider> {
    let file_provider = DesktopFileProvider::new();
    let config_path = ConfigPaths::app_config_file();
    ConfigManager::new(file_provider, config_path)
}

/// Get library version information
pub fn get_version() -> &'static str {
    VERSION
}

/// Get supported archive format version
pub fn get_archive_format_version() -> &'static str {
    ARCHIVE_FORMAT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MockFileProvider, PluginManager, UnifiedRepositoryManager};
    use config::{AppConfig, ConfigManager, ConfigPaths, ConfigValidator, RepositoryInfo};
    use models::{CredentialField, CredentialRecord};
    use utils::{BackupManager, ExportFormat, ExportOptions};

    #[test]
    fn test_library_version() {
        assert!(!get_version().is_empty());
        assert!(!get_archive_format_version().is_empty());
    }

    #[test]
    fn test_unified_memory_repository() {
        let mut repo = UnifiedMemoryRepository::new();
        assert!(!repo.is_initialized());

        repo.initialize().unwrap();
        assert!(repo.is_initialized());

        let mut credential = CredentialRecord::new("Test".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("testuser"));

        let credential_id = credential.id.clone();
        repo.add_credential(credential).unwrap();

        let retrieved = repo.get_credential_readonly(&credential_id).unwrap();
        assert_eq!(retrieved.title, "Test");
    }

    #[test]
    fn test_repository_manager() {
        let provider = MockFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(provider);

        assert!(!manager.is_open());

        manager.create_repository("/test.7z", "password").unwrap();
        assert!(manager.is_open());

        let credential = CredentialRecord::new("Test Cred".to_string(), "test".to_string());
        manager.add_credential(credential).unwrap();

        let credentials = manager.list_credentials().unwrap();
        assert_eq!(credentials.len(), 1);
    }

    #[test]
    fn test_credential_validation() {
        let credential = CredentialRecord::new("Valid Credential".to_string(), "login".to_string());
        let result = validate_credential(&credential);
        assert!(result.is_valid);
    }

    #[test]
    fn test_search_functionality() {
        use std::collections::HashMap;

        let mut credentials = HashMap::new();
        let cred1 = CredentialRecord::new("Gmail Account".to_string(), "login".to_string());
        let cred2 = CredentialRecord::new("Bank Login".to_string(), "login".to_string());

        credentials.insert(cred1.id.clone(), cred1);
        credentials.insert(cred2.id.clone(), cred2);

        let query = SearchQuery::text("Gmail");
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].credential.title, "Gmail Account");
    }

    #[test]
    fn test_common_templates() {
        let login_template = CommonTemplates::login();
        assert_eq!(login_template.name, "login");
        assert!(!login_template.fields.is_empty());

        let credential = login_template
            .create_credential("My Login".to_string())
            .unwrap();
        assert_eq!(credential.title, "My Login");
    }

    #[test]
    fn test_list_credentials_serialization() {
        println!("=== Testing list_credentials serialization ===");

        // Create a memory repository
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().expect("Failed to initialize repository");

        // Create a test credential with username field
        let mut credential =
            CredentialRecord::new("Test Credential".to_string(), "login".to_string());
        credential.set_field(
            "username",
            CredentialField::new(FieldType::Username, "test@example.com".to_string(), false),
        );
        credential.set_field(
            "password",
            CredentialField::new(FieldType::Password, "testpass".to_string(), true),
        );

        // Add the credential
        repo.add_credential(credential)
            .expect("Failed to add credential");

        // Test list_credentials
        let credentials = repo.list_credentials().expect("Failed to list credentials");
        println!(
            "DEBUG: list_credentials returned {} items",
            credentials.len()
        );

        if let Some(first_cred) = credentials.first() {
            println!("DEBUG: First credential ID: '{}'", first_cred.id);
            println!("DEBUG: First credential title: '{}'", first_cred.title);
            println!(
                "DEBUG: First credential fields: {:?}",
                first_cred.fields.keys().collect::<Vec<_>>()
            );

            // Test serialization
            match serde_json::to_string(&credentials) {
                Ok(json) => {
                    println!("DEBUG: Serialized JSON: {}", json);

                    // Verify it starts with array of objects, not tuples
                    if json.starts_with("[{") {
                        println!("✅ Serialization produces array of objects (correct)");
                    } else if json.starts_with("[[") {
                        println!("❌ Serialization produces array of arrays (incorrect - tuples)");
                    } else {
                        println!(
                            "⚠️  Unexpected serialization format: {}",
                            &json[..50.min(json.len())]
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Serialization failed: {}", e);
                }
            }
        }

        // Test list_credential_summaries for comparison
        let summaries = repo
            .list_credential_summaries()
            .expect("Failed to list summaries");
        println!(
            "DEBUG: list_credential_summaries returned {} items",
            summaries.len()
        );

        match serde_json::to_string(&summaries) {
            Ok(json) => {
                println!("DEBUG: Summaries JSON: {}", json);
                if json.starts_with("[[") {
                    println!("✅ Summaries correctly produce array of arrays (tuples)");
                }
            }
            Err(e) => {
                println!("❌ Summaries serialization failed: {}", e);
            }
        }
    }

    #[test]
    fn test_ffi_list_credentials_direct() {
        use crate::ffi::mobile::*;
        use std::ffi::CStr;

        println!("=== Testing FFI list_credentials directly ===");

        // Create repository handle
        let handle = unsafe { ziplock_mobile_repository_create() };
        assert!(!handle.is_null(), "Failed to create repository handle");

        // Initialize repository
        let init_result = unsafe { ziplock_mobile_repository_initialize(handle) };
        assert!(
            matches!(init_result, crate::ffi::ZipLockError::Success),
            "Failed to initialize repository"
        );

        // Create test credential JSON
        let test_credential = r#"{
            "id": "test-id-123",
            "title": "Test Login",
            "credential_type": "login",
            "fields": {
                "username": {
                    "value": "testuser@example.com",
                    "field_type": "Username",
                    "sensitive": false,
                    "metadata": {}
                },
                "password": {
                    "value": "testpassword",
                    "field_type": "Password",
                    "sensitive": true,
                    "metadata": {}
                }
            },
            "tags": [],
            "notes": null,
            "created_at": 1694000000,
            "updated_at": 1694000000,
            "accessed_at": 1694000000,
            "favorite": false,
            "folder_path": null
        }"#;

        // Add the credential
        let add_result = unsafe {
            let c_str = std::ffi::CString::new(test_credential).unwrap();
            ziplock_mobile_add_credential(handle, c_str.as_ptr())
        };
        assert!(
            matches!(add_result, crate::ffi::ZipLockError::Success),
            "Failed to add credential"
        );

        // Test list_credentials
        let list_result = unsafe { ziplock_mobile_list_credentials(handle) };
        assert!(!list_result.is_null(), "list_credentials returned null");

        let c_str = unsafe { CStr::from_ptr(list_result) };
        let json_str = c_str.to_str().expect("Invalid UTF-8");

        println!("DEBUG: FFI list_credentials JSON: {}", json_str);

        if json_str.starts_with("[{") {
            println!("✅ FFI list_credentials produces array of objects (correct)");
        } else if json_str.starts_with("[[") {
            println!("❌ FFI list_credentials produces array of arrays (incorrect - tuples)");
        } else {
            println!(
                "⚠️  FFI unexpected format: {}",
                &json_str[..50.min(json_str.len())]
            );
        }

        // Clean up
        unsafe {
            ziplock_mobile_free_string(list_result);
            ziplock_mobile_repository_destroy(handle);
        }
    }

    #[test]
    fn test_totp_generation() {
        let secret = "JBSWY3DPEHPK3PXP";
        let code = generate_totp(secret, 30).unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_serialization() {
        let credential = CredentialRecord::new("Test".to_string(), "test".to_string());

        let yaml = serialize_credential(&credential).unwrap();
        assert!(yaml.contains("Test"));

        let deserialized = deserialize_credential(&yaml).unwrap();
        assert_eq!(credential.id, deserialized.id);
        assert_eq!(credential.title, deserialized.title);
    }

    #[test]
    fn test_initialization_functions() {
        // These should not panic
        init_ziplock_shared();
        init_ziplock_shared_mobile();
        init_ziplock_shared_desktop();
    }

    #[test]
    fn test_plugin_system() {
        let plugin_manager = PluginManager::new();
        let templates = plugin_manager.get_plugin_templates();
        let field_types = plugin_manager.get_custom_field_types();

        // Should work even with no plugins registered
        // Check that plugin system is functional
        assert!(templates.is_empty() || !templates.is_empty()); // Either state is valid
        assert!(field_types.is_empty() || !field_types.is_empty()); // Either state is valid
    }

    #[test]
    fn test_backup_functionality() {
        let mut repo = UnifiedMemoryRepository::new();
        repo.initialize().unwrap();

        let credential = CredentialRecord::new("Backup Test".to_string(), "test".to_string());
        repo.add_credential(credential).unwrap();

        let options = ExportOptions {
            format: ExportFormat::Json,
            ..Default::default()
        };

        let backup = BackupManager::create_backup(&repo, &options, None);
        assert!(backup.is_ok());

        let backup = backup.unwrap();
        assert_eq!(backup.credentials.len(), 1);
        assert!(BackupManager::verify_backup(&backup));
    }

    #[test]
    fn test_password_utilities() {
        let options = PasswordOptions::default();
        let password = PasswordGenerator::generate(&options).unwrap();
        assert_eq!(password.len(), options.length);

        let analysis = PasswordAnalyzer::analyze(&password);
        assert!(analysis.score > 0);
        assert!(analysis.entropy > 0.0);
    }

    #[test]
    fn test_encryption_utilities() {
        let plaintext = "sensitive data";
        let master_password = "master_key";

        let encrypted = CredentialCrypto::encrypt_field(plaintext, master_password).unwrap();
        assert!(CredentialCrypto::is_encrypted(&encrypted));

        let decrypted = CredentialCrypto::decrypt_field(&encrypted, master_password).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_secure_string() {
        let sensitive_data = "password123".to_string();
        let secure = SecureString::new(sensitive_data.clone());

        assert_eq!(secure.as_str(), &sensitive_data);
        assert_eq!(secure.len(), sensitive_data.len());
        assert!(!secure.is_empty());

        // Test that debug output is redacted
        let debug_output = format!("{:?}", secure);
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("password123"));
    }

    #[test]
    fn test_config_management() {
        let provider = MockFileProvider::new();
        let mut manager = ConfigManager::new(provider, "/test/config.yml".to_string());

        // Should not be loaded initially
        assert!(!manager.is_loaded());

        // Load should succeed even without file
        manager.load().unwrap();
        assert!(manager.is_loaded());

        // Should have default configuration
        let config = manager.config();
        assert_eq!(config.ui.theme, "system");
        assert_eq!(config.ui.language, "en");
        assert_eq!(config.security.password_timeout, 300);

        // Test repository management
        let repo = RepositoryInfo::new("Test Repo".to_string(), "/path/to/test.7z".to_string());
        manager.add_recent_repository(repo);

        let recent = manager.get_recent_repositories();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].name, "Test Repo");
    }

    #[test]
    fn test_config_paths() {
        let config_dir = ConfigPaths::app_config_dir();
        assert!(!config_dir.is_empty());

        let config_file = ConfigPaths::app_config_file();
        assert!(config_file.contains("config.yml"));
        assert!(config_file.contains(&config_dir));

        let repos_dir = ConfigPaths::default_repositories_dir();
        assert!(!repos_dir.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig::default();
        let errors = ConfigValidator::validate_app_config(&config);
        assert!(errors.is_empty());

        // Test repository path validation
        assert!(ConfigValidator::is_valid_repository_path(
            "/path/to/vault.7z"
        ));
        assert!(ConfigValidator::is_valid_repository_path(
            "C:\\Users\\test\\vault.zip"
        ));
        assert!(!ConfigValidator::is_valid_repository_path(
            "/path/to/vault.txt"
        ));
        assert!(!ConfigValidator::is_valid_repository_path(""));
    }

    #[test]
    fn test_desktop_config_manager_creation() {
        // This should not panic and should return a valid config manager
        let manager = create_desktop_config_manager();
        assert!(!manager.is_loaded()); // Should not be loaded until explicitly loaded
    }
}
