# ZipLock Scripts

This directory contains utility scripts for ZipLock development and maintenance, organized into functional subdirectories. These scripts support the unified FFI architecture (no separate backend daemon).

## Directory Structure

### `build/` - Build and Packaging Scripts
Scripts for compiling, building, and packaging ZipLock for distribution.

- **`build-linux.sh`** - Builds ZipLock for Linux platforms
- **`build-mobile.sh`** - Builds shared library for mobile platforms (iOS/Android)
- **`package-deb.sh`** - Creates Debian packages for distribution
- **`package-arch.sh`** - Creates Arch Linux packages and source archives for AUR
- **`test-arch-packaging.sh`** - Tests Arch Linux packaging in containerized environment
- **`test-pkgbuild-validation.sh`** - Validates PKGBUILD version and checksum accuracy without Docker

### `dev/` - Development Scripts
Scripts for development workflow, testing, and debugging.

- **`run-linux.sh`** - Launches the unified ZipLock application for development
- **`run-integration-tests.sh`** - Executes the full integration test suite (FFI architecture)
- **`run-ci-checks.sh`** - Runs complete GitHub CI test suite locally (format, clippy, tests)
- **`run-clippy.sh`** - Quick Clippy linting check (same as GitHub CI)
- **`run-format.sh`** - Quick code formatting check and fix
- **`pre-push.sh`** - Quick pre-push validation (format + clippy, no tests)
- **`test-in-container.sh`** - Test builds in the same containerized environment as CI

### `version/` - Version Management Scripts
Scripts for managing versions and changelogs.

- **`update-version.sh`** - Automated version management and changelog update script
- **`test-changelog-extraction.sh`** - Tests changelog extraction logic used by GitHub Actions

### `migrations/` - Data Migration Scripts
Scripts for migrating data and configuration between versions.

- **`migrate-config-from-toml-to-yaml.sh`** - Migrates configuration files from TOML to YAML format

### `deploy/` - Deployment Scripts
- **`setup-logging.sh`** - Complete logging setup for deployed systems with systemd service creation
- **`manage-service.sh`** - Comprehensive service management (install, start, stop, monitor, logs)
- **`package-for-deployment.sh`** - Creates complete deployment packages with configs and scripts

## Common Usage Examples

### Build and Package
```bash
# Build for Linux (native)
./scripts/build/build-linux.sh --profile release

# Build in same environment as CI (recommended)
./scripts/dev/test-in-container.sh build

# Create Debian package (native)
./scripts/build/package-deb.sh --arch amd64

# Create Debian package in container (same as CI)
./scripts/dev/test-in-container.sh package-deb

# Create Arch Linux package
./scripts/build/package-arch.sh --source-only

# Test in containerized environment (matches CI exactly)
./scripts/dev/test-in-container.sh test

# Test Arch packaging
./scripts/build/test-arch-packaging.sh

# Validate PKGBUILD quickly
./scripts/build/test-pkgbuild-validation.sh
```

