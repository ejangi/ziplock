//! Linux-specific platform integration for ZipLock desktop application

use super::PlatformIntegration;
use std::process::Command;

/// Linux platform integration implementation
pub struct LinuxIntegration {
    desktop_environment: String,
}

impl LinuxIntegration {
    /// Create a new Linux integration instance
    pub fn new() -> Result<Self, String> {
        let desktop_environment = detect_desktop_environment();
        Ok(Self {
            desktop_environment,
        })
    }

    /// Get the detected desktop environment
    pub fn desktop_environment(&self) -> &str {
        &self.desktop_environment
    }

    /// Check if running on Wayland
    pub fn is_wayland(&self) -> bool {
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Check if running on X11
    pub fn is_x11(&self) -> bool {
        std::env::var("DISPLAY").is_ok()
    }
}

impl PlatformIntegration for LinuxIntegration {
    fn register_file_associations(&self) -> Result<(), String> {
        tracing::info!("Registering Linux file associations for .7z and .zip files");

        // Update desktop database to register our .desktop file
        let status = Command::new("update-desktop-database")
            .arg("-q")
            .arg(&format!(
                "{}/.local/share/applications",
                std::env::var("HOME").unwrap_or_default()
            ))
            .status();

        match status {
            Ok(exit_status) if exit_status.success() => {
                tracing::info!("Desktop database updated successfully");
            }
            Ok(_) => {
                tracing::warning!("update-desktop-database failed, but continuing");
            }
            Err(e) => {
                tracing::warning!("Failed to run update-desktop-database: {}", e);
                // Don't fail entirely, as this is not critical
            }
        }

        // Set ZipLock as default handler for .7z files if xdg-mime is available
        if Command::new("which")
            .arg("xdg-mime")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            let _ = Command::new("xdg-mime")
                .args(&["default", "ziplock.desktop", "application/x-7z-compressed"])
                .status();
        }

        Ok(())
    }

    fn setup_system_tray(&self) -> Result<(), String> {
        tracing::info!("Setting up Linux system tray integration");

        // Check if system tray is available
        match &*self.desktop_environment {
            "GNOME" => {
                tracing::info!("GNOME detected - using notification area");
                // GNOME Shell doesn't have a traditional system tray
                // Could integrate with notifications instead
            }
            "KDE" => {
                tracing::info!("KDE detected - using system tray");
                // KDE has full system tray support
            }
            "XFCE" => {
                tracing::info!("XFCE detected - using system tray");
                // XFCE has system tray support
            }
            _ => {
                tracing::info!("Generic Linux desktop - attempting system tray");
            }
        }

        // For now, we'll defer actual system tray implementation
        // This would typically use a library like `tray-icon` or similar
        Ok(())
    }

    fn get_native_theme(&self) -> String {
        // Try to detect dark mode preference
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if theme.contains("dark") || theme.contains("Dark") {
                return "dark".to_string();
            }
        }

        // Check gsettings for GNOME
        if self.desktop_environment == "GNOME" {
            if let Ok(output) = Command::new("gsettings")
                .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                .output()
            {
                if let Ok(theme) = String::from_utf8(output.stdout) {
                    if theme.to_lowercase().contains("dark") {
                        return "dark".to_string();
                    }
                }
            }
        }

        // Check for KDE dark mode
        if self.desktop_environment == "KDE" {
            if let Ok(output) = Command::new("kreadconfig5")
                .args(&["--group", "Colors:Window", "--key", "BackgroundNormal"])
                .output()
            {
                if let Ok(color) = String::from_utf8(output.stdout) {
                    // Simple heuristic: if background is dark (low RGB values)
                    if color.starts_with("0,0,0") || color.starts_with("1,1,1") {
                        return "dark".to_string();
                    }
                }
            }
        }

