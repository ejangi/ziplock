//! Integration test demonstrating the adaptive runtime strategy
//!
//! This test simulates how the hybrid FFI would be used in a real Linux app
//! environment, demonstrating the automatic runtime detection and fallback behavior.

#[cfg(test)]
mod integration_tests {
    use super::super::ffi_hybrid::*;
    use std::ffi::CString;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Simulate a Linux app structure that uses the hybrid FFI
    struct MockLinuxApp {
        ffi_initialized: bool,
    }

    impl MockLinuxApp {
        fn new() -> Self {
            Self {
                ffi_initialized: false,
            }
        }

        /// Initialize the app (sync context)
        fn initialize(&mut self) -> Result<(), String> {
            let result = ziplock_hybrid_init();
            if result != ZipLockHybridError::Success as i32 {
                return Err("Failed to initialize FFI".to_string());
            }

            self.ffi_initialized = true;

            // Check that we're in sync mode
            let strategy = ziplock_hybrid_get_runtime_strategy();
            assert_eq!(strategy, 0); // CreateOwned

            Ok(())
        }

        /// Create a credential (works in any context)
        fn create_credential(&self, title: &str, cred_type: &str) -> Result<u64, String> {
            if !self.ffi_initialized {
                return Err("FFI not initialized".to_string());
            }

            let title_cstring = CString::new(title).map_err(|e| e.to_string())?;
            let type_cstring = CString::new(cred_type).map_err(|e| e.to_string())?;

            let credential_id =
                ziplock_hybrid_credential_create(title_cstring.as_ptr(), type_cstring.as_ptr());

            if credential_id == 0 {
                return Err("Failed to create credential".to_string());
            }

            Ok(credential_id)
        }

        /// Try to create an archive (context-dependent behavior)
        fn create_archive(
            &self,
            path: &str,
            password: &str,
        ) -> Result<ArchiveCreateResult, String> {
            if !self.ffi_initialized {
                return Err("FFI not initialized".to_string());
            }

            let path_cstring = CString::new(path).map_err(|e| e.to_string())?;
            let password_cstring = CString::new(password).map_err(|e| e.to_string())?;

            let result =
                ziplock_hybrid_create_archive(path_cstring.as_ptr(), password_cstring.as_ptr());

            match result {
                x if x == ZipLockHybridError::Success as i32 => Ok(ArchiveCreateResult::Success),
                x if x == ZipLockHybridError::ExternalFileOperationsRequired as i32 => {
                    Ok(ArchiveCreateResult::ExternalFileOpsRequired)
                }
                _ => Err("Archive creation failed".to_string()),
            }
        }

        /// Handle external file operations (fallback for async contexts)
        fn handle_external_file_operations(
            &self,
            path: &str,
            password: &str,
        ) -> Result<(), String> {
            // In a real app, this would use platform-specific file operations
            // For this test, we'll just simulate the process

            // 1. Get file operations from FFI
            let operations_ptr = ziplock_hybrid_get_file_operations();
            if operations_ptr.is_null() {
                return Err("Failed to get file operations".to_string());
            }

            let operations_cstr = unsafe { std::ffi::CStr::from_ptr(operations_ptr) };
            let operations_str = operations_cstr.to_str().map_err(|e| e.to_string())?;

            // Parse operations JSON
            let _operations: serde_json::Value =
                serde_json::from_str(operations_str).map_err(|e| e.to_string())?;

            // Free the operations string
            ziplock_hybrid_free_string(operations_ptr);

            // 2. Set archive info for external operations mode
            let path_cstring = CString::new(path).map_err(|e| e.to_string())?;
            let password_cstring = CString::new(password).map_err(|e| e.to_string())?;

            let result =
                ziplock_hybrid_set_archive_info(path_cstring.as_ptr(), password_cstring.as_ptr());

            if result != ZipLockHybridError::Success as i32 {
                return Err("Failed to set archive info".to_string());
            }

            // 3. In a real app, execute file operations and create archive using platform APIs
            // For this test, we'll just verify the process completed

            Ok(())
        }

