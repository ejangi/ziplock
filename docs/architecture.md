# ZipLock Unified Architecture

## Overview

ZipLock implements a **unified architecture** with pure separation of concerns:

- **Shared Library**: Handles ALL data operations, validation, cryptography, and business logic in memory
- **Platform Code**: Handles file I/O operations through clean callback interfaces
- **No Mixed Responsibilities**: Clear boundaries between memory operations and file operations

This architecture ensures maximum code reuse while respecting platform capabilities and constraints.

## Core Principles

### 1. Single Source of Truth
All credential data operations, validation, and business logic reside in the shared library. No platform-specific data handling logic.

### 2. Clean Separation
File operations are completely separated from memory operations through well-defined interfaces.

### 3. Platform Flexibility
- **Mobile platforms** (Android/iOS): Handle all file operations in native code
- **Desktop platforms** (Linux/Windows/macOS): Can use shared library direct file access or delegate to platform code

### 4. No Runtime Detection
No complex runtime detection or fallback mechanisms. Simple, predictable behavior.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Shared Library Core                          │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              Pure Memory Repository                         ││
│  │                                                             ││
│  │  • Credential CRUD operations                              ││
│  │  • Data validation & cryptography                          ││
│  │  • Business logic & rules                                  ││
│  │  • YAML serialization/deserialization                     ││
│  │  • Repository format compliance                            ││
│  │  • NO file I/O operations                                  ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │            File Operation Interface                         ││
│  │                                                             ││
│  │  trait FileOperationProvider {                             ││
│  │      fn read_archive(path) -> Vec<u8>;                     ││
│  │      fn write_archive(path, data);                         ││
│  │      fn extract_archive(data, password) -> FileMap;        ││
│  │      fn create_archive(files, password) -> Vec<u8>;        ││
│  │  }                                                          ││
│  │                                                             ││
│  │  Uses sevenz-rust2 for in-memory 7z operations with        ││
│  │  AES-256 encryption - no temporary files required          ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Callback Interface
                              │
            ┌─────────────────┴─────────────────┐
            │                                   │
            ▼                                   ▼
  ┌─────────────────┐                 ┌─────────────────┐
  │  Mobile Apps    │                 │  Desktop Apps   │
  │  (Android/iOS)  │                 │ (Linux/Mac/Win) │
  │                 │                 │                 │
  │ ┌─────────────┐ │                 │ ┌─────────────┐ │
  │ │File I/O     │ │                 │ │File I/O     │ │
  │ │Provider     │ │                 │ │Provider     │ │
  │ │             │ │                 │ │(Optional)   │ │
  │ │• SAF        │ │                 │ │• Direct FS  │ │
  │ │• Documents  │ │                 │ │• Or delegate│ │
  │ │• Cloud APIs │ │                 │ │• 7z direct  │ │
  │ │• 7z native  │ │                 │ │             │ │
  │ └─────────────┘ │                 │ └─────────────┘ │
  └─────────────────┘                 └─────────────────┘
```

## Core Components

### 1. Unified Memory Repository

**Location**: `shared/src/core/memory_repository.rs`

Pure in-memory repository with no file I/O:

```rust
pub struct UnifiedMemoryRepository {
    initialized: bool,
    credentials: HashMap<String, CredentialRecord>,
    metadata: RepositoryMetadata,
    modified: bool,
}
```

**Responsibilities**:
- Credential CRUD operations
- Data validation and integrity
- YAML serialization/deserialization
- Repository format compliance
- Business logic enforcement

**NOT Responsible For**:
- File I/O operations
- Archive extraction/creation (delegated to sevenz-rust2 via FileOperationProvider)
- Platform-specific file handling
- Temporary file management

### 2. File Operation Provider

**Location**: `shared/src/core/file_provider.rs`

Interface for all file operations:

```rust
pub trait FileOperationProvider: Send + Sync {
    fn read_archive(&self, path: &str) -> FileResult<Vec<u8>>;
    fn write_archive(&self, path: &str, data: &[u8]) -> FileResult<()>;
    fn extract_archive(&self, data: &[u8], password: &str) -> FileResult<HashMap<String, Vec<u8>>>;
    fn create_archive(&self, files: HashMap<String, Vec<u8>>, password: &str) -> FileResult<Vec<u8>>;
}
```

**Implementations**:
- `DesktopFileProvider`: Direct filesystem + sevenz-rust2 in-memory operations
- Platform-specific providers: Implemented in native code (Android/iOS) using platform 7z libraries

**Cryptographic Operations**:
All 7z archive operations use `sevenz-rust2` for:
- In-memory archive extraction (`ArchiveReader` with `Cursor<Vec<u8>>`)
- In-memory archive creation (`ArchiveWriter` with memory buffers)  
- AES-256 password-based encryption/decryption
- No temporary files - pure memory operations only

### 3. Repository Manager

**Location**: `shared/src/core/repository_manager.rs`

Coordinates between memory repository and file operations:

```rust
pub struct UnifiedRepositoryManager<F: FileOperationProvider> {
    memory_repo: UnifiedMemoryRepository,
    file_provider: F,
    current_path: Option<String>,
    master_password: Option<String>,
}
```

**Workflow**:
1. File provider reads archive file into `Vec<u8>`
2. File provider uses `sevenz-rust2::ArchiveReader` to extract to `HashMap<String, Vec<u8>>`
3. Memory repository loads credential data from file map
4. All credential operations happen in pure memory
5. Memory repository serializes data back to file map
6. File provider uses `sevenz-rust2::ArchiveWriter` to create encrypted archive in memory
7. Platform code writes archive buffer to storage

## Platform Integration

### Mobile Platforms (Android/iOS)

**FFI Interface**: `shared/src/ffi/mobile.rs`

- **Memory-only operations** exposed via FFI
- **JSON-based file map exchange** between native code and shared library
- **No file operations** in FFI layer

**Platform Responsibilities**:
- Archive file reading/writing (SAF, Documents API)
- 7z extraction/creation using native libraries
- File system permissions and security
- UI file selection and management

**Example Workflow** (Android):
```kotlin
// 1. Read archive file
val archiveData = readFromUri(archiveUri)

