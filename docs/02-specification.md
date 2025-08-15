# **ZipLock Application Specification**

## **1\. Project Overview**

* **Application Name:** ZipLock
* **Vision:** To create a portable, secure, and open-source password manager that utilizes an encrypted ZIP file as its primary data store. This allows users to easily manage and sync their credentials across multiple devices using their preferred file synchronization service.
* **License:** The entire project will be released under the Apache 2.0 license.

## **2\. Core Architecture**

The application follows a unified architecture where frontend clients communicate directly with a shared core library through FFI (Foreign Function Interface) bindings. This eliminates the complexity of separate backend services while providing consistent functionality across all platforms.

### **2.1 Shared Core Library**

* **Language:** Rust
* **Functionality:** The core library is responsible for managing the encrypted 7z file containing the user's credentials. It provides a C FFI interface that all platform clients can use.
  * **Universal Implementation:** A single Rust implementation serves all platforms through FFI bindings, ensuring consistent behavior across desktop and mobile platforms.
  * **Storage Management:** It handles all read, write, and encryption operations on the 7z file. The file is locked when in use to prevent data corruption from concurrent access (e.g., by a cloud sync service).
  * **Security:** The core library holds the master key in a secure, in-memory state only after the user has authenticated. It never persists the master key to disk.
  * **Configuration:** The library manages minimal configuration data, such as the path to the user's encrypted 7z file. This data is stored in platform-specific locations:
    * **Linux:** \~/.config/ziplock/config.yml
    * **Windows:** %APPDATA%/ZipLock/config.yml
    * **macOS:** ~/Library/Application Support/ZipLock/config.yml
    * The library also manages user preferences like auto-lock timeout.
* **Encryption:** The 7z file is encrypted using **AES-256** with the user-provided master key.

### **2.2 Frontend Clients**

The frontend clients are native applications that communicate with the shared core library via direct FFI calls. They are responsible for rendering the user interface and translating user actions into library function calls.

* **Linux:** Written in **Rust** using a GUI framework like gtk-rs or iced to ensure native look and feel and compatibility with the **Wayland** display server. Communicates with the shared core library through direct FFI calls.
* **Windows:** Written in **Rust** using a framework like tauri or winrt-rs to provide a native application experience. If this proves too complex, a fallback to a C\# application using WPF or WinForms is an acceptable alternative.
* **Other Platforms:** The architecture is designed to support other platforms in the future. The specification should be extended to include clients for macOS (Swift/SwiftUI), iOS (Swift/SwiftUI), and Android (Kotlin/Jetpack Compose).

### **2.3 Project Structure**

The project will follow a modular, workspace-based folder structure to facilitate development, debugging, and cross-platform compilation.

/ziplock/
├── .github/                       \# GitHub Actions workflows, issue templates, etc.
├── docs/                          \# Project documentation and specifications
│   └── architecture.md
│   └── design.md
│   └── security.md
├── scripts/                       \# Build, test, and deployment scripts
├── shared/                        \# Shared Rust library for data models, encryption logic, etc.
│   ├── src/                       \# Source code for the shared library
│   │   ├── lib.rs
│   │   └── models/
│   │   └── utils/
│   └── Cargo.toml

├── apps/                          \# Root directory for all applications
│   ├── linux/                     \# Linux app (Rust \+ GTK/Iced)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── windows/                   \# Windows frontend (Rust \+ Tauri/winrt-rs)
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json         \# Only if using Tauri
│   ├── macos/                     \# macOS frontend (Swift/SwiftUI \- future)
│   ├── ios/                       \# iOS app (Swift/SwiftUI \- future)
│   └── android/                   \# Android app (Kotlin/Jetpack Compose \- future)
├── .gitignore
├── Cargo.toml                     \# Workspace Cargo.toml file
├── LICENSE
└── README.md

## **3\. Data Structure and Storage**

The data will be stored in a structured manner inside the encrypted ZIP file to ensure organization and portability.

### **3.1 File and Folder Structure**

The root of the encrypted ZIP file will contain two main folders:

* **/credentials/:** This folder will contain a subfolder for each individual credential record.
  * Each credential folder will be named using a sanitized, lowercase, hyphenated version of the credential's title (e.g., "My Google Login" becomes my-google-login).
  * Inside each credential folder, the main data file will be named **record.yml**.
* **/types/:** This folder will store the definitions for custom credential types created by the user.

### **3.2 record.yml Format**

Each record.yml file will use the YAML format and contain the following fields:

* title: The user-facing, human-readable title of the credential.
* credentialType: A string identifying the template used (e.g., "Login", "Secure Note").
* tags: An array of strings for user-defined tags to aid in organization and search.
* fields: A mapping of field names to field data. Each field will be an object containing:
  * type: A string identifying the field type (e.g., "password", "url", "file").
  * value: The actual data for the field. For file types, this will be a path to the file relative to the credential's folder.

