# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.4] - 2025-08-16

### Added
- Added a settings UI to linux.

## [0.2.3] - 2025-08-15

### Fixed
- Made the necessary changes to get the build working with the new architecture

## [0.2.2] - 2025-08-15

### Changed
- Complete the full list of common credential types and expand the add credential screen to make it easier to work with.

## [0.2.1] - 2025-08-15

### Fixed
- Improved many design elements to be more consitant and user friendly

### Changed
- Improved many design elements to be more consistant and user friendly.

## [0.2.0] - 2025-08-15

### Changed
- Moved away from the frontend/backend architecture to a single app with a shared library to facilitate mobile development.

## [0.1.8] - 2025-08-14

### Fixed
- Removed unnecessary configuration parameters.

### Changed
- Simplified configuration format to only include actively implemented features
- Updated sample configuration files and documentation for clarity

## [0.1.7] - 2025-08-14

### Fixed
- Creating the first build. Version 0.1.7

## [0.1.6] - 2025-08-13

### Fixed
- Added the support for One time password (Multi-factor authentication) secrets to be stored, but the Timed code to be displayed

## [0.1.5] - 2025-08-13

### Fixed
- Added the search functionality

## [0.1.4] - 2025-08-13

### Fixed
- Added repository validation to the backend when the archive file is first opened

## [0.1.3] - 2025-08-13

### Fixed
- Added the delete credential functionality.

## [0.1.2] - 2025-08-13

### Fixed
- Updating the build process to include user-friendly notes. Many other changes to the linux app in this early release to give the user basic functionality.

### Changed
- **BREAKING**: Configuration files now use YAML format instead of TOML
  - Backend configuration: `~/.config/ziplock/backend.yml` (was `backend.toml`)
  - App configuration: `~/.config/ziplock/config.yml` (was `config.toml`)
  - All sample configuration files have been updated to YAML format
  - Migration script provided: `./scripts/migrations/migrate-config-from-toml-to-yaml.sh`

### Added
- Configuration migration script (`scripts/migrations/migrate-config-from-toml-to-yaml.sh`) to help users transition from TOML to YAML
- Sample YAML configuration files in the `config/` directory
- Enhanced configuration documentation with YAML examples

### Fixed
- Configuration parsing errors when using `.toml` extensions
- Improved error messages for configuration file format issues

### Migration Guide
If you have existing TOML configuration files:

1. Run the migration script: `./scripts/migrations/migrate-config-from-toml-to-yaml.sh`
2. Verify your settings in the new YAML files
3. Remove old `.toml` files after confirming everything works

The migration script will:
- Convert existing TOML configs to YAML format
- Preserve your existing settings
- Add any missing default values
- Create backups of existing YAML files if they exist

## [0.1.0] - 2024-01-15

### Added
- Initial release of ZipLock password manager
- Encrypted 7z archive storage for credentials
- Linux app with GTK4/iced interface
- Backend service with IPC communication
- Full-text search across all credential fields
- Built-in credential templates (Login, Secure Note, Credit Card, etc.)
- Custom credential type creation
- Password generator with customizable options
- Auto-lock functionality with configurable timeout
- File locking to prevent concurrent access
- TOTP (Time-based One-Time Password) support
- Import/Export functionality
- Tag-based organization system
- Dark/light theme support
- Repository detection and management
- Comprehensive configuration system
- Security-first architecture with AES-256 encryption
- Argon2 key derivation for master password
- Memory protection for sensitive data
- Automatic backup creation and rotation
- Cross-platform shared library for data models
- Extensive test coverage
- Development scripts for easy setup

### Security
- AES-256-GCM encryption for all stored data
- Argon2id key derivation with configurable parameters
- Master key never persisted to disk
- Secure memory management with zeroization
- File integrity verification
