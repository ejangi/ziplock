//! Archive Persistence Integration Test
//!
//! This test validates that the unified architecture can create encrypted 7z archives
//! on disk with credentials, save them, and then successfully load them back with
//! all data integrity preserved.

use std::path::PathBuf;
use ziplock_shared::core::{DesktopFileProvider, UnifiedRepositoryManager};
use ziplock_shared::models::{CredentialField, CredentialRecord};
use ziplock_shared::utils::generate_totp;

/// Test fixture for archive persistence tests
struct ArchivePersistenceTest {
    archive_path: PathBuf,
}

impl ArchivePersistenceTest {
    fn new() -> Self {
        Self::with_name("test_vault")
    }

    fn with_name(name: &str) -> Self {
        // Create tests/results directory if it doesn't exist (from shared package, go to project root)
        let results_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Should have parent directory")
            .join("tests")
            .join("results");
        std::fs::create_dir_all(&results_dir).expect("Failed to create tests/results directory");

        // Generate unique filename to avoid conflicts
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let thread_id = std::thread::current().id();
        let archive_path = results_dir.join(format!("{}_{:?}_{}.7z", name, thread_id, timestamp));

        Self { archive_path }
    }

    fn archive_path_str(&self) -> &str {
        self.archive_path.to_str().expect("Invalid path")
    }

    fn create_test_credentials() -> Vec<CredentialRecord> {
        let mut credentials = Vec::new();

        // Create a login credential with various field types
        let mut login_cred =
            CredentialRecord::new("Gmail Account".to_string(), "login".to_string());
        login_cred.set_field("username", CredentialField::username("user@gmail.com"));
        login_cred.set_field("password", CredentialField::password("SecurePassword123!"));
        login_cred.set_field("url", CredentialField::url("https://mail.google.com"));
        login_cred.set_field("notes", CredentialField::text("Primary email account"));
        login_cred.add_tag("email".to_string());
        login_cred.add_tag("work".to_string());
        credentials.push(login_cred);

        // Create a credit card credential
        let mut card_cred =
            CredentialRecord::new("Chase Visa".to_string(), "credit_card".to_string());
        card_cred.set_field("cardholder", CredentialField::text("John Doe"));
        card_cred.set_field(
            "number",
            CredentialField::new(
                ziplock_shared::models::FieldType::CreditCardNumber,
                "4111-1111-1111-1111".to_string(),
                true,
            ),
        );
        card_cred.set_field(
            "expiry",
            CredentialField::new(
                ziplock_shared::models::FieldType::ExpiryDate,
                "12/25".to_string(),
                false,
            ),
        );
        card_cred.set_field(
            "cvv",
            CredentialField::new(
                ziplock_shared::models::FieldType::Cvv,
                "123".to_string(),
                true,
            ),
        );
        card_cred.add_tag("finance".to_string());
        credentials.push(card_cred);

        // Create a TOTP-enabled credential
        let mut totp_cred = CredentialRecord::new("GitHub".to_string(), "login".to_string());
        totp_cred.set_field("username", CredentialField::username("developer"));
        totp_cred.set_field("password", CredentialField::password("GitHubPass2023"));
        totp_cred.set_field(
            "totp_secret",
            CredentialField::totp_secret("JBSWY3DPEHPK3PXP"),
        );
        totp_cred.set_field("url", CredentialField::url("https://github.com"));
        totp_cred.add_tag("development".to_string());
        totp_cred.favorite = true;
        credentials.push(totp_cred);

        // Create a secure note
        let mut note_cred =
            CredentialRecord::new("Server SSH Keys".to_string(), "secure_note".to_string());
        note_cred.set_field("private_key", CredentialField::new(
            ziplock_shared::models::FieldType::TextArea,
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEF...\n-----END PRIVATE KEY-----".to_string(),
            true,
        ));
        note_cred.set_field(
            "public_key",
            CredentialField::new(
                ziplock_shared::models::FieldType::TextArea,
                "ssh-rsa AAAAB3NzaC1yc2EAAAA...".to_string(),
                false,
            ),
        );
        note_cred.set_field("server", CredentialField::text("production-server-01"));
        note_cred.add_tag("infrastructure".to_string());
        note_cred.add_tag("ssh".to_string());
        credentials.push(note_cred);

        credentials
    }
}

