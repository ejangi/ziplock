//! Update checker service for Linux desktop app
//!
//! This module provides functionality to check for application updates
//! and notify users when new versions are available.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

/// Version information for update checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub last_checked: u64,
    pub installation_method: InstallationMethod,
}

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub latest_release: Option<ReleaseInfo>,
    pub installation_method: InstallationMethod,
    pub last_checked: u64,
}

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub published_at: String,
    pub body: Option<String>,
    pub html_url: Option<String>,
}

/// Installation method detection for update handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationMethod {
    DebianPackage,
    ArchAUR,
    Manual,
    Unknown,
}

/// Update checker service
#[derive(Debug, Clone)]
pub struct UpdateChecker {
    current_version: String,
    check_url: String,
    last_check: Option<SystemTime>,
    check_interval: Duration,
}

impl UpdateChecker {
    /// Detect the installation method based on system characteristics
    pub fn detect_installation_method() -> InstallationMethod {
        // Check if installed via package manager
        if std::path::Path::new("/var/lib/dpkg/info/ziplock.list").exists() {
            InstallationMethod::DebianPackage
        } else if std::path::Path::new("/var/lib/pacman/local").exists()
            && std::process::Command::new("pacman")
                .args(&["-Q", "ziplock"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        {
            InstallationMethod::ArchAUR
        } else if std::env::current_exe()
            .map(|path| {
                path.to_string_lossy().contains("/usr/local/bin")
                    || path.to_string_lossy().contains("/opt")
            })
            .unwrap_or(false)
        {
            InstallationMethod::Manual
        } else {
            InstallationMethod::Unknown
        }
    }

    /// Create a new update checker
    pub fn new() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            check_url: "https://api.github.com/repos/ejangi/ziplock/releases/latest".to_string(),
            last_check: None,
            check_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }

    /// Create update checker with custom parameters
    #[allow(dead_code)]
    pub fn with_config(current_version: String, check_url: String) -> Self {
        Self {
            current_version,
            check_url,
            last_check: None,
            check_interval: Duration::from_secs(24 * 60 * 60),
        }
    }

    /// Check for updates if enough time has passed
    #[allow(dead_code)]
    pub async fn check_for_updates_if_needed(
        &mut self,
    ) -> Option<Result<UpdateCheckResult, anyhow::Error>> {
        let should_check = match self.last_check {
            Some(last) => {
                match SystemTime::now().duration_since(last) {
                    Ok(duration) => duration >= self.check_interval,
                    Err(_) => true, // Clock went backwards, check anyway
                }
            }
            None => true, // Never checked before
        };

        if should_check {
            Some(self.check_for_updates().await)
        } else {
            debug!("Skipping update check, too soon since last check");
            None
        }
    }

    /// Force check for updates regardless of timing
    pub async fn check_for_updates(&mut self) -> Result<UpdateCheckResult, anyhow::Error> {
        info!("Checking for application updates");
        self.last_check = Some(SystemTime::now());

        match self.fetch_latest_version().await {
            Ok(latest_version) => {
                let update_available = self.is_newer_version(&latest_version);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let release_info = if update_available {
                    Some(ReleaseInfo {
                        version: latest_version.clone(),
                        release_notes: None, // TODO: Fetch from GitHub API
                        download_url: Some(format!(
                            "https://github.com/ejangi/ziplock/releases/tag/v{}",
                            latest_version
                        )),
                        published_at: "Unknown".to_string(), // TODO: Fetch from GitHub API
                        body: Some(
                            "New version available with improvements and bug fixes.".to_string(),
                        ),
                        html_url: Some(format!(
                            "https://github.com/ejangi/ziplock/releases/tag/v{}",
                            latest_version
                        )),
                    })
                } else {
                    None
                };

                if update_available {
                    info!(
                        "Update available: {} -> {}",
                        self.current_version, latest_version
                    );
                } else {
                    info!("Application is up to date ({})", self.current_version);
                }

                Ok(UpdateCheckResult {
                    current_version: self.current_version.clone(),
                    latest_version: Some(latest_version),
                    update_available,
                    latest_release: release_info,
                    installation_method: Self::detect_installation_method(),
                    last_checked: now,
                })
            }
            Err(e) => {
                error!("Failed to check for updates: {}", e);
                Err(e)
            }
        }
    }

    /// Fetch the latest version from the remote source
    async fn fetch_latest_version(&self) -> Result<String> {
        debug!("Fetching latest version from: {}", self.check_url);

        // For now, return a mock version. In a real implementation, this would
        // make an HTTP request to the GitHub API or other update source.
        //
        // Example implementation:
        // let client = reqwest::Client::new();
        // let response: GitHubRelease = client
        //     .get(&self.check_url)
        //     .header("User-Agent", "ZipLock")
        //     .send()
        //     .await?
        //     .json()
        //     .await?;
        // Ok(response.tag_name.trim_start_matches('v').to_string())

        // Mock implementation - always return current version to simulate no updates
        Ok(self.current_version.clone())
    }

    /// Check if a version string represents a newer version than current
    fn is_newer_version(&self, other: &str) -> bool {
        match (
            self.parse_version(&self.current_version),
            self.parse_version(other),
        ) {
            (Some(current), Some(latest)) => latest > current,
            _ => false,
        }
    }

    /// Parse a semantic version string into comparable parts
    fn parse_version(&self, version: &str) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();
        if parts.len() >= 3 {
            if let (Ok(major), Ok(minor), Ok(patch)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return Some((major, minor, patch));
            }
        }
        None
    }

