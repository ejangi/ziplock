//! Platform-specific implementations for Linux
//!
//! This module provides Linux-specific functionality for the unified architecture.
//! File operations are now handled by the shared DesktopFileProvider.

/// Check if all required system dependencies are available
#[allow(dead_code)]
pub fn check_system_dependencies() -> Result<(), String> {
    // With the unified architecture, dependencies are managed by DesktopFileProvider
    // This function is kept for potential future platform-specific checks
    Ok(())
}

/// Initialize platform-specific components
#[allow(dead_code)]
pub fn initialize_platform() -> Result<(), String> {
    // Platform initialization for Linux-specific features
    tracing::info!("Linux platform components initialized successfully");
    Ok(())
}
