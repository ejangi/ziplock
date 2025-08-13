# ZipLock Mobile Integration Guide

This document provides a comprehensive guide for integrating ZipLock's core functionality into iOS and Android applications using the C FFI (Foreign Function Interface) bindings.

## Overview

ZipLock provides a C-compatible API that allows mobile applications to access the core password manager functionality written in Rust. This approach enables:

- **Code Reuse**: Share core logic between desktop and mobile platforms
- **Security**: Benefit from Rust's memory safety and the proven cryptographic implementation
- **Performance**: Native performance without the overhead of cross-platform frameworks
- **Consistency**: Identical data models and validation logic across all platforms

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   iOS App       │    │   Android App    │    │  Desktop Apps   │
│ (Swift/SwiftUI) │    │(Kotlin/Compose)  │    │     (Rust)      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │  C FFI Bindings  │
                    │   (ziplock.h)    │
                    └──────────────────┘
                                 │
                    ┌──────────────────┐
                    │  Shared Library  │
                    │     (Rust)       │
                    └──────────────────┘
```

## Building the Shared Library

### For iOS

```bash
# Install Rust targets for iOS
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Build for iOS device (ARM64)
cd ziplock/shared
cargo build --release --target aarch64-apple-ios --features c-api

# Build for iOS simulator (Intel)
cargo build --release --target x86_64-apple-ios --features c-api

# Build for iOS simulator (Apple Silicon)
cargo build --release --target aarch64-apple-ios-sim --features c-api

# Create universal library for simulator
lipo -create \
  target/x86_64-apple-ios/release/libziplock_shared.a \
  target/aarch64-apple-ios-sim/release/libziplock_shared.a \
  -output target/ios-sim-universal/libziplock_shared.a

# Create XCFramework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libziplock_shared.a \
  -headers include/ \
  -library target/ios-sim-universal/libziplock_shared.a \
  -headers include/ \
  -output ZipLockCore.xcframework
```

### For Android

```bash
# Install Android NDK targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Set up environment (adjust paths as needed)
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

# Configure Cargo for cross-compilation
cat >> ~/.cargo/config.toml << EOF
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android21-clang"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi21-clang"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android21-clang"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android21-clang"
EOF

# Build for Android architectures
cd ziplock/shared
cargo build --release --target aarch64-linux-android --features c-api
cargo build --release --target armv7-linux-androideabi --features c-api
cargo build --release --target x86_64-linux-android --features c-api
cargo build --release --target i686-linux-android --features c-api

# Copy libraries to Android project
mkdir -p android/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86_64,x86}
cp target/aarch64-linux-android/release/libziplock_shared.so android/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/libziplock_shared.so android/src/main/jniLibs/armeabi-v7a/
cp target/x86_64-linux-android/release/libziplock_shared.so android/src/main/jniLibs/x86_64/
cp target/i686-linux-android/release/libziplock_shared.so android/src/main/jniLibs/x86/
```

## iOS Integration

### 1. Add Library to Xcode Project

1. Drag the `ZipLockCore.xcframework` into your Xcode project
2. Add the framework to your target's "Frameworks, Libraries, and Embedded Content"
3. Add the `ziplock.h` header to your bridging header or import it in your wrapper

### 2. Create Swift Wrapper

```swift
import Foundation

// MARK: - Error Handling
enum ZipLockError: Error {
    case invalidPointer
    case invalidString
    case validationFailed
    case internalError(String)
    
    init(code: Int32) {
        switch code {
        case -1: self = .invalidPointer
        case -2: self = .invalidString
        case -4: self = .validationFailed
        default: self = .internalError("Unknown error code: \(code)")
        }
    }
}

// MARK: - Core Library
class ZipLockCore {
    static let shared = ZipLockCore()
    
    private init() {
        let result = ziplock_init()
        if result != 0 {
            fatalError("Failed to initialize ZipLock library: \(result)")
        }
    }
    
    var version: String {
        guard let cString = ziplock_get_version() else { return "Unknown" }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }
}

// MARK: - Credential Management
class ZipLockCredential {
    private let handle: OpaquePointer
    
    init(title: String, type: String) throws {
        guard let handle = ziplock_credential_new(title, type) else {
            throw ZipLockError.internalError("Failed to create credential")
        }
        self.handle = handle
    }
    
    deinit {
        ziplock_credential_free(handle)
    }
    
    func addField(name: String, type: ZipLockFieldType, value: String, label: String? = nil, sensitive: Bool = false) throws {
        let result = ziplock_credential_add_field(
            handle,
            name,
            type.rawValue,
            value,
            label,
            sensitive ? 1 : 0
        )
        
        if result != 0 {
            throw ZipLockError(code: result)
        }
    }
    
