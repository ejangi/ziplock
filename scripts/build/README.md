# ZipLock Build Scripts

This directory contains build scripts for creating ZipLock releases across multiple platforms.

## Quick Start

### Complete Release Build
```bash
# Build everything (Linux + Android)
./build-unified-release.sh

# Build with clean environment
./build-unified-release.sh -c -v
```

### Platform-Specific Builds
```bash
# Linux only
./build-linux.sh

# Android only  
./build-android-docker.sh build

# Mobile platforms (Android/iOS)
./build-mobile.sh -p android
```

## Script Overview

### Core Build Scripts

#### `build-unified-release.sh`
**Complete release orchestrator** - Creates unified releases with both Linux and Android artifacts.

```bash
# Usage examples
./build-unified-release.sh                    # Build everything
./build-unified-release.sh -p linux          # Linux only
./build-unified-release.sh -p android        # Android only
./build-unified-release.sh --skip-tests      # Skip testing phase
./build-unified-release.sh --update-checksums # Auto-update PKGBUILD checksums
```

**Output Structure:**
```
target/unified-release/
├── linux/
│   ├── binaries/
│   ├── packages/
│   └── install/
├── android/
│   ├── libraries/
│   ├── headers/
│   └── integration/
├── packaging/
│   ├── arch/
│   └── linux/
└── docs/
```

#### `build-linux.sh`
**Linux desktop build** - Compiles the desktop application and shared library.

```bash
./build-linux.sh                             # Standard build
PROFILE=debug ./build-linux.sh              # Debug build
```

**Creates:**
- Desktop application binary (`ziplock`)
- Shared library (`libziplock_shared.so`)
- Installation structure

#### `build-android-docker.sh`
**Android containerized build** - Uses Docker for consistent Android NDK environment.

```bash
./build-android-docker.sh build             # All architectures
./build-android-docker.sh build arm64       # ARM64 only
./build-android-docker.sh test              # Test libraries
./build-android-docker.sh verify            # Verify environment
./build-android-docker.sh shell             # Interactive shell
```

**Creates:**
- Native libraries for ARM64, ARMv7, x86_64, x86
- C header files
- Android app integration files

#### `build-mobile.sh`
**Mobile platform build** - Native mobile builds without Docker.

```bash
./build-mobile.sh -p android                # Android build
./build-mobile.sh -p ios                    # iOS build (macOS only)
./build-mobile.sh -p all                    # All mobile platforms
```

### Packaging Scripts

#### `package-deb.sh`
Creates Debian/Ubuntu packages (.deb files).

#### `package-arch.sh`
Creates Arch Linux packages (.pkg.tar.* files).

### Testing Scripts

#### `test-android-integration.sh`
**Android library testing**

```bash
./test-android-integration.sh basic         # Basic functionality
./test-android-integration.sh performance   # Performance analysis
./test-android-integration.sh security      # Security checks
./test-android-integration.sh all          # Complete test suite
```

#### `verify-android-symbols.sh`
**Symbol verification for Android libraries**

```bash
./verify-android-symbols.sh verify          # Basic verification
./verify-android-symbols.sh analyze         # Detailed analysis
./verify-android-symbols.sh export          # Export symbol tables
./verify-android-symbols.sh all            # Complete verification
```

## Environment Setup

### Linux Build Requirements
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libgtk-4-dev

# Arch Linux
sudo pacman -S base-devel pkg-config gtk4
```

### Android Build Requirements

**With Docker (Recommended):**
- Docker installed and running
- No additional Android SDK setup needed

**Without Docker:**
- Android NDK r25c or later
- `ANDROID_NDK_HOME` environment variable
- Rust Android targets: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android`

### Rust Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Android targets (if building natively)
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
```

## Build Workflows

### Development Workflow
```bash
# Quick development build
./build-linux.sh

# Test Android changes
./build-android-docker.sh build arm64
./test-android-integration.sh basic
```

### Release Workflow
```bash
# Complete release with updated checksums
./build-unified-release.sh --update-checksums

# Test release
cd target/unified-release
# Verify contents and test installations

# Create Arch package (version automatically updated)
cd packaging/arch
makepkg -si
```

### CI/CD Integration
```bash
# GitHub Actions compatible
./build-unified-release.sh --skip-tests     # Skip interactive tests
```

## Troubleshooting

### Common Issues

#### Linux Build Fails
- **Missing GTK4**: Install `libgtk-4-dev` (Ubuntu) or `gtk4` (Arch)
- **Rust not found**: Ensure `~/.cargo/bin` is in PATH
- **Test failures**: Use `--skip-tests` flag during development

#### Android Build Fails
- **Docker not running**: `sudo systemctl start docker`
- **Image pull fails**: Try `USE_REGISTRY=false` to build locally
- **NDK issues**: Use Docker build instead of native

#### Permission Issues
- **Script not executable**: `chmod +x scripts/build/*.sh`
- **Docker permissions**: Add user to docker group: `sudo usermod -aG docker $USER`

### Debug Mode
```bash
# Enable verbose output
./build-unified-release.sh -v

# Enable bash tracing
bash -x ./build-linux.sh
```

### Getting Help
- Check build logs in `target/` directory
- Run verification scripts for detailed analysis
- Use interactive shells: `./build-android-docker.sh shell`

## Script Dependencies

### Internal Dependencies
- Scripts call each other (e.g., unified build calls platform-specific scripts)
- Shared utility functions in each script

### External Dependencies
- **Required**: `git`, `cargo`, `rustc`
- **Linux**: `pkg-config`, GTK4 development libraries
- **Android**: Docker or Android NDK
- **Packaging**: `dpkg-deb`, `makepkg` (optional)

## Environment Variables

### Common Variables
- `PROFILE`: `debug` or `release` (default: `release`)
- `VERSION`: Override version number
- `VERBOSE`: Enable verbose output
- `UPDATE_CHECKSUMS`: Auto-calculate PKGBUILD checksums (default: `false`)

### Android-Specific
- `USE_REGISTRY`: Use Docker registry image (default: `true`)
- `ANDROID_NDK_HOME`: Path to Android NDK (native builds)
- `REGISTRY`: Docker registry (default: `ghcr.io`)

### Linux-Specific  
- `TARGET_ARCH`: Target architecture (default: `x86_64-unknown-linux-gnu`)

## File Structure

```
scripts/build/
├── README.md                        # This file
├── build-unified-release.sh         # Unified release build
├── build-linux.sh                   # Linux desktop build
├── build-android-docker.sh          # Android Docker build
├── build-mobile.sh                  # Mobile native build
├── package-deb.sh                   # Debian packaging
├── package-arch.sh                  # Arch Linux packaging
├── test-android-integration.sh      # Android testing
└── verify-android-symbols.sh        # Android symbol verification
```

## Contributing

When adding new build scripts:

1. **Follow naming convention**: `build-PLATFORM.sh`, `package-FORMAT.sh`, `test-COMPONENT.sh`
2. **Include help text**: `script.sh -h` should show usage
3. **Add error handling**: Use `set -euo pipefail`
4. **Log progress**: Use consistent logging functions
5. **Update this README**: Document new scripts and workflows

## Integration with CI/CD

These scripts are designed to work in GitHub Actions and other CI environments:

- **No interactive prompts** when using appropriate flags
- **Proper exit codes** for success/failure detection
- **Artifact generation** in predictable locations
- **Environment variable** support for customization