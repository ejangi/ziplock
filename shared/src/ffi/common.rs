//! Common FFI utilities and error handling for ZipLock
//!
//! This module provides shared functionality used by both mobile and desktop
//! FFI interfaces, including error code conversion, string handling, and
//! common data structures.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::core::errors::{CoreError, FileError};

/// FFI-compatible error codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipLockError {
    /// Operation completed successfully
    Success = 0,
    /// Invalid parameter passed to function
    InvalidParameter = 1,
    /// Repository not initialized
    NotInitialized = 2,
    /// Repository already initialized
    AlreadyInitialized = 3,
    /// Serialization/deserialization error
    SerializationError = 4,
    /// Data validation error
    ValidationError = 5,
    /// Out of memory
    OutOfMemory = 6,
    /// File operation error
    FileError = 7,
    /// Credential not found
    CredentialNotFound = 8,
    /// Invalid password
    InvalidPassword = 9,
    /// Archive corrupted or invalid
    CorruptedArchive = 10,
    /// Permission denied
    PermissionDenied = 11,
    /// File not found
    FileNotFound = 12,
    /// Internal error
    InternalError = 99,
}

impl From<CoreError> for ZipLockError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::NotInitialized => ZipLockError::NotInitialized,
            CoreError::AlreadyInitialized => ZipLockError::AlreadyInitialized,
            CoreError::CredentialNotFound { .. } => ZipLockError::CredentialNotFound,
            CoreError::ValidationError { .. } => ZipLockError::ValidationError,
            CoreError::SerializationError { .. } => ZipLockError::SerializationError,
            CoreError::InvalidCredential { .. } => ZipLockError::ValidationError,
            CoreError::StructureError { .. } => ZipLockError::SerializationError,
            CoreError::InternalError { .. } => ZipLockError::InternalError,
            CoreError::FileOperation(file_error) => file_error.into(),
        }
    }
}

impl From<FileError> for ZipLockError {
    fn from(error: FileError) -> Self {
        match error {
            FileError::NotFound { .. } => ZipLockError::FileNotFound,
            FileError::PermissionDenied { .. } => ZipLockError::PermissionDenied,
            FileError::ExtractionFailed { .. } => ZipLockError::FileError,
            FileError::CreationFailed { .. } => ZipLockError::FileError,
            FileError::InvalidPassword => ZipLockError::InvalidPassword,
            FileError::CorruptedArchive { .. } => ZipLockError::CorruptedArchive,
            FileError::IoError { .. } => ZipLockError::FileError,
        }
    }
}

/// Convert a Rust string to a C string
///
/// Returns a pointer to a null-terminated C string that must be freed
/// with `ziplock_free_string`. Returns null on allocation failure.
pub fn rust_string_to_c(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert a C string to a Rust string
///
/// Returns None if the pointer is null or the string is not valid UTF-8
pub fn c_string_to_rust(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe {
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        }
    }
}

/// Free a string allocated by the shared library
///
/// This must be called for every string returned by the shared library
/// to prevent memory leaks.
///
/// # Safety
/// The pointer must have been returned by `rust_string_to_c` or another
/// shared library function that allocates strings.
#[no_mangle]
pub unsafe extern "C" fn ziplock_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Repository handle type for FFI
pub type RepositoryHandle = *mut std::ffi::c_void;

/// Credential handle type for FFI
pub type CredentialHandle = *mut std::ffi::c_void;

/// Convert a boxed value to a handle
pub fn box_to_handle<T>(value: Box<T>) -> *mut std::ffi::c_void {
    Box::into_raw(value) as *mut std::ffi::c_void
}

/// Convert a handle back to a boxed value
///
/// # Safety
/// The handle must have been created by `box_to_handle` and not yet consumed
pub unsafe fn handle_to_box<T>(handle: *mut std::ffi::c_void) -> Option<Box<T>> {
    if handle.is_null() {
        None
    } else {
        Some(Box::from_raw(handle as *mut T))
    }
}

/// Get a reference to the value behind a handle
///
/// # Safety
/// The handle must be valid and point to a value of type T
pub unsafe fn handle_to_ref<'a, T>(handle: *mut std::ffi::c_void) -> Option<&'a T> {
    if handle.is_null() {
        None
    } else {
        Some(&*(handle as *mut T))
    }
}

/// Get a mutable reference to the value behind a handle
///
/// # Safety
/// The handle must be valid and point to a value of type T
pub unsafe fn handle_to_mut<'a, T>(handle: *mut std::ffi::c_void) -> Option<&'a mut T> {
    if handle.is_null() {
        None
    } else {
        Some(&mut *(handle as *mut T))
    }
}

/// Macro for safely executing FFI operations with error handling
#[macro_export]
macro_rules! ffi_try {
    ($operation:expr) => {
        match $operation {
            Ok(value) => value,
            Err(error) => {
                let error_code: $crate::ffi::common::ZipLockError = error.into();
                return error_code;
            }
        }
    };
}

/// Macro for safely executing FFI operations that return handles
#[macro_export]
macro_rules! ffi_try_handle {
    ($operation:expr, $error_ptr:expr) => {
        match $operation {
            Ok(value) => value,
            Err(error) => {
                if !$error_ptr.is_null() {
                    unsafe {
                        *$error_ptr = error.into();
                    }
                }
                return std::ptr::null_mut();
            }
        }
    };
}

/// Macro for safely executing FFI operations that return strings
#[macro_export]
macro_rules! ffi_try_string {
    ($operation:expr, $error_ptr:expr) => {
        match $operation {
            Ok(value) => value,
            Err(error) => {
                if !$error_ptr.is_null() {
                    unsafe {
                        *$error_ptr = error.into();
                    }
                }
                return std::ptr::null_mut();
            }
        }
    };
}

