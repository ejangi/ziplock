//! Credential template system for ZipLock
//!
//! This module provides template functionality for creating standardized
//! credential types with predefined fields and validation rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CredentialField, CredentialRecord, FieldType};
use crate::core::types::{MAX_FIELDS_PER_CREDENTIAL, MAX_TAGS_PER_CREDENTIAL};

/// Template for creating credentials with predefined structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CredentialTemplate {
    /// Template name
    pub name: String,

    /// Template description
    pub description: String,

    /// Field templates for this credential type
    pub fields: Vec<FieldTemplate>,

    /// Default tags to apply
    pub default_tags: Vec<String>,
}

/// Template for individual fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldTemplate {
    /// Field name/identifier
    pub name: String,

    /// Display label for the field
    pub label: String,

    /// Field type
    pub field_type: FieldType,

    /// Whether this field is required
    pub required: bool,

    /// Whether this field is sensitive
    pub sensitive: bool,

    /// Default value for the field
    pub default_value: Option<String>,

    /// Validation rules for the field
    pub validation: Option<FieldValidation>,
}

/// Validation rules for fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValidation {
    /// Minimum length requirement
    pub min_length: Option<usize>,

    /// Maximum length requirement
    pub max_length: Option<usize>,

    /// Regex pattern for validation
    pub pattern: Option<String>,

    /// Custom validation message
    pub message: Option<String>,
}