impl Drop for ArchivePersistenceTest {
    fn drop(&mut self) {
        // Clean up test archive file
        if self.archive_path.exists() {
            let _ = std::fs::remove_file(&self.archive_path);
        }
    }
}

#[test]
fn test_create_archive_with_credentials() {
    let test = ArchivePersistenceTest::with_name("create_archive");
    let file_provider = DesktopFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(file_provider);

    // Create repository
    manager
        .create_repository(test.archive_path_str(), "test_master_password")
        .expect("Failed to create repository");

    assert!(manager.is_open());

    // Add test credentials
    let credentials = ArchivePersistenceTest::create_test_credentials();
    for credential in &credentials {
        manager
            .add_credential(credential.clone())
            .expect("Failed to add credential");
    }

    // Save repository
    manager
        .save_repository()
        .expect("Failed to save repository");

    // Verify archive file exists
    assert!(
        test.archive_path.exists(),
        "Archive file should exist on disk"
    );
    assert!(
        test.archive_path.metadata().unwrap().len() > 0,
        "Archive file should not be empty"
    );

    // Verify credentials in memory
    let loaded_credentials = manager
        .list_credentials()
        .expect("Failed to list credentials");

    assert_eq!(
        loaded_credentials.len(),
        4,
        "Should have 4 credentials in memory"
    );

    // Close repository
    manager.close_repository(true);
    assert!(!manager.is_open());
}

#[test]
fn test_load_archive_and_validate_credentials() {
    let test = ArchivePersistenceTest::with_name("load_archive");

    // First, create and save an archive
    {
        let file_provider = DesktopFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(file_provider);
        manager
            .create_repository(test.archive_path_str(), "test_master_password")
            .expect("Failed to create repository");

        let credentials = ArchivePersistenceTest::create_test_credentials();
        for credential in &credentials {
            manager
                .add_credential(credential.clone())
                .expect("Failed to add credential");
        }

        manager
            .save_repository()
            .expect("Failed to save repository");

        manager.close_repository(true);
    }

    // Now load the archive and validate
    let file_provider = DesktopFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(file_provider);

    manager
        .open_repository(test.archive_path_str(), "test_master_password")
        .expect("Failed to open repository");

    assert!(manager.is_open());

    // Validate all credentials were loaded
    let loaded_credentials = manager
        .list_credentials()
        .expect("Failed to list credentials");

    assert_eq!(
        loaded_credentials.len(),
        4,
        "Should have 4 credentials loaded"
    );

    // Validate specific credential data
    let gmail_cred = loaded_credentials
        .iter()
        .find(|c| c.title == "Gmail Account")
        .expect("Gmail credential should exist");

    assert_eq!(gmail_cred.credential_type, "login");
    assert!(gmail_cred.has_tag("email"));
    assert!(gmail_cred.has_tag("work"));

    let username_field = gmail_cred
        .get_field("username")
        .expect("Username field should exist");
    assert_eq!(username_field.value, "user@gmail.com");

    let password_field = gmail_cred
        .get_field("password")
        .expect("Password field should exist");
    assert_eq!(password_field.value, "SecurePassword123!");
    assert!(password_field.sensitive);

    // Validate credit card credential
    let card_cred = loaded_credentials
        .iter()
        .find(|c| c.title == "Chase Visa")
        .expect("Credit card credential should exist");

    assert_eq!(card_cred.credential_type, "credit_card");
    assert!(card_cred.has_tag("finance"));

    let card_number = card_cred
        .get_field("number")
        .expect("Card number should exist");
    assert_eq!(card_number.value, "4111-1111-1111-1111");
    assert!(card_number.sensitive);

    // Validate TOTP credential and generate code
    let github_cred = loaded_credentials
        .iter()
        .find(|c| c.title == "GitHub")
        .expect("GitHub credential should exist");

    assert!(github_cred.favorite);
    assert!(github_cred.has_tag("development"));

    let totp_secret = github_cred
        .get_field("totp_secret")
        .expect("TOTP secret should exist");
    assert_eq!(totp_secret.value, "JBSWY3DPEHPK3PXP");

    // Generate TOTP code to verify secret is valid
    let totp_code =
        generate_totp(&totp_secret.value, 30).expect("Should be able to generate TOTP code");
    assert_eq!(totp_code.len(), 6);
    assert!(totp_code.chars().all(|c| c.is_ascii_digit()));

    // Validate secure note
    let ssh_cred = loaded_credentials
        .iter()
        .find(|c| c.title == "Server SSH Keys")
        .expect("SSH credential should exist");

    assert_eq!(ssh_cred.credential_type, "secure_note");
    assert!(ssh_cred.has_tag("infrastructure"));
    assert!(ssh_cred.has_tag("ssh"));

    let private_key = ssh_cred
        .get_field("private_key")
        .expect("Private key should exist");
    assert!(private_key.value.contains("BEGIN PRIVATE KEY"));
    assert!(private_key.sensitive);

    let public_key = ssh_cred
        .get_field("public_key")
        .expect("Public key should exist");
    assert!(public_key.value.starts_with("ssh-rsa"));
    assert!(!public_key.sensitive);

    manager.close_repository(true);
}

