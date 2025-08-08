//! Field-related types and utilities for ZipLock credentials
//!
//! This module provides additional field-specific functionality
//! beyond the basic field types defined in the main models module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CredentialField, FieldType};

/// Field generator for creating common field types
pub struct FieldBuilder;

impl FieldBuilder {
    /// Create a new text field with optional configuration
    pub fn text() -> TextFieldBuilder {
        TextFieldBuilder::new()
    }

    /// Create a new password field with optional configuration
    pub fn password() -> PasswordFieldBuilder {
        PasswordFieldBuilder::new()
    }

    /// Create a new email field
    pub fn email() -> EmailFieldBuilder {
        EmailFieldBuilder::new()
    }

    /// Create a new URL field
    pub fn url() -> UrlFieldBuilder {
        UrlFieldBuilder::new()
    }
}

/// Builder for text fields
pub struct TextFieldBuilder {
    value: String,
    label: Option<String>,
    sensitive: bool,
    metadata: HashMap<String, String>,
}

impl TextFieldBuilder {
    fn new() -> Self {
        Self {
            value: String::new(),
            label: None,
            sensitive: false,
            metadata: HashMap::new(),
        }
    }

    pub fn value<S: Into<String>>(mut self, value: S) -> Self {
        self.value = value.into();
        self
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    pub fn metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> CredentialField {
        CredentialField {
            field_type: FieldType::Text,
            value: self.value,
            sensitive: self.sensitive,
            label: self.label,
            metadata: self.metadata,
        }
    }
}

/// Builder for password fields
pub struct PasswordFieldBuilder {
    value: String,
    label: Option<String>,
    metadata: HashMap<String, String>,
}

impl PasswordFieldBuilder {
    fn new() -> Self {
        Self {
            value: String::new(),
            label: None,
            metadata: HashMap::new(),
        }
    }

    pub fn value<S: Into<String>>(mut self, value: S) -> Self {
        self.value = value.into();
        self
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> CredentialField {
        CredentialField {
            field_type: FieldType::Password,
            value: self.value,
            sensitive: true, // Always sensitive for passwords
            label: self.label,
            metadata: self.metadata,
        }
    }
}

/// Builder for email fields
pub struct EmailFieldBuilder {
    value: String,
    label: Option<String>,
    metadata: HashMap<String, String>,
}

impl EmailFieldBuilder {
    fn new() -> Self {
        Self {
            value: String::new(),
            label: None,
            metadata: HashMap::new(),
        }
    }

    pub fn value<S: Into<String>>(mut self, value: S) -> Self {
        self.value = value.into();
        self
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> CredentialField {
        CredentialField {
            field_type: FieldType::Email,
            value: self.value,
            sensitive: false,
            label: self.label,
            metadata: self.metadata,
        }
    }
}

/// Builder for URL fields
pub struct UrlFieldBuilder {
    value: String,
    label: Option<String>,
    metadata: HashMap<String, String>,
}

impl UrlFieldBuilder {
    fn new() -> Self {
        Self {
            value: String::new(),
            label: None,
            metadata: HashMap::new(),
        }
    }

    pub fn value<S: Into<String>>(mut self, value: S) -> Self {
        self.value = value.into();
        self
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> CredentialField {
        CredentialField {
            field_type: FieldType::Url,
            value: self.value,
            sensitive: false,
            label: self.label,
            metadata: self.metadata,
        }
    }
}

/// Utility functions for field manipulation
pub struct FieldUtils;

impl FieldUtils {
    /// Sanitize a field value for logging or display
    pub fn sanitize_for_log(field: &CredentialField) -> String {
        if field.sensitive {
            format!("[{}]", field.field_type.display_name())
        } else if field.value.len() > 50 {
            format!("{}...", &field.value[..47])
        } else {
            field.value.clone()
        }
    }

    /// Get field strength indicator for passwords
    pub fn password_strength(password: &str) -> PasswordStrength {
        let length = password.len();
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digits = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let criteria_met = [has_lowercase, has_uppercase, has_digits, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        match (length, criteria_met) {
            (0..=7, _) => PasswordStrength::VeryWeak,
            (8..=11, 0..=1) => PasswordStrength::Weak,
            (8..=11, 2..=3) => PasswordStrength::Fair,
            (8..=11, 4) => PasswordStrength::Good,
            (12.., 0..=1) => PasswordStrength::Weak,
            (12.., 2) => PasswordStrength::Fair,
            (12.., 3) => PasswordStrength::Good,
            (12.., 4) => PasswordStrength::Strong,
            _ => PasswordStrength::VeryWeak,
        }
    }

    /// Validate email format
    pub fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.len() > 3 && !email.starts_with('@') && !email.ends_with('@')
    }

