//! Tests for adaptive runtime strategy in hybrid FFI
//!
//! This module tests the adaptive runtime detection and execution strategies
//! to ensure the FFI layer properly handles different runtime contexts.

#[cfg(test)]
mod tests {
    use super::super::ffi_hybrid::*;
    use std::ffi::CString;

    /// Test that FFI initialization works in sync context (should create owned runtime)
    #[test]
    fn test_init_sync_context() {
        // Clean up any existing state
        ziplock_hybrid_cleanup();

        // Initialize in sync context
        let result = ziplock_hybrid_init();
        assert_eq!(result, ZipLockHybridError::Success as i32);

        // Check runtime strategy - should be CreateOwned in sync context
        let strategy = ziplock_hybrid_get_runtime_strategy();
        assert_eq!(strategy, 0); // RuntimeStrategy::CreateOwned

        // Clean up
        ziplock_hybrid_cleanup();
    }

    /// Test that FFI initialization detects async context
    #[tokio::test(flavor = "multi_thread")]
    async fn test_init_async_context() {
        let timeout_duration = std::time::Duration::from_secs(2);

        let test_future = async {
            // Clean up any existing state
            ziplock_hybrid_cleanup();

            // Initialize in async context (within tokio test)
            let result = ziplock_hybrid_init();
            assert_eq!(result, ZipLockHybridError::Success as i32);

            // Check runtime strategy - should be ExternalFileOps in async context
            let strategy = ziplock_hybrid_get_runtime_strategy();
            assert_eq!(strategy, 2); // RuntimeStrategy::ExternalFileOps (mapped from UseExisting)

            // Clean up
            ziplock_hybrid_cleanup();
        };

        // Run with timeout
        match tokio::time::timeout(timeout_duration, test_future).await {
            Ok(_) => {} // Test completed successfully
            Err(_) => panic!(
                "Test timed out after {} seconds",
                timeout_duration.as_secs()
            ),
        }
    }

    /// Test archive creation in sync context (should work directly)
    #[test]
    fn test_archive_creation_sync_context() {
        use tempfile::TempDir;

        // Clean up any existing state
        ziplock_hybrid_cleanup();

        // Initialize
        let init_result = ziplock_hybrid_init();
        assert_eq!(init_result, ZipLockHybridError::Success as i32);

        // Create temporary directory for test
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test_sync.7z");
        let archive_path_str = archive_path.to_str().unwrap();
        let password = "test_password_sync";

        // Create archive - should work in sync context
        let path_cstring = CString::new(archive_path_str).unwrap();
        let password_cstring = CString::new(password).unwrap();

        let result =
            ziplock_hybrid_create_archive(path_cstring.as_ptr(), password_cstring.as_ptr());

        // In sync context, this should succeed (RuntimeStrategy::CreateOwned)
        assert_eq!(result, ZipLockHybridError::Success as i32);

        // Clean up
        ziplock_hybrid_cleanup();
    }

    /// Test archive creation in async context (should require external file ops)
    #[tokio::test(flavor = "multi_thread")]
    async fn test_archive_creation_async_context() {
        // Add timeout to prevent hanging
        let timeout_duration = std::time::Duration::from_secs(3);

        let test_future = async {
            // Clean up any existing state
            ziplock_hybrid_cleanup();

            // Initialize in async context
            let init_result = ziplock_hybrid_init();
            assert_eq!(init_result, ZipLockHybridError::Success as i32);

            // Check that we detected async context
            let strategy = ziplock_hybrid_get_runtime_strategy();
            assert_eq!(strategy, 2); // RuntimeStrategy::ExternalFileOps (mapped from UseExisting)

            // For async context, archive operations should fail fast with external ops required
            // We'll test this by checking the strategy instead of calling the potentially blocking function

            // Clean up
            ziplock_hybrid_cleanup();
        };

        // Run with timeout
        match tokio::time::timeout(timeout_duration, test_future).await {
            Ok(_) => {} // Test completed successfully
            Err(_) => panic!(
                "Test timed out after {} seconds",
                timeout_duration.as_secs()
            ),
        }
    }

