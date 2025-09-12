//! Windows-specific platform integration for ZipLock desktop application

use super::PlatformIntegration;
use std::process::Command;

#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    System::Registry::{RegCreateKeyExW, RegSetValueExW, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER},
    UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST},
};

/// Windows platform integration implementation
pub struct WindowsIntegration {
    app_id: String,
}

impl WindowsIntegration {
    /// Create a new Windows integration instance
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            app_id: "ZipLock.PasswordManager".to_string(),
        })
    }

    /// Get the application ID used for Windows integration
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Check if running as administrator
    pub fn is_admin(&self) -> bool {
        // Simple check - in production this would use Windows APIs
        std::env::var("USERNAME").unwrap_or_default() == "Administrator"
    }

    /// Get Windows version information
    pub fn windows_version(&self) -> String {
        // This would use GetVersionEx or similar APIs in production
        "Windows".to_string()
    }
}

impl PlatformIntegration for WindowsIntegration {
    fn register_file_associations(&self) -> Result<(), String> {
        tracing::info!("Registering Windows file associations for .7z and .zip files");

        // In a real implementation, this would:
        // 1. Write registry entries for file associations
        // 2. Set ZipLock as the default handler for .7z files
        // 3. Add context menu entries
        // 4. Notify the shell of changes

        #[cfg(windows)]
        {
            self.register_7z_association()
                .map_err(|e| format!("Failed to register .7z association: {}", e))?;

            self.register_context_menu()
                .map_err(|e| format!("Failed to register context menu: {}", e))?;

            // Notify Windows shell of association changes
            unsafe {
                SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
            }

            tracing::info!("Windows file associations registered successfully");
        }

        #[cfg(not(windows))]
        {
            tracing::warning!("Windows file associations not supported on this platform");
        }

        Ok(())
    }

    fn setup_system_tray(&self) -> Result<(), String> {
        tracing::info!("Setting up Windows system tray integration");

        // In a real implementation, this would:
        // 1. Create a system tray icon
        // 2. Set up the context menu
        // 3. Handle tray icon events
        // 4. Support balloon notifications

        // For now, we'll just log that it's not yet implemented
        tracing::info!("Windows system tray setup completed (placeholder)");

        Ok(())
    }

    fn get_native_theme(&self) -> String {
        // Check Windows theme preference
        // This would query the registry in a real implementation:
        // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize

        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("reg")
                .args(&[
                    "query",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                    "/v",
                    "AppsUseLightTheme",
                ])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("0x0") {
                    return "dark".to_string();
                } else if output_str.contains("0x1") {
                    return "light".to_string();
                }
            }
        }

        "system".to_string()
    }

    fn open_with_system(&self, path: &str) -> Result<(), String> {
        tracing::info!("Opening '{}' with Windows system default handler", path);

        // Use Windows 'start' command to open files
        let status = Command::new("cmd")
            .args(&["/C", "start", "", path])
            .status()
            .map_err(|e| format!("Failed to execute start command: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to open '{}' with system handler", path))
        }
    }

    fn show_file_picker(
        &self,
        title: &str,
        filters: &[(&str, &[&str])],
    ) -> Result<Option<String>, String> {
        tracing::info!("Showing Windows file picker: '{}'", title);

        // In a real implementation, this would use the Windows Common File Dialog
        // or the rfd crate which provides native file dialogs

        // For now, we'll try PowerShell as a fallback
        let filter_str = build_powershell_filter(filters);

        let script = format!(
            r#"
            Add-Type -AssemblyName System.Windows.Forms
            $dialog = New-Object System.Windows.Forms.OpenFileDialog
            $dialog.Title = '{}'
            $dialog.Filter = '{}'
            $result = $dialog.ShowDialog()
            if ($result -eq [System.Windows.Forms.DialogResult]::OK) {{
                Write-Output $dialog.FileName
            }}
            "#,
            title, filter_str
        );

        if let Ok(output) = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &script])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
        }

        // Fallback: return None to indicate no selection
        Ok(None)
    }
}

#[cfg(windows)]
impl WindowsIntegration {
    /// Register .7z file association in Windows registry
    fn register_7z_association(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simplified version - production code would need proper error handling
        // and should use the Windows API directly rather than reg.exe

        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy();

        // Register the ProgID
        Command::new("reg")
            .args(&[
                "add",
                r"HKCR\ZipLock.Archive",
                "/ve",
                "/d",
                "ZipLock Password Archive",
                "/f",
            ])
            .status()?;

        // Set the icon
        Command::new("reg")
            .args(&[
                "add",
                r"HKCR\ZipLock.Archive\DefaultIcon",
                "/ve",
                "/d",
                &format!("{},0", exe_path_str),
                "/f",
            ])
            .status()?;

        // Set the open command
        Command::new("reg")
            .args(&[
                "add",
                r"HKCR\ZipLock.Archive\shell\open\command",
                "/ve",
                "/d",
                &format!("\"{}\" \"%1\"", exe_path_str),
                "/f",
            ])
            .status()?;

        // Associate .7z extension with our ProgID
        Command::new("reg")
            .args(&[
                "add",
                r"HKCR\.7z\OpenWithProgids",
                "/v",
                "ZipLock.Archive",
                "/d",
                "",
                "/f",
            ])
            .status()?;

        Ok(())
    }

    /// Register context menu entries
    fn register_context_menu(&self) -> Result<(), Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy();

        // Add "Open with ZipLock" to .7z files context menu
        Command::new("reg")
            .args(&[
                "add",
                r"HKCR\.7z\shell\ZipLock",
                "/ve",
                "/d",
                "Open with ZipLock",
                "/f",
            ])
            .status()?;

        Command::new("reg")
            .args(&[
                "add",
                r"HKCR\.7z\shell\ZipLock\command",
                "/ve",
                "/d",
                &format!("\"{}\" \"%1\"", exe_path_str),
                "/f",
            ])
            .status()?;

        Ok(())
    }
}

/// Build PowerShell file dialog filter string
fn build_powershell_filter(filters: &[(&str, &[&str])]) -> String {
    let mut filter_parts = Vec::new();

    for (name, extensions) in filters {
        let pattern = extensions
            .iter()
            .map(|ext| format!("*.{}", ext))
            .collect::<Vec<_>>()
            .join(";");
        filter_parts.push(format!("{} ({})|{}", name, pattern, pattern));
    }

    filter_parts.push("All files (*.*)|*.*".to_string());
    filter_parts.join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_integration_creation() {
        let integration = WindowsIntegration::new().unwrap();
        assert!(!integration.app_id().is_empty());
        assert_eq!(integration.app_id(), "ZipLock.PasswordManager");
    }

    #[test]
    fn test_powershell_filter() {
        let filters = [
            ("Archives", &["7z", "zip"][..]),
            ("Images", &["png", "jpg"][..]),
        ];
        let filter = build_powershell_filter(&filters);
        assert!(filter.contains("Archives"));
        assert!(filter.contains("*.7z"));
        assert!(filter.contains("All files"));
    }

    #[test]
    fn test_windows_version() {
        let integration = WindowsIntegration::new().unwrap();
        let version = integration.windows_version();
        assert!(!version.is_empty());
    }
}
