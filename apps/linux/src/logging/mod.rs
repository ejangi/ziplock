//! Comprehensive logging configuration for ZipLock Linux application
//!
//! This module provides structured logging with file rotation, proper formatting,
//! and deployment-ready configuration. It supports both development and production
//! environments with appropriate log levels and output destinations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// YAML configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlLoggingConfig {
    pub console: ConsoleConfig,
    pub file: FileConfig,
    pub rotation: RotationConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    pub enabled: bool,
    pub level: String,
    pub timestamps: bool,
    pub colors: String, // auto, always, never
    pub format: String, // compact, pretty, json
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub enabled: bool,
    pub level: String,
    pub directory: String,
    pub filename: String,
    pub timestamps: bool,
    pub format: String, // compact, detailed, json
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    pub enabled: bool,
    pub max_file_size: String,
    pub max_files: usize,
    pub compress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub thread_ids: bool,
    pub source_location: bool,
    pub performance_tracking: bool,
}

/// Log rotation configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogRotationConfig {
    /// Maximum size of a single log file in bytes (default: 10MB)
    pub max_file_size: u64,
    /// Maximum number of archived log files to keep (default: 5)
    pub max_files: usize,
    /// Whether to compress archived log files (default: true)
    pub compress: bool,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            compress: true,
        }
    }
}

/// Logging configuration for the application
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Directory where log files will be stored
    pub log_dir: PathBuf,
    /// Base name for log files (default: "ziplock")
    pub log_file_name: String,
    /// Log level for console output
    pub console_level: String,
    /// Log level for file output
    pub file_level: String,
    /// Whether to enable console logging
    pub enable_console: bool,
    /// Whether to enable file logging
    pub enable_file: bool,
    /// Log rotation configuration
    pub rotation: LogRotationConfig,

    /// Whether to include thread IDs in logs
    pub include_thread_ids: bool,
    /// Whether to include source code locations in logs
    pub include_source_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: get_default_log_dir(),
            log_file_name: "ziplock".to_string(),
            console_level: "INFO".to_string(),
            file_level: "DEBUG".to_string(),
            enable_console: true,
            enable_file: true,
            rotation: LogRotationConfig::default(),
            include_thread_ids: false,
            include_source_location: false,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration with custom log directory
    #[allow(dead_code)]
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir,
            ..Default::default()
        }
    }

    /// Set console log level
    #[allow(dead_code)]
    pub fn console_level(mut self, level: &str) -> Self {
        self.console_level = level.to_string();
        self
    }

    /// Set file log level
    #[allow(dead_code)]
    pub fn file_level(mut self, level: &str) -> Self {
        self.file_level = level.to_string();
        self
    }

    /// Enable or disable console logging
    #[allow(dead_code)]
    pub fn console_enabled(mut self, enabled: bool) -> Self {
        self.enable_console = enabled;
        self
    }

    /// Enable or disable file logging
    #[allow(dead_code)]
    pub fn file_enabled(mut self, enabled: bool) -> Self {
        self.enable_file = enabled;
        self
    }

    /// Configure log rotation
    #[allow(dead_code)]
    pub fn rotation(mut self, rotation: LogRotationConfig) -> Self {
        self.rotation = rotation;
        self
    }

    /// Enable source code location in logs (useful for debugging)
    #[allow(dead_code)]
    pub fn with_source_location(mut self) -> Self {
        self.include_source_location = true;
        self
    }

    /// Enable thread IDs in logs (useful for debugging async code)
    #[allow(dead_code)]
    pub fn with_thread_ids(mut self) -> Self {
        self.include_thread_ids = true;
        self
    }

    /// Create development configuration with more verbose logging
    pub fn development() -> Self {
        Self {
            console_level: "DEBUG".to_string(),
            file_level: "TRACE".to_string(),

            include_thread_ids: true,
            include_source_location: true,
            ..Default::default()
        }
    }

    /// Create production configuration with optimized logging
    pub fn production() -> Self {
        Self {
            console_level: "WARN".to_string(),
            file_level: "INFO".to_string(),
            include_thread_ids: false,
            include_source_location: false,
            rotation: LogRotationConfig {
                max_file_size: 50 * 1024 * 1024, // 50MB for production
                max_files: 10,
                compress: true,
            },
            ..Default::default()
        }
    }

    /// Get the full path to the current log file
    pub fn current_log_file(&self) -> PathBuf {
        self.log_dir.join(format!("{}.log", self.log_file_name))
    }
}

