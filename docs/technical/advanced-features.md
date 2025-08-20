# Advanced Features Implementation Guide

This document covers the implementation details of advanced ZipLock features including repository validation, cloud storage handling, repository detection, and persistent archive path management.

## Table of Contents

- [Repository Validation System](#repository-validation-system)
- [Cloud Storage Implementation](#cloud-storage-implementation)
- [Repository Detection](#repository-detection)
- [Persistent Archive Path Management](#persistent-archive-path-management)
- [Integration Examples](#integration-examples)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

## Repository Validation System

### Overview

The comprehensive repository validation system ensures archive integrity and content validation when the backend connects to a repository. It replaces basic validation with a multi-layered approach that includes automatic repair capabilities.

### Implementation Details

#### Core Components

**ValidationConfig Structure:**
```rust
pub struct ValidationConfig {
    pub enable_comprehensive_validation: bool,
    pub deep_validation: bool,
    pub check_legacy_formats: bool,
    pub validate_schemas: bool,
    pub auto_repair: bool,
    pub fail_on_critical_issues: bool,
    pub log_validation_details: bool,
}
```

**Default Settings:**
- Comprehensive validation: **Enabled**
- Deep validation: **Enabled** 
- Auto-repair: **Enabled**
- Fail on critical issues: **Enabled**
- Legacy format checking: **Enabled**

#### Validation Flow

**Archive Opening Process:**
1. Extract archive to temporary directory
2. Create `RepositoryValidator` with user configuration settings
3. Perform comprehensive validation analysis
4. Log detailed validation reports (if enabled)
5. Attempt auto-repair for fixable issues
6. Handle critical issues based on configuration
7. Save repaired archive if changes were made
8. Load credentials from validated repository

**Validation Levels:**

1. **Basic Validation** (always performed):
   - Directory structure integrity
   - Required files present
   - Basic file format checks

2. **Comprehensive Validation** (configurable):
   - YAML schema compliance
   - Cross-reference integrity
   - Metadata consistency
   - Credential file validation

3. **Deep Validation** (configurable):
   - Individual credential file parsing
   - Content format verification
   - Advanced consistency checks
   - Performance impact higher but more thorough

#### Auto-Repair Capabilities

**Fixable Issues:**
- Missing directory structures
- Corrupted metadata files
- Schema format inconsistencies
- Legacy format migrations
- File permission issues

**Critical Issues (non-repairable):**
- Corrupted credential data
- Encryption/decryption failures
- Severe structural damage
- Incompatible archive versions

#### Configuration Examples

**Production Profile:**
```yaml
storage:
  validation:
    enable_comprehensive_validation: true
    deep_validation: true
    auto_repair: true
    fail_on_critical_issues: true
    log_validation_details: false
```

**Development Profile:**
```yaml
storage:
  validation:
    enable_comprehensive_validation: true
    deep_validation: false  # Faster validation
    auto_repair: true
    fail_on_critical_issues: false
    log_validation_details: true  # Detailed debugging
```

**Legacy Compatibility Profile:**
```yaml
storage:
  validation:
    enable_comprehensive_validation: false
    deep_validation: false
    check_legacy_formats: true
    validate_schemas: false
    auto_repair: true
    fail_on_critical_issues: false
```

## Cloud Storage Implementation

### Problem Statement

Cloud storage services present unique challenges for ZipLock archives:

1. **Storage Access Framework (SAF) Limitations**: Cloud files provide virtual URIs that don't support traditional file locking
2. **Temporary Caching**: Cloud services cache files unpredictably
3. **Background Sync Conflicts**: Services may sync while ZipLock has files open
4. **No Direct File Path**: Modern storage APIs abstract file paths for security

### Solution Architecture

#### Cloud-Aware File Handling

**CloudFileHandle Structure:**
```rust
pub struct CloudFileHandle {
    original_path: PathBuf,     // Original cloud storage path
    local_path: PathBuf,        // Local working copy path
    _lock_file: LockFile,       // File lock on local copy
    original_hash: String,      // Content hash for conflict detection
    is_cloud_file: bool,        // Whether this is a cloud file
    needs_sync_back: bool,      // Whether changes need to be synced back
}
```

#### Cloud Storage Detection

**Android Patterns:**
```
/Android/data/com.google.android.apps.docs/    # Google Drive
/Android/data/com.dropbox.android/             # Dropbox
/Android/data/com.microsoft.skydrive/          # OneDrive
/Android/data/com.box.android/                 # Box
/Android/data/com.nextcloud.client/            # Nextcloud
```

**Storage Access Framework Detection:**
```
content://com.android.externalstorage.documents/
content://com.google.android.apps.docs.files/
content://com.dropbox.android.provider/
```

#### Copy-to-Local Strategy

**Operation Flow:**
1. Detect cloud storage file using pattern matching
2. Create secure local working copy
3. Establish file lock on local copy
4. Perform all operations on local copy
5. Calculate content hash for conflict detection
6. Sync changes back to original location
7. Clean up local working copy

**Conflict Detection:**
```rust
fn detect_sync_conflict(&self) -> Result<bool, Error> {
    let current_hash = calculate_file_hash(&self.original_path)?;
    Ok(current_hash != self.original_hash)
}
```

#### Best Practices

**For Users:**
- Avoid opening the same archive from multiple devices simultaneously
- Wait for cloud sync to complete before opening archives
- Use local storage for frequently accessed archives
- Enable conflict notifications in cloud storage apps

**For Developers:**
- Always check for sync conflicts before saving
- Implement proper error handling for cloud operations
- Use the CloudFileHandle abstraction consistently
- Log cloud operations for debugging

### Android Cloud Storage Challenges

#### Enhanced File Locking

Traditional `flock()` doesn't work with cloud storage paths, so the implementation uses:
- Local working copies with proper file locking
- Hash-based conflict detection
- Automatic retry mechanisms for temporary failures

#### Conflict Prevention

**Detection Methods:**
- File modification timestamp comparison
- Content hash verification
- Cloud service API integration where available
- User notification for manual resolution

## Repository Detection

### Overview

Automatic repository detection helps users find existing ZipLock archives across their device and cloud storage locations.

### Implementation

#### Detection Algorithms

**File System Scanning:**
```rust
pub fn scan_for_repositories(search_paths: &[PathBuf]) -> Vec<RepositoryCandidate> {
    let mut candidates = Vec::new();
    
    for path in search_paths {
        let found = scan_directory_recursive(path, &ARCHIVE_PATTERNS);
        candidates.extend(found);
    }
    
    candidates
}
```

**Pattern Matching:**
- File extension: `*.7z`
- Magic number verification
- Archive structure validation
- Content type detection

#### Search Locations

**Standard Directories:**
- Documents folder
- Downloads folder
- Desktop (Linux/Windows)
- User-defined custom directories

**Cloud Storage Locations:**
- Google Drive sync folders
- Dropbox local folders
- OneDrive sync directories
- Custom cloud service folders

#### Validation Checks

**Repository Verification:**
1. File accessibility check
2. Archive format validation
3. ZipLock signature verification
4. Corruption detection
5. Permission verification

### Configuration

**Search Settings:**
```yaml
repository:
  detection:
    enable_auto_scan: true
    search_directories:
      - "~/Documents"
      - "~/Downloads"
      - "~/Desktop"
    exclude_directories:
      - "~/.cache"
      - "/tmp"
    max_scan_depth: 3
    scan_cloud_storage: true
```

## Persistent Archive Path Management

### Overview

Persistent archive path management ensures users can quickly reopen recently accessed archives, even when files are moved or accessed through different methods.

### Implementation

#### Path Storage

**Storage Structure:**
```rust
pub struct ArchivePathManager {
    recent_archives: Vec<RecentArchive>,
    path_mappings: HashMap<String, PathBuf>,
    max_recent: usize,
}

pub struct RecentArchive {
    pub path: PathBuf,
    pub display_name: String,
    pub last_accessed: SystemTime,
    pub file_hash: String,
    pub is_accessible: bool,
}
```

#### Path Resolution

**Resolution Strategy:**
1. Try exact path match
2. Check for moved files using file hash
3. Search in known locations
4. Prompt user for new location
5. Update stored path on successful resolution

#### Mobile Integration

**Android Implementation:**
```kotlin
class ArchivePathManager {
    fun storeRecentArchive(path: String, displayName: String) {
        val prefs = context.getSharedPreferences("archive_paths", Context.MODE_PRIVATE)
        val recentArchives = getRecentArchives().toMutableList()
        
        // Add or update entry
        recentArchives.removeAll { it.path == path }
        recentArchives.add(0, RecentArchive(path, displayName, System.currentTimeMillis()))
        
        // Limit to max entries
        if (recentArchives.size > MAX_RECENT_ARCHIVES) {
            recentArchives.subList(MAX_RECENT_ARCHIVES, recentArchives.size).clear()
        }
        
        saveRecentArchives(prefs, recentArchives)
    }
}
```

#### Cloud Storage Compatibility

**URI Persistence:**
- Store both original URI and resolved path
- Handle content:// URIs properly
- Maintain SAF permissions across app restarts
- Graceful handling of expired permissions

## Integration Examples

### Backend Integration

**Repository Opening with Validation:**
```rust
use ziplock_shared::validation::RepositoryValidator;
use ziplock_shared::config::ValidationConfig;

pub fn open_repository(path: &Path, config: &ValidationConfig) -> Result<Repository, Error> {
    // Extract archive
    let temp_dir = extract_archive(path)?;
    
    // Create validator
    let validator = RepositoryValidator::new(config.clone());
    
    // Perform validation
    let validation_result = validator.validate_repository(&temp_dir)?;
    
    if config.log_validation_details {
        log::info!("Validation result: {:?}", validation_result);
    }
    
    // Handle auto-repair
    if config.auto_repair && !validation_result.fixable_issues.is_empty() {
        validator.repair_issues(&temp_dir, &validation_result.fixable_issues)?;
        
        // Re-save archive if repairs were made
        save_archive(path, &temp_dir)?;
    }
    
    // Handle critical issues
    if config.fail_on_critical_issues && !validation_result.critical_issues.is_empty() {
        return Err(Error::CriticalValidationFailure(validation_result.critical_issues));
    }
    
    // Load repository
    Repository::from_directory(&temp_dir)
}
```

### Android Integration

**Create Archive with Cloud Detection:**
```kotlin
class CreateArchiveViewModel : ViewModel() {
    
    suspend fun createArchive(
        destinationPath: String,
        archiveName: String,
        passphrase: String
    ) {
        try {
            val fullPath = constructArchivePath(destinationPath, archiveName)
            
            // FFI handles cloud detection automatically
            val result = ZipLockNative.createArchive(fullPath, passphrase)
            
            if (result.isSuccess()) {
                // Store in recent archives
                ArchivePathManager.storeRecentArchive(fullPath, archiveName)
                
                _uiState.value = _uiState.value.copy(
                    currentStep = CreateArchiveStep.Success,
                    createdArchivePath = fullPath
                )
            } else {
                val errorMessage = ZipLockNativeHelper.getDetailedError(result)
                setError(errorMessage)
            }
        } catch (e: Exception) {
            setError("Archive creation failed: ${e.message}")
        }
    }
}
```

## Configuration

### Complete Configuration Example

```yaml
storage:
  # Validation settings
  validation:
    enable_comprehensive_validation: true
    deep_validation: true
    check_legacy_formats: true
    validate_schemas: true
    auto_repair: true
    fail_on_critical_issues: true
    log_validation_details: false
    
  # Cloud storage settings
  cloud_storage:
    enable_cloud_detection: true
    copy_to_local_strategy: true
    conflict_detection: true
    max_local_cache_size: "1GB"
    cleanup_temp_files: true
    
  # Repository detection settings
  repository_detection:
    enable_auto_scan: true
    max_scan_depth: 3
    scan_cloud_storage: true
    exclude_patterns:
      - "*.tmp"
      - ".cache/*"
      - "/tmp/*"
      
  # Persistent paths settings
  persistent_paths:
    max_recent_archives: 10
    store_file_hashes: true
    auto_resolve_moved_files: true
    cleanup_invalid_entries: true
```

### Environment Variables

```bash
# Override validation behavior
export ZIPLOCK_VALIDATION_MODE=strict|relaxed|legacy

# Cloud storage configuration
export ZIPLOCK_CLOUD_STORAGE_CACHE_DIR=/custom/cache/dir

# Repository detection
export ZIPLOCK_SCAN_DEPTH=5
export ZIPLOCK_EXCLUDE_CLOUD_SCAN=true
```

## Troubleshooting

### Validation Issues

**Problem**: Validation fails with schema errors
**Solution**: 
```bash
# Check repository structure
ziplock validate --repository /path/to/archive.7z --verbose

# Force repair
ziplock validate --repository /path/to/archive.7z --auto-repair

# Use legacy mode
ZIPLOCK_VALIDATION_MODE=legacy ziplock open /path/to/archive.7z
```

### Cloud Storage Issues

**Problem**: Archive not saving to cloud storage
**Solution**:
```bash
# Check cloud storage detection
adb logcat | grep "CloudFileHandle"

# Verify permissions
adb shell dumpsys package com.ziplock | grep -A 10 "permissions"

# Test with local storage first
# Then move to cloud after confirming functionality
```

**Problem**: Sync conflicts detected
**Solution**:
- Close archive in ZipLock
- Wait for cloud sync to complete
- Reopen archive
- Use conflict resolution if prompted

### Repository Detection Issues

**Problem**: Archives not found in scan
**Solution**:
```yaml
# Increase scan depth
repository_detection:
  max_scan_depth: 5
  
# Add custom search paths
search_directories:
  - "/custom/archive/location"
  
# Enable verbose logging
log_validation_details: true
```

### Persistent Path Issues

**Problem**: Recent archives not loading
**Solution**:
```kotlin
// Clear and rebuild recent archives cache
ArchivePathManager.clearRecentArchives()
ArchivePathManager.rebuildFromFilesystem()

// Check file permissions
if (!File(archivePath).canRead()) {
    // Request permissions or show file picker
}
```

## Performance Considerations

### Validation Performance

- **Deep validation**: 2-5x slower but more thorough
- **Schema validation**: Moderate impact, high value
- **Auto-repair**: Minimal impact for most issues
- **Logging**: Significant impact when verbose

### Cloud Storage Performance

- **Copy-to-local**: Initial overhead, faster subsequent operations
- **Hash calculation**: CPU intensive but necessary for conflict detection
- **Sync operations**: Network dependent, implement timeouts

### Memory Usage

- **Validation**: Temporary spike during repository analysis
- **Cloud copying**: Double memory usage during copy operations
- **Path storage**: Minimal impact, configurable limits

## Best Practices

### For Developers

1. **Always use validation**: Enable comprehensive validation in production
2. **Handle cloud storage**: Use CloudFileHandle for any file operations
3. **Implement proper error handling**: Cloud operations can fail unpredictably
4. **Test with real cloud services**: Emulator testing insufficient
5. **Monitor performance**: Validation and cloud operations impact user experience

### For Users

1. **Regular validation**: Run periodic repository health checks
2. **Cloud storage awareness**: Understand sync timing and conflicts
3. **Backup strategy**: Maintain local backups of critical archives
4. **Performance tuning**: Adjust validation settings based on usage patterns

### For System Administrators

1. **Configuration management**: Use appropriate profiles for deployment scenarios
2. **Monitoring**: Log validation results for system health
3. **Security**: Ensure proper permissions for cloud storage access
4. **Backup policies**: Include validation in backup verification procedures

This advanced features implementation provides robust, secure, and user-friendly handling of complex scenarios while maintaining ZipLock's core security and usability principles.