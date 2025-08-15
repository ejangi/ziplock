# ZipLock Technical Documentation

This document serves as the central index for all technical documentation in the ZipLock project. Technical documentation is organized into focused topics, each maintained as individual markdown files in the `docs/technical/` directory.

## ⚠️ IMPORTANT: Technical Documentation Location

**ALL TECHNICAL DOCUMENTATION MUST BE PLACED IN THE `docs/technical/` DIRECTORY.**

ZipLock uses a unified FFI-based architecture where all platform implementations communicate directly with a shared core library. This approach provides consistent functionality across all platforms while eliminating the complexity of separate backend services.

Do not place technical documentation in the root `docs/` directory or other locations. The `docs/technical/` directory is the designated location for all technical content to maintain organization and discoverability.

## Architecture and Design

- [Architecture Overview](architecture.md) - Complete system architecture and component relationships
- [Design Guidelines](design.md) - UI/UX design principles and visual standards
- [Repository Detection Implementation](technical/repository-detection-implementation.md) - Technical implementation details for repository detection
- [Configuration Guide](technical/configuration.md) - Complete configuration reference with examples and profiles

- [Mobile Integration Guide](technical/mobile-integration.md) - Complete mobile platform integration documentation with examples
- [Mobile Shared Implementation](technical/mobile-shared-implementation.md) - Shared library integration for mobile platforms

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

## Platform-Specific Implementation

### All Platforms (Unified Architecture)
- [Mobile Integration Guide](technical/mobile-integration.md) - Complete FFI integration examples for all platforms
- Linux implementation (Rust + iced/GTK4) with direct FFI calls
- Windows implementation (Rust + Tauri) with direct FFI calls
- iOS implementation (Swift + SwiftUI) with C interop
- Android implementation (Kotlin + Jetpack Compose) with JNI
- macOS implementation (Swift + SwiftUI) with C interop

## Development and Testing

- [Build Guide](build.md) - Comprehensive build troubleshooting, containerized builds, and glibc compatibility
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

### Client Integration Examples
### Mobile Platform Examples
- [Mobile Integration Guide](technical/mobile-integration.md) - iOS and Android integration examples
  - Complete iOS Swift implementation with SwiftUI integration
  - Complete Android Kotlin implementation with Jetpack Compose
  - C FFI wrapper patterns
  - Memory management and error handling



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
