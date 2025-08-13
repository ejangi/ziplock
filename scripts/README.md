# ZipLock Scripts

This directory contains utility scripts for ZipLock development and maintenance, organized into functional subdirectories.

## Directory Structure

### `build/` - Build and Packaging Scripts
Scripts for compiling, building, and packaging ZipLock for distribution.

- **`build-linux.sh`** - Builds ZipLock for Linux platforms
- **`build-mobile.sh`** - Builds shared library for mobile platforms (iOS/Android)
- **`package-deb.sh`** - Creates Debian packages for distribution
- **`test-build.sh`** - Tests build process in CI environment
- **`test-build-locally.sh`** - Tests complete build process locally with Docker

### `dev/` - Development Scripts
Scripts for development workflow, testing, and debugging.

- **`run-linux.sh`** - Launches both backend service and frontend GUI for development
- **`run-integration-tests.sh`** - Executes the full integration test suite
- **`run-tests-with-existing-backend.sh`** - Runs tests against an existing backend instance
- **`test-backend-connection.sh`** - Tests backend service connectivity

### `version/` - Version Management Scripts
Scripts for managing versions and changelogs.

- **`update-version.sh`** - Automated version management and changelog update script
- **`test-changelog-extraction.sh`** - Tests changelog extraction logic used by GitHub Actions

### `migrations/` - Data Migration Scripts
Scripts for migrating data and configuration between versions.

- **`migrate-config-from-toml-to-yaml.sh`** - Migrates configuration files from TOML to YAML format

### `deploy/` - Deployment Scripts
Reserved for future deployment automation scripts.

## Common Usage Examples

### Build and Package
```bash
# Build for Linux
./scripts/build/build-linux.sh --profile release

# Create Debian package
./scripts/build/package-deb.sh --arch amd64

# Test build locally with Docker
./scripts/build/test-build-locally.sh
```

### Development
```bash
# Start development environment
./scripts/dev/run-linux.sh

# Run integration tests
./scripts/dev/run-integration-tests.sh

# Test backend connectivity
./scripts/dev/test-backend-connection.sh
```

### Version Management
```bash
# Bug fix (increments patch version)
./scripts/version/update-version.sh patch "Fixed crash when opening large password files"

# New feature (increments minor version)
./scripts/version/update-version.sh minor "Added CSV export functionality"

# Breaking change (increments major version)
./scripts/version/update-version.sh major "New encryption system (requires data migration)"

# Test changelog extraction
./scripts/version/test-changelog-extraction.sh
```

### Configuration Migration
```bash
# Migrate from TOML to YAML configuration
./scripts/migrations/migrate-config-from-toml-to-yaml.sh
```

## Build Scripts Details

### `build/build-linux.sh`
Builds ZipLock for Linux platforms with proper glibc compatibility.

**Usage:**
```bash
./scripts/build/build-linux.sh [--target TARGET] [--profile PROFILE]
```

**Options:**
- `--target`: Rust target triple (default: x86_64-unknown-linux-gnu)
- `--profile`: Build profile (debug/release, default: release)

### `build/build-mobile.sh`
Builds shared library for mobile platforms.

**Usage:**
```bash
./scripts/build/build-mobile.sh [ios|android|all]
```

### `build/package-deb.sh`
Creates Debian packages for distribution.

**Usage:**
```bash
./scripts/build/package-deb.sh [--arch ARCH]
```

**Options:**
- `--arch`: Package architecture (amd64, arm64, default: amd64)

### `build/test-build-locally.sh`
Comprehensive local build testing using Docker containers.

**Usage:**
```bash
./scripts/build/test-build-locally.sh [--clean] [--no-cache] [--skip-test]
```

**Options:**
- `--clean`: Remove existing containers and images
- `--no-cache`: Build Docker images without cache
- `--skip-test`: Skip package installation test

## Development Scripts Details

### `dev/run-linux.sh`
Launches both backend service and frontend GUI for development testing.

