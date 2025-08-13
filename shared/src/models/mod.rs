//! Shared data models for ZipLock
//!
//! This module contains the core data structures used throughout the
//! ZipLock application, including credential records, field types,
//! and validation logic.

pub mod credential;
pub mod field;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

pub use credential::*;
pub use field::*;

/// A complete credential record as stored in the archive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CredentialRecord {
    /// Unique identifier for this credential
    pub id: String,

    /// Human-readable title/name for the credential
    pub title: String,

    /// Type of credential (login, credit_card, note, etc.)
    pub credential_type: String,

    /// Map of field names to field values
    pub fields: HashMap<String, CredentialField>,

    /// Tags for organization and searching
    pub tags: Vec<String>,

    /// Optional notes/description
    pub notes: Option<String>,

    /// When this credential was created
    pub created_at: SystemTime,

    /// When this credential was last modified
    pub updated_at: SystemTime,
}

/// A credential field that can hold different types of data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CredentialField {
    /// The field type (determines how it should be handled)
    pub field_type: FieldType,

    /// The actual value of the field
    pub value: String,

    /// Whether this field contains sensitive data (should be masked in UI)
    pub sensitive: bool,

    /// Optional label for display purposes
    pub label: Option<String>,

    /// Field-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Types of fields that can be stored in credentials
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// Plain text field
    Text,

    /// Password field (sensitive)
    Password,

    /// Email address
    Email,

    /// URL/website
    Url,

    /// Username/login
    Username,

    /// Phone number
    Phone,

    /// Credit card number
    CreditCardNumber,

    /// Credit card expiry date
    ExpiryDate,

    /// Credit card CVV
    Cvv,

    /// TOTP secret for 2FA
    TotpSecret,

    /// Large text field (notes, etc.)
    TextArea,

    /// Numeric field
    Number,

    /// Date field
    Date,

    /// Custom field type
    Custom(String),
}

/// Credential templates for common types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialTemplate {
    /// Template name
    pub name: String,

    /// Template description
    pub description: String,

    /// Default fields for this template
    pub fields: Vec<FieldTemplate>,

    /// Default tags to apply
    pub default_tags: Vec<String>,
}

/// Template for a field in a credential template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldTemplate {
    /// Field name/key
    pub name: String,

    /// Field type
    pub field_type: FieldType,

    /// Display label
    pub label: String,

    /// Whether this field is required
    pub required: bool,

    /// Whether this field is sensitive
    pub sensitive: bool,

    /// Default value (if any)
    pub default_value: Option<String>,

    /// Field validation rules
    pub validation: Option<FieldValidation>,
}

/// Validation rules for fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    /// Minimum length
    pub min_length: Option<usize>,

    /// Maximum length
    pub max_length: Option<usize>,

    /// Regular expression pattern
    pub pattern: Option<String>,

    /// Custom validation message
    pub message: Option<String>,
}