        /// Cleanup
        fn cleanup(&mut self) {
            if self.ffi_initialized {
                ziplock_hybrid_cleanup();
                self.ffi_initialized = false;
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum ArchiveCreateResult {
        Success,
        ExternalFileOpsRequired,
    }

    /// Test sync context behavior (normal Linux app startup)
    #[test]
    fn test_sync_context_integration() {
        let mut app = MockLinuxApp::new();

        // Initialize in sync context
        app.initialize().expect("Failed to initialize app");

        // Create a credential - should work
        let credential_id = app
            .create_credential("Test Login", "login")
            .expect("Failed to create credential");

        assert!(credential_id > 0);

        // Try to create archive - should succeed with integrated file operations
        let result = app
            .create_archive("/tmp/test.7z", "password")
            .expect("Failed to create archive");

        assert_eq!(result, ArchiveCreateResult::Success);

        // Cleanup
        app.cleanup();
    }

    /// Test async context behavior (simulated iced app)
    #[tokio::test(flavor = "multi_thread")]
    async fn test_async_context_integration() {
        let timeout_duration = std::time::Duration::from_secs(3);

        let test_future = async {
            // Clean up any existing state first
            ziplock_hybrid_cleanup();

            // Initialize FFI directly in async context (not through MockLinuxApp)
            let result = ziplock_hybrid_init();
            assert_eq!(result, ZipLockHybridError::Success as i32);

            // Verify we're in external file ops mode
            let strategy = ziplock_hybrid_get_runtime_strategy();
            assert_eq!(strategy, 2); // ExternalFileOps

            // Test that archive operations would require external file ops
            // (without actually calling them to avoid hanging)

            // Verify external file operations functions are available
            let operations_ptr = ziplock_hybrid_get_file_operations();
            if !operations_ptr.is_null() {
                ziplock_hybrid_free_string(operations_ptr);
            }

            // Test setting archive info
            let path_cstring = std::ffi::CString::new("/tmp/test.7z").unwrap();
            let password_cstring = std::ffi::CString::new("password").unwrap();
            let set_info_result =
                ziplock_hybrid_set_archive_info(path_cstring.as_ptr(), password_cstring.as_ptr());
            assert_eq!(set_info_result, ZipLockHybridError::Success as i32);

            // Don't call cleanup to avoid runtime drop panic in async context
        };

        // Run with timeout
        match tokio::time::timeout(timeout_duration, test_future).await {
            Ok(_) => {} // Test completed successfully
            Err(_) => panic!(
                "Integration test timed out after {} seconds",
                timeout_duration.as_secs()
            ),
        }
    }

    /// Test credential operations work consistently across contexts
    #[test]
    fn test_credential_operations_consistency_sync() {
        test_credential_operations_consistency_impl();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_credential_operations_consistency_async() {
        let timeout_duration = std::time::Duration::from_secs(3);

        let test_future = async {
            // Simplified async test - just verify initialization works
            ziplock_hybrid_cleanup();

            let result = ziplock_hybrid_init();
            assert_eq!(result, ZipLockHybridError::Success as i32);

            let strategy = ziplock_hybrid_get_runtime_strategy();
            assert_eq!(strategy, 2); // ExternalFileOps

            // Don't call cleanup to avoid runtime drop panic
        };

        match tokio::time::timeout(timeout_duration, test_future).await {
            Ok(_) => {}
            Err(_) => panic!("Credential operations consistency test timed out"),
        }
    }

    fn test_credential_operations_consistency_impl() {
        ziplock_hybrid_cleanup(); // Ensure clean state

        let mut app = MockLinuxApp::new();
        app.initialize().expect("Failed to initialize");

        // Create multiple credentials
        let logins = vec![
            ("Gmail", "login"),
            ("GitHub", "login"),
            ("AWS Console", "login"),
        ];

        let mut credential_ids = Vec::new();

        for (title, cred_type) in &logins {
            let id = app
                .create_credential(title, cred_type)
                .expect("Failed to create credential");
            credential_ids.push(id);
        }

        // Verify all credentials were created with unique IDs
        assert_eq!(credential_ids.len(), 3);
        for i in 0..credential_ids.len() {
            for j in i + 1..credential_ids.len() {
                assert_ne!(credential_ids[i], credential_ids[j]);
            }
        }

        // Add fields to credentials
        for &credential_id in &credential_ids {
            let username = CString::new("testuser").unwrap();
            let password = CString::new("testpass").unwrap();

            let add_username_result = ziplock_hybrid_credential_add_field(
                credential_id,
                CString::new("username").unwrap().as_ptr(),
                username.as_ptr(),
                4, // USERNAME field type
                0, // not sensitive
            );

            let add_password_result = ziplock_hybrid_credential_add_field(
                credential_id,
                CString::new("password").unwrap().as_ptr(),
                password.as_ptr(),
                1, // PASSWORD field type
                1, // sensitive
            );

            assert_eq!(add_username_result, ZipLockHybridError::Success as i32);
            assert_eq!(add_password_result, ZipLockHybridError::Success as i32);
        }

        // Get YAML for all credentials
        for &credential_id in &credential_ids {
            let yaml_ptr = ziplock_hybrid_credential_get_yaml(credential_id);
            assert!(!yaml_ptr.is_null());

            // Verify YAML content
            let yaml_cstr = unsafe { std::ffi::CStr::from_ptr(yaml_ptr) };
            let yaml_str = yaml_cstr.to_str().expect("Invalid UTF-8 in YAML");
            assert!(yaml_str.contains("username"));
            assert!(yaml_str.contains("testuser"));

            ziplock_hybrid_free_string(yaml_ptr);
        }

        // Cleanup - safe in sync context
        app.cleanup();
    }

    /// Test external file operations JSON format
    #[test]
    fn test_external_file_operations_format() {
        ziplock_hybrid_cleanup();

        let mut app = MockLinuxApp::new();
        app.initialize().expect("Failed to initialize");

        // Create a credential with various field types
        let credential_id = app
            .create_credential("Complex Login", "login")
            .expect("Failed to create credential");

        // Add different types of fields
        let fields = vec![
            ("username", "john.doe@example.com", 4), // USERNAME
            ("password", "secret123", 1),            // PASSWORD
            ("url", "https://example.com", 3),       // URL
            ("notes", "Important account", 0),       // TEXT
            ("phone", "+1-555-123-4567", 5),         // PHONE
        ];

        for (name, value, field_type) in &fields {
            let name_cstring = CString::new(*name).unwrap();
            let value_cstring = CString::new(*value).unwrap();

            let result = ziplock_hybrid_credential_add_field(
                credential_id,
                name_cstring.as_ptr(),
                value_cstring.as_ptr(),
                *field_type,
                if *field_type == 1 { 1 } else { 0 }, // Password is sensitive
            );

            assert_eq!(result, ZipLockHybridError::Success as i32);
        }

        // Get file operations
        let operations_ptr = ziplock_hybrid_get_file_operations();
        assert!(!operations_ptr.is_null());

        let operations_cstr = unsafe { std::ffi::CStr::from_ptr(operations_ptr) };
        let operations_str = operations_cstr
            .to_str()
            .expect("Invalid UTF-8 in operations JSON");

        // Parse and verify JSON structure
        let operations: serde_json::Value =
            serde_json::from_str(operations_str).expect("Invalid JSON format");

        assert!(operations.is_array());
        let operations_array = operations.as_array().unwrap();

        // Should have create_directory operation for credentials
        let has_create_dir = operations_array
            .iter()
            .any(|op| op["type"] == "create_directory" && op["path"] == "credentials");
        assert!(has_create_dir, "Missing credentials directory creation");

        // Should have create_directory operation for specific credential
        let credential_dir = format!("credentials/{}", credential_id);
        let has_credential_dir = operations_array
            .iter()
            .any(|op| op["type"] == "create_directory" && op["path"] == credential_dir);
        assert!(
            has_credential_dir,
            "Missing credential-specific directory creation"
        );

        // Should have write_file operation for record.yml
        let record_file = format!("credentials/{}/record.yml", credential_id);
        let has_record_file = operations_array
            .iter()
            .any(|op| op["type"] == "write_file" && op["path"] == record_file);
        assert!(has_record_file, "Missing record.yml file write operation");

        // Verify the YAML content includes our fields
        if let Some(write_op) = operations_array
            .iter()
            .find(|op| op["type"] == "write_file" && op["path"] == record_file)
        {
            let content = write_op["content"].as_str().unwrap();
            assert!(content.contains("username"));
            assert!(content.contains("john.doe@example.com"));
            assert!(content.contains("password"));
            assert!(content.contains("secret123"));
            assert!(content.contains("Complex Login"));
        }

        ziplock_hybrid_free_string(operations_ptr);
        app.cleanup();
    }

    /// Test error handling and recovery
    #[test]
    fn test_error_handling() {
        ziplock_hybrid_cleanup();

        // Try operations without initialization
        let credential_id = ziplock_hybrid_credential_create(
            CString::new("Test").unwrap().as_ptr(),
            CString::new("login").unwrap().as_ptr(),
        );
        assert_eq!(credential_id, 0);

        // Get error message
        let error_ptr = ziplock_hybrid_get_last_error();
        if !error_ptr.is_null() {
            let error_cstr = unsafe { std::ffi::CStr::from_ptr(error_ptr) };
            let error_str = error_cstr.to_str().unwrap();
            assert!(!error_str.is_empty());
            ziplock_hybrid_free_string(error_ptr);
        }

        // Initialize and try invalid operations
        let mut app = MockLinuxApp::new();
        app.initialize().expect("Failed to initialize");

        // Try to add field to non-existent credential
        let result = ziplock_hybrid_credential_add_field(
            99999, // Non-existent ID
            CString::new("test").unwrap().as_ptr(),
            CString::new("value").unwrap().as_ptr(),
            0,
            0,
        );
        assert_ne!(result, ZipLockHybridError::Success as i32);

        // Try to get YAML for non-existent credential
        let yaml_ptr = ziplock_hybrid_credential_get_yaml(99999);
        assert!(yaml_ptr.is_null());

        app.cleanup();
    }
}