/// Initialize logging with the given configuration
pub fn initialize_logging(config: LoggingConfig) -> Result<()> {
    // Ensure log directory exists
    if config.enable_file {
        fs::create_dir_all(&config.log_dir)
            .with_context(|| format!("Failed to create log directory: {:?}", config.log_dir))?;

        info!("Log directory created/verified: {:?}", config.log_dir);
    }

    // Build the subscriber with layers
    let mut layers = Vec::new();

    // Console layer
    if config.enable_console {
        let console_filter =
            EnvFilter::try_new(&config.console_level).unwrap_or_else(|_| EnvFilter::new("INFO"));

        let console_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(config.include_thread_ids)
            .with_file(config.include_source_location)
            .with_line_number(config.include_source_location)
            .with_ansi(atty::is(atty::Stream::Stdout))
            .with_writer(std::io::stdout)
            .with_filter(console_filter);

        layers.push(console_layer.boxed());
    }

    // File layer with rotation
    if config.enable_file {
        let file_filter =
            EnvFilter::try_new(&config.file_level).unwrap_or_else(|_| EnvFilter::new("DEBUG"));

        let log_file = config.current_log_file();

        // Use tracing-appender for file rotation
        let file_appender =
            tracing_appender::rolling::daily(&config.log_dir, &config.log_file_name);

        let file_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(config.include_thread_ids)
            .with_file(config.include_source_location)
            .with_line_number(config.include_source_location)
            .with_ansi(false)
            .with_writer(file_appender)
            .with_filter(file_filter);

        layers.push(file_layer.boxed());

        info!("File logging enabled: {:?}", log_file);
    }

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(layers)
        .try_init()
        .context("Failed to initialize tracing subscriber")?;

    // Log startup information
    info!("ZipLock logging initialized");
    info!(
        "Console logging: {} (level: {})",
        config.enable_console, config.console_level
    );
    info!(
        "File logging: {} (level: {}) at {:?}",
        config.enable_file, config.file_level, config.log_dir
    );

    Ok(())
}

/// Load logging configuration from YAML file
pub fn load_config_from_file(config_path: &Path, environment: &str) -> Result<LoggingConfig> {
    let config_content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

    let yaml_configs: HashMap<String, YamlLoggingConfig> = serde_yaml::from_str(&config_content)
        .with_context(|| format!("Failed to parse YAML config file: {:?}", config_path))?;

    let yaml_config = yaml_configs
        .get(environment)
        .or_else(|| yaml_configs.get("default"))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No configuration found for environment '{}' and no default config",
                environment
            )
        })?;

    yaml_to_logging_config(yaml_config)
}

/// Convert YAML config to LoggingConfig
fn yaml_to_logging_config(yaml: &YamlLoggingConfig) -> Result<LoggingConfig> {
    let log_dir = expand_directory_path(&yaml.file.directory)?;

    let max_file_size = parse_file_size(&yaml.rotation.max_file_size)?;

    Ok(LoggingConfig {
        log_dir,
        log_file_name: yaml.file.filename.clone(),
        console_level: yaml.console.level.clone(),
        file_level: yaml.file.level.clone(),
        enable_console: yaml.console.enabled,
        enable_file: yaml.file.enabled,
        rotation: LogRotationConfig {
            max_file_size,
            max_files: yaml.rotation.max_files,
            compress: yaml.rotation.compress,
        },
        include_thread_ids: yaml.features.thread_ids,
        include_source_location: yaml.features.source_location,
    })
}

