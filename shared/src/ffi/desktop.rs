//! Desktop FFI interface for ZipLock
//!
//! This module provides the C-compatible FFI interface for desktop platforms
//! (Linux, Windows, macOS). It exposes full repository operations including
//! direct file I/O using the UnifiedRepositoryManager with DesktopFileProvider.
//!
//! # Architecture
//!
//! Desktop platforms can use:
//! - Direct file system access through the shared library
//! - Automatic 7z archive operations using sevenz-rust2
//! - Full repository lifecycle management
//! - Optional custom file providers for specialized storage
//!
//! Shared library handles:
//! - All credential operations
//! - File I/O through DesktopFileProvider or custom providers
//! - Archive creation/extraction with AES-256 encryption
//! - Repository persistence and consistency
//!
//! # Usage Pattern
//!
//! 1. Create repository manager with desired file provider
//! 2. Open or create repository file
//! 3. Perform credential operations
//! 4. Repository automatically handles persistence
//! 5. Close repository when done

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Mutex;

use crate::core::{CoreError, DesktopFileProvider, UnifiedRepositoryManager};
use crate::ffi::common::{c_string_to_rust, rust_string_to_c, ZipLockError};
use crate::models::CredentialRecord;

/// Handle type for desktop repository manager instances
pub type DesktopManagerHandle = *mut DesktopManagerInstance;

/// Configuration for desktop archive operations
#[repr(C)]
pub struct DesktopArchiveConfig {
    /// Compression level (0-9, where 9 is highest compression)
    pub compression_level: c_int,
    /// Whether to enable encryption (1 = enabled, 0 = disabled)
    pub encryption_enabled: c_int,
    /// Archive format version to use
    pub archive_format: c_int,
}

impl Default for DesktopArchiveConfig {
    fn default() -> Self {
        Self {
            compression_level: 7,
            encryption_enabled: 1,
            archive_format: 1,
        }
    }
}

/// Internal repository manager instance for desktop platforms
pub struct DesktopManagerInstance {
    manager: Mutex<UnifiedRepositoryManager<DesktopFileProvider>>,
}

impl DesktopManagerInstance {
    fn new() -> Self {
        let provider = DesktopFileProvider::new();
        Self {
            manager: Mutex::new(UnifiedRepositoryManager::new(provider)),
        }
    }
}

/// Desktop-specific error codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopError {
    Success = 0,
    InvalidParameter = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    FileNotFound = 4,
    InvalidPassword = 5,
    PermissionDenied = 6,
    ArchiveCorrupted = 7,
    SerializationError = 8,
    ValidationError = 9,
    OutOfMemory = 10,
    InternalError = 11,
    RepositoryNotOpen = 12,
}

impl From<ZipLockError> for DesktopError {
    fn from(err: ZipLockError) -> Self {
        match err {
            ZipLockError::Success => DesktopError::Success,
            ZipLockError::InvalidParameter => DesktopError::InvalidParameter,
            ZipLockError::NotInitialized => DesktopError::NotInitialized,
            ZipLockError::AlreadyInitialized => DesktopError::AlreadyInitialized,
            ZipLockError::SerializationError => DesktopError::SerializationError,
            ZipLockError::ValidationError => DesktopError::ValidationError,
            ZipLockError::InternalError => DesktopError::InternalError,
            ZipLockError::FileError => DesktopError::FileNotFound,
            ZipLockError::CredentialNotFound => DesktopError::InvalidParameter,
            ZipLockError::InvalidPassword => DesktopError::InvalidPassword,
            ZipLockError::CorruptedArchive => DesktopError::ArchiveCorrupted,
            ZipLockError::PermissionDenied => DesktopError::PermissionDenied,
            ZipLockError::FileNotFound => DesktopError::FileNotFound,
            ZipLockError::OutOfMemory => DesktopError::OutOfMemory,
        }
    }
}

