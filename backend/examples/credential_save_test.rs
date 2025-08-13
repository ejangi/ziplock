//! Test program to reproduce the credential field saving issue
//!
//! This program tests the complete save/load cycle for credentials to verify
//! that field values are properly preserved when saving to and loading from
//! the encrypted archive.

use std::collections::HashMap;

use tempfile::TempDir;

use ziplock_backend::config::Config;
use ziplock_backend::storage::ArchiveManager;
use ziplock_shared::models::{CredentialField, CredentialRecord, FieldType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== ZipLock Credential Save/Load Test ===\n");

    // Create temporary directory for test
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("test_archive.7z");
    let master_password = "test_password_123456789";

    println!("Test archive path: {:?}", archive_path);
    println!("Master password: {}", master_password);
    println!();

    // Create config with increased file lock timeout
    let mut config = Config::default();
    config.storage.file_lock_timeout = 120; // 2 minutes
    println!(
        "File lock timeout set to: {} seconds",
        config.storage.file_lock_timeout
    );

    // Test 1: Create archive and add credential
    println!("Step 1: Creating archive and adding credential...");
    let archive_manager = ArchiveManager::new(config.storage.clone())?;

    // Create the archive
    archive_manager
        .create_archive(&archive_path, master_password)
        .await?;
    println!("✓ Archive created successfully");

    // Open the archive after creation
    archive_manager
        .open_archive(&archive_path, master_password)
        .await?;
    println!("✓ Archive opened successfully");

    // Create a test credential with multiple field types
    let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());

    // Add various field types with test values
    let mut fields = HashMap::new();

    fields.insert(
        "username".to_string(),
        CredentialField {
            field_type: FieldType::Username,
            value: "testuser@example.com".to_string(),
            sensitive: false,
            label: Some("Username".to_string()),
            metadata: HashMap::new(),
        },
    );

    fields.insert(
        "password".to_string(),
        CredentialField {
            field_type: FieldType::Password,
            value: "super_secure_password_123".to_string(),
            sensitive: true,
            label: Some("Password".to_string()),
            metadata: HashMap::new(),
        },
    );

    fields.insert(
        "website".to_string(),
        CredentialField {
            field_type: FieldType::Url,
            value: "https://example.com/login".to_string(),
            sensitive: false,
            label: Some("Website".to_string()),
            metadata: HashMap::new(),
        },
    );

    fields.insert(
        "notes".to_string(),
        CredentialField {
            field_type: FieldType::TextArea,
            value: "This is a test note with some important information".to_string(),
            sensitive: false,
            label: Some("Notes".to_string()),
            metadata: HashMap::new(),
        },
    );

    credential.fields = fields.clone();
    credential.tags = vec!["test".to_string(), "login".to_string()];
    credential.notes = Some("Test credential for debugging".to_string());

    println!("Original credential fields:");
    for (name, field) in &credential.fields {
        println!(
            "  {}: {} (type: {:?}, sensitive: {})",
            name, field.value, field.field_type, field.sensitive
        );
    }
    println!();

    // Add the credential
    let credential_id = archive_manager.add_credential(credential.clone()).await?;
    println!("✓ Credential added with ID: {}", credential_id);

    // Save the archive
    println!("Saving archive...");
    archive_manager.save_archive().await?;
    println!("✓ Archive saved");

    // Close the archive
    archive_manager.close_archive().await?;
    println!("✓ Archive closed");
    println!();

    // Test 2: Reopen archive and load credential
    println!("Step 2: Reopening archive and loading credential...");

    let archive_manager2 = ArchiveManager::new(config.storage.clone())?;

    // Open the archive
    archive_manager2
        .open_archive(&archive_path, master_password)
        .await?;
    println!("✓ Archive reopened successfully");

    // Load the credential
    let loaded_credential = archive_manager2.get_credential(&credential_id).await?;
    println!("✓ Credential loaded: '{}'", loaded_credential.title);

    println!("Loaded credential fields:");
    for (name, field) in &loaded_credential.fields {
        println!(
            "  {}: {} (type: {:?}, sensitive: {})",
            name, field.value, field.field_type, field.sensitive
        );
    }
    println!();

    // Test 3: Compare original vs loaded
    println!("Step 3: Comparing original vs loaded credential...");

    let mut all_tests_passed = true;

    // Check basic properties
    if loaded_credential.title != credential.title {
        println!(
            "✗ Title mismatch: '{}' != '{}'",
            loaded_credential.title, credential.title
        );
        all_tests_passed = false;
    } else {
        println!("✓ Title matches");
    }

    if loaded_credential.credential_type != credential.credential_type {
        println!(
            "✗ Type mismatch: '{}' != '{}'",
            loaded_credential.credential_type, credential.credential_type
        );
        all_tests_passed = false;
    } else {
        println!("✓ Credential type matches");
    }

    if loaded_credential.tags != credential.tags {
        println!(
            "✗ Tags mismatch: {:?} != {:?}",
            loaded_credential.tags, credential.tags
        );
        all_tests_passed = false;
    } else {
        println!("✓ Tags match");
    }

    if loaded_credential.notes != credential.notes {
        println!(
            "✗ Notes mismatch: {:?} != {:?}",
            loaded_credential.notes, credential.notes
        );
        all_tests_passed = false;
    } else {
        println!("✓ Notes match");
    }

    // Check field count
    if loaded_credential.fields.len() != credential.fields.len() {
        println!(
            "✗ Field count mismatch: {} != {}",
            loaded_credential.fields.len(),
            credential.fields.len()
        );
        all_tests_passed = false;
    } else {
        println!("✓ Field count matches ({})", loaded_credential.fields.len());
    }

    // Check each field
    for (field_name, original_field) in &credential.fields {
        match loaded_credential.fields.get(field_name) {
            Some(loaded_field) => {
                if loaded_field.value != original_field.value {
                    println!(
                        "✗ Field '{}' value mismatch: '{}' != '{}'",
                        field_name, loaded_field.value, original_field.value
                    );
                    all_tests_passed = false;
                } else if loaded_field.field_type != original_field.field_type {
                    println!(
                        "✗ Field '{}' type mismatch: {:?} != {:?}",
                        field_name, loaded_field.field_type, original_field.field_type
                    );
                    all_tests_passed = false;
                } else if loaded_field.sensitive != original_field.sensitive {
                    println!(
                        "✗ Field '{}' sensitivity mismatch: {} != {}",
                        field_name, loaded_field.sensitive, original_field.sensitive
                    );
                    all_tests_passed = false;
                } else {
                    println!("✓ Field '{}' matches perfectly", field_name);
                }
            }
            None => {
                println!("✗ Field '{}' missing in loaded credential", field_name);
                all_tests_passed = false;
            }
        }
    }

    // Check for extra fields in loaded credential
    for field_name in loaded_credential.fields.keys() {
        if !credential.fields.contains_key(field_name) {
            println!("✗ Extra field '{}' found in loaded credential", field_name);
            all_tests_passed = false;
        }
    }

    println!();

    // Test 4: Update credential and verify
    println!("Step 4: Testing credential update...");

    let mut updated_credential = loaded_credential.clone();

    // Modify some field values
    if let Some(username_field) = updated_credential.fields.get_mut("username") {
        username_field.value = "updated_user@example.com".to_string();
    }

    if let Some(password_field) = updated_credential.fields.get_mut("password") {
        password_field.value = "new_super_secure_password_456".to_string();
    }

    // Add a new field
    updated_credential.fields.insert(
        "email".to_string(),
        CredentialField {
            field_type: FieldType::Email,
            value: "user@company.com".to_string(),
            sensitive: false,
            label: Some("Email".to_string()),
            metadata: HashMap::new(),
        },
    );

    updated_credential.title = "Updated Test Login".to_string();

    println!("Updating credential with new values...");
    for (name, field) in &updated_credential.fields {
        println!("  {}: {}", name, field.value);
    }

    // Update the credential
    archive_manager2
        .update_credential(&credential_id, updated_credential.clone())
        .await?;
    println!("✓ Credential updated");

    // Save the archive
    println!("Saving updated archive...");
    archive_manager2.save_archive().await?;
    println!("✓ Updated archive saved");

    // Load it again to verify the update
    let reloaded_credential = archive_manager2.get_credential(&credential_id).await?;
    println!("✓ Credential reloaded");

    println!("Reloaded credential fields:");
    for (name, field) in &reloaded_credential.fields {
        println!("  {}: {}", name, field.value);
    }

    // Verify the updates
    let mut update_tests_passed = true;

    if reloaded_credential.title != updated_credential.title {
        println!(
            "✗ Updated title mismatch: '{}' != '{}'",
            reloaded_credential.title, updated_credential.title
        );
        update_tests_passed = false;
    } else {
        println!("✓ Updated title matches");
    }

    for (field_name, updated_field) in &updated_credential.fields {
        match reloaded_credential.fields.get(field_name) {
            Some(reloaded_field) => {
                if reloaded_field.value != updated_field.value {
                    println!(
                        "✗ Updated field '{}' value mismatch: '{}' != '{}'",
                        field_name, reloaded_field.value, updated_field.value
                    );
                    update_tests_passed = false;
                } else {
                    println!("✓ Updated field '{}' matches", field_name);
                }
            }
            None => {
                println!(
                    "✗ Updated field '{}' missing in reloaded credential",
                    field_name
                );
                update_tests_passed = false;
            }
        }
    }

    println!();

    // Final results
    println!("=== Test Results ===");
    if all_tests_passed && update_tests_passed {
        println!("🎉 ALL TESTS PASSED!");
        println!("Credential fields are being saved and loaded correctly.");
    } else {
        println!("❌ SOME TESTS FAILED!");
        if !all_tests_passed {
            println!("- Initial save/load test failed");
        }
        if !update_tests_passed {
            println!("- Update test failed");
        }
        println!("There appears to be an issue with credential field persistence.");
    }

    // Clean up
    archive_manager2.close_archive().await?;

    Ok(())
}
