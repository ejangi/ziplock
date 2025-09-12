//! Simple in-memory credential store for Linux app
//!
//! This module provides a basic credential storage mechanism that works
//! directly with extracted archive files, avoiding FFI deadlocks in async contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// A simple credential record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCredential {
    pub id: String,
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub credential_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub fields: HashMap<String, String>,
}

/// Simple credential store that works with extracted files
#[derive(Debug)]
pub struct CredentialStore {
    credentials: Arc<Mutex<HashMap<String, SimpleCredential>>>,
    is_unlocked: Arc<Mutex<bool>>,
    archive_path: Arc<Mutex<Option<String>>>,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(HashMap::new())),
            is_unlocked: Arc::new(Mutex::new(false)),
            archive_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Load credentials from extracted archive files
    #[allow(dead_code)]
    pub fn load_from_extracted_files(
        &self,
        extracted_files: HashMap<String, Vec<u8>>,
    ) -> Result<usize, String> {
        info!(
            "Loading credentials from {} extracted files",
            extracted_files.len()
        );

        let mut credentials = self.credentials.lock().unwrap();
        credentials.clear();

        let mut loaded_count = 0;

        // Look for credential files in the credentials directory
        for (file_path, content) in extracted_files {
            if file_path.starts_with("credentials/") && file_path.ends_with(".yml") {
                match self.parse_credential_file(&file_path, &content) {
                    Ok(credential) => {
                        debug!(
                            "Loaded credential: {} ({})",
                            credential.title, credential.id
                        );
                        credentials.insert(credential.id.clone(), credential);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to parse credential file {}: {}", file_path, e);
                    }
                }
            }
        }

        // Mark as unlocked if we found any files (even if no credentials)
        *self.is_unlocked.lock().unwrap() = true;

        info!(
            "Successfully loaded {} credentials from archive",
            loaded_count
        );
        Ok(loaded_count)
    }

    /// Parse a credential file
    #[allow(dead_code)]
    fn parse_credential_file(
        &self,
        file_path: &str,
        content: &[u8],
    ) -> Result<SimpleCredential, String> {
        let content_str =
            std::str::from_utf8(content).map_err(|e| format!("Invalid UTF-8: {}", e))?;

        // Try to parse as YAML first
        if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(content_str) {
            return self.parse_yaml_credential(file_path, &yaml_value);
        }

        // If YAML parsing fails, create a basic credential from the filename
        let credential_id = file_path
            .strip_prefix("credentials/")
            .unwrap_or(file_path)
            .strip_suffix(".yml")
            .unwrap_or(file_path)
            .to_string();

        Ok(SimpleCredential {
            id: credential_id.clone(),
            title: credential_id,
            username: None,
            password: None,
            url: None,
            notes: Some(content_str.to_string()),
            credential_type: "unknown".to_string(),
            created_at: "unknown".to_string(),
            updated_at: "unknown".to_string(),
            fields: HashMap::new(),
        })
    }

    /// Parse YAML credential data
    #[allow(dead_code)]
    fn parse_yaml_credential(
        &self,
        file_path: &str,
        yaml: &serde_yaml::Value,
    ) -> Result<SimpleCredential, String> {
        let credential_id = file_path
            .strip_prefix("credentials/")
            .unwrap_or(file_path)
            .strip_suffix(".yml")
            .unwrap_or(file_path)
            .to_string();

        let title = yaml
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(&credential_id)
            .to_string();

        let username = yaml
            .get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let password = yaml
            .get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let url = yaml
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let notes = yaml
            .get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let credential_type = yaml
            .get("type")
            .or_else(|| yaml.get("credential_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("login")
            .to_string();

        let created_at = yaml
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let updated_at = yaml
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse additional fields
        let mut fields = HashMap::new();
        if let Some(fields_yaml) = yaml.get("fields") {
            if let Some(fields_map) = fields_yaml.as_mapping() {
                for (key, value) in fields_map {
                    if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                        fields.insert(key_str.to_string(), value_str.to_string());
                    }
                }
            }
        }

        Ok(SimpleCredential {
            id: credential_id,
            title,
            username,
            password,
            url,
            notes,
            credential_type,
            created_at,
            updated_at,
            fields,
        })
    }

    /// Check if the store is unlocked
    pub fn is_unlocked(&self) -> bool {
        *self.is_unlocked.lock().unwrap()
    }

    /// Get all credentials
    #[allow(dead_code)]
    pub fn list_credentials(&self) -> Vec<SimpleCredential> {
        if !self.is_unlocked() {
            return Vec::new();
        }

        let credentials = self.credentials.lock().unwrap();
        credentials.values().cloned().collect()
    }

    /// Get a specific credential by ID
    #[allow(dead_code)]
    pub fn get_credential(&self, id: &str) -> Option<SimpleCredential> {
        if !self.is_unlocked() {
            return None;
        }

        let credentials = self.credentials.lock().unwrap();
        credentials.get(id).cloned()
    }

    /// Add or update a credential
    #[allow(dead_code)]
    pub fn save_credential(&self, credential: SimpleCredential) -> Result<(), String> {
        if !self.is_unlocked() {
            return Err("Store is locked".to_string());
        }

        let mut credentials = self.credentials.lock().unwrap();
        credentials.insert(credential.id.clone(), credential);
        Ok(())
    }

    /// Delete a credential
    #[allow(dead_code)]
    pub fn delete_credential(&self, id: &str) -> Result<(), String> {
        if !self.is_unlocked() {
            return Err("Store is locked".to_string());
        }

        let mut credentials = self.credentials.lock().unwrap();
        credentials.remove(id);
        Ok(())
    }

    /// Lock the store
    pub fn lock(&self) {
        *self.is_unlocked.lock().unwrap() = false;
        self.credentials.lock().unwrap().clear();
        info!("Credential store locked");
    }

    /// Set the archive path
    #[allow(dead_code)]
    pub fn set_archive_path(&self, path: Option<String>) {
        *self.archive_path.lock().unwrap() = path;
    }

    /// Get the archive path
    #[allow(dead_code)]
    pub fn get_archive_path(&self) -> Option<String> {
        self.archive_path.lock().unwrap().clone()
    }

    /// Get credential count
    #[allow(dead_code)]
    pub fn credential_count(&self) -> usize {
        if !self.is_unlocked() {
            return 0;
        }
        self.credentials.lock().unwrap().len()
    }

    /// Search credentials by title or username
    #[allow(dead_code)]
    pub fn search_credentials(&self, query: &str) -> Vec<SimpleCredential> {
        if !self.is_unlocked() {
            return Vec::new();
        }

        let credentials = self.credentials.lock().unwrap();
        let query_lower = query.to_lowercase();

        credentials
            .values()
            .filter(|cred| {
                cred.title.to_lowercase().contains(&query_lower)
                    || cred
                        .username
                        .as_ref()
                        .map_or(false, |u| u.to_lowercase().contains(&query_lower))
                    || cred
                        .url
                        .as_ref()
                        .map_or(false, |url| url.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get credentials by type
    #[allow(dead_code)]
    pub fn get_credentials_by_type(&self, credential_type: &str) -> Vec<SimpleCredential> {
        if !self.is_unlocked() {
            return Vec::new();
        }

        let credentials = self.credentials.lock().unwrap();
        credentials
            .values()
            .filter(|cred| cred.credential_type == credential_type)
            .cloned()
            .collect()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Global credential store instance
static CREDENTIAL_STORE: std::sync::OnceLock<CredentialStore> = std::sync::OnceLock::new();

/// Get the global credential store instance
pub fn get_credential_store() -> &'static CredentialStore {
    CREDENTIAL_STORE.get_or_init(CredentialStore::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_store_creation() {
        let store = CredentialStore::new();
        assert!(!store.is_unlocked());
        assert_eq!(store.credential_count(), 0);
    }

    #[test]
    fn test_yaml_credential_parsing() {
        let store = CredentialStore::new();

        let yaml_content = r#"
title: "Test Login"
username: "testuser"
password: "testpass"
url: "https://example.com"
type: "login"
created_at: "2024-01-01T00:00:00Z"
updated_at: "2024-01-01T00:00:00Z"
fields:
  note: "Test note"
"#;

        let credential = store
            .parse_credential_file("credentials/test.yml", yaml_content.as_bytes())
            .unwrap();

        assert_eq!(credential.title, "Test Login");
        assert_eq!(credential.username.unwrap(), "testuser");
        assert_eq!(credential.password.unwrap(), "testpass");
        assert_eq!(credential.url.unwrap(), "https://example.com");
        assert_eq!(credential.credential_type, "login");
        assert_eq!(credential.fields.get("note").unwrap(), "Test note");
    }

    #[test]
    fn test_load_from_extracted_files() {
        let store = CredentialStore::new();

        let mut extracted_files = HashMap::new();
        extracted_files.insert(
            "credentials/test1.yml".to_string(),
            b"title: \"Test 1\"\nusername: \"user1\"".to_vec(),
        );
        extracted_files.insert(
            "credentials/test2.yml".to_string(),
            b"title: \"Test 2\"\nusername: \"user2\"".to_vec(),
        );
        extracted_files.insert("README.md".to_string(), b"This is a readme".to_vec());

        let result = store.load_from_extracted_files(extracted_files).unwrap();
        assert_eq!(result, 2); // Only credential files should be loaded
        assert!(store.is_unlocked());
        assert_eq!(store.credential_count(), 2);

        let credentials = store.list_credentials();
        assert_eq!(credentials.len(), 2);
    }

    #[test]
    fn test_search_functionality() {
        let store = CredentialStore::new();

        let mut extracted_files = HashMap::new();
        extracted_files.insert(
            "credentials/gmail.yml".to_string(),
            b"title: \"Gmail Account\"\nusername: \"user@gmail.com\"\nurl: \"https://gmail.com\""
                .to_vec(),
        );
        extracted_files.insert(
            "credentials/work.yml".to_string(),
            b"title: \"Work Login\"\nusername: \"employee@company.com\"\nurl: \"https://company.com\"".to_vec()
        );

        store.load_from_extracted_files(extracted_files).unwrap();

        let gmail_results = store.search_credentials("gmail");
        assert_eq!(gmail_results.len(), 1);
        assert_eq!(gmail_results[0].title, "Gmail Account");

        let email_results = store.search_credentials("@gmail.com");
        assert_eq!(email_results.len(), 1);

        let all_results = store.search_credentials("com");
        assert_eq!(all_results.len(), 2); // Both have .com in URL or username
    }
}