### **3.3 Credential Types and Fields**

* **Field Types:** The application will support the following standard field types:
  * text (plain text)
  * password (hidden text)
  * email
  * url
  * file (stores a file in the credential folder)
  * one-time password (for TOTP keys)
  * address (a multi-line text field)
  * date
  * phone
* **Built-in Credential Types:** The application will come pre-configured with the following templates:
  * Login (Username, Password, URL)
  * Secure Note (Title, Body Text)
  * Credit Card (Card Number, Expiration, CVV, Cardholder Name)
  * Identity (Name, Birthday, SSN/ID)
  * Password (Password only)
  * Document (Title, File attachment)
  * SSH Key (Key, Passphrase)
  * Bank Account (Account Number, Routing Number, PIN)
  * API Credentials (Key, Secret, URL)
  * Crypto Wallet (Public Key, Private Key/Seed Phrase)
  * Database (Hostname, Port, Username, Password)
  * Software License (License Key, Product Name, Purchase Date)
* **Custom Credential Types:** The user must have the ability to create their own credential types by combining various field types. These custom types will be stored as YAML files in the /types/ folder within the ZIP file.

## **4\. Key Application Features**

### **4.1 Search and Organization**

* **Full-Text Search:** The app will provide a powerful search function that performs a full-text search across all fields of all credential records.
* **Tag-Based Filtering:** The search will prioritize results that match a credential's assigned tags, allowing for more precise filtering.
* **Tag Management:** Users can add, remove, and manage tags for each credential record.

### **4.2 Security Enhancements**

* **Master Key Security:** The master key will be used to derive the encryption key for the ZIP file and should never be stored on disk. The user only needs the Master Key to unlock the app (which the backend uses to decrypt the zip file) and interact with their stored credentials.
* **Automatic Locking:** The application will automatically lock itself after a period of inactivity, requiring the master key to be re-entered.
* **Password Generator:** An integrated, customizable password generator will allow users to create strong, random passwords.

### **4.3 User Experience Enhancements**

* **Auto-fill Integration:** The application should include a mechanism for browser integration, likely via browser extensions, to allow for seamless auto-filling of login credentials.
* **One-Time Password (TOTP) Generation:** For credentials with a TOTP key, the app should be able to automatically generate and display the current six-digit code.
* **Import/Export:** The app should support importing credentials from common formats (e.g., CSV) and exporting its data for backup or migration purposes.
* **Version History:** A feature to save and restore previous versions of a credential record, allowing the user to revert changes if needed.

### **4.4 User Interface Design**

* **Design Principles:**
  * **Flat Design:** The user interface should employ a flat design philosophy, avoiding gradients, shadows, and complex textures to create a clean, modern aesthetic.
  * **Typography:** Use large, highly readable fonts to ensure information is easy to scan and understand.
  * **Icons:** Utilize a set of flat, modern icons that are consistent across all platforms.
  * **Simplicity & Readability:** The UI should be intuitive and straightforward, with a focus on ease of use. Generous use of white space will be employed to reduce visual clutter and improve readability.

## **5\. Development and Testing**

The development process will emphasize a modular, test-driven approach to ensure the codebase is robust, secure, and maintainable.

* **Modular Codebase:** The application's code will be broken down into logical modules and classes. This includes separating the core data models, encryption logic, and API handlers into distinct units to promote clarity and ease of debugging. The shared library, for example, will house the common data structures and utility functions, which will be a key part of this modular design.
* **Unit Testing:** All individual functions, methods, and classes will be thoroughly tested using automated unit tests. This ensures that each small piece of the codebase works as intended in isolation, preventing regressions and simplifying the debugging process.
* **Integration Testing:** Automated integration tests will be implemented to verify that the different modules and services function correctly when working together. This is especially important for the communication between the frontend and backend, as well as the interaction with the encrypted 7z file.
* **Security Testing:** A significant focus of the testing will be on security. Tests will be designed to probe for potential vulnerabilities, including:
  * **Boundary Condition Checks:** Testing methods with unexpected or malformed inputs to ensure they fail gracefully and don't create exploitable conditions.
  * **Input Sanitization:** Verifying that all user-provided data is properly sanitized before it is processed to prevent injection attacks.
  * **Fuzzing:** Using automated tools to generate and submit a high volume of random data to the application's APIs to uncover potential crashes or security flaws.
* **Continuous Integration:** The project will utilize a Continuous Integration (CI) system to automatically run all tests on every code change, ensuring that no new vulnerabilities or bugs are introduced.
