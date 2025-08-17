# ZipLock Mobile Integration Guide

This document provides a comprehensive guide for integrating ZipLock's core functionality into iOS and Android applications using the unified FFI architecture. This is the same approach used by all ZipLock platforms for maximum consistency and maintainability.

## Overview

ZipLock uses a **unified architecture** where all platforms (desktop and mobile) communicate directly with a shared Rust core library through C FFI bindings. This eliminates platform-specific backend services and provides:

- **True Code Reuse**: Identical core implementation across ALL platforms
- **Security**: Rust's memory safety with isolated cryptographic operations
- **Performance**: Direct function calls eliminate IPC overhead
- **Consistency**: Same business logic, validation, and behavior everywhere
- **Simplicity**: No background services, sockets, or complex communication protocols

## Unified Architecture

All ZipLock platforms use the same direct FFI integration pattern:

```
┌─────────────────┐    Direct    ┌─────────────────┐    File I/O   ┌─────────────────┐
│  Frontend UI    │    FFI       │   Shared Core   │ ◄────────────► │ Encrypted 7z    │
│                 │ ◄─────────► │    Library      │               │ Archive         │
│ • Linux (Rust)  │             │     (Rust)      │               │                 │
│ • Windows(Rust) │             │                 │               │                 │
│ • iOS (Swift)   │             │ • Archive Ops   │               │                 │
│ • Android(Kotlin│             │ • Cryptography  │               │                 │
│ • macOS (Swift) │             │ • Validation    │               │                 │
│                 │             │ • C FFI API     │               │                 │
└─────────────────┘             │ • Data Models   │               └─────────────────┘
                                │ • Session Mgmt  │
                                └─────────────────┘
```

### Key Benefits of Unified Architecture

- **No Platform-Specific Backends**: Mobile apps work exactly like desktop apps
- **Consistent Behavior**: Same archive format, validation, and operations everywhere
- **Simplified Testing**: One implementation to test across all platforms
- **Faster Development**: Mobile platforms benefit from all desktop improvements automatically

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


        guard let cString = ziplock_password_generate(
            length,
            includeUppercase ? 1 : 0,
            includeLowercase ? 1 : 0,
            includeNumbers ? 1 : 0,
            includeSymbols ? 1 : 0
        ) else {
            return nil
        }

        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    static func validate(_ password: String) -> PasswordStrength? {
        guard let result = ziplock_password_validate(password) else {
            return nil
        }
        defer { ziplock_password_strength_free(result) }

        let strengthData = result.pointee
        let description = String(cString: strengthData.description)

        guard let level = PasswordStrength.Level(rawValue: strengthData.level) else {
            return nil
        }

        return PasswordStrength(
            level: level,
            score: strengthData.score,
            description: description
        )
    }
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

// MARK: - Utility Functions

