//! macOS-specific platform integration for ZipLock desktop application

use super::PlatformIntegration;
use std::process::Command;

/// macOS platform integration implementation
pub struct MacOSIntegration {
    bundle_id: String,
}

impl MacOSIntegration {
    /// Create a new macOS integration instance
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            bundle_id: "com.ziplock.passwordmanager".to_string(),
        })
    }

    /// Get the bundle identifier used for macOS integration
    pub fn bundle_id(&self) -> &str {
        &self.bundle_id
    }

    /// Check if running on Apple Silicon (M1/M2)
    pub fn is_apple_silicon(&self) -> bool {
        if let Ok(output) = Command::new("uname").arg("-m").output() {
            let arch = String::from_utf8_lossy(&output.stdout);
            return arch.trim() == "arm64";
        }
        false
    }

    /// Get macOS version information
    pub fn macos_version(&self) -> String {
        if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Check if SIP (System Integrity Protection) is enabled
    pub fn is_sip_enabled(&self) -> bool {
        if let Ok(output) = Command::new("csrutil").arg("status").output() {
            let status = String::from_utf8_lossy(&output.stdout);
            return status.contains("enabled");
        }
        true // Assume enabled if we can't determine
    }
}

impl PlatformIntegration for MacOSIntegration {
    fn register_file_associations(&self) -> Result<(), String> {
        tracing::info!("Registering macOS file associations for .7z and .zip files");

        // In a real implementation, this would:
        // 1. Use Launch Services to register file type associations
        // 2. Update the Info.plist if we're in an app bundle
        // 3. Call LSRegisterURL to register the application
        // 4. Use LSSetDefaultHandlerForURLScheme for URL schemes

        // For now, we'll use lsregister to refresh Launch Services database
        let result = Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
            .args(&["-f", "-R", "/Applications/ZipLock.app"])
            .status();

        match result {
            Ok(status) if status.success() => {
                tracing::info!("Launch Services database updated successfully");
            }
            Ok(_) => {
                tracing::warning!("lsregister command failed, but continuing");
            }
            Err(e) => {
                tracing::warning!("Failed to run lsregister: {}", e);
                // Don't fail entirely, as this is not critical
            }
        }

        // Try to set ZipLock as default for .7z files using duti if available
        if Command::new("which")
            .arg("duti")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            let _ = Command::new("duti")
                .args(&["-s", &self.bundle_id, "org.7-zip.7-zip-archive", "all"])
                .status();

            tracing::info!("Attempted to set default handler using duti");
        }

        Ok(())
    }

    fn setup_system_tray(&self) -> Result<(), String> {
        tracing::info!("Setting up macOS menu bar integration");

        // In a real implementation, this would:
        // 1. Create an NSStatusItem for the menu bar
        // 2. Set up the menu bar icon and menu
        // 3. Handle menu item clicks
        // 4. Support system notifications via NSUserNotificationCenter

        // macOS uses menu bar items instead of system tray
        tracing::info!("macOS menu bar setup completed (placeholder)");

        Ok(())
    }

    fn get_native_theme(&self) -> String {
        // Check macOS appearance preference
        if let Ok(output) = Command::new("defaults")
            .args(&["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let style = String::from_utf8_lossy(&output.stdout);
            if style.trim().eq_ignore_ascii_case("dark") {
                return "dark".to_string();
            }
        }

        // Check if system is set to auto (dark mode at night)
        if let Ok(output) = Command::new("defaults")
            .args(&["read", "-g", "AppleInterfaceStyleSwitchesAutomatically"])
            .output()
        {
            let auto_switch = String::from_utf8_lossy(&output.stdout);
            if auto_switch.trim() == "1" {
                return "system".to_string();
            }
        }

        // Default to light theme
        "light".to_string()
    }

    fn open_with_system(&self, path: &str) -> Result<(), String> {
        tracing::info!("Opening '{}' with macOS system default handler", path);

        // Use macOS 'open' command
        let status = Command::new("open")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to execute open command: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            // Try with specific applications as fallback
            let fallback_apps = ["Finder", "Archive Utility", "The Unarchiver"];

            for app in &fallback_apps {
                if let Ok(status) = Command::new("open").args(&["-a", app, path]).status() {
                    if status.success() {
                        tracing::info!("Opened '{}' with {}", path, app);
                        return Ok(());
                    }
                }
            }

            Err(format!("Failed to open '{}' with system handler", path))
        }
    }

    fn show_file_picker(
        &self,
        title: &str,
        filters: &[(&str, &[&str])],
    ) -> Result<Option<String>, String> {
        tracing::info!("Showing macOS file picker: '{}'", title);

        // Try different approaches for file selection

        // 1. Use osascript (AppleScript) for native file dialog
        let script = build_applescript_file_dialog(title, filters);

        if let Ok(output) = Command::new("osascript").args(&["-e", &script]).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() && !path.contains("User canceled") {
                    return Ok(Some(path));
                }
            }
        }

        // 2. Fallback to command line file selection
        tracing::warning!("AppleScript file dialog failed, using command line fallback");

        // Simple fallback - could be enhanced with a TUI file browser
        if let Ok(home) = std::env::var("HOME") {
            let default_path = format!("{}/Documents", home);
            tracing::info!("Consider using file picker in: {}", default_path);
        }

        Ok(None)
    }
}

