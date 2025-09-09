//! Validation utilities for ZipLock
//!
//! This module provides comprehensive validation functions for credentials,
//! fields, and other data structures to ensure data integrity and security.

use regex::Regex;
use std::collections::HashSet;

use crate::core::types::{
    MAX_FIELDS_PER_CREDENTIAL, MAX_FIELD_VALUE_LENGTH, MAX_NOTES_LENGTH, MAX_TAGS_PER_CREDENTIAL,
    MAX_TAG_LENGTH, MAX_TITLE_LENGTH,
};
use crate::models::{CredentialField, CredentialRecord, FieldType};

/// Validation result with detailed error information
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with errors
    pub fn with_errors(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add an error to this validation result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning to this validation result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}

/// Validate a complete credential record
pub fn validate_credential(credential: &CredentialRecord) -> ValidationResult {
    let mut result = ValidationResult::success();

    // Validate basic fields
    result.merge(validate_credential_id(&credential.id));
    result.merge(validate_credential_title(&credential.title));
    result.merge(validate_credential_type(&credential.credential_type));
    if let Some(notes) = &credential.notes {
        result.merge(validate_credential_notes(notes));
    }
    result.merge(validate_credential_tags(&credential.tags));

    // Validate field count
    if credential.fields.len() > MAX_FIELDS_PER_CREDENTIAL {
        result.add_error(format!(
            "Too many fields: {} (maximum {})",
            credential.fields.len(),
            MAX_FIELDS_PER_CREDENTIAL
        ));
    }

    // Validate each field
    for (field_name, field) in &credential.fields {
        let field_result = validate_field(field_name, field);
        result.merge(field_result);
    }

    // Validate timestamps
    if credential.created_at <= 0 {
        result.add_error("Invalid created_at timestamp".to_string());
    }

    if credential.updated_at <= 0 {
        result.add_error("Invalid updated_at timestamp".to_string());
    }

    if credential.updated_at < credential.created_at {
        result.add_error("Updated timestamp cannot be before created timestamp".to_string());
    }

    // Check for duplicate field names (case-insensitive)
    let mut field_names_lower = HashSet::new();
    for field_name in credential.fields.keys() {
        let lower_name = field_name.to_lowercase();
        if !field_names_lower.insert(lower_name) {
            result.add_error(format!("Duplicate field name: {}", field_name));
        }
    }

    result
}

/// Validate a credential ID
pub fn validate_credential_id(id: &str) -> ValidationResult {
    let mut result = ValidationResult::success();

    if id.is_empty() {
        result.add_error("Credential ID cannot be empty".to_string());
    } else if id.len() > 100 {
        result.add_error(format!(
            "Credential ID too long: {} characters (maximum 100)",
            id.len()
        ));
    }

    // Check if it's a valid UUID format (optional but recommended)
    if !id.is_empty() {
        use uuid::Uuid;
        if Uuid::parse_str(id).is_err() {
            result.add_warning("Credential ID is not a valid UUID format".to_string());
        }
    }

    result
}

/// Generate a new UUID for a credential
pub fn generate_credential_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Repair a credential by ensuring it has a valid ID
pub fn repair_credential_id(credential: &mut CredentialRecord) -> bool {
    if credential.id.is_empty() {
        credential.id = generate_credential_id();
        true // Indicates the credential was repaired
    } else {
        false // No repair needed
    }
}

/// Validate a credential title
pub fn validate_credential_title(title: &str) -> ValidationResult {
    let mut result = ValidationResult::success();

    if title.is_empty() {
        result.add_error("Title cannot be empty".to_string());
    } else if title.len() > MAX_TITLE_LENGTH {
        result.add_error(format!(
            "Title too long: {} characters (maximum {})",
            title.len(),
            MAX_TITLE_LENGTH
        ));
    }

    // Check for control characters
    if title
        .chars()
        .any(|c| c.is_control() && c != '\t' && c != '\n')
    {
        result.add_error("Title contains invalid control characters".to_string());
    }

    // Warn about leading/trailing whitespace
    if title != title.trim() {
        result.add_warning("Title has leading or trailing whitespace".to_string());
    }

    result
}

/// Validate a credential type
pub fn validate_credential_type(credential_type: &str) -> ValidationResult {
    let mut result = ValidationResult::success();

    if credential_type.is_empty() {
        result.add_error("Credential type cannot be empty".to_string());
    } else if credential_type.len() > 50 {
        result.add_error("Credential type too long (maximum 50 characters)".to_string());
    }

    // Check for invalid characters
    if !credential_type
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        result.add_error(
            "Credential type can only contain letters, numbers, hyphens, and underscores"
                .to_string(),
        );
    }

    result
}