**Features:**
- Automatically starts backend service
- Waits for backend to be ready
- Launches frontend GUI
- Provides colored output for better debugging
- Handles cleanup on exit

### `dev/run-integration-tests.sh`
Executes comprehensive integration tests across all components.

### `dev/test-backend-connection.sh`
Tests backend service connectivity and basic functionality.

## Version Management Details

### `version/update-version.sh`
Automated version management and changelog update script.

**Usage:**
```bash
./scripts/version/update-version.sh [patch|minor|major] "changelog-entry"
```

**What it does:**
1. Increments version number in all `Cargo.toml` files
2. Adds user-friendly changelog entry to `CHANGELOG.md`
3. Shows summary of changes for review

**Requirements:**
- Must be run from project root directory
- Requires `CHANGELOG.md` and `Cargo.toml` files to exist

**Changelog Guidelines:**
- Write entries for end users, not developers
- Use clear, non-technical language
- Describe what the change means to users

**Good examples:**
- ✅ "Fixed issue where app would crash when opening large password files"
- ✅ "Added ability to export passwords to CSV format"

**Bad examples:**
- ❌ "Refactored PasswordManager::decrypt() method to use async/await"
- ❌ "Updated dependencies in Cargo.toml"

### `version/test-changelog-extraction.sh`
Tests the changelog extraction logic used by GitHub Actions for release notes.

**What it does:**
1. Tests extraction of `[Unreleased]` section
2. Tests extraction of specific version sections
3. Shows available changelog sections
4. Verifies the same logic used in CI/CD pipeline

## Migration Scripts Details

### `migrations/migrate-config-from-toml-to-yaml.sh`
Migrates configuration files from TOML to YAML format for v0.2.0+ compatibility.

**Usage:**
```bash
./scripts/migrations/migrate-config-from-toml-to-yaml.sh
```

**What it does:**
1. Backs up existing TOML configuration files
2. Converts TOML configuration to YAML format
3. Validates converted configuration
4. Provides rollback instructions if needed

## Release Process

1. Make your changes
2. Use `./scripts/version/update-version.sh` to bump version and update changelog
3. Review changes: `git diff`
4. Test thoroughly: `./scripts/build/test-build-locally.sh`
5. Commit: `git add . && git commit -m "Bump version to X.Y.Z"`
6. Tag: `git tag vX.Y.Z`
7. Push: `git push && git push --tags`

The GitHub Actions workflow will automatically:
- Extract changelog content for the version
- Generate release notes combining changelog with installation instructions
- Create GitHub release with artifacts
- Upload Debian packages

## Version Management Guidelines

**Every code change must:**
1. Increment version by at least 0.0.1 using semantic versioning
2. Include a brief, user-friendly changelog entry
3. Use the `scripts/version/update-version.sh` script for consistency

**Semantic Versioning:**
- `MAJOR`: Breaking changes or significant architectural updates
- `MINOR`: New features, backwards-compatible functionality
- `PATCH`: Bug fixes, documentation updates, minor improvements

## Script Maintenance

When adding new scripts:
1. Place them in the appropriate subdirectory based on function
2. Make them executable: `chmod +x script-name.sh`
3. Add proper error handling with `set -e`
4. Include colored output for better UX
5. Document usage with `--help` or usage functions
6. Update this README with the new script
7. Test scripts on clean environment before committing

### Script Categories
- **Build**: Compilation, packaging, distribution
- **Development**: Testing, debugging, development workflow
- **Version**: Version management, changelog maintenance
- **Migration**: Data and configuration migrations
- **Deploy**: Deployment automation (future use)

## Getting Help

For script-specific help, most scripts support the `--help` option:
```bash
./scripts/build/build-linux.sh --help
./scripts/version/update-version.sh --help
```

For general questions about the build system or scripts, refer to:
- [Build Guide](../docs/technical/build.md) - Comprehensive build documentation
- [Technical Documentation](../docs/technical.md) - Architecture and implementation details
- Project issues and discussions on GitHub