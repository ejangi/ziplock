//! Configuration management for ZipLock unified architecture
//!
//! This module provides configuration management capabilities primarily for desktop
//! applications that need to manage application settings and repository information.
//! Mobile platforms typically handle configuration through their native frameworks
//! and use the shared library only for credential operations.
//!
//! # Architecture Integration
//!
//! - **Desktop Apps**: Use full configuration management with file persistence
//! - **Mobile Apps**: May use subset of configuration types for memory operations
//! - **Repository Config**: Integrates with UnifiedRepositoryManager
//! - **File Operations**: Uses FileOperationProvider for config persistence

pub mod app_config;
pub mod repository_config;

pub use app_config::*;
pub use repository_config::*;

use crate::core::{CoreError, CoreResult, FileOperationProvider};

/// Configuration manager for desktop applications
///
/// Handles loading, saving, and managing application configuration files.
/// Uses the FileOperationProvider pattern for cross-platform file operations.
pub struct ConfigManager<F: FileOperationProvider> {
    file_provider: F,
    config_path: String,
    app_config: AppConfig,
    loaded: bool,
}

impl<F: FileOperationProvider> ConfigManager<F> {
    /// Create a new configuration manager
    ///
    /// # Arguments
    /// * `file_provider` - File operation provider for config persistence
    /// * `config_path` - Path to the configuration file
    pub fn new(file_provider: F, config_path: String) -> Self {
        Self {
            file_provider,
            config_path,
            app_config: AppConfig::default(),
            loaded: false,
        }
    }

