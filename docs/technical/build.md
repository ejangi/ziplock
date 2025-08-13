# Build Guide

This document provides comprehensive instructions for building ZipLock on Linux systems, including troubleshooting common issues, creating packages, and setting up automated builds. This guide consolidates build information to serve as the central reference for all build-related topics.

## Table of Contents

- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Build Process](#build-process)
- [Packaging](#packaging)
- [Installation and Testing](#installation-and-testing)
- [Troubleshooting](#troubleshooting)
- [CI/CD and GitHub Actions](#cicd-and-github-actions)
- [Development Guidelines](#development-guidelines)
- [Advanced Topics](#advanced-topics)

## Quick Start

For a standard development build:

```bash
# Clone the repository
git clone https://github.com/ejangi/ziplock.git
cd ziplock

# Make build scripts executable
chmod +x scripts/build-linux.sh
chmod +x scripts/package-deb.sh

# Build everything
./scripts/build-linux.sh

# Create Debian package
./scripts/package-deb.sh
```

This will create:
- Binaries in `target/release/`
- Installation structure in `target/install/`
- Debian package in `target/ziplock_*.deb`

## Prerequisites

### System Requirements

- **Operating System**: Ubuntu 20.04+, Debian 11+, or compatible Linux distribution
- **Architecture**: x86_64 (amd64)
- **Memory**: 2GB RAM minimum (4GB recommended for builds)
- **Disk Space**: 1GB free space for build artifacts

### Development Dependencies

#### Rust Toolchain
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required targets
rustup target add x86_64-unknown-linux-gnu
```

#### System Libraries

**Ubuntu/Debian:**
```bash
# Note: Ubuntu 22.04 ships with GTK 4.6, but the project requires GTK 4.8+
# Add PPA for newer GTK4 version
sudo add-apt-repository -y ppa:savoury1/display
sudo add-apt-repository -y ppa:savoury1/gtk4
sudo apt-get update

# Install base dependencies
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    libx11-dev \
    libxft-dev \
    liblzma-dev \
    git \
    curl

# Install GTK4 and related libraries (4.8+ from PPA)
sudo apt-get install -y \
    libgtk-4-dev \
    libgtk-4-1 \
    libgtk-4-common \
    libadwaita-1-dev \
    libadwaita-1-0 \
    libatk1.0-dev \
    libatk-bridge2.0-dev \
    libgtk-3-dev \
    libgdk-pixbuf-2.0-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libgdk-pixbuf2.0-dev \
    gir1.2-gtk-4.0

# For packaging
sudo apt-get install -y \
    dpkg-dev \
    fakeroot \
    dh-make \
    lintian

# Verify GTK4 version meets requirements
pkg-config --modversion gtk4
# Should show 4.8.0 or higher
```

**Fedora/RHEL/CentOS:**
```bash
# Install development tools
sudo dnf groupinstall "Development Tools"
sudo dnf install -y \
    pkg-config \
    fontconfig-devel \
    freetype-devel \
    libX11-devel \
    libXft-devel \
    xz-devel \
    gtk4-devel \
    libadwaita-devel
```

**Arch Linux:**
```bash
# Install dependencies
sudo pacman -S --needed \
    base-devel \
    rust \
    pkg-config \
    fontconfig \
    freetype2 \
    libx11 \
    libxft \
    xz \
    gtk4 \
    libadwaita

# AUR helper for additional packages
yay -S dpkg
```

## Build Process

### Environment Setup

```bash
# Set environment variables
export RUST_BACKTRACE=1
export CARGO_TARGET_DIR="$(pwd)/target"

# For release builds
export PROFILE=release

# For debug builds
export PROFILE=debug
```

### Building Components

#### 1. Shared Library
```bash
# Build the shared library first
cargo build --profile release -p ziplock-shared
```

#### 2. Backend Service
```bash
# Build the backend service
cargo build --profile release -p ziplock-backend

# Test the backend
./target/release/ziplock-backend --version
./target/release/ziplock-backend --help
```

#### 3. Frontend Application
```bash
# Build the Linux frontend
cargo build --profile release -p ziplock-linux

# Test the frontend
./target/release/ziplock --version
./target/release/ziplock --help
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific component tests
cargo test -p ziplock-shared
cargo test -p ziplock-backend
cargo test -p ziplock-linux

# Run with verbose output
cargo test --workspace -- --nocapture
```

### Code Quality Checks

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo install cargo-audit
cargo audit
```

## Packaging

### Debian Package Structure

The Debian package includes:

```
/usr/bin/ziplock-backend          # Backend service binary
/usr/bin/ziplock                  # Frontend GUI binary
/usr/share/applications/ziplock.desktop  # Desktop entry
/usr/share/icons/hicolor/scalable/apps/ziplock.svg  # Application icon
/lib/systemd/system/ziplock-backend.service  # Systemd service
/etc/ziplock/config.yml           # Default configuration
/var/lib/ziplock/                 # Service state directory
```

### Creating Packages

```bash
# Build the software first
./scripts/build-linux.sh --profile release

# Create the package
./scripts/package-deb.sh --arch amd64

# Verify the package
dpkg-deb --info target/ziplock_*.deb
dpkg-deb --contents target/ziplock_*.deb
lintian target/ziplock_*.deb
```

### Cross-Compilation

```bash
# Build for x86_64
./scripts/build-linux.sh --target x86_64-unknown-linux-gnu
./scripts/package-deb.sh --arch amd64
```

## Installation and Testing

### Package Installation

```bash
# Install the package
sudo dpkg -i target/ziplock_*.deb

# Fix any dependency issues
sudo apt-get install -f

# Verify installation
systemctl status ziplock-backend
ziplock --version
```

### Development Installation

```bash
# Install built binaries locally
sudo cp target/release/ziplock-backend /usr/local/bin/
sudo cp target/release/ziplock /usr/local/bin/

# Install desktop file
sudo cp frontend/linux/resources/ziplock.desktop /usr/share/applications/

# Update desktop database
sudo update-desktop-database
```

### Testing Installation

```bash
# Test backend service
ziplock-backend --version
ziplock-backend --help

# Test frontend application
ziplock --version
ziplock --help

# Test systemd service (if installed via .deb)
sudo systemctl status ziplock-backend
sudo systemctl start ziplock-backend
sudo systemctl stop ziplock-backend
```

### Package Removal

```bash
# Remove package (keeps configuration)
sudo apt-get remove ziplock

# Completely remove package and configuration
sudo apt-get purge ziplock
```

## Troubleshooting

### The glibc Compatibility Issue

#### Problem Description

One of the most common build issues manifests as:

```
ziplock: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by ziplock)
Error: Process completed with exit code 1.
```

#### Root Cause

The problem occurs because:

1. **Build Environment**: GitHub Actions was using `ubuntu-latest` (which could be Ubuntu 24.04 with glibc 2.39)
2. **Target Environment**: Testing was done in Ubuntu 22.04 (which has glibc 2.35)
3. **Binary Incompatibility**: Binaries compiled against newer glibc versions cannot run on systems with older glibc
4. **GTK4 Version Compatibility**: Ubuntu 22.04 ships with GTK 4.6.9, but the project requires GTK 4.8+ for gtk-gui features

#### Technical Details

- **glibc 2.35**: Available in Ubuntu 22.04 LTS
- **glibc 2.39**: Available in Ubuntu 24.04 and newer
- **Forward Compatibility**: glibc is forward-compatible but not backward-compatible
- **Symbol Versioning**: Modern glibc uses symbol versioning, making binaries depend on specific versions

#### Solution Implementation

**1. Containerized Build Process**

We implemented a containerized build process that ensures consistent environments:

```dockerfile
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config \
    libfontconfig1-dev libfreetype6-dev \
    # ... other dependencies

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

**2. Pinned Runner Environment**

All GitHub Actions jobs now use `ubuntu-22.04` instead of `ubuntu-latest`:

```yaml
jobs:
  build-linux:
    runs-on: ubuntu-22.04  # Pinned to specific version
```

**3. Build Environment Verification**

The workflow now includes verification steps:

```bash
# Check glibc version
ldd --version
# Analyze binary dependencies
objdump -T binary | grep GLIBC
```

### Local Testing with Containers

#### Using the Local Test Script

We provide a script to test the containerized build process locally:

```bash
# Run complete build test
./scripts/test-build-locally.sh

# Run with options
./scripts/test-build-locally.sh --clean --no-cache

# Skip package installation test
./scripts/test-build-locally.sh --skip-test
```

#### Manual Local Testing

If you prefer manual testing:

```bash
# 1. Build the Docker image
docker build -f Dockerfile.build -t ziplock-builder .

# 2. Run containerized build
docker run --rm -v $PWD:/workspace ziplock-builder bash -c "
    ./scripts/build-linux.sh --target x86_64-unknown-linux-gnu --profile release
"

# 3. Create package
docker run --rm -v $PWD:/workspace ziplock-builder bash -c "
    ./scripts/package-deb.sh --arch amd64
"

# 4. Test in clean environment
docker run --rm -v $PWD:/workspace ubuntu:22.04 bash -c "
    apt-get update && apt-get install -y ./workspace/target/ziplock_*_amd64.deb
"
```

### Common Build Issues

#### Missing System Dependencies
```bash
# Error: pkg-config not found
sudo apt-get install pkg-config

# Error: fontconfig not found
sudo apt-get install libfontconfig1-dev

# Error: X11 libraries not found
sudo apt-get install libx11-dev libxft-dev

# Error: GTK4 not found or version too old
# Add PPA for newer GTK4 version
sudo add-apt-repository -y ppa:savoury1/display
sudo add-apt-repository -y ppa:savoury1/gtk4
sudo apt-get update
sudo apt-get install -y libgtk-4-dev

# Verify GTK4 version
pkg-config --modversion gtk4
```

#### Rust Toolchain Issues
```bash
# Update Rust
rustup update

# Check Rust version
rustc --version
cargo --version

# Clean cargo cache
cargo clean
rm -rf ~/.cargo/registry/cache
```

#### Build Issues
```bash
# Verify system dependencies are installed
sudo apt-get install libfontconfig1-dev libfreetype6-dev libx11-dev libxft-dev liblzma-dev

# If GTK4 build fails, check version and reinstall from PPA
pkg-config --modversion gtk4
# Should be 4.8.0 or higher
sudo add-apt-repository -y ppa:savoury1/display
sudo add-apt-repository -y ppa:savoury1/gtk4
sudo apt-get update
sudo apt-get install -y libgtk-4-dev libadwaita-1-dev
```

### Common Runtime Issues

#### Package Installation Fails

**Symptoms:**
```
dpkg: dependency problems prevent configuration of ziplock
```

**Solutions:**

1. **Check Dependencies:**
   ```bash
   dpkg-deb --info target/ziplock_*_amd64.deb | grep Depends
   ```

2. **Install Dependencies First:**
   ```bash
   apt-get update
   apt-get install -f  # Fix broken dependencies
   ```

3. **Force Installation (if needed):**
   ```bash
   dpkg -i --force-depends target/ziplock_*_amd64.deb
   apt-get install -f
   ```

#### Binary Not Found After Installation

**Symptoms:**
```
bash: ziplock: command not found
```

**Solutions:**

1. **Check Installation Path:**
   ```bash
   dpkg -L ziplock | grep bin
   which ziplock-backend
   ```

2. **Verify Package Contents:**
   ```bash
   dpkg --contents target/ziplock_*_amd64.deb | grep usr/bin
   ```

3. **Check PATH:**
   ```bash
   echo $PATH
   /usr/bin/ziplock --version
   ```

#### GUI Application Won't Start

**Symptoms:**
```
Failed to initialize GTK
No display available
```

**Solutions:**

1. **For X11:**
   ```bash
   export DISPLAY=:0
   xhost +local:
   ```

2. **For Wayland:**
   ```bash
   export WAYLAND_DISPLAY=wayland-0
   ```

3. **Run in Container with Display:**
   ```bash
   docker run --rm -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix:rw ziplock
   ```

#### Backend Service Won't Start

**Symptoms:**
```
systemctl status ziplock-backend
● ziplock-backend.service - ZipLock Password Manager Backend Service
   Loaded: loaded (/lib/systemd/system/ziplock-backend.service; enabled; vendor preset: enabled)
   Active: failed (Result: exit-code) since ...
```

**Solutions:**

1. **Check Service Logs:**
   ```bash
   sudo journalctl -u ziplock-backend -f
   ```

2. **Check User Permissions:**
   ```bash
   id ziplock
   ls -la /var/lib/ziplock/
   ```

3. **Test Binary Manually:**
   ```bash
   sudo -u ziplock /usr/bin/ziplock-backend --version
   ```

4. **Check Configuration:**
   ```bash
   cat /etc/ziplock/config.yml
   ziplock-backend --config /etc/ziplock/config.yml --check-config
   ```

### Docker Testing

```bash
# Test package in clean Ubuntu environment
docker run --rm -it -v $(pwd):/workspace ubuntu:22.04 bash

# Inside container:
cd /workspace
apt-get update
apt-get install -y ./target/ziplock_*_amd64.deb
ziplock --version
systemctl status ziplock-backend
```

## CI/CD and GitHub Actions

### Automated Builds

The project includes GitHub Actions workflows for:

- **Continuous Integration**: Run tests on every push/PR
- **Security Audits**: Automated dependency security scanning
- **x86_64 Architecture Builds**: Create packages for amd64
- **Release Automation**: Automatic releases when tags are pushed

### GitHub Actions Workflow

#### Build Stages

1. **Environment Setup**: Creates containerized build environment
2. **Dependencies**: Installs all required system and Rust dependencies
3. **Build**: Compiles binaries in container
4. **Package**: Creates .deb package
5. **Test**: Validates package in clean Ubuntu 22.04 environment
6. **Analyze**: Checks binary dependencies and compatibility

#### Workflow Artifacts

The workflow produces several artifacts:

- **ziplock-linux-amd64**: Contains binaries and .deb package
- **build-report**: Detailed build analysis and environment info
- **benchmark-results**: Performance metrics (on main branch)

#### Debugging Failed Builds

1. **Check Build Environment:**
   ```yaml
   - name: Debug environment
     run: |
       cat /etc/os-release
       ldd --version
       rustc --version
   ```

2. **Analyze Binary Dependencies:**
   ```bash
   objdump -T target/release/ziplock-backend | grep GLIBC
   readelf -V target/release/ziplock-backend | grep GLIBC
   ```

3. **Download and Inspect Artifacts:**
   - Go to Actions tab in GitHub
   - Download build artifacts
   - Extract and examine build-report.txt

### Triggering a Release

```bash
# Create and push a new tag
git tag v1.0.0
git push origin v1.0.0

# This will trigger:
# 1. Full test suite
# 2. Security audit
# 3. Multi-architecture builds
# 4. Package creation
# 5. GitHub release with assets
```

### Local Testing of CI

```bash
# Test the same commands that CI runs
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit

# Build for x86_64
./scripts/build-linux.sh --target x86_64-unknown-linux-gnu
```

## Development Guidelines

### Adding New Dependencies

When adding new Rust dependencies:

1. **Check Compatibility:**
   ```bash
   cargo tree | grep -E "(glibc|libc)"
   ```

2. **Test in Container:**
   ```bash
   ./scripts/test-build-locally.sh --clean
   ```

3. **Verify Binary Size:**
   ```bash
   ls -lh target/release/ziplock*
   ```

### Platform-Specific Code

For platform-specific features:

```rust
#[cfg(target_os = "linux")]
mod linux_specific;

#[cfg(target_os = "windows")]
mod windows_specific;
```

### Build Profiles

We maintain several build profiles:

- **dev**: Fast compilation, debug info
- **release**: Optimized, stripped binaries
- **security**: Size-optimized, security-focused

### Testing Checklist

Before submitting PRs:

- [ ] Run `cargo test --workspace`
- [ ] Run `cargo clippy --all-targets`
- [ ] Run `./scripts/test-build-locally.sh`
- [ ] Test package installation in clean environment
- [ ] Verify binary dependencies with `ldd`

### Contributing to Build Process

#### Adding New Dependencies

1. Update `Cargo.toml` with new dependencies
2. Test on multiple Linux distributions
3. Update build documentation
4. Ensure CI/CD still passes

#### Modifying Build Scripts

1. Test changes locally first
2. Update script documentation
3. Test on clean environments
4. Update CI/CD if needed

#### Packaging Improvements

1. Follow Debian policy guidelines
2. Test package installation/removal
3. Verify systemd service integration
4. Check desktop integration

## Advanced Topics

### Performance Considerations

#### Build Time Optimization

- Use `--profile release` for final builds
- Enable LTO (Link Time Optimization) in release profile
- Use `cargo build --jobs $(nproc)` for parallel compilation
- Consider using `sccache` for caching compiled dependencies

#### Runtime Performance

- Backend service uses minimal resources when idle
- Frontend UI is optimized for responsiveness
- 7z compression can be tuned via configuration
- Memory usage scales with vault size

#### Binary Size

- Release builds are optimized for size
- Strip debug symbols for distribution
- Consider UPX compression for deployment
- Shared library reduces total installed size

### Build Performance

#### Speeding Up Builds
```bash
# Use more CPU cores
export CARGO_BUILD_JOBS=$(nproc)

# Use shared target directory
export CARGO_TARGET_DIR=/tmp/cargo-target

# Enable parallel linking
export RUSTFLAGS="-C link-arg=-fuse-ld=gold"
```

#### Reducing Binary Size
```bash
# Build with size optimization
cargo build --profile security

# Strip debug symbols
strip target/release/ziplock*

# Use UPX compression (optional)
upx --best target/release/ziplock*
```

### Debugging Package Issues

#### Package Won't Install
```bash
# Check package integrity
dpkg-deb --info target/ziplock_*.deb
dpkg-deb --contents target/ziplock_*.deb

# Test dependencies
apt-cache depends ziplock

# Force installation (debug only)
sudo dpkg -i --force-depends target/ziplock_*.deb
```

#### Lintian Warnings
```bash
# Run lintian for package quality checks
lintian target/ziplock_*.deb

# Check specific issues
lintian -i target/ziplock_*.deb
```

## Useful Commands and References

### Environment Variables

The build process respects these environment variables:

- `CARGO_TARGET_DIR`: Override cargo target directory
- `RUSTFLAGS`: Additional Rust compiler flags
- `TARGET_ARCH`: Target architecture (default: x86_64-unknown-linux-gnu)
- `PROFILE`: Build profile (dev/release)
- `VERSION`: Override package version

### Useful Commands

```bash
# Check glibc version
ldd --version

# Check binary dependencies
ldd /usr/bin/ziplock-backend

# Analyze symbols
objdump -T binary | grep GLIBC

# Check package metadata
dpkg-deb --info package.deb

# List package contents
dpkg --contents package.deb

# Check installed packages
dpkg -l | grep ziplock
```

### External Links

- [Rust Cross Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [glibc Compatibility](https://sourceware.org/glibc/wiki/Compatibility)
- [Debian Packaging Guide](https://www.debian.org/doc/manuals/debian-new-maintainers-guide/)
- [systemd Service Files](https://www.freedesktop.org/software/systemd/man/systemd.service.html)

## Getting Help

If you encounter issues not covered in this guide:

1. **Check existing issues** on GitHub
2. **Run the local test script** with `--help` for options
3. **Create a new issue** with:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Complete error output
   - Steps to reproduce

The containerized build process should eliminate most environment-related issues, but if you encounter problems, the local testing script can help identify and resolve them quickly.

For additional help, see the [development documentation](development.md) or open an issue on GitHub.