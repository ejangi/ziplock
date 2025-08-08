# **ZipLock Application Architecture**

This document describes the comprehensive architecture of the ZipLock password manager, including the high-level design, security model, validation systems, and key implementation details.

## **1\. Overview**

The core of the application is a **backend service** that manages the encrypted 7z file. All read and write operations to the credential database are funneled through this service. On top of this, **frontend clients** are built for each platform to provide a native user experience. A **shared library** is used to house common data models and logic, ensuring consistency and reducing code duplication.

This modular approach provides several key benefits:

* **Security:** The master key is held exclusively by the backend service in memory, never exposed to the frontend.
* **Portability:** The core logic is isolated in a shared library and the backend service, making it easier to adapt to new platforms.
* **Maintainability:** Separating concerns into distinct modules simplifies debugging and feature development.

## **2\. Component Breakdown**

### **2.1 Backend Service**

The backend is a long-running, persistent service or daemon responsible for all low-level operations.

* **Technology:** Written in **Rust** for performance and memory safety on desktop platforms (Linux, Windows). For mobile platforms like iOS and Android, this functionality will be reimplemented in a platform-native language (e.g., Swift or Kotlin) to align with platform constraints.
* **Responsibilities:**
  * **Secure Storage:** Opens, encrypts, and decrypts the 7z file containing credentials.
  * **Master Key Management:** Receives the master key from the frontend during initial unlock and holds it securely in a temporary, in-memory state.
  * **File Locking:** Manages file locks to prevent external processes or sync services from corrupting the ZIP file during operations.
  * **API Endpoint:** Exposes a well-defined API for frontend clients to interact with.
  * **Repository Validation:** Performs comprehensive validation and auto-repair of repository format and structure.
  * **Session Management:** Maintains secure session state for authenticated clients.

### **2.2 Frontend Clients**

The frontend clients are the user-facing part of the application. They are designed to be thin and simple, focusing solely on the user experience.

* **Technology:**
  * **Linux:** Written in **Rust** using a native GUI toolkit like gtk-rs or iced, with a strong focus on Wayland compatibility.
  * **Windows:** Written in **Rust** using a framework like tauri or winrt-rs, with a fallback to C\# if needed.
  * **Mobile:** Future clients for iOS and Android will be developed in **SwiftUI** and **Jetpack Compose**, respectively.
* **Responsibilities:**
  * **User Interface:** Displays credentials, provides search functionality, and handles user input.
  * **Authentication:** Prompts the user for the master key and sends it to the backend for authentication.
  * **Communication:** Interacts with the backend service through a platform-appropriate communication channel (e.g., local IPC, gRPC, etc.).
  * **Error Display:** Presents user-friendly error messages and status information.
  * **Input Validation:** Provides real-time validation feedback using shared validation logic.

### **2.3 Shared Library**

The shared library is a critical component that houses the common logic required by both the backend and frontend.

* **Technology:** A **Rust crate** that is a dependency for both the backend and Rust-based frontend projects.
* **Contents:**
  * **Data Models:** Defines the data structures for credentials, fields, and custom types, ensuring a consistent data format across the entire application.
  * **YAML Parsing:** Contains the logic for reading and writing data to the record.yml files.
  * **Validation Logic:** Centralized passphrase validation requirements and logic.
  * **Core Logic:** Includes shared utility functions for file path sanitization and other non-critical logic that doesn't need to be reimplemented.

## **3\. Security Architecture**

The foundation of ZipLock's security is its encryption model, designed to protect user data from unauthorized access while maintaining a clear separation of security responsibilities.

### **3.1 Encryption and Key Management**

* **Encryption Standard:** The user's entire credential database will be stored in a single **7z archive**, encrypted using **AES-256**. This is a strong, widely-trusted encryption algorithm.
* **Master Key:** The user's master key is the only key required to unlock the application and access their data. It is never stored on disk. When the user provides the master key, the backend service uses it to derive the encryption key for the 7z file.
* **Key Derivation:** A robust key derivation function (KDF) will be used to turn the user's master key into a strong, cryptographically secure encryption key. This process will include a high iteration count to make brute-force attacks on the master key computationally expensive.

