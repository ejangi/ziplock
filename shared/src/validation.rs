//! Comprehensive Validation Module for ZipLock
//!
//! This module provides shared validation logic for both credentials and master passphrases
//! across both frontend and backend components of ZipLock. It ensures consistent security
//! requirements and user feedback.

use crate::constants::*;
use crate::error::{SharedError, SharedResult};
use crate::models::CredentialRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Master passphrase validation requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PassphraseRequirements {
    /// Minimum length in characters
    pub min_length: usize,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one numeric digit
    pub require_numeric: bool,
    /// Require at least one special character (non-alphanumeric)
    pub require_special: bool,
    /// Maximum length in characters (0 = no limit)
    pub max_length: usize,
    /// Minimum unique character count (helps prevent repeated patterns)
    pub min_unique_chars: usize,
}

impl Default for PassphraseRequirements {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_lowercase: true,
            require_uppercase: true,
            require_numeric: true,
            require_special: true,
            max_length: 0, // No limit
            min_unique_chars: 8,
        }
    }
}

impl PassphraseRequirements {
    /// Create requirements for a "strong" passphrase (default)
    pub fn strong() -> Self {
        Self::default()
    }

    /// Create requirements for a "basic" passphrase (less strict)
    pub fn basic() -> Self {
        Self {
            min_length: 8,
            require_lowercase: true,
            require_uppercase: true,
            require_numeric: true,
            require_special: false,
            max_length: 0,
            min_unique_chars: 6,
        }
    }

    /// Create requirements for a "minimal" passphrase (very permissive)
    pub fn minimal() -> Self {
        Self {
            min_length: 6,
            require_lowercase: false,
            require_uppercase: false,
            require_numeric: false,
            require_special: false,
            max_length: 0,
            min_unique_chars: 3,
        }
    }
}

/// Passphrase strength assessment result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassphraseStrength {
    /// Overall strength level
    pub level: StrengthLevel,
    /// Numeric score (0-100)
    pub score: u8,
    /// List of validation errors
    pub violations: Vec<String>,
    /// List of passed requirements
    pub satisfied: Vec<String>,
    /// Whether the passphrase meets the minimum requirements
    pub meets_requirements: bool,
}

/// Strength level enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrengthLevel {
    /// Empty or very weak passphrase
    VeryWeak,
    /// Weak passphrase
    Weak,
    /// Fair passphrase
    Fair,
    /// Good passphrase
    Good,
    /// Strong passphrase
    Strong,
    /// Very strong passphrase
    VeryStrong,
}

impl StrengthLevel {
    /// Get a human-readable string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            StrengthLevel::VeryWeak => "Very Weak",
            StrengthLevel::Weak => "Weak",
            StrengthLevel::Fair => "Fair",
            StrengthLevel::Good => "Good",
            StrengthLevel::Strong => "Strong",
            StrengthLevel::VeryStrong => "Very Strong",
        }
    }

    /// Get the color associated with this strength level (RGB hex)
    pub fn color_hex(&self) -> &'static str {
        match self {
            StrengthLevel::VeryWeak => "#ef476f", // ERROR_RED from theme.rs
            StrengthLevel::Weak => "#ef476f",     // ERROR_RED from theme.rs
            StrengthLevel::Fair => "#fcbf49",     // Yellow (warning)
            StrengthLevel::Good => "#06d6a0",     // SUCCESS_GREEN from theme.rs
            StrengthLevel::Strong => "#06d6a0",   // SUCCESS_GREEN from theme.rs
            StrengthLevel::VeryStrong => "#8338ec", // LOGO_PURPLE from theme.rs
        }
    }

    /// Check if this strength level is considered acceptable
    pub fn is_acceptable(&self) -> bool {
        matches!(
            self,
            StrengthLevel::Good | StrengthLevel::Strong | StrengthLevel::VeryStrong
        )
    }
}

/// Master passphrase validator
#[derive(Debug, Clone)]
pub struct PassphraseValidator {
    requirements: PassphraseRequirements,
}

impl Default for PassphraseValidator {
    fn default() -> Self {
        Self::new(PassphraseRequirements::default())
    }
}

impl PassphraseValidator {
    /// Create a new validator with the given requirements
    pub fn new(requirements: PassphraseRequirements) -> Self {
        Self { requirements }
    }

