//! Mobile FFI interface for ZipLock
//!
//! This module provides the C-compatible FFI interface for mobile platforms
//! (Android and iOS). It exposes memory-only operations, with file operations
//! handled by native platform code.
//!
//! # Architecture
//!
//! Mobile platforms handle:
//! - Archive file I/O (using SAF on Android, Documents API on iOS)
//! - 7z extraction/creation using native libraries
//! - File system permissions and security
//!
//! Shared library handles:
//! - All credential operations in memory
//! - Data validation and business logic
//! - JSON serialization for file map exchange
//!
//! # Usage Pattern
//!
//! 1. Platform code reads archive file and extracts using native 7z libs
//! 2. Platform code converts extracted files to JSON and passes to shared lib
//! 3. Shared library loads credentials into memory repository
//! 4. All credential operations happen via FFI calls
//! 5. Platform code retrieves file map as JSON when saving needed
//! 6. Platform code creates new archive and writes to storage

use base64::prelude::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Mutex;

use crate::core::{CoreError, UnifiedMemoryRepository};
use crate::ffi::common::{c_string_to_rust, rust_string_to_c, ZipLockError};
use crate::models::CredentialRecord;

/// Handle type for mobile repository instances
pub type MobileRepositoryHandle = *mut MobileRepositoryInstance;

/// Internal repository instance for mobile platforms
pub struct MobileRepositoryInstance {
    repository: Mutex<UnifiedMemoryRepository>,
}

impl MobileRepositoryInstance {
    fn new() -> Self {
        Self {
            repository: Mutex::new(UnifiedMemoryRepository::new()),
        }
    }
}

/// Create a new mobile repository instance
///
/// # Returns
/// * Non-null handle on success
/// * Null on failure (out of memory)
///
/// # Safety
/// The returned handle must be freed with `ziplock_mobile_repository_destroy`
#[no_mangle]
pub extern "C" fn ziplock_mobile_repository_create() -> MobileRepositoryHandle {
    let instance = Box::new(MobileRepositoryInstance::new());
    Box::into_raw(instance)
}

/// Destroy a mobile repository instance
///
/// # Arguments
/// * `handle` - Repository handle to destroy
///
/// # Safety
/// Handle must be valid and not used after this call
#[no_mangle]
pub extern "C" fn ziplock_mobile_repository_destroy(handle: MobileRepositoryHandle) {
    if handle.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(handle);
    }
}

/// Initialize an empty repository
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if handle is null
/// * `ZipLockError::AlreadyInitialized` if already initialized
#[no_mangle]
pub extern "C" fn ziplock_mobile_repository_initialize(
    handle: MobileRepositoryHandle,
) -> ZipLockError {
    if handle.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        match repo.initialize() {
            Ok(()) => ZipLockError::Success,
            Err(CoreError::AlreadyInitialized) => ZipLockError::AlreadyInitialized,
            Err(_) => ZipLockError::InternalError,
        }
    }
}

/// Check if repository is initialized
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * 1 if initialized, 0 if not initialized or handle is invalid
#[no_mangle]
pub extern "C" fn ziplock_mobile_repository_is_initialized(
    handle: MobileRepositoryHandle,
) -> c_int {
    if handle.is_null() {
        return 0;
    }

    unsafe {
        let instance = &*handle;
        let repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return 0,
        };

        if repo.is_initialized() {
            1
        } else {
            0
        }
    }
}

