# ZipLock Cloud Storage Implementation

## Overview

This document describes the cloud storage enhancements implemented in ZipLock to handle archives stored in cloud services like Google Drive, Dropbox, OneDrive, and others on Android devices.

## Problem Statement

When users open ZipLock archives from cloud storage services on Android, several challenges arise:

1. **Storage Access Framework (SAF) Limitations**: Cloud files often provide virtual URIs (`content://`) that don't map to real filesystem paths where traditional file locking (`flock()`) can operate.

2. **Temporary Caching**: Cloud services may cache files in temporary locations that don't support file locking or are cleaned up unpredictably.

3. **Background Sync Conflicts**: Cloud services may sync files in the background while ZipLock has them open, potentially causing data corruption or sync conflicts.

4. **No Direct File Path**: Modern Android storage APIs abstract away file paths for security, making traditional file operations challenging.

## Solution Architecture

### Cloud-Aware File Handling

The solution introduces a `CloudFileHandle` that automatically detects cloud storage scenarios and implements a copy-to-local strategy:

```rust
// Core cloud storage handler
pub struct CloudFileHandle {
    original_path: PathBuf,     // Original cloud storage path
    local_path: PathBuf,        // Local working copy path
    _lock_file: LockFile,       // File lock on local copy
    original_hash: String,      // Content hash for conflict detection
    is_cloud_file: bool,        // Whether this is a cloud file
    needs_sync_back: bool,      // Whether changes need to be synced back
}
```

### Cloud Storage Detection

The system automatically detects cloud storage files using pattern matching:

#### Android Patterns
```
/Android/data/com.google.android.apps.docs/    # Google Drive
/Android/data/com.dropbox.android/             # Dropbox
/Android/data/com.microsoft.skydrive/          # OneDrive
/Android/data/com.box.android/                 # Box
/Android/data/com.nextcloud.client/            # Nextcloud
```

#### Storage Access Framework
```
content://com.android.providers.media.documents/
content://com.android.externalstorage.documents/
```

#### Generic Cloud Indicators
```
/cloud/, /sync/, /googledrive/, /dropbox/, /onedrive/
```

### Copy-to-Local Strategy

When a cloud file is detected:

1. **Hash Calculation**: Calculate content hash of original file for conflict detection
2. **Local Copy Creation**: Copy file to app-private working directory with unique session ID
3. **File Locking**: Create file lock on local copy (guaranteed to work)
4. **Operation Safety**: All archive operations work on the local copy
5. **Sync Back**: Changes are synced back to original location on save/close

### Conflict Detection

The system implements lightweight conflict detection:

- **Content Hashing**: Uses file size + modification time + content sampling
- **External Change Monitoring**: Detects if original cloud file was modified externally
- **Safe Sync Prevention**: Prevents syncing back if conflicts are detected

## Implementation Details

### Enhanced File Locking

The original `FileLock` implementation was enhanced with cloud storage warnings:

```rust
// In FileLock::new()
if is_cloud_storage_path(path) {
    warn!("Cloud storage file detected: {:?}. File locking may not prevent sync conflicts from cloud services.", path);
}
```

### Archive Manager Integration

The `ArchiveManager` was updated to use `CloudFileHandle` instead of direct `FileLock`:

```rust
// Before: Direct file locking
let file_lock = FileLock::new(&lock_path, timeout)?;

// After: Cloud-aware file handling
let cloud_file_handle = CloudFileHandle::new(&path, Some(timeout))?;
let working_path = cloud_file_handle.local_path(); // May differ from original
```

### Automatic Sync Back

Changes are automatically synced back to cloud storage:

```rust
// On save operations
if archive.cloud_file_handle.is_cloud_file() {
    archive.cloud_file_handle.mark_modified();
    archive.cloud_file_handle.sync_back()?;
}
```

## Configuration

Cloud storage behavior is currently hardcoded with these defaults:

- **Cloud Operation Timeout**: 60 seconds (vs 30 seconds for local files)
- **Conflict Detection**: Always enabled for cloud files
- **Automatic Sync Back**: Always enabled
- **Warning Messages**: Always logged for cloud storage detection

## Error Handling

### Cloud Storage Specific Errors

```rust
pub enum CloudStorageError {
    CopyFailed { reason: String },      // Failed to copy to/from cloud
    ContentModified,                    // External modification detected
    TempDirFailed(std::io::Error),     // Working directory creation failed
    LockError(FileLockError),          // File locking failed
    HashFailed { reason: String },      // Content hash calculation failed
}
```

