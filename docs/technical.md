# ZipLock Technical Documentation

This document serves as the central index for all technical documentation in the ZipLock project. Technical documentation is organized into focused topics, each maintained as individual markdown files in the `docs/technical/` directory.

## ⚠️ IMPORTANT: Technical Documentation Location

**ALL TECHNICAL DOCUMENTATION MUST BE PLACED IN THE `docs/technical/` DIRECTORY.**

ZipLock uses a unified hybrid architecture with adaptive runtime strategy where all platform implementations communicate through a shared hybrid FFI layer that intelligently adapts to both platform capabilities and runtime contexts. This approach provides consistent functionality across all platforms with platform and runtime-specific optimizations: mobile platforms handle file operations externally, desktop platforms use integrated filesystem operations when safe, and async contexts automatically fallback to external file operations to prevent runtime conflicts.

Do not place technical documentation in the root `docs/` directory or other locations. The `docs/technical/` directory is the designated location for all technical content to maintain organization and discoverability.

## Architecture and Design

- [Architecture Overview](architecture.md) - Complete system architecture and component relationships
- [Design Guidelines](design.md) - UI/UX design principles and visual standards
- [Configuration Guide](technical/configuration.md) - Complete configuration reference with examples and profiles
- [Advanced Features Guide](technical/advanced-features.md) - Repository validation, cloud storage, repository detection, and persistent archive paths
- [File Association Guide](technical/file-association.md) - Cross-platform .7z file association for Android and Linux
- [FFI Integration Guide](technical/ffi-integration.md) - Comprehensive FFI layer documentation and platform integration patterns

## Security and Cryptography