#[test]
fn test_archive_persistence_across_sessions() {
    let test = ArchivePersistenceTest::with_name("persistence_sessions");
    let credentials = ArchivePersistenceTest::create_test_credentials();
    let original_credential_id = credentials[0].id.clone();

    // Session 1: Create and save
    {
        let file_provider = DesktopFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(file_provider);

        manager
            .create_repository(test.archive_path_str(), "session_test_password")
            .expect("Failed to create repository");

        for credential in &credentials {
            manager
                .add_credential(credential.clone())
                .expect("Failed to add credential");
        }

        manager
            .save_repository()
            .expect("Failed to save repository");

        manager.close_repository(true);
    }

    // Session 2: Load, modify, and save
    {
        let file_provider = DesktopFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(file_provider);

        manager
            .open_repository(test.archive_path_str(), "session_test_password")
            .expect("Failed to open repository");

        // Modify existing credential
        let mut gmail_cred = manager
            .get_credential(&original_credential_id)
            .expect("Failed to get credential")
            .clone();

        gmail_cred.set_field("backup_email", CredentialField::email("backup@example.com"));
        gmail_cred.add_tag("modified".to_string());

        manager
            .update_credential(gmail_cred)
            .expect("Failed to update credential");

        // Add a new credential
        let mut new_cred = CredentialRecord::new("New Service".to_string(), "login".to_string());
        new_cred.set_field("username", CredentialField::username("newuser"));
        new_cred.set_field("password", CredentialField::password("NewPassword123"));

        let new_cred_id = new_cred.id.clone();
        manager
            .add_credential(new_cred)
            .expect("Failed to add new credential");

        manager
            .save_repository()
            .expect("Failed to save repository");

        manager.close_repository(true);
    }

    // Session 3: Load and verify changes persisted
    {
        let file_provider = DesktopFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(file_provider);

        manager
            .open_repository(test.archive_path_str(), "session_test_password")
            .expect("Failed to open repository");

        let loaded_credentials = manager
            .list_credentials()
            .expect("Failed to list credentials");

        assert_eq!(
            loaded_credentials.len(),
            5,
            "Should have 5 credentials after modification"
        );

        // Verify original credential was modified
        let modified_cred = manager
            .get_credential_readonly(&original_credential_id)
            .expect("Failed to get credential");

        assert!(modified_cred.has_tag("modified"));
        assert!(modified_cred.get_field("backup_email").is_some());

        let backup_email = modified_cred.get_field("backup_email").unwrap();
        assert_eq!(backup_email.value, "backup@example.com");

        // Verify new credential exists
        let new_credentials: Vec<_> = loaded_credentials
            .iter()
            .filter(|c| c.title == "New Service")
            .collect();

        assert_eq!(new_credentials.len(), 1, "New credential should exist");
        assert_eq!(
            new_credentials[0].get_field("username").unwrap().value,
            "newuser"
        );

        manager.close_repository(true);
    }
}

#[test]
fn test_wrong_password_fails() {
    let test = ArchivePersistenceTest::with_name("wrong_password");

    // Create archive with correct password
    {
        let file_provider = DesktopFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(file_provider);
        manager
            .create_repository(test.archive_path_str(), "correct_password")
            .expect("Failed to create repository");

        let credential = CredentialRecord::new("Test Cred".to_string(), "login".to_string());
        manager
            .add_credential(credential)
            .expect("Failed to add credential");

        manager
            .save_repository()
            .expect("Failed to save repository");

        manager.close_repository(true);
    }

    // Try to open with wrong password
    {
        let file_provider = DesktopFileProvider::new();
        let mut manager = UnifiedRepositoryManager::new(file_provider);

        let result = manager.open_repository(test.archive_path_str(), "wrong_password");

        assert!(result.is_err(), "Should fail with wrong password");
        assert!(
            !manager.is_open(),
            "Manager should not be open after failed authentication"
        );
    }
}

