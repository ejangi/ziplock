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
            // Extract error messages if available
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

// MARK: - Example Usage

class ZipLockExample {
    static func runExamples() {
        print("ZipLock iOS FFI Example")
        print("========================")
        print("Library Version: \(ZipLockCore.shared.version)")
        print("")

        // Test basic functionality
        testBasicFunctionality()

        // Test credential management
        testCredentialManagement()

        // Test password utilities
        testPasswordUtilities()

        // Test validation
        testValidation()

        // Test utility functions
        testUtilities()
    }

    private static func testBasicFunctionality() {
        print("1. Testing Basic Functionality")
        print("------------------------------")

        // Test echo function
        if let echo = ZipLockUtils.testEcho("Hello, ZipLock!") {
            print("✓ Echo test: \(echo)")
        } else {
            print("✗ Echo test failed")
        }

        print("")
    }

    private static func testCredentialManagement() {
        print("2. Testing Credential Management")
        print("--------------------------------")

        do {
            // Create a new login credential
            let credential = try ZipLockCredential(title: "Example Login", type: "login")
            print("✓ Created credential")

            // Add fields
            try credential.addField(name: "username", type: .username, value: "user@example.com")
            try credential.addField(name: "password", type: .password, value: "SuperSecure123!")
            try credential.addField(name: "website", type: .url, value: "https://example.com")
            print("✓ Added fields")

            // Add tags
            try credential.addTag("work")
            try credential.addTag("important")
            print("✓ Added tags")

            // Retrieve field values
            if let username = credential.getField(name: "username") {
                print("✓ Retrieved username: \(username)")
            }

            // Check tags
            if credential.hasTag("work") {
                print("✓ Has 'work' tag")
            }

            // Validate credential
            try credential.validate()
            print("✓ Credential validation passed")

        } catch {
            print("✗ Credential management test failed: \(error)")
        }

        print("")
    }

    private static func testPasswordUtilities() {
        print("3. Testing Password Utilities")
        print("-----------------------------")

        // Generate password
        if let password = ZipLockPassword.generate(length: 12, includeSymbols: false) {
            print("✓ Generated password: \(password)")

            // Validate password strength
            if let strength = ZipLockPassword.validate(password) {
                print("✓ Password strength: \(strength.level.description) (Score: \(strength.score))")
            } else {
                print("✗ Password strength validation failed")
            }
        } else {
            print("✗ Password generation failed")
        }

        // Test with a known weak password
        if let weakStrength = ZipLockPassword.validate("123456") {
            print("✓ Weak password strength: \(weakStrength.level.description) (Score: \(weakStrength.score))")
        }

        print("")
    }

    private static func testValidation() {
        print("4. Testing Validation")
        print("--------------------")

        // Test email validation
        let emails = [
            ("user@example.com", true),
            ("invalid-email", false),
            ("test@domain.co.uk", true),
            ("@invalid.com", false)
        ]

        for (email, expected) in emails {
            let isValid = ZipLockValidation.isValidEmail(email)
            let status = isValid == expected ? "✓" : "✗"
            print("\(status) Email '\(email)': \(isValid ? "valid" : "invalid")")
        }

        // Test URL validation
        let urls = [
            ("https://example.com", true),
            ("http://test.org", true),
            ("not-a-url", false),
            ("ftp://files.com", false)
        ]

        for (url, expected) in urls {
            let isValid = ZipLockValidation.isValidURL(url)
            let status = isValid == expected ? "✓" : "✗"
            print("\(status) URL '\(url)': \(isValid ? "valid" : "invalid")")
        }

        print("")
    }

    private static func testUtilities() {
        print("5. Testing Utility Functions")
        print("----------------------------")

        // Test credit card formatting
        let cardNumbers = [
            "1234567890123456",
            "4111-1111-1111-1111",
            "1234"
        ]

        for cardNumber in cardNumbers {
            if let formatted = ZipLockUtils.formatCreditCard(cardNumber) {
                print("✓ Credit card '\(cardNumber)' formatted as: \(formatted)")
            } else {
                print("✗ Failed to format credit card: \(cardNumber)")
            }
        }

        // Test TOTP generation (with example secret)
        let totpSecret = "JBSWY3DPEHPK3PXP"  // Example base32 secret
        if let totp = ZipLockUtils.generateTOTP(secret: totpSecret) {
            print("✓ Generated TOTP: \(totp)")
        } else {
            print("✗ TOTP generation failed")
        }

        print("")
    }
}

// MARK: - Template Examples

extension ZipLockExample {
    static func createLoginCredential(title: String, username: String, password: String, website: String) throws -> ZipLockCredential {
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
                        ZipLockExample.runExamples()
                    }
                }
            }
            .navigationTitle("ZipLock FFI Demo")
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

// MARK: - App Entry Point (for standalone testing)

#if os(iOS)
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Initialize ZipLock Core
        _ = ZipLockCore.shared

        // Run examples in console
        ZipLockExample.runExamples()

        return true
    }
}
#endif
