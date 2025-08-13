//! Integration tests for credential persistence and data integrity
//!
//! This module tests the complete flow of credential operations to ensure
//! that changes are properly saved to the backend repository and persist
//! across archive close/reopen cycles.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::sleep;
use uuid::Uuid;

use ziplock_shared::models::{CredentialField, CredentialRecord, FieldType};
use ziplock_backend::storage::ArchiveManager;
use ziplock_backend::config::StorageConfig;

/// Test fixture for credential persistence testing
struct CredentialPersistenceTest {
    temp_dir: TempDir,
    archive_path: PathBuf,
    master_password: String,
    storage_config: StorageConfig,
}

impl CredentialPersistenceTest {
    /// Create a new test fixture
    async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let archive_path = temp_dir.path().join("test_credentials.7z");
        let master_password = "test-master-password-123".to_string();

        let storage_config = StorageConfig {
            backup_count: 0, // Disable backups for testing
            auto_backup: false,
            compression_level: 1, // Fast compression for testing
            solid_compression: false,
            multi_threaded_compression: false,
            file_lock_timeout: Duration::from_secs(5),
        };

        Ok(Self {
            temp_dir,
            archive_path,
            master_password,
            storage_config,
        })
    }

    /// Create a test credential with sample data
    fn create_test_credential(id: Option<String>) -> CredentialRecord {
        let mut credential = CredentialRecord::new(
            "Test Login Credential".to_string(),
            "login".to_string(),
        );

        if let Some(id) = id {
            credential.id = id;
        }

        // Add some test fields
        credential.fields.insert(
            "username".to_string(),
            CredentialField {
                field_type: FieldType::Username,
                value: "testuser@example.com".to_string(),
                sensitive: false,
                label: Some("Username".to_string()),
                metadata: HashMap::new(),
            },
        );

        credential.fields.insert(
            "password".to_string(),
            CredentialField {
                field_type: FieldType::Password,
                value: "super-secret-password-123".to_string(),
                sensitive: true,
                label: Some("Password".to_string()),
                metadata: HashMap::new(),
            },
        );

        credential.fields.insert(
            "url".to_string(),
            CredentialField {
                field_type: FieldType::Url,
                value: "https://example.com/login".to_string(),
                sensitive: false,
                label: Some("Website URL".to_string()),
                metadata: HashMap::new(),
            },
        );

        credential.tags = vec!["test".to_string(), "integration".to_string()];
        credential.notes = Some("This is a test credential for integration testing".to_string());

        credential
    }

    /// Create an updated version of a credential
    fn create_updated_credential(original: &CredentialRecord) -> CredentialRecord {
        let mut updated = original.clone();
        updated.title = "Updated Test Login Credential".to_string();

        // Update the password field
        if let Some(password_field) = updated.fields.get_mut("password") {
            password_field.value = "new-super-secret-password-456".to_string();
        }

        // Add a new field
        updated.fields.insert(
            "security_question".to_string(),
            CredentialField {
                field_type: FieldType::Text,
                value: "What is your favorite color?".to_string(),
                sensitive: false,
                label: Some("Security Question".to_string()),
                metadata: HashMap::new(),
            },
        );

        updated.tags.push("updated".to_string());
        updated.notes = Some("This credential has been updated during integration testing".to_string());

        updated
    }
}

/// Test that credentials are properly created and persisted
#[tokio::test]
async fn test_credential_creation_persistence() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;
    let credential = CredentialPersistenceTest::create_test_credential(None);

    // Create archive and add credential
    let archive_manager = ArchiveManager::new(test.storage_config.clone());
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    let credential_id = archive_manager.add_credential(credential.clone()).await?;

    // Save the archive
    archive_manager.save_archive().await?;
    archive_manager.close_archive().await?;

    // Reopen archive and verify credential exists
    archive_manager
        .open_archive(&test.archive_path, &test.master_password)
        .await?;

    let retrieved_credential = archive_manager.get_credential(&credential_id).await?;

    // Verify all fields match
    assert_eq!(retrieved_credential.title, credential.title);
    assert_eq!(retrieved_credential.credential_type, credential.credential_type);
    assert_eq!(retrieved_credential.fields.len(), credential.fields.len());
    assert_eq!(retrieved_credential.tags, credential.tags);
    assert_eq!(retrieved_credential.notes, credential.notes);

    // Verify specific fields
    assert_eq!(
        retrieved_credential.fields["username"].value,
        "testuser@example.com"
    );
    assert_eq!(
        retrieved_credential.fields["password"].value,
        "super-secret-password-123"
    );
    assert_eq!(
        retrieved_credential.fields["url"].value,
        "https://example.com/login"
    );

    Ok(())
}

