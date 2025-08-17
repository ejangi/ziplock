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
chmod +x scripts/build/build-linux.sh
chmod +x scripts/build/package-deb.sh

# Build everything
./scripts/build/build-linux.sh

# Create Debian package
./scripts/build/package-deb.sh
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

### Related Documentation

Before building, you may want to review:
- [Configuration Guide](configuration.md) - Understanding configuration options for deployment
- [Mobile Integration](mobile-integration.md) - Examples for integrating with mobile platforms

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
# Note: Ubuntu 22.04 ships with GTK 4.6, which is compatible with the project
# Update package lists
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

# Install GTK4 and related libraries (using Ubuntu 22.04 default versions)
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

# Verify GTK4 installation
pkg-config --modversion gtk4
# Should show 4.6.x from Ubuntu repositories
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

#### 2. Unified Application
```bash
# Build the unified ZipLock application (includes FFI client)
cargo build --profile release -p ziplock-linux

# Test the application (GUI app - may require display)
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

ZipLock supports packaging for multiple Linux distributions:

### Debian/Ubuntu Package Structure

The Debian package includes:

```
/usr/bin/ziplock                  # Unified GUI application binary
/usr/lib/libziplock_shared.so     # Shared library (FFI)
/usr/share/applications/ziplock.desktop  # Desktop entry
/usr/share/icons/hicolor/scalable/apps/ziplock.svg  # Application icon
/etc/ziplock/config.yml           # Default configuration
```

### Arch Linux Package Structure

The Arch package includes the same files as Debian, but follows Arch conventions:

```
/usr/bin/ziplock                  # Unified GUI application binary
/usr/lib/libziplock_shared.so     # Shared library (FFI)
/usr/share/applications/ziplock.desktop  # Desktop entry
/usr/share/icons/hicolor/scalable/apps/ziplock.svg  # Application icon
/etc/ziplock/config.yml           # Default configuration
/usr/share/licenses/ziplock/LICENSE  # License file
/usr/share/doc/ziplock/           # Documentation
```

### Creating Packages

#### Debian/Ubuntu Packages

```bash
# Build the software first
./scripts/build/build-linux.sh --profile release

# Create the package
./scripts/build/package-deb.sh --arch amd64

# Verify the package
dpkg-deb --info target/ziplock_*.deb
dpkg-deb --contents target/ziplock_*.deb
lintian target/ziplock_*.deb
```

#### Arch Linux Packages

```bash
# Build the software first
./scripts/build/build-linux.sh --profile release

# Create source archive for AUR
./scripts/build/package-arch.sh --source-only

# Or create binary package (requires Arch Linux)
./scripts/build/package-arch.sh

# Verify the package (on Arch Linux)
pacman -Qip target/ziplock-*.pkg.tar.xz
pacman -Qlp target/ziplock-*.pkg.tar.xz
```

#### AUR (Arch User Repository) Publishing

For AUR submission, use the source-only mode:

```bash
# Create source archive and PKGBUILD
./scripts/build/package-arch.sh --source-only

# Update PKGBUILD with correct SHA256 checksum
# Edit packaging/arch/PKGBUILD and replace 'SKIP' with actual checksum
sha256sum target/ziplock-*.tar.gz

# Test build on Arch Linux
cd packaging/arch
makepkg -si

# Submit to AUR (requires AUR account and SSH keys)
git clone ssh://aur@aur.archlinux.org/ziplock.git aur-ziplock
cp PKGBUILD ziplock.install aur-ziplock/
cd aur-ziplock
makepkg --printsrcinfo > .SRCINFO
git add .
git commit -m "Update to version X.Y.Z"
git push
```

### Cross-Compilation

#### Linux Targets

```bash
# Build for x86_64
./scripts/build/build-linux.sh --target x86_64-unknown-linux-gnu

# Create packages
./scripts/build/package-deb.sh --arch amd64
./scripts/build/package-arch.sh --arch x86_64
```

#### Android Cross-Compilation

ZipLock supports full Android cross-compilation through a containerized build environment. The Android builder provides pre-configured toolchains for all major Android architectures.

##### Supported Android Architectures

- **ARM64-v8a** (`aarch64-linux-android`): Primary Android 64-bit ARM target
- **ARMv7** (`armv7-linux-androideabi`): Legacy 32-bit ARM support  
- **x86_64** (`x86_64-linux-android`): 64-bit emulator support
- **x86** (`i686-linux-android`): 32-bit emulator support

##### Quick Start - Android Builds

```bash
# Build all Android architectures using pre-built container
./scripts/build/build-android-docker.sh build

# Build specific architecture
./scripts/build/build-android-docker.sh build arm64

# Test built libraries
./scripts/build/build-android-docker.sh test