/// Expand directory path with environment variables and home directory
fn expand_directory_path(path_str: &str) -> Result<PathBuf> {
    let expanded = if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            home.join(path_str.strip_prefix("~/").unwrap())
        } else {
            PathBuf::from(path_str)
        }
    } else if path_str.starts_with("./") {
        std::env::current_dir()?.join(path_str.strip_prefix("./").unwrap())
    } else {
        PathBuf::from(path_str)
    };

    Ok(expanded)
}

/// Parse file size string (e.g., "10MB", "1GB") to bytes
fn parse_file_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.to_uppercase();

    if let Some(num_str) = size_str.strip_suffix("GB") {
        let num: f64 = num_str.parse().context("Invalid number in file size")?;
        Ok((num * 1_073_741_824.0) as u64)
    } else if let Some(num_str) = size_str.strip_suffix("MB") {
        let num: f64 = num_str.parse().context("Invalid number in file size")?;
        Ok((num * 1_048_576.0) as u64)
    } else if let Some(num_str) = size_str.strip_suffix("KB") {
        let num: f64 = num_str.parse().context("Invalid number in file size")?;
        Ok((num * 1024.0) as u64)
    } else if let Some(num_str) = size_str.strip_suffix("B") {
        let num: u64 = num_str.parse().context("Invalid number in file size")?;
        Ok(num)
    } else {
        // Assume bytes if no suffix
        let num: u64 = size_str.parse().context("Invalid file size format")?;
        Ok(num)
    }
}

/// Initialize logging with default configuration
pub fn initialize_default_logging() -> Result<()> {
    let environment = get_environment();

    // Try to load from config file first
    let config_path = get_config_file_path();

    let config = if config_path.exists() {
        match load_config_from_file(&config_path, &environment) {
            Ok(config) => {
                info!("Loaded logging configuration from: {:?}", config_path);
                config
            }
            Err(e) => {
                warn!("Failed to load logging config file: {}. Using defaults.", e);
                get_default_config_for_environment(&environment)
            }
        }
    } else {
        get_default_config_for_environment(&environment)
    };

    initialize_logging(config)
}

/// Get the current environment name
pub fn get_environment() -> String {
    std::env::var("ZIPLOCK_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .unwrap_or_else(|_| {
            if is_development_environment() {
                "development".to_string()
            } else {
                "production".to_string()
            }
        })
}

/// Get the path to the configuration file
pub fn get_config_file_path() -> PathBuf {
    // Check for explicit config file path
    if let Ok(config_path) = std::env::var("ZIPLOCK_LOG_CONFIG") {
        return PathBuf::from(config_path);
    }

    // Try various standard locations
    let possible_paths = vec![
        PathBuf::from("./config/logging.yaml"),
        PathBuf::from("./logging.yaml"),
    ];

    if let Some(config_dir) = dirs::config_dir() {
        possible_paths
            .into_iter()
            .chain(vec![
                config_dir.join("ziplock/logging.yaml"),
                config_dir.join("ziplock/config/logging.yaml"),
            ])
            .find(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("./config/logging.yaml"))
    } else {
        possible_paths
            .into_iter()
            .find(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("./config/logging.yaml"))
    }
}

/// Get default configuration for environment
fn get_default_config_for_environment(environment: &str) -> LoggingConfig {
    match environment {
        "development" | "dev" => LoggingConfig::development(),
        "production" | "prod" => LoggingConfig::production(),
        "testing" | "test" => LoggingConfig {
            console_level: "DEBUG".to_string(),
            file_level: "DEBUG".to_string(),
            log_dir: PathBuf::from("./target/test-logs"),
            include_thread_ids: true,
            include_source_location: true,
            ..Default::default()
        },
        "systemd" => LoggingConfig {
            enable_console: false,
            console_level: "OFF".to_string(),
            file_level: "INFO".to_string(),
            log_dir: PathBuf::from("/var/log/ziplock"),
            rotation: LogRotationConfig {
                max_file_size: 100 * 1024 * 1024, // 100MB
                max_files: 20,
                compress: true,
            },
            ..Default::default()
        },
        "docker" => LoggingConfig {
            enable_file: false,
            console_level: "INFO".to_string(),
            ..Default::default()
        },
        _ => LoggingConfig::default(),
    }
}