/// Test that credential updates are properly persisted
#[tokio::test]
async fn test_credential_update_persistence() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;
    let original_credential = CredentialPersistenceTest::create_test_credential(None);

    // Create archive and add credential
    let archive_manager = ArchiveManager::new(test.storage_config.clone());
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    let credential_id = archive_manager.add_credential(original_credential.clone()).await?;
    archive_manager.save_archive().await?;

    // Update the credential
    let updated_credential = CredentialPersistenceTest::create_updated_credential(&original_credential);
    archive_manager
        .update_credential(&credential_id, updated_credential.clone())
        .await?;

    // Save and close archive
    archive_manager.save_archive().await?;
    archive_manager.close_archive().await?;

    // Reopen archive and verify updates persisted
    archive_manager
        .open_archive(&test.archive_path, &test.master_password)
        .await?;

    let retrieved_credential = archive_manager.get_credential(&credential_id).await?;

    // Verify updates were persisted
    assert_eq!(retrieved_credential.title, "Updated Test Login Credential");
    assert_eq!(
        retrieved_credential.fields["password"].value,
        "new-super-secret-password-456"
    );
    assert!(retrieved_credential.fields.contains_key("security_question"));
    assert_eq!(
        retrieved_credential.fields["security_question"].value,
        "What is your favorite color?"
    );
    assert!(retrieved_credential.tags.contains(&"updated".to_string()));
    assert_eq!(
        retrieved_credential.notes,
        Some("This credential has been updated during integration testing".to_string())
    );

    Ok(())
}

/// Test that multiple credential operations are properly persisted
#[tokio::test]
async fn test_multiple_credential_operations_persistence() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;

    // Create archive
    let archive_manager = ArchiveManager::new(test.storage_config.clone());
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    // Create multiple credentials
    let mut credential_ids = Vec::new();
    for i in 0..3 {
        let mut credential = CredentialPersistenceTest::create_test_credential(None);
        credential.title = format!("Test Credential {}", i + 1);
        credential.fields.get_mut("username").unwrap().value = format!("user{}@example.com", i + 1);

        let id = archive_manager.add_credential(credential).await?;
        credential_ids.push(id);
    }

    // Update one credential
    let updated_credential = {
        let original = archive_manager.get_credential(&credential_ids[1]).await?;
        CredentialPersistenceTest::create_updated_credential(&original)
    };
    archive_manager
        .update_credential(&credential_ids[1], updated_credential)
        .await?;

    // Delete one credential
    archive_manager.delete_credential(&credential_ids[2]).await?;

    // Save and close archive
    archive_manager.save_archive().await?;
    archive_manager.close_archive().await?;

    // Reopen and verify state
    archive_manager
        .open_archive(&test.archive_path, &test.master_password)
        .await?;

    let all_credentials = archive_manager.list_credentials().await?;
    assert_eq!(all_credentials.len(), 2); // One deleted, two remain

    // Verify first credential unchanged
    let first_credential = archive_manager.get_credential(&credential_ids[0]).await?;
    assert_eq!(first_credential.title, "Test Credential 1");
    assert_eq!(
        first_credential.fields["username"].value,
        "user1@example.com"
    );

    // Verify second credential was updated
    let second_credential = archive_manager.get_credential(&credential_ids[1]).await?;
    assert_eq!(second_credential.title, "Updated Test Login Credential");
    assert!(second_credential.tags.contains(&"updated".to_string()));

    // Verify third credential was deleted
    assert!(archive_manager.get_credential(&credential_ids[2]).await.is_err());

    Ok(())
}

/// Test credential persistence with special characters and edge cases
#[tokio::test]
async fn test_credential_persistence_edge_cases() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;

    // Create archive
    let archive_manager = ArchiveManager::new(test.storage_config.clone());
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    // Create credential with special characters and edge cases
    let mut credential = CredentialRecord::new(
        "Test with Special Chars: éñ中文🔐".to_string(),
        "custom".to_string(),
    );

    // Add fields with various edge cases
    credential.fields.insert(
        "empty_field".to_string(),
        CredentialField {
            field_type: FieldType::Text,
            value: "".to_string(),
            sensitive: false,
            label: Some("Empty Field".to_string()),
            metadata: HashMap::new(),
        },
    );

    credential.fields.insert(
        "very_long_field".to_string(),
        CredentialField {
            field_type: FieldType::TextArea,
            value: "A".repeat(10000), // Very long text
            sensitive: true,
            label: Some("Very Long Field".to_string()),
            metadata: HashMap::new(),
        },
    );

    credential.fields.insert(
        "special_chars".to_string(),
        CredentialField {
            field_type: FieldType::Text,
            value: "Special: !@#$%^&*(){}[]|\\:;\"'<>?,./ éñ中文🔐".to_string(),
            sensitive: false,
            label: Some("Special Characters".to_string()),
            metadata: HashMap::new(),
        },
    );

    credential.tags = vec![
        "tag with spaces".to_string(),
        "tag-with-dashes".to_string(),
        "éñ中文🔐".to_string(),
    ];

    credential.notes = Some("Notes with\nmultiple\nlines\nand special chars: éñ中文🔐".to_string());

    let credential_id = archive_manager.add_credential(credential.clone()).await?;

    // Save and close archive
    archive_manager.save_archive().await?;
    archive_manager.close_archive().await?;

    // Reopen and verify all edge cases are preserved
    archive_manager
        .open_archive(&test.archive_path, &test.master_password)
        .await?;

    let retrieved_credential = archive_manager.get_credential(&credential_id).await?;

    assert_eq!(retrieved_credential.title, credential.title);
    assert_eq!(retrieved_credential.fields["empty_field"].value, "");
    assert_eq!(retrieved_credential.fields["very_long_field"].value.len(), 10000);
    assert_eq!(
        retrieved_credential.fields["special_chars"].value,
        "Special: !@#$%^&*(){}[]|\\:;\"'<>?,./ éñ中文🔐"
    );
    assert_eq!(retrieved_credential.tags, credential.tags);
    assert_eq!(retrieved_credential.notes, credential.notes);

    Ok(())
}

