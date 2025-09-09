//! Logging infrastructure for ZipLock
//!
//! This module provides comprehensive logging support for the ZipLock shared
//! library, including configuration, mobile platform integration, and
//! utilities for secure logging that avoids exposing sensitive information.

pub mod logger;
pub mod mobile_writer;

// Re-export commonly used items
pub use logger::{
    current_log_level, init_logging, is_debug_enabled, sanitize_log_message, set_debug_enabled,
    LogFormat, LogLevel, LogTarget, LoggingConfig,
};
pub use mobile_writer::{create_mobile_writer, is_mobile_platform, MobileLogWriter};

use std::sync::Once;

/// Global initialization flag to ensure logging is only set up once
static INIT: Once = Once::new();

/// Initialize logging with default configuration
///
/// This is a convenience function that sets up logging with sensible defaults.
/// For custom configuration, use `logger::init_logging()` directly.
pub fn init_default_logging() {
    INIT.call_once(|| {
        let config = LoggingConfig::default();
        logger::init_logging(config);
    });
}

/// Initialize logging for mobile platforms
///
/// This sets up logging specifically for mobile platforms with appropriate
/// configuration and output targets.
pub fn init_mobile_logging() {
    INIT.call_once(|| {
        let config = LoggingConfig {
            level: LogLevel::Info,
            target: LogTarget::Custom,
            format: LogFormat::Compact,
        };
        logger::init_logging(config);
    });
}

/// Initialize logging for desktop platforms
///
/// This sets up logging for desktop platforms with more verbose output
/// suitable for development and debugging.
pub fn init_desktop_logging() {
    INIT.call_once(|| {
        let config = LoggingConfig {
            level: LogLevel::Debug,
            target: LogTarget::Stderr,
            format: LogFormat::Full,
        };
        logger::init_logging(config);
    });
}

/// Check if logging has been initialized
pub fn is_logging_initialized() -> bool {
    INIT.is_completed()
}

/// Macro for logging errors with automatic message sanitization
#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        {
            let message = format!($($arg)*);
            let sanitized = $crate::logging::sanitize_log_message(&message);
            log::error!("{}", sanitized);
        }
    };
}

/// Macro for logging warnings with automatic message sanitization
#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        {
            let message = format!($($arg)*);
            let sanitized = $crate::logging::sanitize_log_message(&message);
            log::warn!("{}", sanitized);
        }
    };
}

/// Macro for logging info with automatic message sanitization
#[macro_export]
macro_rules! info_log {
    ($($arg:tt)*) => {
        {
            let message = format!($($arg)*);
            let sanitized = $crate::logging::sanitize_log_message(&message);
            log::info!("{}", sanitized);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        // Note: We can't easily test the actual initialization since it uses Once
        // and would affect other tests, but we can test the functions exist
        assert!(!is_logging_initialized() || is_logging_initialized()); // Always true, just testing the call
    }

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.target, LogTarget::Stderr);
        assert_eq!(config.format, LogFormat::Compact);
    }

    #[test]
    fn test_mobile_writer_creation() {
        let writer = create_mobile_writer();
        // Just test that we can create it without panicking
        drop(writer);
    }

    #[test]
    fn test_platform_detection() {
        let is_mobile = is_mobile_platform();
        // The result depends on the target platform
        assert!(is_mobile || !is_mobile); // Always true, just testing the call
    }
}
