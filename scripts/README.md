# ZipLock Scripts

This directory contains utility scripts for ZipLock development and maintenance.

## Version Management

### `update-version.sh`
Automated version management and changelog update script.

**Usage:**
```bash
./scripts/update-version.sh [patch|minor|major] "changelog-entry"
```

**Examples:**
```bash
# Bug fix (increments patch version)
./scripts/update-version.sh patch "Fixed crash when opening large password files"

# New feature (increments minor version)
./scripts/update-version.sh minor "Added CSV export functionality"

# Breaking change (increments major version)
./scripts/update-version.sh major "New encryption system (requires data migration)"
```

**What it does:**
1. Increments version number in all `Cargo.toml` files
2. Adds user-friendly changelog entry to `CHANGELOG.md`
3. Shows summary of changes for review

**Requirements:**
- Must be run from project root directory
- Requires `CHANGELOG.md` and `Cargo.toml` files to exist

### `test-changelog-extraction.sh`
Tests the changelog extraction logic used by GitHub Actions for release notes.

**Usage:**
```bash
./scripts/test-changelog-extraction.sh
```

**What it does:**
1. Tests extraction of `[Unreleased]` section
2. Tests extraction of specific version sections
3. Shows available changelog sections
4. Verifies the same logic used in CI/CD pipeline

## Build Scripts

### `build-linux.sh`
Builds ZipLock for Linux platforms.

### `package-deb.sh`
Creates Debian packages for distribution.

### `test-build.sh` / `test-build-locally.sh`
Test build processes locally before CI/CD.

## Development Scripts

### `dev-run-linux.sh`
Runs ZipLock in development mode on Linux.

### `run-integration-tests.sh`
Executes the full integration test suite.

### `test-backend-connection.sh`
Tests backend service connectivity.

## Migration Scripts

Located in `scripts/migrations/` directory for database and configuration migrations.

## Version Management Guidelines

**Every code change must:**
1. Increment version by at least 0.0.1 using semantic versioning
2. Include a brief, user-friendly changelog entry
3. Use the `update-version.sh` script for consistency

**Changelog entries should be:**
- Written for end users, not developers
- Clear and non-technical
- Descriptive of what the change means to users

**Good examples:**
- ✅ "Fixed issue where app would crash when opening large password files"
- ✅ "Added ability to export passwords to CSV format"

**Bad examples:**
- ❌ "Refactored PasswordManager::decrypt() method to use async/await"
- ❌ "Updated dependencies in Cargo.toml"

## Release Process

1. Make your changes
2. Use `update-version.sh` to bump version and update changelog
3. Review changes: `git diff`
4. Test thoroughly
5. Commit: `git add . && git commit -m "Bump version to X.Y.Z"`
6. Tag: `git tag vX.Y.Z`
7. Push: `git push && git push --tags`

The GitHub Actions workflow will automatically:
- Extract changelog content for the version
- Generate release notes combining changelog with installation instructions
- Create GitHub release with artifacts
- Upload Debian packages

## Script Maintenance

When adding new scripts:
1. Make them executable: `chmod +x script-name.sh`
2. Add proper error handling with `set -e`
3. Include colored output for better UX
4. Document usage with `--help` or usage functions
5. Update this README with the new script
6. Test scripts on clean environment before committing