# ZipLock File Association Guide

This document provides comprehensive documentation for the .7z file association feature in ZipLock applications, enabling users to open password archives directly from file managers, email attachments, cloud storage apps, and other sources across Android and Linux platforms.

## Overview

ZipLock automatically registers itself to handle .7z archive files through platform-specific mechanisms. When users encounter a .7z file, ZipLock appears as an option in the "Open with" dialog, providing seamless access to encrypted password archives.

## Android Implementation

### Intent Filter Registration

The Android app registers multiple intent filters in `AndroidManifest.xml` to handle various .7z file scenarios:

```xml
<!-- Intent filter for .7z files with proper MIME type -->
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <data android:mimeType="application/x-7z-compressed" />
</intent-filter>

<!-- Intent filter for .7z files with generic MIME type -->
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <data android:mimeType="application/octet-stream" />
</intent-filter>

<!-- Intent filter for .7z files without MIME type -->
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <data android:scheme="file" />
    <data android:pathPattern=".*\\.7z" />
</intent-filter>

<!-- Intent filter for content URIs -->
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <data android:scheme="content" />
    <data android:pathPattern=".*\\.7z" />
</intent-filter>
```

### Supported Android Sources

- **File Managers**: Files by Google, ES File Explorer, Solid Explorer, etc.
- **Email Attachments**: Gmail, Outlook, Yahoo Mail, etc.
- **Cloud Storage Apps**: Google Drive, Dropbox, OneDrive, Box
- **Web Browsers**: Downloaded .7z files from Chrome, Firefox, etc.
- **Messaging Apps**: WhatsApp, Telegram file sharing
- **Document Viewers**: Any app that can display or share .7z files

### Android User Experience Flow

1. User taps a .7z file in any supported app
2. Android shows "Open with" dialog
3. User selects ZipLock from the list
4. ZipLock launches and automatically navigates to repository opening screen
5. File path is stored for quick reopening
6. Error handling for corrupted or inaccessible files

### Android Implementation Details

**MainActivity Intent Handling:**
```kotlin
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Handle incoming intent
        handleIncomingIntent(intent)
    }
    
    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        intent?.let { handleIncomingIntent(it) }
    }
    
    private fun handleIncomingIntent(intent: Intent) {
        when (intent.action) {
            Intent.ACTION_VIEW -> {
                intent.data?.let { uri ->
                    val filePath = FileUtils.getUsableFilePath(this, uri)
                    if (filePath != null) {
                        // Navigate to repository opening with the file
                        navigateToRepositoryOpening(filePath)
                    } else {
                        showError("Unable to access the selected file")
                    }
                }
            }
        }
    }
}
```

**File Utils Integration:**
```kotlin
object FileUtils {
    fun getUsableFilePath(context: Context, uri: Uri): String? {
        return when (uri.scheme) {
            "file" -> uri.path
            "content" -> handleContentUri(context, uri)
            else -> null
        }
    }
    
    private fun handleContentUri(context: Context, uri: Uri): String? {
        // Handle Storage Access Framework URIs
        // Copy to app cache if necessary for cloud storage
        // Return usable file path for FFI operations
    }
}
```

## Linux Implementation

### Desktop Entry Registration

The Linux app provides a desktop entry file at `apps/linux/resources/ziplock.desktop`:

```desktop
[Desktop Entry]
Version=1.0
Type=Application
Name=ZipLock
GenericName=Password Manager
Comment=A secure, portable password manager using encrypted 7z archives
Icon=ziplock
Exec=ziplock %f
Terminal=false
StartupNotify=true
Categories=Utility;Security;Office;
Keywords=password;manager;security;encryption;vault;
MimeType=application/x-7z-compressed;
StartupWMClass=ziplock
```

**Key Elements:**
- `MimeType=application/x-7z-compressed;` registers ZipLock as a .7z file handler
- `Exec=ziplock %f` passes the file path as a command-line argument
- `Categories` ensure proper placement in application menus

### MIME Type Definition

Comprehensive MIME type definition at `apps/linux/resources/mime/packages/ziplock.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-7z-compressed">
    <comment>7z archive</comment>
    <comment xml:lang="en">7z archive</comment>
    <icon name="application-x-7z-compressed"/>
    <glob-deleteall/>
    <glob pattern="*.7z"/>
    <magic priority="50">
      <match value="7z\274\257\047\034" type="string" offset="0"/>
    </magic>
  </mime-type>
</mime-info>
```

**Features:**
- Proper MIME type definition for system recognition
- Magic number detection for reliable file type identification
- Glob pattern matching for .7z extension
- Icon association for file managers

### Linux Installation Process

**Package Installation:**
```bash
# Install desktop entry
sudo cp apps/linux/resources/ziplock.desktop /usr/share/applications/

# Install MIME type definition
sudo cp apps/linux/resources/mime/packages/ziplock.xml /usr/share/mime/packages/

# Update MIME database
sudo update-mime-database /usr/share/mime

# Update desktop database
sudo update-desktop-database /usr/share/applications
```

**Development Installation:**
```bash
# Install for current user only
cp apps/linux/resources/ziplock.desktop ~/.local/share/applications/
cp apps/linux/resources/mime/packages/ziplock.xml ~/.local/share/mime/packages/

# Update user databases
update-mime-database ~/.local/share/mime
update-desktop-database ~/.local/share/applications
```

### Linux User Experience

1. User double-clicks a .7z file in file manager (Nautilus, Dolphin, Thunar, etc.)
2. System checks registered applications for the MIME type
3. ZipLock launches automatically or appears in "Open with" dialog
4. Application receives file path as command-line argument
5. Repository opening screen loads with the specified file