# Verify build environment
./scripts/build/test-android-readiness.sh
```

##### Android Builder Container

The build system uses a pre-built Docker container hosted on GitHub Container Registry:

```bash
# Pull the latest Android builder image
docker pull ghcr.io/ejangi/ziplock/android-builder:latest

# Run interactive container for development
docker run -it --rm \
  -v "$PWD:/workspace" \
  -w /workspace \
  ghcr.io/ejangi/ziplock/android-builder:latest \
  bash

# Direct compilation example
docker run --rm -v "$PWD:/workspace" -w /workspace \
  ghcr.io/ejangi/ziplock/android-builder:latest \
  bash -c "cd shared && cargo check --target aarch64-linux-android --features c-api"
```

##### Android Build Configuration

The Android builder includes:

- **Base OS**: Ubuntu 22.04 LTS
- **Rust Toolchain**: Latest stable with Android targets
- **Android NDK**: Version 25.2.9519653  
- **API Level**: 21 (Android 5.0+)
- **Pre-configured**: Cross-compilation toolchains, cargo configuration, NDK environment

##### Build Options and Fallbacks

```bash
# Use registry image (default, recommended)
./scripts/build/build-android-docker.sh build

# Force local image build if registry unavailable
USE_REGISTRY=false ./scripts/build/build-android-docker.sh build

# Set environment permanently for offline development
export USE_REGISTRY=false
./scripts/build/build-android-docker.sh build
```

##### Testing Android Builds

```bash
# Test registry image functionality
./scripts/build/test-android-builder-image.sh registry

# Test local image functionality  
./scripts/build/test-android-builder-image.sh local

# Test both images
./scripts/build/test-android-builder-image.sh both

# Verify tools and compilation
docker run --rm ghcr.io/ejangi/ziplock/android-builder:latest \
  bash -c "rustc --version && aarch64-linux-android21-clang --version"
```

##### Android Build Outputs

Successful Android builds produce:

- **Shared Libraries**: `.so` files for each target architecture
- **C Header**: `ziplock.h` for native integration
- **Size**: ~1.5MB per architecture (optimized release builds)
- **Location**: `target/{architecture}/release/`

##### Integration with CI/CD

The Android builder is integrated into GitHub Actions workflows:

- **Automated Builds**: Triggered on changes to Android-related files
- **Registry Authentication**: GitHub Actions token-based
- **Performance**: ~60% faster builds using pre-built images
- **Fallback**: Automatic local build if registry unavailable
- **Weekly Updates**: Container rebuilds every Sunday for security updates

## Installation and Testing

### Package Installation

#### Debian/Ubuntu

```bash
# Install the package
sudo dpkg -i target/ziplock_*.deb

# Fix any dependency issues
sudo apt-get install -f

# Verify installation
ziplock --version
ldconfig -p | grep ziplock
```

#### Arch Linux

```bash
# Install from binary package
sudo pacman -U target/ziplock-*.pkg.tar.xz

# Or install from AUR (once published)
yay -S ziplock
# or
paru -S ziplock

# Launch the application
ziplock

# Verify installation
ziplock --version
```

### Development Installation

```bash
# Install built binaries locally
sudo cp target/release/ziplock-backend /usr/local/bin/
sudo cp target/release/ziplock /usr/local/bin/

# Install desktop file
sudo cp apps/linux/resources/ziplock.desktop /usr/share/applications/

# Update desktop database
sudo update-desktop-database
```

### Testing Installation

```bash
# Test unified application
ziplock --version
ziplock --help

# Test shared library is available
ldconfig -p | grep ziplock
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
4. **GTK4 Version Compatibility**: Ubuntu 22.04 ships with GTK 4.6.9, which is compatible with gtk-gui features

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

#### Using the Containerized Test Script

We provide a script to test in the exact same containerized environment as CI:

```bash
# Run complete test suite (formatting, clippy, tests, build)
./scripts/dev/test-in-container.sh test

# Build binaries only
./scripts/dev/test-in-container.sh build

# Create packages
./scripts/dev/test-in-container.sh package-deb
./scripts/dev/test-in-container.sh package-arch

# Interactive debugging shell
./scripts/dev/test-in-container.sh shell-ubuntu
```

#### Manual Local Testing

If you prefer manual testing:

```bash
# 1. Build the Docker image
docker build -f Dockerfile.build -t ziplock-builder .

# 2. Run containerized build
docker run --rm -v $PWD:/workspace ziplock-builder bash -c "
    ./scripts/build/build-linux.sh --target x86_64-unknown-linux-gnu --profile release
"

# 3. Create package
docker run --rm -v $PWD:/workspace ziplock-builder bash -c "
    ./scripts/build/package-deb.sh --arch amd64
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

# If GTK4 build fails, check version and reinstall
pkg-config --modversion gtk4
# Should be 4.6.x or higher
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

#### Application Won't Start

**Symptoms:**
```
# ZipLock is now a unified GUI application, no systemd service required
ziplock --version
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
ldconfig -p | grep ziplock
```

### Arch Linux Specific Issues

#### PKGBUILD Build Fails

**Symptoms:**
```
==> ERROR: makepkg must be run as a normal user
```

**Solutions:**

1. **Never run makepkg as root:**
   ```bash
   # Correct - run as normal user
   makepkg -si
   
   # Wrong - never do this
   sudo makepkg
   ```

2. **If building in container, create non-root user:**
   ```bash
   useradd -m builder
   su - builder
   cd /path/to/PKGBUILD
   makepkg -si
   ```

#### Package Installation Conflicts

**Symptoms:**
```
error: failed to commit transaction (conflicting files)
ziplock: /usr/bin/ziplock exists in filesystem
```

**Solutions:**

1. **Check for existing installations:**
   ```bash
   pacman -Qs ziplock
   which ziplock
   ```

2. **Remove conflicting packages:**
   ```bash
   sudo pacman -R ziplock-git  # Remove AUR git version
   sudo pacman -U ziplock-*.pkg.tar.xz --overwrite '*'
   ```

#### Missing Dependencies on Arch

**Symptoms:**
```
error while loading shared libraries: libfontconfig.so.1
```

**Solutions:**

1. **Install base dependencies:**
   ```bash
   sudo pacman -S base-devel fontconfig freetype2 libx11 libxft xz
   ```

2. **For GUI dependencies:**
   ```bash
   sudo pacman -S gtk4 libadwaita
   ```

3. **Check package dependencies:**
   ```bash
   pacman -Qip ziplock-*.pkg.tar.xz | grep "Depends On"
   ```

#### AUR Package Out of Date

**Symptoms:**
- AUR package shows older version than latest release

**Solutions:**

1. **Check current version:**
   ```bash
   curl -s https://api.github.com/repos/ejangi/ziplock/releases/latest | grep tag_name
   ```

2. **Flag package out of date on AUR web interface**

3. **Build from source manually:**
   ```bash
   git clone https://github.com/ejangi/ziplock.git
   cd ziplock
   cargo build --release
   ```

#### Systemd Service Issues on Arch

**Symptoms:**
```
ziplock: command not found
```

**Solutions:**

1. **Check service file location (Arch uses /usr/lib):**
   ```bash
   ls -la /usr/bin/ziplock /usr/lib/libziplock_shared.so
   ```

2. **Reload systemd if service was just installed:**
   ```bash
   # Verify installation
   which ziplock
   ldd /usr/bin/ziplock
   ```

3. **Check service file syntax:**
   ```bash
   ldd /usr/bin/ziplock
   ```

### Docker Testing for Arch

```bash
# Test package in clean Arch environment
docker run --rm -it -v $(pwd):/workspace archlinux:latest bash

# Inside container:
cd /workspace
pacman -Syu --noconfirm
pacman -U --noconfirm ziplock-*.pkg.tar.xz
ziplock --version
ldconfig -p | grep ziplock
```

## CI/CD and GitHub Actions

### Automated Builds

The project includes optimized GitHub Actions workflows for:

- **Continuous Integration**: Run tests once on stable Rust toolchain
- **Security Audits**: Automated dependency security scanning with caching
- **Efficient Multi-Distribution Builds**: Create packages using pre-built container images
- **Release Automation**: Automatic releases when tags are pushed
- **Container Image Management**: Pre-built images for consistent environments

### Optimized GitHub Actions Workflow

#### Build Strategy

The workflow uses an efficient artifact-sharing approach with pre-built containers:

**Test and Build Job (`test-and-build`):**
1. **Single Build**: Compiles all binaries once using cached dependencies
2. **Testing**: Runs formatter, clippy, and test suite on stable Rust only
3. **Artifact Upload**: Shares compiled binaries with packaging jobs

**Debian Packaging (`package-debian`):**
1. **Artifact Download**: Reuses pre-compiled binaries
2. **Container Packaging**: Uses pre-built Ubuntu container image
3. **Package Creation**: Creates .deb package without rebuilding
4. **Installation Test**: Validates package in clean environment

**Arch Packaging (`package-arch`):**
1. **Artifact Download**: Reuses pre-compiled binaries  
2. **Container Packaging**: Uses pre-built Arch container image
3. **Source Package**: Creates source archive and PKGBUILD for AUR

**Key Optimizations:**
- **50% faster**: Removed redundant beta Rust testing
- **60% fewer builds**: Build once, package multiple times
- **Consistent environments**: Pre-built container images
- **Better caching**: Optimized cargo and dependency caching
- **Parallel packaging**: Debian and Arch jobs run simultaneously

#### Workflow Artifacts

The optimized workflow produces focused artifacts:

- **compiled-binaries**: Shared binaries used by all packaging jobs
- **debian-package**: Ready-to-install .deb package
- **arch-package**: Source archive and PKGBUILD for AUR
- **benchmark-results**: Performance metrics (on main branch)

#### Container Images

Pre-built container images are maintained automatically and hosted on GitHub Container Registry at `ghcr.io/ejangi/ziplock`:

##### Available Images

- **ubuntu-builder**: Ubuntu 22.04 with Rust toolchain and all dependencies for Linux builds
- **arch-builder**: Arch Linux with build tools and packaging utilities  
- **android-builder**: Ubuntu 22.04 with Android NDK r25c and cross-compilation tools

##### Android Builder Specifications

The `android-builder` image provides a complete Android development environment:

**Base Configuration:**
- Ubuntu 22.04 LTS base with security updates
- Rust toolchain with Android target support
- Android NDK 25.2.9519653 (API level 21+)
- Pre-configured cargo settings for Android cross-compilation

**Supported Targets:**
- `aarch64-linux-android` (ARM64-v8a)
- `armv7-linux-androideabi` (ARMv7)  
- `x86_64-linux-android` (x86_64 emulator)
- `i686-linux-android` (x86 emulator)

**Automated Maintenance:**
- **Weekly Updates**: Images rebuild every Sunday at 2:00 AM UTC
- **Security Patches**: Automatic base image updates included
- **Tool Updates**: Rust toolchain and NDK kept current
- **Vulnerability Scanning**: GitHub's automatic security scanning enabled

**Usage in Workflows:**
```yaml
jobs:
  build-android:
    runs-on: ubuntu-latest
    container: ghcr.io/ejangi/ziplock/android-builder:latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Android libraries
        run: ./scripts/build/build-android-docker.sh build