/// Test that archive integrity is maintained after multiple save/load cycles
#[tokio::test]
async fn test_archive_integrity_multiple_cycles() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;
    let archive_manager = ArchiveManager::new(test.storage_config.clone());

    // Create archive with initial credential
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    let mut credential = CredentialPersistenceTest::create_test_credential(None);
    let credential_id = archive_manager.add_credential(credential.clone()).await?;
    archive_manager.save_archive().await?;

    // Perform multiple save/load cycles with modifications
    for cycle in 0..5 {
        // Close and reopen archive
        archive_manager.close_archive().await?;
        archive_manager
            .open_archive(&test.archive_path, &test.master_password)
            .await?;

        // Modify credential
        credential.title = format!("Credential after cycle {}", cycle + 1);
        credential.fields.get_mut("password").unwrap().value =
            format!("password-cycle-{}", cycle + 1);

        archive_manager
            .update_credential(&credential_id, credential.clone())
            .await?;

        archive_manager.save_archive().await?;

        // Verify modification persisted
        let retrieved = archive_manager.get_credential(&credential_id).await?;
        assert_eq!(retrieved.title, credential.title);
        assert_eq!(
            retrieved.fields["password"].value,
            format!("password-cycle-{}", cycle + 1)
        );
    }

    Ok(())
}

/// Test concurrent access and data consistency (single-threaded simulation)
#[tokio::test]
async fn test_credential_consistency_simulation() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;
    let archive_manager = ArchiveManager::new(test.storage_config.clone());

    // Create archive
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    // Simulate rapid consecutive operations
    let mut credential_ids = Vec::new();

    // Rapid creation
    for i in 0..10 {
        let mut credential = CredentialPersistenceTest::create_test_credential(None);
        credential.title = format!("Rapid Credential {}", i);

        let id = archive_manager.add_credential(credential).await?;
        credential_ids.push(id);

        // Small delay to simulate realistic timing
        sleep(Duration::from_millis(10)).await;
    }

    // Rapid updates
    for (i, id) in credential_ids.iter().enumerate() {
        let mut credential = archive_manager.get_credential(id).await?;
        credential.title = format!("Updated Rapid Credential {}", i);

        archive_manager.update_credential(id, credential).await?;
        sleep(Duration::from_millis(10)).await;
    }

    // Save and verify consistency
    archive_manager.save_archive().await?;
    archive_manager.close_archive().await?;

    // Reopen and verify all operations persisted correctly
    archive_manager
        .open_archive(&test.archive_path, &test.master_password)
        .await?;

    let all_credentials = archive_manager.list_credentials().await?;
    assert_eq!(all_credentials.len(), 10);

    for (i, id) in credential_ids.iter().enumerate() {
        let credential = archive_manager.get_credential(id).await?;
        assert_eq!(credential.title, format!("Updated Rapid Credential {}", i));
    }

    Ok(())
}

/// Integration test helper to verify archive file structure
#[tokio::test]
async fn test_archive_file_structure_validation() -> Result<()> {
    let test = CredentialPersistenceTest::new().await?;
    let archive_manager = ArchiveManager::new(test.storage_config.clone());

    // Create archive with credentials
    archive_manager
        .create_archive(&test.archive_path, &test.master_password)
        .await?;

    let credential = CredentialPersistenceTest::create_test_credential(None);
    let credential_id = archive_manager.add_credential(credential).await?;
    archive_manager.save_archive().await?;
    archive_manager.close_archive().await?;

    // Verify archive file exists and has content
    assert!(test.archive_path.exists());
    let metadata = std::fs::metadata(&test.archive_path)?;
    assert!(metadata.len() > 0);

    // Verify we can validate the repository format
    assert!(archive_manager.validate_archive_file(&test.archive_path).await.is_ok());

    Ok(())
}
