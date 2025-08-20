//! Services module for the ZipLock Linux application
//!
//! This module contains various services that provide functionality
//! across the application, such as clipboard management.

pub mod clipboard;
pub mod credential_store;

pub use clipboard::{ClipboardContentType, ClipboardManager};
pub use credential_store::{get_credential_store, CredentialStore, SimpleCredential};
