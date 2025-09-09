# ZipLock Unified Architecture

This document describes the current unified architecture implementation in ZipLock, which provides pure separation of concerns between memory operations and file operations across all platforms.

## Architecture Overview

ZipLock implements a **unified architecture** with clear separation of responsibilities:

- **Shared Library Core**: Handles ALL data operations, validation, cryptography, and business logic in memory
- **Platform Code**: Handles file I/O operations through clean callback interfaces
- **No Mixed Responsibilities**: Clear boundaries between memory operations and file operations

This architecture ensures maximum code reuse while respecting platform capabilities and constraints.

## Core Principles

### 1. Single Source of Truth
All credential data operations, validation, and business logic reside in the shared library. No platform-specific data handling logic.

### 2. Pure Memory Operations
The core shared library never directly handles file operations - all data processing happens in memory using structured file maps.

### 3. Clean File Operation Abstraction
File operations are completely separated from memory operations through well-defined trait interfaces.

### 4. Platform Flexibility
- **Mobile platforms** (Android/iOS): Handle all file operations in native code using platform APIs
- **Desktop platforms** (Linux/Windows/macOS): Can use shared library direct file access via `sevenz-rust2`

### 5. No Runtime Detection
Simple, predictable behavior without complex runtime detection or fallback mechanisms.

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
│  │            File Operation Provider Trait                    ││
│  │                                                             ││
│  │  trait FileOperationProvider {                             ││
│  │      fn read_archive(path) -> Vec<u8>;                     ││
│  │      fn write_archive(path, data);                         ││
│  │      fn extract_archive(data, password) -> FileMap;        ││
│  │      fn create_archive(files, password) -> Vec<u8>;        ││
│  │  }                                                          ││
│  │                                                             ││
│  │  Uses sevenz-rust2 for in-memory 7z operations on desktop  ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Trait Implementation
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
  │ │             │ │                 │ │             │ │
  │ │• SAF/Docs   │ │                 │ │• sevenz-rust2│ │
  │ │• Native 7z  │ │                 │ │• Direct FS  │ │
  │ │• Platform   │ │                 │ │• In-memory  │ │
  │ │  APIs       │ │                 │ │• AES-256    │ │
  │ └─────────────┘ │                 │ └─────────────┘ │
  └─────────────────┘                 └─────────────────┘
