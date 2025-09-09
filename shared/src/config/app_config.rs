//! Application configuration for ZipLock unified architecture
//!
//! This module provides application configuration structures that are primarily
//! used by desktop applications. Mobile applications typically handle configuration
//! through their native frameworks and use only subset of these structures.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration structure
///
/// Contains all user preferences and settings for desktop applications.
/// Mobile applications may use individual components as needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// User interface configuration
    pub ui: UiConfig,

    /// Security-related settings
    pub security: SecurityConfig,

    /// Application behavior settings
    pub behavior: AppBehaviorConfig,

    /// Repository management settings
    pub repository_settings: RepositoryManagementConfig,

    /// List of recent repositories
    pub repositories: Vec<RepositoryInfo>,
}

/// User interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// UI theme preference ("system", "light", "dark")
    pub theme: String,

    /// Language/locale setting (ISO 639-1 code)
    pub language: String,

    /// Auto-lock timeout in seconds (0 = disabled)
    pub auto_lock_timeout: u64,

    /// Window width (desktop only)
    pub window_width: Option<u32>,

    /// Window height (desktop only)
    pub window_height: Option<u32>,

    /// Font size multiplier (desktop only)
    pub font_scale: Option<f32>,

    /// Whether to show password strength indicators
    pub show_password_strength: bool,

    /// Whether to start minimized to tray (desktop only)
    pub start_minimized: bool,

    /// Whether to show wizard on startup
    pub show_wizard_on_startup: bool,

    /// Whether to minimize to system tray (desktop only)
    pub minimize_to_tray: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Master password timeout in seconds
    pub password_timeout: u64,

    /// Clipboard clear timeout in seconds
    pub clipboard_timeout: u64,

    /// Whether biometric authentication is enabled
    pub biometric_enabled: bool,

    /// Whether to lock on system suspend (desktop only)
    pub lock_on_suspend: bool,

    /// Whether to clear clipboard on lock
    pub clear_clipboard_on_lock: bool,

    /// Maximum number of failed authentication attempts
    pub max_auth_attempts: u32,

    /// Lockout duration after max attempts (seconds)
    pub lockout_duration: u64,
}

/// Application behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppBehaviorConfig {
    /// Whether to automatically check for updates
    pub auto_check_updates: bool,

    /// Whether to enable automatic backups
    pub enable_backup: bool,

    /// Number of backup copies to keep
    pub backup_count: u32,
}

/// Repository management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RepositoryManagementConfig {
    /// Default directory for repositories
    pub default_directory: Option<PathBuf>,

    /// Whether to auto-detect repositories
    pub auto_detect: bool,

    /// Maximum number of recent repositories to remember
    pub max_recent: u32,

    /// Directories to search for repositories
    pub search_directories: Vec<PathBuf>,
}

/// Information about a repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct RepositoryInfo {
    /// Display name for the repository
    pub name: String,

    /// Path to the repository file
    pub path: String,

    /// Last accessed timestamp
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,

    /// Whether this repository is pinned/favorited
    pub pinned: bool,

    /// Repository-specific settings
    pub settings: RepositorySettings,
}

/// Settings specific to a repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RepositorySettings {
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,

    /// Whether to create backups automatically
    pub auto_backup: bool,

    /// Number of backups to keep
    pub backup_count: u32,

    /// Custom sort order for credentials
    pub sort_order: SortOrder,

    /// Default view mode
    pub view_mode: ViewMode,
}

/// Credential sort order options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    /// Sort by title alphabetically
    Title,
    /// Sort by creation date (newest first)
    Created,
    /// Sort by last modified date
    Modified,
    /// Sort by last accessed date
    Accessed,
    /// Sort by credential type
    Type,
    /// Custom user-defined order
    Custom,
}

/// View mode options for credential display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewMode {
    /// List view with details
    List,
    /// Grid/card view
    Grid,
    /// Compact list view
    Compact,
    /// Tree view organized by folders
    Tree,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            security: SecurityConfig::default(),
            behavior: AppBehaviorConfig::default(),
            repository_settings: RepositoryManagementConfig::default(),
            repositories: Vec::new(),
        }
    }
}