```

**Performance Benefits:**
- 60% faster Android builds in CI using cached images
- Eliminated environment setup time and failures
- Consistent build environment across all development machines
- Reduced GitHub Actions minutes usage

**Fallback Mechanism:**
- Automatic fallback to local image build if registry unavailable
- Environment variable control: `USE_REGISTRY=false` forces local builds
- Comprehensive error handling and user feedback

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

#### SIGPIPE Error Fix (Exit Code 141)

**Problem**: GitHub Actions workflows failing with exit code 141 due to SIGPIPE when using `set -euo pipefail` with pipe operations.

**Root Cause**: Commands like `ldd --version | head -1` cause SIGPIPE when `head` exits before the upstream command finishes writing, triggering script failure due to `pipefail`.

**Solution Applied**:

1. **Avoid problematic pipes in critical sections:**
   ```bash
   # Before (problematic):
   set -euo pipefail
   ldd --version | head -1
   
   # After (safe):
   set -euo  # Remove pipefail where pipes are used
   glibc_version=$(ldd --version 2>/dev/null | head -1) || glibc_version='unknown'
   echo "$glibc_version"
   ```

2. **Use variable assignments for pipe operations:**
   ```bash
   # Safe pattern for multiple commands:
   result=$(command1 2>/dev/null | command2) || result="fallback value"
   echo "$result"
   ```

3. **Add proper error handling:**
   ```bash
   # Include error redirection and fallbacks
   binary_deps=$(ldd /path/to/binary 2>/dev/null | head -5) || binary_deps="dependency check failed"
   ```

**Files Modified**:
- `.github/workflows/linux-build.yml` - Fixed pipe operations in container execution
- `scripts/build/build-linux.sh` - Fixed version extraction using `sed -n` instead of `head`

This fix maintains all security and compatibility features while preventing pipeline failures from SIGPIPE errors.

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

#### Using Containerized Testing (Recommended)

```bash
# Test in the same environment as CI
./scripts/dev/test-in-container.sh test

# Build binaries in container
./scripts/dev/test-in-container.sh build

# Create packages in containers
./scripts/dev/test-in-container.sh package-deb
./scripts/dev/test-in-container.sh package-arch

# Interactive container shell for debugging
./scripts/dev/test-in-container.sh shell-ubuntu
```

#### Native Testing (Alternative)

```bash
# Test the same commands that CI runs
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit

# Build for x86_64
./scripts/build/build-linux.sh --target x86_64-unknown-linux-gnu
```

**Note**: Use containerized testing to ensure 100% consistency with CI environment.

## Development Guidelines

### Adding New Dependencies

When adding new Rust dependencies:

1. **Check Compatibility:**
   ```bash
   cargo tree | grep -E "(glibc|libc)"
   ```

2. **Test in Container:**
   ```bash
   ./scripts/dev/test-in-container.sh test --clean
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
- [ ] Run `./scripts/dev/test-in-container.sh test`
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

- Unified application uses minimal resources when idle
- GUI is optimized for responsiveness
- Shared library enables efficient memory usage
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

For additional help, see the [technical documentation index](../technical.md) or open an issue on GitHub.