```

## Current Implementation Status

### ✅ Implemented Components

#### Core Architecture
- **✅ UnifiedMemoryRepository** (`shared/src/core/memory_repository.rs`)
  - Pure in-memory credential operations
  - YAML serialization/deserialization
  - Repository format compliance
  - No file I/O dependencies

- **✅ FileOperationProvider Trait** (`shared/src/core/file_provider.rs`)
  - Clean abstraction for file operations
  - `DesktopFileProvider` implementation using `sevenz-rust2`
  - `MockFileProvider` for testing

- **✅ UnifiedRepositoryManager** (`shared/src/core/repository_manager.rs`)
  - Coordinates memory operations with file operations
  - Generic over file provider implementations
  - Handles repository lifecycle

- **✅ Core Error Handling** (`shared/src/core/errors.rs`)
  - Unified error types for core and file operations
  - Clear error boundaries between components

#### FFI Interfaces
- **✅ Mobile FFI** (`shared/src/ffi/mobile.rs`)
  - Memory-only operations exposed via C FFI
  - JSON-based file map exchange
  - No file operations in FFI layer

- **✅ Desktop FFI** (`shared/src/ffi/desktop.rs`)
  - Full repository operations including file I/O
  - Direct file system access using shared library
  - Optional callback-based file operations

#### Data Models
- **✅ Credential Models** (`shared/src/models/`)
  - Complete credential record structures
  - Field types and validation
  - Template system for common credential types

#### Utilities
- **✅ YAML Operations** (`shared/src/utils/yaml.rs`)
- **✅ TOTP Generation** (`shared/src/utils/totp.rs`)
- **✅ Validation System** (`shared/src/utils/validation.rs`)
- **✅ Search Engine** (`shared/src/utils/search.rs`)
- **✅ Password Utilities** (`shared/src/utils/password.rs`)
- **✅ Encryption Utilities** (`shared/src/utils/encryption.rs`)

#### Configuration Management
- **✅ App Configuration** (`shared/src/config/`)
- **✅ Repository Settings** 
- **✅ Cross-platform Logging** (`shared/src/logging/`)

### 🚧 In Progress / Needs Implementation

#### Platform Integration
- **✅ Android App Integration** - COMPLETED
  - ✅ File operations using Storage Access Framework (SAF)
  - ✅ Native 7z library integration via Apache Commons Compress
  - ✅ Mobile FFI integration with JNA
  - ✅ Repository management and UI integration
  - ✅ Archive operations with unified architecture

- **🚧 Linux Desktop App**
  - Desktop FFI integration
  - Direct sevenz-rust2 usage
  - UI integration with unified architecture

- **🚧 iOS App** (Future)
  - File operations using Documents framework
  - Mobile FFI integration

- **🚧 Windows/macOS Desktop Apps** (Future)
  - Platform-specific file operations
  - Desktop FFI integration

#### Advanced Features
- **🚧 Plugin System** (`shared/src/core/plugins.rs`)
  - Framework implemented but not integrated
  - Custom field types and templates
  - Validation rules

- **🚧 Backup/Restore System** (`shared/src/utils/backup.rs`)
  - Core functionality implemented
  - Platform integration needed

## Core Components

### 1. UnifiedMemoryRepository

**Location**: `shared/src/core/memory_repository.rs`

Pure in-memory repository with no file I/O dependencies:

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
- Archive extraction/creation (delegated to FileOperationProvider)
- Platform-specific file handling

### 2. FileOperationProvider Trait

**Location**: `shared/src/core/file_provider.rs`

Interface for all file operations:

```rust
pub trait FileOperationProvider: Send + Sync {
    fn read_archive(&self, path: &str) -> FileResult<Vec<u8>>;
    fn write_archive(&self, path: &str, data: &[u8]) -> FileResult<()>;
    fn extract_archive(&self, data: &[u8], password: &str) -> FileResult<FileMap>;
    fn create_archive(&self, files: FileMap, password: &str) -> FileResult<Vec<u8>>;
}
```

**Implementations**:
- `DesktopFileProvider`: Uses `sevenz-rust2` for in-memory 7z operations with AES-256 encryption
- `MockFileProvider`: For testing
- Platform-specific providers: Implemented in native code (Android/iOS)

### 3. UnifiedRepositoryManager

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
2. File provider extracts to `HashMap<String, Vec<u8>>` using `sevenz-rust2`
3. Memory repository loads credential data from file map
4. All credential operations happen in pure memory
5. Memory repository serializes data back to file map
6. File provider creates encrypted archive in memory using `sevenz-rust2`
7. Platform code writes archive buffer to storage

## Platform Integration

### Mobile Platforms (Android/iOS)

**Current Status**: 🚧 Android in progress, iOS planned

**Architecture**:
- **Memory-only FFI operations** via `shared/src/ffi/mobile.rs`
- **JSON-based file map exchange** between native code and shared library
- **No file operations** in FFI layer

**Platform Responsibilities**:
- Archive file reading/writing (SAF, Documents API)
- 7z extraction/creation using platform native libraries
- File system permissions and security
- UI file selection and management

**Example Workflow** (Android):
```kotlin
// 1. Read archive file using SAF
val archiveData = contentResolver.openInputStream(archiveUri).readBytes()

// 2. Extract using Android 7z library
val extractedFiles = extract7zArchive(archiveData, password)

