//! Update Checker Module
//!
//! This module provides functionality to check for application updates
//! by querying the GitHub Releases API.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

/// GitHub API endpoint for releases
const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/ejangi/ziplock/releases";

/// Current application version (should match Cargo.toml)
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How often to check for updates automatically (24 hours)
const AUTO_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Maximum time to wait for update check response
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// GitHub Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Release tag name (e.g., "v1.0.0")
    pub tag_name: String,
    /// Release name/title
    pub name: String,
    /// Release description/body (markdown)
    pub body: String,
    /// URL to the release page
    pub html_url: String,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Whether this is a draft
    pub draft: bool,
    /// Release publication date
    pub published_at: Option<DateTime<Utc>>,
    /// Release assets (download files)
    pub assets: Vec<ReleaseAsset>,
}

/// Release asset information (downloadable files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    /// Asset name (filename)
    pub name: String,
    /// Download URL
    pub browser_download_url: String,
    /// File size in bytes
    pub size: u64,
    /// Content type
    pub content_type: String,
}

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    /// Whether an update is available
    pub update_available: bool,
    /// Current version
    pub current_version: String,
    /// Latest available version
    pub latest_version: Option<String>,
    /// Latest release information
    pub latest_release: Option<ReleaseInfo>,
    /// Installation method detected
    pub installation_method: InstallationMethod,
    /// When this check was performed
    pub checked_at: DateTime<Utc>,
    /// Error message if check failed
    pub error: Option<String>,
}

/// Detected installation method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallationMethod {
    /// Installed via .deb package
    DebianPackage,
    /// Installed via AUR (Arch User Repository)
    ArchAUR,
    /// Installed from source/manually
    Manual,
    /// Unknown installation method
    Unknown,
}

impl InstallationMethod {
    /// Get update instructions for this installation method
    pub fn update_instructions(&self, version: &str) -> String {
        match self {
            InstallationMethod::DebianPackage => format!(
                "To update ZipLock:\n\
                1. Download the latest .deb package:\n\
                   wget https://github.com/ejangi/ziplock/releases/download/v{}/ziplock_{}_amd64.deb\n\
                2. Install the update:\n\
                   sudo dpkg -i ziplock_{}_amd64.deb\n\
                3. Fix any dependencies if needed:\n\
                   sudo apt-get install -f",
                version, version, version
            ),
            InstallationMethod::ArchAUR => "To update ZipLock:\n\
                1. Update via your AUR helper:\n\
                   yay -Syu ziplock\n\
                   # or\n\
                   paru -Syu ziplock\n\
                2. Or manually:\n\
                   git clone https://aur.archlinux.org/ziplock.git\n\
                   cd ziplock\n\
                   makepkg -si".to_string(),
            InstallationMethod::Manual => format!(
                "To update ZipLock:\n\
                1. Download the latest source:\n\
                   git clone https://github.com/ejangi/ziplock.git\n\
                   cd ziplock\n\
                   git checkout v{}\n\
                2. Build and install:\n\
                   ./scripts/build/build-linux.sh --profile release\n\
                   sudo cp target/release/ziplock /usr/local/bin/",
                version
            ),
            InstallationMethod::Unknown => format!(
                "Update instructions:\n\
                Visit https://github.com/ejangi/ziplock/releases/tag/v{} \
                to download the latest version for your system.",
                version
            ),
        }
    }
}

/// Update checker service
#[derive(Clone)]
pub struct UpdateChecker {
    client: reqwest::Client,
    last_check: Option<DateTime<Utc>>,
}