/// Load repository from file map JSON
///
/// Platform code should extract the 7z archive using native libraries,
/// then convert the file map to JSON and pass it to this function.
///
/// # Arguments
/// * `handle` - Repository handle
/// * `files_json` - JSON string containing file map (path -> base64 content)
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if handle is null or JSON is invalid
/// * `ZipLockError::NotInitialized` if repository not initialized
/// * `ZipLockError::SerializationError` if JSON parsing fails
#[no_mangle]
pub extern "C" fn ziplock_mobile_repository_load_from_files(
    handle: MobileRepositoryHandle,
    files_json: *const c_char,
) -> ZipLockError {
    if handle.is_null() || files_json.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        let json_str = match c_string_to_rust(files_json) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        // Parse JSON file map
        let file_map: HashMap<String, Vec<u8>> =
            match serde_json::from_str::<HashMap<String, String>>(&json_str) {
                Ok(map) => {
                    // Convert base64 encoded values back to bytes
                    let mut decoded_map = HashMap::new();
                    for (path, base64_str) in map.iter() {
                        if let Ok(bytes) = base64::prelude::BASE64_STANDARD.decode(base64_str) {
                            decoded_map.insert(path.clone(), bytes);
                        } else {
                            // If base64 decode fails, treat as UTF-8 text
                            decoded_map.insert(path.clone(), base64_str.as_bytes().to_vec());
                        }
                    }
                    decoded_map
                }
                Err(_) => return ZipLockError::SerializationError,
            };

        match repo.load_from_files(file_map) {
            Ok(()) => ZipLockError::Success,
            Err(CoreError::NotInitialized) => ZipLockError::NotInitialized,
            Err(CoreError::SerializationError { .. }) => ZipLockError::SerializationError,
            Err(_) => ZipLockError::InternalError,
        }
    }
}

/// Serialize repository to file map JSON
///
/// Returns a JSON string containing the file map that platform code
/// can use to create a new 7z archive.
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * JSON string on success (must be freed with `ziplock_free_string`)
/// * Null on error
#[no_mangle]
pub extern "C" fn ziplock_mobile_repository_serialize_to_files(
    handle: MobileRepositoryHandle,
) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ptr::null_mut(),
        };

        match repo.serialize_to_files() {
            Ok(file_map) => {
                // Convert to base64 encoded JSON for mobile platforms
                let encoded_map: HashMap<String, String> = file_map
                    .into_iter()
                    .map(|(path, data)| (path, base64::prelude::BASE64_STANDARD.encode(data)))
                    .collect();

                match serde_json::to_string(&encoded_map) {
                    Ok(json) => rust_string_to_c(json),
                    Err(_) => ptr::null_mut(),
                }
            }
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Add a new credential to the repository
///
/// # Arguments
/// * `handle` - Repository handle
/// * `credential_json` - JSON string containing credential data
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if parameters are invalid
/// * `ZipLockError::NotInitialized` if repository not initialized
/// * `ZipLockError::SerializationError` if JSON parsing fails
#[no_mangle]
pub extern "C" fn ziplock_mobile_add_credential(
    handle: MobileRepositoryHandle,
    credential_json: *const c_char,
) -> ZipLockError {
    if handle.is_null() || credential_json.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        let json_str = match c_string_to_rust(credential_json) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        let credential: CredentialRecord = match serde_json::from_str(&json_str) {
            Ok(cred) => cred,
            Err(_) => return ZipLockError::SerializationError,
        };

        match repo.add_credential(credential) {
            Ok(()) => ZipLockError::Success,
            Err(CoreError::NotInitialized) => ZipLockError::NotInitialized,
            Err(CoreError::ValidationError { .. }) => ZipLockError::ValidationError,
            Err(_) => ZipLockError::InternalError,
        }
    }
}