    /// Get current version
    #[allow(dead_code)]
    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    /// Set check interval
    #[allow(dead_code)]
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }

    /// Get time since last check
    #[allow(dead_code)]
    pub fn time_since_last_check(&self) -> Option<Duration> {
        self.last_check
            .and_then(|last| SystemTime::now().duration_since(last).ok())
    }
}

impl UpdateChecker {
    /// Check if enough time has passed since last update check
    pub fn should_auto_check(&self) -> bool {
        match self.last_check {
            Some(last) => {
                match SystemTime::now().duration_since(last) {
                    Ok(duration) => duration >= self.check_interval,
                    Err(_) => true, // Clock went backwards, check anyway
                }
            }
            None => true, // Never checked before
        }
    }
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub release response structure (for future HTTP implementation)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    html_url: String,
    published_at: String,
}

impl InstallationMethod {
    /// Get update instructions for this installation method
    pub fn update_instructions(&self) -> String {
        match self {
            InstallationMethod::DebianPackage => {
                "Update using your package manager:\nsudo apt update && sudo apt upgrade ziplock"
                    .to_string()
            }
            InstallationMethod::ArchAUR => {
                "Update using your AUR helper:\nyay -Syu ziplock\nor\nparu -Syu ziplock".to_string()
            }
            InstallationMethod::Manual => {
                "Download the latest release from GitHub and replace the existing installation."
                    .to_string()
            }
            InstallationMethod::Unknown => {
                "Please check the GitHub releases page for update instructions.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let checker = UpdateChecker::new();

        assert_eq!(checker.parse_version("1.0.0"), Some((1, 0, 0)));
        assert_eq!(checker.parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(checker.parse_version("2.10.15"), Some((2, 10, 15)));
        assert_eq!(checker.parse_version("invalid"), None);
        assert_eq!(checker.parse_version("1.0"), None);
    }

    #[test]
    fn test_version_comparison() {
        let mut checker = UpdateChecker::new();
        checker.current_version = "1.0.0".to_string();

        assert!(checker.is_newer_version("1.0.1"));
        assert!(checker.is_newer_version("1.1.0"));
        assert!(checker.is_newer_version("2.0.0"));
        assert!(!checker.is_newer_version("1.0.0"));
        assert!(!checker.is_newer_version("0.9.9"));
    }

    #[tokio::test]
    async fn test_update_check() {
        let mut checker = UpdateChecker::new();
        let result = checker.check_for_updates().await;

        match result {
            UpdateCheckResult::NoUpdateAvailable(info) => {
                assert_eq!(info.current_version, info.latest_version);
                assert!(!info.update_available);
            }
            UpdateCheckResult::UpdateAvailable(_) => {
                // This shouldn't happen with the mock implementation
                panic!("Unexpected update available in test");
            }
            UpdateCheckResult::CheckFailed(_) => {
                // This could happen if network is unavailable, which is fine for tests
            }
        }
    }

    #[test]
    fn test_check_interval() {
        let mut checker = UpdateChecker::new();
        let short_interval = Duration::from_secs(1);
        checker.set_check_interval(short_interval);

        // Simulate a check
        checker.last_check = Some(SystemTime::now() - Duration::from_secs(2));

        // Should indicate a check is needed
        assert!(checker.time_since_last_check().unwrap() > short_interval);
    }

    #[test]
    fn test_version_info_serialization() {
        let version_info = VersionInfo {
            current_version: "1.0.0".to_string(),
            latest_version: "1.1.0".to_string(),
            update_available: true,
            release_notes: Some("Bug fixes and improvements".to_string()),
            download_url: Some("https://github.com/example/releases".to_string()),
            last_checked: 1700000000,
        };

        let json = serde_json::to_string(&version_info).unwrap();
        let deserialized: VersionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(version_info.current_version, deserialized.current_version);
        assert_eq!(version_info.latest_version, deserialized.latest_version);
        assert_eq!(version_info.update_available, deserialized.update_available);
    }
}
