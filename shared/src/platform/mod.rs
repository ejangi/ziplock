//! Platform-specific utilities and detection functions
//!
//! This module provides platform detection and compatibility checking
//! functions for different operating systems and environments.

#[cfg(target_os = "android")]
pub mod android;

#[cfg(not(target_os = "android"))]
pub mod android {
    //! Android platform utilities (stub for non-Android platforms)

    /// Check if running in Android emulator (always false on non-Android)
    pub fn is_android_emulator() -> bool {
        false
    }

    /// Check if running on Android (always false on non-Android)
    pub fn is_android() -> bool {
        false
    }

    /// Get Android environment description
    pub fn get_android_environment_description() -> String {
        "Not Android".to_string()
    }

    /// Check for Android archive issues (always false on non-Android)
    pub fn has_android_archive_issues() -> bool {
        false
    }

    /// Get Android archive warning (always None on non-Android)
    pub fn get_android_archive_warning() -> Option<String> {
        None
    }

    /// Log Android platform info (no-op on non-Android)
    pub fn log_android_platform_info() {
        // No-op for non-Android platforms
    }
}

/// Cross-platform emulator detection
///
/// This function provides a unified interface for detecting
/// emulator environments across different platforms.
pub fn is_emulator() -> bool {
    android::is_android_emulator()
    // TODO: Add other platform emulator detection here
    // || windows::is_windows_emulator()
    // || macos::is_macos_simulator()
}

/// Get platform-specific compatibility warnings
///
/// Returns a warning message if the current platform has known
/// compatibility issues with certain operations.
pub fn get_platform_compatibility_warning() -> Option<String> {
    android::get_android_archive_warning()
    // TODO: Add other platform warnings here
}

/// Check if current platform has known archive operation issues
pub fn has_archive_compatibility_issues() -> bool {
    android::has_android_archive_issues()
    // TODO: Add other platform checks here
}

/// Log comprehensive platform information for debugging
pub fn log_platform_info() {
    android::log_android_platform_info();
    // TODO: Add other platform logging here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_detection() {
        // Should not panic
        let _is_emu = is_emulator();
    }

    #[test]
    fn test_compatibility_warning() {
        // Should not panic
        let _warning = get_platform_compatibility_warning();
    }

    #[test]
    fn test_archive_issues() {
        // Should not panic
        let _has_issues = has_archive_compatibility_issues();
    }

    #[test]
    fn test_platform_logging() {
        // Should not panic
        log_platform_info();
    }
}