/// Validate credential notes
pub fn validate_credential_notes(notes: &str) -> ValidationResult {
    let mut result = ValidationResult::success();

    if notes.len() > MAX_NOTES_LENGTH {
        result.add_error(format!(
            "Notes too long: {} characters (maximum {})",
            notes.len(),
            MAX_NOTES_LENGTH
        ));
    }

    result
}

/// Validate credential tags
pub fn validate_credential_tags(tags: &[String]) -> ValidationResult {
    let mut result = ValidationResult::success();

    if tags.len() > MAX_TAGS_PER_CREDENTIAL {
        result.add_error(format!(
            "Too many tags: {} (maximum {})",
            tags.len(),
            MAX_TAGS_PER_CREDENTIAL
        ));
    }

    let mut unique_tags = HashSet::new();
    for tag in tags {
        // Check tag length
        if tag.len() > MAX_TAG_LENGTH {
            result.add_error(format!(
                "Tag too long: '{}' ({} characters, maximum {})",
                tag,
                tag.len(),
                MAX_TAG_LENGTH
            ));
        }

        // Check for empty tags
        if tag.trim().is_empty() {
            result.add_error("Empty tag found".to_string());
        }

        // Check for duplicates (case-insensitive)
        let tag_lower = tag.to_lowercase();
        if !unique_tags.insert(tag_lower) {
            result.add_error(format!("Duplicate tag: '{}'", tag));
        }

        // Check for invalid characters
        if tag.chars().any(|c| c.is_control()) {
            result.add_error(format!("Tag contains control characters: '{}'", tag));
        }
    }

    result
}

/// Validate a single field
pub fn validate_field(field_name: &str, field: &CredentialField) -> ValidationResult {
    let mut result = ValidationResult::success();

    // Validate field name
    if field_name.is_empty() {
        result.add_error("Field name cannot be empty".to_string());
    } else if field_name.len() > 100 {
        result.add_error(format!(
            "Field name too long: '{}' ({} characters, maximum 100)",
            field_name,
            field_name.len()
        ));
    }

    // Validate field value length
    if field.value.len() > MAX_FIELD_VALUE_LENGTH {
        result.add_error(format!(
            "Field '{}' value too long: {} characters (maximum {})",
            field_name,
            field.value.len(),
            MAX_FIELD_VALUE_LENGTH
        ));
    }

    // Type-specific validation
    result.merge(validate_field_by_type(field_name, field));

    // Validate field label if present
    if let Some(label) = &field.label {
        if label.len() > 200 {
            result.add_error(format!(
                "Field '{}' label too long: {} characters (maximum 200)",
                field_name,
                label.len()
            ));
        }
    }

    result
}

/// Validate field based on its type
pub fn validate_field_by_type(field_name: &str, field: &CredentialField) -> ValidationResult {
    let mut result = ValidationResult::success();

    match field.field_type {
        FieldType::Email => {
            if !field.value.is_empty() && !is_valid_email(&field.value) {
                result.add_error(format!(
                    "Field '{}' is not a valid email address",
                    field_name
                ));
            }
        }
        FieldType::Url => {
            if !field.value.is_empty() && !is_valid_url(&field.value) {
                result.add_error(format!("Field '{}' is not a valid URL", field_name));
            }
        }
        FieldType::Phone => {
            if !field.value.is_empty() && !is_valid_phone(&field.value) {
                result.add_warning(format!(
                    "Field '{}' may not be a valid phone number",
                    field_name
                ));
            }
        }
        FieldType::CreditCardNumber => {
            if !field.value.is_empty() && !is_valid_credit_card(&field.value) {
                result.add_error(format!(
                    "Field '{}' is not a valid credit card number",
                    field_name
                ));
            }
        }
        FieldType::ExpiryDate => {
            if !field.value.is_empty() && !is_valid_expiry_date(&field.value) {
                result.add_error(format!(
                    "Field '{}' is not a valid expiry date (use MM/YY format)",
                    field_name
                ));
            }
        }
        FieldType::Cvv => {
            if !field.value.is_empty() && !is_valid_cvv(&field.value) {
                result.add_error(format!("Field '{}' is not a valid CVV code", field_name));
            }
        }
        FieldType::TotpSecret => {
            if !field.value.is_empty() && !is_valid_totp_secret(&field.value) {
                result.add_error(format!("Field '{}' is not a valid TOTP secret", field_name));
            }
        }
        FieldType::Number => {
            if !field.value.is_empty() && field.value.parse::<f64>().is_err() {
                result.add_error(format!("Field '{}' is not a valid number", field_name));
            }
        }
        FieldType::Date => {
            if !field.value.is_empty() && !is_valid_date(&field.value) {
                result.add_error(format!("Field '{}' is not a valid date", field_name));
            }
        }
        _ => {
            // No specific validation for other field types
        }
    }

    result
}

