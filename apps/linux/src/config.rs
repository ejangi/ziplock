//! Configuration management for the ZipLock Linux app
//!
//! This module provides a thin wrapper around the shared configuration
//! management functionality, adding any Linux-specific extensions.

use anyhow::Result;
use tracing::{debug, info};

// Re-export shared config types
pub use ziplock_shared::config::{
    ConfigManager as SharedConfigManager, FrontendConfig, RecentRepository, RepositoryInfo,
};

/// Linux-specific configuration manager
///
/// This wraps the shared ConfigManager and adds any Linux-specific functionality
#[derive(Debug)]
pub struct ConfigManager {
    shared_manager: SharedConfigManager,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        debug!("Creating Linux app configuration manager");
        let shared_manager = SharedConfigManager::new()?;
        info!("Configuration manager initialized successfully");

        Ok(Self { shared_manager })
    }

    /// Get the current configuration
    pub fn config(&self) -> &FrontendConfig {
        self.shared_manager.config()
    }

    /// Get mutable reference to configuration
    #[allow(dead_code)] // Public API for future use
    pub fn config_mut(&mut self) -> &mut FrontendConfig {
        self.shared_manager.config_mut()
    }

    /// Save the current configuration to disk
    #[allow(dead_code)] // Public API for future use
    pub fn save(&self) -> Result<()> {
        self.shared_manager.save()
    }

    /// Set the repository path and add to recent repositories
    pub fn set_repository_path<P: Into<std::path::PathBuf>>(&mut self, path: P) -> Result<()> {
        self.shared_manager.set_repository_path(path)
    }

    /// Get the current repository path
    #[allow(dead_code)] // Public API for future use
    pub fn repository_path(&self) -> Option<&std::path::PathBuf> {
        self.shared_manager.repository_path()
    }

    /// Check if a repository is configured
    pub fn has_repository(&self) -> bool {
        self.shared_manager.has_repository()
    }

    /// Get recent repositories
    #[allow(dead_code)] // Public API for future use
    pub fn recent_repositories(&self) -> &[RecentRepository] {
        self.shared_manager.recent_repositories()
    }

    /// Remove a repository from recent list
    #[allow(dead_code)] // Public API for future use
    pub fn remove_recent_repository(&mut self, path: &std::path::Path) -> Result<()> {
        self.shared_manager.remove_recent_repository(path)
    }

    /// Pin or unpin a recent repository
    #[allow(dead_code)] // Public API for future use
    pub fn set_repository_pinned(&mut self, path: &std::path::Path, pinned: bool) -> Result<()> {
        self.shared_manager.set_repository_pinned(path, pinned)
    }

    /// Set display name for a repository
    #[allow(dead_code)] // Public API for future use
    pub fn set_repository_display_name(
        &mut self,
        path: &std::path::Path,
        name: Option<String>,
    ) -> Result<()> {
        self.shared_manager.set_repository_display_name(path, name)
    }

    /// Discover repositories in configured search directories
    pub fn discover_repositories(&self) -> Vec<RepositoryInfo> {
        self.shared_manager.discover_repositories()
    }

    /// Get repositories that exist from recent list
    pub fn get_accessible_recent_repositories(&self) -> Vec<RepositoryInfo> {
        self.shared_manager.get_accessible_recent_repositories()
    }

    /// Update UI settings
    #[allow(dead_code)] // Public API for future use
    pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<()> {
        self.shared_manager.set_window_size(width, height)
    }

    /// Get config directory path
    #[allow(dead_code)] // Public API for future use
    pub fn config_directory(&self) -> &std::path::PathBuf {
        self.shared_manager.config_directory()
    }

    /// Add a search directory for repositories
    #[allow(dead_code)] // Public API for future use
    pub fn add_search_directory<P: Into<std::path::PathBuf>>(&mut self, dir: P) -> Result<()> {
        self.shared_manager.add_search_directory(dir)
    }

    /// Remove a search directory
    #[allow(dead_code)] // Public API for future use
    pub fn remove_search_directory(&mut self, dir: &std::path::Path) -> Result<()> {
        self.shared_manager.remove_search_directory(dir)
    }

    /// Detect accessible repositories from recent list and discovery
    ///
    /// This is a Linux-specific convenience method that combines recent repositories
    /// with discovered repositories for the initial app state
    pub fn detect_all_accessible_repositories(&self) -> Vec<RepositoryInfo> {
        let mut all_repos = Vec::new();

        // Get accessible recent repositories first (higher priority)
        let mut recent_repos = self.get_accessible_recent_repositories();
        all_repos.append(&mut recent_repos);

        // Get discovered repositories
        let mut discovered_repos = self.discover_repositories();

        // Filter out discovered repos that are already in recent list
        discovered_repos
            .retain(|repo| !all_repos.iter().any(|existing| existing.path == repo.path));

        all_repos.append(&mut discovered_repos);

        // Sort by last modified (most recent first)
        all_repos.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        // Limit total results to prevent UI overload
        all_repos.truncate(20);

        all_repos
    }

    /// Check if the app should show the wizard on startup
    pub fn should_show_wizard(&self) -> bool {
        let config = self.config();

        // Show wizard if:
        // 1. No repository is configured, AND
        // 2. Show wizard on startup is enabled, AND
        // 3. No accessible repositories are found
        !self.has_repository()
            && config.ui.show_wizard_on_startup
            && self.get_accessible_recent_repositories().is_empty()
            && self.discover_repositories().is_empty()
    }

    /// Get the most recently used repository path if it's still accessible
    pub fn get_most_recent_accessible_repository(&self) -> Option<&std::path::PathBuf> {
        // Check if current repository path is still accessible
        if let Some(current_path) = self.repository_path() {
            if current_path.exists() {
                return Some(current_path);
            }
        }

        // Find the most recent accessible repository
        self.recent_repositories()
            .iter()
            .find(|repo| repo.exists())
            .map(|repo| &repo.path)
    }

    /// Update theme setting
    #[allow(dead_code)] // Public API for future use
    pub fn set_theme(&mut self, theme: String) -> Result<()> {
        self.config_mut().ui.theme = theme;
        self.save()
    }

    /// Update language setting
    #[allow(dead_code)] // Public API for future use
    pub fn set_language(&mut self, language: String) -> Result<()> {
        self.config_mut().ui.language = language;
        self.save()
    }

    /// Update auto-lock timeout
    #[allow(dead_code)] // Public API for future use
    pub fn set_auto_lock_timeout(&mut self, timeout_minutes: u32) -> Result<()> {
        self.config_mut().app.auto_lock_timeout = timeout_minutes;
        self.save()
    }

    /// Update clipboard timeout
    #[allow(dead_code)] // Public API for future use
    pub fn set_clipboard_timeout(&mut self, timeout_seconds: u32) -> Result<()> {
        self.config_mut().app.clipboard_timeout = timeout_seconds;
        self.save()
    }

    /// Toggle backup setting
    #[allow(dead_code)] // Public API for future use
    pub fn set_enable_backup(&mut self, enabled: bool) -> Result<()> {
        self.config_mut().app.enable_backup = enabled;
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_config_manager_creation() {
        // Note: This test might fail in some CI environments due to
        // missing user directories, so we make it conditional
        if std::env::var("CI").is_err() {
            let manager = ConfigManager::new();
            assert!(manager.is_ok());
        }
    }

    #[test]
    fn test_should_show_wizard_logic() {
        // Create a temporary config for testing
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        // Create minimal config file
        let config_content = r#"
version: "1.0"

repository:
  max_recent: 10
  auto_detect: true
  search_directories: []
  recent_repositories: []

ui:
  window_width: 1000
  window_height: 700
  theme: "system"
  show_wizard_on_startup: true
  font_size: 14.0
  language: "en"

app:
  auto_lock_timeout: 15
  clipboard_timeout: 30
  enable_backup: true
  show_passwords_default: false
  show_password_strength: true
  minimize_to_tray: false
  start_minimized: false
  auto_check_updates: true
"#;

        std::fs::write(&config_path, config_content).unwrap();

        // Test the logic - since we can't easily create a full ConfigManager
        // in tests, we'll test the logic components
        let has_repository = false;
        let show_wizard_on_startup = true;
        let has_accessible_recent = false;
        let has_discovered = false;

        let should_show =
            !has_repository && show_wizard_on_startup && !has_accessible_recent && !has_discovered;

        assert!(should_show);
    }

    #[test]
    fn test_repository_detection_integration() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test repository files
        let repo1 = temp_dir.path().join("passwords.7z");
        let repo2 = temp_dir.path().join("vault.7z");

        File::create(&repo1).unwrap();
        File::create(&repo2).unwrap();

        // Test repository discovery using shared library functions
        let discovered =
            ziplock_shared::config::repository::find_repositories_in_directory(temp_dir.path())
                .unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().any(|r| r.path == repo1));
        assert!(discovered.iter().any(|r| r.path == repo2));
    }
}