        // Default to system theme detection
        "system".to_string()
    }

    fn open_with_system(&self, path: &str) -> Result<(), String> {
        tracing::info!("Opening '{}' with system default handler", path);

        // Try xdg-open first (most common)
        if Command::new("xdg-open")
            .arg(path)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Ok(());
        }

        // Fallback to desktop environment specific commands
        let commands = match &*self.desktop_environment {
            "GNOME" => vec!["gnome-open", "gio open"],
            "KDE" => vec!["kde-open", "kioclient exec"],
            "XFCE" => vec!["exo-open", "thunar"],
            _ => vec!["open", "firefox", "chromium"],
        };

        for cmd in commands {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mut command = Command::new(parts[0]);
            if parts.len() > 1 {
                command.args(&parts[1..]);
            }
            command.arg(path);

            if command.status().map(|s| s.success()).unwrap_or(false) {
                return Ok(());
            }
        }

        Err(format!("Failed to open '{}' with system handler", path))
    }

    fn show_file_picker(
        &self,
        title: &str,
        filters: &[(&str, &[&str])],
    ) -> Result<Option<String>, String> {
        tracing::info!("Showing Linux file picker: '{}'", title);

        // Try different file picker dialogs in order of preference

        // 1. Try native GTK file picker via zenity
        if let Ok(output) = Command::new("zenity")
            .arg("--file-selection")
            .arg("--title")
            .arg(title)
            .args(build_zenity_filters(filters))
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
        }

        // 2. Try KDE file picker
        if self.desktop_environment == "KDE" {
            if let Ok(output) = Command::new("kdialog")
                .arg("--getopenfilename")
                .arg(".")
                .arg(&build_kdialog_filter(filters))
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(Some(path));
                    }
                }
            }
        }

        // 3. Fallback to simple directory listing (not ideal, but works)
        tracing::warning!(
            "No native file picker available, user will need to specify path manually"
        );

        Ok(None)
    }
}

/// Detect the desktop environment
fn detect_desktop_environment() -> String {
    // Check various environment variables to determine DE
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        return de.to_uppercase();
    }

    if let Ok(de) = std::env::var("DESKTOP_SESSION") {
        return de.to_uppercase();
    }

    if std::env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return "GNOME".to_string();
    }

    if std::env::var("KDE_FULL_SESSION").is_ok() {
        return "KDE".to_string();
    }

    if std::env::var("XFCE4_SESSION").is_ok() {
        return "XFCE".to_string();
    }

    // Check for running processes as a fallback
    if Command::new("pgrep")
        .arg("gnome-session")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return "GNOME".to_string();
    }

    if Command::new("pgrep")
        .arg("ksmserver")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return "KDE".to_string();
    }

    "UNKNOWN".to_string()
}

/// Build zenity file filters
fn build_zenity_filters(filters: &[(&str, &[&str])]) -> Vec<String> {
    let mut args = Vec::new();

    for (name, extensions) in filters {
        let pattern = extensions
            .iter()
            .map(|ext| format!("*.{}", ext))
            .collect::<Vec<_>>()
            .join(" ");

        args.push("--file-filter".to_string());
        args.push(format!("{} | {}", name, pattern));
    }

    // Add "All files" filter
    args.push("--file-filter".to_string());
    args.push("All files | *".to_string());

    args
}

/// Build KDE dialog file filter
fn build_kdialog_filter(filters: &[(&str, &[&str])]) -> String {
    let mut filter_parts = Vec::new();

    for (name, extensions) in filters {
        let pattern = extensions
            .iter()
            .map(|ext| format!("*.{}", ext))
            .collect::<Vec<_>>()
            .join(" ");
        filter_parts.push(format!("{} ({})", name, pattern));
    }

    filter_parts.push("All files (*)".to_string());
    filter_parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_integration_creation() {
        let integration = LinuxIntegration::new().unwrap();
        assert!(!integration.desktop_environment().is_empty());
    }

    #[test]
    fn test_display_detection() {
        let integration = LinuxIntegration::new().unwrap();
        // At least one should be true in a normal Linux environment
        let has_display = integration.is_wayland() || integration.is_x11();
        // Note: This might fail in headless CI environments
        if std::env::var("CI").is_err() {
            assert!(has_display);
        }
    }

    #[test]
    fn test_zenity_filters() {
        let filters = [
            ("Archives", &["7z", "zip"][..]),
            ("Images", &["png", "jpg"][..]),
        ];
        let args = build_zenity_filters(&filters);
        assert!(args.len() >= 4); // At least 2 filters + "All files"
        assert!(args.contains(&"--file-filter".to_string()));
    }

    #[test]
    fn test_kdialog_filter() {
        let filters = [("Archives", &["7z", "zip"][..])];
        let filter = build_kdialog_filter(&filters);
        assert!(filter.contains("Archives"));
        assert!(filter.contains("*.7z"));
        assert!(filter.contains("All files"));
    }
}