impl UpdateChecker {
    /// Create a new update checker
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(format!("ZipLock/{}", CURRENT_VERSION))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            last_check: None,
        }
    }

    /// Check for updates manually
    pub async fn check_for_updates(&mut self) -> Result<UpdateCheckResult> {
        info!("Checking for updates...");

        let installation_method = self.detect_installation_method().await;
        debug!("Detected installation method: {:?}", installation_method);

        let result = match self.fetch_latest_release().await {
            Ok(latest_release) => {
                let current_version = CURRENT_VERSION.to_string();
                let latest_version = latest_release.tag_name.trim_start_matches('v').to_string();

                let update_available = self.is_newer_version(&latest_version, &current_version)?;

                if update_available {
                    info!(
                        "Update available: {} -> {}",
                        current_version, latest_version
                    );
                } else {
                    info!("Application is up to date ({})", current_version);
                }

                UpdateCheckResult {
                    update_available,
                    current_version,
                    latest_version: Some(latest_version),
                    latest_release: Some(latest_release),
                    installation_method,
                    checked_at: Utc::now(),
                    error: None,
                }
            }
            Err(e) => {
                error!("Failed to check for updates: {}", e);
                UpdateCheckResult {
                    update_available: false,
                    current_version: CURRENT_VERSION.to_string(),
                    latest_version: None,
                    latest_release: None,
                    installation_method,
                    checked_at: Utc::now(),
                    error: Some(e.to_string()),
                }
            }
        };

        self.last_check = Some(result.checked_at);
        Ok(result)
    }

    /// Check if automatic update check should be performed
    pub fn should_auto_check(&self) -> bool {
        match self.last_check {
            Some(last_check) => {
                let elapsed = Utc::now().signed_duration_since(last_check);
                elapsed.to_std().unwrap_or(Duration::MAX) >= AUTO_CHECK_INTERVAL
            }
            None => true, // Never checked before
        }
    }

    /// Get the time of last update check
    pub fn last_check_time(&self) -> Option<DateTime<Utc>> {
        self.last_check
    }

    /// Fetch the latest release from GitHub API
    async fn fetch_latest_release(&self) -> Result<ReleaseInfo> {
        let url = format!("{}/latest", GITHUB_RELEASES_API);

        debug!("Fetching latest release from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch release info: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("GitHub API request failed: {}", response.status()));
        }

        let release: ReleaseInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse release info: {}", e))?;

        // Skip draft releases
        if release.draft {
            return Err(anyhow!("Latest release is a draft"));
        }

        Ok(release)
    }

    /// Compare version strings to determine if new version is available
    fn is_newer_version(&self, latest: &str, current: &str) -> Result<bool> {
        // Parse semantic versions
        let latest_parts = self.parse_version(latest)?;
        let current_parts = self.parse_version(current)?;

        // Compare major, minor, patch
        for (latest_part, current_part) in latest_parts.iter().zip(current_parts.iter()) {
            match latest_part.cmp(current_part) {
                std::cmp::Ordering::Greater => return Ok(true),
                std::cmp::Ordering::Less => return Ok(false),
                std::cmp::Ordering::Equal => continue,
            }
        }

        // If all compared parts are equal, check if latest has more parts
        Ok(latest_parts.len() > current_parts.len())
    }

    /// Parse version string into numeric components
    fn parse_version(&self, version: &str) -> Result<Vec<u32>> {
        let clean_version = version.trim_start_matches('v');

        // Split by dots and parse each part
        let parts: Result<Vec<u32>, _> = clean_version
            .split('.')
            .map(|part| {
                // Handle pre-release suffixes (e.g., "1.0.0-beta.1")
                let clean_part = part.split('-').next().unwrap_or(part);
                clean_part.parse::<u32>()
            })
            .collect();

        parts.map_err(|e| anyhow!("Invalid version format '{}': {}", version, e))
    }

    /// Detect how the application was installed
    async fn detect_installation_method(&self) -> InstallationMethod {
        // Check if installed via dpkg (Debian/Ubuntu)
        if self.is_installed_via_dpkg().await {
            return InstallationMethod::DebianPackage;
        }

        // Check if installed via pacman (Arch Linux)
        if self.is_installed_via_pacman().await {
            return InstallationMethod::ArchAUR;
        }

        // Check if running from /usr/local/bin (manual install)
        if self.is_manual_installation().await {
            return InstallationMethod::Manual;
        }

        InstallationMethod::Unknown
    }

    /// Check if installed via dpkg
    async fn is_installed_via_dpkg(&self) -> bool {
        match tokio::process::Command::new("dpkg")
            .args(["-l", "ziplock"])
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Check if installed via pacman
    async fn is_installed_via_pacman(&self) -> bool {
        match tokio::process::Command::new("pacman")
            .args(["-Q", "ziplock"])
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Check if manually installed
    async fn is_manual_installation(&self) -> bool {
        // Check if current executable is in /usr/local/bin or similar
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(path_str) = current_exe.to_str() {
                return path_str.starts_with("/usr/local/")
                    || path_str.starts_with("/opt/")
                    || path_str.contains("target/release"); // Development build
            }
        }
        false
    }
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let checker = UpdateChecker::new();

        assert_eq!(checker.parse_version("1.0.0").unwrap(), vec![1, 0, 0]);
        assert_eq!(checker.parse_version("v2.1.3").unwrap(), vec![2, 1, 3]);
        assert_eq!(checker.parse_version("0.2.5-beta").unwrap(), vec![0, 2, 5]);
    }

    #[test]
    fn test_version_comparison() {
        let checker = UpdateChecker::new();

        assert!(checker.is_newer_version("1.0.1", "1.0.0").unwrap());
        assert!(checker.is_newer_version("1.1.0", "1.0.9").unwrap());
        assert!(checker.is_newer_version("2.0.0", "1.9.9").unwrap());
        assert!(!checker.is_newer_version("1.0.0", "1.0.0").unwrap());
        assert!(!checker.is_newer_version("1.0.0", "1.0.1").unwrap());
    }

    #[test]
    fn test_installation_method_instructions() {
        let methods = vec![
            InstallationMethod::DebianPackage,
            InstallationMethod::ArchAUR,
            InstallationMethod::Manual,
            InstallationMethod::Unknown,
        ];

        for method in methods {
            let instructions = method.update_instructions("1.0.0");

            // Instructions should not be empty
            assert!(!instructions.is_empty());

            // Should contain version number (except for AUR which doesn't include specific version)
            if !matches!(method, InstallationMethod::ArchAUR) {
                assert!(
                    instructions.contains("1.0.0"),
                    "Instructions for {:?} should contain version: {}",
                    method,
                    instructions
                );
            }

            // Should contain appropriate commands based on method
            match method {
                InstallationMethod::DebianPackage => {
                    assert!(instructions.contains("dpkg"));
                    assert!(instructions.contains("wget"));
                }
                InstallationMethod::ArchAUR => {
                    assert!(instructions.contains("yay") || instructions.contains("paru"));
                }
                InstallationMethod::Manual => {
                    assert!(instructions.contains("git"));
                    assert!(instructions.contains("build"));
                }
                InstallationMethod::Unknown => {
                    assert!(instructions.contains("github.com"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_auto_check_interval() {
        let mut checker = UpdateChecker::new();

        // Should check when never checked before
        assert!(checker.should_auto_check());

        // Set last check to now
        checker.last_check = Some(Utc::now());

        // Should not check immediately after
        assert!(!checker.should_auto_check());
    }

    #[tokio::test]
    async fn test_update_check_basic() {
        let mut checker = UpdateChecker::new();

        // Attempt to check for updates
        match checker.check_for_updates().await {
            Ok(result) => {
                // Basic validation of result structure
                assert!(!result.current_version.is_empty());
                assert!(result.checked_at > Utc::now() - chrono::Duration::minutes(1));
                assert!(matches!(
                    result.installation_method,
                    InstallationMethod::DebianPackage
                        | InstallationMethod::ArchAUR
                        | InstallationMethod::Manual
                        | InstallationMethod::Unknown
                ));
            }
            Err(_) => {
                // Network errors are acceptable in CI/test environments
                println!("Update check failed (network error expected in some environments)");
            }
        }
    }

    #[test]
    fn test_version_parsing_edge_cases() {
        let checker = UpdateChecker::new();

        // Valid versions
        assert!(checker.parse_version("1.0.0").is_ok());
        assert!(checker.parse_version("v1.0.0").is_ok());
        assert!(checker.parse_version("0.2.5").is_ok());
        assert!(checker.parse_version("10.20.30").is_ok());

        // Invalid versions should fail gracefully
        assert!(checker.parse_version("invalid").is_err());
        assert!(checker.parse_version("1.0.x").is_err());
        assert!(checker.parse_version("").is_err());
    }

    #[test]
    fn test_current_version_validity() {
        let checker = UpdateChecker::new();
        // Should be able to parse the current version constant
        assert!(checker.parse_version(CURRENT_VERSION).is_ok());
    }
}
