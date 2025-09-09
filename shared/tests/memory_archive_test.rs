//! Memory Archive Integration Test
//!
//! This test validates that the unified architecture can create encrypted 7z archives
//! entirely in memory using the FileOperationProvider interface, without touching
//! the filesystem, and validate the archive contents and structure.

use std::collections::HashMap;
use ziplock_shared::core::{FileOperationProvider, UnifiedRepositoryManager};
use ziplock_shared::models::{CredentialField, CredentialRecord};
use ziplock_shared::utils::{deserialize_credential, generate_totp};

/// Custom memory-only file provider for testing in-memory archive operations
#[derive(Debug, Clone)]
struct MemoryFileProvider {
    /// Archives stored in memory (path -> archive data)
    archives: HashMap<String, Vec<u8>>,
    /// Whether operations should simulate failures
    should_fail: bool,
}

impl MemoryFileProvider {
    fn new() -> Self {
        Self {
            archives: HashMap::new(),
            should_fail: false,
        }
    }

    fn with_failure() -> Self {
        Self {
            archives: HashMap::new(),
            should_fail: true,
        }
    }

    fn get_archive_data(&self, path: &str) -> Option<Vec<u8>> {
        self.archives.get(path).cloned()
    }

    fn archive_count(&self) -> usize {
        self.archives.len()
    }
}

impl FileOperationProvider for MemoryFileProvider {
    fn read_archive(&self, path: &str) -> ziplock_shared::core::FileResult<Vec<u8>> {
        if self.should_fail {
            return Err(ziplock_shared::core::FileError::NotFound {
                path: path.to_string(),
            });
        }

        self.archives
            .get(path)
            .cloned()
            .ok_or_else(|| ziplock_shared::core::FileError::NotFound {
                path: path.to_string(),
            })
    }

    fn write_archive(&self, _path: &str, _data: &[u8]) -> ziplock_shared::core::FileResult<()> {
        if self.should_fail {
            return Err(ziplock_shared::core::FileError::PermissionDenied {
                path: _path.to_string(),
            });
        }

        // In a real memory provider, we would store the data
        // For this test, we simulate success
        Ok(())
    }

    fn extract_archive(
        &self,
        data: &[u8],
        password: &str,
    ) -> ziplock_shared::core::FileResult<ziplock_shared::core::FileMap> {
        if self.should_fail {
            return Err(ziplock_shared::core::FileError::InvalidPassword);
        }

        // Use the same extraction logic as DesktopFileProvider for consistency
        // Check if this is a mock 7z format
        if data.len() >= 6 && &data[0..6] == b"7z\xBC\xAF\x27\x1C" {
            // Validate password
            if data.len() >= 11 {
                let stored_hash = data[6];
                let provided_hash = password
                    .bytes()
                    .fold(0u32, |acc, b| acc.wrapping_add(b as u32));

                if stored_hash != (provided_hash & 0xFF) as u8 {
                    return Err(ziplock_shared::core::FileError::InvalidPassword);
                }

                // Extract files from mock format
                let mut file_map = HashMap::new();
                let mut offset = 7;

                // Read file count
                if offset + 4 > data.len() {
                    return Err(ziplock_shared::core::FileError::CorruptedArchive {
                        message: "Invalid archive: missing file count".to_string(),
                    });
                }
                let file_count = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]) as usize;
                offset += 4;

                // Extract each file
                for _ in 0..file_count {
                    // Read path length
                    if offset + 4 > data.len() {
                        return Err(ziplock_shared::core::FileError::CorruptedArchive {
                            message: "Invalid archive: missing path length".to_string(),
                        });
                    }
                    let path_len = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]) as usize;
                    offset += 4;

                    // Read path
                    if offset + path_len > data.len() {
                        return Err(ziplock_shared::core::FileError::CorruptedArchive {
                            message: "Invalid archive: path truncated".to_string(),
                        });
                    }
                    let path = String::from_utf8(data[offset..offset + path_len].to_vec())
                        .map_err(|_| ziplock_shared::core::FileError::CorruptedArchive {
                            message: "Invalid path encoding".to_string(),
                        })?;
                    offset += path_len;

