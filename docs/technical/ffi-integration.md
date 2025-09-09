# FFI Integration Guide - Current Implementation

This document describes the implemented FFI interfaces in ZipLock's unified architecture, providing practical integration examples for mobile and desktop platforms.

## Table of Contents

- [Overview](#overview)
- [Implementation Status](#implementation-status)
- [Architecture Summary](#architecture-summary)
- [Mobile FFI Interface](#mobile-ffi-interface)
- [Desktop FFI Interface](#desktop-ffi-interface)
- [Error Handling](#error-handling)
- [Memory Management](#memory-management)
- [Integration Examples](#integration-examples)
- [Testing](#testing)

## Overview

ZipLock's unified architecture provides **two distinct FFI interfaces** that respect platform capabilities:

- **Mobile FFI** (`shared/src/ffi/mobile.rs`): Memory-only operations with JSON file map exchange
- **Desktop FFI** (`shared/src/ffi/desktop.rs`): Full repository operations with direct file I/O via sevenz-rust2

This approach maximizes code reuse while optimizing for each platform's strengths.

## Implementation Status

### ✅ Fully Implemented
- **Core Architecture**: UnifiedMemoryRepository, FileOperationProvider, UnifiedRepositoryManager
- **Mobile FFI Interface** (`shared/src/ffi/mobile.rs`) - Complete C API
- **Desktop FFI Interface** (`shared/src/ffi/desktop.rs`) - Complete C API
- **Common FFI Utilities** (`shared/src/ffi/common.rs`) - Error handling, string management
- **Error System**: Unified error handling with FFI conversion
- **Memory Management**: Safe string allocation/deallocation across FFI boundary

### 🚧 Platform Integration In Progress
- **Android App**: Mobile FFI integration needed
- **Linux Desktop**: Desktop FFI integration needed

### 📋 Planned
- **iOS App**: Will use Mobile FFI
- **Windows/macOS Desktop**: Will use Desktop FFI

## Architecture Summary

### Mobile Platform Flow
```
┌─────────────────┐    JSON File Map    ┌──────────────────┐
│   Mobile App    │ ◄──────────────────► │   Mobile FFI     │
│                 │                      │                  │
│ • File I/O      │                      │ • Memory ops     │
│ • Native 7z     │                      │ • Validation     │
│ • Platform APIs │                      │ • Business logic │
└─────────────────┘                      └──────────────────┘
```

### Desktop Platform Flow
```
┌─────────────────┐    Direct calls     ┌──────────────────┐
│  Desktop App    │ ◄─────────────────► │   Desktop FFI    │
│                 │                      │                  │
│ • UI layer      │                      │ • Full repo ops  │
│ • Configuration │                      │ • File I/O       │
│ • User input    │                      │ • sevenz-rust2   │
└─────────────────┘                      └──────────────────┘
```

## Mobile FFI Interface

**Location**: `shared/src/ffi/mobile.rs`

The mobile FFI provides memory-only operations where the mobile app handles all file I/O.

### Repository Lifecycle

```c
// Create a new repository handle
long ziplock_mobile_repository_create(void);

// Initialize the repository for use
int ziplock_mobile_repository_initialize(long handle);

// Check if repository is ready
int ziplock_mobile_repository_is_initialized(long handle);

// Clean up resources
void ziplock_mobile_repository_destroy(long handle);
```

### File Map Operations

Mobile platforms handle 7z archive extraction/creation and exchange file contents as JSON:

```c
// Load data from extracted archive files (JSON format)
int ziplock_mobile_repository_load_from_files(long handle, const char* files_json);

// Get current repository state as file map (for archive creation)
char* ziplock_mobile_repository_serialize_to_files(long handle);
```

**JSON File Map Format**:
```json
{
  "metadata.yml": "dmVyc2lvbjogIjEuMCI...",  // Base64 encoded YAML
  "credentials/uuid1/record.yml": "aWQ6IHV1aWQ...",
  "credentials/uuid2/record.yml": "aWQ6IHV1aWQ..."
}
```

### Credential Operations

All credential operations work with JSON strings:

```c
// Add a new credential (returns credential ID as JSON)
char* ziplock_mobile_add_credential(long handle, const char* credential_json);

// Retrieve credential by ID (returns full credential as JSON)
char* ziplock_mobile_get_credential(long handle, const char* credential_id);

// Update existing credential
int ziplock_mobile_update_credential(long handle, const char* credential_json);

// Delete credential by ID
int ziplock_mobile_delete_credential(long handle, const char* credential_id);

// List all credentials (returns JSON array)
char* ziplock_mobile_list_credentials(long handle);
```

### Repository State

```c
// Check if repository has unsaved changes
int ziplock_mobile_is_modified(long handle);

// Mark repository as saved (resets modified flag)
void ziplock_mobile_mark_saved(long handle);

// Get repository statistics (returns JSON)
char* ziplock_mobile_get_stats(long handle);

// Clear all credentials (useful for tests)
void ziplock_mobile_clear_credentials(long handle);
```

## Desktop FFI Interface

**Location**: `shared/src/ffi/desktop.rs`

The desktop FFI provides full repository operations including direct file I/O via sevenz-rust2.

### Repository Manager

```c
// Create repository manager handle
long ziplock_desktop_manager_create(void);

// Clean up manager resources
void ziplock_desktop_manager_destroy(long handle);
```

### Archive Operations

```c
// Create new password-protected archive
int ziplock_desktop_create_repository(long handle, const char* path, const char* password);

// Open existing archive
int ziplock_desktop_open_repository(long handle, const char* path, const char* password);

// Save current changes to archive
int ziplock_desktop_save_repository(long handle);

// Close current archive
int ziplock_desktop_close_repository(long handle);

// Change archive password
int ziplock_desktop_change_password(long handle, const char* new_password);
```

### Credential Operations

Similar to mobile but works directly with archives:

```c
// Add credential to currently open archive
char* ziplock_desktop_add_credential(long handle, const char* credential_json);

// Get credential from archive
char* ziplock_desktop_get_credential(long handle, const char* credential_id);

// Update credential in archive
int ziplock_desktop_update_credential(long handle, const char* credential_json);

// Delete credential from archive
int ziplock_desktop_delete_credential(long handle, const char* credential_id);

// List all credentials in archive
char* ziplock_desktop_list_credentials(long handle);
```

### Repository Status

```c
// Check if repository is open
int ziplock_desktop_is_open(long handle);

// Check for unsaved changes
int ziplock_desktop_is_modified(long handle);

// Get current archive path
char* ziplock_desktop_current_path(long handle);

// Get repository statistics
char* ziplock_desktop_get_stats(long handle);
```

## Error Handling

All FFI functions use consistent error codes defined in `shared/src/ffi/common.rs`:

```c
typedef enum {
    ZIPLOCK_SUCCESS = 0,
    ZIPLOCK_INVALID_PARAMETER = 1,
    ZIPLOCK_NOT_INITIALIZED = 2,
    ZIPLOCK_ALREADY_INITIALIZED = 3,
    ZIPLOCK_CREDENTIAL_NOT_FOUND = 4,
    ZIPLOCK_VALIDATION_FAILED = 5,
    ZIPLOCK_CRYPTO_ERROR = 6,
    ZIPLOCK_OUT_OF_MEMORY = 7,
    ZIPLOCK_INTERNAL_ERROR = 8,
    ZIPLOCK_SERIALIZATION_ERROR = 9,
    ZIPLOCK_FILE_NOT_FOUND = 10,
    ZIPLOCK_INVALID_PASSWORD = 11,
    ZIPLOCK_PERMISSION_DENIED = 12,
    ZIPLOCK_ARCHIVE_CORRUPTED = 13
} ZipLockError;
```

## Memory Management

### String Allocation

All FFI functions that return strings use consistent allocation:

```c
// Clean up strings returned by FFI functions
void ziplock_free_string(char* ptr);

### Archive Operations

Mobile platforms handle 7z archive creation and extraction via FFI:

```c
// Create encrypted archive in temporary location
char* ziplock_mobile_create_temp_archive(const char* files_json, const char* password);

// Extract encrypted archive from temporary location
char* ziplock_mobile_extract_temp_archive(const char* archive_path, const char* password);
```

Both functions work with JSON file maps for data exchange:

**Input/Output JSON File Map Format**:
```json
{
  "metadata.yml": "dmVyc2lvbjogIjEuMCI...",  // Base64 encoded YAML
  "credentials/uuid1/record.yml": "aWQ6IHV1aWQ...",
  "credentials/uuid2/record.yml": "aWQ6IHV1aWQ..."
}
```

// All platforms use the same cleanup
void ziplock_desktop_free_string(char* ptr);  // Same implementation
void ziplock_mobile_free_string(char* ptr);   // Same implementation
```

### Best Practices

1. **Always free returned strings**:
```c
char* result = ziplock_mobile_get_credential(handle, id);
if (result != NULL) {
    // Use result...
    ziplock_free_string(result);
}
```

2. **Check return codes**:
```c
int result = ziplock_mobile_add_credential(handle, json);
if (result != ZIPLOCK_SUCCESS) {
    // Handle error...
}
```

3. **Validate handles**:
```c
if (handle <= 0) {
    return ZIPLOCK_INVALID_PARAMETER;
}
```

## Integration Examples

### Android Integration Example

```kotlin
class ZipLockRepository {
    private var handle: Long = 0
    
    fun initialize(): Boolean {
        handle = ZipLockNative.ziplock_mobile_repository_create()
        return ZipLockNative.ziplock_mobile_repository_initialize(handle) == 0
    }
    
    fun loadFromArchive(archiveUri: Uri, password: String): Boolean {
        // 1. Read archive using SAF
        val archiveData = contentResolver.openInputStream(archiveUri)?.readBytes()
            ?: return false
            
        // 2. Extract using Apache Commons Compress or native library
        val extractedFiles = extractArchive(archiveData, password)
        
        // 3. Convert to JSON and load
        val filesJson = gson.toJson(extractedFiles)
        return ZipLockNative.ziplock_mobile_repository_load_from_files(handle, filesJson) == 0
    }
    
    fun addCredential(credential: Credential): String? {
        val credentialJson = gson.toJson(credential)
        val result = ZipLockNative.ziplock_mobile_add_credential(handle, credentialJson)
        return result.takeIf { it != null }?.also { 
            ZipLockNative.ziplock_free_string(it) 
        }
    }
    
    fun saveToArchive(archiveUri: Uri, password: String): Boolean {
        // 1. Get current state as file map
        val filesJson = ZipLockNative.ziplock_mobile_repository_serialize_to_files(handle)
            ?: return false
            
        try {
            // 2. Convert JSON to file map
            val fileMap = gson.fromJson<Map<String, String>>(filesJson, Map::class.java)
            
            // 3. Create archive and write using SAF
            val archiveData = createArchive(fileMap, password)
            contentResolver.openOutputStream(archiveUri)?.use { it.write(archiveData) }
            
            // 4. Mark as saved
            ZipLockNative.ziplock_mobile_mark_saved(handle)
            return true
        } finally {
            ZipLockNative.ziplock_free_string(filesJson)
        }
    }
}
```

### Linux Desktop Integration Example

```cpp
class ZipLockManager {
private:
    long handle = 0;
    
public:
    bool initialize() {
        handle = ziplock_desktop_manager_create();
        return handle > 0;
    }
    
    bool openRepository(const std::string& path, const std::string& password) {
        int result = ziplock_desktop_open_repository(handle, path.c_str(), password.c_str());
        return result == ZIPLOCK_SUCCESS;
    }
    
    std::optional<std::string> addCredential(const std::string& credentialJson) {
        char* result = ziplock_desktop_add_credential(handle, credentialJson.c_str());
        if (result == nullptr) return std::nullopt;
        
        std::string credentialId(result);
        ziplock_desktop_free_string(result);
        return credentialId;
    }
    
    bool save() {
        return ziplock_desktop_save_repository(handle) == ZIPLOCK_SUCCESS;
    }
    
    ~ZipLockManager() {
        if (handle > 0) {
            ziplock_desktop_manager_destroy(handle);
        }
    }
};
```

## Testing

### Unit Tests

Both FFI interfaces include comprehensive test suites:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mobile_repository_lifecycle() {
        let handle = ziplock_mobile_repository_create();
        assert!(handle > 0);
        
        let result = ziplock_mobile_repository_initialize(handle);
        assert_eq!(result, ZipLockError::Success as i32);
        
        ziplock_mobile_repository_destroy(handle);
    }
    
    #[test]
    fn test_desktop_repository_operations() {
        let handle = ziplock_desktop_manager_create();
        assert!(handle > 0);
        
        // Test operations...
        
        ziplock_desktop_manager_destroy(handle);
    }
}
```

### Integration Testing

Platform integration tests verify end-to-end functionality with actual file operations.

## Conclusion

The ZipLock FFI interfaces provide clean, platform-appropriate APIs for integrating with the unified architecture. Mobile platforms get memory-only operations optimized for platform file handling, while desktop platforms get full repository operations with direct file I/O.

Both interfaces share the same underlying core architecture, ensuring consistent behavior while respecting platform capabilities and constraints.