impl Default for AppBehaviorConfig {
    fn default() -> Self {
        Self {
            auto_check_updates: true,
            enable_backup: true,
            backup_count: 3,
        }
    }
}

impl Default for RepositoryManagementConfig {
    fn default() -> Self {
        Self {
            default_directory: None,
            auto_detect: true,
            max_recent: 10,
            search_directories: Vec::new(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "en".to_string(),
            auto_lock_timeout: 300, // 5 minutes
            window_width: Some(1200),
            window_height: Some(800),
            font_scale: Some(14.0),
            show_password_strength: true,
            start_minimized: false,
            show_wizard_on_startup: true,
            minimize_to_tray: false,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            password_timeout: 300, // 5 minutes
            clipboard_timeout: 30, // 30 seconds
            biometric_enabled: false,
            lock_on_suspend: true,
            clear_clipboard_on_lock: true,
            max_auth_attempts: 5,
            lockout_duration: 300, // 5 minutes
        }
    }
}

impl Default for RepositorySettings {
    fn default() -> Self {
        Self {
            auto_save_interval: 0, // Disabled by default
            auto_backup: true,
            backup_count: 5,
            sort_order: SortOrder::Title,
            view_mode: ViewMode::List,
        }
    }
}

impl RepositoryInfo {
    /// Create a new repository info entry
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            last_accessed: None,
            pinned: false,
            settings: RepositorySettings::default(),
        }
    }

    /// Create a repository info with current timestamp
    pub fn with_access_time(name: String, path: String) -> Self {
        Self {
            name,
            path,
            last_accessed: Some(chrono::Utc::now()),
            pinned: false,
            settings: RepositorySettings::default(),
        }
    }

    /// Update the last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = Some(chrono::Utc::now());
    }

    /// Check if this repository was recently accessed (within 24 hours)
    pub fn is_recently_accessed(&self) -> bool {
        if let Some(last_accessed) = self.last_accessed {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(last_accessed);
            duration.num_hours() < 24
        } else {
            false
        }
    }

    /// Get a user-friendly display name
    pub fn display_name(&self) -> String {
        if self.name.is_empty() {
            // Extract filename from path as fallback
            let filename = if self.path.contains('\\') {
                // Windows path
                self.path
                    .split('\\')
                    .next_back()
                    .unwrap_or("Unnamed Repository")
            } else {
                // Unix path
                self.path
                    .split('/')
                    .next_back()
                    .unwrap_or("Unnamed Repository")
            };

            // Remove common archive extensions
            if let Some(name) = filename.strip_suffix(".7z") {
                name.to_string()
            } else if let Some(name) = filename.strip_suffix(".zip") {
                name.to_string()
            } else {
                filename.to_string()
            }
        } else {
            self.name.clone()
        }
    }
}

/// Configuration presets for different usage scenarios
pub struct ConfigPresets;

impl ConfigPresets {
    /// High security configuration preset
    pub fn high_security() -> AppConfig {
        let mut config = AppConfig::default();
        config.security.password_timeout = 60; // 1 minute
        config.security.clipboard_timeout = 10; // 10 seconds
        config.security.max_auth_attempts = 3;
        config.security.lockout_duration = 600; // 10 minutes
        config.ui.auto_lock_timeout = 120; // 2 minutes
        config.behavior.auto_check_updates = false; // Disable for security
        config.behavior.enable_backup = true; // Keep backups for security
        config
    }

    /// Development/testing configuration preset
    pub fn development() -> AppConfig {
        let mut config = AppConfig::default();
        config.security.password_timeout = 3600; // 1 hour
        config.security.clipboard_timeout = 300; // 5 minutes
        config.ui.auto_lock_timeout = 3600; // 1 hour
        config.behavior.auto_check_updates = false; // Disable for development
        config
    }