    /// Validate URL format
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Format credit card number for display (mask middle digits)
    pub fn format_credit_card_for_display(card_number: &str) -> String {
        let digits: String = card_number.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 4 {
            return "****".to_string();
        }

        let visible_start = if digits.len() <= 8 { 2 } else { 4 };
        let visible_end = 4;

        if digits.len() <= visible_start + visible_end {
            return "*".repeat(digits.len());
        }

        let start = &digits[..visible_start];
        let end = &digits[digits.len() - visible_end..];
        let middle_length = digits.len() - visible_start - visible_end;

        format!("{} {} {}", start, "*".repeat(middle_length), end)
    }
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrength {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "Very Weak",
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Fair => "Fair",
            PasswordStrength::Good => "Good",
            PasswordStrength::Strong => "Strong",
        }
    }

    /// Get a color indicator for UI display
    pub fn color(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "#ff4444", // Red
            PasswordStrength::Weak => "#ff8800",     // Orange
            PasswordStrength::Fair => "#ffbb00",     // Yellow
            PasswordStrength::Good => "#88bb00",     // Light Green
            PasswordStrength::Strong => "#44bb44",   // Green
        }
    }

    /// Get a score from 0-100
    pub fn score(&self) -> u8 {
        match self {
            PasswordStrength::VeryWeak => 20,
            PasswordStrength::Weak => 40,
            PasswordStrength::Fair => 60,
            PasswordStrength::Good => 80,
            PasswordStrength::Strong => 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_builders() {
        let text_field = FieldBuilder::text()
            .value("Test Value")
            .label("Test Label")
            .sensitive(true)
            .metadata("key", "value")
            .build();

        assert_eq!(text_field.value, "Test Value");
        assert_eq!(text_field.label, Some("Test Label".to_string()));
        assert!(text_field.sensitive);
        assert_eq!(text_field.metadata.get("key"), Some(&"value".to_string()));

        let password_field = FieldBuilder::password()
            .value("secret123")
            .label("Password")
            .build();

        assert_eq!(password_field.value, "secret123");
        assert!(password_field.sensitive); // Should always be true for passwords
        assert_eq!(password_field.field_type, FieldType::Password);
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(
            FieldUtils::password_strength("weak"),
            PasswordStrength::VeryWeak
        );
        assert_eq!(
            FieldUtils::password_strength("password123"),
            PasswordStrength::Fair
        );
        assert_eq!(
            FieldUtils::password_strength("Password123!"),
            PasswordStrength::Strong
        );
        assert_eq!(
            FieldUtils::password_strength("SuperSecure123!@#"),
            PasswordStrength::Strong
        );
    }

    #[test]
    fn test_email_validation() {
        assert!(FieldUtils::is_valid_email("test@example.com"));
        assert!(FieldUtils::is_valid_email("user@domain.org"));
        assert!(!FieldUtils::is_valid_email("invalid"));
        assert!(!FieldUtils::is_valid_email("@domain.com"));
        assert!(!FieldUtils::is_valid_email("user@"));
    }

    #[test]
    fn test_url_validation() {
        assert!(FieldUtils::is_valid_url("https://example.com"));
        assert!(FieldUtils::is_valid_url("http://example.com"));
        assert!(!FieldUtils::is_valid_url("ftp://example.com"));
        assert!(!FieldUtils::is_valid_url("example.com"));
    }

    #[test]
    fn test_credit_card_formatting() {
        assert_eq!(
            FieldUtils::format_credit_card_for_display("1234567890123456"),
            "1234 ******** 3456"
        );
        assert_eq!(
            FieldUtils::format_credit_card_for_display("1234-5678-9012-3456"),
            "1234 ******** 3456"
        );
        assert_eq!(FieldUtils::format_credit_card_for_display("1234"), "****");
    }

    #[test]
    fn test_field_sanitization() {
        let sensitive_field = CredentialField::password("secret123");
        let sanitized = FieldUtils::sanitize_for_log(&sensitive_field);
        assert_eq!(sanitized, "[Password]");

        let long_field = CredentialField::text("a".repeat(100));
        let sanitized = FieldUtils::sanitize_for_log(&long_field);
        assert!(sanitized.ends_with("..."));
        assert!(sanitized.len() <= 50);
    }

    #[test]
    fn test_password_strength_properties() {
        let strength = PasswordStrength::Strong;
        assert_eq!(strength.description(), "Strong");
        assert_eq!(strength.score(), 100);
        assert!(!strength.color().is_empty());
    }
}