impl CredentialTemplate {
    /// Create a new credential template
    pub fn new<S: Into<String>>(name: S, description: S) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            fields: Vec::new(),
            default_tags: Vec::new(),
        }
    }

    /// Add a field template to this credential template
    pub fn add_field(&mut self, field: FieldTemplate) -> Result<(), String> {
        if self.fields.len() >= MAX_FIELDS_PER_CREDENTIAL {
            return Err(format!(
                "Cannot add more than {} fields to a template",
                MAX_FIELDS_PER_CREDENTIAL
            ));
        }

        // Check for duplicate field names
        if self.fields.iter().any(|f| f.name == field.name) {
            return Err(format!("Field '{}' already exists in template", field.name));
        }

        self.fields.push(field);
        Ok(())
    }

    /// Add a default tag
    pub fn add_tag<S: Into<String>>(&mut self, tag: S) -> Result<(), String> {
        if self.default_tags.len() >= MAX_TAGS_PER_CREDENTIAL {
            return Err(format!(
                "Cannot add more than {} tags to a template",
                MAX_TAGS_PER_CREDENTIAL
            ));
        }

        let tag = tag.into();
        if !self.default_tags.contains(&tag) {
            self.default_tags.push(tag);
        }
        Ok(())
    }

    /// Create a credential record from this template
    pub fn create_credential(&self, title: String) -> Result<CredentialRecord, String> {
        let mut credential = CredentialRecord::new(title, self.name.clone());

        // Add default tags
        for tag in &self.default_tags {
            credential.add_tag(tag.clone());
        }

        // Add fields from template
        for field_template in &self.fields {
            let field = CredentialField {
                field_type: field_template.field_type.clone(),
                value: field_template.default_value.clone().unwrap_or_default(),
                sensitive: field_template.sensitive,
                label: Some(field_template.label.clone()),
                metadata: HashMap::new(),
            };

            credential.set_field(&field_template.name, field);
        }

        Ok(credential)
    }

    /// Validate that a credential matches this template
    pub fn validate_credential(&self, credential: &CredentialRecord) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required fields
        for field_template in &self.fields {
            if field_template.required {
                match credential.get_field(&field_template.name) {
                    Some(field) if field.value.is_empty() => {
                        errors.push(format!(
                            "Required field '{}' is empty",
                            field_template.label
                        ));
                    }
                    None => {
                        errors.push(format!(
                            "Required field '{}' is missing",
                            field_template.label
                        ));
                    }
                    Some(field) => {
                        // Validate field if present
                        if let Some(validation) = &field_template.validation {
                            if let Err(validation_error) = validation.validate(&field.value) {
                                errors.push(format!(
                                    "Field '{}': {}",
                                    field_template.label, validation_error
                                ));
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Convert template name to display-friendly format
    ///
    /// Converts snake_case and kebab-case names to Title Case
    /// Examples: "credit_card" -> "Credit Card", "ssh-key" -> "SSH Key"
    pub fn to_display_name(&self) -> String {
        self.name
            .replace('_', " ")
            .replace('-', " ")
            .split_whitespace()
            .map(|word| {
                if word.to_uppercase() == word && word.len() <= 4 {
                    // Keep acronyms like "SSH", "API" uppercase
                    word.to_uppercase()
                } else {
                    // Title case for regular words
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first
                            .to_uppercase()
                            .chain(chars.as_str().to_lowercase().chars())
                            .collect(),
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Get field template by name
    pub fn get_field_template(&self, name: &str) -> Option<&FieldTemplate> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get required field names
    pub fn required_fields(&self) -> Vec<&str> {
        self.fields
            .iter()
            .filter(|f| f.required)
            .map(|f| f.name.as_str())
            .collect()
    }
}

impl FieldTemplate {
    /// Create a new field template
    pub fn new<S: Into<String>>(name: S, label: S, field_type: FieldType, required: bool) -> Self {
        let is_sensitive = field_type.is_sensitive_by_default();
        Self {
            name: name.into(),
            label: label.into(),
            field_type,
            required,
            sensitive: is_sensitive,
            default_value: None,
            validation: Some(FieldValidation::new()),
        }
    }

    /// Set whether this field is sensitive
    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    /// Set a default value for this field
    pub fn default_value<S: Into<String>>(mut self, value: S) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set validation rules for this field
    pub fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = Some(validation);
        self
    }
}

impl FieldValidation {
    /// Create a new field validation
    pub fn new() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            message: None,
        }
    }

    /// Set minimum length requirement
    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = Some(length);
        self
    }

    /// Set maximum length requirement
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    /// Set regex pattern requirement
    pub fn pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set custom validation message
    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Validate a value against these rules
    pub fn validate(&self, value: &str) -> Result<(), String> {
        if let Some(min_len) = self.min_length {
            if value.len() < min_len {
                return Err(self
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Must be at least {} characters long", min_len)));
            }
        }

        if let Some(max_len) = self.max_length {
            if value.len() > max_len {
                return Err(self.message.clone().unwrap_or_else(|| {
                    format!("Must be no more than {} characters long", max_len)
                }));
            }
        }

        if let Some(pattern) = &self.pattern {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(value) {
                    return Err(self
                        .message
                        .clone()
                        .unwrap_or_else(|| "Invalid format".to_string()));
                }
            }
        }

        Ok(())
    }
}

/// Common credential templates
pub struct CommonTemplates;

impl CommonTemplates {
    /// Login credential template
    pub fn login() -> CredentialTemplate {
        let mut template = CredentialTemplate::new(
            "login",
            "Standard login credentials with username and password",
        );

        template
            .add_field(
                FieldTemplate::new("username", "Username", FieldType::Username, true)
                    .validation(FieldValidation::new().min_length(1)),
            )
            .unwrap();

        template
            .add_field(
                FieldTemplate::new("password", "Password", FieldType::Password, true)
                    .validation(FieldValidation::new().min_length(1)),
            )
            .unwrap();

        template
            .add_field(FieldTemplate::new("url", "Website", FieldType::Url, false))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "totp_secret",
                "TOTP Secret",
                FieldType::TotpSecret,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "notes",
                "Notes",
                FieldType::TextArea,
                false,
            ))
            .unwrap();

        template.add_tag("login").unwrap();

        template
    }

    /// Credit card template
    pub fn credit_card() -> CredentialTemplate {
        let mut template = CredentialTemplate::new(
            "credit_card",
            "Credit card information with security details",
        );

        template
            .add_field(
                FieldTemplate::new("cardholder", "Cardholder Name", FieldType::Text, true)
                    .validation(FieldValidation::new().min_length(1)),
            )
            .unwrap();

        template
            .add_field(
                FieldTemplate::new("number", "Card Number", FieldType::CreditCardNumber, true)
                    .validation(
                        FieldValidation::new()
                            .pattern(r"^\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}$")
                            .message("Invalid card number format"),
                    ),
            )
            .unwrap();

        template
            .add_field(
                FieldTemplate::new("expiry", "Expiry Date", FieldType::ExpiryDate, true)
                    .validation(
                        FieldValidation::new()
                            .pattern(r"^\d{2}/\d{2}$")
                            .message("Use MM/YY format"),
                    ),
            )
            .unwrap();

        template
            .add_field(
                FieldTemplate::new("cvv", "CVV", FieldType::Cvv, true).validation(
                    FieldValidation::new()
                        .pattern(r"^\d{3,4}$")
                        .message("CVV must be 3-4 digits"),
                ),
            )
            .unwrap();

        template.add_tag("payment").unwrap();
        template.add_tag("credit_card").unwrap();

        template
    }

    /// Secure note template
    pub fn secure_note() -> CredentialTemplate {
        let mut template =
            CredentialTemplate::new("secure_note", "Secure text note for sensitive information");

        template
            .add_field(FieldTemplate::new(
                "content",
                "Content",
                FieldType::TextArea,
                true,
            ))
            .unwrap();

        template.add_tag("note").unwrap();

        template
    }

    /// Identity template
    pub fn identity() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("identity", "Personal identity information");

        template
            .add_field(FieldTemplate::new(
                "first_name",
                "First Name",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "last_name",
                "Last Name",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "email",
                "Email",
                FieldType::Email,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "phone",
                "Phone",
                FieldType::Phone,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "address",
                "Address",
                FieldType::TextArea,
                false,
            ))
            .unwrap();

        template.add_tag("identity").unwrap();

        template
    }

    /// Wi-Fi credentials template
    pub fn wifi() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("wifi", "Wi-Fi network credentials");

        template
            .add_field(FieldTemplate::new(
                "ssid",
                "Network Name (SSID)",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "password",
                "Password",
                FieldType::Password,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "security",
                "Security Type",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template.add_tag("wifi").unwrap();
        template.add_tag("network").unwrap();

        template
    }

    /// Database connection template
    pub fn database() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("database", "Database connection credentials");

        template
            .add_field(FieldTemplate::new("host", "Host", FieldType::Text, true))
            .unwrap();

        template
            .add_field(FieldTemplate::new("port", "Port", FieldType::Number, false))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "database",
                "Database Name",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "username",
                "Username",
                FieldType::Username,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "password",
                "Password",
                FieldType::Password,
                true,
            ))
            .unwrap();

        template.add_tag("database").unwrap();
        template.add_tag("server").unwrap();

        template
    }

    /// API credentials template
    pub fn api_key() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("api_key", "API key and credentials");

        template
            .add_field(FieldTemplate::new(
                "service",
                "Service Name",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "api_key",
                "API Key",
                FieldType::Password,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "secret",
                "API Secret",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "endpoint",
                "API Endpoint",
                FieldType::Url,
                false,
            ))
            .unwrap();

        template.add_tag("api").unwrap();
        template.add_tag("developer").unwrap();

        template
    }

    /// Password storage template
    pub fn password() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("password", "Standalone password storage");

        template
            .add_field(FieldTemplate::new(
                "password",
                "Password",
                FieldType::Password,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "url",
                "Associated URL",
                FieldType::Url,
                false,
            ))
            .unwrap();

        template.add_tag("password").unwrap();

        template
    }

    /// Document/file template
    pub fn document() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("document", "Document or file credentials");

        template
            .add_field(FieldTemplate::new(
                "filename",
                "Filename",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "password",
                "Document Password",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "location",
                "File Location",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template.add_tag("document").unwrap();
        template.add_tag("file").unwrap();

        template
    }

    /// SSH key template
    pub fn ssh_key() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("ssh_key", "SSH key credentials");

        template
            .add_field(FieldTemplate::new(
                "username",
                "Username",
                FieldType::Username,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "hostname",
                "Hostname/Server",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "private_key",
                "Private Key",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "passphrase",
                "Key Passphrase",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template.add_tag("ssh").unwrap();
        template.add_tag("server").unwrap();

        template
    }

    /// Bank account template
    pub fn bank_account() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("bank_account", "Bank account credentials");

        template
            .add_field(FieldTemplate::new(
                "account_number",
                "Account Number",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "routing_number",
                "Routing Number",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "bank_name",
                "Bank Name",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new("pin", "PIN", FieldType::Password, false))
            .unwrap();

        template.add_tag("bank").unwrap();
        template.add_tag("finance").unwrap();

        template
    }

    /// API credentials template (different from api_key)
    pub fn api_credentials() -> CredentialTemplate {
        let mut template =
            CredentialTemplate::new("api_credentials", "API authentication credentials");

        template
            .add_field(FieldTemplate::new(
                "api_key",
                "API Key",
                FieldType::Password,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "secret_key",
                "Secret Key",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "client_id",
                "Client ID",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "endpoint",
                "API Endpoint",
                FieldType::Url,
                false,
            ))
            .unwrap();

        template.add_tag("api").unwrap();
        template.add_tag("credentials").unwrap();

        template
    }

    /// Cryptocurrency wallet template
    pub fn crypto_wallet() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("crypto_wallet", "Cryptocurrency wallet");

        template
            .add_field(FieldTemplate::new(
                "wallet_address",
                "Wallet Address",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "private_key",
                "Private Key",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "seed_phrase",
                "Seed Phrase",
                FieldType::Password,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "currency",
                "Currency",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template.add_tag("crypto").unwrap();
        template.add_tag("wallet").unwrap();
        template.add_tag("cryptocurrency").unwrap();

        template
    }

    /// Software license template
    pub fn software_license() -> CredentialTemplate {
        let mut template = CredentialTemplate::new("software_license", "Software license key");

        template
            .add_field(FieldTemplate::new(
                "software_name",
                "Software Name",
                FieldType::Text,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "license_key",
                "License Key",
                FieldType::Password,
                true,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "version",
                "Software Version",
                FieldType::Text,
                false,
            ))
            .unwrap();

        template
            .add_field(FieldTemplate::new(
                "email",
                "Registered Email",
                FieldType::Email,
                false,
            ))
            .unwrap();

        template.add_tag("software").unwrap();
        template.add_tag("license").unwrap();

        template
    }

    /// Get all common templates
    pub fn all() -> Vec<CredentialTemplate> {
        vec![
            Self::login(),
            Self::credit_card(),
            Self::secure_note(),
            Self::identity(),
            Self::wifi(),
            Self::database(),
            Self::api_key(),
            Self::password(),
            Self::document(),
            Self::ssh_key(),
            Self::bank_account(),
            Self::api_credentials(),
            Self::crypto_wallet(),
            Self::software_license(),
        ]
    }

    /// Get template by name
    pub fn get_by_name(name: &str) -> Option<CredentialTemplate> {
        Self::all().into_iter().find(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = CredentialTemplate::new("test", "Test template");
        assert_eq!(template.name, "test");
        assert_eq!(template.description, "Test template");
        assert!(template.fields.is_empty());
        assert!(template.default_tags.is_empty());
    }

    #[test]
    fn test_field_template_creation() {
        let field = FieldTemplate::new("username", "Username", FieldType::Username, true);
        assert_eq!(field.name, "username");
        assert_eq!(field.label, "Username");
        assert_eq!(field.field_type, FieldType::Username);
        assert!(field.required);
        assert!(!field.sensitive); // Username is not sensitive by default
    }

    #[test]
    fn test_template_field_addition() {
        let mut template = CredentialTemplate::new("login", "Login template");
        let field = FieldTemplate::new("username", "Username", FieldType::Username, true);

        assert!(template.add_field(field).is_ok());
        assert_eq!(template.fields.len(), 1);

        // Test duplicate field name
        let duplicate_field = FieldTemplate::new("username", "Username 2", FieldType::Text, false);
        assert!(template.add_field(duplicate_field).is_err());
    }

    #[test]
    fn test_credential_creation_from_template() {
        let template = CommonTemplates::login();
        let credential = template
            .create_credential("Test Login".to_string())
            .unwrap();

        assert_eq!(credential.title, "Test Login");
        assert_eq!(credential.credential_type, "login");
        assert!(credential.has_tag("login"));
        assert!(credential.get_field("username").is_some());
        assert!(credential.get_field("password").is_some());
        assert!(credential.get_field("totp_secret").is_some());
    }

    #[test]
    fn test_login_template_includes_totp_field() {
        let template = CommonTemplates::login();

        // Verify template has all expected fields
        assert_eq!(template.fields.len(), 5); // username, password, url, totp_secret, notes

        // Verify specific fields exist
        let field_names: Vec<&str> = template.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"username"));
        assert!(field_names.contains(&"password"));
        assert!(field_names.contains(&"url"));
        assert!(field_names.contains(&"totp_secret"));
        assert!(field_names.contains(&"notes"));

        // Verify TOTP field properties
        let totp_field = template
            .fields
            .iter()
            .find(|f| f.name == "totp_secret")
            .unwrap();
        assert_eq!(totp_field.field_type, FieldType::TotpSecret);
        assert_eq!(totp_field.label, "TOTP Secret");
        assert!(!totp_field.required); // TOTP should be optional
    }

    #[test]
    fn test_field_validation() {
        let validation = FieldValidation::new()
            .min_length(3)
            .max_length(10)
            .pattern(r"^[a-zA-Z]+$")
            .message("Custom error");

        assert!(validation.validate("hello").is_ok());
        assert!(validation.validate("hi").is_err()); // Too short
        assert!(validation.validate("verylongtext").is_err()); // Too long
        assert!(validation.validate("hello123").is_err()); // Invalid pattern
    }

    #[test]
    fn test_template_validation() {
        let template = CommonTemplates::login();
        let mut credential = template.create_credential("Test".to_string()).unwrap();

        // Should pass with filled required fields
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));
        assert!(template.validate_credential(&credential).is_ok());

        // Should fail with empty required field
        credential.set_field("username", CredentialField::username(""));
        assert!(template.validate_credential(&credential).is_err());
    }

    #[test]
    fn test_to_display_name() {
        let mut template = CredentialTemplate::new("credit_card", "Credit Card template");
        assert_eq!(template.to_display_name(), "Credit Card");

        template.name = "ssh_key".to_string();
        assert_eq!(template.to_display_name(), "Ssh Key");

        template.name = "api_credentials".to_string();
        assert_eq!(template.to_display_name(), "API Credentials");

        template.name = "secure-note".to_string();
        assert_eq!(template.to_display_name(), "Secure Note");

        template.name = "simple".to_string();
        assert_eq!(template.to_display_name(), "Simple");

        template.name = "multi_word_template".to_string();
        assert_eq!(template.to_display_name(), "Multi Word Template");
    }

    #[test]
    fn test_common_templates() {
        let templates = CommonTemplates::all();
        assert!(!templates.is_empty());

        let login_template = CommonTemplates::get_by_name("login").unwrap();
        assert_eq!(login_template.name, "login");

        let required_fields = login_template.required_fields();
        assert!(required_fields.contains(&"username"));
        assert!(required_fields.contains(&"password"));
    }

    #[test]
    fn test_credit_card_template() {
        let template = CommonTemplates::credit_card();
        let mut credential = template.create_credential("My Card".to_string()).unwrap();

        // Test valid card number
        credential.set_field(
            "number",
            CredentialField::new(
                FieldType::CreditCardNumber,
                "4532123456789012".to_string(),
                true,
            ),
        );
        credential.set_field("cardholder", CredentialField::text("John Doe"));
        credential.set_field(
            "expiry",
            CredentialField::new(FieldType::ExpiryDate, "12/25".to_string(), true),
        );
        credential.set_field(
            "cvv",
            CredentialField::new(FieldType::Cvv, "123".to_string(), true),
        );

        let validation_result = template.validate_credential(&credential);
        assert!(
            validation_result.is_ok(),
            "Validation failed: {:?}",
            validation_result
        );
    }
}