    /// Create a validator with strong requirements (default)
    pub fn strong() -> Self {
        Self::new(PassphraseRequirements::strong())
    }

    /// Create a validator with basic requirements
    pub fn basic() -> Self {
        Self::new(PassphraseRequirements::basic())
    }

    /// Create a validator with minimal requirements
    pub fn minimal() -> Self {
        Self::new(PassphraseRequirements::minimal())
    }

    /// Get the current requirements
    pub fn requirements(&self) -> &PassphraseRequirements {
        &self.requirements
    }

    /// Validate a passphrase and return detailed results
    pub fn validate(&self, passphrase: &str) -> PassphraseStrength {
        let mut violations = Vec::new();
        let mut satisfied = Vec::new();

        if passphrase.is_empty() {
            return PassphraseStrength {
                level: StrengthLevel::VeryWeak,
                score: 0,
                violations: vec!["Passphrase cannot be empty".to_string()],
                satisfied: Vec::new(),
                meets_requirements: false,
            };
        }

        // Check length requirements
        if passphrase.len() < self.requirements.min_length {
            violations.push(format!(
                "Must be at least {} characters long",
                self.requirements.min_length
            ));
        } else {
            satisfied.push(format!(
                "Length requirement met ({} chars)",
                passphrase.len()
            ));
        }

        if self.requirements.max_length > 0 && passphrase.len() > self.requirements.max_length {
            violations.push(format!(
                "Must be no more than {} characters long",
                self.requirements.max_length
            ));
        }

        // Check character requirements
        let has_lowercase = passphrase.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = passphrase.chars().any(|c| c.is_ascii_uppercase());
        let has_numeric = passphrase.chars().any(|c| c.is_ascii_digit());
        let has_special = passphrase.chars().any(|c| !c.is_alphanumeric());

        if self.requirements.require_lowercase {
            if has_lowercase {
                satisfied.push("Contains lowercase letters".to_string());
            } else {
                violations.push("Must contain at least one lowercase letter".to_string());
            }
        }

        if self.requirements.require_uppercase {
            if has_uppercase {
                satisfied.push("Contains uppercase letters".to_string());
            } else {
                violations.push("Must contain at least one uppercase letter".to_string());
            }
        }

        if self.requirements.require_numeric {
            if has_numeric {
                satisfied.push("Contains numbers".to_string());
            } else {
                violations.push("Must contain at least one number".to_string());
            }
        }

        if self.requirements.require_special {
            if has_special {
                satisfied.push("Contains special characters".to_string());
            } else {
                violations.push("Must contain at least one special character".to_string());
            }
        }

        // Check unique characters
        let unique_chars: HashSet<char> = passphrase.chars().collect();
        if unique_chars.len() < self.requirements.min_unique_chars {
            violations.push(format!(
                "Must contain at least {} unique characters",
                self.requirements.min_unique_chars
            ));
        } else {
            satisfied.push(format!(
                "Sufficient character variety ({} unique chars)",
                unique_chars.len()
            ));
        }

        let meets_requirements = violations.is_empty();
        let score = self.calculate_score(passphrase);
        let level = self.score_to_level(score);

        PassphraseStrength {
            level,
            score,
            violations,
            satisfied,
            meets_requirements,
        }
    }

    /// Quick check if a passphrase meets the minimum requirements
    pub fn meets_requirements(&self, passphrase: &str) -> bool {
        self.validate(passphrase).meets_requirements
    }

