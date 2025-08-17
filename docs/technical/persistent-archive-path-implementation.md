# ZipLock Persistent Archive Path Implementation

## Overview

This document describes the implementation of persistent archive path memory in ZipLock, enabling users to quickly reopen their most recently used archive with just their passphrase, eliminating the need to re-select the file path each time.

## Feature Requirements

Based on the user request: "ensure that once a user has selected an archive file to open that the app keeps a persistent memory of the file path so that each subsequent time the app is opened the user only needs to enter their passphrase and click 'Open'."

### Key Behaviors
1. **Remember Last Archive**: App automatically saves the path of successfully opened archives
2. **Auto-Open Flow**: On subsequent launches, show quick-open interface for last used archive
3. **Seamless Experience**: Users only need to enter passphrase, not re-select file
4. **Fallback Option**: Users can still choose to open different archives
5. **Cross-Platform**: Consistent behavior across Linux and Android platforms

## Implementation Architecture

### Linux Implementation

#### Configuration Manager Enhancement
The existing `ConfigManager` already had the required functionality but wasn't being utilized:

```rust
// apps/linux/src/config.rs
impl ConfigManager {
    // Method was marked as dead_code but is now active
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
}
```

#### Startup Flow Modification
Modified the main application startup logic to prioritize the most recently used repository:

```rust
// apps/linux/src/main.rs - Message::ConfigReady handler
// Check for most recently used repository first
if let Some(most_recent_path) = config_manager.get_most_recent_accessible_repository() {
    info!("Auto-opening most recently used repository: {:?}", most_recent_path);
    let open_view = OpenRepositoryView::with_repository(most_recent_path.clone());
    self.state = AppState::OpenRepositoryActive(open_view);
    self.config_manager = Some(config_manager);
    return Command::none();
}
```

#### Existing Persistence Infrastructure
The Linux implementation leverages the existing robust configuration system:

- **Repository Path Storage**: `config.repository.path`
- **Recent Repositories**: `config.repository.recent_repositories` with metadata
- **Auto-Detection**: `config.repository.auto_detect` capability
- **YAML Persistence**: Configuration saved to `~/.config/ziplock/config.yaml`

### Android Implementation

#### New Configuration Manager
Created a dedicated Android configuration manager using SharedPreferences:

```kotlin
// apps/mobile/android/.../config/AndroidConfigManager.kt
class AndroidConfigManager(private val context: Context) {
    private val sharedPreferences = context.getSharedPreferences("ziplock_config", Context.MODE_PRIVATE)
    
    fun setLastArchivePath(archivePath: String)
    fun getLastOpenedArchivePath(): String?
    fun hasValidLastArchive(): Boolean
    fun clearLastArchivePath()
}
```

#### RepositoryViewModel Integration
Enhanced the existing RepositoryViewModel to use the configuration manager:

```kotlin
// apps/mobile/android/.../viewmodel/RepositoryViewModel.kt
class RepositoryViewModel(private val context: Context) : ViewModel() {
    private val configManager: AndroidConfigManager = AndroidConfigManager(context)
    
    fun openRepository(filePath: String, passphrase: String) {
        // ... existing logic ...
        
        // Save the successfully opened archive path
        configManager.setLastArchivePath(filePath)
    }
}
```

#### Auto-Open UI Screen
Created a new screen for the auto-open experience:

```kotlin
// apps/mobile/android/.../MainActivity.kt
@Composable
fun AutoOpenArchiveScreen(
    repositoryViewModel: RepositoryViewModel,
    onArchiveOpened: (String) -> Unit,
    onSelectDifferent: () -> Unit,
    modifier: Modifier = Modifier
) {
    // Shows "Welcome Back" interface with:
    // - Archive file name display
    // - Passphrase input field
    // - "Open Archive" button
    // - "Choose Different Archive" option
}
```

#### Enhanced MainActivity Flow
Modified the main app flow to detect and handle auto-open scenarios:

```kotlin
fun MainApp(repositoryViewModel: RepositoryViewModel) {
    val initialScreen = if (repositoryViewModel.hasValidLastArchive()) {
        Screen.AutoOpenLastArchive
    } else {
        Screen.RepositorySelection
    }
    
    when (currentScreen) {
        Screen.AutoOpenLastArchive -> AutoOpenArchiveScreen(...)
        Screen.RepositorySelection -> RepositorySelectionScreen(...)
        // ... other screens
    }
}
```

## User Experience Flow

### First-Time Usage
1. User launches ZipLock
2. App shows repository selection screen (no previous archive)
3. User selects archive file and enters passphrase
4. Archive opens successfully
5. **Archive path is automatically saved**

### Subsequent Usage
1. User launches ZipLock
2. **App detects last used archive and shows "Welcome Back" screen**
3. App displays archive filename and passphrase field
4. User enters passphrase and clicks "Open Archive"
5. Archive opens immediately (no file selection needed)