### User-Friendly Messages

- **Cloud Detection**: "Cloud storage file detected. Working with local copy for safety."
- **Sync Conflicts**: "Archive was modified by external sync service. Please reload."
- **Sync Success**: "Successfully synced changes back to cloud storage."

## Testing

### Cloud Storage Detection Tests

```rust
#[test]
fn test_cloud_storage_detection() {
    // Android patterns
    assert!(is_cloud_storage_path(Path::new(
        "/Android/data/com.google.android.apps.docs/files/test.7z"
    )));
    
    // SAF URIs
    assert!(is_cloud_storage_path(Path::new(
        "content://com.android.providers.media.documents/document/test"
    )));
    
    // Should not detect normal paths
    assert!(!is_cloud_storage_path(Path::new("/home/user/documents/test.7z")));
}
```

### Conflict Detection Tests

```rust
#[test]
fn test_conflict_detection() {
    let original_hash = calculate_file_hash(&path)?;
    
    // Modify file externally
    std::fs::write(&path, b"modified content")?;
    let new_hash = calculate_file_hash(&path)?;
    
    // Should detect difference
    assert_ne!(original_hash, new_hash);
}
```

## Security Considerations

### Temporary File Security

- **App-Private Storage**: Working copies stored in app-private directories only
- **Session Isolation**: Each operation uses unique session directories
- **Automatic Cleanup**: Working directories cleaned up on completion or app termination
- **No System Temp**: No sensitive data in shared system temporary directories

### Conflict Prevention

- **Hash Verification**: Prevents data corruption from undetected external changes
- **Lock Validation**: File locks prevent concurrent local access during cloud operations
- **User Warnings**: Clear warnings about cloud storage risks and limitations

## Performance Impact

### Minimal Overhead for Local Files

- Local files continue to use direct file locking with no performance impact
- Cloud detection adds minimal path string analysis overhead

### Cloud File Operations

- **Initial Copy**: One-time cost to copy cloud file to local storage
- **Working Operations**: All operations on local copy (full performance)
- **Sync Back**: One-time cost to sync changes back to cloud storage
- **Memory Usage**: Temporary storage equal to archive size during operations

## Limitations and Future Enhancements

### Current Limitations

1. **Single Device Safety**: Cannot prevent conflicts across multiple devices
2. **Basic Conflict Detection**: Simple hash-based detection may miss some edge cases
3. **No Real-time Sync Monitoring**: Cannot detect background sync while app is running
4. **Provider Agnostic**: No cloud service-specific optimizations

### Future Enhancements

1. **Real-time Sync Monitoring**: Detect when cloud services are actively syncing
2. **Conflict Resolution UI**: User interface for handling detected conflicts
3. **Provider-Specific Optimizations**: Leverage cloud service APIs for better integration
4. **Offline Mode**: Better handling when cloud files are not available
5. **Multi-Device Coordination**: Cross-device locking mechanisms

## Migration and Compatibility

### Backward Compatibility

- All existing file operations continue to work unchanged
- Local files maintain identical behavior and performance
- No configuration changes required

### Automatic Migration

- Cloud storage detection is automatic and transparent
- No user action required to benefit from enhancements
- Graceful degradation if cloud operations fail

## Monitoring and Debugging

### Log Messages

```
INFO  Opening archive from cloud storage: /Android/data/com.google.../passwords.7z
INFO  Working with local copy: /data/data/com.ziplock/cache/session_1234/passwords.7z
WARN  Cloud storage file detected. File locking may not prevent sync conflicts.
INFO  Syncing changes back to cloud storage: /Android/data/com.google.../passwords.7z
```

### Debug Information

- Cloud storage pattern detection results
- Working directory paths and cleanup status
- Content hash calculations and conflict detection
- Sync back operation success/failure details

## Conclusion

The cloud storage enhancements provide robust handling of archives stored in cloud services while maintaining full backward compatibility and security. The implementation automatically detects cloud storage scenarios and applies appropriate safety measures without requiring user configuration or intervention.

Key benefits:

- **Automatic Detection**: No user configuration required
- **Safe Operations**: Copy-to-local prevents sync conflicts
- **Conflict Prevention**: Hash-based detection prevents data corruption
- **Transparent Operation**: Existing workflows continue unchanged
- **Security Focused**: App-private storage and automatic cleanup
- **Performance Optimized**: Minimal impact on local file operations

This implementation provides a solid foundation for cloud storage support that can be enhanced with additional features as needed.