/// Validate an email address
pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email) && email.len() <= 254
}

/// Validate a URL
pub fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Validate a phone number (basic validation)
pub fn is_valid_phone(phone: &str) -> bool {
    let phone_clean = phone.replace([' ', '-', '(', ')', '+'], "");
    phone_clean.len() >= 7 && phone_clean.chars().all(|c| c.is_ascii_digit())
}

/// Validate a credit card number using Luhn algorithm
pub fn is_valid_credit_card(card_number: &str) -> bool {
    let digits: String = card_number.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    // Luhn algorithm
    let mut sum = 0;
    let mut is_even = false;

    for digit_char in digits.chars().rev() {
        let mut digit = digit_char.to_digit(10).unwrap() as u32;

        if is_even {
            digit *= 2;
            if digit > 9 {
                digit = digit / 10 + digit % 10;
            }
        }

        sum += digit;
        is_even = !is_even;
    }

    sum % 10 == 0
}

/// Validate expiry date in MM/YY format
pub fn is_valid_expiry_date(expiry: &str) -> bool {
    let expiry_regex = Regex::new(r"^(0[1-9]|1[0-2])/[0-9]{2}$").unwrap();
    expiry_regex.is_match(expiry)
}

/// Validate CVV code
pub fn is_valid_cvv(cvv: &str) -> bool {
    cvv.len() >= 3 && cvv.len() <= 4 && cvv.chars().all(|c| c.is_ascii_digit())
}

/// Validate TOTP secret (base32)
pub fn is_valid_totp_secret(secret: &str) -> bool {
    let clean_secret = secret.replace(' ', "").to_uppercase();
    if clean_secret.is_empty() {
        return false;
    }

    clean_secret
        .chars()
        .all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=".contains(c))
}

/// Validate date in various formats
pub fn is_valid_date(date: &str) -> bool {
    // Accept various date formats
    let date_patterns = [
        r"^\d{4}-\d{2}-\d{2}$", // YYYY-MM-DD
        r"^\d{2}/\d{2}/\d{4}$", // MM/DD/YYYY
        r"^\d{2}-\d{2}-\d{4}$", // MM-DD-YYYY
        r"^\d{4}/\d{2}/\d{2}$", // YYYY/MM/DD
    ];

    date_patterns
        .iter()
        .any(|pattern| Regex::new(pattern).unwrap().is_match(date))
}

/// Validate password strength and return recommendations
pub fn validate_password_strength(password: &str) -> ValidationResult {
    let mut result = ValidationResult::success();

    if password.is_empty() {
        result.add_error("Password cannot be empty".to_string());
        return result;
    }

    let length = password.len();
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digits = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if length < 8 {
        result.add_error("Password must be at least 8 characters long".to_string());
    } else if length < 12 {
        result.add_warning(
            "Password should be at least 12 characters for better security".to_string(),
        );
    }

    if !has_lowercase {
        result.add_warning("Password should contain lowercase letters".to_string());
    }

    if !has_uppercase {
        result.add_warning("Password should contain uppercase letters".to_string());
    }

    if !has_digits {
        result.add_warning("Password should contain numbers".to_string());
    }

    if !has_special {
        result.add_warning("Password should contain special characters".to_string());
    }

    // Check for common patterns
    if password.to_lowercase().contains("password") {
        result.add_warning("Password should not contain the word 'password'".to_string());
    }

    if is_sequential(&password) {
        result.add_warning("Password should not contain sequential characters".to_string());
    }

    if has_repeated_chars(&password) {
        result.add_warning("Password should not have many repeated characters".to_string());
    }

    result
}

/// Check if string contains sequential characters
fn is_sequential(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    for window in chars.windows(3) {
        let a = window[0] as u32;
        let b = window[1] as u32;
        let c = window[2] as u32;

        if b == a + 1 && c == b + 1 {
            return true; // Ascending sequence
        }
        if b == a - 1 && c == b - 1 {
            return true; // Descending sequence
        }
    }

    false
}

