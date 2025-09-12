//! Cross-platform desktop implementations
//!
//! This module provides platform-specific functionality for the unified desktop application.
//! File operations are handled by the shared DesktopFileProvider, while this module
//! handles platform-specific integration like file associations, system tray, etc.

// Platform-specific modules
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

// Re-export platform-specific functionality
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
pub use windows::*;

/// Common trait for platform integration
pub trait PlatformIntegration {
    /// Register file associations for .7z and .zip files
    fn register_file_associations(&self) -> Result<(), String>;

    /// Setup system tray/notification area integration
    fn setup_system_tray(&self) -> Result<(), String>;

    /// Get the native theme preference (light/dark/system)
    fn get_native_theme(&self) -> String;

    /// Open a file or URL using the system default handler
    fn open_with_system(&self, path: &str) -> Result<(), String>;

    /// Show a native file picker dialog
    fn show_file_picker(
        &self,
        title: &str,
        filters: &[(&str, &[&str])],
    ) -> Result<Option<String>, String>;
}

/// Check if all required system dependencies are available
pub fn check_system_dependencies() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        // Check for GTK4 and other Linux dependencies
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            return Err("No display server found (DISPLAY or WAYLAND_DISPLAY)".to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows-specific dependency checks if needed
    }

    #[cfg(target_os = "macos")]
    {
        // macOS-specific dependency checks if needed
    }

    Ok(())
}

/// Initialize platform-specific components
pub fn initialize_platform() -> Result<Box<dyn PlatformIntegration>, String> {
    #[cfg(target_os = "linux")]
    {
        tracing::info!("Initializing Linux platform components");
        Ok(Box::new(linux::LinuxIntegration::new()?))
    }

    #[cfg(target_os = "windows")]
    {
        tracing::info!("Initializing Windows platform components");
        Ok(Box::new(windows::WindowsIntegration::new()?))
    }

    #[cfg(target_os = "macos")]
    {
        tracing::info!("Initializing macOS platform components");
        Ok(Box::new(macos::MacOSIntegration::new()?))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}