/// Get a credential by ID
///
/// # Arguments
/// * `handle` - Repository handle
/// * `credential_id` - Credential ID to retrieve
///
/// # Returns
/// * JSON string containing credential data (must be freed with `ziplock_free_string`)
/// * Null if not found or error
#[no_mangle]
pub extern "C" fn ziplock_mobile_get_credential(
    handle: MobileRepositoryHandle,
    credential_id: *const c_char,
) -> *mut c_char {
    if handle.is_null() || credential_id.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ptr::null_mut(),
        };

        let id_str = match c_string_to_rust(credential_id) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match repo.get_credential_readonly(&id_str) {
            Ok(credential) => match serde_json::to_string(credential) {
                Ok(json) => rust_string_to_c(json),
                Err(_) => ptr::null_mut(),
            },
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Update an existing credential
///
/// # Arguments
/// * `handle` - Repository handle
/// * `credential_json` - JSON string containing updated credential data
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if parameters are invalid
/// * `ZipLockError::NotInitialized` if repository not initialized
/// * `ZipLockError::SerializationError` if JSON parsing fails
#[no_mangle]
pub extern "C" fn ziplock_mobile_update_credential(
    handle: MobileRepositoryHandle,
    credential_json: *const c_char,
) -> ZipLockError {
    if handle.is_null() || credential_json.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        let json_str = match c_string_to_rust(credential_json) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        let credential: CredentialRecord = match serde_json::from_str(&json_str) {
            Ok(cred) => cred,
            Err(_) => return ZipLockError::SerializationError,
        };

        match repo.update_credential(credential) {
            Ok(()) => ZipLockError::Success,
            Err(CoreError::NotInitialized) => ZipLockError::NotInitialized,
            Err(CoreError::CredentialNotFound { .. }) => ZipLockError::InvalidParameter,
            Err(CoreError::ValidationError { .. }) => ZipLockError::ValidationError,
            Err(_) => ZipLockError::InternalError,
        }
    }
}

/// Delete a credential by ID
///
/// # Arguments
/// * `handle` - Repository handle
/// * `credential_id` - ID of credential to delete
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if parameters are invalid
/// * `ZipLockError::NotInitialized` if repository not initialized
#[no_mangle]
pub extern "C" fn ziplock_mobile_delete_credential(
    handle: MobileRepositoryHandle,
    credential_id: *const c_char,
) -> ZipLockError {
    if handle.is_null() || credential_id.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        let id_str = match c_string_to_rust(credential_id) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        match repo.delete_credential(&id_str) {
            Ok(_) => ZipLockError::Success,
            Err(CoreError::NotInitialized) => ZipLockError::NotInitialized,
            Err(CoreError::CredentialNotFound { .. }) => ZipLockError::InvalidParameter,
            Err(_) => ZipLockError::InternalError,
        }
    }
}

/// List all credentials in the repository
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * JSON array string containing credential summaries (must be freed with `ziplock_free_string`)
/// * Null if error
#[no_mangle]
pub extern "C" fn ziplock_mobile_list_credentials(handle: MobileRepositoryHandle) -> *mut c_char {
    if handle.is_null() {
        eprintln!("DEBUG: handle is null");
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("DEBUG: Failed to lock repository: {:?}", e);
                return ptr::null_mut();
            }
        };

        eprintln!("DEBUG: Calling repo.list_credentials()");
        match repo.list_credentials() {
            Ok(credentials) => {
                eprintln!("DEBUG: Got {} credentials", credentials.len());
                eprintln!(
                    "DEBUG: First credential (if any): {:?}",
                    credentials.first()
                );

                // Additional debugging - show structure of each credential
                for (i, cred) in credentials.iter().enumerate() {
                    eprintln!(
                        "DEBUG: Credential {}: ID='{}', Title='{}', Type='{}', Fields={:?}",
                        i,
                        cred.id,
                        cred.title,
                        cred.credential_type,
                        cred.fields.keys().collect::<Vec<_>>()
                    );
                }

                match serde_json::to_string(&credentials) {
                    Ok(json) => {
                        eprintln!("DEBUG: Serialized JSON length: {}", json.len());
                        eprintln!(
                            "DEBUG: Serialized JSON first 200 chars: {}",
                            if json.len() > 200 {
                                &json[..200]
                            } else {
                                &json
                            }
                        );
                        eprintln!("DEBUG: Full serialized JSON: {}", json);
                        rust_string_to_c(json)
                    }
                    Err(e) => {
                        eprintln!("DEBUG: JSON serialization failed: {:?}", e);
                        ptr::null_mut()
                    }
                }
            }
            Err(e) => {
                eprintln!("DEBUG: list_credentials failed: {:?}", e);
                ptr::null_mut()
            }
        }
    }
}

