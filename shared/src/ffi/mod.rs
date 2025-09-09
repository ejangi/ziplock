//! Foreign Function Interface (FFI) modules for ZipLock
//!
//! This module provides FFI interfaces for different platforms to integrate
//! with the ZipLock shared library. It includes platform-specific optimizations
//! and interfaces that respect the capabilities and constraints of each target.

pub mod common;
pub mod desktop;
pub mod mobile;

// Re-export common functionality
pub use common::{
    c_string_to_rust, rust_string_to_c, ziplock_free_string, ziplock_get_version,
    ziplock_set_log_level, CredentialHandle, FfiLogLevel, RepositoryHandle, VersionInfo,
    ZipLockError,
};

// Re-export platform-specific modules
pub use desktop::{
    ziplock_desktop_add_credential, ziplock_desktop_change_password,
    ziplock_desktop_close_repository, ziplock_desktop_create_repository,
    ziplock_desktop_current_path, ziplock_desktop_delete_credential, ziplock_desktop_free_string,
    ziplock_desktop_get_credential, ziplock_desktop_get_stats, ziplock_desktop_is_modified,
    ziplock_desktop_is_open, ziplock_desktop_list_credentials, ziplock_desktop_manager_create,
    ziplock_desktop_manager_destroy, ziplock_desktop_open_repository,
    ziplock_desktop_save_repository, ziplock_desktop_update_credential, DesktopArchiveConfig,
    DesktopError, DesktopManagerHandle,
};
pub use mobile::{
    ziplock_mobile_add_credential, ziplock_mobile_clear_credentials,
    ziplock_mobile_create_temp_archive, ziplock_mobile_delete_credential,
    ziplock_mobile_extract_temp_archive, ziplock_mobile_free_string, ziplock_mobile_get_credential,
    ziplock_mobile_get_stats, ziplock_mobile_is_modified, ziplock_mobile_list_credentials,
    ziplock_mobile_mark_saved, ziplock_mobile_repository_create, ziplock_mobile_repository_destroy,
    ziplock_mobile_repository_initialize, ziplock_mobile_repository_is_initialized,
    ziplock_mobile_repository_load_from_files, ziplock_mobile_repository_serialize_to_files,
    ziplock_mobile_update_credential, MobileRepositoryHandle,
};

/// Check if this is a mobile platform build
pub const fn is_mobile_build() -> bool {
    cfg!(any(target_os = "android", target_os = "ios"))
}

/// Check if this is a desktop platform build
pub const fn is_desktop_build() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))
}

/// Get platform identifier string
pub fn get_platform_name() -> &'static str {
    #[cfg(target_os = "android")]
    return "Android";

    #[cfg(target_os = "ios")]
    return "iOS";

    #[cfg(target_os = "linux")]
    return "Linux";

    #[cfg(target_os = "windows")]
    return "Windows";

    #[cfg(target_os = "macos")]
    return "macOS";

    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    return "Unknown";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = get_platform_name();
        assert!(!platform.is_empty());
        assert!(platform != "Unknown"); // Should detect our platform correctly

        // Test build flags
        if is_mobile_build() {
            assert!(!is_desktop_build());
        } else if is_desktop_build() {
            assert!(!is_mobile_build());
        }
    }

    #[test]
    fn test_version_functions() {
        let version = ziplock_get_version();
        assert!(version.major < 100); // Just ensure it's reasonable
        assert!(version.minor < 100); // Just ensure it's reasonable
        assert!(version.patch < 1000); // Just ensure it's reasonable
    }

    #[test]
    fn test_string_handling() {
        let test_string = "Hello, FFI World!".to_string();
        let c_ptr = rust_string_to_c(test_string.clone());
        assert!(!c_ptr.is_null());

        let converted_back = c_string_to_rust(c_ptr);
        assert_eq!(converted_back, Some(test_string));

        unsafe {
            ziplock_free_string(c_ptr);
        }
    }

    #[test]
    fn test_log_level_setting() {
        let result = ziplock_set_log_level(FfiLogLevel::Debug);
        assert_eq!(result, ZipLockError::Success);

        let result = ziplock_set_log_level(FfiLogLevel::Info);
        assert_eq!(result, ZipLockError::Success);
    }
}
