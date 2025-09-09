# ZipLock Technical Documentation

This document serves as the central index for all technical documentation in the ZipLock project. Technical documentation is organized into focused topics, each maintained as individual markdown files in the `docs/technical/` directory.

## ⚠️ IMPORTANT: Technical Documentation Location

**ALL TECHNICAL DOCUMENTATION MUST BE PLACED IN THE `docs/technical/` DIRECTORY.**

ZipLock uses a **unified architecture** with pure separation of concerns where all platform implementations communicate through a shared core library that handles ALL data operations in memory using sevenz-rust2, while platform-specific code handles file I/O operations through clean callback interfaces. This approach provides maximum code reuse while respecting platform capabilities and constraints.

Do not place technical documentation in the root `docs/` directory or other locations. The `docs/technical/` directory is the designated location for all technical content to maintain organization and discoverability.

## Architecture and Design

- [Architecture Overview](architecture.md) - Complete unified system architecture and component relationships
- [Design Guidelines](design.md) - UI/UX design principles and visual standards
- [Configuration Guide](technical/configuration.md) - Complete configuration reference with examples and profiles
- [Advanced Features Guide](technical/advanced-features.md) - Repository validation, cloud storage, and advanced functionality

## Unified Architecture Implementation

- [Unified Architecture Proposal](technical/unified-architecture-proposal.md) - Complete architectural design and rationale
- [Implementation Roadmap](technical/implementation-roadmap.md) - Detailed 5-week implementation plan with concrete steps
- [Starter Implementation Guide](technical/starter-implementation.md) - Production-ready code examples and getting started guide

## Platform Integration

### Mobile Platforms
- [Android Development Guide](technical/android.md) - Comprehensive Android development, setup, and mobile integration
- [File Association Guide](technical/file-association.md) - Cross-platform .7z file association for Android and Linux

### Desktop Platforms
- [Build Guide](technical/build.md) - Comprehensive build troubleshooting, containerized builds, and platform setup

### Cross-Platform Integration
- [FFI Integration Guide](technical/ffi-integration.md) - FFI layer documentation for mobile vs desktop platforms

## Unified Architecture Overview

ZipLock implements a unified architecture with complete separation of concerns that provides consistent functionality across all platforms while optimizing for each environment's strengths:

### Core Principles

1. **Single Source of Truth**: The shared library handles ALL data operations, validation, cryptography, and business logic in memory using sevenz-rust2
2. **Clean Separation**: File I/O operations are handled through clear callback interfaces, not mixed into core logic
3. **Platform Flexibility**: Mobile platforms handle file operations in native code, desktop platforms use sevenz-rust2 for in-memory 7z operations with AES-256 encryption
4. **No Runtime Detection**: Eliminates complex runtime detection and fallback mechanisms
5. **Synchronous Core**: Pure synchronous operations with async wrappers where needed

### Architecture Components

#### Shared Library Core
- **Pure Memory Repository**: Handles all credential operations, validation, and business logic in memory
- **File Operation Interface**: Clean callback interface for platform-specific file handling using sevenz-rust2 for desktop platforms
- **No File I/O**: Core library never directly handles file operations
- **No Platform Detection**: Simple, predictable behavior without runtime complexity

#### Platform Integration
- **Mobile Platforms (Android, iOS)**: Handle ALL file operations in native code using platform 7z libraries, use memory-only FFI
- **Desktop Platforms (Linux, macOS, Windows)**: Use sevenz-rust2 for in-memory 7z operations with AES-256 encryption through full FFI interface

### Benefits of Unified Architecture

- **Architectural Purity**: Achieves maximum functionality in shared library without compromise
- **Platform Flexibility**: Each platform optimized for its strengths and constraints with consistent AES-256 encryption
- **Simplified Codebase**: No runtime detection, temporary files, or fallback mechanisms
- **Better Testing**: Memory operations easily unit tested, file operations mockable using sevenz-rust2
- **Maintainability**: Clear boundaries and single responsibility per component