// 2. Extract using Android 7z library (Apache Commons Compress or native libs)
val extractedFiles = extract7zArchive(archiveData, password)

// 3. Convert to JSON and pass to shared library
val filesJson = gson.toJson(extractedFiles)
ZipLockBridge.loadFromFiles(repositoryHandle, filesJson)

// 4. Perform credential operations via FFI
ZipLockBridge.addCredential(repositoryHandle, credentialJson)

// Note: Shared library uses sevenz-rust2 internally for desktop platforms
// but mobile platforms handle 7z operations natively as shown above
```

### Desktop Platforms (Linux/Windows/macOS)

**FFI Interface**: `shared/src/ffi/desktop.rs`

- **Full repository operations** including file I/O
- **Direct file system access** using shared library
- **Optional callback-based** file operations

**Integration Options**:

**Option 1: Direct (Recommended)**
```rust
let manager = UnifiedRepositoryManager::new(DesktopFileProvider::new());
manager.open_repository("/path/to/archive.7z", "password")?;
manager.add_credential(credential)?;
manager.save_repository()?;
```

**Option 2: Callback-based**
```rust
let custom_provider = MyCustomFileProvider::new();
let manager = UnifiedRepositoryManager::new(custom_provider);
// Same operations...
```

**Internal sevenz-rust2 Usage**:
```rust
impl FileOperationProvider for DesktopFileProvider {
    fn extract_archive(&self, data: &[u8], password: &str) -> FileResult<HashMap<String, Vec<u8>>> {
        let cursor = Cursor::new(data);
        let reader = sevenz_rust2::ArchiveReader::new(cursor)?;
        // Extract files to memory HashMap - no temporary files
    }
    