/// Initialize logging for systemd service deployment
#[allow(dead_code)]
pub fn initialize_systemd_logging() -> Result<()> {
    let config = LoggingConfig {
        enable_console: false, // systemd will capture stdout/stderr
        enable_file: true,
        console_level: "OFF".to_string(),
        file_level: "INFO".to_string(),
        log_dir: PathBuf::from("/var/log/ziplock"),
        rotation: LogRotationConfig {
            max_file_size: 100 * 1024 * 1024, // 100MB for systemd services
            max_files: 20,
            compress: true,
        },
        ..Default::default()
    };

    initialize_logging(config)
}

/// Get the default log directory based on the environment
pub fn get_default_log_dir() -> PathBuf {
    // Try to use XDG_CACHE_HOME or fallback to ~/.cache
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("ziplock").join("logs")
    } else {
        // Fallback to /tmp for systems without home directory
        PathBuf::from("/tmp/ziplock/logs")
    }
}

/// Check if we're running in a development environment
pub fn is_development_environment() -> bool {
    // Check various indicators of development environment
    std::env::var("ZIPLOCK_ENV").unwrap_or_default() == "development"
        || std::env::var("RUST_ENV").unwrap_or_default() == "development"
        || cfg!(debug_assertions)
        || std::env::var("CARGO_PKG_NAME").is_ok()
}

/// Clean up old log files based on rotation configuration
#[allow(dead_code)]
pub fn cleanup_old_logs(config: &LoggingConfig) -> Result<()> {
    if !config.enable_file {
        return Ok(());
    }

    let log_dir = &config.log_dir;
    if !log_dir.exists() {
        return Ok(());
    }

    let mut log_files = Vec::new();

    // Find all log files with our pattern
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(&config.log_file_name)
                && (name.ends_with(".log") || name.contains(".log."))
            {
                if let Ok(metadata) = entry.metadata() {
                    log_files.push((path, metadata.modified().unwrap_or(std::time::UNIX_EPOCH)));
                }
            }
        }
    }

    // Sort by modification time, newest first
    log_files.sort_by_key(|(_, time)| std::cmp::Reverse(*time));

    // Remove files beyond the limit
    if log_files.len() > config.rotation.max_files {
        for (path, _) in log_files.into_iter().skip(config.rotation.max_files) {
            match fs::remove_file(&path) {
                Ok(()) => info!("Removed old log file: {:?}", path),
                Err(e) => warn!("Failed to remove old log file {:?}: {}", path, e),
            }
        }
    }

    Ok(())
}

/// Setup log rotation as a background task
#[allow(dead_code)]
pub async fn setup_log_rotation_task(config: LoggingConfig) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Check every hour

    loop {
        interval.tick().await;

        if let Err(e) = cleanup_old_logs(&config) {
            warn!("Log cleanup failed: {}", e);
        }
    }
}

/// Create a systemd service file for log management
#[allow(dead_code)]
pub fn generate_systemd_service_config() -> String {
    r#"[Unit]
Description=ZipLock Password Manager
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/ziplock
Restart=always
RestartSec=5
User=ziplock
Group=ziplock

# Logging configuration
StandardOutput=journal
StandardError=journal
SyslogIdentifier=ziplock

# Log rotation is handled by systemd
LogRateLimitIntervalSec=0
LogRateLimitBurst=0

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/ziplock

# Create log directory
ExecStartPre=/bin/mkdir -p /var/log/ziplock
ExecStartPre=/bin/chown ziplock:ziplock /var/log/ziplock

[Install]
WantedBy=multi-user.target
"#
    .to_string()
}