                    // Read data length
                    if offset + 4 > data.len() {
                        return Err(ziplock_shared::core::FileError::CorruptedArchive {
                            message: "Invalid archive: missing data length".to_string(),
                        });
                    }
                    let data_len = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]) as usize;
                    offset += 4;

                    // Read file data
                    if offset + data_len > data.len() {
                        return Err(ziplock_shared::core::FileError::CorruptedArchive {
                            message: "Invalid archive: file data truncated".to_string(),
                        });
                    }
                    let file_data = data[offset..offset + data_len].to_vec();
                    offset += data_len;

                    file_map.insert(path, file_data);
                }

                return Ok(file_map);
            }
        }

        // Fallback: return minimal metadata for empty archives
        let mut file_map = HashMap::new();
        file_map.insert(
            "metadata.yml".to_string(),
            b"version: \"1.0\"\nformat: \"memory-v1\"\ncredential_count: 0".to_vec(),
        );
        Ok(file_map)
    }

    fn create_archive(
        &self,
        files: ziplock_shared::core::FileMap,
        password: &str,
    ) -> ziplock_shared::core::FileResult<Vec<u8>> {
        if self.should_fail {
            return Err(ziplock_shared::core::FileError::CreationFailed {
                message: "Simulated failure".to_string(),
            });
        }

        // Create mock 7z archive format that can be extracted
        let mut archive_data = Vec::new();

        // Mock 7z header
        archive_data.extend_from_slice(b"7z\xBC\xAF\x27\x1C");

        // Store password hash for validation
        let password_hash = password
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        archive_data.push((password_hash & 0xFF) as u8);

        // Store file count
        archive_data.extend_from_slice(&(files.len() as u32).to_le_bytes());

        // Store each file
        for (path, data) in files {
            // Path length and path
            archive_data.extend_from_slice(&(path.len() as u32).to_le_bytes());
            archive_data.extend_from_slice(path.as_bytes());

            // Data length and data
            archive_data.extend_from_slice(&(data.len() as u32).to_le_bytes());
            archive_data.extend_from_slice(&data);
        }

        Ok(archive_data)
    }
}

/// Test fixture for memory archive tests
struct MemoryArchiveTest {
    provider: MemoryFileProvider,
    archive_path: String,
}

impl MemoryArchiveTest {
    fn new() -> Self {
        Self {
            provider: MemoryFileProvider::new(),
            archive_path: "/memory/test_vault.7z".to_string(),
        }
    }

