# ZipLock Mobile Integration

This document provides instructions for integrating ZipLock's shared library into iOS and Android applications using C FFI (Foreign Function Interface) bindings.

## Overview

The ZipLock shared library (`ziplock-shared`) provides a C-compatible API that allows mobile applications to access core password manager functionality while maintaining a single source of truth for cryptographic operations, data models, and business logic.

### Benefits

- **Code Reuse**: Share 80%+ of core logic between desktop and mobile platforms
- **Security**: Consistent cryptographic implementation across all platforms
- **Performance**: Native performance without cross-platform framework overhead
- **Maintainability**: Single codebase for data models and validation logic

## Architecture

```
┌─────────────────┐    ┌──────────────────┐
│   iOS App       │    │   Android App    │
│ (Swift/SwiftUI) │    │(Kotlin/Compose)  │
└─────────────────┘    └──────────────────┘
         │                       │
         └───────────────────────┼─────────────────
                                 │
                    ┌──────────────────┐
                    │  C FFI Bindings  │
                    │   (ziplock.h)    │
                    └──────────────────┘
                                 │
                    ┌──────────────────┐
                    │  Shared Library  │
                    │ (ziplock_shared) │
                    └──────────────────┘
```

## Building for Mobile

### Prerequisites

1. **Rust toolchain** with mobile targets
2. **iOS**: Xcode and iOS SDK
3. **Android**: Android NDK r21+ 

### Quick Start

```bash
# Install required targets
rustup target add aarch64-apple-ios aarch64-linux-android

# Build for iOS (ARM64)
cargo build --release --target aarch64-apple-ios --features c-api

# Build for Android (ARM64)
cargo build --release --target aarch64-linux-android --features c-api
```

### Automated Build

Use the provided build script for all platforms:

```bash
# Build for all mobile platforms
./scripts/build/build-mobile.sh all

# Build for iOS only
./scripts/build/build-mobile.sh ios

# Build for Android only
./scripts/build/build-mobile.sh android
```

## Integration Guide

### iOS Integration

1. **Add Framework**
   ```bash
   # Copy the generated XCFramework to your project
   cp mobile-builds/ios/xcframework/ZipLockCore.xcframework /path/to/ios/project/
   ```

2. **Basic Swift Usage**
   ```swift
   import ZipLockCore
   
   // Initialize library
   let core = ZipLockCore.shared
   print("ZipLock version: \(core.version)")
   
   // Create credential
   let credential = try ZipLockCredential(title: "My Login", type: "login")
   try credential.addField(name: "username", type: .username, value: "user@example.com")
   try credential.addField(name: "password", type: .password, value: "secret123")
   
   // Generate password
   if let password = ZipLockPassword.generate(length: 16) {
       print("Generated: \(password)")
   }
   ```

### Android Integration

1. **Add Libraries**
   ```bash
   # Copy libraries to your Android project
   cp -r mobile-builds/android/jniLibs/* app/src/main/jniLibs/
   ```

2. **Basic Kotlin Usage**
   ```kotlin
   import com.yourapp.ziplock.*
   
   // Initialize library
   val core = ZipLockCore.getInstance()
   println("ZipLock version: ${core.version}")
   
   // Create credential
   ZipLockCredential.create("My Login", "login").use { credential ->
       credential.addField("username", ZipLockFieldType.USERNAME, "user@example.com")
       credential.addField("password", ZipLockFieldType.PASSWORD, "secret123")
   }
   
   // Generate password
   val password = ZipLockPassword.generate(length = 16)
   println("Generated: $password")
   ```

## API Reference

### Core Functions

| Function | Description |
|----------|-------------|
| `ziplock_init()` | Initialize the library |
| `ziplock_get_version()` | Get library version |
| `ziplock_credential_new()` | Create new credential |
| `ziplock_password_generate()` | Generate secure password |
| `ziplock_email_validate()` | Validate email format |

### Memory Management

