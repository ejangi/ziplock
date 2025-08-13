//! UI Module for ZipLock Linux Frontend
//!
//! This module contains all user interface components for the Linux frontend,
//! including the setup wizard, main interface, and various dialog components.

pub mod components;
pub mod theme;
pub mod views;

// Re-export commonly used UI components
pub use components::*;
pub use theme::{button_styles, create_ziplock_theme, progress_bar_styles, utils};
pub use views::{RepositoryWizard, WizardMessage};
