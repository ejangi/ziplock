//! Credential-specific utilities and extensions
//!
//! This module provides additional functionality for working with credentials
//! beyond the basic data structures, including import/export, validation,
//! and credential management utilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use super::{CredentialField, CredentialRecord};

/// Credential import/export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialExport {
    /// Export format version
    pub version: String,
    /// Export timestamp
    pub exported_at: SystemTime,
    /// List of exported credentials
    pub credentials: Vec<CredentialRecord>,
    /// Export metadata
    pub metadata: HashMap<String, String>,
}

/// Credential statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStats {
    /// Total number of credentials
    pub total_credentials: usize,
    /// Credentials by type
    pub by_type: HashMap<String, usize>,
    /// Credentials by tag
    pub by_tag: HashMap<String, usize>,
    /// Number of credentials with weak passwords
    pub weak_passwords: usize,
    /// Number of credentials with duplicate passwords
    pub duplicate_passwords: usize,
    /// Number of credentials without 2FA
    pub missing_2fa: usize,
}

/// Credential utilities
pub struct CredentialUtils;

impl CredentialUtils {
    /// Generate a secure password with specified parameters
    pub fn generate_password(length: usize, include_symbols: bool) -> String {
        use rand::Rng;

        let mut charset =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string();
        if include_symbols {
            charset.push_str("!@#$%^&*()-_=+[]{}|;:,.<>?");
        }

        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap()
            })
            .collect()
    }

    /// Check if two credentials are duplicates (same website/service)
    pub fn are_duplicates(cred1: &CredentialRecord, cred2: &CredentialRecord) -> bool {
        if cred1.id == cred2.id {
            return false; // Same credential
        }

        // Check for same website
        if let (Some(url1), Some(url2)) = (
            cred1
                .get_field("website")
                .or_else(|| cred1.get_field("url")),
            cred2
                .get_field("website")
                .or_else(|| cred2.get_field("url")),
        ) {
            if Self::normalize_url(&url1.value) == Self::normalize_url(&url2.value) {
                return true;
            }
        }

        // Check for same title (case insensitive)
        if cred1.title.to_lowercase() == cred2.title.to_lowercase() {
            return true;
        }

        false
    }

    /// Normalize URL for comparison (remove protocol, www, etc.)
    fn normalize_url(url: &str) -> String {
        let mut normalized = url.to_lowercase();

        // Remove protocol
        if let Some(pos) = normalized.find("://") {
            normalized = normalized[pos + 3..].to_string();
        }

        // Remove www.
        if normalized.starts_with("www.") {
            normalized = normalized[4..].to_string();
        }

        // Remove trailing slash and path
        if let Some(pos) = normalized.find('/') {
            normalized = normalized[..pos].to_string();
        }

        normalized
    }

    /// Find credentials with weak passwords
    pub fn find_weak_passwords(credentials: &[CredentialRecord]) -> Vec<String> {
        use crate::models::field::FieldUtils;

        credentials
            .iter()
            .filter_map(|cred| {
                if let Some(password_field) = cred.get_field("password") {
                    let strength = FieldUtils::password_strength(&password_field.value);
                    if matches!(
                        strength,
                        crate::models::field::PasswordStrength::VeryWeak
                            | crate::models::field::PasswordStrength::Weak
                    ) {
                        Some(cred.id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find credentials with duplicate passwords
    pub fn find_duplicate_passwords(credentials: &[CredentialRecord]) -> Vec<Vec<String>> {
        let mut password_map: HashMap<String, Vec<String>> = HashMap::new();

        // Group credentials by password
        for cred in credentials {
            if let Some(password_field) = cred.get_field("password") {
                if !password_field.value.is_empty() {
                    password_map
                        .entry(password_field.value.clone())
                        .or_default()
                        .push(cred.id.clone());
                }
            }
        }

        // Return groups with more than one credential
        password_map
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }

    /// Find credentials missing 2FA
    pub fn find_missing_2fa(credentials: &[CredentialRecord]) -> Vec<String> {
        credentials
            .iter()
            .filter(|cred| {
                // Only check login-type credentials
                if cred.credential_type == "login" {
                    // Check if TOTP field exists and has a value
                    if let Some(totp_field) =
                        cred.get_field("totp").or_else(|| cred.get_field("2fa"))
                    {
                        totp_field.value.is_empty()
                    } else {
                        true // No TOTP field at all
                    }
                } else {
                    false // Not a login credential
                }
            })
            .map(|cred| cred.id.clone())
            .collect()
    }

    /// Generate credential statistics
    pub fn generate_stats(credentials: &[CredentialRecord]) -> CredentialStats {
        let mut by_type = HashMap::new();
        let mut by_tag = HashMap::new();

        for cred in credentials {
            // Count by type
            *by_type.entry(cred.credential_type.clone()).or_insert(0) += 1;

            // Count by tags
            for tag in &cred.tags {
                *by_tag.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        CredentialStats {
            total_credentials: credentials.len(),
            by_type,
            by_tag,
            weak_passwords: Self::find_weak_passwords(credentials).len(),
            duplicate_passwords: Self::find_duplicate_passwords(credentials).len(),
            missing_2fa: Self::find_missing_2fa(credentials).len(),
        }
    }

    /// Create a credential from common patterns
    pub fn create_from_pattern(pattern: &str, title: String) -> Option<CredentialRecord> {
        use crate::models::template::CommonTemplates;

        match pattern.to_lowercase().as_str() {
            "login" | "website" => CommonTemplates::login().create_credential(title).ok(),
            "credit_card" | "card" => CommonTemplates::credit_card().create_credential(title).ok(),
            "note" | "secure_note" => CommonTemplates::secure_note().create_credential(title).ok(),
            "identity" | "personal" => CommonTemplates::identity().create_credential(title).ok(),
            "password" => CommonTemplates::password().create_credential(title).ok(),
            "document" | "file" => CommonTemplates::document().create_credential(title).ok(),
            "ssh_key" | "ssh" => CommonTemplates::ssh_key().create_credential(title).ok(),
            "bank_account" | "bank" => CommonTemplates::bank_account()
                .create_credential(title)
                .ok(),
            "api_credentials" => CommonTemplates::api_credentials()
                .create_credential(title)
                .ok(),
            "crypto_wallet" | "wallet" | "crypto" => CommonTemplates::crypto_wallet()
                .create_credential(title)
                .ok(),
            "software_license" | "license" => CommonTemplates::software_license()
                .create_credential(title)
                .ok(),
            "wifi" => CommonTemplates::wifi().create_credential(title).ok(),
            "database" | "db" => CommonTemplates::database().create_credential(title).ok(),
            "api" => CommonTemplates::api_credentials()
                .create_credential(title)
                .ok(),
            "api_key" => CommonTemplates::api_key().create_credential(title).ok(),
            _ => None,
        }
    }

    /// Import credentials from CSV format
    pub fn import_from_csv(csv_data: &str) -> Result<Vec<CredentialRecord>, String> {
        let mut credentials = Vec::new();
        let lines: Vec<&str> = csv_data.lines().collect();

        if lines.is_empty() {
            return Err("Empty CSV data".to_string());
        }

        // Parse header
        let headers: Vec<&str> = lines[0].split(',').map(|h| h.trim()).collect();

        // Process data rows
        for (row_idx, line) in lines.iter().skip(1).enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let values: Vec<&str> = line
                .split(',')
                .map(|v| v.trim().trim_matches('"'))
                .collect();

            if values.len() != headers.len() {
                return Err(format!(
                    "Row {} has {} values but expected {}",
                    row_idx + 2,
                    values.len(),
                    headers.len()
                ));
            }

            let mut credential = CredentialRecord::new(
                values.first().unwrap_or(&"Untitled").to_string(),
                "login".to_string(),
            );

            // Map CSV columns to credential fields
            for (header, value) in headers.iter().zip(values.iter()) {
                if value.is_empty() {
                    continue;
                }

                match header.to_lowercase().as_str() {
                    "title" | "name" => credential.title = value.to_string(),
                    "username" | "user" => {
                        credential.set_field("username", CredentialField::username(*value));
                    }
                    "password" => {
                        credential.set_field("password", CredentialField::password(*value));
                    }
                    "email" => {
                        credential.set_field("email", CredentialField::email(*value));
                    }
                    "url" | "website" => {
                        credential.set_field("website", CredentialField::url(*value));
                    }
                    "notes" => {
                        credential.notes = Some(value.to_string());
                    }
                    "tags" => {
                        credential.tags = value.split(';').map(|t| t.trim().to_string()).collect();
                    }
                    _ => {
                        // Add as custom field
                        credential.set_field(*header, CredentialField::text(*value));
                    }
                }
            }

            credentials.push(credential);
        }

        Ok(credentials)
    }

    /// Export credentials to CSV format
    pub fn export_to_csv(credentials: &[CredentialRecord]) -> String {
        if credentials.is_empty() {
            return String::new();
        }

        let mut csv = String::new();

        // Headers
        csv.push_str("Title,Username,Password,Email,Website,Notes,Tags,Type\n");

        for cred in credentials {
            let username = cred
                .get_field("username")
                .map(|f| f.value.as_str())
                .unwrap_or("");
            let password = cred
                .get_field("password")
                .map(|f| f.value.as_str())
                .unwrap_or("");
            let email = cred
                .get_field("email")
                .map(|f| f.value.as_str())
                .unwrap_or("");
            let website = cred
                .get_field("website")
                .or_else(|| cred.get_field("url"))
                .map(|f| f.value.as_str())
                .unwrap_or("");
            let notes = cred.notes.as_deref().unwrap_or("");
            let tags = cred.tags.join(";");

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                cred.title.replace('"', "\"\""),
                username.replace('"', "\"\""),
                password.replace('"', "\"\""),
                email.replace('"', "\"\""),
                website.replace('"', "\"\""),
                notes.replace('"', "\"\""),
                tags.replace('"', "\"\""),
                cred.credential_type.replace('"', "\"\""),
            ));
        }

        csv
    }

    /// Search credentials using various criteria
    pub fn search_credentials<'a>(
        credentials: &'a [CredentialRecord],
        query: &str,
        search_fields: bool,
        search_tags: bool,
        search_notes: bool,
    ) -> Vec<&'a CredentialRecord> {
        let query_lower = query.to_lowercase();

        credentials
            .iter()
            .filter(|cred| {
                // Search in title (always enabled)
                if cred.title.to_lowercase().contains(&query_lower) {
                    return true;
                }

                // Search in credential type
                if cred.credential_type.to_lowercase().contains(&query_lower) {
                    return true;
                }

                // Search in fields
                if search_fields {
                    for (field_name, field) in &cred.fields {
                        if field_name.to_lowercase().contains(&query_lower)
                            || (!field.sensitive
                                && field.value.to_lowercase().contains(&query_lower))
                        {
                            return true;
                        }
                    }
                }

                // Search in tags
                if search_tags {
                    for tag in &cred.tags {
                        if tag.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }
                }

                // Search in notes
                if search_notes {
                    if let Some(notes) = &cred.notes {
                        if notes.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }
                }

                false
            })
            .collect()
    }
}

impl CredentialExport {
    /// Create a new export
    pub fn new(credentials: Vec<CredentialRecord>) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "ziplock".to_string());
        metadata.insert(
            "total_credentials".to_string(),
            credentials.len().to_string(),
        );

        Self {
            version: "1.0".to_string(),
            exported_at: SystemTime::now(),
            credentials,
            metadata,
        }
    }

    /// Add metadata to the export
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_utils_create_from_pattern_all_types() {
        // Test all credential types from specification
        let test_cases = vec![
            ("login", "My Login", "login"),
            ("website", "My Website", "login"),
            ("credit_card", "My Card", "credit_card"),
            ("card", "My Credit Card", "credit_card"),
            ("secure_note", "My Note", "secure_note"),
            ("note", "My Secure Note", "secure_note"),
            ("identity", "My Identity", "identity"),
            ("personal", "My Personal Info", "identity"),
            ("password", "My Password", "password"),
            ("document", "My Document", "document"),
            ("file", "My File", "document"),
            ("ssh_key", "My SSH Key", "ssh_key"),
            ("ssh", "My SSH", "ssh_key"),
            ("bank_account", "My Bank Account", "bank_account"),
            ("bank", "My Bank", "bank_account"),
            ("api_credentials", "My API", "api_credentials"),
            ("api", "My API Creds", "api_credentials"),
            ("crypto_wallet", "My Wallet", "crypto_wallet"),
            ("wallet", "My Crypto Wallet", "crypto_wallet"),
            ("crypto", "My Crypto", "crypto_wallet"),
            ("database", "My Database", "database"),
            ("db", "My DB", "database"),
            ("software_license", "My License", "software_license"),
            ("license", "My Software License", "software_license"),
        ];

        for (pattern, title, expected_type) in test_cases {
            let result = CredentialUtils::create_from_pattern(pattern, title.to_string());
            assert!(
                result.is_some(),
                "Failed to create credential for pattern: {}",
                pattern
            );

            let cred = result.unwrap();
            assert_eq!(cred.title, title);
            assert_eq!(cred.credential_type, expected_type);
            assert!(
                !cred.fields.is_empty(),
                "No fields for pattern: {}",
                pattern
            );
        }

        // Test unknown pattern returns None
        let unknown = CredentialUtils::create_from_pattern("unknown_pattern", "Test".to_string());
        assert!(unknown.is_none());
    }

    #[test]
    fn test_password_generation() {
        let password = CredentialUtils::generate_password(12, false);
        assert_eq!(password.len(), 12);
        assert!(password.chars().all(|c| c.is_alphanumeric()));

        let password_with_symbols = CredentialUtils::generate_password(16, true);
        assert_eq!(password_with_symbols.len(), 16);
    }

    #[test]
    fn test_url_normalization() {
        assert_eq!(
            CredentialUtils::normalize_url("https://www.example.com/path"),
            "example.com"
        );
        assert_eq!(
            CredentialUtils::normalize_url("http://example.com"),
            "example.com"
        );
        assert_eq!(
            CredentialUtils::normalize_url("www.example.com"),
            "example.com"
        );
    }

    #[test]
    fn test_duplicate_detection() {
        let mut cred1 = CredentialRecord::new("Example".to_string(), "login".to_string());
        cred1.set_field("website", CredentialField::url("https://example.com"));

        let mut cred2 = CredentialRecord::new("Example Site".to_string(), "login".to_string());
        cred2.set_field("website", CredentialField::url("https://www.example.com"));

        assert!(CredentialUtils::are_duplicates(&cred1, &cred2));
    }

    #[test]
    fn test_weak_password_detection() {
        let mut cred1 = CredentialRecord::new("Test1".to_string(), "login".to_string());
        cred1.set_field("password", CredentialField::password("weak"));

        let mut cred2 = CredentialRecord::new("Test2".to_string(), "login".to_string());
        cred2.set_field("password", CredentialField::password("SuperSecure123!"));

        let credentials = vec![cred1, cred2];
        let weak = CredentialUtils::find_weak_passwords(&credentials);

        assert_eq!(weak.len(), 1);
    }

    #[test]
    fn test_duplicate_password_detection() {
        let mut cred1 = CredentialRecord::new("Test1".to_string(), "login".to_string());
        cred1.set_field("password", CredentialField::password("same_password"));

        let mut cred2 = CredentialRecord::new("Test2".to_string(), "login".to_string());
        cred2.set_field("password", CredentialField::password("same_password"));

        let credentials = vec![cred1, cred2];
        let duplicates = CredentialUtils::find_duplicate_passwords(&credentials);

        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].len(), 2);
    }

    #[test]
    fn test_csv_import_export() {
        let csv_data = "Title,Username,Password,Email,Website\nTest Site,testuser,testpass,test@example.com,https://example.com";

        let credentials = CredentialUtils::import_from_csv(csv_data).unwrap();
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].title, "Test Site");

        let exported = CredentialUtils::export_to_csv(&credentials);
        assert!(exported.contains("Test Site"));
        assert!(exported.contains("testuser"));
    }

    #[test]
    fn test_credential_search() {
        let mut cred1 = CredentialRecord::new("GitHub".to_string(), "login".to_string());
        cred1.set_field("username", CredentialField::username("developer"));
        cred1.tags.push("coding".to_string());

        let mut cred2 = CredentialRecord::new("Gmail".to_string(), "login".to_string());
        cred2.set_field("email", CredentialField::email("user@gmail.com"));

        let credentials = vec![cred1, cred2];

        // Search by title
        let results = CredentialUtils::search_credentials(&credentials, "git", false, false, false);
        assert_eq!(results.len(), 1);

        // Search by field
        let results =
            CredentialUtils::search_credentials(&credentials, "developer", true, false, false);
        assert_eq!(results.len(), 1);

        // Search by tag
        let results =
            CredentialUtils::search_credentials(&credentials, "coding", false, true, false);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_credential_stats() {
        let mut cred1 = CredentialRecord::new("Test1".to_string(), "login".to_string());
        cred1.tags.push("work".to_string());

        let mut cred2 = CredentialRecord::new("Test2".to_string(), "credit_card".to_string());
        cred2.tags.push("personal".to_string());

        let credentials = vec![cred1, cred2];
        let stats = CredentialUtils::generate_stats(&credentials);

        assert_eq!(stats.total_credentials, 2);
        assert_eq!(stats.by_type.get("login"), Some(&1));
        assert_eq!(stats.by_type.get("credit_card"), Some(&1));
        assert_eq!(stats.by_tag.get("work"), Some(&1));
        assert_eq!(stats.by_tag.get("personal"), Some(&1));
    }
}
