//! Simple credential test to isolate the save/load issue
//!
//! This test focuses on the in-memory credential operations first,
//! then gradually adds file operations to isolate where the problem occurs.

use std::collections::HashMap;
use tempfile::TempDir;
use tokio;
use ziplock_backend::config::Config;
use ziplock_backend::storage::ArchiveManager;
use ziplock_shared::models::{CredentialField, CredentialRecord, FieldType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Simple ZipLock Credential Test ===\n");

    // Create temporary directory for test
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("simple_test.7z");
    let master_password = "test123456789";

    println!("Test archive path: {:?}", archive_path);
    println!("Master password: {}", master_password);
    println!();

    // Create config with reduced complexity
    let mut config = Config::default();
    config.storage.auto_backup = false; // Disable backup to reduce complexity
    config.storage.compression.level = 1; // Use minimal compression
    config.storage.compression.solid = false; // Disable solid compression

    println!("Test 1: Create and open archive...");
    let archive_manager = ArchiveManager::new(config.storage.clone())?;

    // Create the archive
    archive_manager
        .create_archive(&archive_path, master_password)
        .await?;
    println!("✓ Archive created");

    // Open the archive
    archive_manager
        .open_archive(&archive_path, master_password)
        .await?;
    println!("✓ Archive opened");

    println!("\nTest 2: Create simple credential...");

    // Create a very simple credential
    let mut credential = CredentialRecord::new("Simple Test".to_string(), "login".to_string());

    // Add just two basic fields
    let mut fields = HashMap::new();

    fields.insert(
        "username".to_string(),
        CredentialField {
            field_type: FieldType::Username,
            value: "testuser".to_string(),
            sensitive: false,
            label: Some("Username".to_string()),
            metadata: HashMap::new(),
        },
    );

    fields.insert(
        "password".to_string(),
        CredentialField {
            field_type: FieldType::Password,
            value: "testpass".to_string(),
            sensitive: true,
            label: Some("Password".to_string()),
            metadata: HashMap::new(),
        },
    );

    credential.fields = fields;

    println!("Creating credential with fields:");
    for (name, field) in &credential.fields {
        println!("  {}: {} (type: {:?})", name, field.value, field.field_type);
    }

    // Add the credential
    let credential_id = archive_manager.add_credential(credential.clone()).await?;
    println!("✓ Credential added with ID: {}", credential_id);

    println!("\nTest 3: Read credential back from memory...");

    // Get the credential back (should work from memory)
    let loaded_credential = archive_manager.get_credential(&credential_id).await?;
    println!(
        "✓ Credential loaded from memory: '{}'",
        loaded_credential.title
    );

    println!("Loaded fields:");
    for (name, field) in &loaded_credential.fields {
        println!("  {}: {} (type: {:?})", name, field.value, field.field_type);
    }

    // Compare fields
    let mut memory_test_passed = true;
    for (field_name, original_field) in &credential.fields {
        match loaded_credential.fields.get(field_name) {
            Some(loaded_field) => {
                if loaded_field.value != original_field.value {
                    println!(
                        "✗ Memory test failed: field '{}' value mismatch",
                        field_name
                    );
                    memory_test_passed = false;
                } else {
                    println!("✓ Field '{}' matches in memory", field_name);
                }
            }
            None => {
                println!("✗ Memory test failed: field '{}' missing", field_name);
                memory_test_passed = false;
            }
        }
    }

    if memory_test_passed {
        println!("✓ Memory operations working correctly!");
    } else {
        println!("✗ Memory operations have issues!");
        return Ok(());
    }

    println!("\nTest 4: Manual save operation (without auto-backup)...");

    // Try saving without closing first
    println!("Attempting to save archive...");
    match archive_manager.save_archive().await {
        Ok(_) => println!("✓ Archive saved successfully"),
        Err(e) => {
            println!("✗ Save failed: {}", e);
            return Err(e.into());
        }
    }

    println!("\nTest 5: Close and reopen to test persistence...");

    // Close the archive
    archive_manager.close_archive().await?;
    println!("✓ Archive closed");

    // Create a new archive manager instance
    let archive_manager2 = ArchiveManager::new(config.storage.clone())?;

    // Reopen the archive
    archive_manager2
        .open_archive(&archive_path, master_password)
        .await?;
    println!("✓ Archive reopened");

    // Load the credential again
    let reloaded_credential = archive_manager2.get_credential(&credential_id).await?;
    println!("✓ Credential reloaded: '{}'", reloaded_credential.title);

    println!("Reloaded fields:");
    for (name, field) in &reloaded_credential.fields {
        println!("  {}: {} (type: {:?})", name, field.value, field.field_type);
    }

    // Final comparison
    let mut persistence_test_passed = true;
    for (field_name, original_field) in &credential.fields {
        match reloaded_credential.fields.get(field_name) {
            Some(reloaded_field) => {
                if reloaded_field.value != original_field.value {
                    println!(
                        "✗ Persistence test failed: field '{}' value mismatch: '{}' != '{}'",
                        field_name, reloaded_field.value, original_field.value
                    );
                    persistence_test_passed = false;
                } else {
                    println!("✓ Field '{}' persisted correctly", field_name);
                }
            }
            None => {
                println!(
                    "✗ Persistence test failed: field '{}' missing after reload",
                    field_name
                );
                persistence_test_passed = false;
            }
        }
    }

    println!("\n=== Final Results ===");
    if memory_test_passed && persistence_test_passed {
        println!("🎉 ALL TESTS PASSED!");
        println!("Credential fields are being saved and loaded correctly.");
    } else {
        println!("❌ TESTS FAILED!");
        if !memory_test_passed {
            println!("- Memory operations failed");
        }
        if !persistence_test_passed {
            println!("- Persistence operations failed");
            println!("- The issue is in the save/load cycle");
        }
    }

    // Clean up
    archive_manager2.close_archive().await?;

    Ok(())
}