### Alternative Flow
- User can click "Choose Different Archive" to browse for different file
- This maintains access to full file selection functionality when needed

## Cloud Storage Support

### Android Cloud Storage Compatibility
The implementation works seamlessly with cloud storage services:

```kotlin
private fun isFileAccessible(path: String): Boolean {
    return try {
        when {
            path.startsWith("content://") -> {
                // Handle Android content URIs (Google Drive, Dropbox, etc.)
                val uri = android.net.Uri.parse(path)
                context.contentResolver.openInputStream(uri)?.use { true } ?: false
            }
            else -> {
                // Handle regular file paths
                File(path).exists() && File(path).canRead()
            }
        }
    } catch (e: Exception) {
        false
    }
}
```

### Cloud Storage Patterns Supported
- Google Drive: `content://com.google.android.apps.docs/...`
- Dropbox: `content://com.dropbox.android/...`
- OneDrive: `content://com.microsoft.skydrive/...`
- Local storage: Regular file system paths

## Configuration Persistence

### Linux Configuration
```yaml
# ~/.config/ziplock/config.yaml
repository:
  path: "/home/user/Documents/passwords.7z"
  recent_repositories:
    - path: "/home/user/Documents/passwords.7z"
      last_accessed: "2024-01-15T10:30:00Z"
      display_name: "My Passwords"
      pinned: false
ui:
  show_wizard_on_startup: true
```

### Android Configuration
```kotlin
// SharedPreferences: "ziplock_config"
{
    "last_archive_path": "content://com.google.android.apps.docs/document/123",
    "last_archive_accessed": 1705312200000,
    "show_wizard_on_startup": true,
    "auto_lock_timeout": 15,
    "theme": "system"
}
```

## Error Handling and Edge Cases

### File Accessibility Validation
Both platforms validate file accessibility before showing auto-open:

1. **File Existence**: Check if file still exists at saved path
2. **Permission Check**: Verify app can read the file
3. **Cloud URI Validation**: Test content URI accessibility (Android)
4. **Graceful Fallback**: Show repository selection if validation fails

### Security Considerations
- **No Passphrase Storage**: Only file paths are persisted, never passphrases
- **App-Private Storage**: Configuration stored in app-private directories
- **URI Validation**: Cloud URIs validated before access attempts
- **Automatic Cleanup**: Invalid paths automatically removed from recent list

## Testing

### Linux Testing
Existing configuration system tests cover the persistence functionality:
```rust
#[test]
fn test_repository_manager_recent_list() {
    // Tests recent repository tracking and persistence
}
```

### Android Testing
Comprehensive test suite for AndroidConfigManager:
```kotlin
// apps/mobile/android/.../test/.../AndroidConfigManagerTest.kt
@Test
fun `setLastArchivePath should persist archive path`()

@Test
fun `hasValidLastArchive should return false for non-existent file`()

@Test
fun `configuration persistence should survive manager recreation`()
```

## Migration and Backward Compatibility

### Linux Migration
- **No Migration Required**: Leverages existing configuration infrastructure
- **Backward Compatible**: Existing installs automatically gain the feature
- **Configuration Preservation**: All existing settings remain unchanged

### Android Migration
- **Automatic Initialization**: New installs start with default configuration
- **Graceful Degradation**: Apps without saved archives show normal selection screen
- **No Data Loss**: Existing app data remains unaffected

## Performance Impact

### Linux Performance
- **Minimal Overhead**: Single path existence check on startup
- **Cached Results**: Configuration loaded once and cached
- **No Network I/O**: Local file operations only

### Android Performance
- **SharedPreferences Efficiency**: Fast key-value storage access
- **Lazy Loading**: Configuration loaded only when needed
- **URI Validation Optimization**: Content resolver queries cached

## Future Enhancements

### Potential Improvements
1. **Multiple Archive Support**: Quick-switch between recent archives
2. **Archive Metadata**: Store archive size, entry count, last modified
3. **Cloud Sync Status**: Indicate cloud sync status for cloud-stored archives
4. **Biometric Integration**: Use biometric authentication for quick access
5. **Archive Health Monitoring**: Background validation of saved archive paths

### Cross-Platform Enhancements
1. **Shared Configuration Format**: Sync settings between Linux and Android
2. **Cloud Configuration Sync**: Store preferences in cloud for multi-device use
3. **Import/Export Settings**: Allow configuration backup and restore

## Conclusion

The persistent archive path implementation successfully addresses the user requirement by:

✅ **Automatically remembering** the last opened archive file path
✅ **Eliminating repeated file selection** for returning users  
✅ **Providing seamless access** with just passphrase entry
✅ **Maintaining security** by never storing passphrases
✅ **Supporting cloud storage** for modern file access patterns
✅ **Ensuring cross-platform consistency** between Linux and Android
✅ **Preserving user choice** with alternative archive selection options

The implementation leverages existing infrastructure where possible (Linux) and creates appropriate new components where needed (Android), ensuring a robust and maintainable solution that enhances user experience without compromising security.