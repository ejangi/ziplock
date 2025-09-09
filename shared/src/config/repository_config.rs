//! Repository-specific configuration for ZipLock unified architecture
//!
//! This module provides configuration structures for individual repositories,
//! including validation rules, field templates, and repository-specific settings
//! that can be stored within or alongside the repository data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{CoreError, CoreResult};
use crate::models::{CredentialTemplate, FieldType};

/// Repository-specific configuration
///
/// This configuration is typically stored within the repository itself
/// (as part of the metadata) or alongside it. It contains settings that
/// are specific to how this particular repository should behave.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepositoryConfig {
    /// Repository metadata and identification
    pub metadata: RepositoryMetadata,

    /// Security and encryption settings
    pub security: RepositorySecurity,

    /// Field validation rules
    pub validation: ValidationConfig,

    /// Custom field templates for this repository
    pub templates: Vec<CredentialTemplate>,

    /// Custom field types defined for this repository
    pub custom_fields: HashMap<String, CustomFieldDefinition>,

    /// Repository behavior settings
    pub behavior: RepositoryBehavior,

    /// Integration settings (cloud sync, etc.)
    pub integration: IntegrationConfig,
}

/// Repository metadata and identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Repository name/title
    pub name: String,

    /// Repository description
    pub description: Option<String>,

    /// Repository version (for migrations)
    pub version: String,

    /// Repository format version
    pub format_version: String,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last modification timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,

    /// Repository creator/owner information
    pub owner: Option<String>,

    /// Repository tags for organization
    pub tags: Vec<String>,

    /// Repository icon/identifier
    pub icon: Option<String>,
}

/// Security and encryption configuration for the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySecurity {
    /// Minimum password length for credentials stored in this repository
    pub min_password_length: u32,

    /// Require uppercase letters in passwords
    pub require_uppercase: bool,

    /// Require lowercase letters in passwords
    pub require_lowercase: bool,

    /// Require numbers in passwords
    pub require_numbers: bool,

    /// Require special characters in passwords
    pub require_symbols: bool,

    /// Encryption algorithm used (informational)
    pub encryption_method: String,

    /// Key derivation method (informational)
    pub key_derivation: String,

    /// Number of iterations for key derivation
    pub iterations: u32,

    /// Whether to enforce password complexity for new credentials
    pub enforce_password_policy: bool,

    /// Password history length (prevent reusing recent passwords)
    pub password_history_length: u32,
}

/// Validation configuration for repository data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationConfig {
    /// URL validation settings
    pub url_validation: UrlValidation,

    /// Email validation settings
    pub email_validation: EmailValidation,

    /// Phone number validation settings
    pub phone_validation: PhoneValidation,

    /// Credit card validation settings
    pub credit_card_validation: CreditCardValidation,

    /// Custom validation rules
    pub custom_rules: Vec<ValidationRule>,
}

/// URL field validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlValidation {
    /// Whether to validate URL format
    pub validate_format: bool,

    /// Whether to check if URL is reachable
    pub check_reachability: bool,

    /// Allowed URL schemes (http, https, ftp, etc.)
    pub allowed_schemes: Vec<String>,

    /// Blocked domains/hosts
    pub blocked_domains: Vec<String>,
}

/// Email field validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailValidation {
    /// Whether to validate email format
    pub validate_format: bool,

    /// Whether to check if domain exists
    pub check_domain: bool,

    /// Allowed email domains (empty = allow all)
    pub allowed_domains: Vec<String>,

    /// Blocked email domains
    pub blocked_domains: Vec<String>,
}

/// Phone number validation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhoneValidation {
    /// Whether to validate phone number format
    pub validate_format: bool,

    /// Default country code for validation
    pub default_country_code: Option<String>,

    /// Allowed country codes (empty = allow all)
    pub allowed_countries: Vec<String>,

    /// Required phone number format (E164, national, etc.)
    pub required_format: Option<String>,
}

/// Credit card validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardValidation {
    /// Whether to validate credit card number using Luhn algorithm
    pub validate_luhn: bool,

    /// Whether to check card type (Visa, MasterCard, etc.)
    pub detect_card_type: bool,

    /// Allowed card types
    pub allowed_card_types: Vec<String>,

    /// Whether to validate expiry date
    pub validate_expiry: bool,
}

/// Custom validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name/identifier
    pub name: String,

    /// Fields this rule applies to
    pub applicable_fields: Vec<String>,

    /// Regular expression pattern for validation
    pub regex_pattern: Option<String>,

    /// Minimum length requirement
    pub min_length: Option<u32>,

    /// Maximum length requirement
    pub max_length: Option<u32>,

    /// Whether this rule is required or just a warning
    pub severity: ValidationSeverity,

    /// Error message to display when validation fails
    pub error_message: String,
}

