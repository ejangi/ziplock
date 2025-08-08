//! Shared configuration management for ZipLock applications
//!
//! This module provides cross-platform configuration management that can be used
//! by all frontend implementations. It handles user preferences, repository paths,
//! and provides repository detection functionality.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

pub mod paths;
pub mod repository;

use crate::error::SharedResult;

/// Main configuration structure for frontend applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// Repository settings
    pub repository: RepositoryConfig,

    /// UI preferences
    pub ui: UiConfig,

    /// Application settings
    pub app: AppConfig,

    /// Configuration version for future migrations
    pub version: String,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Path to the currently selected repository (zip file)
    pub path: Option<PathBuf>,

    /// Default directory for creating new repositories
    pub default_directory: Option<PathBuf>,

    /// Recently used repositories with metadata
    pub recent_repositories: Vec<RecentRepository>,

    /// Maximum number of recent repositories to remember
    pub max_recent: usize,

    /// Automatically detect repositories on startup
    pub auto_detect: bool,

    /// Search additional directories for repositories
    pub search_directories: Vec<PathBuf>,
}

/// Information about a recently used repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentRepository {
    /// Path to the repository file
    pub path: PathBuf,

    /// Last accessed timestamp
    pub last_accessed: SystemTime,

    /// Display name (defaults to filename)
    pub display_name: Option<String>,

    /// Whether this repository is pinned (always shown)
    pub pinned: bool,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Window width
    pub window_width: u32,

    /// Window height
    pub window_height: u32,

    /// Theme selection ("light", "dark", "system")
    pub theme: String,

    /// Remember window size and position
    pub remember_window_state: bool,

    /// Show wizard on startup if no repository is configured
    pub show_wizard_on_startup: bool,

    /// Font size for UI elements
    pub font_size: f32,

    /// UI language/locale
    pub language: String,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Auto-lock timeout in minutes (0 = disabled)
    pub auto_lock_timeout: u32,

    /// Clear clipboard after copying password (seconds)
    pub clipboard_timeout: u32,

    /// Enable auto-backup
    pub enable_backup: bool,

    /// Show passwords by default (not recommended)
    pub show_passwords_default: bool,

    /// Enable password strength indicators
    pub show_password_strength: bool,

    /// Minimize to system tray on close
    pub minimize_to_tray: bool,

    /// Start minimized
    pub start_minimized: bool,

    /// Check for updates automatically
    pub auto_check_updates: bool,
}

/// Repository information for detection and validation
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryInfo {
    /// Path to the repository file
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Last modified timestamp
    pub last_modified: SystemTime,

    /// Whether the file is accessible for reading
    pub accessible: bool,

    /// Display name
    pub display_name: String,

    /// Whether this is a valid ZipLock repository format
    pub is_valid_format: bool,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            repository: RepositoryConfig::default(),
            ui: UiConfig::default(),
            app: AppConfig::default(),
            version: "1.0".to_string(),
        }
    }
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        let default_directory = dirs::document_dir()
            .or_else(|| dirs::home_dir())
            .map(|p| p.join("ZipLock"));

        Self {
            path: None,
            default_directory,
            recent_repositories: Vec::new(),
            max_recent: 10,
            auto_detect: true,
            search_directories: Vec::new(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width: 1000,
            window_height: 700,
            theme: "system".to_string(),
            remember_window_state: true,
            show_wizard_on_startup: true,
            font_size: 14.0,
            language: "en".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_lock_timeout: 15, // 15 minutes
            clipboard_timeout: 30, // 30 seconds
            enable_backup: true,
            show_passwords_default: false,
            show_password_strength: true,
            minimize_to_tray: false,
            start_minimized: false,
            auto_check_updates: true,
        }
    }
}

impl RecentRepository {
    /// Create a new recent repository entry
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        let display_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Self {
            path,
            last_accessed: SystemTime::now(),
            display_name,
            pinned: false,
        }
    }

    /// Update the last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Get the display name or filename
    pub fn display_name(&self) -> String {
        self.display_name.clone().unwrap_or_else(|| {
            self.path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        })
    }

    /// Check if the repository file exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

impl RepositoryInfo {
    /// Create repository info from a file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> SharedResult<Self> {
        let path = path.as_ref().to_path_buf();

        let metadata = fs::metadata(&path).map_err(|e| crate::SharedError::Internal {
            message: format!("Failed to read file metadata: {}", e),
        })?;

        let display_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(Self {
            path,
            size: metadata.len(),
            last_modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            accessible: true,
            display_name,
            is_valid_format: false, // Will be validated separately
        })
    }

    /// Check if this appears to be a ZipLock repository based on extension
    pub fn has_valid_extension(&self) -> bool {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("7z"))
            .unwrap_or(false)
    }
}