/// Create logrotate configuration for manual log rotation
#[allow(dead_code)]
pub fn generate_logrotate_config() -> String {
    format!(
        r#"/var/log/ziplock/*.log {{
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 ziplock ziplock
    postrotate
        systemctl reload ziplock.service > /dev/null 2>&1 || true
    endscript
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert!(config.enable_console);
        assert!(config.enable_file);
        assert_eq!(config.console_level, "INFO");
        assert_eq!(config.file_level, "DEBUG");
    }

    #[test]
    fn test_development_config() {
        let config = LoggingConfig::development();
        assert_eq!(config.console_level, "DEBUG");
        assert_eq!(config.file_level, "TRACE");
        assert!(config.include_thread_ids);
        assert!(config.include_source_location);
    }

    #[test]
    fn test_production_config() {
        let config = LoggingConfig::production();
        assert_eq!(config.console_level, "WARN");
        assert_eq!(config.file_level, "INFO");
        assert!(!config.include_thread_ids);
        assert!(!config.include_source_location);
        assert_eq!(config.rotation.max_file_size, 50 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_log_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");

        let config = LoggingConfig::new(log_dir.clone());

        // Directory shouldn't exist initially
        assert!(!log_dir.exists());

        // Initialize logging should create the directory
        initialize_logging(config).unwrap();

        // Directory should now exist
        assert!(log_dir.exists());
    }

    #[test]
    fn test_log_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = LoggingConfig::new(temp_dir.path().to_path_buf());

        let expected_path = temp_dir.path().join("ziplock.log");
        assert_eq!(config.current_log_file(), expected_path);
    }

    #[test]
    fn test_file_size_parsing() {
        assert_eq!(parse_file_size("10MB").unwrap(), 10 * 1024 * 1024);
        assert_eq!(parse_file_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_file_size("500KB").unwrap(), 500 * 1024);
        assert_eq!(parse_file_size("1024B").unwrap(), 1024);
        assert_eq!(parse_file_size("1024").unwrap(), 1024);
    }

    #[test]
    fn test_directory_expansion() {
        let result = expand_directory_path("./test-logs").unwrap();
        assert!(result.to_string_lossy().ends_with("test-logs"));

        let result = expand_directory_path("/var/log/ziplock").unwrap();
        assert_eq!(result, PathBuf::from("/var/log/ziplock"));
    }

    #[test]
    fn test_environment_detection() {
        // Test default environment detection
        let env = get_environment();
        assert!(env == "development" || env == "production");
    }

    #[test]
    fn test_yaml_config_structure() {
        let yaml_content = r#"
default:
  console:
    enabled: true
    level: "INFO"
    timestamps: true
    colors: "auto"
    format: "compact"
  file:
    enabled: true
    level: "DEBUG"
    directory: "./logs"
    filename: "test.log"
    timestamps: true
    format: "detailed"
  rotation:
    enabled: true
    max_file_size: "10MB"
    max_files: 5
    compress: true
  features:
    thread_ids: false
    source_location: false
    performance_tracking: false
"#;

        let configs: HashMap<String, YamlLoggingConfig> =
            serde_yaml::from_str(yaml_content).unwrap();
        assert!(configs.contains_key("default"));

        let config = &configs["default"];
        assert_eq!(config.console.level, "INFO");
        assert_eq!(config.file.filename, "test.log");
        assert_eq!(config.rotation.max_files, 5);
    }

    #[test]
    fn test_systemd_service_generation() {
        let service_config = generate_systemd_service_config();
        assert!(service_config.contains("Description=ZipLock Password Manager"));
        assert!(service_config.contains("ExecStart=/usr/bin/ziplock"));
        assert!(service_config.contains("StandardOutput=journal"));
    }

    #[test]
    fn test_logrotate_config_generation() {
        let logrotate_config = generate_logrotate_config();
        assert!(logrotate_config.contains("/var/log/ziplock/*.log"));
        assert!(logrotate_config.contains("daily"));
        assert!(logrotate_config.contains("rotate 30"));
    }
}