/// Check if string has too many repeated characters
fn has_repeated_chars(s: &str) -> bool {
    let mut char_counts = std::collections::HashMap::new();
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    // More than 40% of characters are the same
    let max_count = char_counts.values().max().unwrap_or(&0);
    (*max_count as f64) / (s.len() as f64) > 0.4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CredentialField;

    #[test]
    fn test_credential_title_validation() {
        assert!(validate_credential_title("Valid Title").is_valid);
        assert!(!validate_credential_title("").is_valid);
        assert!(!validate_credential_title(&"x".repeat(MAX_TITLE_LENGTH + 1)).is_valid);

        let result = validate_credential_title("  Title with spaces  ");
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.email+tag@domain.org"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user.domain.com"));
    }

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://localhost:8080"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url("file:///path/to/file"));
    }

    #[test]
    fn test_phone_validation() {
        assert!(is_valid_phone("1234567890"));
        assert!(is_valid_phone("+1 (555) 123-4567"));
        assert!(is_valid_phone("555-123-4567"));
        assert!(!is_valid_phone("123"));
        assert!(!is_valid_phone("abc-def-ghij"));
    }

    #[test]
    fn test_credit_card_validation() {
        // Valid test cards (these pass Luhn algorithm)
        assert!(is_valid_credit_card("4111111111111111")); // Valid Visa test number
        assert!(is_valid_credit_card("4111-1111-1111-1111"));
        assert!(is_valid_credit_card("4111 1111 1111 1111"));

        // Invalid cards
        assert!(!is_valid_credit_card("4532123456789013")); // Wrong checksum
        assert!(!is_valid_credit_card("123")); // Too short
        assert!(!is_valid_credit_card("12345678901234567890")); // Too long
    }

    #[test]
    fn test_expiry_date_validation() {
        assert!(is_valid_expiry_date("12/25"));
        assert!(is_valid_expiry_date("01/30"));
        assert!(!is_valid_expiry_date("13/25")); // Invalid month
        assert!(!is_valid_expiry_date("12/2025")); // Wrong format
        assert!(!is_valid_expiry_date("1/25")); // Missing leading zero
    }

    #[test]
    fn test_cvv_validation() {
        assert!(is_valid_cvv("123"));
        assert!(is_valid_cvv("4567"));
        assert!(!is_valid_cvv("12"));
        assert!(!is_valid_cvv("12345"));
        assert!(!is_valid_cvv("abc"));
    }

    #[test]
    fn test_totp_secret_validation() {
        assert!(is_valid_totp_secret("JBSWY3DPEHPK3PXP"));
        assert!(is_valid_totp_secret("jbswy3dpehpk3pxp")); // Should handle lowercase
        assert!(is_valid_totp_secret("JBSW Y3DP EHPK 3PXP")); // Should handle spaces
        assert!(!is_valid_totp_secret(""));
        assert!(!is_valid_totp_secret("invalid!@#$"));
    }

    #[test]
    fn test_date_validation() {
        assert!(is_valid_date("2023-12-25"));
        assert!(is_valid_date("12/25/2023"));
        assert!(is_valid_date("12-25-2023"));
        assert!(is_valid_date("2023/12/25"));
        assert!(!is_valid_date("invalid-date"));
        assert!(!is_valid_date("25/12/23")); // Wrong format
    }

    #[test]
    fn test_password_strength_validation() {
        let weak_result = validate_password_strength("weak");
        assert!(!weak_result.is_valid);
        assert!(!weak_result.errors.is_empty());

        let medium_result = validate_password_strength("Password123");
        assert!(medium_result.is_valid);
        assert!(!medium_result.warnings.is_empty());

        let strong_result = validate_password_strength("SuperSecure123!@#Random");
        assert!(strong_result.is_valid);
        // Strong passwords might still have some warnings
    }

    #[test]
    fn test_field_validation() {
        let email_field = CredentialField::email("user@example.com");
        let result = validate_field("email", &email_field);
        assert!(result.is_valid);

        let invalid_email_field = CredentialField::email("invalid-email");
        let result = validate_field("email", &invalid_email_field);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("not a valid email")));
    }

    #[test]
    fn test_credential_validation() {
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("email", CredentialField::email("user@example.com"));

        let result = validate_credential(&credential);
        assert!(result.is_valid);

        // Test with invalid email
        credential.set_field("email", CredentialField::email("invalid-email"));
        let result = validate_credential(&credential);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_tags_validation() {
        let valid_tags = vec!["work".to_string(), "important".to_string()];
        let result = validate_credential_tags(&valid_tags);
        assert!(result.is_valid);

        // Test duplicate tags
        let duplicate_tags = vec!["work".to_string(), "Work".to_string()];
        let result = validate_credential_tags(&duplicate_tags);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Duplicate tag")));

        // Test too many tags
        let too_many_tags: Vec<String> = (0..MAX_TAGS_PER_CREDENTIAL + 1)
            .map(|i| format!("tag{}", i))
            .collect();
        let result = validate_credential_tags(&too_many_tags);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Too many tags")));
    }

    #[test]
    fn test_validation_result_operations() {
        let mut result = ValidationResult::success();
        assert!(result.is_valid);

        result.add_error("Test error".to_string());
        assert!(!result.is_valid);

        result.add_warning("Test warning".to_string());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);

        let mut other_result = ValidationResult::success();
        other_result.add_error("Another error".to_string());

        result.merge(other_result);
        assert_eq!(result.errors.len(), 2);
    }
}