/// Validation rule severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    /// Rule failure prevents saving
    Error,
    /// Rule failure shows warning but allows saving
    Warning,
    /// Rule failure shows info message
    Info,
}

/// Custom field type definition for repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldDefinition {
    /// Field type name
    pub name: String,

    /// Display label for this field type
    pub display_name: String,

    /// Description of this field type
    pub description: Option<String>,

    /// Base field type this extends
    pub base_type: FieldType,

    /// Whether this field should be treated as sensitive
    pub is_sensitive: bool,

    /// Default validation rules for this field type
    pub validation_rules: Vec<String>,

    /// Input mask or format string
    pub input_mask: Option<String>,

    /// Default value for new fields of this type
    pub default_value: Option<String>,
}

/// Repository behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepositoryBehavior {
    /// Auto-save settings
    pub auto_save: AutoSaveConfig,

    /// Backup settings
    pub backup: BackupConfig,

    /// Import/export settings
    pub import_export: ImportExportConfig,

    /// Search and indexing settings
    pub search: SearchConfig,
}

/// Auto-save configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    /// Whether auto-save is enabled
    pub enabled: bool,

    /// Auto-save interval in seconds
    pub interval_seconds: u64,

    /// Whether to save on credential modification
    pub save_on_modify: bool,

    /// Whether to save on application focus loss
    pub save_on_focus_loss: bool,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Whether automatic backups are enabled
    pub enabled: bool,

    /// Backup interval in hours
    pub interval_hours: u64,

    /// Number of backups to retain
    pub retention_count: u32,

    /// Backup storage location (relative to repository)
    pub backup_location: Option<String>,

    /// Whether to compress backups
    pub compress_backups: bool,
}

/// Import/export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExportConfig {
    /// Supported import formats
    pub supported_import_formats: Vec<String>,

    /// Supported export formats
    pub supported_export_formats: Vec<String>,

    /// Default export format
    pub default_export_format: String,

    /// Whether to include metadata in exports
    pub include_metadata: bool,

    /// Whether to include sensitive fields in exports
    pub include_sensitive_fields: bool,
}

/// Search and indexing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Whether to enable full-text search indexing
    pub enable_full_text_search: bool,

    /// Fields to include in search index
    pub indexed_fields: Vec<String>,

    /// Whether to index sensitive fields
    pub index_sensitive_fields: bool,

    /// Search result limit
    pub max_search_results: u32,

    /// Whether to enable fuzzy search
    pub enable_fuzzy_search: bool,

    /// Fuzzy search threshold (0.0 - 1.0)
    pub fuzzy_threshold: f32,
}

/// Integration configuration for external services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationConfig {
    /// Cloud sync settings
    pub cloud_sync: Option<CloudSyncConfig>,

    /// Two-factor authentication settings
    pub two_factor: TwoFactorConfig,

    /// Plugin configurations
    pub plugins: HashMap<String, serde_yaml::Value>,
}

/// Cloud synchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncConfig {
    /// Cloud provider name
    pub provider: String,

    /// Whether sync is enabled
    pub enabled: bool,

    /// Sync interval in minutes
    pub sync_interval_minutes: u64,

    /// Whether to sync automatically
    pub auto_sync: bool,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
}

/// Conflict resolution strategies for cloud sync
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    /// Always use local version
    PreferLocal,
    /// Always use remote version
    PreferRemote,
    /// Use the newer version
    PreferNewer,
    /// Prompt user to choose
    PromptUser,
    /// Create duplicate entries
    CreateDuplicate,
}

/// Two-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorConfig {
    /// Whether 2FA is required for this repository
    pub required: bool,

    /// Supported 2FA methods
    pub supported_methods: Vec<TwoFactorMethod>,

    /// Default TOTP settings
    pub totp_settings: TotpSettings,
}

/// Two-factor authentication methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TwoFactorMethod {
    /// Time-based One-Time Password
    Totp,
    /// SMS-based authentication
    Sms,
    /// Email-based authentication
    Email,
    /// Hardware security key
    SecurityKey,
    /// Biometric authentication
    Biometric,
}

/// TOTP (Time-based One-Time Password) settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSettings {
    /// TOTP code length (usually 6 or 8)
    pub digits: u32,

    /// Time step in seconds (usually 30)
    pub time_step: u64,

    /// Hash algorithm (SHA1, SHA256, SHA512)
    pub algorithm: String,
}

