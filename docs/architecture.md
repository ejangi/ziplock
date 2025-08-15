# **ZipLock Application Architecture**

This document describes the comprehensive architecture of the ZipLock password manager, including the high-level design, security model, validation systems, and key implementation details.

## **1\. Overview**

ZipLock uses a **unified architecture** where **frontend clients** communicate directly with a **shared core library** through C FFI (Foreign Function Interface) bindings. This eliminates the complexity of separate backend services while providing consistent functionality across all platforms.

This unified approach provides several key benefits:

* **Security:** The master key is held securely within the shared library's memory space, with cryptographic operations isolated from UI code.
* **Portability:** A single shared library implementation works across all platforms (Linux, Windows, iOS, Android, macOS) through FFI bindings.
* **Maintainability:** One implementation to maintain, test, and debug across all platforms.
* **Performance:** Direct function calls eliminate overhead and serialization costs.
* **Simplicity:** No background services or complex communication protocols.

## **2\. Component Breakdown**

### **2.1 Backend Service**

The shared core library provides all cryptographic and file operations through a C FFI interface.

* **Technology:** Written in **Rust** and compiled as a shared library with C FFI bindings for universal platform compatibility.
* **Responsibilities:**
  * **Secure Storage:** Opens, encrypts, and decrypts the 7z file containing credentials.
  * **Master Key Management:** Securely manages master keys in memory with automatic cleanup.
  * **File Locking:** Manages file locks to prevent corruption during sync operations.
  * **FFI Interface:** Exposes a comprehensive C API for all credential and archive operations.
  * **Repository Validation:** Performs comprehensive validation and auto-repair of repository format and structure.
  * **Session Management:** Maintains secure session state within the library context.
  * **Memory Safety:** Rust's memory safety guarantees protect against common security vulnerabilities.

### **2.2 Frontend Clients**

The frontend clients are platform-native applications that provide the user interface while delegating all security-critical operations to the shared library.

* **Technology:**
  * **Linux:** **Rust** using iced/GTK4 with direct FFI calls to the shared library.
  * **Windows:** **Rust** using Tauri with direct FFI calls to the shared library.
  * **iOS:** **Swift + SwiftUI** calling the shared library through Swift C interop.
  * **Android:** **Kotlin + Jetpack Compose** calling the shared library through JNI.
  * **macOS:** **Swift + SwiftUI** calling the shared library through Swift C interop.
* **Responsibilities:**
  * **User Interface:** Platform-native UI components for optimal user experience.
  * **Authentication:** Prompts for master key and passes it securely to the shared library.
  * **FFI Integration:** Direct function calls to the shared library's C API.
  * **Error Display:** Converts library error codes to user-friendly messages.
  * **Input Validation:** Uses shared validation logic through FFI calls.

### **2.3 Shared Library**

The shared library is the core of ZipLock, containing all business logic and providing a C FFI interface for universal platform compatibility.

* **Technology:** A **Rust crate** compiled as a shared library (.so/.dll/.dylib) with C header files for FFI integration.
* **Contents:**
  * **Data Models:** Credential, field, and archive data structures with C-compatible representations.
  * **Archive Operations:** Complete 7z file creation, opening, saving, and validation logic.
  * **Cryptographic Operations:** All encryption, decryption, and key derivation functions.
  * **Validation Logic:** Comprehensive passphrase and credential validation.
  * **C FFI Interface:** Complete API for archive management, credential operations, and validation.
  * **Memory Management:** Safe memory allocation and cleanup for cross-language compatibility.
  * **Session Management:** Secure session handling within the library context.
  * **Utility Functions:** File operations, search, and other shared functionality.

## **3\. Security Architecture**

The foundation of ZipLock's security is its encryption model, designed to protect user data from unauthorized access while maintaining a clear separation of security responsibilities.

### **3.1 Encryption and Key Management**

* **Encryption Standard:** The user's entire credential database will be stored in a single **7z archive**, encrypted using **AES-256**. This is a strong, widely-trusted encryption algorithm.
* **Master Key:** The user's master key is the only key required to unlock the application and access their data. It is never stored on disk. When the user provides the master key, the backend service uses it to derive the encryption key for the 7z file.
* **Key Derivation:** A robust key derivation function (KDF) will be used to turn the user's master key into a strong, cryptographically secure encryption key. This process will include a high iteration count to make brute-force attacks on the master key computationally expensive.

### **3.2 Backend & Frontend Security Model**

ZipLock follows a strict client-server security model to ensure that sensitive operations are handled in a protected environment.