impl CredentialRecord {
    /// Create a new credential record with generated ID
    pub fn new(title: String, credential_type: String) -> Self {
        let now = SystemTime::now();

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            credential_type,
            fields: HashMap::new(),
            tags: Vec::new(),
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a credential from a template
    pub fn from_template(template: &CredentialTemplate, title: String) -> Self {
        let mut credential = Self::new(title, template.name.clone());

        // Add default fields from template
        for field_template in &template.fields {
            let field = CredentialField {
                field_type: field_template.field_type.clone(),
                value: field_template.default_value.clone().unwrap_or_default(),
                sensitive: field_template.sensitive,
                label: Some(field_template.label.clone()),
                metadata: HashMap::new(),
            };

            credential.fields.insert(field_template.name.clone(), field);
        }

        // Add default tags
        credential.tags = template.default_tags.clone();

        credential
    }

    /// Add or update a field
    pub fn set_field<S: Into<String>>(&mut self, name: S, field: CredentialField) {
        self.fields.insert(name.into(), field);
        self.updated_at = SystemTime::now();
    }

    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&CredentialField> {
        self.fields.get(name)
    }

    /// Remove a field
    pub fn remove_field(&mut self, name: &str) -> Option<CredentialField> {
        self.updated_at = SystemTime::now();
        self.fields.remove(name)
    }

    /// Add a tag if it doesn't already exist
    pub fn add_tag<S: Into<String>>(&mut self, tag: S) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = SystemTime::now();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = SystemTime::now();
            true
        } else {
            false
        }
    }

    /// Check if this credential has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Get all sensitive fields
    pub fn sensitive_fields(&self) -> Vec<(&String, &CredentialField)> {
        self.fields
            .iter()
            .filter(|(_, field)| field.sensitive)
            .collect()
    }

    /// Validate the credential record
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required fields
        if self.title.trim().is_empty() {
            errors.push("Title cannot be empty".to_string());
        }

        if self.credential_type.trim().is_empty() {
            errors.push("Credential type cannot be empty".to_string());
        }

        if self.id.trim().is_empty() {
            errors.push("ID cannot be empty".to_string());
        }

        // Validate individual fields
        for (name, field) in &self.fields {
            if let Err(field_errors) = field.validate() {
                for error in field_errors {
                    errors.push(format!("Field '{name}': {error}"));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Create a sanitized version for search/display (removes sensitive data)
    pub fn sanitized(&self) -> Self {
        let mut sanitized = self.clone();

        for field in sanitized.fields.values_mut() {
            if field.sensitive {
                field.value = "***".to_string();
            }
        }

        sanitized
    }
}

impl CredentialField {
    /// Create a new text field
    pub fn text<S: Into<String>>(value: S) -> Self {
        Self {
            field_type: FieldType::Text,
            value: value.into(),
            sensitive: false,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new password field
    pub fn password<S: Into<String>>(value: S) -> Self {
        Self {
            field_type: FieldType::Password,
            value: value.into(),
            sensitive: true,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new email field
    pub fn email<S: Into<String>>(value: S) -> Self {
        Self {
            field_type: FieldType::Email,
            value: value.into(),
            sensitive: false,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new URL field
    pub fn url<S: Into<String>>(value: S) -> Self {
        Self {
            field_type: FieldType::Url,
            value: value.into(),
            sensitive: false,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new username field
    pub fn username<S: Into<String>>(value: S) -> Self {
        Self {
            field_type: FieldType::Username,
            value: value.into(),
            sensitive: false,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new TOTP secret field
    pub fn totp_secret<S: Into<String>>(value: S) -> Self {
        Self {
            field_type: FieldType::TotpSecret,
            value: value.into(),
            sensitive: true,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the label for this field
    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set whether this field is sensitive
    pub fn with_sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    /// Add metadata to this field
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Validate this field
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Basic validation based on field type
        match &self.field_type {
            FieldType::Email => {
                if !self.value.is_empty() && !self.value.contains('@') {
                    errors.push("Invalid email format".to_string());
                }
            }
            FieldType::Url => {
                if !self.value.is_empty() && !self.value.starts_with("http") {
                    errors.push("URL should start with http:// or https://".to_string());
                }
            }
            FieldType::CreditCardNumber => {
                if !self.value.is_empty()
                    && self.value.chars().filter(|c| c.is_ascii_digit()).count() < 13
                {
                    errors.push("Credit card number should have at least 13 digits".to_string());
                }
            }
            _ => {} // No specific validation for other types
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get a display-safe version of the value
    pub fn display_value(&self) -> String {
        if self.sensitive {
            "***".to_string()
        } else {
            self.value.clone()
        }
    }
}

impl Default for CredentialField {
    fn default() -> Self {
        Self {
            field_type: FieldType::Text,
            value: String::new(),
            sensitive: false,
            label: None,
            metadata: HashMap::new(),
        }
    }
}

impl FieldType {
    /// Get all built-in field types
    pub fn built_in_types() -> Vec<FieldType> {
        vec![
            FieldType::Text,
            FieldType::Password,
            FieldType::Email,
            FieldType::Url,
            FieldType::Username,
            FieldType::Phone,
            FieldType::CreditCardNumber,
            FieldType::ExpiryDate,
            FieldType::Cvv,
            FieldType::TotpSecret,
            FieldType::TextArea,
            FieldType::Number,
            FieldType::Date,
        ]
    }

    /// Get the display name for this field type
    pub fn display_name(&self) -> &str {
        match self {
            FieldType::Text => "Text",
            FieldType::Password => "Password",
            FieldType::Email => "Email",
            FieldType::Url => "URL",
            FieldType::Username => "Username",
            FieldType::Phone => "Phone",
            FieldType::CreditCardNumber => "Credit Card Number",
            FieldType::ExpiryDate => "Expiry Date",
            FieldType::Cvv => "CVV",
            FieldType::TotpSecret => "TOTP Secret",
            FieldType::TextArea => "Text Area",
            FieldType::Number => "Number",
            FieldType::Date => "Date",
            FieldType::Custom(name) => name,
        }
    }

    /// Check if this field type typically contains sensitive data
    pub fn is_sensitive_by_default(&self) -> bool {
        matches!(
            self,
            FieldType::Password
                | FieldType::CreditCardNumber
                | FieldType::Cvv
                | FieldType::TotpSecret
        )
    }
}

/// Common credential templates
pub struct CommonTemplates;

impl CommonTemplates {
    /// Login credential template
    pub fn login() -> CredentialTemplate {
        CredentialTemplate {
            name: "login".to_string(),
            description: "Website or application login".to_string(),
            fields: vec![
                FieldTemplate {
                    name: "username".to_string(),
                    field_type: FieldType::Username,
                    label: "Username".to_string(),
                    required: false,
                    sensitive: false,
                    default_value: None,
                    validation: None,
                },
                FieldTemplate {
                    name: "password".to_string(),
                    field_type: FieldType::Password,
                    label: "Password".to_string(),
                    required: false,
                    sensitive: true,
                    default_value: None,
                    validation: Some(FieldValidation {
                        min_length: Some(8),
                        max_length: None,
                        pattern: None,
                        message: Some("Password must be at least 8 characters".to_string()),
                    }),
                },
                FieldTemplate {
                    name: "website".to_string(),
                    field_type: FieldType::Url,
                    label: "Website".to_string(),
                    required: false,
                    sensitive: false,
                    default_value: None,
                    validation: None,
                },
                FieldTemplate {
                    name: "totp".to_string(),
                    field_type: FieldType::TotpSecret,
                    label: "2FA Secret".to_string(),
                    required: false,
                    sensitive: true,
                    default_value: None,
                    validation: None,
                },
            ],
            default_tags: vec!["login".to_string()],
        }
    }

    /// Credit card template
    pub fn credit_card() -> CredentialTemplate {
        CredentialTemplate {
            name: "credit_card".to_string(),
            description: "Credit card information".to_string(),
            fields: vec![
                FieldTemplate {
                    name: "cardholder".to_string(),
                    field_type: FieldType::Text,
                    label: "Cardholder Name".to_string(),
                    required: false,
                    sensitive: false,
                    default_value: None,
                    validation: None,
                },
                FieldTemplate {
                    name: "number".to_string(),
                    field_type: FieldType::CreditCardNumber,
                    label: "Card Number".to_string(),
                    required: false,
                    sensitive: true,
                    default_value: None,
                    validation: None,
                },
                FieldTemplate {
                    name: "expiry".to_string(),
                    field_type: FieldType::ExpiryDate,
                    label: "Expiry Date".to_string(),
                    required: false,
                    sensitive: false,
                    default_value: None,
                    validation: None,
                },
                FieldTemplate {
                    name: "cvv".to_string(),
                    field_type: FieldType::Cvv,
                    label: "CVV".to_string(),
                    required: false,
                    sensitive: true,
                    default_value: None,
                    validation: None,
                },
            ],
            default_tags: vec!["credit_card".to_string()],
        }
    }

    /// Secure note template
    pub fn secure_note() -> CredentialTemplate {
        CredentialTemplate {
            name: "secure_note".to_string(),
            description: "Secure note or document".to_string(),
            fields: vec![FieldTemplate {
                name: "content".to_string(),
                field_type: FieldType::TextArea,
                label: "Content".to_string(),
                required: false,
                sensitive: true,
                default_value: None,
                validation: None,
            }],
            default_tags: vec!["note".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_record_creation() {
        let cred = CredentialRecord::new("Test Login".to_string(), "login".to_string());

        assert_eq!(cred.title, "Test Login");
        assert_eq!(cred.credential_type, "login");
        assert!(!cred.id.is_empty());
        assert!(cred.fields.is_empty());
        assert!(cred.tags.is_empty());
    }

    #[test]
    fn test_credential_field_creation() {
        let field = CredentialField::password("secret123")
            .with_label("Password")
            .with_metadata("strength", "strong");

        assert_eq!(field.value, "secret123");
        assert_eq!(field.field_type, FieldType::Password);
        assert!(field.sensitive);
        assert_eq!(field.label, Some("Password".to_string()));
        assert_eq!(field.metadata.get("strength"), Some(&"strong".to_string()));
    }

    #[test]
    fn test_credential_operations() {
        let mut cred = CredentialRecord::new("Test".to_string(), "login".to_string());

        // Add field
        cred.set_field("username", CredentialField::username("testuser"));
        assert!(cred.get_field("username").is_some());

        // Add tag
        cred.add_tag("important");
        assert!(cred.has_tag("important"));

        // Remove tag
        assert!(cred.remove_tag("important"));
        assert!(!cred.has_tag("important"));

        // Remove field
        assert!(cred.remove_field("username").is_some());
        assert!(cred.get_field("username").is_none());
    }

    #[test]
    fn test_field_validation() {
        // Valid email
        let email_field = CredentialField::email("test@example.com");
        assert!(email_field.validate().is_ok());

        // Invalid email
        let invalid_email = CredentialField::email("invalid-email");
        assert!(invalid_email.validate().is_err());

        // Valid URL
        let url_field = CredentialField::url("https://example.com");
        assert!(url_field.validate().is_ok());

        // Invalid URL
        let invalid_url = CredentialField::url("not-a-url");
        assert!(invalid_url.validate().is_err());
    }

    #[test]
    fn test_credential_from_template() {
        let template = CommonTemplates::login();
        let cred = CredentialRecord::from_template(&template, "GitHub Login".to_string());

        assert_eq!(cred.title, "GitHub Login");
        assert_eq!(cred.credential_type, "login");
        assert!(cred.get_field("username").is_some());
        assert!(cred.get_field("password").is_some());
        assert!(cred.get_field("website").is_some());
        assert!(cred.has_tag("login"));
    }

    #[test]
    fn test_sensitive_field_handling() {
        let mut cred = CredentialRecord::new("Test".to_string(), "login".to_string());
        cred.set_field("password", CredentialField::password("secret"));
        cred.set_field("username", CredentialField::username("user"));

        let sensitive_fields = cred.sensitive_fields();
        assert_eq!(sensitive_fields.len(), 1);
        assert_eq!(sensitive_fields[0].0, "password");

        let sanitized = cred.sanitized();
        assert_eq!(sanitized.get_field("password").unwrap().value, "***");
        assert_eq!(sanitized.get_field("username").unwrap().value, "user");
    }
}