    fn create_comprehensive_credentials() -> Vec<CredentialRecord> {
        let mut credentials = Vec::new();

        // Banking credential with sensitive information
        let mut bank_cred =
            CredentialRecord::new("Bank of America".to_string(), "banking".to_string());
        bank_cred.set_field(
            "account_number",
            CredentialField::new(
                ziplock_shared::models::FieldType::Number,
                "1234567890".to_string(),
                true,
            ),
        );
        bank_cred.set_field(
            "routing_number",
            CredentialField::new(
                ziplock_shared::models::FieldType::Number,
                "021000021".to_string(),
                false,
            ),
        );
        bank_cred.set_field("pin", CredentialField::password("7890"));
        bank_cred.set_field("url", CredentialField::url("https://bankofamerica.com"));
        bank_cred.add_tag("finance".to_string());
        bank_cred.add_tag("banking".to_string());
        credentials.push(bank_cred);

        // Wi-Fi credential
        let mut wifi_cred = CredentialRecord::new("Home WiFi".to_string(), "wifi".to_string());
        wifi_cred.set_field("ssid", CredentialField::text("HomeNetwork_5G"));
        wifi_cred.set_field("password", CredentialField::password("WiFiPassword123!"));
        wifi_cred.set_field("security", CredentialField::text("WPA2"));
        wifi_cred.add_tag("network".to_string());
        credentials.push(wifi_cred);

        // API key credential
        let mut api_cred = CredentialRecord::new("OpenAI API".to_string(), "api_key".to_string());
        api_cred.set_field("api_key", CredentialField::password("sk-1234567890abcdef"));
        api_cred.set_field(
            "endpoint",
            CredentialField::url("https://api.openai.com/v1"),
        );
        api_cred.set_field("usage_limit", CredentialField::text("$50/month"));
        api_cred.add_tag("development".to_string());
        api_cred.add_tag("ai".to_string());
        api_cred.favorite = true;
        credentials.push(api_cred);

        // Database credential with connection details
        let mut db_cred =
            CredentialRecord::new("Production DB".to_string(), "database".to_string());
        db_cred.set_field("host", CredentialField::text("prod-db.company.com"));
        db_cred.set_field(
            "port",
            CredentialField::new(
                ziplock_shared::models::FieldType::Number,
                "5432".to_string(),
                false,
            ),
        );
        db_cred.set_field("username", CredentialField::username("db_admin"));
        db_cred.set_field("password", CredentialField::password("SecureDbPass2023!"));
        db_cred.set_field("database", CredentialField::text("production"));
        db_cred.set_field("connection_string", CredentialField::new(
            ziplock_shared::models::FieldType::TextArea,
            "postgresql://db_admin:SecureDbPass2023!@prod-db.company.com:5432/production?sslmode=require".to_string(),
            true,
        ));
        db_cred.add_tag("infrastructure".to_string());
        db_cred.add_tag("database".to_string());
        credentials.push(db_cred);

        // TOTP-enabled social media account
        let mut social_cred =
            CredentialRecord::new("Twitter Business".to_string(), "social".to_string());
        social_cred.set_field("username", CredentialField::username("@company_official"));
        social_cred.set_field("email", CredentialField::email("social@company.com"));
        social_cred.set_field("password", CredentialField::password("TwitterPass2023!"));
        social_cred.set_field(
            "totp_secret",
            CredentialField::totp_secret("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"),
        );
        social_cred.set_field(
            "recovery_codes",
            CredentialField::new(
                ziplock_shared::models::FieldType::TextArea,
                "12345678\n87654321\n11111111\n22222222".to_string(),
                true,
            ),
        );
        social_cred.add_tag("social_media".to_string());
        social_cred.add_tag("business".to_string());
        credentials.push(social_cred);

        credentials
    }
}

#[test]
fn test_create_memory_archive() {
    let test = MemoryArchiveTest::new();
    let mut manager = UnifiedRepositoryManager::new(test.provider.clone());

    // Create repository in memory
    manager
        .create_repository(&test.archive_path, "memory_test_password")
        .expect("Failed to create repository in memory");

    assert!(manager.is_open());

    // Add credentials
    let credentials = MemoryArchiveTest::create_comprehensive_credentials();
    for credential in &credentials {
        manager
            .add_credential(credential.clone())
            .expect("Failed to add credential");
    }

    // Export to file map (this tests the memory serialization)
    let file_map = manager
        .export_to_file_map()
        .expect("Failed to export to file map");

    // Verify file map structure
    assert!(file_map.contains_key("metadata.yml"));

    let metadata_content = String::from_utf8(file_map["metadata.yml"].clone())
        .expect("Metadata should be valid UTF-8");
    assert!(metadata_content.contains("version:"));
    assert!(metadata_content.contains("credential_count: 5"));

    // Verify each credential has its file
    let credential_files: Vec<_> = file_map
        .keys()
        .filter(|k| k.starts_with("credentials/") && k.ends_with("/record.yml"))
        .collect();

    assert_eq!(credential_files.len(), 5, "Should have 5 credential files");

    // Verify credential data integrity
    for (file_path, file_data) in &file_map {
        if file_path.starts_with("credentials/") && file_path.ends_with("/record.yml") {
            let credential_yaml = String::from_utf8(file_data.clone())
                .expect("Credential file should be valid UTF-8");

            let credential = deserialize_credential(&credential_yaml)
                .expect("Should be able to deserialize credential");

            // Verify basic credential structure
            assert!(!credential.id.is_empty());
            assert!(!credential.title.is_empty());
            assert!(!credential.credential_type.is_empty());
            assert!(credential.created_at > 0);
            assert!(credential.updated_at > 0);
        }
    }

    manager.close_repository(true);
}

