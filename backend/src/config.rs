//! Configuration management for the ZipLock backend daemon
//!
//! This module handles loading, validation, and management of configuration
//! settings for the backend service. Configuration can be loaded from files,
//! environment variables, and command-line arguments.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::error::{BackendResult, ConfigError};
use ziplock_shared::{PassphraseRequirements, ValidationPresets};

/// Main configuration structure for the backend daemon
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// IPC communication settings
    pub ipc: IpcConfig,

    /// Storage and archive settings
    pub storage: StorageConfig,

    /// Security and cryptography settings
    pub security: SecurityConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Performance and resource limits
    pub limits: LimitsConfig,
}

/// IPC (Inter-Process Communication) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    /// Unix domain socket path for IPC communication
    pub socket_path: PathBuf,

    /// Socket file permissions (octal)
    pub socket_permissions: u32,

    /// Maximum number of concurrent client connections
    pub max_connections: usize,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Request processing timeout in seconds
    pub request_timeout: u64,

    /// Enable IPC request/response logging (for debugging)
    pub log_requests: bool,
}

/// Storage and archive management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Default directory for storing archives
    pub default_archive_dir: PathBuf,

    /// Maximum archive file size in MB (0 = unlimited)
    pub max_archive_size_mb: u64,

    /// Number of backup copies to maintain
    pub backup_count: usize,

    /// Enable automatic backup creation
    pub auto_backup: bool,

    /// Backup directory path (defaults to archive_dir/backups)
    pub backup_dir: Option<PathBuf>,

    /// File lock timeout in seconds
    pub file_lock_timeout: u64,

    /// Temporary directory for archive operations
    pub temp_dir: Option<PathBuf>,

    /// Enable archive integrity checking
    pub verify_integrity: bool,

    /// Minimum password length for master password
    pub min_password_length: Option<usize>,

    /// 7z compression configuration
    pub compression: CompressionConfig,
}

/// 7z compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression level (0-9, where 9 is highest compression)
    pub level: u8,

    /// Enable solid compression (better compression but slower random access)
    pub solid: bool,

    /// Enable multi-threaded compression
    pub multi_threaded: bool,

    /// Dictionary size for LZMA2 compression in MB
    pub dictionary_size_mb: u32,

    /// Block size for solid compression in MB (0 = auto)
    pub block_size_mb: u32,
}

/// Security and cryptography configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Argon2 key derivation iteration count
    pub key_derivation_iterations: u32,

    /// Argon2 memory usage in KB
    pub key_derivation_memory_kb: u32,

    /// Argon2 parallelism factor
    pub key_derivation_parallelism: u32,

    /// Master key auto-lock timeout in seconds (0 = never)
    pub auto_lock_timeout: u64,

    /// Maximum number of failed authentication attempts
    pub max_auth_attempts: usize,

    /// Lockout duration after max attempts in seconds
    pub auth_lockout_duration: u64,

    /// Enable memory protection features
    pub memory_protection: bool,

    /// Clear clipboard after timeout in seconds (0 = disabled)
    pub clipboard_clear_timeout: u64,

    /// Master passphrase validation requirements
    pub passphrase_requirements: PassphraseRequirements,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Enable file logging
    pub file_logging: bool,

    /// Log file path
    pub log_file: Option<PathBuf>,

    /// Maximum log file size in MB
    pub max_log_size_mb: u64,

    /// Number of log files to rotate
    pub log_rotation_count: usize,

    /// Enable structured JSON logging
    pub json_format: bool,

    /// Enable logging of sensitive operations (for auditing)
    pub audit_logging: bool,
}

/// Performance and resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum memory usage in MB (0 = unlimited)
    pub max_memory_mb: u64,

    /// Maximum number of cached credentials
    pub max_cached_credentials: usize,

    /// Credential cache TTL in seconds
    pub cache_ttl: u64,

    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,

    /// Operation timeout in seconds
    pub operation_timeout: u64,

    /// Enable performance metrics collection
    pub enable_metrics: bool,
}