impl Default for RepositoryMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            name: "ZipLock Repository".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            format_version: "1.0".to_string(),
            created_at: now,
            modified_at: now,
            owner: None,
            tags: Vec::new(),
            icon: None,
        }
    }
}

impl Default for RepositorySecurity {
    fn default() -> Self {
        Self {
            min_password_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_numbers: false,
            require_symbols: false,
            encryption_method: "AES-256".to_string(),
            key_derivation: "PBKDF2".to_string(),
            iterations: 100000,
            enforce_password_policy: false,
            password_history_length: 5,
        }
    }
}

impl Default for UrlValidation {
    fn default() -> Self {
        Self {
            validate_format: true,
            check_reachability: false,
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            blocked_domains: Vec::new(),
        }
    }
}

impl Default for CreditCardValidation {
    fn default() -> Self {
        Self {
            validate_luhn: true,
            detect_card_type: true,
            allowed_card_types: vec![
                "Visa".to_string(),
                "MasterCard".to_string(),
                "American Express".to_string(),
                "Discover".to_string(),
            ],
            validate_expiry: true,
        }
    }
}

impl Default for EmailValidation {
    fn default() -> Self {
        Self {
            validate_format: true,
            check_domain: false,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
        }
    }
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 300, // 5 minutes
            save_on_modify: false,
            save_on_focus_loss: true,
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 24,
            retention_count: 7,
            backup_location: None,
            compress_backups: true,
        }
    }
}

impl Default for ImportExportConfig {
    fn default() -> Self {
        Self {
            supported_import_formats: vec![
                "json".to_string(),
                "csv".to_string(),
                "1password".to_string(),
                "bitwarden".to_string(),
                "keepass".to_string(),
            ],
            supported_export_formats: vec!["json".to_string(), "csv".to_string()],
            default_export_format: "json".to_string(),
            include_metadata: true,
            include_sensitive_fields: false,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enable_full_text_search: true,
            indexed_fields: vec![
                "title".to_string(),
                "username".to_string(),
                "url".to_string(),
                "notes".to_string(),
            ],
            index_sensitive_fields: false,
            max_search_results: 100,
            enable_fuzzy_search: true,
            fuzzy_threshold: 0.8,
        }
    }
}

impl Default for TwoFactorConfig {
    fn default() -> Self {
        Self {
            required: false,
            supported_methods: vec![TwoFactorMethod::Totp],
            totp_settings: TotpSettings::default(),
        }
    }
}

impl Default for TotpSettings {
    fn default() -> Self {
        Self {
            digits: 6,
            time_step: 30,
            algorithm: "SHA1".to_string(),
        }
    }
}

impl RepositoryConfig {
    /// Create a new repository configuration with given name
    pub fn new(name: String) -> Self {
        let mut config = Self::default();
        config.metadata.name = name;
        config
    }

