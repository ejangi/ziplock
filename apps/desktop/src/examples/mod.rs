//! Examples for Linux Adaptive Integration
//!
//! This module contains comprehensive examples showing how to properly
//! integrate the adaptive hybrid architecture in Linux desktop applications.

pub mod adaptive_integration;

pub use adaptive_integration::{example_usage, AdaptiveLinuxIntegration};

/// Run all examples
pub async fn run_all_examples() -> Result<(), String> {
    println!("Running Linux adaptive integration examples...");

    // Run the main adaptive integration example
    example_usage().await?;

    println!("All examples completed successfully!");
    Ok(())
}

/// Check system requirements for examples
pub fn check_example_requirements() -> Result<(), String> {
    crate::platform::check_system_dependencies()?;
    println!("✅ All system requirements met for examples");
    Ok(())
}