impl Default for IpcConfig {
    fn default() -> Self {
        let socket_path = dirs::runtime_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("ziplock")
            .join("backend.sock");

        Self {
            socket_path,
            socket_permissions: 0o600, // Owner read/write only
            max_connections: 10,
            connection_timeout: 30,
            request_timeout: 60,
            log_requests: false,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        let default_dir = dirs::data_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("ziplock");

        Self {
            default_archive_dir: default_dir,
            max_archive_size_mb: 100, // 100 MB limit
            backup_count: 5,
            auto_backup: true,
            backup_dir: None,
            file_lock_timeout: 30,
            temp_dir: None,
            verify_integrity: true,
            min_password_length: Some(12),
            compression: CompressionConfig::default(),
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 6,               // Balanced compression level
            solid: false,           // Disable solid compression by default for better random access
            multi_threaded: true,   // Enable multi-threading for better performance
            dictionary_size_mb: 64, // 64 MB dictionary size for good compression
            block_size_mb: 0,       // Auto block size
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            key_derivation_iterations: 100_000,
            key_derivation_memory_kb: 65_536, // 64 MB
            key_derivation_parallelism: 4,
            auto_lock_timeout: 900, // 15 minutes
            max_auth_attempts: 5,
            auth_lockout_duration: 300, // 5 minutes
            memory_protection: true,
            clipboard_clear_timeout: 30,
            passphrase_requirements: ValidationPresets::production(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_logging: false,
            log_file: None,
            max_log_size_mb: 10,
            log_rotation_count: 5,
            json_format: false,
            audit_logging: true,
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 0, // Unlimited
            max_cached_credentials: 1000,
            cache_ttl: 300, // 5 minutes
            max_concurrent_operations: 10,
            operation_timeout: 120, // 2 minutes
            enable_metrics: false,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> BackendResult<Self> {
        let path = path.as_ref();
        debug!("Loading configuration from: {:?}", path);

        if !path.exists() {
            return Err(ConfigError::NotFound {
                path: path.to_string_lossy().to_string(),
            }
            .into());
        }

        // Check file permissions for security
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path).context("Failed to read config file metadata")?;
            let permissions = metadata.permissions().mode() & 0o777;

            // Config file should not be world-readable
            if permissions & 0o044 != 0 {
                warn!(
                    "Configuration file {:?} has overly permissive permissions: {:o}",
                    path, permissions
                );
            }
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        info!("Configuration loaded successfully from: {:?}", path);
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> BackendResult<()> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = serde_yaml::to_string(self).context("Failed to serialize configuration")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        // Set secure permissions on the config file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600); // Owner read/write only
            fs::set_permissions(path, perms)?;
        }

        info!("Configuration saved to: {:?}", path);
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> BackendResult<()> {
        // Validate IPC settings
        if self.ipc.max_connections == 0 {
            return Err(ConfigError::Invalid {
                field: "ipc.max_connections".to_string(),
                reason: "must be greater than 0".to_string(),
            }
            .into());
        }

        if self.ipc.connection_timeout == 0 {
            return Err(ConfigError::Invalid {
                field: "ipc.connection_timeout".to_string(),
                reason: "must be greater than 0".to_string(),
            }
            .into());
        }

        // Validate storage settings
        if self.storage.backup_count > 100 {
            warn!(
                "Large backup_count ({}), this may consume significant disk space",
                self.storage.backup_count
            );
        }

        // Validate compression settings
        if self.storage.compression.level > 9 {
            return Err(ConfigError::Invalid {
                field: "storage.compression.level".to_string(),
                reason: "compression level must be between 0-9".to_string(),
            }
            .into());
        }

        if self.storage.compression.dictionary_size_mb > 1536 {
            return Err(ConfigError::Invalid {
                field: "storage.compression.dictionary_size_mb".to_string(),
                reason: "dictionary size must not exceed 1536 MB".to_string(),
            }
            .into());
        }

        if self.storage.compression.block_size_mb > 0 && self.storage.compression.block_size_mb < 1
        {
            return Err(ConfigError::Invalid {
                field: "storage.compression.block_size_mb".to_string(),
                reason: "block size must be 0 (auto) or at least 1 MB".to_string(),
            }
            .into());
        }

        // Validate security settings
        if self.security.key_derivation_iterations < 10_000 {
            return Err(ConfigError::Invalid {
                field: "security.key_derivation_iterations".to_string(),
                reason: "must be at least 10,000 for security".to_string(),
            }
            .into());
        }

        if self.security.passphrase_requirements.min_length < 6 {
            return Err(ConfigError::Invalid {
                field: "security.passphrase_requirements.min_length".to_string(),
                reason: "must be at least 6 characters".to_string(),
            }
            .into());
        }

        // Validate logging settings
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::Invalid {
                field: "logging.level".to_string(),
                reason: format!("must be one of: {}", valid_levels.join(", ")),
            }
            .into());
        }

        // Ensure required directories exist or can be created
        self.ensure_directories()?;

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Ensure required directories exist
    fn ensure_directories(&self) -> BackendResult<()> {
        // Create socket directory
        if let Some(parent) = self.ipc.socket_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create socket directory: {:?}", parent))?;
        }

        // Create archive directory
        fs::create_dir_all(&self.storage.default_archive_dir).with_context(|| {
            format!(
                "Failed to create archive directory: {:?}",
                self.storage.default_archive_dir
            )
        })?;

        // Create backup directory if specified
        if let Some(backup_dir) = &self.storage.backup_dir {
            fs::create_dir_all(backup_dir)
                .with_context(|| format!("Failed to create backup directory: {:?}", backup_dir))?;
        }

        // Create log directory if file logging is enabled
        if self.logging.file_logging {
            if let Some(log_file) = &self.logging.log_file {
                if let Some(parent) = log_file.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create log directory: {:?}", parent))?;
                }
            }
        }

        Ok(())
    }

