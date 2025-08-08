//! Simple integration test to verify credential persistence
//!
//! This test verifies that credentials are properly saved to the repository
//! after being created or updated, and can be retrieved after archive
//! close/reopen cycles.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

use ziplock_shared::models::{CredentialField, CredentialRecord, FieldType};
use ziplock_backend::storage::ArchiveManager;
use ziplock_backend::config::StorageConfig;

/// Test that a newly created credential is properly saved and can be retrieved
#[tokio::test]
async fn test_credential_save_and_retrieve() {
    // Setup test environment
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let archive_path = temp_dir.path().join("test.7z");
    let master_password = "test-password-123";

    let config = StorageConfig {
        backup_count: 0,
        auto_backup: false,
        compression_level: 1,
        solid_compression: false,
        multi_threaded_compression: false,
        file_lock_timeout: Duration::from_secs(5),
    };

    // Create archive manager
    let archive_manager = ArchiveManager::new(config);

    // Create new archive
    archive_manager
        .create_archive(&archive_path, master_password)
        .await
        .expect("Failed to create archive");

    // Create a test credential
    let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());

    credential.fields.insert(
        "username".to_string(),
        CredentialField {
            field_type: FieldType::Username,
            value: "testuser".to_string(),
            sensitive: false,
            label: Some("Username".to_string()),
            metadata: HashMap::new(),
        },
    );

    credential.fields.insert(
        "password".to_string(),
        CredentialField {
            field_type: FieldType::Password,
            value: "secret123".to_string(),
            sensitive: true,
            label: Some("Password".to_string()),
            metadata: HashMap::new(),
        },
    );

    // Add credential to archive
    let credential_id = archive_manager
        .add_credential(credential.clone())
        .await
        .expect("Failed to add credential");

    // Save the archive (this is the key step that was missing)
    archive_manager
        .save_archive()
        .await
        .expect("Failed to save archive");

    // Close the archive
    archive_manager
        .close_archive()
        .await
        .expect("Failed to close archive");

    // Reopen the archive (simulating app restart)
    archive_manager
        .open_archive(&archive_path, master_password)
        .await
        .expect("Failed to reopen archive");

    // Retrieve the credential
    let retrieved_credential = archive_manager
        .get_credential(&credential_id)
        .await
        .expect("Failed to retrieve credential");

    // Verify the credential data was preserved
    assert_eq!(retrieved_credential.title, "Test Login");
    assert_eq!(retrieved_credential.credential_type, "login");
    assert_eq!(retrieved_credential.fields.len(), 2);
    assert_eq!(retrieved_credential.fields["username"].value, "testuser");
    assert_eq!(retrieved_credential.fields["password"].value, "secret123");

    println!("✅ Credential save and retrieve test passed!");
}

/// Test that credential updates are properly saved
#[tokio::test]
async fn test_credential_update_persistence() {
    // Setup test environment
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let archive_path = temp_dir.path().join("test_update.7z");
    let master_password = "test-password-456";

    let config = StorageConfig {
        backup_count: 0,
        auto_backup: false,
        compression_level: 1,
        solid_compression: false,
        multi_threaded_compression: false,
        file_lock_timeout: Duration::from_secs(5),
    };

    let archive_manager = ArchiveManager::new(config);

    // Create archive and initial credential
    archive_manager
        .create_archive(&archive_path, master_password)
        .await
        .expect("Failed to create archive");

    let mut credential = CredentialRecord::new("Original Title".to_string(), "login".to_string());
    credential.fields.insert(
        "password".to_string(),
        CredentialField {
            field_type: FieldType::Password,
            value: "original-password".to_string(),
            sensitive: true,
            label: Some("Password".to_string()),
            metadata: HashMap::new(),
        },
    );

    let credential_id = archive_manager
        .add_credential(credential)
        .await
        .expect("Failed to add credential");

    archive_manager
        .save_archive()
        .await
        .expect("Failed to save archive");

    // Update the credential
    let mut updated_credential = CredentialRecord::new("Updated Title".to_string(), "login".to_string());
    updated_credential.id = credential_id.clone();
    updated_credential.fields.insert(
        "password".to_string(),
        CredentialField {
            field_type: FieldType::Password,
            value: "new-password-123".to_string(),
            sensitive: true,
            label: Some("Password".to_string()),
            metadata: HashMap::new(),
        },
    );

    archive_manager
        .update_credential(&credential_id, updated_credential)
        .await
        .expect("Failed to update credential");

    // Save after update (this is critical!)
    archive_manager
        .save_archive()
        .await
        .expect("Failed to save archive after update");

    // Close and reopen
    archive_manager.close_archive().await.expect("Failed to close archive");
    archive_manager
        .open_archive(&archive_path, master_password)
        .await
        .expect("Failed to reopen archive");

    // Verify the updates were saved
    let retrieved_credential = archive_manager
        .get_credential(&credential_id)
        .await
        .expect("Failed to retrieve updated credential");

    assert_eq!(retrieved_credential.title, "Updated Title");
    assert_eq!(retrieved_credential.fields["password"].value, "new-password-123");

    println!("✅ Credential update persistence test passed!");
}

/// Test to verify the bug exists without auto-save
#[tokio::test]
async fn test_verify_bug_without_save() {
    // This test demonstrates the bug when save is not called
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let archive_path = temp_dir.path().join("test_bug.7z");
    let master_password = "test-password-bug";

    let config = StorageConfig {
        backup_count: 0,
        auto_backup: false,
        compression_level: 1,
        solid_compression: false,
        multi_threaded_compression: false,
        file_lock_timeout: Duration::from_secs(5),
    };

    let archive_manager = ArchiveManager::new(config);

    // Create archive and credential
    archive_manager
        .create_archive(&archive_path, master_password)
        .await
        .expect("Failed to create archive");

    let mut credential = CredentialRecord::new("Test Credential".to_string(), "login".to_string());
    credential.fields.insert(
        "username".to_string(),
        CredentialField {
            field_type: FieldType::Username,
            value: "testuser".to_string(),
            sensitive: false,
            label: Some("Username".to_string()),
            metadata: HashMap::new(),
        },
    );

    let credential_id = archive_manager
        .add_credential(credential)
        .await
        .expect("Failed to add credential");

    // DON'T save the archive - this simulates the bug
    // archive_manager.save_archive().await.expect("Failed to save archive");

    // Close and reopen
    archive_manager.close_archive().await.expect("Failed to close archive");
    archive_manager
        .open_archive(&archive_path, master_password)
        .await
        .expect("Failed to reopen archive");

    // Try to retrieve the credential - this should fail because it wasn't saved
    let result = archive_manager.get_credential(&credential_id).await;

    // This assertion verifies the bug exists - the credential should not be found
    // because it was never saved to disk
    assert!(result.is_err(), "Expected credential to not be found (bug verification)");

    println!("✅ Bug verification test passed - credential was lost as expected without save!");
}
