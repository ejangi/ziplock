# Build Troubleshooting Guide

This document provides comprehensive guidance for troubleshooting build issues with ZipLock, particularly focusing on the glibc compatibility problem that was resolved in the GitHub Actions workflow.

## Table of Contents

- [Overview](#overview)
- [The glibc Compatibility Issue](#the-glibc-compatibility-issue)
- [Solution Implementation](#solution-implementation)
- [Local Testing](#local-testing)
- [Troubleshooting Common Issues](#troubleshooting-common-issues)
- [GitHub Actions Workflow](#github-actions-workflow)
- [Development Guidelines](#development-guidelines)

## Overview

ZipLock is a cross-platform password manager written in Rust. The build process involves compiling native binaries for different target platforms and packaging them for distribution. This document addresses build-related issues, particularly those related to binary compatibility across different Linux distributions.

## The glibc Compatibility Issue

### Problem Description

The original issue manifested as:

```
ziplock: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by ziplock)
Error: Process completed with exit code 1.
```

### Root Cause

The problem occurred because:

1. **Build Environment**: GitHub Actions was using `ubuntu-latest` (which could be Ubuntu 24.04 with glibc 2.39)
2. **Target Environment**: Testing was done in Ubuntu 22.04 (which has glibc 2.35)
3. **Binary Incompatibility**: Binaries compiled against newer glibc versions cannot run on systems with older glibc
4. **GTK4 Version Compatibility**: Ubuntu 22.04 ships with GTK 4.6.9, but the project requires GTK 4.8+ for gtk-gui features

### Technical Details

- **glibc 2.35**: Available in Ubuntu 22.04 LTS
- **glibc 2.39**: Available in Ubuntu 24.04 and newer
- **Forward Compatibility**: glibc is forward-compatible but not backward-compatible
- **Symbol Versioning**: Modern glibc uses symbol versioning, making binaries depend on specific versions

## Solution Implementation

### 1. Containerized Build Process

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

### 2. Pinned Runner Environment

All GitHub Actions jobs now use `ubuntu-22.04` instead of `ubuntu-latest`:

```yaml
jobs:
  build-linux:
    runs-on: ubuntu-22.04  # Pinned to specific version
```

### 3. Build Environment Verification

The workflow now includes verification steps:

```bash
# Check glibc version
ldd --version
# Analyze binary dependencies
objdump -T binary | grep GLIBC
```

## Local Testing

### Using the Local Test Script

We provide a script to test the containerized build process locally:

```bash
# Run complete build test
./scripts/test-build-locally.sh

# Run with options
./scripts/test-build-locally.sh --clean --no-cache

# Skip package installation test
./scripts/test-build-locally.sh --skip-test
```

### Manual Local Testing

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

## Troubleshooting Common Issues

### Issue: Package Installation Fails

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

### Issue: Binary Not Found After Installation

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

### Issue: GUI Application Won't Start

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

### Issue: Service Won't Start

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
   journalctl -u ziplock-backend.service -f
   ```

2. **Check User Permissions:**
   ```bash
   id ziplock
   ls -la /var/lib/ziplock
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

## GitHub Actions Workflow

### Build Stages

1. **Environment Setup**: Creates containerized build environment
2. **Dependencies**: Installs all required system and Rust dependencies
3. **Build**: Compiles binaries in container
4. **Package**: Creates .deb package
5. **Test**: Validates package in clean Ubuntu 22.04 environment
6. **Analyze**: Checks binary dependencies and compatibility

### Workflow Artifacts

The workflow produces several artifacts:

- **ziplock-linux-amd64**: Contains binaries and .deb package
- **build-report**: Detailed build analysis and environment info
- **benchmark-results**: Performance metrics (on main branch)

### Debugging Failed Builds

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

## Additional Resources

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

### Environment Variables

The build process respects these environment variables:

- `CARGO_TARGET_DIR`: Override cargo target directory
- `RUSTFLAGS`: Additional Rust compiler flags
- `TARGET_ARCH`: Target architecture (default: x86_64-unknown-linux-gnu)
- `PROFILE`: Build profile (dev/release)
- `VERSION`: Override package version

### Links

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