/// Configuration manager for frontend applications
#[derive(Debug)]
pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
    config: FrontendConfig,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let config_dir = paths::get_config_directory()?;
        let config_file = config_dir.join("config.toml");

        // Ensure config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
            info!("Created config directory: {:?}", config_dir);
        }

        // Load existing config or create default
        let config = if config_file.exists() {
            Self::load_config(&config_file)?
        } else {
            let default_config = FrontendConfig::default();
            Self::save_config(&config_file, &default_config)?;
            info!("Created default config file: {:?}", config_file);
            default_config
        };

        Ok(Self {
            config_dir,
            config_file,
            config,
        })
    }

    /// Load configuration from file
    fn load_config(path: &Path) -> Result<FrontendConfig> {
        debug!("Loading config from: {:?}", path);

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let mut config: FrontendConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        // Validate and clean up recent repositories
        config.repository.recent_repositories.retain(|repo| {
            if repo.exists() {
                true
            } else {
                warn!(
                    "Removing non-existent repository from recent list: {:?}",
                    repo.path
                );
                false
            }
        });

        info!("Successfully loaded config from: {:?}", path);
        Ok(config)
    }

    /// Save configuration to file
    fn save_config(path: &Path, config: &FrontendConfig) -> Result<()> {
        debug!("Saving config to: {:?}", path);

        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        info!("Successfully saved config to: {:?}", path);
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &FrontendConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut FrontendConfig {
        &mut self.config
    }

    /// Save the current configuration to disk
    pub fn save(&self) -> Result<()> {
        Self::save_config(&self.config_file, &self.config)
    }

    /// Set the repository path and add to recent repositories
    pub fn set_repository_path<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
        let path = path.into();
        debug!("Setting repository path to: {:?}", path);

        // Update or add to recent repositories
        let mut recent_repo = RecentRepository::new(&path);

        // Check if it already exists in recent list
        if let Some(existing) = self
            .config
            .repository
            .recent_repositories
            .iter_mut()
            .find(|r| r.path == path)
        {
            existing.touch();
            if existing.pinned {
                recent_repo.pinned = true;
            }
            if existing.display_name.is_some() {
                recent_repo.display_name = existing.display_name.clone();
            }
        }

        // Remove existing entry and add to front
        self.config
            .repository
            .recent_repositories
            .retain(|r| r.path != path);
        self.config
            .repository
            .recent_repositories
            .insert(0, recent_repo);

        // Limit the number of recent repositories (keep pinned ones)
        let max_recent = self.config.repository.max_recent;
        if self.config.repository.recent_repositories.len() > max_recent {
            // Separate pinned and unpinned repositories
            let (mut pinned, mut unpinned): (Vec<_>, Vec<_>) = self
                .config
                .repository
                .recent_repositories
                .drain(..)
                .partition(|r| r.pinned);

            // Sort by last accessed
            unpinned.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

            // Keep all pinned + up to max_recent unpinned
            pinned.extend(unpinned.into_iter().take(max_recent));
            self.config.repository.recent_repositories = pinned;
        }

        self.config.repository.path = Some(path);
        self.save()
    }

    /// Get the current repository path
    pub fn repository_path(&self) -> Option<&PathBuf> {
        self.config.repository.path.as_ref()
    }

    /// Check if a repository is configured
    pub fn has_repository(&self) -> bool {
        self.config.repository.path.is_some()
    }

    /// Get recent repositories
    pub fn recent_repositories(&self) -> &[RecentRepository] {
        &self.config.repository.recent_repositories
    }

    /// Remove a repository from recent list
    pub fn remove_recent_repository(&mut self, path: &Path) -> Result<()> {
        self.config
            .repository
            .recent_repositories
            .retain(|r| r.path != path);

        // If this was the current repository, clear it
        if self.config.repository.path.as_ref() == Some(&path.to_path_buf()) {
            self.config.repository.path = None;
        }

        self.save()
    }

    /// Pin or unpin a recent repository
    pub fn set_repository_pinned(&mut self, path: &Path, pinned: bool) -> Result<()> {
        if let Some(repo) = self
            .config
            .repository
            .recent_repositories
            .iter_mut()
            .find(|r| r.path == path)
        {
            repo.pinned = pinned;
            self.save()
        } else {
            Ok(())
        }
    }

    /// Set display name for a repository
    pub fn set_repository_display_name(&mut self, path: &Path, name: Option<String>) -> Result<()> {
        if let Some(repo) = self
            .config
            .repository
            .recent_repositories
            .iter_mut()
            .find(|r| r.path == path)
        {
            repo.display_name = name;
            self.save()
        } else {
            Ok(())
        }
    }

    /// Discover repositories in configured search directories
    pub fn discover_repositories(&self) -> Vec<RepositoryInfo> {
        let mut repositories = Vec::new();

        // Search in default directory
        if let Some(default_dir) = &self.config.repository.default_directory {
            if let Ok(repos) = repository::find_repositories_in_directory(default_dir) {
                repositories.extend(repos);
            }
        }

        // Search in additional directories
        for search_dir in &self.config.repository.search_directories {
            if let Ok(repos) = repository::find_repositories_in_directory(search_dir) {
                repositories.extend(repos);
            }
        }

        // Remove duplicates and sort by last modified
        repositories.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        repositories.dedup_by(|a, b| a.path == b.path);

        repositories
    }

    /// Get repositories that exist from recent list
    pub fn get_accessible_recent_repositories(&self) -> Vec<RepositoryInfo> {
        self.config
            .repository
            .recent_repositories
            .iter()
            .filter(|r| r.exists())
            .filter_map(|r| RepositoryInfo::from_path(&r.path).ok())
            .collect()
    }

    /// Update UI settings
    pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<()> {
        if self.config.ui.remember_window_state {
            self.config.ui.window_width = width;
            self.config.ui.window_height = height;
            self.save()
        } else {
            Ok(())
        }
    }

    /// Get config directory path
    pub fn config_directory(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Add a search directory for repositories
    pub fn add_search_directory<P: Into<PathBuf>>(&mut self, dir: P) -> Result<()> {
        let dir = dir.into();
        if !self.config.repository.search_directories.contains(&dir) {
            self.config.repository.search_directories.push(dir);
            self.save()
        } else {
            Ok(())
        }
    }

    /// Remove a search directory
    pub fn remove_search_directory(&mut self, dir: &Path) -> Result<()> {
        self.config
            .repository
            .search_directories
            .retain(|d| d != dir);
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = FrontendConfig::default();
        assert!(config.repository.path.is_none());
        assert!(config.ui.window_width > 0);
        assert!(config.ui.window_height > 0);
        assert_eq!(config.app.language, "en");
        assert_eq!(config.version, "1.0");
    }

    #[test]
    fn test_config_serialization() {
        let config = FrontendConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: FrontendConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.ui.window_width, deserialized.ui.window_width);
        assert_eq!(
            config.app.auto_lock_timeout,
            deserialized.app.auto_lock_timeout
        );
        assert_eq!(config.version, deserialized.version);
    }

    #[test]
    fn test_recent_repository() {
        let path = PathBuf::from("/test/repo.7z");
        let mut recent = RecentRepository::new(&path);

        assert_eq!(recent.path, path);
        assert_eq!(recent.display_name(), "repo");
        assert!(!recent.pinned);

        recent.pinned = true;
        let old_time = recent.last_accessed;
        std::thread::sleep(std::time::Duration::from_millis(1));
        recent.touch();
        assert!(recent.last_accessed > old_time);
    }

    #[test]
    fn test_repository_info() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.7z");

        // Create a test file
        File::create(&repo_path).unwrap();

        let info = RepositoryInfo::from_path(&repo_path).unwrap();
        assert_eq!(info.path, repo_path);
        assert_eq!(info.display_name, "test");
        assert!(info.has_valid_extension());
        assert!(info.accessible);
    }

    #[test]
    fn test_repository_manager_recent_list() {
        let mut config = FrontendConfig::default();
        config.repository.max_recent = 3;

        // Simulate adding repositories
        let paths = vec![
            PathBuf::from("/test/repo1.7z"),
            PathBuf::from("/test/repo2.7z"),
            PathBuf::from("/test/repo3.7z"),
            PathBuf::from("/test/repo4.7z"),
        ];

        let mut recent_repos = Vec::new();
        for path in &paths {
            let mut repo = RecentRepository::new(path);
            // Pin the second repository
            if path.to_string_lossy().contains("repo2") {
                repo.pinned = true;
            }
            recent_repos.insert(0, repo);
        }

        config.repository.recent_repositories = recent_repos;

        // Simulate trimming logic
        let max_recent = config.repository.max_recent;
        if config.repository.recent_repositories.len() > max_recent {
            let (pinned, mut unpinned): (Vec<_>, Vec<_>) = config
                .repository
                .recent_repositories
                .drain(..)
                .partition(|r| r.pinned);

            unpinned.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
            let mut result = pinned;
            result.extend(unpinned.into_iter().take(max_recent));
            config.repository.recent_repositories = result;
        }

        // Should keep pinned repo plus 3 most recent
        assert!(config.repository.recent_repositories.len() <= max_recent + 1);
        assert!(config
            .repository
            .recent_repositories
            .iter()
            .any(|r| r.pinned));
    }
}
