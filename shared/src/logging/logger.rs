//! Logging configuration and utilities for ZipLock
//!
//! This module provides centralized logging configuration for the ZipLock
//! shared library, with support for different log levels and output targets.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

/// Global flag to track if debug logging is enabled
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// One-time initialization flag for logging
static INIT: Once = Once::new();

/// Set whether debug logging is enabled
///
/// This affects the behavior of debug logging throughout the application.
/// When disabled, debug logs are filtered out for better performance.
///
/// # Arguments
/// * `enabled` - Whether to enable debug logging
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if debug logging is currently enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Logging configuration structure
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub target: LogTarget,
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            target: LogTarget::Stderr,
            format: LogFormat::Compact,
        }
    }
}

/// Log levels supported by the logging system
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// Convert log level to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }

    /// Parse log level from string
    pub fn from_str(s: &str) -> Option<LogLevel> {
        match s.to_uppercase().as_str() {
            "ERROR" => Some(LogLevel::Error),
            "WARN" | "WARNING" => Some(LogLevel::Warn),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" => Some(LogLevel::Debug),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

/// Log output targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTarget {
    /// Log to stderr
    Stderr,
    /// Log to stdout
    Stdout,
    /// Log to platform-specific system log
    System,
    /// Log to a custom writer (mobile platforms)
    Custom,
}

/// Log format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Compact format: [LEVEL] message
    Compact,
    /// Full format: timestamp [LEVEL] target: message
    Full,
    /// JSON format for structured logging
    Json,
}

/// Initialize logging with the given configuration
///
/// This should be called once at application startup. Subsequent calls
/// will be ignored.
///
/// # Arguments
/// * `config` - Logging configuration to use
pub fn init_logging(config: LoggingConfig) {
    INIT.call_once(|| {
        // Set debug flag based on log level
        set_debug_enabled(config.level >= LogLevel::Debug);

        // Initialize the actual logging backend
        // In a real implementation, this would configure env_logger, tracing, etc.
        init_logging_backend(config);
    });
}

/// Internal function to initialize the logging backend
fn init_logging_backend(config: LoggingConfig) {
    // This is a placeholder for actual logging initialization
    // In a real implementation, you would configure your logging library here

    #[cfg(feature = "env_logger")]
    {
        use std::io::Write;

        let mut builder = env_logger::Builder::new();

        // Set log level filter
        let filter_level = match config.level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        };

        builder.filter_level(filter_level);

        // Set format based on configuration
        match config.format {
            LogFormat::Compact => {
                builder
                    .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()));
            }
            LogFormat::Full => {
                builder.format(|buf, record| {
                    writeln!(
                        buf,
                        "{} [{}] {}: {}",
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.level(),
                        record.target(),
                        record.args()
                    )
                });
            }
            LogFormat::Json => {
                builder.format(|buf, record| {
                    writeln!(
                        buf,
                        r#"{{"timestamp":"{}","level":"{}","target":"{}","message":"{}"}}"#,
                        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                        record.level(),
                        record.target(),
                        record.args()
                    )
                });
            }
        }

        // Set output target
        match config.target {
            LogTarget::Stderr => {
                builder.target(env_logger::Target::Stderr);
            }
            LogTarget::Stdout => {
                builder.target(env_logger::Target::Stdout);
            }
            _ => {
                // Default to stderr for other targets
                builder.target(env_logger::Target::Stderr);
            }
        }

        builder.init();
    }

    #[cfg(not(feature = "env_logger"))]
    {
        // Fallback: just store the config for potential future use
        // In a minimal implementation, we might just print to stderr
        eprintln!("Logging initialized with level: {:?}", config.level);
    }
}

/// Macro for conditional debug logging
///
/// This only logs if debug logging is enabled, providing better performance
/// when debug logging is disabled.
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::logging::logger::is_debug_enabled() {
            log::debug!($($arg)*);
        }
    };
}

/// Macro for conditional trace logging
#[macro_export]
macro_rules! trace_log {
    ($($arg:tt)*) => {
        if $crate::logging::logger::is_debug_enabled() {
            log::trace!($($arg)*);
        }
    };
}

/// Helper function to sanitize log messages by removing sensitive data
pub fn sanitize_log_message(message: &str) -> String {
    // Replace potential passwords, tokens, and other sensitive data patterns
    let sensitive_patterns = [
        (r"password[=:\s]+[^\s]+", "password=***"),
        (r"token[=:\s]+[^\s]+", "token=***"),
        (r"key[=:\s]+[^\s]+", "key=***"),
        (r"secret[=:\s]+[^\s]+", "secret=***"),
        (r"auth[=:\s]+[^\s]+", "auth=***"),
    ];

    let mut sanitized = message.to_string();

    for (pattern, replacement) in &sensitive_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, *replacement).to_string();
        }
    }

    sanitized
}

/// Get current log level as string for display
pub fn current_log_level() -> String {
    if is_debug_enabled() {
        "DEBUG".to_string()
    } else {
        "INFO".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    #[test]
    fn test_log_level_string_conversion() {
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");

        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("TRACE"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_debug_enabled_flag() {
        // Test initial state
        let initial_state = is_debug_enabled();

        // Test setting and getting
        set_debug_enabled(true);
        assert!(is_debug_enabled());

        set_debug_enabled(false);
        assert!(!is_debug_enabled());

        // Restore initial state
        set_debug_enabled(initial_state);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.target, LogTarget::Stderr);
        assert_eq!(config.format, LogFormat::Compact);
    }

    #[test]
    fn test_sanitize_log_message() {
        let message = "User logged in with password=secret123 and token=abc123def";
        let sanitized = sanitize_log_message(message);

        assert!(!sanitized.contains("secret123"));
        assert!(!sanitized.contains("abc123def"));
        assert!(sanitized.contains("password=***"));
        assert!(sanitized.contains("token=***"));
    }

    #[test]
    fn test_sanitize_various_patterns() {
        let test_cases = [
            ("password: mysecret", "password=***"),
            ("auth=bearer_token_here", "auth=***"),
            ("secret key=verysecret", "secret=***"),
            ("normal message", "normal message"),
        ];

        for (input, expected_partial) in &test_cases {
            let sanitized = sanitize_log_message(input);
            if expected_partial.contains("***") {
                assert!(
                    sanitized.contains("***"),
                    "Expected sanitization in '{}', got '{}'",
                    input,
                    sanitized
                );
            } else {
                assert_eq!(sanitized, *expected_partial);
            }
        }
    }

    #[test]
    fn test_current_log_level() {
        set_debug_enabled(true);
        assert_eq!(current_log_level(), "DEBUG");

        set_debug_enabled(false);
        assert_eq!(current_log_level(), "INFO");
    }
}