## Security and Cryptography

- Security architecture and threat model (see [Architecture Overview](architecture.md#security-architecture))
- **Memory-Based Security**: All sensitive operations happen in shared library memory using sevenz-rust2
- **Consistent Cryptography**: AES-256 encryption via sevenz-rust2 across desktop platforms, equivalent security on mobile platforms
- **Platform File Security**: Each platform implements appropriate file security measures
- **Data Validation**: All data validated at shared library boundaries

## Storage and Data Management

- **7z Archive Format**: Consistent password-protected archive format across platforms using sevenz-rust2 for desktop operations
- **Repository Format v1.0**: Standardized YAML-based credential storage structure
- **Memory Operations**: All data operations happen in memory without temporary files using sevenz-rust2
- **File Operation Delegation**: Clean separation between data and file operations

## Current Implementation Architecture

### ✅ Implemented Core Modules
```
shared/src/
├── core/                           # ✅ Pure business logic (no I/O)
│   ├── memory_repository.rs        # ✅ In-memory credential operations
│   ├── file_provider.rs            # ✅ File operation trait + implementations
│   ├── repository_manager.rs       # ✅ Coordinates memory + file operations
│   ├── errors.rs                   # ✅ Unified error handling
│   ├── types.rs                    # ✅ Shared types and constants
│   └── plugins.rs                  # ✅ Plugin system framework
├── ffi/                           # ✅ FFI interfaces
│   ├── mobile.rs                   # ✅ Memory-only FFI (iOS/Android)
│   ├── desktop.rs                  # ✅ Full FFI with file operations
│   ├── common.rs                   # ✅ Shared FFI utilities
│   └── mod.rs                      # ✅ FFI module exports
├── models/                        # ✅ Data structures
│   ├── credential.rs               # ✅ CredentialRecord and related types
│   ├── field.rs                    # ✅ CredentialField and field types
│   ├── template.rs                 # ✅ CredentialTemplate and common templates
│   └── mod.rs                      # ✅ Model exports
├── utils/                         # ✅ Utilities
│   ├── totp.rs                     # ✅ TOTP generation and validation
│   ├── yaml.rs                     # ✅ YAML serialization/deserialization
│   ├── validation.rs               # ✅ General validation utilities
│   ├── search.rs                   # ✅ Search and filtering utilities
│   ├── password.rs                 # ✅ Password generation/analysis
│   ├── encryption.rs               # ✅ Cryptographic utilities
│   ├── backup.rs                   # ✅ Backup/restore functionality
│   └── mod.rs                      # ✅ Utility exports
├── config/                        # ✅ Configuration management
│   ├── app_config.rs              # ✅ Application configuration
│   ├── repository_config.rs        # ✅ Repository settings
│   └── mod.rs                      # ✅ Config exports
├── logging/                       # ✅ Logging infrastructure
│   ├── logger.rs                  # ✅ Core logging implementation
│   ├── mobile_writer.rs           # ✅ Mobile-specific log writers
│   └── mod.rs                      # ✅ Logging exports
└── lib.rs                         # ✅ Clean public API exports
```

### ✅ FFI Interfaces (Implemented)

#### Mobile FFI (`shared/src/ffi/mobile.rs`) - Memory-Only Operations
- **✅ Repository Lifecycle**: Create, initialize, destroy memory repositories
- **✅ File Exchange**: JSON-based file map serialization/deserialization  
- **✅ Credential Operations**: Pure memory CRUD operations
- **✅ No File I/O**: Mobile platforms handle all archive operations natively
- **✅ Error Handling**: Unified error codes and string management

#### Desktop FFI (`shared/src/ffi/desktop.rs`) - Full Operations with sevenz-rust2
- **✅ Repository Manager**: Complete lifecycle with automatic persistence
- **✅ Archive Operations**: Direct 7z operations with AES-256 encryption via sevenz-rust2
- **✅ File Integration**: Automatic save/load with platform file systems
- **✅ Password Management**: Archive password change functionality
- **✅ Repository Status**: Open/modified/path queries

### Platform-Specific Implementation

#### Mobile Platforms - Native File Operations + Memory FFI
- **✅ Android** (Kotlin) - COMPLETED:
  - ✅ Native 7z operations using Apache Commons Compress
  - ✅ SAF (Storage Access Framework) integration for archive access
  - ✅ JSON-based file map exchange with shared library
  - ✅ Memory-only credential operations via mobile FFI
  - ✅ Complete UI integration with unified architecture
  - ✅ Comprehensive integration tests
- **iOS** (Swift) - PLANNED:
  - Native 7z operations using iOS libraries or p7zip
  - Documents API integration for archive access
  - JSON-based file map exchange with shared library
  - Memory-only credential operations via FFI

#### Desktop Platforms - Full Integration with sevenz-rust2
- **Linux** (Rust + iced):
  - Direct Rust API integration using `UnifiedRepositoryManager<DesktopFileProvider>`
  - In-memory 7z operations via sevenz-rust2 with AES-256 encryption
  - Automatic persistence and file operations handled by shared library
  - Repository service wrapper for async UI integration
- **Windows** (Rust + iced):
  - Direct Rust API integration using `UnifiedRepositoryManager<DesktopFileProvider>`
  - In-memory 7z operations via sevenz-rust2 with AES-256 encryption
  - Automatic persistence and file operations handled by shared library
  - Repository service wrapper for async UI integration
- **macOS** (Swift + SwiftUI):
  - Desktop FFI integration for non-Rust applications
  - Direct sevenz-rust2 integration for in-memory 7z operations
  - AES-256 encryption with automatic persistence
  - Full repository manager via desktop FFI

## Development Status and Next Steps

### ✅ Completed Components
- **Core unified architecture**: Pure memory operations with file provider abstraction
- **FFI interfaces**: Both mobile (memory-only) and desktop (full operations) APIs
- **Data models**: Complete credential, field, and template system
- **Utilities**: TOTP, YAML, validation, search, password, encryption, backup
- **Configuration management**: App and repository settings with cross-platform paths
- **Recent repository persistence**: Automatic tracking and loading of recently opened repositories
- **Logging system**: Cross-platform logging with mobile-specific writers
- **Error handling**: Unified error system across all components

#### Credential Templates and Field Types

The system includes a comprehensive credential template system with full TOTP (Time-based One-Time Password) support:

**Core Field Types**:
- `Text`: Plain text fields
- `Password`: Sensitive password fields with masking
- `Email`: Email address validation
- `Url`: Website URL validation  
- `Username`: Username fields
- `Phone`: Phone number fields
- `CreditCardNumber`: Credit card validation
- `ExpiryDate`: MM/YY date format
- `Cvv`: Credit card CVV codes
- `TotpSecret`: TOTP secret keys with live code generation
- `TextArea`: Multi-line text fields

**Login Credential Template**:
The login template now includes comprehensive 2FA support:
- `username` (required): Username field
- `password` (required): Password field  
- `url` (optional): Website URL
- `totp_secret` (optional): TOTP secret for 2FA authentication
- `notes` (optional): Additional notes

**TOTP Implementation**:
- Real-time TOTP code generation and display
- Automatic clipboard integration with security timeout
- Support for standard TOTP secrets (Base32 encoded)
- Visual countdown timer for code expiration
- Seamless integration with credential forms

**Available Templates**:
- Login (with TOTP support)
- Credit Card
- Secure Note
- Identity
- Password
- Document
- SSH Key
- Bank Account
- API Credentials
- Crypto Wallet
- Database
- Software License

#### Recent Repository Management

The application automatically tracks and manages recently accessed repositories for improved user experience:

**Configuration Persistence**:
- Repositories tracked in `~/.config/ziplock/config.yml` on Linux
- Each repository entry includes path, name, last accessed time, and settings
- Automatic cleanup of inaccessible repositories
- Secure storage - only paths stored, no sensitive data

**Auto-Opening Behavior**:
- Most recently accessed repository automatically selected on app startup
- Users only need to enter passphrase - no file selection required
- Graceful fallback to repository selection if no recent repositories found
- Visual indicators distinguish auto-selected vs manually selected repositories

**Implementation Details**:
```rust
pub struct ConfigManager {
    // Tracks recent repositories with metadata
    fn get_most_recent_accessible_repository(&self) -> Option<String>;
    fn set_repository_path(&mut self, path: String) -> Result<()>;
    fn add_recent_repository(&mut self, repo_info: RepositoryInfo);
}

pub struct OpenRepositoryView {
    auto_selected: bool,  // Indicates if repository was auto-selected
    // Enhanced UI messaging based on selection method
}
```

**User Experience Benefits**:
- Streamlined workflow for regular users
- Reduced friction in daily password management
- Maintains security while improving convenience
- Clear visual feedback about repository selection source

### 🚧 In Progress
- **Linux desktop integration**: Desktop FFI integration with UI updates

### 📋 Planned
- **iOS app development**: Mobile FFI integration for iOS platform
- **Windows/macOS desktop apps**: Desktop FFI integration for additional platforms
- **Plugin system activation**: Framework exists but not yet integrated with platforms
- **Advanced backup features**: Core implemented but platform integration needed

### ✅ Recently Completed
- **Android app integration**: Mobile FFI integration with native file operations complete
  - FFI-based archive creation using `ziplock_mobile_create_temp_archive`
  - FFI-based archive extraction using `ziplock_mobile_extract_temp_archive`
  - SAF integration with temporary file workflow
  - Complete elimination of Apache Commons Compress encryption vulnerabilities
  - Comprehensive end-to-end test coverage implemented

### 🗑️ Removed Legacy Components
- **Hybrid FFI system**: Removed mixed-responsibility architecture
- **Backend API layer**: Replaced with direct core integration  
- **IPC-based client**: Replaced with FFI interfaces
- **Platform detection logic**: Replaced with compile-time selection
- **Archive management layer**: Replaced with file provider abstraction
- **Legacy memory repository**: Replaced with unified implementation
- **Legacy test files**: Removed outdated tests for deprecated components (ArchiveLifecycleTest.kt, CredentialFormTest.kt)

## Development and Testing

- [Build Guide](technical/build.md) - Comprehensive build setup and troubleshooting
- [Android Troubleshooting](technical/android.md#troubleshooting) - Android-specific build and debugging issues
- **Testing Strategy**:
  - **Unit Tests**: Pure memory operations with >90% coverage requirement
  - **Integration Tests**: Mock file providers and comprehensive FFI testing
  - **Platform Tests**: Real file operations on each platform
- **Recent Fixes**: Resolved Android Studio build errors (conflicting method overloads, legacy test cleanup)
    - Desktop: sevenz-rust2 integration testing with AES-256 encryption
    - Mobile: Native 7z library integration with JSON exchange testing
  - **Cross-Platform Tests**: Ensure repository compatibility across all platforms
- **Development Environment**: Rust workspace with comprehensive cross-platform support
- **Quality Standards**:
  - Comprehensive testing with mock providers for all components
  - Security validation with sevenz-rust2 for desktop and equivalent security on mobile
  - Performance benchmarking for memory operations and file I/O
  - Documentation standards with concrete code examples

## Architecture Migration Benefits

The unified architecture migration has delivered significant improvements:

### Code Quality
- **~40% reduction in complexity**: Removed mixed-responsibility patterns
- **Better testability**: Clear separation enables isolated unit testing
- **Improved maintainability**: Well-defined component boundaries
- **Consistent behavior**: Same core logic across all platforms

### Performance  
- **No temporary files**: All operations use in-memory processing with sevenz-rust2
- **Direct buffer operations**: Efficient archive handling using `Cursor<Vec<u8>>`
- **Platform optimization**: Each platform uses its optimal file handling approach
- **Reduced memory copying**: Clean interfaces minimize data duplication

### Platform Flexibility
- **Mobile-optimized**: Memory-only FFI respects mobile platform constraints
- **Desktop-optimized**: Full file operations via sevenz-rust2 for desktop efficiency  
- **Consistent security**: AES-256 encryption across all platforms
- **Future-proof**: Easy to add new platforms with existing interfaces

## Version Management and Release Process

### Version Numbering
- **Every change to the codebase MUST increment the version number by at least 0.0.1**
- Follow [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`
  - `MAJOR`: Breaking changes or significant architectural updates
  - `MINOR`: New features, backwards-compatible functionality
  - `PATCH`: Bug fixes, documentation updates, minor improvements
- Update version in `Cargo.toml` files before committing changes

### Changelog Maintenance
- **Every change MUST include a brief, user-friendly entry in `CHANGELOG.md`**
- Entries should be written for end users, not developers
- Use clear, non-technical language describing what the change means to users
- Place new entries in the `[Unreleased]` section following the established format

### Helper Script
- Use `scripts/version/update-version.sh` to automate version bumps and changelog updates
- Example: `./scripts/version/update-version.sh patch "Fixed crash when opening large files"`
- The script automatically updates all Cargo.toml files and adds entries to CHANGELOG.md

## Scripts Organization

The `scripts/` directory is organized into functional subdirectories to maintain clarity and structure:

### `scripts/build/` - Build and Packaging
- **`build-unified-release.sh`** - Creates complete releases with both Linux and Android artifacts
- **`build-linux.sh`** - Builds ZipLock for Linux desktop platforms
- **`build-android-docker.sh`** - Builds Android libraries using containerized NDK environment
- **`build-mobile.sh`** - Builds shared library for mobile platforms (iOS/Android) natively
- **`package-deb.sh`** - Creates Debian packages for distribution
- **`package-arch.sh`** - Creates Arch Linux packages for distribution
- **`test-android-integration.sh`** - Comprehensive Android library testing
- **`verify-android-symbols.sh`** - Android library symbol verification and analysis

### `scripts/dev/` - Development and Testing
- **`run-linux.sh`** - Launches Linux application for development
- **`run-integration-tests.sh`** - Executes full integration test suite
- **`verify-android-setup.sh`** - Verifies Android development environment setup
- **`verify-file-association.sh`** - Verifies .7z file association configuration

### `scripts/version/` - Version Management
- **`update-version.sh`** - Automated version bumps and changelog updates

## Performance and Optimization

- **Memory Efficiency**: No temporary files, direct memory operations using sevenz-rust2
- **Platform Optimization**: Each platform uses optimal file access patterns with sevenz-rust2 for desktop
- **Reduced Complexity**: Simplified code paths improve performance
- **Benchmarking**: Continuous performance monitoring and validation of sevenz-rust2 operations

## Contributing Technical Documentation

When adding new technical documentation:

1. **Create individual files** in the `docs/technical/` directory
2. **Use descriptive filenames** that clearly indicate the content (e.g., `mobile-integration.md`)
3. **Follow naming convention** of lowercase words separated by hyphens
4. **Update this index** by adding appropriate links in the relevant sections
5. **Cross-reference related documents** to maintain documentation cohesion
6. **Include code examples** and diagrams where appropriate
7. **Maintain consistent formatting** following the project's documentation standards

## Migration from Legacy Architecture

### Deprecated Components (Removed)
- `ffi_hybrid.rs` and related adaptive runtime logic - replaced with clean mobile/desktop separation
- `client/hybrid.rs` - hybrid client no longer needed with pure architecture
- `platform/` directory - runtime detection eliminated with static platform selection
- Temporary file handling in archive operations - replaced with sevenz-rust2 in-memory operations
- Mixed-responsibility modules - replaced with single-purpose organized modules
- Scattered utility functions - consolidated into organized `utils/` modules
- Monolithic FFI interface - replaced with platform-specific `ffi/mobile.rs` and `ffi/desktop.rs`

### Migration Benefits
- **~40% reduction in code complexity** through elimination of runtime detection and hybrid strategies
- **Improved testability** with clear separation of concerns:
  - Pure memory operations easily unit tested
  - File operations mockable through clean trait interfaces
  - Platform-specific FFI interfaces independently testable
- **Better performance**:
  - Desktop: sevenz-rust2 in-memory operations eliminate temporary files
  - Mobile: Native 7z libraries optimized for each platform
  - No runtime overhead from platform detection or strategy selection
- **Platform flexibility** allowing each platform to optimize for its strengths:
  - Desktop: Direct sevenz-rust2 integration with AES-256 encryption
  - Mobile: Native file handling with platform-optimized 7z libraries
  - Consistent credential operations across all platforms
- **Maintainability** through clear boundaries and single responsibilities:
  - Each module has one clear purpose
  - No mixed file I/O and business logic
  - Clean FFI interfaces with platform-specific optimizations
- **Developer Experience**:
  - Clear module organization makes codebase navigation intuitive
  - Comprehensive documentation with concrete code examples
  - Organized utility functions reduce code duplication

## Protected Files

The following documentation files should **not be edited** as they serve specific project management purposes:

- `docs/01-initial-prompt.txt` - Original project requirements and prompt
- `docs/TODO.md` - Project task tracking and development roadmap

These files provide historical context and project planning information that should remain unchanged to preserve the development history and planning artifacts.

## Technical Documentation Index

### ✅ Current Architecture Documentation
- **[Unified Architecture](technical/unified-architecture.md)** - Complete architecture overview and implementation status
- **[FFI Integration Guide](technical/ffi-integration.md)** - Mobile and desktop FFI interfaces with examples
- **[Implementation Status](technical/implementation-status.md)** - Current progress and next steps

### ✅ Platform-Specific Documentation  
- **[Linux Desktop Integration](technical/linux-desktop-integration.md)** - Linux desktop app integration status and roadmap
- **[Android Development Guide](technical/android.md)** - Android app integration with unified architecture
- **[Build Guide](technical/build.md)** - Comprehensive build instructions and troubleshooting

### ✅ Feature Documentation
- **[Configuration Guide](technical/configuration.md)** - All configuration options and examples
- **[Advanced Features](technical/advanced-features.md)** - Repository validation, cloud storage, file associations
- **[File Association Guide](technical/file-association.md)** - .7z file handling across platforms

### 🗑️ Removed Documentation
- `implementation-roadmap.md` - Replaced by implementation-status.md
- `starter-implementation.md` - Architecture now fully implemented
- `unified-architecture-proposal.md` - Architecture now implemented and documented

## Documentation Standards

All technical documentation follows these guidelines:

- **Implementation Status**: Clearly marked with ✅ (implemented), 🚧 (in progress), 📋 (planned)
- **Clear headings** with proper markdown hierarchy
- **Code examples** formatted with appropriate syntax highlighting
- **Diagrams and flowcharts** for complex processes
- **Cross-references** to related documentation
- **Version compatibility notes** where applicable
- **Security considerations** for implementation details
- **Platform-specific notes** when features differ across platforms

For questions about technical documentation or suggestions for new topics, please open an issue or start a discussion in the project repository.

## Quick Start for Developers

1. **Read the [Architecture Overview](architecture.md)** to understand the unified architecture
2. **Review the [Implementation Roadmap](technical/implementation-roadmap.md)** for development planning
3. **Use the [Starter Implementation Guide](technical/starter-implementation.md)** for concrete code examples
4. **Check platform-specific guides**:
   - [Android Development Guide](technical/android.md) for mobile development
   - [Build Guide](technical/build.md) for desktop development
5. **Follow [FFI Integration Guide](technical/ffi-integration.md)** for platform integration

The unified architecture provides a clean foundation for building secure, maintainable, and performant password management across all supported platforms.