#[test]
fn test_archive_integrity_validation() {
    let test = ArchivePersistenceTest::with_name("integrity_validation");
    let file_provider = DesktopFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(file_provider);

    // Create repository with credentials
    manager
        .create_repository(test.archive_path_str(), "integrity_test")
        .expect("Failed to create repository");

    let credentials = ArchivePersistenceTest::create_test_credentials();
    for credential in &credentials {
        manager
            .add_credential(credential.clone())
            .expect("Failed to add credential");
    }

    manager
        .save_repository()
        .expect("Failed to save repository");

    // Verify repository integrity
    let integrity_issues = manager
        .verify_integrity()
        .expect("Failed to verify integrity");

    assert!(
        integrity_issues.is_empty(),
        "Repository integrity should be valid: {:?}",
        integrity_issues
    );

    // Get statistics
    let stats = manager.get_stats().expect("Failed to get stats");
    assert_eq!(stats.credential_count, 4);
    assert!(stats.metadata.credential_count > 0);

    manager.close_repository(true);
}

#[cfg(test)]
mod test_utils {
    use super::*;

    /// Helper to create a minimal credential for testing
    pub fn create_minimal_credential(title: &str) -> CredentialRecord {
        let mut cred = CredentialRecord::new(title.to_string(), "test".to_string());
        cred.set_field("test_field", CredentialField::text("test_value"));
        cred
    }

    /// Helper to verify basic credential properties
    pub fn verify_credential_basics(cred: &CredentialRecord, expected_title: &str) {
        assert_eq!(cred.title, expected_title);
        assert!(!cred.id.is_empty());
        assert!(cred.created_at > 0);
        assert!(cred.updated_at > 0);
    }
}

// Additional test for edge cases and error conditions
#[test]
fn test_edge_cases() {
    let test = ArchivePersistenceTest::with_name("edge_cases");
    let file_provider = DesktopFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(file_provider);

    // Create repository
    manager
        .create_repository(test.archive_path_str(), "edge_case_test")
        .expect("Failed to create repository");

    // Test credential with special characters
    let mut special_cred = CredentialRecord::new(
        "Spéciál Çhárâctërs & Émojis 🔒".to_string(),
        "login".to_string(),
    );
    special_cred.set_field("username", CredentialField::username("üser@dömain.com"));
    special_cred.set_field("password", CredentialField::password("P@ssw0rd!@#$%^&*()"));
    special_cred.set_field(
        "notes",
        CredentialField::text("Notes with\nmultiple\nlines and symbols: !@#$%^&*()"),
    );

    manager
        .add_credential(special_cred.clone())
        .expect("Failed to add credential with special characters");

    // Test empty credential (minimum viable)
    let mut empty_cred = CredentialRecord::new("Empty".to_string(), "test".to_string());
    empty_cred.notes = Some("".to_string());

    manager
        .add_credential(empty_cred)
        .expect("Failed to add minimal credential");

    // Save and reload
    manager
        .save_repository()
        .expect("Failed to save repository with edge case credentials");

    manager.close_repository(true);

    // Reload and verify
    let file_provider = DesktopFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(file_provider);

    manager
        .open_repository(test.archive_path_str(), "edge_case_test")
        .expect("Failed to reopen repository");

    let loaded_credentials = manager
        .list_credentials()
        .expect("Failed to list credentials");

    assert_eq!(loaded_credentials.len(), 2);

    // Verify special characters preserved
    let special_loaded = loaded_credentials
        .iter()
        .find(|c| c.title.contains("Spéciál"))
        .expect("Special character credential should exist");

    assert_eq!(special_loaded.title, "Spéciál Çhárâctërs & Émojis 🔒");
    assert_eq!(
        special_loaded.get_field("username").unwrap().value,
        "üser@dömain.com"
    );
    assert_eq!(
        special_loaded.get_field("password").unwrap().value,
        "P@ssw0rd!@#$%^&*()"
    );

    manager.close_repository(true);
}