    /// Validate the repository configuration
    pub fn validate(&self) -> CoreResult<()> {
        // Validate metadata
        if self.metadata.name.is_empty() {
            return Err(CoreError::ValidationError {
                message: "Repository name cannot be empty".to_string(),
            });
        }

        // Validate security settings
        if self.security.min_password_length < 4 {
            return Err(CoreError::ValidationError {
                message: "Minimum password length must be at least 4".to_string(),
            });
        }

        if self.security.iterations < 10000 {
            return Err(CoreError::ValidationError {
                message: "Key derivation iterations should be at least 10,000".to_string(),
            });
        }

        // Validate search configuration
        if self.behavior.search.fuzzy_threshold < 0.0 || self.behavior.search.fuzzy_threshold > 1.0
        {
            return Err(CoreError::ValidationError {
                message: "Fuzzy search threshold must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate TOTP settings
        if self.integration.two_factor.totp_settings.digits < 6
            || self.integration.two_factor.totp_settings.digits > 8
        {
            return Err(CoreError::ValidationError {
                message: "TOTP digits must be 6, 7, or 8".to_string(),
            });
        }

        Ok(())
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.metadata.modified_at = chrono::Utc::now();
    }

    /// Add a custom field definition
    pub fn add_custom_field(&mut self, field_def: CustomFieldDefinition) {
        self.custom_fields.insert(field_def.name.clone(), field_def);
        self.touch();
    }

    /// Remove a custom field definition
    pub fn remove_custom_field(&mut self, field_name: &str) -> bool {
        let removed = self.custom_fields.remove(field_name).is_some();
        if removed {
            self.touch();
        }
        removed
    }

    /// Get a custom field definition by name
    pub fn get_custom_field(&self, field_name: &str) -> Option<&CustomFieldDefinition> {
        self.custom_fields.get(field_name)
    }

    /// Add a custom template
    pub fn add_template(&mut self, template: CredentialTemplate) {
        self.templates.push(template);
        self.touch();
    }

    /// Remove a template by name
    pub fn remove_template(&mut self, template_name: &str) -> bool {
        let original_len = self.templates.len();
        self.templates.retain(|t| t.name != template_name);
        let removed = self.templates.len() < original_len;
        if removed {
            self.touch();
        }
        removed
    }

    /// Get a template by name
    pub fn get_template(&self, template_name: &str) -> Option<&CredentialTemplate> {
        self.templates.iter().find(|t| t.name == template_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_repository_config() {
        let config = RepositoryConfig::default();

        assert_eq!(config.metadata.name, "ZipLock Repository");
        assert_eq!(config.metadata.format_version, "1.0");
        assert_eq!(config.security.min_password_length, 8);
        assert_eq!(config.security.encryption_method, "AES-256");
        assert!(config.validation.url_validation.validate_format);
        assert!(!config.behavior.auto_save.enabled);
        assert!(config.behavior.backup.enabled);
    }

    #[test]
    fn test_repository_config_new() {
        let config = RepositoryConfig::new("My Vault".to_string());
        assert_eq!(config.metadata.name, "My Vault");
    }

    #[test]
    fn test_repository_config_validation() {
        let mut config = RepositoryConfig::default();

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Empty name should fail
        config.metadata.name = "".to_string();
        assert!(config.validate().is_err());

        // Reset name
        config.metadata.name = "Test".to_string();

        // Low password length should fail
        config.security.min_password_length = 2;
        assert!(config.validate().is_err());

        // Reset password length
        config.security.min_password_length = 8;

        // Low iterations should fail
        config.security.iterations = 1000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_custom_field_management() {
        let mut config = RepositoryConfig::default();

        let field_def = CustomFieldDefinition {
            name: "ssn".to_string(),
            display_name: "Social Security Number".to_string(),
            description: Some("US Social Security Number".to_string()),
            base_type: FieldType::Text,
            is_sensitive: true,
            validation_rules: vec!["ssn_format".to_string()],
            input_mask: Some("###-##-####".to_string()),
            default_value: None,
        };

        config.add_custom_field(field_def.clone());
        assert_eq!(config.custom_fields.len(), 1);

        let retrieved = config.get_custom_field("ssn").unwrap();
        assert_eq!(retrieved.display_name, "Social Security Number");
        assert!(retrieved.is_sensitive);

        assert!(config.remove_custom_field("ssn"));
        assert_eq!(config.custom_fields.len(), 0);
        assert!(!config.remove_custom_field("nonexistent"));
    }

    #[test]
    fn test_serialization() {
        let config = RepositoryConfig::default();

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("metadata"));
        assert!(yaml.contains("security"));
        assert!(yaml.contains("validation"));

        let deserialized: RepositoryConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.metadata.name, deserialized.metadata.name);
        assert_eq!(
            config.security.min_password_length,
            deserialized.security.min_password_length
        );
    }

    #[test]
    fn test_validation_severity() {
        let severities = vec![
            ValidationSeverity::Error,
            ValidationSeverity::Warning,
            ValidationSeverity::Info,
        ];

        for severity in severities {
            let yaml = serde_yaml::to_string(&severity).unwrap();
            let deserialized: ValidationSeverity = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(severity, deserialized);
        }
    }

    #[test]
    fn test_conflict_resolution() {
        let strategies = vec![
            ConflictResolution::PreferLocal,
            ConflictResolution::PreferRemote,
            ConflictResolution::PreferNewer,
            ConflictResolution::PromptUser,
            ConflictResolution::CreateDuplicate,
        ];

        for strategy in strategies {
            let yaml = serde_yaml::to_string(&strategy).unwrap();
            let deserialized: ConflictResolution = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(strategy, deserialized);
        }
    }

    #[test]
    fn test_two_factor_methods() {
        let methods = vec![
            TwoFactorMethod::Totp,
            TwoFactorMethod::Sms,
            TwoFactorMethod::Email,
            TwoFactorMethod::SecurityKey,
            TwoFactorMethod::Biometric,
        ];

        for method in methods {
            let yaml = serde_yaml::to_string(&method).unwrap();
            let deserialized: TwoFactorMethod = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(method, deserialized);
        }
    }
}