### Development
```bash
# Start unified development application
./scripts/dev/run-linux.sh

# Run integration tests (FFI architecture)
./scripts/dev/run-integration-tests.sh

# Test in exact CI environment (recommended)
./scripts/dev/test-in-container.sh test

# Run CI checks locally before pushing
./scripts/dev/run-ci-checks.sh

# Interactive shell in CI environment for debugging
./scripts/dev/test-in-container.sh shell-ubuntu

# Quick clippy check
./scripts/dev/run-clippy.sh --fix

# Quick format check and fix
./scripts/dev/run-format.sh --fix

# Quick pre-push validation
./scripts/dev/pre-push.sh --fix
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

### `build/package-arch.sh`
Creates Arch Linux packages and source archives for AUR submission.

**Usage:**
```bash
./scripts/build/package-arch.sh [--arch ARCH] [--source-only] [--skip-tests]
```

**Options:**
- `--arch`: Package architecture (x86_64, default: x86_64)
- `--source-only`: Create only source archive for AUR (recommended)
- `--skip-tests`: Skip package installation tests

### `build/test-arch-packaging.sh`
Tests Arch Linux packaging process in containerized environment.

**Usage:**
```bash
./scripts/build/test-arch-packaging.sh [--full-build] [--clean-images]
```

**Options:**
- `--full-build`: Run complete makepkg build test (slow)
- `--clean-images`: Remove Docker images after test

### `build/test-pkgbuild-validation.sh`
Validates PKGBUILD file for version and checksum accuracy without requiring Docker.

**Usage:**
```bash
./scripts/build/test-pkgbuild-validation.sh [--fix-suggestions]
```

**Options:**
- `--fix-suggestions`: Show fix suggestions even if tests pass

**What it validates:**
- PKGBUILD file existence and syntax
- Required variables (pkgname, pkgver, pkgrel, pkgdesc)
- Version consistency with Cargo.toml
- SHA256 checksum format and validity (not 'SKIP')
- Source URL format and version consistency
- Install script existence

**Features:**
- Fast validation without Docker overhead
- Detailed error messages with fix suggestions
- Validates against actual source archive if available
- Ensures PKGBUILD is ready for Arch Linux packaging



## Development Scripts Details

### `dev/run-linux.sh`
Launches the unified ZipLock application for development testing.

**Features:**
- Builds and launches unified application (FFI architecture)
- No separate backend daemon required
- Provides colored output for better debugging
- Supports debug mode for verbose logging
- Memory-efficient single process operation

### `dev/run-integration-tests.sh`
Executes comprehensive integration tests for the unified FFI architecture.

**Features:**
- Tests shared library C API functionality
- Validates FFI client integration
- No backend daemon management required
- Memory-efficient single process testing

### `dev/run-ci-checks.sh`
Runs the complete GitHub CI test suite locally before pushing.

**Usage:**
```bash
./scripts/dev/run-ci-checks.sh [OPTIONS]
```

**Options:**
- `--skip-format`: Skip formatting check
- `--skip-clippy`: Skip Clippy linting
- `--skip-tests`: Skip running tests
- `--fix-format`: Automatically fix formatting issues

**What it does:**
1. Checks system dependencies (GTK4, Rust components)
2. Runs `cargo fmt --check` for code formatting
3. Runs Clippy linting on all packages with same flags as CI
4. Runs tests on all packages with same configuration as CI

### `dev/run-clippy.sh`
Quick Clippy linting check using the same configuration as GitHub CI.

**Usage:**
```bash
./scripts/dev/run-clippy.sh [--fix]
```

**Options:**
- `--fix`: Run clippy with automatic fixes

**Features:**
- Matches exact Clippy configuration from GitHub workflow
- Checks backend, shared library, and frontend packages
- Uses same warning levels and allowed lints as CI
- Fast feedback for linting issues before pushing

### `dev/run-format.sh`
Quick code formatting check using the same configuration as GitHub CI.

**Usage:**
```bash
./scripts/dev/run-format.sh [--fix|--check]
```

**Options:**
- `--fix`: Automatically fix formatting issues
- `--check`: Only check formatting (default)

**Features:**
- Matches exact formatting configuration from GitHub workflow
- Fast feedback for formatting issues before pushing
- Can automatically fix issues with `--fix` option

### `dev/pre-push.sh`
Quick pre-push validation that runs formatting and Clippy checks (skips tests for speed).

**Usage:**
```bash
./scripts/dev/pre-push.sh [--fix|--full]
```

**Options:**
- `--fix`: Automatically fix formatting and clippy issues
- `--full`: Run complete CI test suite including tests

**Features:**
- Fast validation before pushing (format + clippy only)
- Can automatically fix common issues
- Option to run full test suite when needed
- Designed for quick developer feedback loop

### `dev/test-in-container.sh`
Test builds in the exact same containerized environment used by CI/CD pipelines.

**Usage:**
```bash
./scripts/dev/test-in-container.sh [OPTIONS] COMMAND
```

**Commands:**
- `test`: Run full test suite in Ubuntu container
- `build`: Build binaries in Ubuntu container
- `package-deb`: Create Debian package in Ubuntu container
- `package-arch`: Create Arch package in Arch container
- `shell-ubuntu`: Open interactive shell in Ubuntu container
- `shell-arch`: Open interactive shell in Arch container
- `update-images`: Pull latest container images

**Options:**
- `--no-cache`: Don't use Docker build cache
- `--clean`: Clean target directory first
- `--help`: Show help message

**Features:**
- Uses same pre-built container images as GitHub Actions
- Eliminates "works locally, fails in CI" issues
- Interactive debugging shells for troubleshooting
- Automatic container image management
- Perfect environment consistency with CI

**Examples:**
```bash
# Test in exact CI environment
./scripts/dev/test-in-container.sh test

