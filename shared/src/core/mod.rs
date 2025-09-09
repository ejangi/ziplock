//! Core modules for the ZipLock unified architecture
//!
//! This module contains the core components of the unified architecture:
//! - Pure memory repository for credential operations
//! - File operation provider interface for platform abstraction
//! - Repository manager that coordinates memory and file operations
//! - Error handling and type definitions

pub mod errors;
pub mod file_provider;
pub mod memory_repository;
pub mod plugins;
pub mod repository_manager;
pub mod types;

// Re-export commonly used items
pub use errors::{CoreError, CoreResult, FileError, FileResult};
pub use file_provider::{DesktopFileProvider, FileOperationProvider, MockFileProvider};
pub use memory_repository::UnifiedMemoryRepository;
pub use plugins::{
    Plugin, PluginCapability, PluginManager, PluginMetadata, PluginRegistry, ValidationRule,
    ValidationSeverity,
};
pub use repository_manager::UnifiedRepositoryManager;
pub use types::{FileMap, RepositoryMetadata, RepositoryStats};

/// Version information for the core library
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Repository format version
pub const REPOSITORY_FORMAT_VERSION: &str = "1.0";

/// Default repository structure version
pub const REPOSITORY_STRUCTURE_VERSION: &str = "1.0";