### **3.2 Backend & Frontend Security Model**

ZipLock follows a strict client-server security model to ensure that sensitive operations are handled in a protected environment.

* **Trusted Backend:** The backend service is the single point of trust. It is the only component that ever handles the unencrypted master key and performs cryptographic operations. It holds the master key in a secure, in-memory state only after the user has successfully authenticated and it will be wiped from memory when the application is locked.
* **Untrusted Frontend:** Frontend clients are considered untrusted from a security perspective. Their sole purpose is to present the user interface and pass the master key to the backend during the unlock process. They never store or process the master key or unencrypted credentials.
* **Secure Communication:** Communication between the frontend client and the backend service will be secured via a platform-appropriate Inter-Process Communication (IPC) mechanism to prevent eavesdropping by other processes on the system.

### **3.3 Data Integrity and Storage**

The data storage mechanism is designed for both security and portability.

* **File Format:** The password database is a single encrypted ziplock.7z file. This portable format allows users to store the file on local disk, a USB drive, or a cloud sync folder of their choice.
* **Record Integrity:** Each credential is stored as a record.yml file within the archive. The structured YAML format helps ensure data integrity and makes it easy to read and parse.
* **File Locking:** To prevent data corruption, the backend service will employ file locking mechanisms to ensure only one process can access the 7z file at a time, especially important for preventing issues with concurrent cloud synchronization.

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

The backend implements a session-based authentication system for secure multi-request operations:

* **Session Creation:** Clients must establish a session before database operations
* **Session Tracking:** Backend tracks session state internally with unique session IDs
* **Session Security:** Sessions are cleared on disconnect for security
* **Automatic Session Management:** IPC client automatically creates sessions when needed

### **5.2 Session Flow**

1. Client connects to backend via Unix socket
2. `CreateSession` request sent (no session ID required)
3. Backend responds with unique session ID
4. All subsequent requests include session ID
5. Session cleared on disconnect or error

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

Frontend clients communicate with the backend service via a platform-specific Inter-Process Communication (IPC) mechanism. This API defines a set of requests and responses for all supported operations, such as:

* Creating and managing sessions
* Unlocking the database with a master key
* Creating, reading, updating, and deleting credentials
* Searching for credentials by title, tags, or content
* Creating and managing custom credential types
* Retrieving user configuration data
* Repository validation and repair operations

### **7.1 IPC Protocol**

* **Transport:** Unix domain sockets on Linux, named pipes on Windows
* **Message Format:** JSON-based request/response protocol
* **Session Management:** Session IDs required for database operations
* **Error Handling:** Comprehensive error codes and user-friendly messages

## **8\. Error Handling Architecture**

### **8.1 Error Classification**

* **IPC Errors:** Connection failures, protocol errors, backend communication issues
* **Authentication Errors:** Invalid passphrases, session failures
* **Validation Errors:** Input validation, repository format issues
* **Storage Errors:** File access, corruption, permission issues

### **8.2 Error Message Conversion**

The system includes intelligent error message conversion from technical backend errors to user-friendly messages:

* "Failed to bind to socket" → "Unable to start the backend service..."
* "Authentication failed" → "Incorrect passphrase. Please check..."
* "Archive not found" → "The password archive file could not be found..."

## **9\. Architectural Diagram**

```
┌----------------┐          ┌-------------------┐          ┌--------------┐
|                | <------> |  Backend Service  | <------> |  Encrypted   |
| Frontend Client|          |  (Rust, Swift,    |          |  Data Store  |
| (Rust, C#, etc.)|          |   Kotlin)         |          |  (7z file)   |
|                |          └-------------------┘          |              |
└----------------┘                ^                        └--------------┘
                                  |
                                  | Uses
                                  |
                           ┌-------------------┐
                           |  Shared Library   |
                           |  (Rust Crate)     |
                           |  - Data Models    |
                           |  - Validation     |
                           |  - Utilities      |
                           └-------------------┘
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