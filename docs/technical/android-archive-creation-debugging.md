# Android Archive Creation Debugging Guide

## Problem Summary

When clicking the "Create Archive" button in the archive creation wizard, the app would fail with an "Internal error" message and subsequently crash with a mutex destruction error.

## Root Cause Analysis

### Primary Issue: Content URI to File Path Conversion Missing

The Android Storage Access Framework (SAF) returns content URIs (e.g., `content://com.android.externalstorage.documents/tree/primary%3ADocuments`) when users select destinations. These URIs cannot be used directly as file paths by the native Rust library.

### Error Flow

1. **User Selection**: User selects destination via SAF → returns content URI
2. **Invalid Path Construction**: App constructs path like `content://...Documents/ZipLock.7z`
3. **Native Library Failure**: Rust FFI receives content URI as if it were a file path
4. **File Operations Fail**: Rust code tries `path.exists()`, `fs::create_dir_all()` on invalid path
5. **Error Mapping**: Failed operations return `ZipLockError::InternalError` (code 9)
6. **Resource Corruption**: Improper cleanup leads to mutex destruction and app crash

### Log Analysis

```
CreateArchiveViewModel: Creating archive at: content://com.android.externalstorage.documents/tree/primary%3ADocuments/ZipLock.7z
CreateArchiveViewModel: Archive creation result: success=false, error=Internal error
libc: FORTIFY: pthread_mutex_lock called on a destroyed mutex
libc: Fatal signal 6 (SIGABRT), code -1 (SI_QUEUE) in tid 13240 (hwuiTask0), pid 13216 (com.ziplock)
```

## Solution Implementation

### 1. Content URI Handling Utility (`FileUtils.kt`)

Created a comprehensive utility class for converting content URIs to usable file paths:

```kotlin
object FileUtils {
    fun getWritableArchivePath(context: Context, destinationUri: Uri, archiveName: String): WritableArchiveInfo
    fun copyBackToDestination(context: Context, workingFilePath: String, destinationUri: Uri): Boolean
    fun getUsableFilePath(context: Context, uri: Uri, fileName: String): String
    fun isCloudStorageUri(uri: Uri): Boolean
}
```

### 2. Copy-to-Local Strategy

For content URIs:
1. Create working copy in app's private cache directory
2. Perform archive operations on real file path
3. Copy completed archive back to original destination
4. Clean up temporary files

### 3. Updated Archive Creation Flow

Modified `CreateArchiveViewModel.createArchive()` to:
- Convert content URI to writable file path
- Use real file path for native library operations
- Handle copy-back for content URI destinations
- Provide better error messages
- Clean up temporary files on failure

### 4. UI Integration

Updated `CreateArchiveWizard` to:
- Pass Android Context to ViewModel
- Use new `startArchiveCreation(context)` method
- Maintain existing user experience

## Testing the Fix

### Manual Testing Steps

1. **Open Android Studio** and run the app
2. **Navigate to Create Archive** wizard
3. **Select destination** using document picker (should return content URI)
4. **Enter archive name** and proceed
5. **Create strong passphrase** and confirm
6. **Click "Create Archive"** button
7. **Verify success** - archive should be created without errors

### Expected Behavior

- No "Internal error" messages
- No app crashes
- Archive successfully created in selected location
- Proper progress indication
- Clear error messages if issues occur

### Log Verification

Successful creation should show:
```
CreateArchiveViewModel: Creating archive at working path: /data/data/com.ziplock/cache/new_archives/1234567890_ZipLock.7z
CreateArchiveViewModel: Will copy back to: content://...
CreateArchiveViewModel: Archive creation result: success=true, error=null
CreateArchiveViewModel: Copying archive back to final destination
```

## Error Handling Improvements

### Better Error Messages

| Native Error | User-Friendly Message |
|--------------|----------------------|
| "Internal error" | "Failed to create archive. The selected location may not be writable or the filename may be invalid." |
| "Permission denied" | "Permission denied. Please check that you have write access to the selected folder." |
| "Invalid parameter provided" | "Invalid archive name or destination. Please check your inputs and try again." |

### Recovery Mechanisms

- Automatic cleanup of temporary files on failure
- Proper error state management in UI
- Fallback to previous wizard step on creation failure
- Clear instructions for user recovery

## Cloud Storage Considerations

The solution properly handles various cloud storage scenarios:

### Detected Cloud Storage Patterns

- Google Drive: `/Android/data/com.google.android.apps.docs/`
- Dropbox: `/Android/data/com.dropbox.android/`
- OneDrive: `/Android/data/com.microsoft.skydrive/`
- SAF URIs: `content://com.android.externalstorage.documents/`

### Safety Measures

- All operations performed on local copies
- Content hash verification for conflict detection
- Automatic cleanup of temporary files
- Proper resource management

## Performance Impact

### Minimal Overhead for Local Files

- Local file operations unchanged
- No performance impact for direct file access
- Cloud detection adds negligible overhead

### Content URI Operations

- **Initial Copy**: One-time cost for content URI → local file
- **Archive Creation**: Full performance on local file
- **Copy Back**: One-time cost for local file → destination
- **Memory**: Temporary storage ≈ archive size during operation

## Security Considerations

### Temporary File Security

- Working files stored in app-private directories only
- Unique session directories prevent conflicts
- Automatic cleanup on completion/failure
- No sensitive data in shared system directories

### Permission Model

- Respects Android's scoped storage requirements
- Uses SAF for secure access to user-selected locations
- No broad storage permissions required
- Proper cleanup prevents data leakage

## Future Enhancements

### Potential Improvements

1. **Real-time Progress**: Stream-based copying with progress updates
2. **Conflict Resolution**: UI for handling destination file conflicts
3. **Offline Mode**: Better handling when cloud files unavailable
4. **Provider Optimization**: Cloud service-specific optimizations
5. **Background Operations**: Support for background archive creation

### Monitoring

Key metrics to track:
- Archive creation success rate
- Content URI vs. file URI usage patterns
- Temporary file cleanup effectiveness
- User error recovery patterns

## Debugging Commands

### ADB Log Filtering

```bash
# Monitor archive creation logs
adb logcat | grep -E "(CreateArchiveViewModel|FileUtils|ZipLockNative)"

# Watch for crashes
adb logcat | grep -E "(FATAL|libc.*FORTIFY)"

# Monitor file operations
adb logcat | grep -E "(Creating archive|Archive creation result)"
```

### File System Inspection

```bash
# Check temporary files
adb shell ls -la /data/data/com.ziplock/cache/

# Monitor storage usage
adb shell df /data/data/com.ziplock/
```

## Known Limitations

### Current Constraints

1. **Single Device Safety**: Cannot prevent conflicts across multiple devices
2. **Basic Conflict Detection**: Simple hash-based detection
3. **Provider Agnostic**: No cloud service-specific optimizations
4. **Synchronous Operations**: Copy operations block UI thread

### Workarounds

- Clear user warnings about cloud storage limitations
- Graceful degradation when operations fail
- Comprehensive error messages for user guidance
- Automatic retry mechanisms where appropriate

## Conclusion

This fix resolves the fundamental incompatibility between Android's content URI system and the native library's file path expectations. The solution maintains security, provides good user experience, and handles edge cases appropriately while preserving the existing app architecture.

The copy-to-local strategy ensures reliable archive creation across all Android storage scenarios while maintaining proper resource management and security boundaries.