/// Create a new desktop repository manager
///
/// # Returns
/// * Non-null handle on success
/// * Null on failure (out of memory)
///
/// # Safety
/// The returned handle must be freed with `ziplock_desktop_manager_destroy`
#[no_mangle]
pub extern "C" fn ziplock_desktop_manager_create() -> DesktopManagerHandle {
    let instance = Box::new(DesktopManagerInstance::new());
    Box::into_raw(instance)
}

/// Destroy a desktop repository manager
///
/// # Arguments
/// * `handle` - Manager handle to destroy
///
/// # Safety
/// Handle must be valid and not used after this call
#[no_mangle]
pub extern "C" fn ziplock_desktop_manager_destroy(handle: DesktopManagerHandle) {
    if handle.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(handle);
    }
}

/// Create a new repository file
///
/// # Arguments
/// * `handle` - Manager handle
/// * `path` - Path where to create the repository
/// * `password` - Master password for encryption
/// * `config` - Archive configuration (can be null for defaults)
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if parameters are invalid
/// * `DesktopError::PermissionDenied` if cannot write to path
/// * `DesktopError::InternalError` for other errors
#[no_mangle]
pub extern "C" fn ziplock_desktop_create_repository(
    handle: DesktopManagerHandle,
    path: *const c_char,
    password: *const c_char,
    config: *const DesktopArchiveConfig,
) -> DesktopError {
    if handle.is_null() || path.is_null() || password.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        let path_str = match c_string_to_rust(path) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        let password_str = match c_string_to_rust(password) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        // TODO: Use config if provided (currently using defaults)
        if !config.is_null() {
            let _config = &*config;
            // Future: Apply configuration settings
        }

        match manager.create_repository(&path_str, &password_str) {
            Ok(()) => DesktopError::Success,
            Err(CoreError::FileOperation(crate::core::FileError::PermissionDenied { .. })) => {
                DesktopError::PermissionDenied
            }
            Err(CoreError::ValidationError { .. }) => DesktopError::ValidationError,
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// Open an existing repository file
///
/// # Arguments
/// * `handle` - Manager handle
/// * `path` - Path to the repository file
/// * `password` - Master password for decryption
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if parameters are invalid
/// * `DesktopError::FileNotFound` if repository doesn't exist
/// * `DesktopError::InvalidPassword` if password is wrong
/// * `DesktopError::ArchiveCorrupted` if archive is damaged
#[no_mangle]
pub extern "C" fn ziplock_desktop_open_repository(
    handle: DesktopManagerHandle,
    path: *const c_char,
    password: *const c_char,
) -> DesktopError {
    if handle.is_null() || path.is_null() || password.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        let path_str = match c_string_to_rust(path) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        let password_str = match c_string_to_rust(password) {
            Some(s) => s,
            None => return DesktopError::InvalidPassword,
        };

        match manager.open_repository(&path_str, &password_str) {
            Ok(()) => DesktopError::Success,
            Err(CoreError::FileOperation(crate::core::FileError::NotFound { .. })) => {
                DesktopError::FileNotFound
            }
            Err(CoreError::FileOperation(crate::core::FileError::InvalidPassword)) => {
                DesktopError::InvalidPassword
            }
            Err(CoreError::FileOperation(crate::core::FileError::CorruptedArchive { .. })) => {
                DesktopError::ArchiveCorrupted
            }
            Err(CoreError::FileOperation(crate::core::FileError::PermissionDenied { .. })) => {
                DesktopError::PermissionDenied
            }
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// Save the repository to disk
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if handle is invalid
/// * `DesktopError::RepositoryNotOpen` if no repository is open
/// * `DesktopError::PermissionDenied` if cannot write to file
#[no_mangle]
pub extern "C" fn ziplock_desktop_save_repository(handle: DesktopManagerHandle) -> DesktopError {
    if handle.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        if !manager.is_open() {
            return DesktopError::RepositoryNotOpen;
        }

        match manager.save_repository() {
            Ok(()) => DesktopError::Success,
            Err(CoreError::FileOperation(crate::core::FileError::PermissionDenied { .. })) => {
                DesktopError::PermissionDenied
            }
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// Close the current repository
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if handle is invalid
#[no_mangle]
pub extern "C" fn ziplock_desktop_close_repository(handle: DesktopManagerHandle) -> DesktopError {
    if handle.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        match manager.close_repository(false) {
            Ok(()) => DesktopError::Success,
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// Add a new credential to the repository
///
/// # Arguments
/// * `handle` - Manager handle
/// * `credential_json` - JSON string containing credential data
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if parameters are invalid
/// * `DesktopError::RepositoryNotOpen` if no repository is open
/// * `DesktopError::SerializationError` if JSON parsing fails
/// * `DesktopError::ValidationError` if credential is invalid
#[no_mangle]
pub extern "C" fn ziplock_desktop_add_credential(
    handle: DesktopManagerHandle,
    credential_json: *const c_char,
) -> DesktopError {
    if handle.is_null() || credential_json.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        if !manager.is_open() {
            return DesktopError::RepositoryNotOpen;
        }

        let json_str = match c_string_to_rust(credential_json) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        let credential: CredentialRecord = match serde_json::from_str(&json_str) {
            Ok(cred) => cred,
            Err(_) => return DesktopError::SerializationError,
        };

        match manager.add_credential(credential) {
            Ok(()) => DesktopError::Success,
            Err(CoreError::ValidationError { .. }) => DesktopError::ValidationError,
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// Get a credential by ID
///
/// # Arguments
/// * `handle` - Manager handle
/// * `credential_id` - Credential ID to retrieve
///
/// # Returns
/// * JSON string containing credential data (must be freed with `ziplock_desktop_free_string`)
/// * Null if not found or error
#[no_mangle]
pub extern "C" fn ziplock_desktop_get_credential(
    handle: DesktopManagerHandle,
    credential_id: *const c_char,
) -> *mut c_char {
    if handle.is_null() || credential_id.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return ptr::null_mut(),
        };

        if !manager.is_open() {
            return ptr::null_mut();
        }

        let id_str = match c_string_to_rust(credential_id) {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match manager.get_credential_readonly(&id_str) {
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
/// * `handle` - Manager handle
/// * `credential_json` - JSON string containing updated credential data
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if parameters are invalid
/// * `DesktopError::RepositoryNotOpen` if no repository is open
/// * `DesktopError::SerializationError` if JSON parsing fails
/// * `DesktopError::ValidationError` if credential is invalid
#[no_mangle]
pub extern "C" fn ziplock_desktop_update_credential(
    handle: DesktopManagerHandle,
    credential_json: *const c_char,
) -> DesktopError {
    if handle.is_null() || credential_json.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        if !manager.is_open() {
            return DesktopError::RepositoryNotOpen;
        }

        let json_str = match c_string_to_rust(credential_json) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        let credential: CredentialRecord = match serde_json::from_str(&json_str) {
            Ok(cred) => cred,
            Err(_) => return DesktopError::SerializationError,
        };

        match manager.update_credential(credential) {
            Ok(()) => DesktopError::Success,
            Err(CoreError::CredentialNotFound { .. }) => DesktopError::InvalidParameter,
            Err(CoreError::ValidationError { .. }) => DesktopError::ValidationError,
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// Delete a credential by ID
///
/// # Arguments
/// * `handle` - Manager handle
/// * `credential_id` - ID of credential to delete
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if parameters are invalid
/// * `DesktopError::RepositoryNotOpen` if no repository is open
#[no_mangle]
pub extern "C" fn ziplock_desktop_delete_credential(
    handle: DesktopManagerHandle,
    credential_id: *const c_char,
) -> DesktopError {
    if handle.is_null() || credential_id.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        if !manager.is_open() {
            return DesktopError::RepositoryNotOpen;
        }

        let id_str = match c_string_to_rust(credential_id) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        match manager.delete_credential(&id_str) {
            Ok(_) => DesktopError::Success,
            Err(CoreError::CredentialNotFound { .. }) => DesktopError::InvalidParameter,
            Err(_) => DesktopError::InternalError,
        }
    }
}

/// List all credentials in the repository
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * JSON array string containing credential summaries (must be freed with `ziplock_desktop_free_string`)
/// * Null if error
#[no_mangle]
pub extern "C" fn ziplock_desktop_list_credentials(handle: DesktopManagerHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return ptr::null_mut(),
        };

        if !manager.is_open() {
            return ptr::null_mut();
        }

        match manager.list_credential_summaries() {
            Ok(summaries) => match serde_json::to_string(&summaries) {
                Ok(json) => rust_string_to_c(json),
                Err(_) => ptr::null_mut(),
            },
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Check if repository is open
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * 1 if repository is open, 0 if not open or handle is invalid
#[no_mangle]
pub extern "C" fn ziplock_desktop_is_open(handle: DesktopManagerHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }

    unsafe {
        let instance = &*handle;
        let manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return 0,
        };

        if manager.is_open() {
            1
        } else {
            0
        }
    }
}

/// Check if repository has been modified
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * 1 if modified, 0 if not modified or handle is invalid
#[no_mangle]
pub extern "C" fn ziplock_desktop_is_modified(handle: DesktopManagerHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }

    unsafe {
        let instance = &*handle;
        let manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return 0,
        };

        if manager.is_modified() {
            1
        } else {
            0
        }
    }
}

/// Get current repository path
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * String containing current repository path (must be freed with `ziplock_desktop_free_string`)
/// * Null if no repository is open or error
#[no_mangle]
pub extern "C" fn ziplock_desktop_current_path(handle: DesktopManagerHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return ptr::null_mut(),
        };

        match manager.current_path() {
            Some(path) => rust_string_to_c(path.to_string()),
            None => ptr::null_mut(),
        }
    }
}

/// Get repository statistics
///
/// # Arguments
/// * `handle` - Manager handle
///
/// # Returns
/// * JSON string containing repository stats (must be freed with `ziplock_desktop_free_string`)
/// * Null if error
#[no_mangle]
pub extern "C" fn ziplock_desktop_get_stats(handle: DesktopManagerHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let instance = &*handle;
        let manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return ptr::null_mut(),
        };

        if !manager.is_open() {
            return ptr::null_mut();
        }

        match manager.get_stats() {
            Ok(stats) => match serde_json::to_string(&stats) {
                Ok(json) => rust_string_to_c(json),
                Err(_) => ptr::null_mut(),
            },
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Change the master password of the repository
///
/// # Arguments
/// * `handle` - Manager handle
/// * `new_password` - New master password
///
/// # Returns
/// * `DesktopError::Success` on success
/// * `DesktopError::InvalidParameter` if parameters are invalid
/// * `DesktopError::RepositoryNotOpen` if no repository is open
#[no_mangle]
pub extern "C" fn ziplock_desktop_change_password(
    handle: DesktopManagerHandle,
    new_password: *const c_char,
) -> DesktopError {
    if handle.is_null() || new_password.is_null() {
        return DesktopError::InvalidParameter;
    }

    unsafe {
        let instance = &*handle;
        let mut manager = match instance.manager.lock() {
            Ok(mgr) => mgr,
            Err(_) => return DesktopError::InternalError,
        };

        if !manager.is_open() {
            return DesktopError::RepositoryNotOpen;
        }

        let password_str = match c_string_to_rust(new_password) {
            Some(s) => s,
            None => return DesktopError::InvalidParameter,
        };

        match manager.change_master_password(&password_str) {
            Ok(()) => DesktopError::Success,
            Err(_) => DesktopError::InternalError,
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
pub extern "C" fn ziplock_desktop_free_string(str_ptr: *mut c_char) {
    if str_ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(str_ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CredentialField, CredentialRecord, FieldType};
    use std::path::PathBuf;

    fn get_test_results_dir() -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        // Go up one level from shared/ to project root
        path.pop();
        path.push("tests");
        path.push("results");
        std::fs::create_dir_all(&path).ok();
        path
    }

    #[test]
    fn test_desktop_manager_lifecycle() {
        // Create manager
        let handle = ziplock_desktop_manager_create();
        assert!(!handle.is_null());

        // Check initial state
        let is_open = ziplock_desktop_is_open(handle);
        assert_eq!(is_open, 0);

        // Destroy
        ziplock_desktop_manager_destroy(handle);
    }

    #[test]
    fn test_repository_operations() {
        let test_dir = get_test_results_dir();
        let repo_path = test_dir.join("test_operations.7z");
        let repo_path_str = repo_path.to_string_lossy();

        let handle = ziplock_desktop_manager_create();

        // Create repository
        let path_cstr = CString::new(repo_path_str.as_ref()).unwrap();
        let password_cstr = CString::new("testpassword").unwrap();

        let result = ziplock_desktop_create_repository(
            handle,
            path_cstr.as_ptr(),
            password_cstr.as_ptr(),
            ptr::null(),
        );
        assert_eq!(result, DesktopError::Success);

        // Check if open
        let is_open = ziplock_desktop_is_open(handle);
        assert_eq!(is_open, 1);

        // Add credential
        let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());
        credential.set_field(
            "username",
            CredentialField::new(FieldType::Username, "testuser".to_string(), false),
        );

        let credential_json = serde_json::to_string(&credential).unwrap();
        let cred_cstr = CString::new(credential_json).unwrap();

        let result = ziplock_desktop_add_credential(handle, cred_cstr.as_ptr());
        assert_eq!(result, DesktopError::Success);

        // Get credential
        let id_cstr = CString::new(credential.id.clone()).unwrap();
        let retrieved_ptr = ziplock_desktop_get_credential(handle, id_cstr.as_ptr());
        assert!(!retrieved_ptr.is_null());
        ziplock_desktop_free_string(retrieved_ptr);

        // List credentials
        let list_ptr = ziplock_desktop_list_credentials(handle);
        assert!(!list_ptr.is_null());
        ziplock_desktop_free_string(list_ptr);

        // Save repository
        let result = ziplock_desktop_save_repository(handle);
        assert_eq!(result, DesktopError::Success);

        // Close repository
        let result = ziplock_desktop_close_repository(handle);
        assert_eq!(result, DesktopError::Success);

        // Check if closed
        let is_open = ziplock_desktop_is_open(handle);
        assert_eq!(is_open, 0);

        ziplock_desktop_manager_destroy(handle);
    }

    #[test]
    fn test_open_existing_repository() {
        let test_dir = get_test_results_dir();
        let repo_path = test_dir.join("existing.7z");
        let repo_path_str = repo_path.to_string_lossy();
        let path_cstr = CString::new(repo_path_str.as_ref()).unwrap();
        let password_cstr = CString::new("password123").unwrap();

        // Create initial repository
        let handle1 = ziplock_desktop_manager_create();
        let result = ziplock_desktop_create_repository(
            handle1,
            path_cstr.as_ptr(),
            password_cstr.as_ptr(),
            ptr::null(),
        );
        assert_eq!(result, DesktopError::Success);

        // Add a credential
        let credential = CredentialRecord::new("Existing".to_string(), "login".to_string());
        let credential_json = serde_json::to_string(&credential).unwrap();
        let cred_cstr = CString::new(credential_json).unwrap();
        ziplock_desktop_add_credential(handle1, cred_cstr.as_ptr());

        // Save and close
        ziplock_desktop_save_repository(handle1);
        ziplock_desktop_close_repository(handle1);
        ziplock_desktop_manager_destroy(handle1);

        // Open with new manager
        let handle2 = ziplock_desktop_manager_create();
        let result =
            ziplock_desktop_open_repository(handle2, path_cstr.as_ptr(), password_cstr.as_ptr());
        assert_eq!(result, DesktopError::Success);

        // Verify credential exists
        let id_cstr = CString::new(credential.id).unwrap();
        let retrieved_ptr = ziplock_desktop_get_credential(handle2, id_cstr.as_ptr());
        assert!(!retrieved_ptr.is_null());
        ziplock_desktop_free_string(retrieved_ptr);

        ziplock_desktop_manager_destroy(handle2);
    }

    #[test]
    fn test_error_conditions() {
        // Test null handle
        let result = ziplock_desktop_create_repository(
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
        );
        assert_eq!(result, DesktopError::InvalidParameter);

        // Test operations on closed repository
        let handle = ziplock_desktop_manager_create();

        let credential_json = r#"{"id":"test","title":"Test"}"#;
        let cred_cstr = CString::new(credential_json).unwrap();
        let result = ziplock_desktop_add_credential(handle, cred_cstr.as_ptr());
        assert_eq!(result, DesktopError::RepositoryNotOpen);

        // Test opening non-existent file
        let path_cstr = CString::new("/nonexistent/path.7z").unwrap();
        let password_cstr = CString::new("password").unwrap();
        let result =
            ziplock_desktop_open_repository(handle, path_cstr.as_ptr(), password_cstr.as_ptr());
        assert_eq!(result, DesktopError::FileNotFound);

        ziplock_desktop_manager_destroy(handle);
    }

    #[test]
    fn test_invalid_password() {
        let test_dir = get_test_results_dir();
        let repo_path = test_dir.join("password_test.7z");
        let repo_path_str = repo_path.to_string_lossy();
        let path_cstr = CString::new(repo_path_str.as_ref()).unwrap();
        let password_cstr = CString::new("correct").unwrap();
        let wrong_password_cstr = CString::new("wrong").unwrap();

        // Create repository with correct password
        let handle1 = ziplock_desktop_manager_create();
        ziplock_desktop_create_repository(
            handle1,
            path_cstr.as_ptr(),
            password_cstr.as_ptr(),
            ptr::null(),
        );
        ziplock_desktop_save_repository(handle1);
        ziplock_desktop_close_repository(handle1);
        ziplock_desktop_manager_destroy(handle1);

        // Try to open with wrong password
        let handle2 = ziplock_desktop_manager_create();
        let result = ziplock_desktop_open_repository(
            handle2,
            path_cstr.as_ptr(),
            wrong_password_cstr.as_ptr(),
        );
        assert_eq!(result, DesktopError::InvalidPassword);

        ziplock_desktop_manager_destroy(handle2);
    }

    #[test]
    fn test_repository_stats() {
        let test_dir = get_test_results_dir();
        let repo_path = test_dir.join("stats.7z");
        let repo_path_str = repo_path.to_string_lossy();

        let handle = ziplock_desktop_manager_create();
        let path_cstr = CString::new(repo_path_str.as_ref()).unwrap();
        let password_cstr = CString::new("password").unwrap();

        ziplock_desktop_create_repository(
            handle,
            path_cstr.as_ptr(),
            password_cstr.as_ptr(),
            ptr::null(),
        );

        // Get initial stats
        let stats_ptr = ziplock_desktop_get_stats(handle);
        assert!(!stats_ptr.is_null());
        ziplock_desktop_free_string(stats_ptr);

        // Add credential and check stats again
        let credential = CredentialRecord::new("Test".to_string(), "test".to_string());
        let credential_json = serde_json::to_string(&credential).unwrap();
        let cred_cstr = CString::new(credential_json).unwrap();
        ziplock_desktop_add_credential(handle, cred_cstr.as_ptr());

        let stats_ptr = ziplock_desktop_get_stats(handle);
        assert!(!stats_ptr.is_null());
        ziplock_desktop_free_string(stats_ptr);

        ziplock_desktop_manager_destroy(handle);
    }

    #[test]
    fn test_change_password() {
        let test_dir = get_test_results_dir();
        let repo_path = test_dir.join("change_password.7z");
        let repo_path_str = repo_path.to_string_lossy();

        let handle = ziplock_desktop_manager_create();
        let path_cstr = CString::new(repo_path_str.as_ref()).unwrap();
        let old_password_cstr = CString::new("oldpassword").unwrap();
        let new_password_cstr = CString::new("newpassword").unwrap();

        ziplock_desktop_create_repository(
            handle,
            path_cstr.as_ptr(),
            old_password_cstr.as_ptr(),
            ptr::null(),
        );

        // Change password
        let result = ziplock_desktop_change_password(handle, new_password_cstr.as_ptr());
        assert_eq!(result, DesktopError::Success);

        ziplock_desktop_manager_destroy(handle);
    }
}
