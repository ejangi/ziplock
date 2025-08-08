# Linux Build Guide

This document provides comprehensive instructions for building ZipLock on Linux systems, including creating Debian packages and setting up automated builds.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Build](#quick-build)
- [Detailed Build Process](#detailed-build-process)
- [Debian Package Creation](#debian-package-creation)
- [Cross-Compilation](#cross-compilation)
- [GitHub Actions CI/CD](#github-actions-cicd)
- [Installation and Testing](#installation-and-testing)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Operating System**: Ubuntu 20.04+, Debian 11+, or compatible Linux distribution
- **Architecture**: x86_64 (amd64) or aarch64 (arm64)
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
rustup target add aarch64-unknown-linux-gnu  # For ARM64 builds
```

#### System Libraries
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    libx11-dev \
    libxft-dev \
    liblzma-dev \
    libgtk-4-dev \
    libadwaita-1-dev \
    git \
    curl

# For packaging
sudo apt-get install -y \
    dpkg-dev \
    fakeroot \
    dh-make \
    lintian

# For cross-compilation (ARM64)
sudo apt-get install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu
```

#### Fedora/RHEL/CentOS
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

# For cross-compilation
sudo dnf install -y gcc-aarch64-linux-gnu
```

#### Arch Linux
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

## Quick Build

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

## Detailed Build Process

### 1. Environment Setup

```bash
# Set environment variables
export RUST_BACKTRACE=1
export CARGO_TARGET_DIR="$(pwd)/target"

# For release builds
export PROFILE=release

# For debug builds
export PROFILE=debug
```

### 2. Building Components

#### Shared Library
```bash
# Build the shared library first
cargo build --profile release -p ziplock-shared
```

#### Backend Service
```bash
# Build the backend service
cargo build --profile release -p ziplock-backend

# Test the backend
./target/release/ziplock-backend --version
./target/release/ziplock-backend --help
```

#### Frontend Application
```bash
# Build the Linux frontend
cargo build --profile release -p ziplock-linux

# Test the frontend
./target/release/ziplock --version
./target/release/ziplock --help
```

### 3. Running Tests

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

### 4. Code Quality Checks

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

## Debian Package Creation

### Package Structure

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

### Manual Package Creation

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

### Package Removal

```bash
# Remove package (keeps configuration)
sudo apt-get remove ziplock

# Completely remove package and configuration
sudo apt-get purge ziplock
```

## Cross-Compilation

### Building for ARM64

```bash
# Install ARM64 toolchain
rustup target add aarch64-unknown-linux-gnu
sudo apt-get install gcc-aarch64-linux-gnu

# Set cross-compilation environment
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar

# Build for ARM64
./scripts/build-linux.sh --target aarch64-unknown-linux-gnu

# Create ARM64 package
./scripts/package-deb.sh --arch arm64
```

### Building for Multiple Architectures

```bash
# Build script for multiple architectures
for arch in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
    case $arch in
        x86_64-unknown-linux-gnu)
            deb_arch=amd64
            ;;
        aarch64-unknown-linux-gnu)
            deb_arch=arm64
            export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
            export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
            ;;
    esac
    
    echo "Building for $arch ($deb_arch)..."
    ./scripts/build-linux.sh --target $arch --profile release
    ./scripts/package-deb.sh --arch $deb_arch
done
```

## GitHub Actions CI/CD

### Automated Builds

The project includes GitHub Actions workflows for:

- **Continuous Integration**: Run tests on every push/PR
- **Security Audits**: Automated dependency security scanning
- **Multi-Architecture Builds**: Create packages for amd64 and arm64
- **Release Automation**: Automatic releases when tags are pushed

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

# Build for both architectures (if cross-compilation is set up)
./scripts/build-linux.sh --target x86_64-unknown-linux-gnu
./scripts/build-linux.sh --target aarch64-unknown-linux-gnu
```

## Installation and Testing

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

## Troubleshooting

### Common Build Issues

#### Missing System Dependencies
```bash
# Error: pkg-config not found
sudo apt-get install pkg-config

# Error: fontconfig not found
sudo apt-get install libfontconfig1-dev

# Error: X11 libraries not found
sudo apt-get install libx11-dev libxft-dev
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

#### Cross-Compilation Issues
```bash
# Install missing cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu

# Set correct environment variables
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
```

### Runtime Issues

#### Backend Service Won't Start
```bash
# Check service status
sudo systemctl status ziplock-backend

# View service logs
sudo journalctl -u ziplock-backend -f

# Check configuration
sudo cat /etc/ziplock/config.yml

# Verify permissions
sudo ls -la /var/lib/ziplock/
```

#### Frontend Won't Launch
```bash
# Check for missing libraries
ldd target/release/ziplock

# Run with debug output
RUST_LOG=debug ziplock

# Check display environment
echo $DISPLAY
echo $WAYLAND_DISPLAY
```

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

## Performance Considerations

### Build Time Optimization

- Use `--profile release` for final builds
- Enable LTO (Link Time Optimization) in release profile
- Use `cargo build --jobs $(nproc)` for parallel compilation
- Consider using `sccache` for caching compiled dependencies

### Runtime Performance

- Backend service uses minimal resources when idle
- Frontend UI is optimized for responsiveness
- 7z compression can be tuned via configuration
- Memory usage scales with vault size

### Binary Size

- Release builds are optimized for size
- Strip debug symbols for distribution
- Consider UPX compression for deployment
- Shared library reduces total installed size

## Contributing to Build Process

### Adding New Dependencies

1. Update `Cargo.toml` with new dependencies
2. Test on multiple Linux distributions
3. Update build documentation
4. Ensure CI/CD still passes

### Modifying Build Scripts

1. Test changes locally first
2. Update script documentation
3. Test on clean environments
4. Update CI/CD if needed

### Packaging Improvements

1. Follow Debian policy guidelines
2. Test package installation/removal
3. Verify systemd service integration
4. Check desktop integration

For additional help, see the [development documentation](development.md) or open an issue on GitHub.