/// Check if repository has been modified
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * 1 if modified, 0 if not modified or handle is invalid
#[no_mangle]
pub extern "C" fn ziplock_mobile_is_modified(handle: MobileRepositoryHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }

    unsafe {
        let instance = &*handle;
        let repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return 0,
        };

        if repo.is_modified() {
            1
        } else {
            0
        }
    }
}

/// Mark repository as saved (clear modified flag)
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if handle is invalid
#[no_mangle]
pub extern "C" fn ziplock_mobile_mark_saved(handle: MobileRepositoryHandle) -> ZipLockError {
    if handle.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        repo.mark_saved();
        ZipLockError::Success
    }
}

/// Get repository statistics
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * JSON string containing repository stats (must be freed with `ziplock_free_string`)
/// * Null if error
#[no_mangle]
pub extern "C" fn ziplock_mobile_get_stats(handle: MobileRepositoryHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ptr::null_mut(),
        };

        match repo.get_stats() {
            Ok(stats) => match serde_json::to_string(&stats) {
                Ok(json) => rust_string_to_c(json),
                Err(_) => ptr::null_mut(),
            },
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Clear all credentials from the repository
///
/// # Arguments
/// * `handle` - Repository handle
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if handle is invalid
/// * `ZipLockError::NotInitialized` if repository not initialized
#[no_mangle]
pub extern "C" fn ziplock_mobile_clear_credentials(handle: MobileRepositoryHandle) -> ZipLockError {
    if handle.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut repo = match instance.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return ZipLockError::InternalError,
        };

        match repo.clear() {
            Ok(()) => ZipLockError::Success,
            Err(CoreError::NotInitialized) => ZipLockError::NotInitialized,
            Err(_) => ZipLockError::InternalError,
        }
    }
}

/// Free a string returned by this library
///
/// # Arguments
/// * `str_ptr` - String pointer to free
///
/// # Safety
/// Pointer must have been returned by this library and not already freed
#[no_mangle]
pub extern "C" fn ziplock_mobile_free_string(str_ptr: *mut c_char) {
    if str_ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(str_ptr);
    }
}

/// Create an encrypted archive from file map JSON to a temporary file location
///
/// This function creates a properly encrypted 7z archive using sevenz-rust2 and saves it
/// to a temporary file that can be accessed by the mobile platform for moving to the
/// final destination using platform-specific file operations (like SAF on Android).
///
/// # Arguments
/// * `files_json` - JSON string containing file map (path -> base64 content mappings)
/// * `password` - Password for AES-256 archive encryption
/// * `temp_path_out` - Output buffer to receive the temporary file path (caller must free)
///
/// # Returns
/// * `ZipLockError::Success` on success
/// * `ZipLockError::InvalidParameter` if parameters are invalid
/// * `ZipLockError::SerializationError` if JSON parsing fails
/// * `ZipLockError::CryptoError` if archive creation/encryption fails
/// * `ZipLockError::OutOfMemory` if memory allocation fails
///
/// # Safety
/// The caller must free the returned temp_path_out string using ziplock_mobile_free_string
#[no_mangle]
pub extern "C" fn ziplock_mobile_create_temp_archive(
    files_json: *const c_char,
    password: *const c_char,
    temp_path_out: *mut *mut c_char,
) -> ZipLockError {
    if files_json.is_null() || password.is_null() || temp_path_out.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        // Initialize output to null
        *temp_path_out = ptr::null_mut();

        // Parse input parameters
        let json_str = match c_string_to_rust(files_json) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        let password_str = match c_string_to_rust(password) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        if password_str.is_empty() {
            return ZipLockError::InvalidParameter; // Archive must be encrypted
        }

        // Parse JSON file map
        let file_map_raw: HashMap<String, String> = match serde_json::from_str(&json_str) {
            Ok(map) => map,
            Err(_) => return ZipLockError::SerializationError,
        };

        // Convert base64 content to bytes
        let mut file_map = HashMap::new();
        for (path, base64_content) in file_map_raw {
            let content = match BASE64_STANDARD.decode(base64_content) {
                Ok(bytes) => bytes,
                Err(_) => return ZipLockError::SerializationError,
            };
            file_map.insert(path, content);
        }

        // Create temporary file path
        let temp_id = uuid::Uuid::new_v4();
        let temp_path = std::env::temp_dir().join(format!("ziplock_temp_{}.7z", temp_id));

        // Use DesktopFileProvider to create encrypted archive
        use crate::core::file_provider::{DesktopFileProvider, FileOperationProvider};
        let provider = DesktopFileProvider::new();

        match provider.create_archive(file_map, &password_str) {
            Ok(archive_data) => {
                // Write archive to temporary file
                match std::fs::write(&temp_path, archive_data) {
                    Ok(()) => {
                        // Return temporary file path
                        let path_string = temp_path.to_string_lossy().to_string();
                        *temp_path_out = rust_string_to_c(path_string);
                        ZipLockError::Success
                    }
                    Err(_) => ZipLockError::InternalError,
                }
            }
            Err(_) => ZipLockError::FileError,
        }
    }
}