// 3. Convert to JSON and pass to shared library
val filesJson = gson.toJson(extractedFiles)
ZipLockBridge.loadFromFiles(repositoryHandle, filesJson)

// 4. Perform credential operations via FFI
ZipLockBridge.addCredential(repositoryHandle, credentialJson)
```

### Desktop Platforms (Linux/Windows/macOS)

**Current Status**: 🚧 Linux in progress, Windows/macOS planned

**Architecture**:
- **Full repository operations** via `shared/src/ffi/desktop.rs`
- **Direct file system access** using shared library
- **sevenz-rust2 integration** for 7z operations

**Integration Options**:

**Option 1: Direct (Recommended)**
```rust
let manager = UnifiedRepositoryManager::new(DesktopFileProvider::new());
manager.open_repository("/path/to/archive.7z", "password")?;
manager.add_credential(credential)?;
manager.save_repository()?;
```

**Option 2: Custom File Provider**
```rust
let custom_provider = MyCustomFileProvider::new();
let manager = UnifiedRepositoryManager::new(custom_provider);
```

## Security Architecture

### Data Security
- **Memory Operations**: All sensitive operations use secure memory handling
- **Consistent Crypto**: AES-256 encryption via `sevenz-rust2` on desktop
- **Platform Crypto**: Mobile platforms use equivalent native security
- **Data Validation**: All data validated at shared library boundaries

### File Security
- **Platform-Specific**: Each platform implements appropriate file security
- **Archive Integrity**: Shared library validates all loaded data
- **Password Protection**: Consistent password handling across platforms
- **Error Boundaries**: No sensitive data leakage through errors

## Repository Format

### Version 1.0 Structure
```
archive.7z (AES-256 encrypted)
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
format: "unified-memory-v1"
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

## Performance Characteristics

### Memory Operations
- **O(1)** credential access by ID
- **O(n)** credential listing and search
- **Efficient serialization** using YAML
- **No file I/O overhead** during operations

### File Operations
- **In-memory processing** using `sevenz-rust2` (no temporary files)
- **Direct buffer operations** with `Cursor<Vec<u8>>`
- **AES-256 encryption** handled entirely in memory
- **Platform-optimized** file access patterns

## Development Guidelines

### Adding New Platforms
1. Implement `FileOperationProvider` for the platform
2. Create platform-specific FFI interface if needed
3. Handle platform file system peculiarities
4. Maintain data operation compatibility

### Extending Core Functionality
- **Data operations**: Add to `UnifiedMemoryRepository`
- **File operations**: Extend `FileOperationProvider` trait
- **Validation**: Add to shared validation modules
- **UI features**: Build on core operations via FFI

### Testing Strategy
- **Unit Tests**: Pure memory repository operations
- **Integration Tests**: Repository manager with mock file providers
- **Platform Tests**: Real file operations on each platform

## Migration Benefits

Compared to the previous mixed-responsibility architecture:

- **✅ ~40% reduction in code complexity**
- **✅ Better testability** with clear separation
- **✅ Improved performance** (no temporary files)
- **✅ Platform flexibility** (each optimized for strengths)
- **✅ Maintainability** (clear boundaries and responsibilities)
- **✅ Consistent behavior** across all platforms

## Next Steps

### Immediate Priorities
1. **Complete Android Integration**
   - Finish SAF file operations
   - Integrate mobile FFI
   - UI updates for unified architecture

2. **Complete Linux Desktop Integration**
   - Update UI to use desktop FFI
   - Remove legacy IPC dependencies
   - Performance optimization

### Future Development
1. **iOS App Development**
2. **Windows/macOS Desktop Apps**
3. **Advanced Plugin System**
4. **Enhanced Backup/Restore Features**
5. **Cloud Storage Integration**

## Conclusion

The unified architecture successfully achieves the original design goals of pure separation of concerns while maximizing code reuse across platforms. The implementation provides a solid foundation for cross-platform password management with optimal performance and maintainability.