* **Trusted Core Library:** The shared core library is the single point of trust. It is the only component that ever handles the unencrypted master key and performs cryptographic operations. It holds the master key in a secure, in-memory state only after the user has successfully authenticated and it will be wiped from memory when the application is locked.
* **Untrusted Frontend:** Frontend clients are considered untrusted from a security perspective. Their sole purpose is to present the user interface and pass the master key to the core library during the unlock process. They never store or process the master key or unencrypted credentials.
* **Direct Integration:** Communication between the frontend client and the core library uses direct FFI calls within the same process, eliminating external communication channels and reducing attack surface.

### **3.3 Data Integrity and Storage**

The data storage mechanism is designed for both security and portability.

* **File Format:** The password database is a single encrypted ziplock.7z file. This portable format allows users to store the file on local disk, a USB drive, or a cloud sync folder of their choice.
* **Record Integrity:** Each credential is stored as a record.yml file within the archive. The structured YAML format helps ensure data integrity and makes it easy to read and parse.
* **File Locking:** To prevent data corruption, the core library employs file locking mechanisms to ensure only one process can access the 7z file at a time, especially important for preventing issues with concurrent cloud synchronization.

### **3.4 Threat Model and Mitigations**

The security design is intended to mitigate the following primary threats:

* **Threat:** A user's computer is stolen or compromised, but the master key is unknown.
  * **Mitigation:** The entire database is encrypted with a strong key derived from the master key. Without the master key, the data is unreadable.
* **Threat:** An attacker gains access to a running ZipLock session while the user is away.
  * **Mitigation:** The application will automatically lock itself after a user-configurable period of inactivity, requiring re-authentication with the master key.
* **Threat:** A malicious program attempts to read credentials from the frontend.
  * **Mitigation:** The frontend never handles unencrypted credentials. It only receives encrypted data from the backend to display, minimizing the attack surface. Additionally, sensitive fields like passwords are masked by default.
* **Threat:** An attacker tries to guess the master key through brute force.
  * **Mitigation:** The use of a high-iteration key derivation function makes a brute-force attack on the master key computationally prohibitive.

## **4\. Validation Systems**

### **4.1 Repository Validation and Repair**

ZipLock implements a comprehensive repository validation system to ensure data integrity and compatibility with the repository format specification.

#### **Repository Format Version 1.0**

ZipLock repositories follow a specific structure:

```
/
├── metadata.yml              # Repository metadata and version info
├── credentials/               # Credential storage directory
│   ├── .gitkeep              # Ensures directory preservation in archives
│   └── credential-id/         # Individual credential folders
│       └── record.yml        # Credential data in YAML format
└── types/                    # Custom credential type definitions
    ├── .gitkeep              # Ensures directory preservation in archives
    └── custom-type.yml       # Custom type definitions
```

#### **Validation System Components**

* **RepositoryValidator:** The main validation engine that checks repository structure and content.
* **ValidationReport:** Comprehensive report containing validation results, issues, and statistics.
* **Auto-Repair System:** Automatically fixes common issues like missing directories, legacy format migration, and structural problems.

#### **Validation Checks**

1. **Structure Validation:** Verifies presence of required directories and files
2. **Format Validation:** Parses and validates YAML files and version compatibility
3. **Content Validation:** Verifies credential data integrity and custom type definitions
4. **Legacy Format Detection:** Identifies and migrates old format credential files

#### **Integration with Archive Operations**

* Repository format validation occurs during archive opening
* Auto-repair is triggered when issues are detected
* Repaired archives are automatically saved
* Normal archive operations proceed after validation

### **4.2 Shared Master Passphrase Validation**

The master passphrase validation requirements are centralized in the `ziplock-shared` library to ensure consistency between frontend user interface and backend security enforcement.

#### **Validation Architecture**

* **PassphraseRequirements:** Configurable validation requirements
* **PassphraseValidator:** Core validation engine
* **PassphraseStrength:** Detailed validation results with strength levels
* **ValidationPresets:** Common requirement configurations (production, development, legacy)

#### **Default Requirements**

```rust
PassphraseRequirements {
    min_length: 12,
    require_lowercase: true,
    require_uppercase: true,
    require_numeric: true,
    require_special: true,
    max_length: 0, // No limit
    min_unique_chars: 8,
}
```

#### **Integration Points**

* **Backend:** SecurityConfig.passphrase_requirements field, API validation
* **Frontend:** Real-time feedback, visual indicators, submit validation
* **Shared Logic:** Consistent validation across all components

## **5\. Session Management Architecture**

### **5.1 Session-Based Authentication**