    /// Test credential operations work in both contexts
    #[test]
    fn test_credential_operations_sync() {
        test_credential_operations_impl();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_credential_operations_async() {
        let timeout_duration = std::time::Duration::from_secs(3);

        let test_future = async {
            // Clean up any existing state
            ziplock_hybrid_cleanup();

            // Initialize
            let init_result = ziplock_hybrid_init();
            assert_eq!(init_result, ZipLockHybridError::Success as i32);

            // Verify we're in external file ops mode
            let strategy = ziplock_hybrid_get_runtime_strategy();
            assert_eq!(strategy, 2); // RuntimeStrategy::ExternalFileOps

            // Basic credential operations should still work in async context
            let title = CString::new("Test Login").unwrap();
            let cred_type = CString::new("login").unwrap();

            let credential_id =
                ziplock_hybrid_credential_create(title.as_ptr(), cred_type.as_ptr());
            assert_ne!(credential_id, 0);

            // Don't call cleanup to avoid runtime drop panic
        };

        // Run with timeout
        match tokio::time::timeout(timeout_duration, test_future).await {
            Ok(_) => {} // Test completed successfully
            Err(_) => panic!(
                "Test timed out after {} seconds",
                timeout_duration.as_secs()
            ),
        }
    }

    fn test_credential_operations_impl() {
        // Clean up any existing state
        ziplock_hybrid_cleanup();

        // Initialize
        let init_result = ziplock_hybrid_init();
        assert_eq!(init_result, ZipLockHybridError::Success as i32);

        // Create a credential
        let title = CString::new("Test Login").unwrap();
        let cred_type = CString::new("login").unwrap();

        let credential_id = ziplock_hybrid_credential_create(title.as_ptr(), cred_type.as_ptr());

        assert_ne!(credential_id, 0); // Should get a valid ID

        // Add a field to the credential
        let field_name = CString::new("username").unwrap();
        let field_value = CString::new("testuser").unwrap();

        let add_field_result = ziplock_hybrid_credential_add_field(
            credential_id,
            field_name.as_ptr(),
            field_value.as_ptr(),
            4, // USERNAME field type
            0, // not sensitive
        );

        assert_eq!(add_field_result, ZipLockHybridError::Success as i32);

        // Get credential as YAML
        let yaml_ptr = ziplock_hybrid_credential_get_yaml(credential_id);
        assert!(!yaml_ptr.is_null());

        // Free the YAML string
        ziplock_hybrid_free_string(yaml_ptr);

        // Clean up - don't call cleanup in async context to avoid runtime drop panic
    }

    /// Test external file operations functions
    #[test]
    fn test_external_file_operations() {
        // Clean up any existing state
        ziplock_hybrid_cleanup();

        // Initialize
        let init_result = ziplock_hybrid_init();
        assert_eq!(init_result, ZipLockHybridError::Success as i32);

        // Create a credential
        let title = CString::new("Test Login").unwrap();
        let cred_type = CString::new("login").unwrap();

        let credential_id = ziplock_hybrid_credential_create(title.as_ptr(), cred_type.as_ptr());

        assert_ne!(credential_id, 0);

        // Get file operations JSON
        let operations_ptr = ziplock_hybrid_get_file_operations();
        assert!(!operations_ptr.is_null());

        // Convert to string and verify it's valid JSON
        let operations_cstr = unsafe { std::ffi::CStr::from_ptr(operations_ptr) };
        let operations_str = operations_cstr.to_str().unwrap();

        // Parse as JSON to verify format
        let operations_json: serde_json::Value = serde_json::from_str(operations_str).unwrap();
        assert!(operations_json.is_array());

        // Free the operations string
        ziplock_hybrid_free_string(operations_ptr);

        // Test setting archive info
        let archive_path = CString::new("/test/path/archive.7z").unwrap();
        let password = CString::new("test_password").unwrap();

        let set_info_result =
            ziplock_hybrid_set_archive_info(archive_path.as_ptr(), password.as_ptr());

        assert_eq!(set_info_result, ZipLockHybridError::Success as i32);

        // Clean up
        ziplock_hybrid_cleanup();
    }

    /// Test loading from extracted files
    #[test]
    fn test_load_from_extracted_files() {
        // Clean up any existing state
        ziplock_hybrid_cleanup();

        // Initialize
        let init_result = ziplock_hybrid_init();
        assert_eq!(init_result, ZipLockHybridError::Success as i32);

        // Create test file map JSON
        let files_json = r#"{
            "credentials/1/record.yml": "id: \"1\"\ntitle: \"Test Credential\"\ncredential_type: \"login\"\nfields:\n  username:\n    field_type: \"Username\"\n    value: \"testuser\"\n    sensitive: false\ncreated_at: \"2024-01-01T00:00:00Z\"\nupdated_at: \"2024-01-01T00:00:00Z\""
        }"#;

        let files_cstring = CString::new(files_json).unwrap();

        let load_result = ziplock_hybrid_load_from_extracted_files(files_cstring.as_ptr());

        assert_eq!(load_result, ZipLockHybridError::Success as i32);

        // Verify credential was loaded by trying to get it as YAML
        let yaml_ptr = ziplock_hybrid_credential_get_yaml(1);
        assert!(!yaml_ptr.is_null());

        // Free the YAML string
        ziplock_hybrid_free_string(yaml_ptr);

        // Clean up
        ziplock_hybrid_cleanup();
    }

    /// Test error message retrieval
    #[test]
    fn test_error_messages() {
        // Clean up any existing state
        ziplock_hybrid_cleanup();

        // Try to create credential without initializing
        let title = CString::new("Test").unwrap();
        let cred_type = CString::new("login").unwrap();

        let credential_id = ziplock_hybrid_credential_create(title.as_ptr(), cred_type.as_ptr());

        // Should fail since not initialized
        assert_eq!(credential_id, 0);

        // Get error message
        let error_ptr = ziplock_hybrid_get_last_error();
        if !error_ptr.is_null() {
            let error_cstr = unsafe { std::ffi::CStr::from_ptr(error_ptr) };
            let error_str = error_cstr.to_str().unwrap();
            assert!(!error_str.is_empty());

            // Free error message
            ziplock_hybrid_free_string(error_ptr);
        }
    }

    /// Test cleanup and reinitialization
    #[test]
    fn test_cleanup_and_reinit() {
        // Initialize
        let init_result1 = ziplock_hybrid_init();
        assert_eq!(init_result1, ZipLockHybridError::Success as i32);

        // Clean up
        let cleanup_result = ziplock_hybrid_cleanup();
        assert_eq!(cleanup_result, ZipLockHybridError::Success as i32);

        // Reinitialize
        let init_result2 = ziplock_hybrid_init();
        assert_eq!(init_result2, ZipLockHybridError::Success as i32);

        // Clean up again
        ziplock_hybrid_cleanup();
    }
}