    func getField(name: String) -> String? {
        guard let cString = ziplock_credential_get_field(handle, name) else { return nil }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }
    
    func addTag(_ tag: String) throws {
        let result = ziplock_credential_add_tag(handle, tag)
        if result != 0 {
            throw ZipLockError(code: result)
        }
    }
}

// MARK: - Field Types
enum ZipLockFieldType: Int32 {
    case text = 0
    case password = 1
    case email = 2
    case url = 3
    case username = 4
    case phone = 5
    case creditCardNumber = 6
    case expiryDate = 7
    case cvv = 8
    case totpSecret = 9
    case textArea = 10
    case number = 11
    case date = 12
    case custom = 13
}

// MARK: - Password Utilities
class ZipLockPassword {
    static func generate(
        length: UInt32 = 16,
        includeUppercase: Bool = true,
        includeLowercase: Bool = true,
        includeNumbers: Bool = true,
        includeSymbols: Bool = true
    ) -> String? {
        guard let cString = ziplock_password_generate(
            length,
            includeUppercase ? 1 : 0,
            includeLowercase ? 1 : 0,
            includeNumbers ? 1 : 0,
            includeSymbols ? 1 : 0
        ) else { return nil }
        
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }
    
    static func validate(_ password: String) -> PasswordStrength? {
        guard let result = ziplock_password_validate(password) else { return nil }
        defer { ziplock_password_strength_free(result) }
        
        let strengthData = result.pointee
        let description = String(cString: strengthData.description)
        
        return PasswordStrength(
            level: PasswordStrengthLevel(rawValue: strengthData.level) ?? .veryWeak,
            score: strengthData.score,
            description: description
        )
    }
}

struct PasswordStrength {
    let level: PasswordStrengthLevel
    let score: UInt32
    let description: String
}

enum PasswordStrengthLevel: Int32 {
    case veryWeak = 0
    case weak = 1
    case fair = 2
    case good = 3
    case strong = 4
}

// MARK: - Validation Utilities
class ZipLockValidation {
    static func isValidEmail(_ email: String) -> Bool {
        return ziplock_email_validate(email) == 1
    }
    
    static func isValidURL(_ url: String) -> Bool {
        return ziplock_url_validate(url) == 1
    }
}
```

### 3. Usage Example

```swift
import SwiftUI

struct ContentView: View {
    @State private var credentials: [ZipLockCredential] = []
    @State private var password = ""
    @State private var strength: PasswordStrength?
    
    var body: some View {
        VStack {
            Text("ZipLock Core v\(ZipLockCore.shared.version)")
            
            TextField("Password", text: $password)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .onChange(of: password) { newValue in
                    strength = ZipLockPassword.validate(newValue)
                }
            
            if let strength = strength {
                HStack {
                    Text("Strength: \(strength.description)")
                    Text("Score: \(strength.score)")
                }
            }
            
            Button("Generate Password") {
                if let generated = ZipLockPassword.generate() {
                    password = generated
                }
            }
            
            Button("Create Credential") {
                createSampleCredential()
            }
        }
        .padding()
    }
    
    private func createSampleCredential() {
        do {
            let credential = try ZipLockCredential(title: "Example Login", type: "login")
            try credential.addField(name: "username", type: .username, value: "user@example.com")
            try credential.addField(name: "password", type: .password, value: password, sensitive: true)
            try credential.addTag("example")
            
            credentials.append(credential)
        } catch {
            print("Error creating credential: \(error)")
        }
    }
}
```

## Android Integration

### 1. Add Library to Android Project

Add the following to your `app/build.gradle`:

```gradle
android {
    // ... existing configuration
    
    sourceSets {
        main {
            jniLibs.srcDirs = ['src/main/jniLibs']
        }
    }
}
```

### 2. Create Kotlin Wrapper

```kotlin
package com.example.ziplock

import java.io.File

// MARK: - Error Handling
sealed class ZipLockError : Exception() {
    object InvalidPointer : ZipLockError()
    object InvalidString : ZipLockError()
    object ValidationFailed : ZipLockError()
    data class InternalError(val code: Int) : ZipLockError()
    
    companion object {
        fun fromCode(code: Int): ZipLockError = when (code) {
            -1 -> InvalidPointer
            -2 -> InvalidString
            -4 -> ValidationFailed
            else -> InternalError(code)
        }
    }
}

// MARK: - Native Interface
object ZipLockNative {
    init {
        System.loadLibrary("ziplock_shared")
    }
    
    // Library management
    external fun ziplock_init(): Int
    external fun ziplock_get_version(): String?
    external fun ziplock_string_free(ptr: Long)
    