class ZipLockUtils {
    static func formatCreditCard(_ cardNumber: String) -> String? {
        guard let cString = ziplock_credit_card_format(cardNumber) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    static func generateTOTP(secret: String, timeStep: UInt32 = 30) -> String? {
        guard let cString = ziplock_totp_generate(secret, timeStep) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    static func testEcho(_ input: String) -> String? {
        guard let cString = ziplock_test_echo(input) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }
}

// MARK: - SwiftUI Integration Example

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

### 3. Complete iOS Example

Here's a comprehensive iOS implementation demonstrating all major ZipLock integration patterns:

```swift
//
//  ios-example.swift
//  ZipLock Mobile FFI Example
//
//  Example demonstrating how to use ZipLock's C API from iOS Swift applications.
//  This file shows the complete integration pattern including error handling,
//  memory management, and proper Swift idioms.
//

import Foundation

// MARK: - Error Types

enum ZipLockError: Error, LocalizedError {
    case initializationFailed(Int32)
    case invalidPointer
    case invalidString
    case fieldError(String)
    case validationFailed(String)
    case internalError(String)

    var errorDescription: String? {
        switch self {
        case .initializationFailed(let code):
            return "Failed to initialize ZipLock library with error code: \(code)"
        case .invalidPointer:
            return "Invalid pointer passed to ZipLock function"
        case .invalidString:
            return "Invalid string encoding"
        case .fieldError(let message):
            return "Field error: \(message)"
        case .validationFailed(let message):
            return "Validation failed: \(message)"
        case .internalError(let message):
            return "Internal ZipLock error: \(message)"
        }
    }

    init(code: Int32) {
        switch code {
        case -1: self = .invalidPointer
        case -2: self = .invalidString
        case -3: self = .fieldError("Invalid field")
        case -4: self = .validationFailed("Validation failed")
        default: self = .internalError("Error code: \(code)")
        }
    }
}

// MARK: - Field Types

enum ZipLockFieldType: Int32, CaseIterable {
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

    var displayName: String {
        switch self {
        case .text: return "Text"
        case .password: return "Password"
        case .email: return "Email"
        case .url: return "URL"
        case .username: return "Username"
        case .phone: return "Phone"
        case .creditCardNumber: return "Credit Card"
        case .expiryDate: return "Expiry Date"
        case .cvv: return "CVV"
        case .totpSecret: return "TOTP Secret"
        case .textArea: return "Text Area"
        case .number: return "Number"
        case .date: return "Date"
        case .custom: return "Custom"
        }
    }

    var isSensitiveByDefault: Bool {
        switch self {
        case .password, .cvv, .totpSecret:
            return true
        default:
            return false
        }
    }
}

// MARK: - Password Strength

struct PasswordStrength {
    enum Level: Int32 {
        case veryWeak = 0
        case weak = 1
        case fair = 2
        case good = 3
        case strong = 4

        var description: String {
            switch self {
            case .veryWeak: return "Very Weak"
            case .weak: return "Weak"
            case .fair: return "Fair"
            case .good: return "Good"
            case .strong: return "Strong"
            }
        }

        var color: String {
            switch self {
            case .veryWeak: return "#FF4444"
            case .weak: return "#FF8800"
            case .fair: return "#FFBB00"
            case .good: return "#88BB00"
            case .strong: return "#44BB44"
            }
        }
    }

    let level: Level
    let score: UInt32
    let description: String
}

// MARK: - Core Library Manager

class ZipLockCore {
    static let shared = ZipLockCore()

    private init() {
        let result = ziplock_init()
        if result != 0 {
            fatalError("Failed to initialize ZipLock library: \(result)")
        }
    }

    var version: String {
        guard let cString = ziplock_get_version() else {
            return "Unknown"
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    func enableDebugLogging(_ enabled: Bool) {
        _ = ziplock_debug_logging(enabled ? 1 : 0)
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

    convenience init(fromTemplate template: String, title: String) throws {
        guard let handle = ziplock_credential_from_template(template, title) else {
            throw ZipLockError.internalError("Failed to create credential from template")
        }
        self.handle = handle
    }

    deinit {
        ziplock_credential_free(handle)
    }

    func addField(
        name: String,
        type: ZipLockFieldType,
        value: String,
        label: String? = nil,
        sensitive: Bool? = nil
    ) throws {
        let isSensitive = sensitive ?? type.isSensitiveByDefault
        let result = ziplock_credential_add_field(
            handle,
            name,
            type.rawValue,
            value,
            label,
            isSensitive ? 1 : 0
        )

        if result != 0 {
            throw ZipLockError(code: result)
        }
    }

    func getField(name: String) -> String? {
        guard let cString = ziplock_credential_get_field(handle, name) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    func removeField(name: String) throws {
        let result = ziplock_credential_remove_field(handle, name)
        if result != 0 {
            throw ZipLockError(code: result)
        }
    }

    func addTag(_ tag: String) throws {
        let result = ziplock_credential_add_tag(handle, tag)
        if result != 0 {
            throw ZipLockError(code: result)
        }
    }

    func removeTag(_ tag: String) throws {
        let result = ziplock_credential_remove_tag(handle, tag)
        if result != 0 {
            throw ZipLockError(code: result)
        }
    }

    func hasTag(_ tag: String) -> Bool {
        let result = ziplock_credential_has_tag(handle, tag)
        return result == 1
    }

    func validate() throws {
        guard let validationResult = ziplock_credential_validate(handle) else {
            throw ZipLockError.internalError("Failed to validate credential")
        }
        defer { ziplock_validation_result_free(validationResult) }

        let result = validationResult.pointee
        if result.is_valid == 0 {
            var errors: [String] = []
            if result.error_count > 0 && result.errors != nil {
                for i in 0..<Int(result.error_count) {
                    if let errorPtr = result.errors.advanced(by: i).pointee {
                        errors.append(String(cString: errorPtr))
                    }
                }
            }
            let errorMessage = errors.isEmpty ? "Unknown validation error" : errors.joined(separator: ", ")
            throw ZipLockError.validationFailed(errorMessage)
        }
    }
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
        ) else {
            return nil
        }

        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    static func validate(_ password: String) -> PasswordStrength? {
        guard let result = ziplock_password_validate(password) else {
            return nil
        }
        defer { ziplock_password_strength_free(result) }

        let strengthData = result.pointee
        let description = String(cString: strengthData.description)

        guard let level = PasswordStrength.Level(rawValue: strengthData.level) else {
            return nil
        }

        return PasswordStrength(
            level: level,
            score: strengthData.score,
            description: description
        )
    }
}

// MARK: - Validation and Utility Functions

class ZipLockValidation {
    static func isValidEmail(_ email: String) -> Bool {
        return ziplock_email_validate(email) == 1
    }

    static func isValidURL(_ url: String) -> Bool {
        return ziplock_url_validate(url) == 1
    }
}

class ZipLockUtils {
    static func formatCreditCard(_ cardNumber: String) -> String? {
        guard let cString = ziplock_credit_card_format(cardNumber) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    static func generateTOTP(secret: String, timeStep: UInt32 = 30) -> String? {
        guard let cString = ziplock_totp_generate(secret, timeStep) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }

    static func testEcho(_ input: String) -> String? {
        guard let cString = ziplock_test_echo(input) else {
            return nil
        }
        defer { ziplock_string_free(cString) }
        return String(cString: cString)
    }
}

// MARK: - SwiftUI Integration Example

#if canImport(SwiftUI)
import SwiftUI

@available(iOS 13.0, *)
struct ZipLockExampleView: View {
    @State private var password = ""
    @State private var passwordStrength: PasswordStrength?
    @State private var generatedPassword = ""
    @State private var email = ""
    @State private var isEmailValid = false

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Library Info")) {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text(ZipLockCore.shared.version)
                            .foregroundColor(.secondary)
                    }
                }

                Section(header: Text("Password Testing")) {
                    TextField("Enter password", text: $password)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .onChange(of: password) { newValue in
                            passwordStrength = ZipLockPassword.validate(newValue)
                        }

                    if let strength = passwordStrength {
                        HStack {
                            Text("Strength:")
                            Text(strength.level.description)
                                .foregroundColor(Color(hex: strength.level.color))
                            Spacer()
                            Text("\(strength.score)/100")
                                .foregroundColor(.secondary)
                        }
                    }

                    Button("Generate Password") {
                        if let generated = ZipLockPassword.generate() {
                            generatedPassword = generated
                            password = generated
                        }
                    }

                    if !generatedPassword.isEmpty {
                        Text("Generated: \(generatedPassword)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                Section(header: Text("Email Validation")) {
                    TextField("Enter email", text: $email)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .onChange(of: email) { newValue in
                            isEmailValid = ZipLockValidation.isValidEmail(newValue)
                        }

                    HStack {
                        Text("Valid:")
                        Image(systemName: isEmailValid ? "checkmark.circle.fill" : "xmark.circle.fill")
                            .foregroundColor(isEmailValid ? .green : .red)
                        Spacer()
                    }
                }

                Section(header: Text("Test Functions")) {
                    Button("Run All Tests") {
                        runAllTests()
                    }

                    Button("Create Sample Credential") {
                        createSampleCredential()
                    }
                }
            }
            .navigationTitle("ZipLock FFI Demo")
        }
    }

    private func runAllTests() {
        print("Running ZipLock FFI Tests...")
        
        // Test echo function
        if let echo = ZipLockUtils.testEcho("Hello from iOS!") {
            print("✓ Echo test: \(echo)")
        }
        
        // Test credit card formatting
        if let formatted = ZipLockUtils.formatCreditCard("1234567890123456") {
            print("✓ Credit card formatted: \(formatted)")
        }
        
        // Test TOTP generation
        if let totp = ZipLockUtils.generateTOTP(secret: "JBSWY3DPEHPK3PXP") {
            print("✓ TOTP generated: \(totp)")
        }
    }

    private func createSampleCredential() {
        do {
            let credential = try ZipLockCredential(title: "Example Login", type: "login")
            try credential.addField(name: "username", type: .username, value: "user@example.com")
            try credential.addField(name: "password", type: .password, value: "SuperSecure123!")
            try credential.addField(name: "website", type: .url, value: "https://example.com")
            try credential.addTag("example")
            try credential.addTag("ios")
            try credential.validate()
            print("✓ Sample credential created and validated")
        } catch {
            print("✗ Error creating credential: \(error)")
        }
    }
}

@available(iOS 13.0, *)
extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6: // RGB (24-bit)
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // ARGB (32-bit)
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (1, 1, 1, 0)
        }

        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue:  Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}

#endif

// MARK: - Template Examples

extension ZipLockCredential {
    static func createLoginCredential(
        title: String,
        username: String,
        password: String,
        website: String
    ) throws -> ZipLockCredential {
        let credential = try ZipLockCredential(fromTemplate: "login", title: title)
        try credential.addField(name: "username", type: .username, value: username)
        try credential.addField(name: "password", type: .password, value: password)
        try credential.addField(name: "website", type: .url, value: website)
        return credential
    }

    static func createCreditCardCredential(
        title: String,
        cardNumber: String,
        expiryDate: String,
        cvv: String,
        cardholderName: String
    ) throws -> ZipLockCredential {
        let credential = try ZipLockCredential(fromTemplate: "credit_card", title: title)
        try credential.addField(name: "card_number", type: .creditCardNumber, value: cardNumber)
        try credential.addField(name: "expiry_date", type: .expiryDate, value: expiryDate)
        try credential.addField(name: "cvv", type: .cvv, value: cvv)
        try credential.addField(name: "cardholder_name", type: .text, value: cardholderName)
        return credential
    }

    static func createSecureNoteCredential(title: String, content: String) throws -> ZipLockCredential {
        let credential = try ZipLockCredential(fromTemplate: "secure_note", title: title)
        try credential.addField(name: "content", type: .textArea, value: content)
        return credential
    }
}

// MARK: - CommonTemplates FFI Integration

/// C structures for FFI interop
struct CCredentialTemplate {
    var name: UnsafeMutablePointer<CChar>?
    var description: UnsafeMutablePointer<CChar>?
    var field_count: Int32
    var fields: UnsafeMutablePointer<CFieldTemplate>?
    var tag_count: Int32
    var tags: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
}

struct CFieldTemplate {
    var name: UnsafeMutablePointer<CChar>?
    var field_type: UnsafeMutablePointer<CChar>?
    var label: UnsafeMutablePointer<CChar>?
    var required: Int32
    var sensitive: Int32
    var default_value: UnsafeMutablePointer<CChar>?
    var validation_min_length: Int32
    var validation_max_length: Int32
    var validation_pattern: UnsafeMutablePointer<CChar>?
    var validation_message: UnsafeMutablePointer<CChar>?
}

/// Credential template structure from FFI
struct CredentialTemplate {
    let name: String
    let description: String
    let fields: [FieldTemplate]
    let defaultTags: [String]
}

/// Field template structure from FFI
struct FieldTemplate {
    let name: String
    let fieldType: String
    let label: String
    let required: Bool
    let sensitive: Bool
    let defaultValue: String?
    let validation: FieldValidation?
}

/// Field validation structure from FFI
struct FieldValidation {
    let minLength: Int?
    let maxLength: Int?
    let pattern: String?
    let message: String?
}

/// FFI functions for template access
extension ZipLockCore {
    // External FFI declarations
    private static let ziplock_templates_get_all: @convention(c) (UnsafeMutablePointer<UnsafeMutablePointer<CCredentialTemplate>?>?, UnsafeMutablePointer<Int32>?) -> Int32 = dlsym(dlopen(nil, RTLD_LAZY), "ziplock_templates_get_all").assumingMemoryBound(to: (@convention(c) (UnsafeMutablePointer<UnsafeMutablePointer<CCredentialTemplate>?>?, UnsafeMutablePointer<Int32>?) -> Int32).self).pointee
    
    private static let ziplock_template_get_by_name: @convention(c) (UnsafePointer<CChar>?, UnsafeMutablePointer<CCredentialTemplate>?) -> Int32 = dlsym(dlopen(nil, RTLD_LAZY), "ziplock_template_get_by_name").assumingMemoryBound(to: (@convention(c) (UnsafePointer<CChar>?, UnsafeMutablePointer<CCredentialTemplate>?) -> Int32).self).pointee
    
    private static let ziplock_templates_free: @convention(c) (UnsafeMutablePointer<CCredentialTemplate>?, Int32) -> Void = dlsym(dlopen(nil, RTLD_LAZY), "ziplock_templates_free").assumingMemoryBound(to: (@convention(c) (UnsafeMutablePointer<CCredentialTemplate>?, Int32) -> Void).self).pointee
    
    private static let ziplock_template_free: @convention(c) (UnsafeMutablePointer<CCredentialTemplate>?) -> Void = dlsym(dlopen(nil, RTLD_LAZY), "ziplock_template_free").assumingMemoryBound(to: (@convention(c) (UnsafeMutablePointer<CCredentialTemplate>?) -> Void).self).pointee

    /// Get all available credential templates
    static func getAllTemplates() -> [CredentialTemplate] {
        var templatesPtr: UnsafeMutablePointer<CCredentialTemplate>?
        var count: Int32 = 0
        
        let result = ziplock_templates_get_all(&templatesPtr, &count)
        guard result == 0, let templates = templatesPtr, count > 0 else {
            return []
        }
        
        defer { ziplock_templates_free(templates, count) }
        
        var templateList: [CredentialTemplate] = []
        for i in 0..<Int(count) {
            let cTemplate = templates.advanced(by: i).pointee
            if let template = convertCTemplateToSwift(cTemplate) {
                templateList.append(template)
            }
        }
        
        return templateList
    }
    
    /// Get a specific credential template by name
    static func getTemplate(name: String) -> CredentialTemplate? {
        var cTemplate = CCredentialTemplate()
        let result = name.withCString { namePtr in
            ziplock_template_get_by_name(namePtr, &cTemplate)
        }
        
        guard result == 0 else { return nil }
        defer { ziplock_template_free(&cTemplate) }
        
        return convertCTemplateToSwift(cTemplate)
    }
    
    /// Available template names for quick reference
    static let availableTemplateNames = [
        "login", "credit_card", "secure_note", "identity",
        "password", "document", "ssh_key", "bank_account",
        "api_credentials", "crypto_wallet", "database", "software_license"
    ]
    
    // Helper function to convert C template to Swift
    private static func convertCTemplateToSwift(_ cTemplate: CCredentialTemplate) -> CredentialTemplate? {
        guard let namePtr = cTemplate.name,
              let descPtr = cTemplate.description else { return nil }
        
        let name = String(cString: namePtr)
        let description = String(cString: descPtr)
        
        // Convert fields
        var fields: [FieldTemplate] = []
        if let fieldsPtr = cTemplate.fields, cTemplate.field_count > 0 {
            for i in 0..<Int(cTemplate.field_count) {
                let cField = fieldsPtr.advanced(by: i).pointee
                if let field = convertCFieldTemplateToSwift(cField) {
                    fields.append(field)
                }
            }
        }
        
        // Convert tags
        var tags: [String] = []
        if let tagsPtr = cTemplate.tags, cTemplate.tag_count > 0 {
            for i in 0..<Int(cTemplate.tag_count) {
                if let tagPtr = tagsPtr.advanced(by: i).pointee {
                    tags.append(String(cString: tagPtr))
                }
            }
        }
        
        return CredentialTemplate(name: name, description: description, fields: fields, defaultTags: tags)
    }
    
    // Helper function to convert C field template to Swift
    private static func convertCFieldTemplateToSwift(_ cField: CFieldTemplate) -> FieldTemplate? {
        guard let namePtr = cField.name,
              let typePtr = cField.field_type,
              let labelPtr = cField.label else { return nil }
        
        let name = String(cString: namePtr)
        let fieldType = String(cString: typePtr)
        let label = String(cString: labelPtr)
        let required = cField.required != 0
        let sensitive = cField.sensitive != 0
        let defaultValue = cField.default_value != nil ? String(cString: cField.default_value) : nil
        
        var validation: FieldValidation?
        if cField.validation_min_length >= 0 || cField.validation_max_length >= 0 ||
           cField.validation_pattern != nil || cField.validation_message != nil {
            validation = FieldValidation(
                minLength: cField.validation_min_length >= 0 ? Int(cField.validation_min_length) : nil,
                maxLength: cField.validation_max_length >= 0 ? Int(cField.validation_max_length) : nil,
                pattern: cField.validation_pattern != nil ? String(cString: cField.validation_pattern) : nil,
                message: cField.validation_message != nil ? String(cString: cField.validation_message) : nil
            )
        }
        
        return FieldTemplate(
            name: name,
            fieldType: fieldType,
            label: label,
            required: required,
            sensitive: sensitive,
            defaultValue: defaultValue,
            validation: validation
        )
    }
}

// MARK: - Template Usage Examples

extension ZipLockCredential {
    /// Create a credential from a template with validation
    static func createFromTemplate(_ templateName: String, title: String) throws -> ZipLockCredential {
        // Get template definition for validation
        guard let template = ZipLockCore.getTemplate(name: templateName) else {
            throw ZipLockError.internalError("Unknown template: \(templateName)")
        }
        
        // Create credential
        let credential = try ZipLockCredential(fromTemplate: templateName, title: title)
        
        // Add default tags
        for tag in template.defaultTags {
            try credential.addTag(tag)
        }
        
        return credential
    }
    
    /// Get template definition for a credential type
    static func getTemplateInfo(for templateName: String) -> CredentialTemplate? {
        return ZipLockCore.getTemplate(name: templateName)
    }
    
    /// Get all available templates
    static func getAllAvailableTemplates() -> [CredentialTemplate] {
        return ZipLockCore.getAllTemplates()
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

### 4. Complete Android Example

Here's a comprehensive Android implementation demonstrating all major ZipLock integration patterns:

```kotlin
//
//  android-example.kt
//  ZipLock Mobile FFI Example
//
//  Example demonstrating how to use ZipLock's C API from Android Kotlin applications.
//  This file shows the complete integration pattern including error handling,
//  memory management, and proper Kotlin idioms.
//

package com.example.ziplock

import android.app.Application
import android.content.Context
import android.util.Log
import kotlinx.coroutines.*
import java.util.concurrent.ConcurrentHashMap

// MARK: - Error Types

sealed class ZipLockError : Exception() {
    object InitializationFailed : ZipLockError()
    object InvalidPointer : ZipLockError()
    object InvalidString : ZipLockError()
    data class FieldError(val message: String) : ZipLockError()
    data class ValidationFailed(val message: String) : ZipLockError()
    data class InternalError(val code: Int, val message: String = "") : ZipLockError()

    override val message: String
        get() = when (this) {
            is InitializationFailed -> "Failed to initialize ZipLock library"
            is InvalidPointer -> "Invalid pointer passed to ZipLock function"
            is InvalidString -> "Invalid string encoding"
            is FieldError -> "Field error: $message"
            is ValidationFailed -> "Validation failed: $message"
            is InternalError -> "Internal ZipLock error (code $code): $message"
        }

    companion object {
        fun fromCode(code: Int): ZipLockError = when (code) {
            -1 -> InvalidPointer
            -2 -> InvalidString
            -3 -> FieldError("Invalid field")
            -4 -> ValidationFailed("Validation failed")
            else -> InternalError(code)
        }
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
    CUSTOM(13);

    val displayName: String
        get() = when (this) {
            TEXT -> "Text"
            PASSWORD -> "Password"
            EMAIL -> "Email"
            URL -> "URL"
            USERNAME -> "Username"
            PHONE -> "Phone"
            CREDIT_CARD_NUMBER -> "Credit Card"
            EXPIRY_DATE -> "Expiry Date"
            CVV -> "CVV"
            TOTP_SECRET -> "TOTP Secret"
            TEXT_AREA -> "Text Area"
            NUMBER -> "Number"
            DATE -> "Date"
            CUSTOM -> "Custom"
        }

    val isSensitiveByDefault: Boolean
        get() = when (this) {
            PASSWORD, CVV, TOTP_SECRET -> true
            else -> false
        }

    companion object {
        fun fromValue(value: Int): ZipLockFieldType? = values().find { it.value == value }
    }
}

// MARK: - Password Strength

data class PasswordStrength(
    val level: Level,
    val score: UInt,
    val description: String
) {
    enum class Level(val value: Int) {
        VERY_WEAK(0),
        WEAK(1),
        FAIR(2),
        GOOD(3),
        STRONG(4);

        val description: String
            get() = when (this) {
                VERY_WEAK -> "Very Weak"
                WEAK -> "Weak"
                FAIR -> "Fair"
                GOOD -> "Good"
                STRONG -> "Strong"
            }

        val color: String
            get() = when (this) {
                VERY_WEAK -> "#FF4444"
                WEAK -> "#FF8800"
                FAIR -> "#FFBB00"
                GOOD -> "#88BB00"
                STRONG -> "#44BB44"
            }

        companion object {
            fun fromValue(value: Int): Level? = values().find { it.value == value }
        }
    }
}

// MARK: - Native Interface

object ZipLockNative {
    private const val TAG = "ZipLockNative"

    init {
        try {
            System.loadLibrary("ziplock_shared")
            Log.d(TAG, "ZipLock native library loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load ZipLock native library", e)
            throw ZipLockError.InitializationFailed
        }
    }

    // Library management
    external fun ziplock_init(): Int
    external fun ziplock_get_version(): String?
    external fun ziplock_get_last_error(): String?
    external fun ziplock_string_free(ptr: Long)

    // Credential management
    external fun ziplock_credential_new(title: String, type: String): Long
    external fun ziplock_credential_from_template(template: String, title: String): Long
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
    external fun ziplock_credential_remove_field(handle: Long, name: String): Int
    external fun ziplock_credential_add_tag(handle: Long, tag: String): Int
    external fun ziplock_credential_remove_tag(handle: Long, tag: String): Int
    external fun ziplock_credential_has_tag(handle: Long, tag: String): Int
    external fun ziplock_credential_validate(handle: Long): Long

    // Password utilities
    external fun ziplock_password_generate(
        length: Int,
        includeUppercase: Int,
        includeLowercase: Int,
        includeNumbers: Int,
        includeSymbols: Int
    ): String?
    external fun ziplock_password_validate(password: String): Long
    external fun ziplock_password_strength_free(handle: Long)

    // Validation
    external fun ziplock_email_validate(email: String): Int
    external fun ziplock_url_validate(url: String): Int
    external fun ziplock_validation_result_free(handle: Long)

    // Utilities
    external fun ziplock_credit_card_format(cardNumber: String): String?
    external fun ziplock_totp_generate(secret: String, timeStep: Int): String?
    external fun ziplock_test_echo(input: String): String?

    // Debug
    external fun ziplock_debug_logging(enabled: Int): Int
}

// MARK: - Core Library Manager

class ZipLockCore private constructor() {
    companion object {
        @Volatile
        private var INSTANCE: ZipLockCore? = null

        fun getInstance(): ZipLockCore {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: ZipLockCore().also { INSTANCE = it }
            }
        }
    }

    private val isInitialized: Boolean

    init {
        val result = ZipLockNative.ziplock_init()
        isInitialized = result == 0
        if (!isInitialized) {
            val error = ZipLockNative.ziplock_get_last_error() ?: "Unknown error"
            throw ZipLockError.InternalError(result, error)
        }
        Log.d("ZipLockCore", "ZipLock library initialized successfully")
    }

    val version: String
        get() = ZipLockNative.ziplock_get_version() ?: "Unknown"

    fun enableDebugLogging(enabled: Boolean) {
        ZipLockNative.ziplock_debug_logging(if (enabled) 1 else 0)
    }

    fun getLastError(): String? = ZipLockNative.ziplock_get_last_error()
}

// MARK: - Credential Management

class ZipLockCredential private constructor(private val handle: Long) : AutoCloseable {
    private var isClosed = false

    companion object {
        fun create(title: String, type: String): ZipLockCredential {
            val handle = ZipLockNative.ziplock_credential_new(title, type)
            if (handle == 0L) {
                throw ZipLockError.InternalError(-1, "Failed to create credential")
            }
            return ZipLockCredential(handle)
        }

        fun fromTemplate(template: String, title: String): ZipLockCredential {
            val handle = ZipLockNative.ziplock_credential_from_template(template, title)
            if (handle == 0L) {
                throw ZipLockError.InternalError(-1, "Failed to create credential from template")
            }
            return ZipLockCredential(handle)
        }
    }

    fun addField(
        name: String,
        type: ZipLockFieldType,
        value: String,
        label: String? = null,
        sensitive: Boolean? = null
    ) {
        checkNotClosed()
        val isSensitive = sensitive ?: type.isSensitiveByDefault
        val result = ZipLockNative.ziplock_credential_add_field(
            handle, name, type.value, value, label, if (isSensitive) 1 else 0
        )
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun getField(name: String): String? {
        checkNotClosed()
        return ZipLockNative.ziplock_credential_get_field(handle, name)
    }

    fun removeField(name: String) {
        checkNotClosed()
        val result = ZipLockNative.ziplock_credential_remove_field(handle, name)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun addTag(tag: String) {
        checkNotClosed()
        val result = ZipLockNative.ziplock_credential_add_tag(handle, tag)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun removeTag(tag: String) {
        checkNotClosed()
        val result = ZipLockNative.ziplock_credential_remove_tag(handle, tag)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun hasTag(tag: String): Boolean {
        checkNotClosed()
        return ZipLockNative.ziplock_credential_has_tag(handle, tag) == 1
    }

    fun validate() {
        checkNotClosed()
        val validationHandle = ZipLockNative.ziplock_credential_validate(handle)
        if (validationHandle == 0L) {
            throw ZipLockError.InternalError(-1, "Failed to validate credential")
        }

        // Note: In a real implementation, you would parse the validation result
        // For now, we just free the handle
        ZipLockNative.ziplock_validation_result_free(validationHandle)
    }

    private fun checkNotClosed() {
        if (isClosed) {
            throw IllegalStateException("Credential has been closed")
        }
    }

    override fun close() {
        if (!isClosed) {
            ZipLockNative.ziplock_credential_free(handle)
            isClosed = true
        }
    }
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

    fun validate(password: String): PasswordStrength? {
        val handle = ZipLockNative.ziplock_password_validate(password)
        if (handle == 0L) return null

        // Note: In a real implementation, you would parse the C struct
        // For this example, we'll return a mock result
        ZipLockNative.ziplock_password_strength_free(handle)

        // Mock implementation - in reality, you'd parse the actual result
        val score = when {
            password.length < 8 -> 20u
            password.length < 12 -> 40u
            password.any { it.isDigit() } && password.any { it.isLetter() } -> 80u
            else -> 60u
        }

        val level = when (score.toInt()) {
            in 0..20 -> PasswordStrength.Level.VERY_WEAK
            in 21..40 -> PasswordStrength.Level.WEAK
            in 41..60 -> PasswordStrength.Level.FAIR
            in 61..80 -> PasswordStrength.Level.GOOD
            else -> PasswordStrength.Level.STRONG
        }

        return PasswordStrength(level, score, level.description)
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

// MARK: - Utility Functions

object ZipLockUtils {
    fun formatCreditCard(cardNumber: String): String? {
        return ZipLockNative.ziplock_credit_card_format(cardNumber)
    }

    fun generateTOTP(secret: String, timeStep: Int = 30): String? {
        return ZipLockNative.ziplock_totp_generate(secret, timeStep)
    }

    fun testEcho(input: String): String? {
        return ZipLockNative.ziplock_test_echo(input)
    }
}

// MARK: - Jetpack Compose UI Integration Example

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
    var passwordStrength by remember { mutableStateOf<PasswordStrength?>(null) }
    var generatedPassword by remember { mutableStateOf("") }
    var email by remember { mutableStateOf("") }
    var isEmailValid by remember { mutableStateOf(false) }
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        Text(
            text = "ZipLock Core v${ZipLockCore.getInstance().version}",
            style = MaterialTheme.typography.h6
        )
        
        Spacer(modifier = Modifier.height(16.dp))
        
        // Password testing section
        OutlinedTextField(
            value = password,
            onValueChange = { 
                password = it
                passwordStrength = ZipLockPassword.validate(it)
            },
            label = { Text("Test Password") },
            modifier = Modifier.fillMaxWidth()
        )
        
        passwordStrength?.let { strength ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text("Strength: ${strength.level.description}")
                Text("Score: ${strength.score}/100")
            }
        }
        
        Button(
            onClick = {
                ZipLockPassword.generate()?.let { 
                    generatedPassword = it
                    password = it
                }
            },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("Generate Password")
        }
        
        if (generatedPassword.isNotEmpty()) {
            Text(
                text = "Generated: $generatedPassword",
                style = MaterialTheme.typography.caption
            )
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        // Email validation section
        OutlinedTextField(
            value = email,
            onValueChange = { 
                email = it
                isEmailValid = ZipLockValidation.isValidEmail(it)
            },
            label = { Text("Test Email") },
            modifier = Modifier.fillMaxWidth()
        )
        
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text("Valid Email:")
            Icon(
                imageVector = if (isEmailValid) Icons.Default.CheckCircle else Icons.Default.Cancel,
                contentDescription = if (isEmailValid) "Valid" else "Invalid",
                tint = if (isEmailValid) Color.Green else Color.Red
            )
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        Button(
            onClick = { createSampleCredential() },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("Create Sample Credential")
        }
        
        Button(
            onClick = { runAllTests() },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("Run All Tests")
        }
    }
}

private fun createSampleCredential() {
    try {
        ZipLockCredential.create("Example Login", "login").use { credential ->
            credential.addField("username", ZipLockFieldType.USERNAME, "user@example.com")
            credential.addField("password", ZipLockFieldType.PASSWORD, "secret", sensitive = true)
            credential.addField("website", ZipLockFieldType.URL, "https://example.com")
            credential.addTag("example")
            credential.addTag("test")
            credential.validate()
            Log.i("ZipLock", "Sample credential created and validated successfully")
        }
    } catch (e: ZipLockError) {
        Log.e("ZipLock", "Error creating credential", e)
    }
}

private fun runAllTests() {
    Log.i("ZipLock", "Running ZipLock FFI tests...")
    
    // Test echo
    ZipLockUtils.testEcho("Hello from Android!")?.let {
        Log.i("ZipLock", "Echo test successful: $it")
    }
    
    // Test credit card formatting
    ZipLockUtils.formatCreditCard("1234567890123456")?.let {
        Log.i("ZipLock", "Credit card formatted: $it")
    }
    
    // Test TOTP
    ZipLockUtils.generateTOTP("JBSWY3DPEHPK3PXP")?.let {
        Log.i("ZipLock", "TOTP generated: $it")
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

## Cloud Storage Integration

### Android Cloud Storage Challenges

When users open ZipLock archives from cloud storage services (Google Drive, Dropbox, OneDrive, etc.) on Android, several challenges arise that the ZipLock system addresses automatically:

#### Storage Access Framework (SAF) Issues

Android's Storage Access Framework provides `content://` URIs that don't map to real filesystem paths where traditional file locking can operate. ZipLock handles this by:

- **Automatic Detection**: The system detects cloud storage patterns in file paths
- **Copy-to-Local Strategy**: Cloud files are automatically copied to app-private storage for safe operations
- **Sync-Back Mechanism**: Changes are synced back to the original cloud location when saving

#### Common Cloud Storage Patterns Detected

```rust
// Android cloud storage cache patterns
/Android/data/com.google.android.apps.docs/       // Google Drive
/Android/data/com.dropbox.android/                // Dropbox  
/Android/data/com.microsoft.skydrive/             // OneDrive
/Android/data/com.box.android/                    // Box
/Android/data/com.nextcloud.client/               // Nextcloud

// Storage Access Framework URIs
content://com.android.providers.media.documents/

// Generic cloud storage indicators
/cloud/, /sync/, /googledrive/, /dropbox/
```

#### Enhanced File Locking for Cloud Storage

The `CloudFileHandle` provides cloud-aware file operations:

```kotlin
// When opening a cloud file, ZipLock automatically:
// 1. Detects it's from cloud storage
// 2. Copies to local working directory  
// 3. Creates file locks on local copy
// 4. Monitors for external changes
// 5. Syncs back on save/close

// Example warning log:
// "Cloud storage file detected: /Android/data/com.google.android.apps.docs/files/passwords.7z. 
//  Working with local copy: /data/data/com.ziplock/cache/session_123456/passwords.7z"
```

#### Conflict Detection and Prevention

ZipLock implements content-based conflict detection for cloud files:

- **Hash-Based Monitoring**: Tracks file content changes using size + modification time + content sampling
- **External Change Detection**: Warns if the original cloud file was modified by sync services
- **Safe Sync-Back**: Only syncs changes back if no external modifications detected

### Best Practices for Cloud Storage

#### For Users
- **Single Device Editing**: Avoid editing the same archive on multiple devices simultaneously
- **Sync Completion**: Ensure cloud sync is complete before opening archives
- **Local Backup**: Keep local backups of important archives

#### For Developers
- **Always Use CloudFileHandle**: Replace direct file operations with cloud-aware handles
- **Monitor Cloud Patterns**: Watch logs for cloud storage detection warnings
- **Test Cloud Scenarios**: Include cloud storage simulation in integration tests

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

// MARK: - Android CommonTemplates Integration

### Template Data Classes

```kotlin
data class CredentialTemplate(
    val name: String,
    val description: String,
    val fields: List<FieldTemplate>,
    val defaultTags: List<String>
)

data class FieldTemplate(
    val name: String,
    val fieldType: String,
    val label: String,
    val required: Boolean,
    val sensitive: Boolean,
    val defaultValue: String?,
    val validation: FieldValidation?
)

data class FieldValidation(
    val minLength: Int?,
    val maxLength: Int?,
    val pattern: String?,
    val message: String?
)
```

### Template Functions

The Android FFI already includes the CommonTemplates functions in `ZipLockNative.kt`:

```kotlin
// Get all available templates
val templates = ZipLockNative.getAllTemplates()
for (template in templates) {
    Log.d("Template", "${template.name}: ${template.description}")
    for (field in template.fields) {
        Log.d("Field", "  ${field.label} (${field.fieldType})")
    }
}

// Get specific template
val loginTemplate = ZipLockNative.getTemplateByName("login")
if (loginTemplate != null) {
    Log.d("Template", "Login template has ${loginTemplate.fields.size} fields")
}

// Get available template names
val availableNames = ZipLockNative.getAvailableTemplateNames()
Log.d("Templates", "Available: ${availableNames.joinToString(", ")}")
```

### Template Usage Examples

```kotlin
// Create credential from template with validation
fun createLoginCredential(title: String, username: String, password: String, website: String): ZipLockCredential {
    // Get template for validation
    val template = ZipLockNative.getTemplateByName("login")
        ?: throw IllegalArgumentException("Login template not found")
    
    // Create credential from template
    val credential = ZipLockCredential.fromTemplate("login", title)
    
    // Add fields based on template
    credential.addField("username", ZipLockFieldType.USERNAME, username)
    credential.addField("password", ZipLockFieldType.PASSWORD, password)
    credential.addField("website", ZipLockFieldType.URL, website)
    
    // Add default tags from template
    for (tag in template.defaultTags) {
        credential.addTag(tag)
    }
    
    return credential
}

// Create credit card credential
fun createCreditCardCredential(
    title: String,
    cardNumber: String,
    expiryDate: String,
    cvv: String,
    cardholderName: String
): ZipLockCredential {
    val credential = ZipLockCredential.fromTemplate("credit_card", title)
    credential.addField("cardholder", ZipLockFieldType.TEXT, cardholderName)
    credential.addField("number", ZipLockFieldType.CREDIT_CARD_NUMBER, cardNumber)
    credential.addField("expiry", ZipLockFieldType.EXPIRY_DATE, expiryDate)
    credential.addField("cvv", ZipLockFieldType.CVV, cvv)
    return credential
}

// Validate field against template requirements
fun validateFieldAgainstTemplate(templateName: String, fieldName: String, value: String): Boolean {
    val template = ZipLockNative.getTemplateByName(templateName) ?: return false
    val fieldTemplate = template.fields.find { it.name == fieldName } ?: return false
    
    // Check required field
    if (fieldTemplate.required && value.isBlank()) {
        return false
    }
    
    // Check validation rules
    fieldTemplate.validation?.let { validation ->
        validation.minLength?.let { minLength ->
            if (value.length < minLength) return false
        }
        validation.maxLength?.let { maxLength ->
            if (value.length > maxLength) return false
        }
        validation.pattern?.let { pattern ->
            if (!value.matches(Regex(pattern))) return false
        }
    }
    
    return true
}
```

### Template Management Utility

```kotlin
object TemplateManager {
    private var cachedTemplates: List<CredentialTemplate>? = null
    
    fun getAllTemplates(): List<CredentialTemplate> {
        if (cachedTemplates == null) {
            cachedTemplates = ZipLockNative.getAllTemplates()
        }
        return cachedTemplates ?: emptyList()
    }
    
    fun getTemplate(name: String): CredentialTemplate? {
        return getAllTemplates().find { it.name == name }
    }
    
    fun getTemplateNames(): List<String> {
        return getAllTemplates().map { it.name }
    }
    
    fun getTemplatesForUI(): List<Pair<String, String>> {
        return getAllTemplates().map { it.name to it.description }
    }
    
    fun invalidateCache() {
        cachedTemplates = null
    }
}
```

## Performance Considerations

- Minimize FFI boundary crossings for performance-critical operations
- Cache frequently accessed data on the mobile side
- Use batch operations when possible
- Consider background threading for long-running operations

### Cloud Storage Security

#### Temporary File Handling
- Cloud files are copied to app-private storage during operations
- Working directories use unique session identifiers
- Temporary files are securely cleaned up on operation completion
- No sensitive data remains in system temporary directories

#### Sync Conflict Prevention
- Content hashing prevents data corruption from sync conflicts
- File locks prevent concurrent local access during cloud operations
- User warnings alert to potential cloud storage risks

## Future Enhancements

### Enhanced Cloud Storage Support
- **Real-time Sync Monitoring**: Detect cloud service background sync activity
- **Conflict Resolution UI**: User interface for handling sync conflicts
- **Multi-Provider Optimization**: Provider-specific optimizations for different cloud services
- **Offline Mode**: Better handling of offline cloud file access

The C API provides a foundation for:
- Repository management (opening/saving encrypted archives)
- Advanced search functionality
- Synchronization capabilities
- Biometric authentication integration
- Auto-fill service integration

This mobile integration approach ensures that ZipLock can provide a consistent, secure, and performant experience across all platforms while maintaining a single source of truth for core functionality.