# Build with clean environment
./scripts/dev/test-in-container.sh build --clean

# Debug interactively
./scripts/dev/test-in-container.sh shell-ubuntu

# Create packages in containers
./scripts/dev/test-in-container.sh package-deb
./scripts/dev/test-in-container.sh package-arch
```

## Quick Reference

### Most Common Workflows

**Before pushing code:**
```bash
# Quick check (recommended for most commits)
./scripts/dev/pre-push.sh

# Quick check with auto-fix
./scripts/dev/pre-push.sh --fix

# Full validation in exact CI environment (recommended)
./scripts/dev/test-in-container.sh test

# Full validation native (alternative)
./scripts/dev/run-ci-checks.sh
```

**Individual checks:**
```bash
# Format only
./scripts/dev/run-format.sh --fix

# Clippy only
./scripts/dev/run-clippy.sh

# Full CI suite (containerized - recommended)
./scripts/dev/test-in-container.sh test

# Full CI suite (native)
./scripts/dev/run-ci-checks.sh --fix-format

# Build in CI environment
./scripts/dev/test-in-container.sh build
```

### Script Comparison

| Script | Format | Clippy | Tests | Speed | Environment | Use Case |
|--------|--------|--------|-------|-------|-------------|----------|
| `pre-push.sh` | ✓ | ✓ | ✗ | Fast | Native | Quick pre-push validation |
| `test-in-container.sh test` | ✓ | ✓ | ✓ | Medium | Container | Complete CI validation (recommended) |
| `run-ci-checks.sh` | ✓ | ✓ | ✓ | Slow | Native | Complete CI validation (alternative) |
| `test-in-container.sh build` | ✗ | ✗ | ✗ | Medium | Container | Build in CI environment |
| `run-format.sh` | ✓ | ✗ | ✗ | Very Fast | Native | Format-only check |
| `run-clippy.sh` | ✗ | ✓ | ✗ | Fast | Native | Clippy-only check |

## Arch Linux Packaging

### Creating AUR Packages

```bash
# Build software first
./scripts/build/build-linux.sh --profile release

# Create source archive for AUR
./scripts/build/package-arch.sh --source-only

# Test packaging
./scripts/build/test-arch-packaging.sh

# Validate PKGBUILD before submission
./scripts/build/test-pkgbuild-validation.sh

# Update PKGBUILD with correct checksum if needed
sha256sum target/ziplock-*.tar.gz
# Edit packaging/arch/PKGBUILD and replace 'SKIP' with actual checksum

# Submit to AUR (requires AUR account)
git clone ssh://aur@aur.archlinux.org/ziplock.git aur-ziplock
cp packaging/arch/PKGBUILD packaging/arch/ziplock.install aur-ziplock/
cd aur-ziplock
makepkg --printsrcinfo > .SRCINFO
git add .
git commit -m "Update to version X.Y.Z"
git push
```

### Testing Arch Packages

```bash
# Quick PKGBUILD validation (no Docker required)
./scripts/build/test-pkgbuild-validation.sh

# Quick packaging validation with Docker
./scripts/build/test-arch-packaging.sh

# Full build test (slow)
./scripts/build/test-arch-packaging.sh --full-build

# Test on real Arch system
cd packaging/arch
makepkg -si
```

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

## Deployment Scripts Details

### `deploy/setup-logging.sh`
Comprehensive logging setup script for production deployments:

**Features:**
- Creates system user and group for ZipLock
- Sets up log directories with proper permissions
- Generates systemd service file with security settings
- Configures logrotate for automatic log rotation
- Sets up systemd journal persistent storage
- Validates configuration before deployment

**Usage:**
```bash
# Standard installation
sudo ./scripts/deploy/setup-logging.sh

# Custom configuration
sudo ./scripts/deploy/setup-logging.sh --user myuser --log-dir /opt/logs
```

### `deploy/manage-service.sh`
Complete service management tool for deployed ZipLock instances:

**Commands:**
- `install` - Install ZipLock as systemd service
- `start/stop/restart` - Service lifecycle management
- `status` - Detailed service status and health
- `logs` - View systemd and application logs
- `monitor` - Interactive health monitoring
- `test` - Validate service configuration
- `uninstall` - Complete service removal

**Usage:**
```bash
# Service management
sudo ./scripts/deploy/manage-service.sh install
sudo ./scripts/deploy/manage-service.sh start
sudo ./scripts/deploy/manage-service.sh monitor

