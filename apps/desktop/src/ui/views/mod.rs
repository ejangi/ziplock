//! UI Views Module
//!
//! This module contains the main views for the ZipLock Linux app.
//! Views represent complete screens or major UI sections.

// Views will be added here as needed:
// - Main password list view
// - Credential detail view
// - Search/filter view
// - Settings view
// - About view
// etc.

pub mod add_credential;
pub mod edit_credential;
pub mod main;
pub mod open_repository;
pub mod settings;
pub mod wizard;

// Re-export views that are actually used by main.rs
pub use add_credential::{AddCredentialMessage, AddCredentialView};
pub use edit_credential::{EditCredentialMessage, EditCredentialView};
pub use open_repository::{OpenRepositoryMessage, OpenRepositoryView};
pub use settings::{SettingsMessage, SettingsView};
pub use wizard::{RepositoryWizard, WizardMessage};