    fn create_archive(&self, files: HashMap<String, Vec<u8>>, password: &str) -> FileResult<Vec<u8>> {
        let mut output = Vec::new();
        let mut writer = sevenz_rust2::ArchiveWriter::new(&mut output)?;
        // Create encrypted archive in memory buffer
    }
}
```

## Security Architecture

### Data Security
- **Memory Operations**: All sensitive data operations happen in shared library using sevenz-rust2
- **Consistent Crypto**: AES-256 encryption via sevenz-rust2 across desktop platforms
- **Platform Crypto**: Mobile platforms use native 7z libraries with equivalent security
- **Data Validation**: All data validated at shared library boundaries
- **Memory Safety**: Secure memory handling for credentials, no temporary files

### File Security
- **Platform-Specific**: Each platform implements appropriate file security
- **Archive Integrity**: Shared library validates all loaded data
- **Password Protection**: Consistent password handling across platforms
- **Error Boundaries**: No sensitive data leakage through errors

## Repository Format

### Version 1.0 Structure
```
archive.7z (password protected)
├── metadata.yml              # Repository metadata
├── credentials/
│   ├── {uuid1}/
│   │   └── record.yml        # Individual credential
│   ├── {uuid2}/
│   │   └── record.yml
│   └── index.yml             # Optional: credential index
└── attachments/              # Future: file attachments
```

### Metadata Format
```yaml
version: "1.0"
format: "memory-v1"
created_at: 1700000000
last_modified: 1700000001
credential_count: 42
structure_version: "1.0"
generator: "ziplock-unified"
```

## Error Handling

### Core Errors
```rust
pub enum CoreError {
    NotInitialized,
    CredentialNotFound { id: String },
    ValidationError { message: String },
    SerializationError { message: String },
    FileOperation(FileError),
}
```

### File Errors
```rust
pub enum FileError {
    NotFound { path: String },
    PermissionDenied { path: String },
    ExtractionFailed { message: String },
    CreationFailed { message: String },
    InvalidPassword,
    CorruptedArchive { message: String },
}
```

### Error Boundaries
- **Memory operations**: Return `CoreError`
- **File operations**: Return `FileError`
- **Platform integration**: Convert to platform-specific errors
- **User interface**: Convert to user-friendly messages

## Performance Characteristics

### Memory Operations
- **O(1)** credential access by ID
- **O(n)** credential listing and search
- **Efficient serialization** using YAML
- **No file I/O overhead** during operations

### File Operations
- **sevenz-rust2 in-memory processing** (no temporary files)
- **Direct buffer operations** using `Cursor<Vec<u8>>` and memory buffers
- **AES-256 encryption** handled entirely in memory by sevenz-rust2
- **Platform-optimized** file access patterns
- **Minimal memory footprint** for archive operations

## Testing Strategy

### Unit Tests
- **Memory repository**: 100% pure unit tests
- **File providers**: Tests with mock implementations
- **Repository manager**: Integration tests with mock file providers

### Integration Tests
- **Platform-specific**: Real file operations on each platform
- **Cross-platform**: Same repository across different platforms
- **Performance**: Benchmarking against baseline implementations

### Security Tests
- **Data integrity**: Validate serialization/deserialization
- **Error handling**: Ensure no data leakage
- **Memory safety**: Validate secure memory handling

## Migration from Legacy Architecture

### Deprecated Components (Removed)
- `ffi_hybrid.rs` - Mixed responsibility FFI
- Runtime detection logic
- Temporary file handling
- Adaptive strategy selection
- Platform-specific fallbacks

### Migration Benefits
- **Simplified codebase**: ~40% reduction in complexity
- **Better testability**: Clear separation enables better testing
- **Improved performance**: Elimination of temporary files
- **Platform flexibility**: Each platform optimized for its strengths
- **Maintainability**: Clear boundaries and responsibilities

## Development Guidelines

### Adding New Platforms
1. Implement `FileOperationProvider` for the platform
2. Create platform-specific FFI interface
3. Handle platform file system peculiarities
4. Maintain data operation compatibility

### Extending Functionality
- **Data operations**: Add to `UnifiedMemoryRepository`
- **File operations**: Extend `FileOperationProvider` trait
- **Validation**: Add to shared validation module
- **UI components**: Follow credential form component patterns
- **Templates**: Extend `CommonTemplates` with new credential types

## UI Integration and TOTP Support

### Credential Form System

The application implements a comprehensive credential form system with specialized components:

**TOTP Field Component**:
- Real-time TOTP code generation from Base32 secrets
- Visual countdown timer showing code expiration
- One-click copy functionality with automatic clipboard clearing
- Secure secret input with validation
- Integration with credential templates

**Form Component Architecture**:
```rust
pub struct CredentialForm {
    template: Option<CredentialTemplate>,
    field_values: HashMap<String, String>,
    totp_fields: HashMap<String, TotpField>,
    field_sensitivity: HashMap<String, bool>,
}
```

**Login Credential Template Integration**:
The login template now includes comprehensive 2FA support:
- Username and password fields (required)
- Website URL field (optional)  
- TOTP secret field (optional) with live code generation
- Notes field (optional) for additional information

**Security Features**:
- TOTP codes automatically cleared from clipboard after timeout
- Sensitive fields masked by default with toggle visibility
- Real-time validation for TOTP secret format
- Secure memory handling for sensitive data

### Cross-Platform UI Considerations

**Desktop Platforms**:
- Native file dialogs for repository selection
- Recent repository persistence with automatic path restoration
- Keyboard shortcuts and accessibility support
- Responsive layout adapting to window sizes
- System clipboard integration with security timeouts

**Mobile Platforms**:
- Touch-optimized input fields and buttons
- Platform-native file picker integration
- Secure keyboard handling for sensitive fields
- Biometric authentication integration (planned)

This UI integration ensures consistent user experience across all platforms while maintaining the security and functionality of the unified architecture.