//! UI Module for ZipLock Linux App
//!
//! This module contains all user interface components for the Linux app,
//! including the setup wizard, main interface, and various dialog components.

pub mod components;
pub mod theme;
pub mod views;

// Re-export only used UI components
pub use theme::{create_ziplock_theme, utils};
// Views are imported directly from their modules, so no re-exports needed
