//! ZipLock Backend Library
//!
//! This module exposes the backend functionality as a library so it can be
//! used by tests and examples.

pub mod api;
pub mod config;
pub mod error;
pub mod ipc;
pub mod storage;

// Re-export commonly used types
pub use api::ApiHandlers;
pub use config::Config;
pub use error::{BackendError, BackendResult};
pub use storage::ArchiveManager;