#[test]
fn test_memory_archive_round_trip() {
    let provider = MemoryFileProvider::new();
    let archive_path = "/memory/round_trip_test.7z";

    // Create and populate repository
    let _original_credentials = {
        let mut manager = UnifiedRepositoryManager::new(provider.clone());

        manager
            .create_repository(archive_path, "round_trip_password")
            .expect("Failed to create repository");

        let credentials = MemoryArchiveTest::create_comprehensive_credentials();
        for credential in &credentials {
            manager
                .add_credential(credential.clone())
                .expect("Failed to add credential");
        }

        // Export to file map for archive creation
        let file_map = manager
            .export_to_file_map()
            .expect("Failed to export to file map");

        // Create archive data
        let archive_data = provider
            .create_archive(file_map, "round_trip_password")
            .expect("Failed to create archive");

        manager.close_repository(true);

        // Store archive data for later extraction
        let original_creds = manager.list_credentials().unwrap_or_default();

        // Simulate storing the archive
        assert!(!archive_data.is_empty());
        assert!(archive_data.len() > 100); // Should be substantial

        original_creds
    };

    // Now extract and verify
    let provider = MemoryFileProvider::new();

    // Create archive data manually for testing
    let mut manager = UnifiedRepositoryManager::new(provider.clone());
    manager
        .create_repository("/memory/temp.7z", "round_trip_password")
        .expect("Failed to create temp repository");

    let credentials = MemoryArchiveTest::create_comprehensive_credentials();
    for credential in &credentials {
        manager
            .add_credential(credential.clone())
            .expect("Failed to add credential to temp");
    }

    let file_map = manager.export_to_file_map().expect("Failed to export temp");

    let archive_data = provider
        .create_archive(file_map.clone(), "round_trip_password")
        .expect("Failed to create test archive");

    // Extract the archive
    let extracted_files = provider
        .extract_archive(&archive_data, "round_trip_password")
        .expect("Failed to extract archive");

    // Verify extracted structure matches original
    assert_eq!(
        extracted_files
            .keys()
            .collect::<std::collections::BTreeSet<_>>(),
        file_map.keys().collect::<std::collections::BTreeSet<_>>()
    );

    // Import back to repository and verify
    let mut new_manager = UnifiedRepositoryManager::new(provider);
    new_manager
        .import_from_file_map(extracted_files)
        .expect("Failed to import from file map");

    let loaded_credentials = new_manager
        .list_credentials()
        .expect("Failed to list loaded credentials");

    assert_eq!(loaded_credentials.len(), 5);

    // Verify specific credentials
    let bank_cred = loaded_credentials
        .iter()
        .find(|c| c.title == "Bank of America")
        .expect("Bank credential should exist");

    assert_eq!(bank_cred.credential_type, "banking");
    assert!(bank_cred.has_tag("finance"));
    assert!(bank_cred.has_tag("banking"));

    let account_number = bank_cred
        .get_field("account_number")
        .expect("Account number should exist");
    assert_eq!(account_number.value, "1234567890");
    assert!(account_number.sensitive);

    // Test TOTP functionality
    let twitter_cred = loaded_credentials
        .iter()
        .find(|c| c.title == "Twitter Business")
        .expect("Twitter credential should exist");

    assert!(twitter_cred.has_tag("social_media"));
    let totp_secret = twitter_cred
        .get_field("totp_secret")
        .expect("TOTP secret should exist");

    let totp_code = generate_totp(&totp_secret.value, 30).expect("Should generate TOTP code");
    assert_eq!(totp_code.len(), 6);
    assert!(totp_code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_memory_archive_with_invalid_password() {
    let provider = MemoryFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(provider.clone());

    // Create repository
    manager
        .create_repository("/memory/password_test.7z", "correct_password")
        .expect("Failed to create repository");

    let credential = CredentialRecord::new("Test".to_string(), "test".to_string());
    manager
        .add_credential(credential)
        .expect("Failed to add credential");

    let file_map = manager.export_to_file_map().expect("Failed to export");

    // Create archive with correct password
    let archive_data = provider
        .create_archive(file_map, "correct_password")
        .expect("Failed to create archive");

    // Try to extract with wrong password
    let extract_result = provider.extract_archive(&archive_data, "wrong_password");

    assert!(extract_result.is_err(), "Should fail with wrong password");

    if let Err(error) = extract_result {
        assert!(matches!(
            error,
            ziplock_shared::core::FileError::InvalidPassword
        ));
    }

    // Verify correct password works
    let correct_extract = provider
        .extract_archive(&archive_data, "correct_password")
        .expect("Should succeed with correct password");

    assert!(correct_extract.contains_key("metadata.yml"));

    manager.close_repository(true);
}

#[test]
fn test_memory_archive_serialization_integrity() {
    let provider = MemoryFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(provider.clone());

    manager
        .create_repository("/memory/serialization_test.7z", "test_password")
        .expect("Failed to create repository");

    // Create credential with complex data
    let mut complex_cred =
        CredentialRecord::new("Complex Test Credential".to_string(), "complex".to_string());

    // Add various field types
    complex_cred.set_field("text_field", CredentialField::text("Simple text"));
    complex_cred.set_field("password_field", CredentialField::password("Secret123!"));
    complex_cred.set_field("email_field", CredentialField::email("test@example.com"));
    complex_cred.set_field(
        "url_field",
        CredentialField::url("https://example.com/path?param=value"),
    );
    complex_cred.set_field("username_field", CredentialField::username("testuser"));
    complex_cred.set_field(
        "phone_field",
        CredentialField::new(
            ziplock_shared::models::FieldType::Phone,
            "+1 (555) 123-4567".to_string(),
            false,
        ),
    );

    // Multi-line text area
    complex_cred.set_field(
        "notes_field",
        CredentialField::new(
            ziplock_shared::models::FieldType::TextArea,
            "Line 1\nLine 2\nLine 3 with special chars: !@#$%^&*()\nLine 4 with unicode: 测试 🔒"
                .to_string(),
            false,
        ),
    );

    // Date field
    complex_cred.set_field(
        "date_field",
        CredentialField::new(
            ziplock_shared::models::FieldType::Date,
            "2023-12-25".to_string(),
            false,
        ),
    );

    // Number field
    complex_cred.set_field(
        "number_field",
        CredentialField::new(
            ziplock_shared::models::FieldType::Number,
            "42".to_string(),
            false,
        ),
    );

    // Custom field
    complex_cred.set_field(
        "custom_field",
        CredentialField::new(
            ziplock_shared::models::FieldType::Custom("Custom Type".to_string()),
            "Custom value".to_string(),
            true,
        ),
    );

    // Add tags and metadata
    complex_cred.add_tag("test".to_string());
    complex_cred.add_tag("complex".to_string());
    complex_cred.add_tag("serialization".to_string());
    complex_cred.notes =
        Some("Credential notes with special characters: ñáéíóú ÇÜ 测试".to_string());
    complex_cred.favorite = true;
    complex_cred.folder_path = Some("Test/Folder/Path".to_string());

    let original_id = complex_cred.id.clone();

    manager
        .add_credential(complex_cred)
        .expect("Failed to add complex credential");

    // Export and create archive
    let file_map = manager.export_to_file_map().expect("Failed to export");

    let archive_data = provider
        .create_archive(file_map, "test_password")
        .expect("Failed to create archive");

    manager.close_repository(true);

    // Extract and import back
    let extracted_files = provider
        .extract_archive(&archive_data, "test_password")
        .expect("Failed to extract");

    let mut new_manager = UnifiedRepositoryManager::new(provider);
    new_manager
        .import_from_file_map(extracted_files)
        .expect("Failed to import");

    // Verify all data was preserved
    let loaded_creds = new_manager
        .list_credentials()
        .expect("Failed to list credentials");

    assert_eq!(loaded_creds.len(), 1);

    let loaded_cred = &loaded_creds[0];
    assert_eq!(loaded_cred.id, original_id);
    assert_eq!(loaded_cred.title, "Complex Test Credential");
    assert_eq!(loaded_cred.credential_type, "complex");
    assert!(loaded_cred.favorite);
    assert_eq!(
        loaded_cred.folder_path.as_ref().unwrap(),
        "Test/Folder/Path"
    );
    assert!(loaded_cred.notes.as_ref().unwrap().contains("ñáéíóú"));

    // Verify all fields
    assert_eq!(
        loaded_cred.get_field("text_field").unwrap().value,
        "Simple text"
    );
    assert_eq!(
        loaded_cred.get_field("password_field").unwrap().value,
        "Secret123!"
    );
    assert!(loaded_cred.get_field("password_field").unwrap().sensitive);
    assert_eq!(
        loaded_cred.get_field("email_field").unwrap().value,
        "test@example.com"
    );
    assert_eq!(
        loaded_cred.get_field("url_field").unwrap().value,
        "https://example.com/path?param=value"
    );
    assert_eq!(
        loaded_cred.get_field("username_field").unwrap().value,
        "testuser"
    );
    assert_eq!(
        loaded_cred.get_field("phone_field").unwrap().value,
        "+1 (555) 123-4567"
    );

    let notes_field = loaded_cred.get_field("notes_field").unwrap();
    assert!(notes_field.value.contains("Line 1\nLine 2\nLine 3"));
    assert!(notes_field.value.contains("测试 🔒"));

    assert_eq!(
        loaded_cred.get_field("date_field").unwrap().value,
        "2023-12-25"
    );
    assert_eq!(loaded_cred.get_field("number_field").unwrap().value, "42");
    assert_eq!(
        loaded_cred.get_field("custom_field").unwrap().value,
        "Custom value"
    );
    assert!(loaded_cred.get_field("custom_field").unwrap().sensitive);

    // Verify tags
    assert!(loaded_cred.has_tag("test"));
    assert!(loaded_cred.has_tag("complex"));
    assert!(loaded_cred.has_tag("serialization"));
}

#[test]
fn test_memory_provider_failure_modes() {
    let failing_provider = MemoryFileProvider::with_failure();
    let mut manager = UnifiedRepositoryManager::new(failing_provider);

    // Test creation failure
    let create_result = manager.create_repository("/memory/fail_test.7z", "password");
    assert!(create_result.is_err(), "Should fail with failing provider");

    // Test with working provider but simulate individual operation failures
    let working_provider = MemoryFileProvider::new();
    let mut working_manager = UnifiedRepositoryManager::new(working_provider.clone());

    working_manager
        .create_repository("/memory/working.7z", "password")
        .expect("Should succeed with working provider");

    let credential = CredentialRecord::new("Test".to_string(), "test".to_string());
    working_manager
        .add_credential(credential)
        .expect("Should add credential successfully");

    // Test extract failure with failing provider
    let failing_provider = MemoryFileProvider::with_failure();
    let extract_result = failing_provider.extract_archive(b"test_data", "password");
    assert!(
        extract_result.is_err(),
        "Extract should fail with failing provider"
    );

    // Test create archive failure
    let create_archive_result = failing_provider.create_archive(HashMap::new(), "password");
    assert!(
        create_archive_result.is_err(),
        "Create archive should fail with failing provider"
    );
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Helper to create a credential with specific field count
    pub fn create_credential_with_fields(title: &str, field_count: usize) -> CredentialRecord {
        let mut cred = CredentialRecord::new(title.to_string(), "test".to_string());

        for i in 0..field_count {
            let field_name = format!("field_{}", i);
            let field_value = format!("value_{}", i);
            cred.set_field(&field_name, CredentialField::text(&field_value));
        }

        cred
    }

    /// Helper to validate file map structure
    pub fn validate_file_map_structure(file_map: &ziplock_shared::core::FileMap) {
        assert!(
            file_map.contains_key("metadata.yml"),
            "Should contain metadata"
        );

        let credential_files: Vec<_> = file_map
            .keys()
            .filter(|k| k.starts_with("credentials/") && k.ends_with("/record.yml"))
            .collect();

        assert!(
            !credential_files.is_empty(),
            "Should have at least one credential file"
        );

        // Validate metadata format
        let metadata_content = String::from_utf8(file_map["metadata.yml"].clone())
            .expect("Metadata should be valid UTF-8");
        assert!(metadata_content.contains("version:"));
        assert!(metadata_content.contains("credential_count:"));
    }
}
