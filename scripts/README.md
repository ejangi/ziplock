# ZipLock Scripts

This directory contains utility scripts for ZipLock development and maintenance, organized into functional subdirectories. These scripts support the unified FFI architecture (no separate backend daemon).

## Directory Structure

### `build/` - Build and Packaging Scripts
Scripts for compiling, building, and packaging ZipLock for distribution.

- **`build-linux.sh`** - Builds ZipLock for Linux platforms
- **`build-mobile.sh`** - Builds shared library for mobile platforms (iOS/Android)
- **`package-deb.sh`** - Creates Debian packages for distribution
- **`package-arch.sh`** - Creates Arch Linux packages and source archives for AUR
- **`test-build.sh`** - Tests build process in CI environment
- **`test-build-locally.sh`** - Tests complete build process locally with Docker
- **`test-arch-packaging.sh`** - Tests Arch Linux packaging in containerized environment

### `dev/` - Development Scripts
Scripts for development workflow, testing, and debugging.

- **`run-linux.sh`** - Launches the unified ZipLock application for development
- **`run-integration-tests.sh`** - Executes the full integration test suite (FFI architecture)
- **`run-ci-checks.sh`** - Runs complete GitHub CI test suite locally (format, clippy, tests)
- **`run-clippy.sh`** - Quick Clippy linting check (same as GitHub CI)
- **`run-format.sh`** - Quick code formatting check and fix
- **`pre-push.sh`** - Quick pre-push validation (format + clippy, no tests)

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

# Create Arch Linux package
./scripts/build/package-arch.sh --source-only

# Test build locally with Docker
./scripts/build/test-build-locally.sh

# Test Arch packaging
./scripts/build/test-arch-packaging.sh
```

### Development
```bash
# Start unified development application
./scripts/dev/run-linux.sh

# Run integration tests (FFI architecture)
./scripts/dev/run-integration-tests.sh

# Run CI checks locally before pushing
./scripts/dev/run-ci-checks.sh

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

## Quick Reference

### Most Common Workflows

**Before pushing code:**
```bash
# Quick check (recommended for most commits)
./scripts/dev/pre-push.sh

# Quick check with auto-fix
./scripts/dev/pre-push.sh --fix

# Full validation (recommended before important commits)
./scripts/dev/run-ci-checks.sh
```

**Individual checks:**
```bash
# Format only
./scripts/dev/run-format.sh --fix

# Clippy only
./scripts/dev/run-clippy.sh

# Full CI suite
./scripts/dev/run-ci-checks.sh --fix-format
```

### Script Comparison

| Script | Format | Clippy | Tests | Speed | Use Case |
|--------|--------|--------|-------|-------|----------|
| `pre-push.sh` | ✓ | ✓ | ✗ | Fast | Quick pre-push validation |
| `run-ci-checks.sh` | ✓ | ✓ | ✓ | Slow | Complete CI validation |
| `run-format.sh` | ✓ | ✗ | ✗ | Very Fast | Format-only check |
| `run-clippy.sh` | ✗ | ✓ | ✗ | Fast | Clippy-only check |

## Arch Linux Packaging

### Creating AUR Packages

```bash
# Build software first
./scripts/build/build-linux.sh --profile release

# Create source archive for AUR
./scripts/build/package-arch.sh --source-only

# Test packaging
./scripts/build/test-arch-packaging.sh

# Update PKGBUILD with correct checksum
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
# Quick validation
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