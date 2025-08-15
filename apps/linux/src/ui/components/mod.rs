//! UI Components Module
//!
//! This module contains reusable UI components for the ZipLock Linux app.

pub mod button;
pub mod credential_form;
pub mod toast;
pub mod totp_field;

// Future UI components will be added here as needed:
// - Password strength indicator
// - Credential list items
// - Dialog boxes
// - Custom input widgets
// etc.

// Re-export components that are actually used by other modules
pub use credential_form::{CredentialForm, CredentialFormConfig, CredentialFormMessage};