# Log management
sudo ./scripts/deploy/manage-service.sh logs 100
sudo ./scripts/deploy/manage-service.sh file-logs-follow
```

### `deploy/package-for-deployment.sh`
Creates comprehensive deployment packages with all necessary components:

**Package Contents:**
- Optimized binary for target platform
- Configuration templates for all environments
- Installation and management scripts
- Systemd service files
- Complete documentation
- Examples and troubleshooting guides

**Usage:**
```bash
# Standard package creation
./scripts/deploy/package-for-deployment.sh

# Custom configuration
./scripts/deploy/package-for-deployment.sh \
    --target aarch64-unknown-linux-gnu \
    --build-mode release \
    --package-name ziplock-arm64
```

## Development Scripts Details - Logging

### `dev/test-logging.sh`
Comprehensive logging system testing with multiple environments:

**Test Types:**
- Local development logging
- Production simulation
- Docker container logging
- Log rotation functionality
- Performance analysis

**Usage:**
```bash
# Full test suite
./scripts/dev/test-logging.sh

# Docker-only tests
./scripts/dev/test-logging.sh --docker-only

# Local tests only
./scripts/dev/test-logging.sh --local-only --skip-build
```

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
4. Test thoroughly: `./scripts/dev/test-in-container.sh test`
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

## FFI Architecture Notes

The scripts in this directory have been updated for ZipLock's unified FFI architecture:

- **No Backend Daemon**: Scripts no longer manage separate backend processes
- **Shared Library**: Applications use libziplock_shared via FFI calls
- **Single Process**: More memory-efficient operation
- **Simplified Testing**: No IPC/socket connectivity concerns
- **Universal Compatibility**: Same architecture works on desktop and mobile

## Script Maintenance

When adding new scripts:
1. Place them in the appropriate subdirectory based on function
2. Make them executable: `chmod +x script-name.sh`
3. Add proper error handling with `set -e`
4. Include colored output for better UX
5. Consider FFI architecture (no backend daemon assumptions)
6. Document usage with `--help` or usage functions
7. Update this README with the new script
8. Test scripts on clean environment before committing

### Script Categories
- **Build**: Compilation, packaging, distribution
- **Development**: Testing, debugging, development workflow
- **Version**: Version management, changelog maintenance
- **Migration**: Data and configuration migrations
- **Deploy**: Deployment automation (future use)

## Logging Configuration

ZipLock uses a sophisticated logging system with environment-specific configurations:

### Configuration File
The logging system reads from `config/logging.yaml` with support for multiple environments:
- `development` - Verbose logging with debug features
- `production` - Optimized logging for deployed systems
- `systemd` - Integration with systemd journal
- `docker` - Container-optimized logging
- `testing` - Comprehensive logging for test environments

### Environment Variables
Override logging behavior with environment variables:
```bash
ZIPLOCK_ENV=production           # Select logging profile
ZIPLOCK_LOG_DIR=/custom/path     # Override log directory
ZIPLOCK_LOG_CONSOLE_LEVEL=debug  # Override console log level
ZIPLOCK_LOG_FILE_LEVEL=info      # Override file log level
```

### Log Files
- **Application logs**: `/var/log/ziplock/ziplock.log`
- **Systemd logs**: `journalctl -u ziplock`
- **Rotation**: Automatic daily rotation, 30-day retention
- **Compression**: Old logs compressed automatically

## Getting Help

For script-specific help, most scripts support `--help`:
```bash
./scripts/build/build-linux.sh --help
./scripts/dev/run-integration-tests.sh --help
./scripts/deploy/manage-service.sh --help
./scripts/deploy/setup-logging.sh --help
./scripts/version/update-version.sh --help
```

For questions about the build system, logging, or suggestions for new scripts, please open an issue in the project repository.
For general questions about the build system or scripts, refer to:
- [Build Guide](../docs/technical/build.md) - Comprehensive build documentation
- [Technical Documentation](../docs/technical.md) - Architecture and implementation details
- Project issues and discussions on GitHub