use crate::core::memory_repository::UnifiedMemoryRepository;
use crate::ffi::mobile::*;
use crate::models::{CredentialField, CredentialRecord, FieldType};
use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;

#[test]
fn test_list_credentials_serialization() {
    println!("=== Testing list_credentials serialization ===");

    // Create a memory repository
    let mut repo = UnifiedMemoryRepository::new();
    repo.initialize().expect("Failed to initialize repository");

    // Create a test credential with username field
    let mut credential = CredentialRecord::new("Test Credential".to_string(), "login".to_string());
    credential.set_field(
        "username",
        CredentialField::new("test@example.com", FieldType::Username, false),
    );
    credential.set_field(
        "password",
        CredentialField::new("testpass", FieldType::Password, true),
    );

    // Add the credential
    repo.add_credential(credential)
        .expect("Failed to add credential");

    // Test list_credentials
    let credentials = repo.list_credentials().expect("Failed to list credentials");
    println!(
        "DEBUG: list_credentials returned {} items",
        credentials.len()
    );

    if let Some(first_cred) = credentials.first() {
        println!("DEBUG: First credential ID: '{}'", first_cred.id);
        println!("DEBUG: First credential title: '{}'", first_cred.title);
        println!(
            "DEBUG: First credential fields: {:?}",
            first_cred.fields.keys().collect::<Vec<_>>()
        );

        // Test serialization
        match serde_json::to_string(&credentials) {
            Ok(json) => {
                println!("DEBUG: Serialized JSON: {}", json);

                // Verify it starts with array of objects, not tuples
                if json.starts_with("[{") {
                    println!("✅ Serialization produces array of objects (correct)");
                } else if json.starts_with("[[") {
                    println!("❌ Serialization produces array of arrays (incorrect - tuples)");
                } else {
                    println!(
                        "⚠️  Unexpected serialization format: {}",
                        &json[..50.min(json.len())]
                    );
                }
            }
            Err(e) => {
                println!("❌ Serialization failed: {}", e);
            }
        }
    }

    // Test list_credential_summaries for comparison
    let summaries = repo
        .list_credential_summaries()
        .expect("Failed to list summaries");
    println!(
        "DEBUG: list_credential_summaries returned {} items",
        summaries.len()
    );

    match serde_json::to_string(&summaries) {
        Ok(json) => {
            println!("DEBUG: Summaries JSON: {}", json);
            if json.starts_with("[[") {
                println!("✅ Summaries correctly produce array of arrays (tuples)");
            }
        }
        Err(e) => {
            println!("❌ Summaries serialization failed: {}", e);
        }
    }
}

#[test]
fn test_ffi_list_credentials_direct() {
    println!("=== Testing FFI list_credentials directly ===");

    // Create repository handle
    let handle = unsafe { ziplock_mobile_repository_create() };
    assert!(!handle.is_null(), "Failed to create repository handle");

    // Initialize repository
    let init_result = unsafe { ziplock_mobile_repository_initialize(handle) };
    assert_eq!(init_result, 0, "Failed to initialize repository");

    // Create test credential JSON
    let test_credential = r#"{
        "id": "test-id-123",
        "title": "Test Login",
        "credential_type": "login",
        "fields": {
            "username": {
                "value": "testuser@example.com",
                "field_type": "Username",
                "sensitive": false,
                "metadata": {}
            },
            "password": {
                "value": "testpassword",
                "field_type": "Password",
                "sensitive": true,
                "metadata": {}
            }
        },
        "tags": [],
        "notes": null,
        "created_at": 1694000000,
        "updated_at": 1694000000,
        "accessed_at": 1694000000,
        "favorite": false,
        "folder_path": null
    }"#;

    // Add the credential
    let add_result = unsafe {
        let c_str = std::ffi::CString::new(test_credential).unwrap();
        ziplock_mobile_add_credential(handle, c_str.as_ptr())
    };
    assert_eq!(add_result, 0, "Failed to add credential");

    // Test list_credentials
    let list_result = unsafe { ziplock_mobile_list_credentials(handle) };
    assert!(!list_result.is_null(), "list_credentials returned null");

    let c_str = unsafe { CStr::from_ptr(list_result) };
    let json_str = c_str.to_str().expect("Invalid UTF-8");

    println!("DEBUG: FFI list_credentials JSON: {}", json_str);

    if json_str.starts_with("[{") {
        println!("✅ FFI list_credentials produces array of objects (correct)");
    } else if json_str.starts_with("[[") {
        println!("❌ FFI list_credentials produces array of arrays (incorrect - tuples)");
    } else {
        println!(
            "⚠️  FFI unexpected format: {}",
            &json_str[..50.min(json_str.len())]
        );
    }

    // Clean up
    unsafe {
        ziplock_mobile_free_string(list_result);
        ziplock_mobile_repository_destroy(handle);
    }
}
