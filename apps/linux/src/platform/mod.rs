//! Platform-specific implementations for Linux
//!
//! This module provides Linux-specific functionality that complements
//! the adaptive hybrid architecture, particularly for handling external
//! file operations when runtime conflicts are detected.

pub mod file_operations;

pub use file_operations::{
    FileOperationInstruction, FileOperations, LinuxFileOperationError, LinuxFileOperationResult,
    LinuxFileOperationsHandler,
};

/// Check if all required system dependencies are available
pub fn check_system_dependencies() -> Result<(), String> {
    // Check for 7z availability
    if !LinuxFileOperationsHandler::check_7z_availability() {
        return Err(
            "7z command not found. Please install p7zip-full package: sudo apt install p7zip-full"
                .to_string(),
        );
    }

    Ok(())
}

/// Initialize platform-specific components
pub fn initialize_platform() -> Result<(), String> {
    // Check system dependencies
    check_system_dependencies()?;

    tracing::info!("Linux platform components initialized successfully");
    Ok(())
}