    /// Calculate a numeric strength score (0-100)
    fn calculate_score(&self, passphrase: &str) -> u8 {
        if passphrase.is_empty() {
            return 0;
        }

        let mut score = 0u8;
        let length = passphrase.len();

        // Length scoring (0-40 points)
        score += match length {
            0..=5 => 0,
            6..=7 => 10,
            8..=11 => 20,
            12..=15 => 30,
            16..=19 => 35,
            _ => 40,
        };

        // Character variety scoring (0-40 points)
        let has_lowercase = passphrase.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = passphrase.chars().any(|c| c.is_ascii_uppercase());
        let has_digits = passphrase.chars().any(|c| c.is_ascii_digit());
        let has_special = passphrase.chars().any(|c| !c.is_alphanumeric());

        let variety_count = [has_lowercase, has_uppercase, has_digits, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        score += match variety_count {
            0 => 0,
            1 => 10,
            2 => 20,
            3 => 30,
            4 => 40,
            _ => 40,
        };

        // Unique character bonus (0-20 points)
        let unique_chars: HashSet<char> = passphrase.chars().collect();
        let uniqueness_ratio = unique_chars.len() as f32 / passphrase.len() as f32;
        score += match uniqueness_ratio {
            r if r >= 0.9 => 20,
            r if r >= 0.8 => 15,
            r if r >= 0.7 => 10,
            r if r >= 0.6 => 5,
            _ => 0,
        };

        score.min(100)
    }

    /// Convert numeric score to strength level
    fn score_to_level(&self, score: u8) -> StrengthLevel {
        match score {
            0..=20 => StrengthLevel::VeryWeak,
            21..=40 => StrengthLevel::Weak,
            41..=60 => StrengthLevel::Fair,
            61..=80 => StrengthLevel::Good,
            81..=95 => StrengthLevel::Strong,
            96..=100 => StrengthLevel::VeryStrong,
            _ => StrengthLevel::VeryStrong, // For any score > 100 (shouldn't happen)
        }
    }
}

/// Common validation presets
pub struct ValidationPresets;

impl ValidationPresets {
    /// Get requirements for production use (strong security)
    pub fn production() -> PassphraseRequirements {
        PassphraseRequirements::strong()
    }

    /// Get requirements for development/testing (more permissive)
    pub fn development() -> PassphraseRequirements {
        PassphraseRequirements::basic()
    }

    /// Get requirements for legacy compatibility (minimal)
    pub fn legacy() -> PassphraseRequirements {
        PassphraseRequirements::minimal()
    }
}

/// Validation result for use in APIs
pub type PassphraseValidationResult = SharedResult<PassphraseStrength>;

/// Validate a master passphrase with given requirements
pub fn validate_master_passphrase(
    passphrase: &str,
    requirements: &PassphraseRequirements,
) -> PassphraseValidationResult {
    let validator = PassphraseValidator::new(requirements.clone());
    Ok(validator.validate(passphrase))
}

/// Quick validation check - returns error if passphrase doesn't meet requirements
pub fn validate_master_passphrase_strict(
    passphrase: &str,
    requirements: &PassphraseRequirements,
) -> SharedResult<()> {
    let strength = validate_master_passphrase(passphrase, requirements)?;

    if !strength.meets_requirements {
        return Err(SharedError::Validation {
            message: format!(
                "Master passphrase does not meet requirements: {}",
                strength.violations.join("; ")
            ),
        });
    }

    Ok(())
}

/// Enhanced passphrase validator that includes common pattern checking
pub struct EnhancedPassphraseValidator {
    validator: PassphraseValidator,
    check_common_patterns: bool,
}

impl Default for EnhancedPassphraseValidator {
    fn default() -> Self {
        Self::new(PassphraseRequirements::default(), true)
    }
}

impl EnhancedPassphraseValidator {
    /// Create a new enhanced validator
    pub fn new(requirements: PassphraseRequirements, check_common_patterns: bool) -> Self {
        Self {
            validator: PassphraseValidator::new(requirements),
            check_common_patterns,
        }
    }

    /// Validate passphrase with enhanced checks
    pub fn validate(&self, passphrase: &str) -> PassphraseStrength {
        let mut strength = self.validator.validate(passphrase);

        // Add common pattern checks if enabled
        if self.check_common_patterns {
            let weak_patterns = CommonPatterns::has_weak_patterns(passphrase);
            for pattern in weak_patterns {
                strength.violations.push(pattern);
                // Reduce score for weak patterns
                strength.score = strength.score.saturating_sub(10);
            }

            // Recalculate level based on adjusted score
            strength.level = self.validator.score_to_level(strength.score);

            // Update meets_requirements if we found new violations
            if !strength.violations.is_empty() {
                strength.meets_requirements = false;
            }
        }

        strength
    }