    /// Get the effective backup directory
    pub fn backup_directory(&self) -> PathBuf {
        self.storage
            .backup_dir
            .clone()
            .unwrap_or_else(|| self.storage.default_archive_dir.join("backups"))
    }

    /// Get the effective temporary directory
    pub fn temp_directory(&self) -> PathBuf {
        self.storage
            .temp_dir
            .clone()
            .unwrap_or_else(|| std::env::temp_dir().join("ziplock"))
    }

    /// Convert timeout seconds to Duration
    pub fn auto_lock_duration(&self) -> Option<Duration> {
        if self.security.auto_lock_timeout == 0 {
            None
        } else {
            Some(Duration::from_secs(self.security.auto_lock_timeout))
        }
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.ipc.connection_timeout)
    }

    /// Get request timeout as Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.ipc.request_timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml_str = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml_str).unwrap();

        // Compare a few key fields
        assert_eq!(
            config.security.key_derivation_iterations,
            deserialized.security.key_derivation_iterations
        );
        assert_eq!(config.ipc.max_connections, deserialized.ipc.max_connections);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Test invalid max_connections
        config.ipc.max_connections = 0;
        assert!(config.validate().is_err());

        config.ipc.max_connections = 10;

        // Test invalid key derivation iterations
        config.security.key_derivation_iterations = 1000;
        assert!(config.validate().is_err());

        config.security.key_derivation_iterations = 50_000;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_file_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::default();

        // Test saving
        config.save(temp_file.path()).unwrap();

        // Test loading
        let loaded_config = Config::load(temp_file.path()).unwrap();

        // Compare key fields
        assert_eq!(
            config.security.key_derivation_iterations,
            loaded_config.security.key_derivation_iterations
        );
    }

    #[test]
    fn test_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();

        // Set paths to temp directory
        config.storage.default_archive_dir = temp_dir.path().join("archives");
        config.storage.backup_dir = Some(temp_dir.path().join("backups"));

        assert!(config.validate().is_ok());
        assert!(config.storage.default_archive_dir.exists());
        assert!(config.backup_directory().exists());
    }
}