    /// Mobile-friendly configuration preset
    pub fn mobile() -> AppConfig {
        let mut config = AppConfig::default();
        config.ui.window_width = None;
        config.ui.window_height = None;
        config.ui.font_scale = Some(1.2); // Larger text for mobile
        config.ui.start_minimized = false;
        config.ui.minimize_to_tray = false; // Not applicable on mobile
        config.security.biometric_enabled = true;
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_config() {
        let config = AppConfig::default();

        assert_eq!(config.ui.theme, "system");
        assert_eq!(config.ui.language, "en");
        assert_eq!(config.ui.auto_lock_timeout, 300);
        assert_eq!(config.security.password_timeout, 300);
        assert_eq!(config.security.clipboard_timeout, 30);
        assert!(config.behavior.auto_check_updates);
        assert!(config.behavior.enable_backup);
        assert_eq!(config.behavior.backup_count, 3);
        assert!(config.repository_settings.auto_detect);
        assert_eq!(config.repository_settings.max_recent, 10);
        assert!(config.repositories.is_empty());
    }

    #[test]
    fn test_repository_info() {
        let mut repo = RepositoryInfo::new("Test Repo".to_string(), "/path/to/repo.7z".to_string());

        assert_eq!(repo.display_name(), "Test Repo");
        assert!(!repo.is_recently_accessed());

        repo.touch();
        assert!(repo.is_recently_accessed());
        assert!(repo.last_accessed.is_some());
    }

    #[test]
    fn test_repository_display_name_fallback() {
        let repo = RepositoryInfo::new("".to_string(), "/path/to/my-vault.7z".to_string());
        assert_eq!(repo.display_name(), "my-vault");

        let repo_windows =
            RepositoryInfo::new("".to_string(), "C:\\Users\\test\\vault.7z".to_string());
        assert_eq!(repo_windows.display_name(), "vault");

        let repo_no_ext = RepositoryInfo::new("".to_string(), "/path/to/vault".to_string());
        assert_eq!(repo_no_ext.display_name(), "vault");
    }

    #[test]
    fn test_config_presets() {
        let high_sec = ConfigPresets::high_security();
        assert_eq!(high_sec.security.password_timeout, 60);
        assert_eq!(high_sec.security.max_auth_attempts, 3);

        let dev = ConfigPresets::development();
        assert_eq!(dev.security.password_timeout, 3600);
        assert_eq!(dev.ui.auto_lock_timeout, 3600);

        let mobile = ConfigPresets::mobile();
        assert!(mobile.ui.window_width.is_none());
        assert!(mobile.security.biometric_enabled);
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default();

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("theme"));
        assert!(yaml.contains("system"));
        assert!(yaml.contains("behavior"));
        assert!(yaml.contains("repository_settings"));

        let deserialized: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.ui.theme, deserialized.ui.theme);
        assert_eq!(
            config.security.password_timeout,
            deserialized.security.password_timeout
        );
        assert_eq!(
            config.behavior.auto_check_updates,
            deserialized.behavior.auto_check_updates
        );
        assert_eq!(
            config.repository_settings.auto_detect,
            deserialized.repository_settings.auto_detect
        );
    }

    #[test]
    fn test_sort_order_serialization() {
        let orders = vec![
            SortOrder::Title,
            SortOrder::Created,
            SortOrder::Modified,
            SortOrder::Accessed,
            SortOrder::Type,
            SortOrder::Custom,
        ];

        for order in orders {
            let yaml = serde_yaml::to_string(&order).unwrap();
            let deserialized: SortOrder = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(order, deserialized);
        }
    }

    #[test]
    fn test_view_mode_serialization() {
        let modes = vec![
            ViewMode::List,
            ViewMode::Grid,
            ViewMode::Compact,
            ViewMode::Tree,
        ];

        for mode in modes {
            let yaml = serde_yaml::to_string(&mode).unwrap();
            let deserialized: ViewMode = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(mode, deserialized);
        }
    }
}