- Security architecture and threat model (see [Architecture Overview](architecture.md#security-architecture))
- Encryption implementation details
- Key management and derivation processes
- Authentication and session management

## Storage and Data Management

- 7z archive format usage and optimization
- Data structure specifications
- Backup and recovery mechanisms
- File locking and concurrent access prevention

## Client Integration

- Unified FFI client interface for all platforms
- Direct function call integration patterns
- Error handling and memory management
- Session management within shared library

## Adaptive Hybrid Architecture Overview

ZipLock implements a unified hybrid architecture with adaptive runtime strategy that provides consistent functionality across all platforms and runtime contexts while optimizing for each environment's strengths:

### Mobile Platforms (Android, iOS)
- **Adaptive Hybrid FFI**: Handles data operations, validation, and cryptography in memory
- **Platform Code**: Manages file operations, Storage Access Framework (Android), and UI
- **Runtime Strategy**: Always uses external file operations mode
- **Benefits**: Leverages platform-specific file APIs while maintaining data consistency

### Desktop Platforms - Sync Context (Linux, macOS, Windows)
- **Adaptive Hybrid FFI**: Handles both data operations AND filesystem operations
- **Platform Code**: Provides native UI and integrates with hybrid client
- **Runtime Strategy**: Creates own Tokio runtime for integrated operations
- **Benefits**: Self-contained operations with optimal performance

### Desktop Platforms - Async Context (Linux with iced, etc.)
- **Adaptive Hybrid FFI**: Handles data operations, automatically detects async context
- **Platform Code**: Handles file operations when FFI detects runtime conflicts
- **Runtime Strategy**: Uses existing runtime or delegates to external file operations
- **Benefits**: Prevents nested runtime panics while maintaining functionality

## Runtime Strategy Detection

The hybrid FFI layer automatically detects the calling context and adapts its execution strategy:

1. **Standalone Context**: No existing async runtime detected → Creates own runtime for integrated operations
2. **Async Context**: Existing async runtime detected → Uses external file operations to prevent conflicts
3. **Mobile Context**: Platform detection → Always uses external file operations regardless of runtime

## Platform-Specific Implementation

### All Platforms (Unified Adaptive Hybrid Architecture)
- [Android Development Guide](technical/android.md) - Comprehensive Android development, setup, mobile integration, debugging, and file association
- [Android Hybrid Migration Guide](technical/android-hybrid-migration.md) - Adaptive hybrid architecture implementation and migration details
- [FFI Integration Guide](technical/ffi-integration.md) - Cross-platform native library integration with runtime adaptation for iOS, Android, and desktop
- Linux implementation (Rust + iced/GTK4) with adaptive hybrid FFI calls and runtime-safe operations
- Windows implementation (Rust + Tauri) with adaptive hybrid FFI calls and runtime-safe operations
- iOS implementation (Swift + SwiftUI) with adaptive hybrid FFI calls and external file operations
- Android implementation (Kotlin + Jetpack Compose) with adaptive hybrid FFI calls and external file operations
- macOS implementation (Swift + SwiftUI) with adaptive hybrid FFI calls and runtime-safe operations

### Runtime Safety Features
- **Automatic Runtime Detection**: FFI layer detects async/sync calling contexts
- **Nested Runtime Prevention**: Prevents Tokio runtime panics in async environments
- **Graceful Fallback**: Seamlessly switches to external file operations when needed
- **Context Adaptation**: Single API adapts behavior based on calling environment
- **Cross-Platform Consistency**: Same adaptive behavior across all platforms and contexts

## Development and Testing

- [Build Guide](technical/build.md) - Comprehensive build troubleshooting, containerized builds, and glibc compatibility
- Build system configuration
- Testing strategies and coverage requirements
- Continuous integration setup
- Code quality standards and linting rules

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
- Example entries:
  - ✅ "Fixed issue where app would crash when opening large password files"
  - ✅ "Added ability to export passwords to CSV format"
  - ❌ "Refactored PasswordManager::decrypt() method to use async/await"
  - ❌ "Updated dependencies in Cargo.toml"

### Release Notes
- Release notes are automatically generated from the CHANGELOG.md file
- The build system extracts the relevant version section and includes it in GitHub releases
- Ensure changelog entries are complete and user-focused before creating releases

### Helper Script
- Use `scripts/version/update-version.sh` to automate version bumps and changelog updates
- Example: `./scripts/version/update-version.sh patch "Fixed crash when opening large files"`
- The script automatically updates all Cargo.toml files and adds entries to CHANGELOG.md
- Supports patch, minor, and major version increments following semantic versioning

## Scripts Organization

The `scripts/` directory is organized into functional subdirectories to maintain clarity and structure:

### `scripts/build/` - Build and Packaging
- **`build-linux.sh`** - Builds ZipLock for Linux platforms with glibc compatibility
- **`build-mobile.sh`** - Builds shared library for mobile platforms (iOS/Android) - see [Mobile Integration Guide](technical/mobile-integration.md)
- **`package-deb.sh`** - Creates Debian packages for distribution
- **`test-build.sh`** - Tests build process in CI environment
- **`test-build-locally.sh`** - Comprehensive local build testing with Docker

### `scripts/dev/` - Development and Testing
- **`run-linux.sh`** - Launches backend service and frontend GUI for development
- **`run-integration-tests.sh`** - Executes full integration test suite
- **`run-tests-with-existing-backend.sh`** - Runs tests against existing backend
- **`test-backend-connection.sh`** - Tests backend service connectivity
- **`verify-android-setup.sh`** - Verifies Android development environment setup
- **`verify-file-association.sh`** - Verifies .7z file association configuration for Android
- **`verify-linux-file-association.sh`** - Verifies .7z file association configuration for Linux
- **`verify-arch-file-association.sh`** - Verifies .7z file association configuration for Arch Linux
- **`android-quick-test-build.sh`** - Quick build and test for Android file associations

### `scripts/version/` - Version Management
- **`update-version.sh`** - Automated version bumps and changelog updates
- **`test-changelog-extraction.sh`** - Tests changelog extraction for releases

### `scripts/migrations/` - Data and Configuration Migration
- **`migrate-config-from-toml-to-yaml.sh`** - Migrates TOML config to YAML format

### `scripts/deploy/` - Deployment Automation
Reserved for future deployment scripts and automation tools.

For detailed usage instructions and examples, see `scripts/README.md`.

## Performance and Optimization

- [Configuration Guide](technical/configuration.md) - Performance tuning through configuration
- Compression algorithm selection and tuning
- Memory management best practices
- UI responsiveness optimization
- Resource usage monitoring

## Contributing Technical Documentation

When adding new technical documentation:

1. **Create individual files** in the `docs/technical/` directory
2. **Use descriptive filenames** that clearly indicate the content (e.g., `encryption-implementation.md`)
3. **Follow naming convention** of lowercase words separated by hyphens
4. **Update this index** by adding appropriate links in the relevant sections
6. **Cross-reference related documents** to maintain documentation cohesion
7. **Include code examples** and diagrams where appropriate
8. **Maintain consistent formatting** following the project's documentation standards

## Example Documentation and Integration Patterns

**Note**: As of the latest update, all example files have been reorganized from the `examples/` directory into relevant technical documentation for better discoverability and maintenance. This includes configuration examples, FFI client implementations, and mobile platform integration code.

The ZipLock project includes comprehensive examples and integration patterns distributed across focused technical guides:

### Configuration Examples
- [Configuration Guide](technical/configuration.md) - Complete configuration reference with examples and deployment profiles
  - Production, development, and legacy compatibility profiles
  - Environment variable overrides
  - Validation configuration examples
  - YAML migration from TOML

### FFI Integration Examples
- [FFI Integration Guide](technical/ffi-integration.md) - Cross-platform native library integration
  - Complete iOS Swift implementation with SwiftUI integration
  - Complete Android Kotlin/JNI implementation with Jetpack Compose
  - C FFI wrapper patterns and common templates
  - Memory management and error handling
  - Performance optimization and debugging

### Advanced Implementation Examples
- [Advanced Features Guide](technical/advanced-features.md) - Advanced system implementations
  - Repository validation with auto-repair capabilities
  - Cloud storage conflict detection and resolution
  - Repository detection algorithms
  - Persistent archive path management



## Protected Files

The following documentation files should **not be edited** as they serve specific project management purposes:

- `docs/01-initial-prompt.txt` - Original project requirements and prompt
- `docs/TODO.md` - Project task tracking and development roadmap

These files provide historical context and project planning information that should remain unchanged to preserve the development history and planning artifacts.

## Documentation Standards

All technical documentation should follow these guidelines:

- **Clear headings** with proper markdown hierarchy
- **Code examples** formatted with appropriate syntax highlighting
- **Diagrams and flowcharts** for complex processes
- **Cross-references** to related documentation
- **Version compatibility notes** where applicable
- **Security considerations** for implementation details
- **Platform-specific notes** when features differ across platforms

For questions about technical documentation or suggestions for new topics, please open an issue or start a discussion in the project repository.
