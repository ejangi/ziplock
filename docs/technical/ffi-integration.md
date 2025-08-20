# FFI Integration Guide

This document provides comprehensive guidance for integrating ZipLock's Rust core library with mobile and desktop applications through Foreign Function Interface (FFI).

## Table of Contents

- [Overview](#overview)
- [FFI Architecture](#ffi-architecture)
- [C Interface Layer](#c-interface-layer)
- [Platform Integration](#platform-integration)
- [Common Templates](#common-templates)
- [Error Handling](#error-handling)
- [Memory Management](#memory-management)
- [Testing and Validation](#testing-and-validation)
- [Performance Considerations](#performance-considerations)
- [Troubleshooting](#troubleshooting)

## Overview

ZipLock's FFI layer enables seamless integration between the Rust core library and applications written in other languages. For Android, this uses a hybrid bridge model to prevent emulator crashes while maintaining full functionality:

## Standard FFI Architecture (Desktop/iOS)
```
Application Layer (Swift/C++)
        ↓
Platform-Specific Wrapper
        ↓
C FFI Interface
        ↓
Rust Core Library (Full Archive + Content Management)
```

## Hybrid Bridge Architecture (Android)
```
Android App (Kotlin)
        ↓
Phase 1: File System Operations (Kotlin - Apache Commons Compress)
        ↓
Extract Archive Contents to Temporary Directory
        ↓
Phase 2: Content Management (C FFI Interface → Rust Core Library)
        ↓
Receive Updated Contents from Native Library
        ↓
Phase 3: Save Back to File System (Kotlin - Android SAF Support)
```

### Key Benefits

- **Single Source of Truth**: Core logic implemented once in Rust
- **Memory Safety**: Rust's ownership system prevents common vulnerabilities
- **Performance**: Native performance across all platforms
- **Consistency**: Identical behavior across Android, iOS, and desktop platforms
- **Security**: Centralized cryptographic operations

## FFI Architecture

### Core Principles

1. **Platform Optimization**: Each platform uses the most suitable approach
2. **Crash Prevention**: Android uses hybrid bridge to avoid emulator issues
3. **Functionality Preservation**: All content management remains in Rust
4. **Performance**: Minimal overhead with optimal libraries per layer

### Architecture Models

#### Standard Model (Desktop/iOS)
- **Direct FFI**: Applications call Rust library directly for all operations
- **Full Integration**: Single FFI interface handles file system and content operations
- **Proven Stability**: Works reliably across desktop and iOS platforms

#### Hybrid Bridge Model (Android)
- **Three-Phase Process**: File operations → Content management → File save
- **Crash Prevention**: File operations use safe Kotlin libraries
- **Full Functionality**: Content operations use proven Rust implementation
- **Android Integration**: Native SAF support for content URIs

## FFI Architecture

### Core Components

**FFI Interface (`shared/src/ffi.rs`):**
```rust
// Core credential operations
#[no_mangle]
pub extern "C" fn ziplock_credential_new(
    title: *const c_char,
    username: *const c_char,
    password: *const c_char,
    url: *const c_char,
    notes: *const c_char
) -> *mut ZipLockCredential;

// Repository operations
#[no_mangle]
pub extern "C" fn ziplock_repository_open(
    path: *const c_char,
    passphrase: *const c_char
) -> ZipLockResult;

// Password generation
#[no_mangle]
pub extern "C" fn ziplock_password_generate(
    length: u32,
    include_uppercase: bool,
    include_lowercase: bool,
    include_numbers: bool,
    include_special: bool
) -> *mut c_char;
```

**C Header File (`shared/include/ziplock.h`):**
```c
#ifndef ZIPLOCK_H
#define ZIPLOCK_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct ZipLockCredential ZipLockCredential;
typedef struct ZipLockRepository ZipLockRepository;

// Result codes
typedef enum {
    ZIPLOCK_SUCCESS = 0,
    ZIPLOCK_ERROR_INVALID_ARGUMENT = 1,
    ZIPLOCK_ERROR_FILE_NOT_FOUND = 2,
    ZIPLOCK_ERROR_PERMISSION_DENIED = 3,
    ZIPLOCK_ERROR_INVALID_PASSPHRASE = 4,
    ZIPLOCK_ERROR_CORRUPTION = 5,
    ZIPLOCK_ERROR_CLOUD_STORAGE = 6,
    ZIPLOCK_ERROR_NETWORK = 7,
    ZIPLOCK_ERROR_UNKNOWN = 99
} ZipLockResult;

// Core functions
ZipLockResult ziplock_init(void);
const char* ziplock_get_version(void);
void ziplock_cleanup(void);

// Credential management
ZipLockCredential* ziplock_credential_new(
    const char* title,
    const char* username,
    const char* password,
    const char* url,
    const char* notes
);

void ziplock_credential_free(ZipLockCredential* credential);

// Repository operations
ZipLockResult ziplock_repository_open(const char* path, const char* passphrase);
ZipLockResult ziplock_repository_create(const char* path, const char* passphrase);
ZipLockResult ziplock_repository_save(void);
ZipLockResult ziplock_repository_close(void);

// Password generation
char* ziplock_password_generate(
    uint32_t length,
    bool include_uppercase,
    bool include_lowercase,
    bool include_numbers,
    bool include_special
);

// String management
void ziplock_string_free(char* str);

// Error handling
const char* ziplock_get_last_error(void);
ZipLockResult ziplock_validate_passphrase_strength(const char* passphrase);

#ifdef __cplusplus
}
#endif

#endif // ZIPLOCK_H
```

## Content Management Interface

### Memory Repository Interface (Centralized Architecture)

The new centralized memory repository provides consistent file structure management across all platforms:

```c
// Memory repository initialization
int ziplock_hybrid_init(void);
const char* ziplock_hybrid_get_version(void);
const char* ziplock_hybrid_get_last_error(void);
int ziplock_hybrid_cleanup(void);

// Repository content loading and file operations
int ziplock_hybrid_repository_load_content(const char* files_json);
const char* ziplock_hybrid_repository_get_file_operations(void);

// Credential CRUD operations
const char* ziplock_hybrid_repository_add_credential(const char* credential_yaml);
const char* ziplock_hybrid_repository_get_credential(const char* credential_id);
int ziplock_hybrid_repository_update_credential(const char* credential_yaml);
int ziplock_hybrid_repository_delete_credential(const char* credential_id);

// Query operations
const char* ziplock_hybrid_repository_list_credentials(void);
const char* ziplock_hybrid_repository_search_credentials(const char* query);

// Repository metadata
const char* ziplock_hybrid_repository_get_metadata(void);
const char* ziplock_hybrid_repository_get_structure(void);

// Memory management
void ziplock_hybrid_string_free(const char* ptr);
```

### Legacy Content Operations (Deprecated)

These operations are being replaced by the memory repository interface:

```c
// Repository content management (DEPRECATED - use memory repository instead)
int ziplock_open_extracted_contents(const char* extracted_dir, const char* password);
int ziplock_list_credentials(CredentialArray* out_credentials);
int ziplock_save_credential(const Credential* credential);
int ziplock_delete_credential(const char* credential_id);
int ziplock_close_repository(void);

// Data validation and crypto (still supported)
int ziplock_generate_password(const PasswordConfig* config, char* out_password, size_t buffer_size);
int ziplock_validate_email(const char* email);
int ziplock_validate_url(const char* url);
int ziplock_encrypt_data(const char* data, const char* password, char* out_encrypted, size_t buffer_size);
```

### Platform-Specific File Operations

#### Desktop/iOS (Direct FFI)
```c
// Direct archive operations (works reliably on desktop/iOS)
int ziplock_archive_open(const char* path, const char* password);
int ziplock_archive_create(const char* path, const char* password);
int ziplock_archive_save(void);
```

#### Android (Memory Repository Bridge)
```kotlin
// 1. Extract archive using Kotlin (platform-specific)
val archiveManager = ArchiveManager(context)
val extractResult = archiveManager.openArchive(archivePath, password, tempDir)

// 2. Load extracted files into memory repository
val memoryRepo = ZipLockMemoryRepository()
memoryRepo.initialize()
val filesMap = loadFilesFromDirectory(tempDir)
val loadResult = memoryRepo.loadContent(filesMap)

// 3. Perform operations through memory repository
val credentials = memoryRepo.listCredentials()
val credentialId = memoryRepo.addCredential(credential)

// 4. Get file operations for persistence
val fileOperations = memoryRepo.getFileOperations()

// 5. Execute file operations and create archive
executeFileOperations(tempDir, fileOperations)
val saveResult = archiveManager.createArchive(archivePath, password, tempDir)
```

#### Legacy Android (Hybrid Bridge - Deprecated)
```kotlin
// File operations handled by Kotlin bridge
class ArchiveManager {
    fun validateArchive(path: String, password: String): ValidationResult
    fun openArchive(path: String, password: String, extractDir: File): ExtractionResult  
    fun saveArchive(path: String, password: String, contentsDir: File): SaveResult
}

// Content operations still use FFI
val result = ZipLockNative.openExtractedContents(extractDir.path, password)
val credentials = ZipLockNative.listCredentials()
```

## C Interface Layer

### String Handling

**Rust Implementation:**
```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Convert C string to Rust string
unsafe fn c_str_to_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        return None;
    }
    
    match CStr::from_ptr(c_str).to_str() {
        Ok(s) => Some(s.to_string()),
        Err(_) => None,
    }
}

// Convert Rust string to C string (caller must free)
fn string_to_c_str(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// Free C string allocated by Rust
#[no_mangle]
pub extern "C" fn ziplock_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
```

### Error Handling

**Global Error State:**
```rust
use std::sync::Mutex;

lazy_static! {
    static ref LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
}

fn set_last_error(error: String) {
    if let Ok(mut last_error) = LAST_ERROR.lock() {
        *last_error = Some(error);
    }
}

#[no_mangle]
pub extern "C" fn ziplock_get_last_error() -> *const c_char {
    if let Ok(last_error) = LAST_ERROR.lock() {
        if let Some(ref error) = *last_error {
            return error.as_ptr() as *const c_char;
        }
    }
    std::ptr::null()
}
```

### Memory Management

**Object Lifetime Management:**
```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref OBJECT_REGISTRY: Mutex<HashMap<usize, Arc<dyn Any + Send + Sync>>> = 
        Mutex::new(HashMap::new());
}

// Register object and return handle
fn register_object<T: 'static + Send + Sync>(obj: T) -> *mut T {
    let arc = Arc::new(obj);
    let handle = Arc::as_ptr(&arc) as *mut T;
    
    if let Ok(mut registry) = OBJECT_REGISTRY.lock() {
        registry.insert(handle as usize, arc);
    }
    
    handle
}

// Unregister and cleanup object
fn unregister_object<T>(handle: *mut T) {
    if !handle.is_null() {
        if let Ok(mut registry) = OBJECT_REGISTRY.lock() {
            registry.remove(&(handle as usize));
        }
    }
}
```

## Platform Integration

### Android Hybrid Bridge Integration

The Android platform uses a three-phase hybrid approach to eliminate crashes while maintaining full functionality:

#### Phase 1: File System Validation (Kotlin)
```kotlin
class HybridRepositoryManager {
    suspend fun openRepository(archivePath: String, masterPassword: String): RepositoryResult<RepositoryMetadata> {
        // 1. Safe validation using Apache Commons Compress
        val validationResult = archiveManager.validateArchive(archivePath, masterPassword)
        if (!validationResult.success) {
            return RepositoryResult(success = false, errorMessage = validationResult.errorMessage)
        }
        
        // 2. Extract contents to temporary directory
        val tempDir = Files.createTempDirectory("ziplock_extract_").toFile()
        val extractResult = archiveManager.openArchive(archivePath, masterPassword, tempDir)
        
        // Continue to Phase 2...
    }
}
```

#### Phase 2: Content Management (Native FFI)
```kotlin
// 3. Hand extracted contents to native library
val nativeResult = ZipLockNative.openExtractedContents(tempDir.absolutePath, masterPassword)
if (!nativeResult.success) {
    return RepositoryResult(success = false, errorMessage = nativeResult.errorMessage)
}

// 4. All content operations now use native library
val credentials = ZipLockNative.listCredentials() // Works normally
val saveResult = ZipLockNative.saveCredential(newCredential) // Works normally
```

#### Phase 3: File System Save Back (Kotlin)
```kotlin
// 5. When saving, get updated contents from native library and save back
suspend fun saveRepository(): RepositoryResult<Boolean> {
    // Native library has updated the extracted contents
    // Now save back to original file system location
    val saveResult = archiveManager.saveArchive(originalPath, password, extractedContentsDir)
    return RepositoryResult(success = saveResult.success, errorMessage = saveResult.errorMessage)
}
```

### Desktop/iOS Direct Integration

Desktop and iOS platforms use direct FFI integration:

```swift
// iOS - Direct FFI calls work reliably
let result = ziplock_archive_open(archivePath, password)
let credentials = ziplock_list_credentials()
let saveResult = ziplock_save_credential(credential)
```

```cpp
// Desktop - Direct FFI calls work reliably  
int result = ziplock_archive_open(archive_path.c_str(), password.c_str());
CredentialArray credentials = ziplock_list_credentials();
int save_result = ziplock_save_credential(&credential);
```

## Platform Integration

### Android Integration

**JNI Wrapper (Kotlin):**
```kotlin
class ZipLockNative {
    companion object {
        init {
            System.loadLibrary("ziplock_shared")
        }
        
        // Native method declarations
        @JvmStatic
        external fun init(): Int
        
        @JvmStatic
        external fun getVersion(): String
        
        @JvmStatic
        external fun credentialNew(
            title: String,
            username: String,
            password: String,
            url: String,
            notes: String
        ): Long
        
        @JvmStatic
        external fun credentialFree(handle: Long)
        
        @JvmStatic
        external fun repositoryOpen(path: String, passphrase: String): Int
        
        @JvmStatic
        external fun passwordGenerate(
            length: Int,
            includeUppercase: Boolean,
            includeLowercase: Boolean,
            includeNumbers: Boolean,
            includeSpecial: Boolean
        ): String
        
        @JvmStatic
        external fun getLastError(): String
    }
}
```

**JNI Implementation:**
```c
#include <jni.h>
#include "ziplock.h"

// Helper function to create Java string
jstring create_jstring(JNIEnv *env, const char *str) {
    if (str == NULL) {
        return NULL;
    }
    return (*env)->NewStringUTF(env, str);
}

// Convert Java string to C string
const char* jstring_to_cstring(JNIEnv *env, jstring jstr) {
    if (jstr == NULL) {
        return NULL;
    }
    return (*env)->GetStringUTFChars(env, jstr, NULL);
}

// Release Java string
void release_cstring(JNIEnv *env, jstring jstr, const char *cstr) {
    if (jstr != NULL && cstr != NULL) {
        (*env)->ReleaseStringUTFChars(env, jstr, cstr);
    }
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ZipLockNative_init(JNIEnv *env, jclass clazz) {
    return (jint)ziplock_init();
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ZipLockNative_getVersion(JNIEnv *env, jclass clazz) {
    const char *version = ziplock_get_version();
    return create_jstring(env, version);
}

JNIEXPORT jlong JNICALL
Java_com_ziplock_ZipLockNative_credentialNew(
    JNIEnv *env, jclass clazz,
    jstring title, jstring username, jstring password, 
    jstring url, jstring notes) {
    
    const char *c_title = jstring_to_cstring(env, title);
    const char *c_username = jstring_to_cstring(env, username);
    const char *c_password = jstring_to_cstring(env, password);
    const char *c_url = jstring_to_cstring(env, url);
    const char *c_notes = jstring_to_cstring(env, notes);
    
    ZipLockCredential *credential = ziplock_credential_new(
        c_title, c_username, c_password, c_url, c_notes);
    
    // Release strings
    release_cstring(env, title, c_title);
    release_cstring(env, username, c_username);
    release_cstring(env, password, c_password);
    release_cstring(env, url, c_url);
    release_cstring(env, notes, c_notes);
    
    return (jlong)credential;
}

JNIEXPORT void JNICALL
Java_com_ziplock_ZipLockNative_credentialFree(JNIEnv *env, jclass clazz, jlong handle) {
    ziplock_credential_free((ZipLockCredential*)handle);
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ZipLockNative_passwordGenerate(
    JNIEnv *env, jclass clazz,
    jint length, jboolean uppercase, jboolean lowercase, 
    jboolean numbers, jboolean special) {
    
    char *password = ziplock_password_generate(
        (uint32_t)length, 
        uppercase == JNI_TRUE,
        lowercase == JNI_TRUE,
        numbers == JNI_TRUE,
        special == JNI_TRUE
    );
    
    jstring result = create_jstring(env, password);
    ziplock_string_free(password);
    
    return result;
}
```

### iOS Integration

**Swift Wrapper:**
```swift
import Foundation

class ZipLockNative {
    
    static let shared = ZipLockNative()
    
    private init() {
        ziplock_init()
    }
    
    deinit {
        ziplock_cleanup()
    }
    
    func getVersion() -> String {
        guard let cString = ziplock_get_version() else {
            return "Unknown"
        }
        return String(cString: cString)
    }
    
    func createCredential(
        title: String,
        username: String,
        password: String,
        url: String,
        notes: String
    ) -> UnsafeMutablePointer<ZipLockCredential>? {
        
        return title.withCString { titlePtr in
            username.withCString { usernamePtr in
                password.withCString { passwordPtr in
                    url.withCString { urlPtr in
                        notes.withCString { notesPtr in
                            return ziplock_credential_new(
                                titlePtr, usernamePtr, passwordPtr, urlPtr, notesPtr
                            )
                        }
                    }
                }
            }
        }
    }
    
    func freeCredential(_ credential: UnsafeMutablePointer<ZipLockCredential>) {
        ziplock_credential_free(credential)
    }
    
    func openRepository(path: String, passphrase: String) -> ZipLockResult {
        return path.withCString { pathPtr in
            passphrase.withCString { passphrasePtr in
                return ziplock_repository_open(pathPtr, passphrasePtr)
            }
        }
    }
    
    func generatePassword(
        length: UInt32,
        includeUppercase: Bool,
        includeLowercase: Bool,
        includeNumbers: Bool,
        includeSpecial: Bool
    ) -> String? {
        
        guard let cString = ziplock_password_generate(
            length, includeUppercase, includeLowercase, includeNumbers, includeSpecial
        ) else {
            return nil
        }
        
        let result = String(cString: cString)
        ziplock_string_free(UnsafeMutablePointer(mutating: cString))
        
        return result
    }
    
    func getLastError() -> String? {
        guard let cString = ziplock_get_last_error() else {
            return nil
        }
        return String(cString: cString)
    }
}
```

## Common Templates

### Error Handling Template

**Kotlin Error Handling:**
```kotlin
object ZipLockNativeHelper {
    
    fun validateLibrary(): Boolean {
        return try {
            ZipLockNative.init() == 0
        } catch (e: UnsatisfiedLinkError) {
            Log.e("ZipLock", "Native library not available", e)
            false
        }
    }
    
    fun mapErrorCode(errorCode: Int): String {
        return when (errorCode) {
            0 -> "Success"
            1 -> "Invalid argument provided"
            2 -> "File not found"
            3 -> "Permission denied"
            4 -> "Invalid passphrase"
            5 -> "Archive corruption detected"
            6 -> "Cloud storage error"
            7 -> "Network error"
            else -> "Unknown error (code: $errorCode)"
        }
    }
    
    fun getDetailedError(errorCode: Int): String {
        val baseMessage = mapErrorCode(errorCode)
        val lastError = try {
            ZipLockNative.getLastError()
        } catch (e: Exception) {
            null
        }
        
        return if (lastError.isNullOrBlank()) {
            baseMessage
        } else {
            "$baseMessage: $lastError"
        }
    }
    
    inline fun <T> safeCall(operation: () -> T): Result<T> {
        return try {
            if (!validateLibrary()) {
                Result.failure(Exception("Native library not available"))
            } else {
                Result.success(operation())
            }
        } catch (e: Exception) {
            Log.e("ZipLock", "Native call failed", e)
            Result.failure(e)
        }
    }
}
```

### Data Transfer Template

**Credential Data Class:**
```kotlin
data class Credential(
    val id: String,
    val title: String,
    val username: String,
    val password: String,
    val url: String,
    val notes: String,
    val createdAt: Long,
    val modifiedAt: Long
) {
    companion object {
        fun fromNative(handle: Long): Credential? {
            return ZipLockNativeHelper.safeCall {
                // Extract data from native credential object
                Credential(
                    id = ZipLockNative.credentialGetId(handle),
                    title = ZipLockNative.credentialGetTitle(handle),
                    username = ZipLockNative.credentialGetUsername(handle),
                    password = ZipLockNative.credentialGetPassword(handle),
                    url = ZipLockNative.credentialGetUrl(handle),
                    notes = ZipLockNative.credentialGetNotes(handle),
                    createdAt = ZipLockNative.credentialGetCreatedAt(handle),
                    modifiedAt = ZipLockNative.credentialGetModifiedAt(handle)
                )
            }.getOrNull()
        }
    }
    
    fun toNative(): Long? {
        return ZipLockNativeHelper.safeCall {
            ZipLockNative.credentialNew(title, username, password, url, notes)
        }.getOrNull()
    }
}
```

### Passphrase Validation Template

**Strength Validation:**
```kotlin
data class PassphraseStrengthResult(
    val score: Int,
    val level: StrengthLevel,
    val isValid: Boolean,
    val requirements: List<String>,
    val satisfied: List<String>
)

enum class StrengthLevel {
    VERY_WEAK, WEAK, FAIR, GOOD, STRONG, VERY_STRONG
}

object PassphraseValidator {
    
    fun validateStrength(passphrase: String): PassphraseStrengthResult {
        // Try FFI validation first
        val ffiResult = ZipLockNativeHelper.safeCall {
            ZipLockNative.validatePassphraseStrength(passphrase)
        }
        
        return if (ffiResult.isSuccess) {
            parseFFIValidationResult(ffiResult.getOrThrow())
        } else {
            // Fallback to local validation
            createFallbackValidation(passphrase)
        }
    }
    
    private fun createFallbackValidation(passphrase: String): PassphraseStrengthResult {
        val requirements = mutableListOf<String>()
        val satisfied = mutableListOf<String>()
        
        // Length check
        if (passphrase.length < 12) {
            requirements.add("Must be at least 12 characters long")
        } else {
            satisfied.add("Sufficient length (${passphrase.length} characters)")
        }
        
        // Character type checks
        if (!passphrase.any { it.isUpperCase() }) {
            requirements.add("Must contain uppercase letters")
        } else {
            satisfied.add("Contains uppercase letters")
        }
        
        if (!passphrase.any { it.isLowerCase() }) {
            requirements.add("Must contain lowercase letters")
        } else {
            satisfied.add("Contains lowercase letters")
        }
        
        if (!passphrase.any { it.isDigit() }) {
            requirements.add("Must contain numbers")
        } else {
            satisfied.add("Contains numbers")
        }
        
        val specialChars = "!@#$%^&*()_+-=[]{}|;:,.<>?"
        if (!passphrase.any { it in specialChars }) {
            requirements.add("Must contain special characters")
        } else {
            satisfied.add("Contains special characters")
        }
        
        // Calculate score
        val score = calculateScore(passphrase, requirements.isEmpty())
        val level = determineStrengthLevel(score)
        
        return PassphraseStrengthResult(
            score = score,
            level = level,
            isValid = requirements.isEmpty() && score >= 60,
            requirements = requirements,
            satisfied = satisfied
        )
    }
    
    private fun calculateScore(passphrase: String, meetsRequirements: Boolean): Int {
        var score = 0
        
        // Base score for length
        score += minOf(passphrase.length * 4, 40)
        
        // Character variety bonus
        if (passphrase.any { it.isUpperCase() }) score += 10
        if (passphrase.any { it.isLowerCase() }) score += 10
        if (passphrase.any { it.isDigit() }) score += 10
        if (passphrase.any { "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(it) }) score += 15
        
        // Entropy bonus for unique characters
        val uniqueChars = passphrase.toSet().size
        score += minOf(uniqueChars * 2, 15)
        
        // Penalty for common patterns
        if (hasCommonPatterns(passphrase)) {
            score -= 20
        }
        
        return maxOf(0, minOf(100, score))
    }
    
    private fun hasCommonPatterns(passphrase: String): Boolean {
        val lower = passphrase.lowercase()
        val commonPatterns = listOf(
            "password", "123456", "qwerty", "abc", "111", "000"
        )
        return commonPatterns.any { lower.contains(it) }
    }
    
    private fun determineStrengthLevel(score: Int): StrengthLevel {
        return when (score) {
            in 0..20 -> StrengthLevel.VERY_WEAK
            in 21..40 -> StrengthLevel.WEAK
            in 41..60 -> StrengthLevel.FAIR
            in 61..80 -> StrengthLevel.GOOD
            in 81..95 -> StrengthLevel.STRONG
            else -> StrengthLevel.VERY_STRONG
        }
    }
}
```

## Error Handling

### Comprehensive Error Management

**Error Types:**
```rust
#[derive(Debug, Clone)]
pub enum ZipLockError {
    InvalidArgument(String),
    FileNotFound(String),
    PermissionDenied(String),
    InvalidPassphrase,
    ArchiveCorruption(String),
    CloudStorageError(String),
    NetworkError(String),
    CryptoError(String),
    ValidationError(String),
    Unknown(String),
}

impl ZipLockError {
    pub fn to_error_code(&self) -> u32 {
        match self {
            ZipLockError::InvalidArgument(_) => 1,
            ZipLockError::FileNotFound(_) => 2,
            ZipLockError::PermissionDenied(_) => 3,
            ZipLockError::InvalidPassphrase => 4,
            ZipLockError::ArchiveCorruption(_) => 5,
            ZipLockError::CloudStorageError(_) => 6,
            ZipLockError::NetworkError(_) => 7,
            ZipLockError::CryptoError(_) => 8,
            ZipLockError::ValidationError(_) => 9,
            ZipLockError::Unknown(_) => 99,
        }
    }
    
    pub fn to_message(&self) -> &str {
        match self {
            ZipLockError::InvalidArgument(msg) => msg,
            ZipLockError::FileNotFound(path) => path,
            ZipLockError::PermissionDenied(msg) => msg,
            ZipLockError::InvalidPassphrase => "Invalid passphrase provided",
            ZipLockError::ArchiveCorruption(msg) => msg,
            ZipLockError::CloudStorageError(msg) => msg,
            ZipLockError::NetworkError(msg) => msg,
            ZipLockError::CryptoError(msg) => msg,
            ZipLockError::ValidationError(msg) => msg,
            ZipLockError::Unknown(msg) => msg,
        }
    }
}
```

### Platform-Specific Error Handling

**Android Exception Mapping:**
```kotlin
sealed class ZipLockException(message: String, cause: Throwable? = null) : Exception(message, cause) {
    class InvalidArgument(message: String) : ZipLockException(message)
    class FileNotFound(path: String) : ZipLockException("File not found: $path")
    class PermissionDenied(message: String) : ZipLockException(message)
    class InvalidPassphrase : ZipLockException("Invalid passphrase provided")
    class ArchiveCorruption(message: String) : ZipLockException(message)
    class CloudStorageError(message: String) : ZipLockException(message)
    class NetworkError(message: String) : ZipLockException(message)
    class Unknown(message: String) : ZipLockException(message)
    
    companion object {
        fun fromErrorCode(code: Int, message: String): ZipLockException {
            return when (code) {
                1 -> InvalidArgument(message)
                2 -> FileNotFound(message)
                3 -> PermissionDenied(message)
                4 -> InvalidPassphrase()
                5 -> ArchiveCorruption(message)
                6 -> CloudStorageError(message)
                7 -> NetworkError(message)
                else -> Unknown(message)
            }
        }
    }
}
```

## Memory Management

### RAII Pattern Implementation

**Rust Side:**
```rust
pub struct SafeString {
    ptr: *mut c_char,
}

impl SafeString {
    pub fn new(s: String) -> Self {
        SafeString {
            ptr: string_to_c_str(s),
        }
    }
    
    pub fn as_ptr(&self) -> *const c_char {
        self.ptr
    }
}

impl Drop for SafeString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(self.ptr);
            }
        }
    }
}
```

**Platform Wrapper:**
```kotlin
class NativeString private constructor(private val ptr: Long) : AutoCloseable {
    
    companion object {
        fun create(value: String): NativeString {
            val ptr = ZipLockNative.stringCreate(value)
            return NativeString(ptr)
        }
    }
    
    fun getValue(): String {
        return ZipLockNative.stringGetValue(ptr)
    }
    
    override fun close() {
        if (ptr != 0L) {
            ZipLockNative.stringFree(ptr)
        }
    }
}

// Usage with try-with-resources
fun example() {
    NativeString.create("Hello, World!").use { nativeStr ->
        val value = nativeStr.getValue()
        println(value)
    } // Automatically cleaned up
}
```

## Testing and Validation

### Unit Testing FFI Layer

**Rust Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_credential_lifecycle() {
        let title = CString::new("Test").unwrap();
        let username = CString::new("user").unwrap();
        let password = CString::new("pass").unwrap();
        let url = CString::new("https://example.com").unwrap();
        let notes = CString::new("notes").unwrap();
        
        let credential = unsafe {
            ziplock_credential_new(
                title.as_ptr(),
                username.as_ptr(),
                password.as_ptr(),
                url.as_ptr(),
                notes.as_ptr()
            )
        };
        
        assert!(!credential.is_null());
        
        unsafe {
            ziplock_credential_free(credential);
        }
    }
    
    #[test]
    fn test_string_handling() {
        let original = "Test string with émojis 🔒";
        let c_string = string_to_c_str(original.to_string());
        
        assert!(!c_string.is_null());
        
        let recovered = unsafe {
            c_str_to_string(c_string).unwrap()
        };
        
        assert_eq!(original, recovered);
        
        unsafe {
            ziplock_string_free(c_string);
        }
    }
}
```

### Integration Testing

**Android Integration Tests:**
```kotlin
@RunWith(AndroidJUnit4::class)
class ZipLockNativeTest {
    
    @Test
    fun testLibraryInitialization() {
        assertTrue("Library should initialize successfully", 
                  ZipLockNativeHelper.validateLibrary())
    }
    
    @Test
    fun testCredentialOperations() {
        val result = ZipLockNativeHelper.safeCall {
            val handle = ZipLockNative.credentialNew(
                "Gmail", "user@example.com", "password123", 
                "https://gmail.com", "Personal email"
            )
            
            assertNotEquals("Handle should not be null", 0L, handle)
            
            ZipLockNative.credentialFree(handle)
            true
        }
        
        assertTrue("Credential operations should succeed", result.isSuccess)
    }
    
    @Test
    fun testPasswordGeneration() {
        val password = ZipLockNativeHelper.safeCall {
            ZipLockNative.passwordGenerate(16, true, true, true, true)
        }
        
        assertTrue("Password generation should succeed", password.isSuccess)
        assertEquals("Password should be 16 characters", 16, password.getOrThrow().length)
    }
    
    @Test
    fun testErrorHandling() {
        val result = ZipLockNativeHelper.safeCall {
            ZipLockNative.repositoryOpen("nonexistent.7z", "password")
        }
        
        assertTrue("Should fail for nonexistent file", result.isFailure)
    }
}
```

## Performance Considerations

### Memory Optimization

**Object Pooling:**
```rust
use std::sync::Mutex;
use std::collections::VecDeque;

lazy_static! {
    static ref STRING_POOL: Mutex<VecDeque<CString>> = Mutex::new(VecDeque::new());
}

fn get_pooled_string(s: String) -> *mut c_char {
    if let Ok(mut pool) = STRING_POOL.lock() {
        if let Some(mut pooled) = pool.pop_front() {
            // Reuse existing CString if possible
            if pooled.as_bytes().len() >= s.len() {
                // Truncate and reuse
                unsafe {
                    let ptr = pooled.as_ptr() as *mut c_char;
                    std::ptr::copy_nonoverlapping(s.as_ptr(), ptr as *mut u8, s.len());
                    *((ptr as *mut u8).add(s.len())) = 0; // Null terminator
                    return pooled.into_raw();
                }
            }
    
            // Create new string if pool is empty or no suitable string found
            string_to_c_str(s)
        }

        fn return_pooled_string(ptr: *mut c_char) {
            if !ptr.is_null() {
                let c_string = unsafe { CString::from_raw(ptr) };
                if let Ok(mut pool) = STRING_POOL.lock() {
                    if pool.len() < 10 { // Limit pool size
                        pool.push_back(c_string);
                        return;
                    }
                }
                // If pool is full, just drop the string
            }
        }
        ```

        ### Threading Considerations

        **Thread Safety:**
        ```rust
        use std::sync::atomic::{AtomicBool, Ordering};

        static LIBRARY_INITIALIZED: AtomicBool = AtomicBool::new(false);

        #[no_mangle]
        pub extern "C" fn ziplock_init() -> ZipLockResult {
            if LIBRARY_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                // Initialize logging, crypto, etc.
                match initialize_library() {
                    Ok(_) => ZipLockResult::ZIPLOCK_SUCCESS,
                    Err(e) => {
                        set_last_error(format!("Initialization failed: {}", e));
                        LIBRARY_INITIALIZED.store(false, Ordering::SeqCst);
                        ZipLockResult::ZIPLOCK_ERROR_UNKNOWN
                    }

        ### Android UI Integration Timing

        **Critical Timing Issue:**
        Android UI components may attempt to load data before archive operations complete. This requires careful coordination between UI state and FFI operations.

        **Problem Pattern:**
        ```kotlin
        // PROBLEMATIC - Race condition
        class CredentialsViewModel : ViewModel() {
            init {
                loadCredentials() // Called before archive is fully open
            }
        }
        ```

        **Solution Pattern:**
        ```kotlin
        // FIXED - Wait for confirmed repository state
        LaunchedEffect(repositoryState) {
            if (repositoryState is HybridRepositoryState.Open) {
                delay(500) // Allow background initialization to complete
                credentialsViewModel.loadCredentials()
            }
        }
        ```

        **Key Considerations:**
        - Archive opening is asynchronous and may take several seconds
        - UI composition happens immediately on navigation
        - FFI calls should only occur after archive state is confirmed
        - Always validate `isArchiveOpen()` before data operations
        - Use repository state management rather than immediate UI triggers
                }
            } else {
                ZipLockResult::ZIPLOCK_SUCCESS // Already initialized
            }
        }
        ```

        ### Asynchronous Operations

        **Future-Based API:**
        ```rust
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        pub struct AsyncOperation {
            future: Pin<Box<dyn Future<Output = Result<String, ZipLockError>> + Send>>,
        }

        #[no_mangle]
        pub extern "C" fn ziplock_repository_open_async(
            path: *const c_char,
            passphrase: *const c_char,
            callback: extern "C" fn(result: ZipLockResult, handle: *mut c_void),
            user_data: *mut c_void,
        ) -> ZipLockResult {
            let path_str = match unsafe { c_str_to_string(path) } {
                Some(s) => s,
                None => return ZipLockResult::ZIPLOCK_ERROR_INVALID_ARGUMENT,
            };
    
            let passphrase_str = match unsafe { c_str_to_string(passphrase) } {
                Some(s) => s,
                None => return ZipLockResult::ZIPLOCK_ERROR_INVALID_ARGUMENT,
            };
    
            // Spawn async operation
            tokio::spawn(async move {
                let result = open_repository_async(&path_str, &passphrase_str).await;
                match result {
                    Ok(handle) => callback(ZipLockResult::ZIPLOCK_SUCCESS, handle as *mut c_void),
                    Err(e) => {
                        set_last_error(e.to_message().to_string());
                        callback(e.to_error_code() as ZipLockResult, std::ptr::null_mut());
                    }
                }
            });
    
            ZipLockResult::ZIPLOCK_SUCCESS
        }
        ```

        ## Troubleshooting

        ### Common Issues

        **Library Loading Problems:**

        *Android:*
        ```kotlin
        // Check library availability
        fun checkLibraryStatus(): String {
            return try {
                System.loadLibrary("ziplock_shared")
                "Library loaded successfully"
            } catch (e: UnsatisfiedLinkError) {
                "Library not found: ${e.message}"
            }
        }

        // Debug library path
        fun debugLibraryPath() {
            val libraryPath = System.getProperty("java.library.path")
            Log.d("ZipLock", "Library path: $libraryPath")
    
            val context = getApplicationContext()
            val nativeLibraryDir = context.applicationInfo.nativeLibraryDir
            Log.d("ZipLock", "Native library dir: $nativeLibraryDir")
        }
        ```

        *iOS:*
        ```swift
        // Check framework bundle
        func checkFrameworkBundle() {
            guard let bundle = Bundle(identifier: "com.ziplock.framework") else {
                print("Framework bundle not found")
                return
            }
    
            let frameworkPath = bundle.bundlePath
            print("Framework path: \(frameworkPath)")
    
            // Check if library exists
            let libraryPath = bundle.path(forResource: "libziplock_shared", ofType: "a")
            print("Library path: \(libraryPath ?? "Not found")")
        }
        ```

        ### Memory Leak Detection

        **Rust Memory Tracking:**
        ```rust
        use std::sync::atomic::{AtomicUsize, Ordering};

        static ALLOCATED_OBJECTS: AtomicUsize = AtomicUsize::new(0);
        static FREED_OBJECTS: AtomicUsize = AtomicUsize::new(0);

        pub fn track_allocation() {
            ALLOCATED_OBJECTS.fetch_add(1, Ordering::Relaxed);
        }

        pub fn track_deallocation() {
            FREED_OBJECTS.fetch_add(1, Ordering::Relaxed);
        }

        #[no_mangle]
        pub extern "C" fn ziplock_get_memory_stats() -> *mut c_char {
            let allocated = ALLOCATED_OBJECTS.load(Ordering::Relaxed);
            let freed = FREED_OBJECTS.load(Ordering::Relaxed);
            let leaked = allocated.saturating_sub(freed);
    
            let stats = format!("Allocated: {}, Freed: {}, Leaked: {}", allocated, freed, leaked);
            string_to_c_str(stats)
        }
        ```

        **Platform Memory Monitoring:**
        ```kotlin
        object MemoryMonitor {
            private var activeHandles = mutableSetOf<Long>()
    
            fun trackHandle(handle: Long) {
                synchronized(activeHandles) {
                    activeHandles.add(handle)
                }
            }
    
            fun releaseHandle(handle: Long) {
                synchronized(activeHandles) {
                    activeHandles.remove(handle)
                }
            }
    
            fun getLeakedHandles(): Set<Long> {
                synchronized(activeHandles) {
                    return activeHandles.toSet()
                }
            }
    
            fun logMemoryStats() {
                val nativeStats = ZipLockNative.getMemoryStats()
                val managedLeaks = getLeakedHandles().size
                Log.i("MemoryMonitor", "Native stats: $nativeStats")
                Log.i("MemoryMonitor", "Managed leaks: $managedLeaks")
            }
        }
        ```

        ### Performance Debugging

        **FFI Call Timing:**
        ```kotlin
        class FFIProfiler {
            private val callTimes = mutableMapOf<String, MutableList<Long>>()
    
            inline fun <T> profile(operation: String, block: () -> T): T {
                val startTime = System.nanoTime()
                try {
                    return block()
                } finally {
                    val endTime = System.nanoTime()
                    val duration = endTime - startTime
            
                    synchronized(callTimes) {
                        callTimes.getOrPut(operation) { mutableListOf() }.add(duration)
                    }
                }
            }
    
            fun generateReport(): String {
                synchronized(callTimes) {
                    return callTimes.entries.joinToString("\n") { (operation, times) ->
                        val avg = times.average() / 1_000_000 // Convert to milliseconds
                        val max = times.maxOrNull()?.let { it / 1_000_000 } ?: 0.0
                        val min = times.minOrNull()?.let { it / 1_000_000 } ?: 0.0
                        "$operation: avg=${avg}ms, max=${max}ms, min=${min}ms, calls=${times.size}"
                    }
                }
            }
        }

        // Usage
        val profiler = FFIProfiler()
        val result = profiler.profile("credentialNew") {
            ZipLockNative.credentialNew("title", "user", "pass", "url", "notes")
        }
        ```

        ### Error Diagnosis

        **Debug Helper Functions:**
        ```rust
        #[cfg(debug_assertions)]
        #[no_mangle]
        pub extern "C" fn ziplock_debug_validate_handle(handle: *const c_void) -> bool {
            // Validate that handle points to valid memory
            if handle.is_null() {
                return false;
            }
    
            // Check if handle is in our registry
            if let Ok(registry) = OBJECT_REGISTRY.lock() {
                return registry.contains_key(&(handle as usize));
            }
    
            false
        }

        #[cfg(debug_assertions)]
        #[no_mangle]
        pub extern "C" fn ziplock_debug_get_backtrace() -> *mut c_char {
            let backtrace = std::backtrace::Backtrace::capture();
            string_to_c_str(format!("{}", backtrace))
        }
        ```

        ## Best Practices

        ### Security Guidelines

        1. **Input Validation**: Always validate all inputs from FFI calls
        2. **Memory Safety**: Use RAII patterns and careful pointer management
        3. **Error Handling**: Never panic across FFI boundaries
        4. **Sensitive Data**: Clear sensitive data immediately after use
        5. **Thread Safety**: Use appropriate synchronization primitives

        ### Performance Guidelines

        1. **Minimize FFI Calls**: Batch operations when possible
        2. **Efficient Data Transfer**: Use bulk operations for large datasets
        3. **Memory Pooling**: Reuse objects to reduce allocation overhead
        4. **Async Operations**: Use callbacks for long-running operations
        5. **Profiling**: Regular performance monitoring and optimization

        ### Maintainability Guidelines

        1. **Consistent API**: Follow naming conventions across platforms
        2. **Documentation**: Comprehensive API documentation with examples
        3. **Testing**: Extensive unit and integration test coverage
        4. **Versioning**: Careful API versioning for backward compatibility
        5. **Error Messages**: Clear, actionable error messages for developers

        ## Future Enhancements

        ### Planned Improvements

        1. **WebAssembly Support**: Extend FFI to support web applications
        2. **Advanced Threading**: Better async/await integration
        3. **Performance Optimizations**: Zero-copy data transfer where possible
        4. **Enhanced Error Types**: More granular error reporting
        5. **Platform-Specific Features**: OS-specific optimizations

        ### API Evolution

        **Version 2.0 Considerations:**
        ```rust
        // Enhanced result types
        pub struct ZipLockOperationResult {
            pub success: bool,
            pub error_code: u32,
            pub error_message: *const c_char,
            pub data: *mut c_void,
            pub data_size: usize,
        }

        // Batch operations
        #[no_mangle]
        pub extern "C" fn ziplock_credentials_batch_create(
            credentials: *const CredentialData,
            count: usize,
            results: *mut ZipLockOperationResult,
        ) -> ZipLockResult;

        // Streaming API
        #[no_mangle]
        pub extern "C" fn ziplock_repository_stream_credentials(
            callback: extern "C" fn(*const CredentialData, *mut c_void),
            user_data: *mut c_void,
        ) -> ZipLockResult;
        ```

## Adaptive Runtime Architecture Integration Patterns

### Runtime Strategy Detection Implementation

The adaptive hybrid architecture automatically detects runtime contexts and selects the optimal execution strategy:

```c
// Runtime strategy detection in FFI
typedef enum {
    ZIPLOCK_RUNTIME_STRATEGY_CREATE_OWNED = 0,
    ZIPLOCK_RUNTIME_STRATEGY_EXTERNAL_FILE_OPS = 1,
    ZIPLOCK_RUNTIME_STRATEGY_USE_EXISTING = 2
} ZipLockRuntimeStrategy;

// Automatic context detection
int ziplock_hybrid_detect_runtime_context(void);
ZipLockRuntimeStrategy ziplock_hybrid_get_runtime_strategy(void);

// Archive operations with adaptive behavior
int ziplock_hybrid_create_archive(const char* archive_path, const char* password);
int ziplock_hybrid_open_archive(const char* archive_path, const char* password);
```

### Platform Integration Cookbook

#### Linux Integration (Async Context)

```rust
use ziplock_shared::client::hybrid::{HybridClientError, ZipLockHybridClient};
use crate::platform::LinuxFileOperationsHandler;

async fn handle_archive_creation(path: PathBuf, password: String) -> Result<(), String> {
    let client = ZipLockHybridClient::new()?;
    
    match client.create_archive_adaptive(path.clone(), password.clone()).await {
        Ok(()) => {
            // Direct success - archive created by hybrid FFI
            info!("Archive created via integrated operations");
            Ok(())
        }
        Err(HybridClientError::ExternalFileOpsRequired { file_operations }) => {
            // External file operations required - use platform handler
            info!("Using Linux platform file operations");
            
            let mut handler = LinuxFileOperationsHandler::new();
            handler.execute_file_operations(&file_operations).await?;
            
            // Open the created archive in memory
            client.open_archive_adaptive(path, password).await?;
            Ok(())
        }
        Err(e) => Err(format!("Archive creation failed: {:?}", e))
    }
}
```

#### Android Integration (Mobile Context)

```kotlin
class AndroidArchiveHandler(private val context: Context) {
    
    suspend fun createArchive(uri: Uri, password: String): Result<Unit> {
        return withContext(Dispatchers.IO) {
            try {
                // Mobile platforms always use external file operations
                val client = ZipLockHybridClient()
                val result = client.createArchiveAdaptive(uri.path, password)
                
                when (result.errorCode) {
                    ZIPLOCK_HYBRID_SUCCESS -> {
                        // Unexpected - mobile should always require external ops
                        Log.w(TAG, "Mobile platform got direct success - unusual")
                        Result.success(Unit)
                    }
                    ZIPLOCK_HYBRID_EXTERNAL_FILE_OPERATIONS_REQUIRED -> {
                        // Expected path for mobile
                        val fileOps = client.getFileOperations()
                        executeAndroidFileOperations(fileOps, uri)
                        Result.success(Unit)
                    }
                    else -> {
                        val error = client.getLastError()
                        Result.failure(Exception("Archive creation failed: $error"))
                    }
                }
            } catch (e: Exception) {
                Result.failure(e)
            }
        }
    }
    
    private suspend fun executeAndroidFileOperations(
        fileOpsJson: String,
        uri: Uri
    ) {
        val fileOps = Json.decodeFromString<FileOperations>(fileOpsJson)
        
        for (operation in fileOps.operations) {
            when (operation.type) {
                "create_archive" -> createArchiveWithSAF(operation, uri)
                "extract_archive" -> extractArchiveWithSAF(operation, uri)
                "update_archive" -> updateArchiveWithSAF(operation, uri)
            }
        }
    }
}
```

#### iOS Integration (Mobile Context)

```swift
class iOSArchiveHandler {
    
    func createArchive(at url: URL, password: String) async throws {
        let client = ZipLockHybridClient()
        
        let result = await client.createArchiveAdaptive(path: url.path, password: password)
        
        switch result.errorCode {
        case ZIPLOCK_HYBRID_SUCCESS:
            // Unexpected for mobile platforms
            print("Warning: Mobile platform got direct success")
            
        case ZIPLOCK_HYBRID_EXTERNAL_FILE_OPERATIONS_REQUIRED:
            // Expected path for iOS
            let fileOpsJson = client.getFileOperations()
            try await executeIOSFileOperations(fileOpsJson, at: url)
            
        default:
            let errorMessage = client.getLastError()
            throw ArchiveError.creationFailed(errorMessage)
        }
    }
    
    private func executeIOSFileOperations(_ fileOpsJson: String, at url: URL) async throws {
        let fileOps = try JSONDecoder().decode(FileOperations.self, from: fileOpsJson.data(using: .utf8)!)
        
        for operation in fileOps.operations {
            switch operation.type {
            case "create_archive":
                try await createArchiveWithDocuments(operation, at: url)
            case "extract_archive":
                try await extractArchiveWithDocuments(operation, at: url)
            case "update_archive":
                try await updateArchiveWithDocuments(operation, at: url)
            default:
                print("Unknown operation type: \(operation.type)")
            }
        }
    }
}
```

#### Windows Integration (Desktop Sync Context)

```cpp
#include "ziplock.h"
#include <windows.h>
#include <iostream>

class WindowsArchiveHandler {
public:
    bool createArchive(const std::string& path, const std::string& password) {
        // Initialize hybrid client
        if (ziplock_hybrid_init() != ZIPLOCK_HYBRID_SUCCESS) {
            std::cerr << "Failed to initialize hybrid client" << std::endl;
            return false;
        }
        
        // Attempt archive creation with adaptive behavior
        int result = ziplock_hybrid_create_archive(path.c_str(), password.c_str());
        
        switch (result) {
        case ZIPLOCK_HYBRID_SUCCESS:
            // Direct success - Windows sync context created own runtime
            std::cout << "Archive created via integrated operations" << std::endl;
            return true;
            
        case ZIPLOCK_HYBRID_EXTERNAL_FILE_OPERATIONS_REQUIRED:
            // Async context detected - use external file operations
            std::cout << "External file operations required" << std::endl;
            return handleExternalFileOperations(path, password);
            
        default:
            const char* error = ziplock_hybrid_get_last_error();
            std::cerr << "Archive creation failed: " << error << std::endl;
            return false;
        }
    }
    
private:
    bool handleExternalFileOperations(const std::string& path, const std::string& password) {
        const char* fileOpsJson = ziplock_hybrid_get_file_operations();
        if (!fileOpsJson) {
            std::cerr << "Failed to get file operations" << std::endl;
            return false;
        }
        
        // Parse and execute file operations using Windows APIs
        bool success = executeWindowsFileOperations(fileOpsJson, path, password);
        ziplock_hybrid_string_free(fileOpsJson);
        
        return success;
    }
};
```

### Runtime Strategy Decision Tree

```
Application Startup
       ↓
   Initialize Hybrid FFI
       ↓
   Call Archive Operation
       ↓
   Runtime Context Detection
       ↓
┌─────────────────┬─────────────────┬─────────────────┐
│  Mobile Platform │ Desktop Platform │ Desktop Platform │
│     Detected     │   Sync Context   │  Async Context   │
└─────────────────┴─────────────────┴─────────────────┘
       ↓                    ↓                    ↓
External File Ops    Create Own Runtime    External File Ops
    Required              Strategy            Required
       ↓                    ↓                    ↓
Platform Handles      Direct Archive      Platform Handles
File Operations        Operations         File Operations
       ↓                    ↓                    ↓
  Load into Memory     Operation Complete   Load into Memory
     Repository                               Repository
       ↓                    ↓                    ↓
   Operation Complete   ┌─Success─┐         Operation Complete
                       │         │
                       └─Error──┘
```

### Integration Best Practices

#### Error Handling Strategy

```rust
pub async fn robust_archive_operation<F, T>(
    operation: F,
    fallback_description: &str,
) -> Result<T, String>
where
    F: Future<Output = Result<T, HybridClientError>>,
{
    match operation.await {
        Ok(result) => Ok(result),
        Err(HybridClientError::ExternalFileOpsRequired { file_operations }) => {
            info!("External file operations required: {}", fallback_description);
            
            // Platform-specific fallback implementation
            handle_external_file_operations(file_operations).await
        }
        Err(HybridClientError::Shared(shared_error)) => {
            error!("Shared library error: {}", shared_error);
            Err(format!("Operation failed: {}", shared_error))
        }
        Err(HybridClientError::RuntimeContextError { message }) => {
            warn!("Runtime context error: {}", message);
            Err(format!("Runtime context issue: {}", message))
        }
        Err(e) => {
            error!("Unexpected error: {:?}", e);
            Err(format!("Unexpected error: {:?}", e))
        }
    }
}
```

#### Performance Monitoring

```rust
use std::time::Instant;
use tracing::{info, warn};

pub struct RuntimeMetrics {
    pub strategy_selections: HashMap<String, u64>,
    pub operation_timings: HashMap<String, Duration>,
    pub fallback_frequency: f64,
}

impl RuntimeMetrics {
    pub fn record_strategy_selection(&mut self, strategy: &str) {
        *self.strategy_selections.entry(strategy.to_string()).or_insert(0) += 1;
    }
    
    pub async fn timed_operation<F, T>(&mut self, name: &str, operation: F) -> T
    where
        F: Future<Output = T>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        
        self.operation_timings.insert(name.to_string(), duration);
        
        if duration > Duration::from_secs(1) {
            warn!("Slow operation detected: {} took {:?}", name, duration);
        }
        
        result
    }
    
    pub fn calculate_fallback_rate(&self) -> f64 {
        let total_ops: u64 = self.strategy_selections.values().sum();
        let external_ops = self.strategy_selections.get("external_file_ops").unwrap_or(&0);
        
        if total_ops > 0 {
            (*external_ops as f64) / (total_ops as f64)
        } else {
            0.0
        }
    }
}
```

### Testing Strategy for Adaptive Architecture

#### Unit Tests for Runtime Detection

```rust
#[cfg(test)]
mod adaptive_tests {
    use super::*;
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_sync_context_detection() {
        // Test without any async runtime
        let strategy = detect_runtime_context();
        assert_eq!(strategy, RuntimeStrategy::CreateOwned);
    }
    
    #[tokio::test]
    async fn test_async_context_detection() {
        // Test within async context
        let strategy = detect_runtime_context();
        assert_eq!(strategy, RuntimeStrategy::ExternalFileOps);
    }
    
    #[test]
    fn test_mobile_platform_override() {
        // This would be compiled for mobile targets
        #[cfg(target_os = "android")]
        {
            let strategy = detect_runtime_context();
            assert_eq!(strategy, RuntimeStrategy::ExternalFileOps);
        }
    }
}
```

#### Integration Tests for Platform Fallbacks

```rust
#[tokio::test]
async fn test_linux_external_file_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let archive_path = temp_dir.path().join("test.7z");
    let password = "test123".to_string();
    
    // Force external file operations context
    let client = ZipLockHybridClient::new().unwrap();
    let result = client.create_archive_adaptive(archive_path.clone(), password.clone()).await;
    
    match result {
        Ok(()) => {
            // Direct success - verify archive exists
            assert!(archive_path.exists());
        }
        Err(HybridClientError::ExternalFileOpsRequired { file_operations }) => {
            // Expected in async test context
            let mut handler = LinuxFileOperationsHandler::new();
            handler.execute_file_operations(&file_operations).await.unwrap();
            
            // Verify the archive was created
            assert!(archive_path.exists());
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
```

### Troubleshooting Guide

#### Common Runtime Detection Issues

**Problem**: Always getting external file operations required
**Solution**: Check if running in async context. Use sync wrapper if needed:

```rust
fn sync_archive_operation(path: PathBuf, password: String) -> Result<(), String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = ZipLockHybridClient::new()?;
        client.create_archive_adaptive(path, password).await
    })
}
```

**Problem**: Runtime panics in nested contexts
**Solution**: Ensure proper runtime detection:

```rust
// Good: Detects existing runtime
tokio::main!
async fn main() {
    let client = ZipLockHybridClient::new().unwrap();
    // This will detect async context and use external file ops
    let result = client.create_archive_adaptive(path, password).await;
}

// Bad: Creates nested runtime
tokio::main!
async fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { /* This creates nested runtime panic */ });
}
```

#### Platform-Specific Debugging

**Linux**: Check 7z availability and permissions
```bash
which 7z
7z --help
ls -la /tmp/ziplock_*
```

**Android**: Verify Storage Access Framework permissions
```kotlin
if (!hasStoragePermission()) {
    requestStoragePermission()
}
```

**iOS**: Check document directory access
```swift
let documentsPath = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)
print("Documents directory: \(documentsPath)")
```

This adaptive integration approach ensures that ZipLock works reliably across all platforms and runtime contexts while maintaining maximum performance and functionality.

        This comprehensive FFI integration guide provides a robust foundation for integrating ZipLock's Rust core with applications across multiple platforms, ensuring security, performance, and maintainability.