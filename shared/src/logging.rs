//! Logging infrastructure for ZipLock shared library
//!
//! This module provides configurable logging support for the ZipLock shared library,
//! with special consideration for mobile platforms and FFI integration.

use std::sync::{Arc, Mutex, OnceLock};
use tracing::Level;

/// Global logging configuration
static LOGGING_CONFIG: OnceLock<Arc<Mutex<LoggingConfig>>> = OnceLock::new();

/// Logging configuration structure
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Whether debug logging is enabled
    pub debug_enabled: bool,
    /// Log level filter
    pub level: Level,
    /// Whether to include timestamps
    pub include_timestamps: bool,
    /// Whether to include thread information
    pub include_thread_info: bool,
    /// Whether to include span information
    pub include_spans: bool,
    /// Custom log target prefix
    pub target_prefix: Option<String>,
    /// Maximum log line length (for mobile platforms)
    pub max_line_length: Option<usize>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug_enabled: false,
            level: Level::INFO,
            include_timestamps: true,
            include_thread_info: false,
            include_spans: false,
            target_prefix: Some("ZipLock".to_string()),
            max_line_length: Some(1024), // Android logcat has limits
        }
    }
}

/// Initialize the logging system
pub fn init_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = LoggingConfig::default();

    // Store the configuration globally
    LOGGING_CONFIG
        .set(Arc::new(Mutex::new(config.clone())))
        .map_err(|_| "Logging already initialized")?;

    // Simple initialization for FFI compatibility
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Try to initialize a simple subscriber
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    println!("ZipLock logging system initialized");
    Ok(())
}

/// Update logging configuration
pub fn configure_logging(
    config: LoggingConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Update the stored configuration
    if let Some(global_config) = LOGGING_CONFIG.get() {
        if let Ok(mut stored_config) = global_config.lock() {
            *stored_config = config.clone();
        }
    }

    // Update the subscriber with new configuration
    setup_subscriber(&config)?;

    if config.debug_enabled {
        tracing::info!("Logging configuration updated: debug_enabled=true");
    } else {
        println!("Logging configuration updated: debug_enabled=false");
    }
    Ok(())
}

/// Enable or disable debug logging
pub fn set_debug_enabled(enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(global_config) = LOGGING_CONFIG.get() {
        let config = if let Ok(mut stored_config) = global_config.lock() {
            stored_config.debug_enabled = enabled;
            stored_config.level = if enabled { Level::DEBUG } else { Level::INFO };
            stored_config.clone()
        } else {
            return Err("Failed to acquire logging config lock".into());
        };

        setup_subscriber(&config)?;

        if enabled {
            tracing::debug!("Debug logging enabled");
            println!("Debug logging enabled successfully");
        } else {
            println!("Debug logging disabled");
        }
    }

    Ok(())
}

/// Check if debug logging is enabled
pub fn is_debug_enabled() -> bool {
    LOGGING_CONFIG
        .get()
        .and_then(|config| config.lock().ok())
        .map(|config| config.debug_enabled)
        .unwrap_or(false)
}

/// Get current logging configuration
pub fn get_config() -> LoggingConfig {
    LOGGING_CONFIG
        .get()
        .and_then(|config| config.lock().ok())
        .map(|config| config.clone())
        .unwrap_or_default()
}

/// Set up the tracing subscriber based on configuration
fn setup_subscriber(
    config: &LoggingConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simple approach for FFI compatibility
    let log_level = if config.debug_enabled {
        "debug"
    } else {
        "info"
    };

    // Set environment variable for log level
    std::env::set_var("RUST_LOG", format!("ziplock_shared={}", log_level));

    // Try to set up a new subscriber
    let subscriber_result = tracing_subscriber::fmt()
        .with_max_level(if config.debug_enabled {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .with_target(true)
        .with_thread_ids(config.include_thread_info)
        .with_thread_names(config.include_thread_info)
        .try_init();

    match subscriber_result {
        Ok(_) => {
            println!("Tracing subscriber configured successfully");
        }
        Err(_) => {
            // Subscriber already exists, just update the environment variable
            println!(
                "Tracing subscriber already configured, updated log level to {}",
                log_level
            );
        }
    }

    Ok(())
}

/// Custom writer for mobile platforms that respects line length limits
pub struct MobileLogWriter {
    max_length: usize,
    prefix: String,
}

impl MobileLogWriter {
    pub fn new(max_length: usize, prefix: String) -> Self {
        Self { max_length, prefix }
    }

    fn split_message(&self, message: &str) -> Vec<String> {
        if message.len() <= self.max_length {
            return vec![message.to_string()];
        }

        let mut parts = Vec::new();
        let mut remaining = message;

        while remaining.len() > self.max_length {
            let split_pos = remaining[..self.max_length]
                .rfind(' ')
                .unwrap_or(self.max_length);

            parts.push(remaining[..split_pos].to_string());
            remaining = remaining[split_pos..].trim_start();
        }

        if !remaining.is_empty() {
            parts.push(remaining.to_string());
        }

        parts
    }
}

impl std::io::Write for MobileLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = String::from_utf8_lossy(buf);
        let parts = self.split_message(&message);

        for (i, part) in parts.iter().enumerate() {
            let formatted = if parts.len() > 1 {
                format!("{} ({}/{}) {}", self.prefix, i + 1, parts.len(), part)
            } else {
                format!("{} {}", self.prefix, part)
            };

            // For mobile platforms, we'll use println! which should route to platform logs
            println!("{}", formatted);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Logging macros for consistent usage throughout the library
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if $crate::logging::is_debug_enabled() {
            tracing::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

/// Performance logging utilities
pub mod perf {
    use std::time::Instant;
    use tracing::Span;

    /// Simple performance timer
    pub struct PerfTimer {
        name: String,
        start: Instant,
        _span: Span,
    }

    impl PerfTimer {
        pub fn new(name: &str) -> Self {
            let span = tracing::debug_span!("perf_timer", name = name);
            Self {
                name: name.to_string(),
                start: Instant::now(),
                _span: span,
            }
        }
    }

    impl Drop for PerfTimer {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed();
            crate::log_debug!(
                "Performance [{}]: {:.2}ms",
                self.name,
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }

    /// Macro for easy performance timing
    #[macro_export]
    macro_rules! perf_timer {
        ($name:expr) => {
            let _timer = $crate::logging::perf::PerfTimer::new($name);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(!config.debug_enabled);
        assert_eq!(config.level, Level::INFO);
        assert!(config.include_timestamps);
        assert!(!config.include_thread_info);
        assert_eq!(config.target_prefix, Some("ZipLock".to_string()));
        assert_eq!(config.max_line_length, Some(1024));
    }

    #[test]
    fn test_mobile_log_writer_split() {
        let writer = MobileLogWriter::new(10, "TEST".to_string());
        let parts = writer.split_message("This is a very long message that should be split");

        assert!(parts.len() > 1);
        for part in &parts {
            assert!(part.len() <= 10);
        }
    }

    #[test]
    fn test_mobile_log_writer_short_message() {
        let writer = MobileLogWriter::new(100, "TEST".to_string());
        let parts = writer.split_message("Short message");

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "Short message");
    }

    #[test]
    fn test_debug_enabled_state() {
        // This test might interfere with other tests since it uses global state
        // In a real scenario, we might want to use a different approach for testing
        let initial_state = is_debug_enabled();

        // The default should be false
        assert!(!initial_state);
    }
}