    /// Check if passphrase meets requirements (strict)
    pub fn meets_requirements(&self, passphrase: &str) -> bool {
        self.validate(passphrase).meets_requirements
    }
}

/// Common patterns to check against (weak passphrases)
pub struct CommonPatterns;

impl CommonPatterns {
    /// Check if passphrase contains common weak patterns
    pub fn has_weak_patterns(passphrase: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let lower_passphrase = passphrase.to_lowercase();

        // Check for common passwords
        let common_passwords = [
            "password",
            "123456",
            "123456789",
            "qwerty",
            "abc123",
            "password123",
            "admin",
            "letmein",
            "welcome",
            "monkey",
            "dragon",
            "master",
            "shadow",
            "12345678",
            "qwerty123",
            "111111",
            "123123",
            "password1",
            "1234567890",
        ];

        for common in &common_passwords {
            if lower_passphrase.contains(common) {
                patterns.push(format!("Contains common pattern: {common}"));
            }
        }

        // Check for keyboard patterns
        let keyboard_patterns = [
            "qwertyuiop",
            "asdfghjkl",
            "zxcvbnm",
            "qwerty",
            "asdf",
            "zxcv",
            "1234567890",
            "abcdefg",
            "123abc",
            "abc123",
        ];

        for pattern in &keyboard_patterns {
            if lower_passphrase.contains(pattern) {
                patterns.push(format!("Contains keyboard pattern: {pattern}"));
            }
        }

        // Check for repeated characters or simple patterns
        if has_repeated_chars(passphrase, 3) {
            patterns.push("Contains 3+ repeated characters in a row".to_string());
        }

        if has_sequential_chars(passphrase, 4) {
            patterns.push("Contains 4+ sequential characters".to_string());
        }

        patterns
    }
}

/// Check for repeated characters (e.g., "aaa", "111")
fn has_repeated_chars(passphrase: &str, min_length: usize) -> bool {
    if passphrase.len() < min_length {
        return false;
    }

    let chars: Vec<char> = passphrase.chars().collect();
    for i in 0..chars.len().saturating_sub(min_length - 1) {
        let mut all_same = true;
        let first_char = chars[i];

        for j in 1..min_length {
            if chars[i + j] != first_char {
                all_same = false;
                break;
            }
        }

        if all_same {
            return true;
        }
    }

    false
}

/// Check for sequential characters (e.g., "abcd", "1234")
fn has_sequential_chars(passphrase: &str, min_length: usize) -> bool {
    if passphrase.len() < min_length {
        return false;
    }

    let chars: Vec<char> = passphrase.chars().collect();
    for i in 0..chars.len().saturating_sub(min_length - 1) {
        let mut is_sequential = true;

        for j in 1..min_length {
            let current = chars[i + j] as u32;
            let previous = chars[i + j - 1] as u32;

            if current != previous + 1 {
                is_sequential = false;
                break;
            }
        }

        if is_sequential {
            return true;
        }
    }

    false
}

// ========== CREDENTIAL VALIDATION ==========
// Moving existing validation functions from lib.rs

/// Validate credential record against library constraints
pub fn validate_credential(credential: &CredentialRecord) -> SharedResult<()> {
    // Check title length
    if credential.title.len() > MAX_CREDENTIAL_TITLE_LENGTH {
        return Err(SharedError::Validation {
            message: format!(
                "Title too long: {} > {}",
                credential.title.len(),
                MAX_CREDENTIAL_TITLE_LENGTH
            ),
        });
    }

    // Check number of fields
    if credential.fields.len() > MAX_FIELDS_PER_CREDENTIAL {
        return Err(SharedError::Validation {
            message: format!(
                "Too many fields: {} > {}",
                credential.fields.len(),
                MAX_FIELDS_PER_CREDENTIAL
            ),
        });
    }

    // Check number of tags
    if credential.tags.len() > MAX_TAGS_PER_CREDENTIAL {
        return Err(SharedError::Validation {
            message: format!(
                "Too many tags: {} > {}",
                credential.tags.len(),
                MAX_TAGS_PER_CREDENTIAL
            ),
        });
    }

    // Check tag lengths
    for tag in &credential.tags {
        if tag.len() > MAX_TAG_LENGTH {
            return Err(SharedError::Validation {
                message: format!("Tag too long: '{tag}' > {MAX_TAG_LENGTH}"),
            });
        }
    }

    // Check notes length
    if let Some(notes) = &credential.notes {
        if notes.len() > MAX_NOTES_LENGTH {
            return Err(SharedError::Validation {
                message: format!("Notes too long: {} > {}", notes.len(), MAX_NOTES_LENGTH),
            });
        }
    }

    // Check field value lengths
    for (field_name, field) in &credential.fields {
        if field.value.len() > MAX_FIELD_VALUE_LENGTH {
            return Err(SharedError::Validation {
                message: format!(
                    "Field '{}' value too long: {} > {}",
                    field_name,
                    field.value.len(),
                    MAX_FIELD_VALUE_LENGTH
                ),
            });
        }
    }

    // Run the credential's own validation
    credential
        .validate()
        .map_err(|errors| SharedError::Validation {
            message: errors.join(", "),
        })?;

    Ok(())
}

/// Check if a string is a valid credential ID (UUID format)
pub fn is_valid_credential_id(id: &str) -> bool {
    // Simple UUID format check
    id.len() == 36
        && id.chars().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// Sanitize a string for safe use as a filename or identifier
pub fn sanitize_identifier(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Validation utilities for consolidating passphrase validation functionality
pub struct ValidationUtils;

impl ValidationUtils {
    /// Create a validator from backend configuration
    pub fn from_config(min_password_length: usize) -> PassphraseValidator {
        let requirements = PassphraseRequirements {
            min_length: min_password_length,
            ..PassphraseRequirements::default()
        };
        PassphraseValidator::new(requirements)
    }

    /// Create a validator for repository creation (production requirements)
    pub fn for_creation() -> PassphraseValidator {
        PassphraseValidator::new(ValidationPresets::production())
    }

    /// Create a minimal validator for repository opening
    /// Note: For opening, any passphrase that can decrypt is valid
    pub fn for_opening() -> PassphraseValidator {
        PassphraseValidator::minimal()
    }

    /// Validate passphrase and return detailed result
    pub fn validate_with_details(
        passphrase: &str,
        validator: &PassphraseValidator,
    ) -> PassphraseStrength {
        validator.validate(passphrase)
    }

    /// Validate passphrase and return boolean result
    pub fn validate_meets_requirements(passphrase: &str, validator: &PassphraseValidator) -> bool {
        validator.meets_requirements(passphrase)
    }

    /// Validate passphrase strictly (for backend API usage)
    /// Returns error if requirements not met
    pub fn validate_strict(
        passphrase: &str,
        validator: &PassphraseValidator,
    ) -> crate::SharedResult<()> {
        let strength = validator.validate(passphrase);

        if !strength.meets_requirements {
            return Err(crate::SharedError::Validation {
                message: format!(
                    "Master passphrase does not meet requirements: {}",
                    strength.violations.join("; ")
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_requirements() {
        let req = PassphraseRequirements::default();
        assert_eq!(req.min_length, 12);
        assert!(req.require_lowercase);
        assert!(req.require_uppercase);
        assert!(req.require_numeric);
        assert!(req.require_special);
    }

    #[test]
    fn test_strength_levels() {
        assert_eq!(StrengthLevel::Strong.as_str(), "Strong");
        assert_eq!(StrengthLevel::Weak.color_hex(), "#ef476f"); // Updated to match ERROR_RED
        assert!(StrengthLevel::Good.is_acceptable());
        assert!(!StrengthLevel::Weak.is_acceptable());
    }

    #[test]
    fn test_passphrase_validation() {
        let validator = PassphraseValidator::default();

        // Test empty passphrase
        let result = validator.validate("");
        assert_eq!(result.level, StrengthLevel::VeryWeak);
        assert_eq!(result.score, 0);
        assert!(!result.meets_requirements);

        // Test weak passphrase
        let result = validator.validate("weak");
        assert!(!result.meets_requirements);
        assert!(!result.violations.is_empty());

        // Test strong passphrase
        let result = validator.validate("MySecurePassphrase123!");
        assert!(result.meets_requirements);
        assert!(result.score > 80);
        assert!(result.level.is_acceptable());
    }

    #[test]
    fn test_basic_requirements() {
        let validator = PassphraseValidator::basic();
        let result = validator.validate("Password1");

        // Should meet basic requirements (8+ chars, upper, lower, number)
        assert!(result.meets_requirements);

        // But might not be very strong due to common pattern
        let enhanced = EnhancedPassphraseValidator::new(PassphraseRequirements::basic(), true);
        let enhanced_result = enhanced.validate("Password1");
        assert!(!enhanced_result.violations.is_empty()); // Should catch "password" pattern
    }

    #[test]
    fn test_common_patterns() {
        let patterns = CommonPatterns::has_weak_patterns("password123");
        assert!(!patterns.is_empty());

        let patterns = CommonPatterns::has_weak_patterns("MyUniqueSecretPhrase456!");
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_repeated_chars() {
        assert!(has_repeated_chars("aaabbb", 3));
        assert!(has_repeated_chars("password111", 3));
        assert!(!has_repeated_chars("password1", 3));
        assert!(!has_repeated_chars("ab", 3));
    }

    #[test]
    fn test_sequential_chars() {
        assert!(has_sequential_chars("abcd", 4));
        assert!(has_sequential_chars("1234", 4));
        assert!(has_sequential_chars("xyz", 3));
        assert!(!has_sequential_chars("aceg", 4));
        assert!(!has_sequential_chars("abc", 4));
    }

    #[test]
    fn test_validation_presets() {
        let prod = ValidationPresets::production();
        let dev = ValidationPresets::development();
        let legacy = ValidationPresets::legacy();

        assert!(prod.min_length >= dev.min_length);
        assert!(dev.min_length >= legacy.min_length);

        // Production should be most strict
        assert!(prod.require_special);
        assert!(prod.require_uppercase);

        // Legacy should be most permissive
        assert!(legacy.min_length <= 8);
    }

    #[test]
    fn test_enhanced_validator() {
        let enhanced = EnhancedPassphraseValidator::default();

        // Test with common password
        let result = enhanced.validate("password123");
        assert!(!result.meets_requirements);
        assert!(result
            .violations
            .iter()
            .any(|v| v.contains("common pattern")));

        // Test with strong unique passphrase
        let result = enhanced.validate("MyUniqueSecretPhrase789!");
        assert!(result.meets_requirements);
        assert!(result.score > 80);
    }

    #[test]
    fn test_validation_functions() {
        let req = PassphraseRequirements::default();

        // Test successful validation
        let result = validate_master_passphrase("StrongPassphrase123!", &req);
        assert!(result.is_ok());

        // Test strict validation with weak passphrase
        let result = validate_master_passphrase_strict("weak", &req);
        assert!(result.is_err());

        // Test strict validation with strong passphrase
        let result = validate_master_passphrase_strict("StrongPassphrase123!", &req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_credential_validation() {
        use crate::models::CredentialRecord;

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
    fn test_frontend_backend_consistency() {
        // Test that frontend and backend would accept/reject the same passphrases
        let production_requirements = ValidationPresets::production();
        let validator = PassphraseValidator::new(production_requirements.clone());

        // Test cases with expected results
        let test_cases = vec![
            ("", false),                                      // Empty
            ("weak", false),                                  // Too short, no variety
            ("password", false),                              // Common pattern, no variety
            ("Password1", false),                             // Too short for production
            ("Password123", false),                           // Too short for production
            ("Password123!", true),                           // 12 chars, meets all requirements
            ("MyPassword123!", true),                         // 14 chars, meets all requirements
            ("SuperSecurePassphrase123!", true),              // Strong passphrase
            ("verylongpassphrasewithnouppercase123!", false), // No uppercase
            ("VERYLONGPASSPHRASEWITHNOSPECIAL123", false),    // No special chars
            ("MySecure123!", true),                           // 12 chars, meets all requirements
            ("MySecurePassphrase", false),                    // No numbers or special chars
            ("mysecure123!", false),                          // No uppercase
            ("MYSECURE123!", false),                          // No lowercase
        ];

        for (passphrase, expected_valid) in test_cases {
            // Test shared validation
            let shared_result =
                validate_master_passphrase_strict(passphrase, &production_requirements);
            let shared_valid = shared_result.is_ok();

            // Test frontend validator
            let frontend_valid = validator.meets_requirements(passphrase);

            // Both should agree
            assert_eq!(
                shared_valid, expected_valid,
                "Shared validation mismatch for '{}': expected {}, got {}",
                passphrase, expected_valid, shared_valid
            );
            assert_eq!(
                frontend_valid, expected_valid,
                "Frontend validation mismatch for '{}': expected {}, got {}",
                passphrase, expected_valid, frontend_valid
            );
            assert_eq!(
                shared_valid, frontend_valid,
                "Frontend and shared validation disagree for '{}': shared={}, frontend={}",
                passphrase, shared_valid, frontend_valid
            );

            println!(
                "✓ '{}' -> valid: {} (consistent)",
                passphrase, expected_valid
            );
        }
    }

    #[test]
    fn test_strength_level_theme_consistency() {
        // Verify that strength levels map to appropriate theme colors
        assert_eq!(StrengthLevel::VeryWeak.color_hex(), "#ef476f"); // ERROR_RED
        assert_eq!(StrengthLevel::Weak.color_hex(), "#ef476f"); // ERROR_RED
        assert_eq!(StrengthLevel::Good.color_hex(), "#06d6a0"); // SUCCESS_GREEN
        assert_eq!(StrengthLevel::Strong.color_hex(), "#06d6a0"); // SUCCESS_GREEN
        assert_eq!(StrengthLevel::VeryStrong.color_hex(), "#8338ec"); // LOGO_PURPLE

        // Verify acceptable levels
        assert!(!StrengthLevel::VeryWeak.is_acceptable());
        assert!(!StrengthLevel::Weak.is_acceptable());
        assert!(!StrengthLevel::Fair.is_acceptable());
        assert!(StrengthLevel::Good.is_acceptable());
        assert!(StrengthLevel::Strong.is_acceptable());
        assert!(StrengthLevel::VeryStrong.is_acceptable());
    }

    #[test]
    fn test_validation_utils_from_config() {
        let validator = ValidationUtils::from_config(8);
        let requirements = validator.requirements();
        assert_eq!(requirements.min_length, 8);
        assert!(requirements.require_lowercase);
        assert!(requirements.require_uppercase);
        assert!(requirements.require_numeric);
        assert!(requirements.require_special);
    }

    #[test]
    fn test_validation_utils_for_creation() {
        let validator = ValidationUtils::for_creation();
        let requirements = validator.requirements();
        // Should match production requirements
        assert_eq!(requirements.min_length, 12);
        assert!(requirements.require_lowercase);
        assert!(requirements.require_uppercase);
        assert!(requirements.require_numeric);
        assert!(requirements.require_special);
    }

    #[test]
    fn test_validation_utils_for_opening() {
        let validator = ValidationUtils::for_opening();
        let requirements = validator.requirements();
        // Should match minimal requirements
        assert_eq!(requirements.min_length, 6);
        assert!(!requirements.require_lowercase);
        assert!(!requirements.require_uppercase);
        assert!(!requirements.require_numeric);
        assert!(!requirements.require_special);
    }

    #[test]
    fn test_validation_utils_validate_with_details() {
        let validator = ValidationUtils::for_creation();
        let result = ValidationUtils::validate_with_details("StrongPassphrase123!", &validator);
        assert!(result.meets_requirements);
        assert!(result.level.is_acceptable());
        assert!(result.score > 0);
        assert!(result.violations.is_empty());
        assert!(!result.satisfied.is_empty());
    }

    #[test]
    fn test_validation_utils_validate_meets_requirements() {
        let validator = ValidationUtils::for_creation();

        // Strong passphrase should meet requirements
        assert!(ValidationUtils::validate_meets_requirements(
            "StrongPassphrase123!",
            &validator
        ));

        // Weak passphrase should not meet requirements
        assert!(!ValidationUtils::validate_meets_requirements(
            "weak", &validator
        ));
    }

    #[test]
    fn test_validation_utils_validate_strict() {
        let validator = ValidationUtils::for_creation();

        // Strong passphrase should pass strict validation
        let result = ValidationUtils::validate_strict("StrongPassphrase123!", &validator);
        assert!(result.is_ok());

        // Weak passphrase should fail strict validation
        let result = ValidationUtils::validate_strict("weak", &validator);
        assert!(result.is_err());

        if let Err(error) = result {
            assert!(error.to_string().contains("does not meet requirements"));
        }
    }

    #[test]
    fn test_validation_utils_consistency() {
        // Test that all ValidationUtils methods work consistently
        let validator = ValidationUtils::for_creation();
        let test_passphrase = "MySecurePassphrase123!";

        let detailed_result = ValidationUtils::validate_with_details(test_passphrase, &validator);
        let boolean_result =
            ValidationUtils::validate_meets_requirements(test_passphrase, &validator);
        let strict_result = ValidationUtils::validate_strict(test_passphrase, &validator);

        // All methods should agree on whether the passphrase is valid
        assert_eq!(detailed_result.meets_requirements, boolean_result);
        assert_eq!(boolean_result, strict_result.is_ok());
    }
}