    // Credential management
    external fun ziplock_credential_new(title: String, type: String): Long
    external fun ziplock_credential_free(handle: Long)
    external fun ziplock_credential_add_field(
        handle: Long,
        name: String,
        fieldType: Int,
        value: String,
        label: String?,
        sensitive: Int
    ): Int
    external fun ziplock_credential_get_field(handle: Long, name: String): String?
    external fun ziplock_credential_add_tag(handle: Long, tag: String): Int
    
    // Password utilities
    external fun ziplock_password_generate(
        length: Int,
        includeUppercase: Int,
        includeLowercase: Int,
        includeNumbers: Int,
        includeSymbols: Int
    ): String?
    
    // Validation
    external fun ziplock_email_validate(email: String): Int
    external fun ziplock_url_validate(url: String): Int
}

// MARK: - Core Library
class ZipLockCore private constructor() {
    companion object {
        val instance: ZipLockCore by lazy { ZipLockCore() }
    }
    
    init {
        val result = ZipLockNative.ziplock_init()
        if (result != 0) {
            throw RuntimeException("Failed to initialize ZipLock library: $result")
        }
    }
    
    val version: String
        get() = ZipLockNative.ziplock_get_version() ?: "Unknown"
}

// MARK: - Credential Management
class ZipLockCredential(title: String, type: String) : AutoCloseable {
    private val handle: Long = ZipLockNative.ziplock_credential_new(title, type)
        .takeIf { it != 0L } ?: throw ZipLockError.InternalError(-1)
    
    fun addField(
        name: String,
        type: ZipLockFieldType,
        value: String,
        label: String? = null,
        sensitive: Boolean = false
    ) {
        val result = ZipLockNative.ziplock_credential_add_field(
            handle, name, type.value, value, label, if (sensitive) 1 else 0
        )
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }
    
    fun getField(name: String): String? {
        return ZipLockNative.ziplock_credential_get_field(handle, name)
    }
    
    fun addTag(tag: String) {
        val result = ZipLockNative.ziplock_credential_add_tag(handle, tag)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }
    
    override fun close() {
        ZipLockNative.ziplock_credential_free(handle)
    }
}

// MARK: - Field Types
enum class ZipLockFieldType(val value: Int) {
    TEXT(0),
    PASSWORD(1),
    EMAIL(2),
    URL(3),
    USERNAME(4),
    PHONE(5),
    CREDIT_CARD_NUMBER(6),
    EXPIRY_DATE(7),
    CVV(8),
    TOTP_SECRET(9),
    TEXT_AREA(10),
    NUMBER(11),
    DATE(12),
    CUSTOM(13)
}

// MARK: - Password Utilities
object ZipLockPassword {
    fun generate(
        length: Int = 16,
        includeUppercase: Boolean = true,
        includeLowercase: Boolean = true,
        includeNumbers: Boolean = true,
        includeSymbols: Boolean = true
    ): String? {
        return ZipLockNative.ziplock_password_generate(
            length,
            if (includeUppercase) 1 else 0,
            if (includeLowercase) 1 else 0,
            if (includeNumbers) 1 else 0,
            if (includeSymbols) 1 else 0
        )
    }
}

// MARK: - Validation Utilities
object ZipLockValidation {
    fun isValidEmail(email: String): Boolean {
        return ZipLockNative.ziplock_email_validate(email) == 1
    }
    
    fun isValidURL(url: String): Boolean {
        return ZipLockNative.ziplock_url_validate(url) == 1
    }
}
```

### 3. JNI Implementation

Create `jni_bindings.c` in your Android project:

```c
#include <jni.h>
#include "ziplock.h"

// Helper function to create Java string from C string
jstring create_jstring(JNIEnv *env, const char *str) {
    if (str == NULL) return NULL;
    jstring result = (*env)->NewStringUTF(env, str);
    ziplock_string_free((char*)str);
    return result;
}

// Library management
JNIEXPORT jint JNICALL
Java_com_example_ziplock_ZipLockNative_ziplock_1init(JNIEnv *env, jobject thiz) {
    return ziplock_init();
}

JNIEXPORT jstring JNICALL
Java_com_example_ziplock_ZipLockNative_ziplock_1get_1version(JNIEnv *env, jobject thiz) {
    return create_jstring(env, ziplock_get_version());
}

// Credential management
JNIEXPORT jlong JNICALL
Java_com_example_ziplock_ZipLockNative_ziplock_1credential_1new(
    JNIEnv *env, jobject thiz, jstring title, jstring type) {
    
    const char *title_str = (*env)->GetStringUTFChars(env, title, NULL);
    const char *type_str = (*env)->GetStringUTFChars(env, type, NULL);
    
    jlong result = (jlong)ziplock_credential_new(title_str, type_str);
    
    (*env)->ReleaseStringUTFChars(env, title, title_str);
    (*env)->ReleaseStringUTFChars(env, type, type_str);
    
    return result;
}