/// Version information structure for FFI
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl VersionInfo {
    /// Create version info from version string
    pub fn from_version_string(version: &str) -> Self {
        let parts: Vec<&str> = version.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Get library version information
#[no_mangle]
pub extern "C" fn ziplock_get_version() -> VersionInfo {
    VersionInfo::from_version_string(env!("CARGO_PKG_VERSION"))
}

/// Get last error message (if any)
///
/// This is not thread-safe and should only be used for debugging.
/// Returns null if no error message is available.
#[no_mangle]
pub extern "C" fn ziplock_get_last_error() -> *mut c_char {
    // In a real implementation, we might store the last error in thread-local storage
    // For now, we return a generic message
    rust_string_to_c("Check function return codes for error information".to_string())
}

/// Log level constants for FFI
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiLogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl From<crate::logging::LogLevel> for FfiLogLevel {
    fn from(level: crate::logging::LogLevel) -> Self {
        match level {
            crate::logging::LogLevel::Error => FfiLogLevel::Error,
            crate::logging::LogLevel::Warn => FfiLogLevel::Warn,
            crate::logging::LogLevel::Info => FfiLogLevel::Info,
            crate::logging::LogLevel::Debug => FfiLogLevel::Debug,
            crate::logging::LogLevel::Trace => FfiLogLevel::Trace,
        }
    }
}

impl From<FfiLogLevel> for crate::logging::LogLevel {
    fn from(level: FfiLogLevel) -> Self {
        match level {
            FfiLogLevel::Error => crate::logging::LogLevel::Error,
            FfiLogLevel::Warn => crate::logging::LogLevel::Warn,
            FfiLogLevel::Info => crate::logging::LogLevel::Info,
            FfiLogLevel::Debug => crate::logging::LogLevel::Debug,
            FfiLogLevel::Trace => crate::logging::LogLevel::Trace,
        }
    }
}

/// Set logging level
#[no_mangle]
pub extern "C" fn ziplock_set_log_level(level: FfiLogLevel) -> ZipLockError {
    let rust_level: crate::logging::LogLevel = level.into();
    crate::logging::set_debug_enabled(rust_level >= crate::logging::LogLevel::Debug);
    ZipLockError::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_error_conversion() {
        let core_error = CoreError::NotInitialized;
        let ffi_error: ZipLockError = core_error.into();
        assert_eq!(ffi_error, ZipLockError::NotInitialized);

        let file_error = FileError::InvalidPassword;
        let ffi_error: ZipLockError = file_error.into();
        assert_eq!(ffi_error, ZipLockError::InvalidPassword);
    }

    #[test]
    fn test_string_conversion() {
        let rust_string = "Hello, World!".to_string();
        let c_ptr = rust_string_to_c(rust_string.clone());
        assert!(!c_ptr.is_null());

        let converted_back = c_string_to_rust(c_ptr);
        assert_eq!(converted_back, Some(rust_string));

        unsafe {
            ziplock_free_string(c_ptr);
        }
    }

    #[test]
    fn test_null_string_handling() {
        let result = c_string_to_rust(ptr::null());
        assert_eq!(result, None);

        unsafe {
            ziplock_free_string(ptr::null_mut()); // Should not crash
        }
    }

    #[test]
    fn test_handle_operations() {
        let value = Box::new(42i32);
        let handle = box_to_handle(value);
        assert!(!handle.is_null());

        unsafe {
            let ref_value = handle_to_ref::<i32>(handle);
            assert_eq!(ref_value, Some(&42));

            let mut_value = handle_to_mut::<i32>(handle);
            assert_eq!(mut_value, Some(&mut 42));

            let boxed_back = handle_to_box::<i32>(handle);
            assert_eq!(boxed_back, Some(Box::new(42)));
        }
    }

    #[test]
    fn test_null_handle_operations() {
        unsafe {
            let ref_value = handle_to_ref::<i32>(ptr::null_mut());
            assert_eq!(ref_value, None);

            let mut_value = handle_to_mut::<i32>(ptr::null_mut());
            assert_eq!(mut_value, None);

            let boxed = handle_to_box::<i32>(ptr::null_mut());
            assert_eq!(boxed, None);
        }
    }

    #[test]
    fn test_version_info() {
        let version = VersionInfo::from_version_string("1.2.3");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);

        let version = VersionInfo::from_version_string("0.1");
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);

        let version = ziplock_get_version();
        // Just ensure it doesn't crash
        assert!(version.major < 100); // Just ensure it's reasonable
    }

    #[test]
    fn test_log_level_conversion() {
        let ffi_level = FfiLogLevel::Debug;
        let rust_level: crate::logging::LogLevel = ffi_level.into();
        let back_to_ffi: FfiLogLevel = rust_level.into();
        assert_eq!(ffi_level, back_to_ffi);
    }

    #[test]
    fn test_set_log_level() {
        let result = ziplock_set_log_level(FfiLogLevel::Debug);
        assert_eq!(result, ZipLockError::Success);
        assert!(crate::logging::is_debug_enabled());

        let result = ziplock_set_log_level(FfiLogLevel::Info);
        assert_eq!(result, ZipLockError::Success);
        assert!(!crate::logging::is_debug_enabled());
    }

    #[test]
    fn test_get_last_error() {
        let error_ptr = ziplock_get_last_error();
        assert!(!error_ptr.is_null());

        let error_message = c_string_to_rust(error_ptr);
        assert!(error_message.is_some());
        assert!(!error_message.unwrap().is_empty());

        unsafe {
            ziplock_free_string(error_ptr);
        }
    }
}