**Critical**: All returned pointers must be freed using the appropriate `*_free` functions:

```c
// Example proper memory management
char* password = ziplock_password_generate(16, 1, 1, 1, 1);
if (password != NULL) {
    // Use password...
    ziplock_string_free(password);  // Always free!
}
```

### Error Handling

Functions return error codes:
- `0` = Success
- Negative values = Error codes (see `ziplock_error_t` enum)

```c
int result = ziplock_credential_add_field(cred, "username", 4, "user", NULL, 0);
if (result != 0) {
    // Handle error
    char* error = ziplock_get_last_error();
    // ... process error ...
    ziplock_string_free(error);
}
```

## Security Considerations

### Memory Safety
- Always check for NULL pointers before use
- Free all allocated memory using provided functions
- Use RAII patterns (Swift `deinit`, Kotlin `AutoCloseable`)

### Threading
- C API functions are thread-safe for read operations
- Serialize write operations to the same credential object
- Consider using platform-specific threading primitives

### String Handling
- All strings are UTF-8 encoded and null-terminated
- Validate input strings on the mobile side
- Never access freed string pointers

## Testing

### Running Tests

```bash
# Test the C API
cargo test --features c-api

# Test specific FFI functions
cargo test ffi::tests --features c-api
```

### Integration Testing

Example iOS test:
```swift
func testZipLockIntegration() {
    XCTAssertNotEqual(ZipLockCore.shared.version, "Unknown")
    
    do {
        let credential = try ZipLockCredential(title: "Test", type: "login")
        try credential.addField(name: "username", type: .username, value: "test")
        XCTAssertEqual(credential.getField(name: "username"), "test")
    } catch {
        XCTFail("Integration test failed: \(error)")
    }
}
```

Example Android test:
```kotlin
@Test
fun testZipLockIntegration() {
    assertNotEquals("Unknown", ZipLockCore.getInstance().version)
    
    ZipLockCredential.create("Test", "login").use { credential ->
        credential.addField("username", ZipLockFieldType.USERNAME, "test")
        assertEquals("test", credential.getField("username"))
    }
}
```

## Troubleshooting

### Common Issues

1. **Library not loading**
   - Verify the library is included in your app bundle
   - Check target architecture matches device architecture
   - Ensure `ziplock_init()` is called before other functions

2. **Build failures**
   - Verify Rust targets are installed: `rustup target list --installed`
   - Check Android NDK path: `echo $ANDROID_NDK_HOME`
   - Ensure correct iOS SDK version

3. **Runtime crashes**
   - Check for NULL pointer dereferences
   - Verify proper memory management
   - Enable debug logging: `ziplock_debug_logging(1)`

### Debug Logging

Enable debug output to diagnose issues:

```swift
// iOS
ZipLockCore.shared.enableDebugLogging(true)
```

```kotlin
// Android
ZipLockCore.getInstance().enableDebugLogging(true)
```

## Performance Tips

1. **Minimize FFI calls** - Batch operations when possible
2. **Cache results** - Store frequently accessed data on mobile side
3. **Background processing** - Use async patterns for long operations
4. **Memory pooling** - Reuse credential objects when appropriate

## Examples

Complete working examples are available in:
- [Mobile Integration Guide](mobile-integration.md) - Complete iOS and Android integration examples
- [Mobile Integration](mobile-integration.md) - Platform-specific integration examples

## Support

For integration issues:
1. Check the troubleshooting section above
2. Review the integration examples in the [Mobile Integration Guide](mobile-integration.md)
3. Run tests with `cargo test --features c-api`
4. Enable debug logging for detailed diagnostics

## Next Steps

The C FFI provides a foundation for:
- Repository management (encrypted archive operations)
- Advanced search and filtering
- Biometric authentication integration
- Auto-fill service implementation
- Synchronization capabilities

This approach ensures ZipLock can deliver a consistent, secure experience across all platforms while maintaining code quality and security standards.