JNIEXPORT void JNICALL
Java_com_example_ziplock_ZipLockNative_ziplock_1credential_1free(
    JNIEnv *env, jobject thiz, jlong handle) {
    ziplock_credential_free((ziplock_credential_t*)handle);
}

// Password utilities
JNIEXPORT jstring JNICALL
Java_com_example_ziplock_ZipLockNative_ziplock_1password_1generate(
    JNIEnv *env, jobject thiz,
    jint length, jint uppercase, jint lowercase, jint numbers, jint symbols) {
    
    return create_jstring(env, ziplock_password_generate(
        (uint32_t)length, uppercase, lowercase, numbers, symbols));
}

// Add more JNI functions as needed...
```

### 4. Usage Example

```kotlin
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        setContent {
            ZipLockApp()
        }
    }
}

@Composable
fun ZipLockApp() {
    var password by remember { mutableStateOf("") }
    var credentials by remember { mutableStateOf(listOf<ZipLockCredential>()) }
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        Text("ZipLock Core v${ZipLockCore.instance.version}")
        
        OutlinedTextField(
            value = password,
            onValueChange = { password = it },
            label = { Text("Password") }
        )
        
        Button(
            onClick = {
                ZipLockPassword.generate()?.let { password = it }
            }
        ) {
            Text("Generate Password")
        }
        
        Button(
            onClick = {
                createSampleCredential()
            }
        ) {
            Text("Create Credential")
        }
    }
}

private fun createSampleCredential() {
    try {
        ZipLockCredential("Example Login", "login").use { credential ->
            credential.addField("username", ZipLockFieldType.USERNAME, "user@example.com")
            credential.addField("password", ZipLockFieldType.PASSWORD, "secret", sensitive = true)
            credential.addTag("example")
        }
    } catch (e: ZipLockError) {
        Log.e("ZipLock", "Error creating credential", e)
    }
}
```

## Security Considerations

### Memory Management
- Always call the appropriate `*_free` functions to prevent memory leaks
- Use RAII patterns (Swift's `deinit`, Kotlin's `AutoCloseable`)
- Never access freed pointers

### String Handling
- All strings are UTF-8 encoded
- Always check for null pointers before using returned strings
- Free strings immediately after converting to native types

### Error Handling
- Always check return codes from functions
- Handle errors gracefully and provide user feedback
- Log errors for debugging but don't expose sensitive information

### Threading
- The C API is thread-safe for read operations
- Serialize write operations to the same credential
- Consider using platform-specific threading primitives for coordination

## Testing

### Unit Testing

Create test functions to verify the integration:

```swift
// iOS Tests
func testZipLockIntegration() {
    XCTAssertNotEqual(ZipLockCore.shared.version, "Unknown")
    
    do {
        let credential = try ZipLockCredential(title: "Test", type: "login")
        try credential.addField(name: "username", type: .username, value: "test")
        
        let username = credential.getField(name: "username")
        XCTAssertEqual(username, "test")
    } catch {
        XCTFail("Integration test failed: \(error)")
    }
}
```

```kotlin
// Android Tests
@Test
fun testZipLockIntegration() {
    assertNotEquals("Unknown", ZipLockCore.instance.version)
    
    ZipLockCredential("Test", "login").use { credential ->
        credential.addField("username", ZipLockFieldType.USERNAME, "test")
        assertEquals("test", credential.getField("username"))
    }
}
```

## Troubleshooting

### Common Issues

1. **Library not loading**: Ensure the shared library is properly included in your app bundle
2. **Crashes on function calls**: Check that `ziplock_init()` was called successfully
3. **Memory leaks**: Verify all allocated objects are being freed
4. **Build errors**: Ensure correct target architectures and NDK versions

### Debug Logging

Enable debug logging to diagnose issues:

```swift
// iOS
ZipLockCore.enableDebugLogging(true)
```

```kotlin
// Android
ZipLockNative.ziplock_debug_logging(1)
```

## Performance Considerations

- Minimize FFI boundary crossings for performance-critical operations
- Cache frequently accessed data on the mobile side
- Use batch operations when possible
- Consider background threading for long-running operations

## Future Enhancements

The C API provides a foundation for:
- Repository management (opening/saving encrypted archives)
- Advanced search functionality
- Synchronization capabilities
- Biometric authentication integration
- Auto-fill service integration

This mobile integration approach ensures that ZipLock can provide a consistent, secure, and performant experience across all platforms while maintaining a single source of truth for core functionality.