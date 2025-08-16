//! Services module for the ZipLock Linux application
//!
//! This module contains various services that provide functionality
//! across the application, such as clipboard management.

pub mod clipboard;

pub use clipboard::{ClipboardContentType, ClipboardManager};