impl MacOSIntegration {
    /// Register URL scheme handler for macOS
    pub fn register_url_scheme(&self, scheme: &str) -> Result<(), String> {
        tracing::info!("Registering URL scheme '{}' for macOS", scheme);

        // In a real implementation, this would use Launch Services APIs
        // For now, we'll use defaults to attempt registration
        let plist_path = format!("/Applications/ZipLock.app/Contents/Info.plist");

        if std::path::Path::new(&plist_path).exists() {
            tracing::info!("Found app bundle, URL scheme should be registered via Info.plist");
        } else {
            tracing::warning!("App bundle not found, URL scheme registration skipped");
        }

        Ok(())
    }

    /// Check if the app is running in sandbox mode
    pub fn is_sandboxed(&self) -> bool {
        // Check if we're running in a sandbox by trying to access restricted areas
        std::env::var("APP_SANDBOX_CONTAINER_ID").is_ok()
    }

    /// Get the path to the app's container directory
    pub fn container_path(&self) -> Option<String> {
        if let Ok(home) = std::env::var("HOME") {
            let container_path = format!("{}/Library/Containers/{}", home, self.bundle_id);
            if std::path::Path::new(&container_path).exists() {
                return Some(container_path);
            }
        }
        None
    }
}

/// Build AppleScript for native file dialog
fn build_applescript_file_dialog(title: &str, filters: &[(&str, &[&str])]) -> String {
    let mut script = format!(
        r#"set chosenFile to choose file with prompt "{}" "#,
        title.replace("\"", "\\\"")
    );

    if !filters.is_empty() {
        let types: Vec<String> = filters
            .iter()
            .flat_map(|(_, extensions)| extensions.iter().map(|ext| format!("\"{}\"", ext)))
            .collect();

        if !types.is_empty() {
            script.push_str(&format!("of type {{{}}}", types.join(", ")));
        }
    }

    script.push_str("\nPOSIX path of chosenFile");
    script
}

/// Build file type filters for macOS UTI (Uniform Type Identifiers)
fn build_macos_file_types(filters: &[(&str, &[&str])]) -> Vec<String> {
    let mut types = Vec::new();

    for (_, extensions) in filters {
        for ext in *extensions {
            match *ext {
                "7z" => types.push("org.7-zip.7-zip-archive".to_string()),
                "zip" => types.push("com.pkware.zip-archive".to_string()),
                "pdf" => types.push("com.adobe.pdf".to_string()),
                "png" => types.push("public.png".to_string()),
                "jpg" | "jpeg" => types.push("public.jpeg".to_string()),
                _ => types.push(format!("public.filename-extension.{}", ext)),
            }
        }
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_integration_creation() {
        let integration = MacOSIntegration::new().unwrap();
        assert!(!integration.bundle_id().is_empty());
        assert_eq!(integration.bundle_id(), "com.ziplock.passwordmanager");
    }

    #[test]
    fn test_applescript_generation() {
        let filters = [("Archives", &["7z", "zip"][..])];
        let script = build_applescript_file_dialog("Select Archive", &filters);
        assert!(script.contains("Select Archive"));
        assert!(script.contains("\"7z\""));
        assert!(script.contains("\"zip\""));
    }

    #[test]
    fn test_file_types_mapping() {
        let filters = [("Archives", &["7z", "zip"][..])];
        let types = build_macos_file_types(&filters);
        assert!(types.contains(&"org.7-zip.7-zip-archive".to_string()));
        assert!(types.contains(&"com.pkware.zip-archive".to_string()));
    }

    #[test]
    fn test_macos_version() {
        let integration = MacOSIntegration::new().unwrap();
        let version = integration.macos_version();
        assert!(!version.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_apple_silicon_detection() {
        let integration = MacOSIntegration::new().unwrap();
        // This test will pass on both Intel and Apple Silicon Macs
        let _is_arm = integration.is_apple_silicon();
        // Just ensure the method doesn't panic
    }
}