The core library implements a session-based authentication system for secure multi-request operations:

* **Session Creation:** Clients must establish a session before database operations
* **Session Tracking:** Core library tracks session state internally with unique session IDs
* **Session Security:** Sessions are cleared when the application is locked for security
* **Automatic Session Management:** FFI client automatically creates sessions when needed

### **5.2 Session Flow**

1. Client initializes connection to core library via FFI
2. `CreateSession` request sent (no session ID required)
3. Core library responds with unique session ID
4. All subsequent requests include session ID
5. Session cleared on lock or error

## **6\. Open Repository Implementation Architecture**

### **6.1 Repository Opening Workflow**

The Open Repository functionality provides a complete workflow for accessing existing ZipLock repositories:

* **File Selection:** Native file dialog integration for .7z file selection
* **Passphrase Authentication:** Secure passphrase input with visual feedback
* **Backend Integration:** Session-based authentication and repository unlocking
* **State Management:** Comprehensive state machine for user experience

### **6.2 State Management Architecture**

```rust
pub enum OpenState {
    Input,           // User selecting file and entering passphrase
    Opening,         // Backend processing the repository unlock
    Complete,        // Successfully opened
    Cancelled,       // User cancelled operation
    Error(String),   // Error occurred with message
}
```

### **6.3 Security Considerations**

* **Passphrase Handling:** Temporary memory storage, secure text input, no persistence
* **File Access:** Validation of permissions, safe path handling
* **Session Management:** Automatic session creation before database operations

## **7\. Communication Architecture**

Frontend clients communicate with the shared library through direct C FFI function calls. This provides a clean, efficient interface for all supported operations, such as:

* Creating and managing sessions
* Creating and unlocking archives with master keys
* Creating, reading, updating, and deleting credentials
* Searching for credentials by title, tags, or content
* Password generation and validation
* Repository validation and repair operations

### **7.1 FFI Interface**

* **Transport:** Direct function calls through C FFI
* **Data Format:** C-compatible structures with proper memory management
* **Session Management:** Session state maintained within the library
* **Error Handling:** Return codes and error structures for comprehensive error reporting
* **Memory Safety:** Automatic cleanup and explicit free functions for safe memory management

## **8\. Error Handling Architecture**

### **8.1 Error Classification**

* **FFI Errors:** Invalid pointers, parameter validation, memory allocation issues
* **Authentication Errors:** Invalid passphrases, session failures
* **Validation Errors:** Input validation, repository format issues
* **Storage Errors:** File access, corruption, permission issues
* **Cryptographic Errors:** Encryption/decryption failures, key derivation issues

### **8.2 Error Message Conversion**

The system includes intelligent error message conversion from technical library errors to user-friendly messages:

* "Invalid pointer" → "Internal error occurred. Please restart the application..."
* "Authentication failed" → "Incorrect passphrase. Please check..."
* "Archive not found" → "The password archive file could not be found..."
* "Cryptographic error" → "Unable to decrypt data. The file may be corrupted..."

## **9\. Architectural Diagram**

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

## **10\. Development Architecture**

### **10.1 Modular Design**

* **Clear Separation:** Backend, frontend, and shared components have distinct responsibilities
* **Shared Dependencies:** Common logic centralized in shared library
* **Platform Adaptation:** Architecture supports multiple frontend implementations

### **10.2 Testing Strategy**

* **Unit Testing:** Individual component testing with comprehensive coverage
* **Integration Testing:** Cross-component communication and workflow testing
* **Security Testing:** Validation logic, encryption, and threat model verification
* **Platform Testing:** Multi-platform compatibility and behavior consistency

### **10.3 Build and Deployment**

* **Workspace Structure:** Rust workspace for coordinated builds
* **Platform Targets:** Support for multiple architectures and operating systems
* **Continuous Integration:** Automated testing and validation across platforms
* **Package Distribution:** Platform-specific packaging and distribution methods

### **10.4 Documentation Architecture**

* **Technical Documentation Structure:** Additional technical documentation should be added to the `docs/technical/` directory as individual `*.md` files and linked into the `docs/technical.md` document for centralized navigation
* **Protected Documentation Files:** The following files should be left alone and not edited:
  * `docs/01-initial-prompt.txt` - Contains the original project prompt and requirements
  * `docs/TODO.md` - Project task tracking and development roadmap
* **Documentation Standards:** All technical documentation should follow consistent formatting and include cross-references to related components

This comprehensive architecture ensures that ZipLock maintains high security standards, provides excellent user experience, and supports future expansion to additional platforms while maintaining code quality and maintainability.