/// Extract archive from temporary file path to file map (JSON)
///
/// This function complements the temp archive creation by providing
/// FFI-based extraction that ensures proper decryption using sevenz-rust2.
///
/// # Parameters
/// * `archive_path` - Path to the 7z archive file
/// * `password` - Password for decryption
/// * `files_json_out` - Output parameter for JSON file map (path -> base64 content)
///
/// # Returns
/// * `ZipLockError::Success` on success with file map in `files_json_out`
/// * Error code on failure
#[no_mangle]
pub extern "C" fn ziplock_mobile_extract_temp_archive(
    archive_path: *const c_char,
    password: *const c_char,
    files_json_out: *mut *mut c_char,
) -> ZipLockError {
    if archive_path.is_null() || password.is_null() || files_json_out.is_null() {
        return ZipLockError::InvalidParameter;
    }

    unsafe {
        // Initialize output to null
        *files_json_out = ptr::null_mut();

        // Parse input parameters
        let path_str = match c_string_to_rust(archive_path) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        let password_str = match c_string_to_rust(password) {
            Some(s) => s,
            None => return ZipLockError::InvalidParameter,
        };

        if password_str.is_empty() {
            return ZipLockError::InvalidParameter; // Password required for encrypted archive
        }

        // Check if archive file exists
        let archive_file_path = std::path::Path::new(&path_str);
        if !archive_file_path.exists() {
            return ZipLockError::FileNotFound;
        }

        // Use DesktopFileProvider to extract encrypted archive
        use crate::core::file_provider::{DesktopFileProvider, FileOperationProvider};
        let provider = DesktopFileProvider::new();

        // Read archive data from file
        let archive_data = match std::fs::read(archive_file_path) {
            Ok(data) => data,
            Err(_) => return ZipLockError::FileError,
        };

        // Extract using the file provider
        match provider.extract_archive(&archive_data, &password_str) {
            Ok(file_map) => {
                // Convert byte content to base64 for JSON transport
                let mut base64_map = HashMap::new();
                for (path, content) in file_map {
                    let base64_content = BASE64_STANDARD.encode(content);
                    base64_map.insert(path, base64_content);
                }

                // Serialize to JSON
                match serde_json::to_string(&base64_map) {
                    Ok(json_str) => {
                        *files_json_out = rust_string_to_c(json_str);
                        ZipLockError::Success
                    }
                    Err(_) => ZipLockError::SerializationError,
                }
            }
            Err(_) => ZipLockError::InvalidPassword,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, CredentialRecord, FieldType};

    #[test]
    fn test_mobile_repository_lifecycle() {
        // Create repository
        let handle = ziplock_mobile_repository_create();
        assert!(!handle.is_null());

        // Initialize
        let result = ziplock_mobile_repository_initialize(handle);
        assert_eq!(result, ZipLockError::Success);

        // Check initialization
        let is_init = ziplock_mobile_repository_is_initialized(handle);
        assert_eq!(is_init, 1);

        // Test double initialization
        let result = ziplock_mobile_repository_initialize(handle);
        assert_eq!(result, ZipLockError::AlreadyInitialized);

        // Destroy
        ziplock_mobile_repository_destroy(handle);
    }

    #[test]
    fn test_credential_operations() {
        let handle = ziplock_mobile_repository_create();
        ziplock_mobile_repository_initialize(handle);

        // Create test credential
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field(
            "username",
            CredentialField::new(FieldType::Username, "testuser".to_string(), false),
        );

        let credential_json = serde_json::to_string(&credential).unwrap();
        let c_json = CString::new(credential_json).unwrap();

        // Add credential
        let result = ziplock_mobile_add_credential(handle, c_json.as_ptr());
        assert_eq!(result, ZipLockError::Success);

        // Get credential
        let c_id = CString::new(credential.id.clone()).unwrap();
        let retrieved_ptr = ziplock_mobile_get_credential(handle, c_id.as_ptr());
        assert!(!retrieved_ptr.is_null());

        // Free the returned string
        ziplock_mobile_free_string(retrieved_ptr);

        // List credentials
        let list_ptr = ziplock_mobile_list_credentials(handle);
        assert!(!list_ptr.is_null());
        ziplock_mobile_free_string(list_ptr);

        // Delete credential
        let result = ziplock_mobile_delete_credential(handle, c_id.as_ptr());
        assert_eq!(result, ZipLockError::Success);

        ziplock_mobile_repository_destroy(handle);
    }

    #[test]
    fn test_file_map_serialization() {
        let handle = ziplock_mobile_repository_create();
        ziplock_mobile_repository_initialize(handle);

        // Add some test data
        let credential = CredentialRecord::new("Test".to_string(), "test".to_string());
        let credential_json = serde_json::to_string(&credential).unwrap();
        let c_json = CString::new(credential_json).unwrap();
        ziplock_mobile_add_credential(handle, c_json.as_ptr());

        // Serialize to file map
        let files_ptr = ziplock_mobile_repository_serialize_to_files(handle);
        assert!(!files_ptr.is_null());

        // Free the string
        ziplock_mobile_free_string(files_ptr);

        ziplock_mobile_repository_destroy(handle);
    }

    #[test]
    fn test_null_parameter_handling() {
        // Test null handle
        let result = ziplock_mobile_repository_initialize(ptr::null_mut());
        assert_eq!(result, ZipLockError::InvalidParameter);

        let is_init = ziplock_mobile_repository_is_initialized(ptr::null_mut());
        assert_eq!(is_init, 0);

        let result = ziplock_mobile_add_credential(ptr::null_mut(), ptr::null());
        assert_eq!(result, ZipLockError::InvalidParameter);

        // Test null credential JSON with valid handle
        let handle = ziplock_mobile_repository_create();
        let result = ziplock_mobile_add_credential(handle, ptr::null());
        assert_eq!(result, ZipLockError::InvalidParameter);

        ziplock_mobile_repository_destroy(handle);
    }

    #[test]
    fn test_repository_stats() {
        let handle = ziplock_mobile_repository_create();
        ziplock_mobile_repository_initialize(handle);

        // Get initial stats
        let stats_ptr = ziplock_mobile_get_stats(handle);
        assert!(!stats_ptr.is_null());
        ziplock_mobile_free_string(stats_ptr);

        // Add credential and check stats again
        let credential = CredentialRecord::new("Test".to_string(), "test".to_string());
        let credential_json = serde_json::to_string(&credential).unwrap();
        let c_json = CString::new(credential_json).unwrap();
        ziplock_mobile_add_credential(handle, c_json.as_ptr());

        let stats_ptr = ziplock_mobile_get_stats(handle);
        assert!(!stats_ptr.is_null());
        ziplock_mobile_free_string(stats_ptr);

        ziplock_mobile_repository_destroy(handle);
    }
}