### Linux Command-Line Integration

**Command-Line Handling:**
```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let file_path = &args[1];
        if file_path.ends_with(".7z") {
            // Open repository directly
            open_repository_file(file_path);
        }
    } else {
        // Launch normal application UI
        launch_application();
    }
}
```

## Cross-Platform Considerations

### Security

**Android Security:**
- Permission handling for external storage access
- Content URI validation to prevent path traversal
- Secure handling of Storage Access Framework permissions
- Malware protection through file type validation

**Linux Security:**
- File permission validation before opening
- Path sanitization for command-line arguments
- Integration with system security policies
- Sandboxing considerations for Flatpak/Snap packages

### File Handling Differences

**Android Challenges:**
- Storage Access Framework complexity
- Content URIs vs. file paths
- Cloud storage virtual files
- Permission management
- Background app restrictions

**Linux Advantages:**
- Direct file system access
- Standard file paths
- Mature MIME type system
- Consistent desktop integration
- Command-line flexibility

### Cloud Storage Integration

**Android Cloud Storage:**
- Automatic detection of cloud storage providers
- Copy-to-local strategy for safe operations
- Storage Access Framework integration
- Handling of virtual files and sync conflicts

**Linux Cloud Storage:**
- Local sync folder detection
- Standard file system operations
- Integration with cloud client applications
- FUSE filesystem support

## Testing and Validation

### Android Testing

**Manual Testing:**
```bash
# Test with file manager
adb shell am start -a android.intent.action.VIEW -d "file:///storage/emulated/0/Download/test.7z"

# Test with content URI
adb shell am start -a android.intent.action.VIEW -d "content://com.android.externalstorage.documents/document/primary%3ADownload%2Ftest.7z"

# Check intent filters
adb shell dumpsys package com.ziplock | grep -A 10 "android.intent.action.VIEW"
```

**Automated Testing:**
```kotlin
@Test
fun testFileAssociationIntent() {
    val intent = Intent(Intent.ACTION_VIEW).apply {
        data = Uri.parse("file:///path/to/test.7z")
        addCategory(Intent.CATEGORY_DEFAULT)
    }
    
    val activities = context.packageManager.queryIntentActivities(intent, 0)
    assertTrue("ZipLock should handle .7z files", 
               activities.any { it.activityInfo.packageName == "com.ziplock" })
}
```

### Linux Testing

**Manual Testing:**
```bash
# Test MIME type registration
file --mime-type test.7z
# Should return: application/x-7z-compressed

# Test desktop association
xdg-open test.7z
# Should launch ZipLock

# Check registered applications
xdg-mime query default application/x-7z-compressed
# Should include ziplock.desktop
```

**Package Testing:**
```bash
# Test desktop entry validation
desktop-file-validate /usr/share/applications/ziplock.desktop

# Test MIME type definition
xmllint --schema /usr/share/mime/freedesktop.org.xml /usr/share/mime/packages/ziplock.xml
```

## Troubleshooting

### Android Issues

**File Association Not Working:**
```bash
# Check if intent filters are registered
adb shell dumpsys package com.ziplock | grep "android.intent.action.VIEW"

# Verify MIME type handling
adb shell am start -a android.intent.action.VIEW -t "application/x-7z-compressed" -d "file:///path/to/test.7z"

# Check app permissions
adb shell dumpsys package com.ziplock | grep -A 5 "permissions"
```

**Content URI Issues:**
```kotlin
// Debug content URI resolution
Log.d("FileAssociation", "Received URI: $uri")
Log.d("FileAssociation", "Resolved path: ${FileUtils.getUsableFilePath(context, uri)}")
```

### Linux Issues

**MIME Type Not Recognized:**
```bash
# Reinstall MIME type definition
sudo cp ziplock.xml /usr/share/mime/packages/
sudo update-mime-database /usr/share/mime

# Check MIME type database
grep -r "7z" /usr/share/mime/
```

**Desktop Entry Problems:**
```bash
# Validate desktop entry
desktop-file-validate ziplock.desktop

# Check for syntax errors
cat ziplock.desktop | grep -E "^(Name|Exec|MimeType)"

# Refresh desktop database
sudo update-desktop-database /usr/share/applications
```

**File Manager Integration:**
```bash
# Test with different file managers
nautilus /path/to/test.7z      # GNOME
dolphin /path/to/test.7z       # KDE
thunar /path/to/test.7z        # XFCE

# Check default application
xdg-mime query default application/x-7z-compressed
```

## Best Practices

### Development

1. **Test Multiple Sources**: Verify file association works from various apps and sources
2. **Handle Edge Cases**: Account for corrupted files, permission issues, and network failures
3. **User Feedback**: Provide clear error messages for file access problems
4. **Performance**: Optimize file handling for large archives and slow storage
5. **Security**: Validate all file inputs and sanitize paths

### Deployment

1. **Package Installation**: Ensure proper registration during package installation
2. **User Education**: Document file association features in user guides
3. **Compatibility**: Test with popular file managers and cloud storage apps
4. **Updates**: Handle file association updates during app upgrades
5. **Uninstallation**: Clean up registrations when app is removed

### User Experience

1. **Seamless Integration**: File association should feel natural and fast
2. **Error Handling**: Graceful degradation when files are inaccessible
3. **Visual Feedback**: Clear indication when ZipLock is handling a file
4. **Consistency**: Uniform behavior across different file sources
5. **Documentation**: Clear instructions for users on how to use file associations

This comprehensive file association implementation ensures ZipLock integrates seamlessly with the operating system and provides users with convenient access to their password archives from any application that handles .7z files.