    /// Load configuration from file
    ///
    /// If the configuration file doesn't exist, uses default configuration.
    /// This method is safe to call multiple times.
    pub fn load(&mut self) -> CoreResult<()> {
        match self.file_provider.read_archive(&self.config_path) {
            Ok(data) => {
                let config_str =
                    String::from_utf8(data).map_err(|e| CoreError::SerializationError {
                        message: format!("Invalid UTF-8 in config file: {e}"),
                    })?;

                self.app_config = serde_yaml::from_str(&config_str).map_err(|e| {
                    CoreError::SerializationError {
                        message: format!("Failed to parse config YAML: {e}"),
                    }
                })?;

                self.loaded = true;
                Ok(())
            }
            Err(_) => {
                // Config file doesn't exist, use defaults
                self.app_config = AppConfig::default();
                self.loaded = true;
                Ok(())
            }
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> CoreResult<()> {
        if !self.loaded {
            return Err(CoreError::NotInitialized);
        }

        let config_yaml =
            serde_yaml::to_string(&self.app_config).map_err(|e| CoreError::SerializationError {
                message: format!("Failed to serialize config: {e}"),
            })?;

        self.file_provider
            .write_archive(&self.config_path, config_yaml.as_bytes())
            .map_err(CoreError::FileOperation)?;

        Ok(())
    }

    /// Get immutable reference to configuration
    pub fn config(&self) -> &AppConfig {
        &self.app_config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.app_config
    }

    /// Check if configuration has been loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Add a repository to the recent repositories list
    pub fn add_recent_repository(&mut self, repo_info: RepositoryInfo) {
        // Remove existing entry if present
        self.app_config
            .repositories
            .retain(|r| r.path != repo_info.path);

        // Add to front of list
        self.app_config.repositories.insert(0, repo_info);

        // Keep only the most recent entries
        const MAX_RECENT: usize = 10;
        if self.app_config.repositories.len() > MAX_RECENT {
            self.app_config.repositories.truncate(MAX_RECENT);
        }
    }

    /// Remove a repository from the recent repositories list
    pub fn remove_recent_repository(&mut self, path: &str) {
        self.app_config.repositories.retain(|r| r.path != path);
    }

    /// Update the last accessed time for a repository
    pub fn touch_repository(&mut self, path: &str) {
        if let Some(repo) = self
            .app_config
            .repositories
            .iter_mut()
            .find(|r| r.path == path)
        {
            repo.last_accessed = Some(chrono::Utc::now());
        }
    }

    /// Get recent repositories sorted by last accessed (most recent first)
    pub fn get_recent_repositories(&self) -> Vec<&RepositoryInfo> {
        let mut repos: Vec<&RepositoryInfo> = self.app_config.repositories.iter().collect();
        repos.sort_by(|a, b| match (a.last_accessed, b.last_accessed) {
            (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        repos
    }
}

/// Default configuration paths for different platforms
pub struct ConfigPaths;

impl ConfigPaths {
    /// Get the default application config directory for the current platform
    #[cfg(target_os = "linux")]
    pub fn app_config_dir() -> String {
        if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            format!("{xdg_config_home}/ziplock")
        } else if let Ok(home) = std::env::var("HOME") {
            format!("{home}/.config/ziplock")
        } else {
            "./.config/ziplock".to_string()
        }
    }

    #[cfg(target_os = "windows")]
    pub fn app_config_dir() -> String {
        if let Ok(appdata) = std::env::var("APPDATA") {
            format!("{}\\ZipLock", appdata)
        } else {
            ".\\config".to_string()
        }
    }

    #[cfg(target_os = "macos")]
    pub fn app_config_dir() -> String {
        if let Ok(home) = std::env::var("HOME") {
            format!("{}/Library/Application Support/ZipLock", home)
        } else {
            "./config".to_string()
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    pub fn app_config_dir() -> String {
        "./config".to_string()
    }

    /// Get the default application config file path
    pub fn app_config_file() -> String {
        format!("{}/config.yml", Self::app_config_dir())
    }

    /// Get the default repositories directory
    pub fn default_repositories_dir() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                format!("{home}/Documents/ZipLock")
            } else {
                "./repositories".to_string()
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                format!("{}\\Documents\\ZipLock", userprofile)
            } else {
                ".\\repositories".to_string()
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                format!("{}/Documents/ZipLock", home)
            } else {
                "./repositories".to_string()
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            "./repositories".to_string()
        }
    }
}

/// Configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate application configuration
    pub fn validate_app_config(config: &AppConfig) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate UI configuration
        if config.ui.auto_lock_timeout == 0 {
            errors.push("Auto lock timeout cannot be zero".to_string());
        }

        if config.ui.auto_lock_timeout > 86400 {
            errors.push("Auto lock timeout cannot exceed 24 hours".to_string());
        }

        // Validate security configuration
        if config.security.password_timeout > 3600 {
            errors.push("Password timeout should not exceed 1 hour for security".to_string());
        }

        if config.security.clipboard_timeout > 300 {
            errors.push("Clipboard timeout should not exceed 5 minutes for security".to_string());
        }

        // Validate repository paths
        for repo in &config.repositories {
            if repo.path.is_empty() {
                errors.push(format!("Repository '{}' has empty path", repo.name));
            }

            if repo.name.is_empty() {
                errors.push(format!("Repository at '{}' has empty name", repo.path));
            }
        }

        errors
    }

    /// Check if a repository path appears to be valid
    pub fn is_valid_repository_path(path: &str) -> bool {
        !path.is_empty() && (path.ends_with(".7z") || path.ends_with(".zip"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::MockFileProvider;

    #[test]
    fn test_config_manager_lifecycle() {
        let provider = MockFileProvider::new();
        let config_path = "/test/config.yml".to_string();

        let mut manager = ConfigManager::new(provider, config_path);
        assert!(!manager.is_loaded());

        // Load should succeed even if file doesn't exist
        manager.load().unwrap();
        assert!(manager.is_loaded());

        // Should have default config
        assert_eq!(manager.config().ui.theme, "system");
    }

    #[test]
    fn test_recent_repositories_management() {
        let provider = MockFileProvider::new();
        let mut manager = ConfigManager::new(provider, "/test/config.yml".to_string());
        manager.load().unwrap();

        let repo1 = RepositoryInfo {
            name: "Test Repo 1".to_string(),
            path: "/path/to/repo1.7z".to_string(),
            last_accessed: None,
            pinned: false,
            settings: Default::default(),
        };

        let repo2 = RepositoryInfo {
            name: "Test Repo 2".to_string(),
            path: "/path/to/repo2.7z".to_string(),
            last_accessed: None,
            pinned: false,
            settings: Default::default(),
        };

        manager.add_recent_repository(repo1);
        manager.add_recent_repository(repo2);

        let recent = manager.get_recent_repositories();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].name, "Test Repo 2"); // Most recently accessed first
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        let errors = ConfigValidator::validate_app_config(&config);
        assert!(errors.is_empty());

        // Test invalid timeout
        config.ui.auto_lock_timeout = 0;
        let errors = ConfigValidator::validate_app_config(&config);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_repository_path_validation() {
        assert!(ConfigValidator::is_valid_repository_path(
            "/path/to/repo.7z"
        ));
        assert!(ConfigValidator::is_valid_repository_path(
            "/path/to/repo.zip"
        ));
        assert!(!ConfigValidator::is_valid_repository_path(
            "/path/to/repo.txt"
        ));
        assert!(!ConfigValidator::is_valid_repository_path(""));
    }

    #[test]
    fn test_config_paths() {
        let config_dir = ConfigPaths::app_config_dir();
        assert!(!config_dir.is_empty());

        let config_file = ConfigPaths::app_config_file();
        assert!(config_file.contains("config.yml"));

        let repos_dir = ConfigPaths::default_repositories_dir();
        assert!(!repos_dir.is_empty());
    }
}
