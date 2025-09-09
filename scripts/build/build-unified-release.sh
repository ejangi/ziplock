#!/bin/bash
set -euo pipefail

# ZipLock Unified Release Build Script
# Builds both Linux and Android artifacts for a complete release

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
RELEASE_DIR="$PROJECT_ROOT/target/unified-release"
VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')}"
BUILD_TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
GIT_BRANCH="$(git branch --show-current 2>/dev/null || echo 'unknown')"

# Build options
PLATFORMS=""
CLEAN=false
VERBOSE=false
SKIP_TESTS=false
SKIP_PACKAGES=false
PARALLEL_BUILDS=true
DOCKER_BUILDS=true
UPDATE_CHECKSUMS=false

# Logging functions
log_header() {
    echo
    echo -e "${CYAN}================================================${NC}"
    echo -e "${CYAN} $1${NC}"
    echo -e "${CYAN}================================================${NC}"
}

log_section() {
    echo
    echo -e "${BLUE}=== $1 ===${NC}"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${CYAN}→${NC} $1"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build unified ZipLock release with Linux and Android artifacts.

OPTIONS:
    -p, --platforms PLATFORMS    Platforms to build: linux, android, all (default: all)
    -c, --clean                  Clean all build artifacts before building
    -v, --verbose                Enable verbose output
    --skip-tests                 Skip running tests
    --skip-packages              Skip creating distribution packages
    --no-parallel                Disable parallel builds
    --no-docker                  Don't use Docker for Android builds
    --update-checksums           Calculate and update PKGBUILD checksums (requires curl)
    --version VERSION            Override version number
    -o, --output DIR             Output directory (default: target/unified-release)
    -h, --help                   Show this help message

PLATFORMS:
    linux      Build Linux desktop application and packages
    android    Build Android native libraries
    all        Build for all platforms (default)

EXAMPLES:
    $0                          # Build everything
    $0 -p linux                 # Build only Linux
    $0 -p android --no-docker   # Build Android without Docker
    $0 -c -v                    # Clean build with verbose output
    $0 --skip-tests             # Build without running tests

OUTPUTS:
    The script creates a unified release structure in the output directory:

    target/unified-release/
    ├── linux/
    │   ├── binaries/           # Linux executables
    │   ├── packages/           # .deb and .pkg files
    │   └── install/            # Installation structure
    ├── android/
    │   ├── libraries/          # .so files by architecture
    │   ├── headers/            # C header files
    │   └── integration/        # Android app integration files
    ├── packaging/              # Platform packaging files
    │   ├── arch/               # Arch Linux PKGBUILD and install scripts
    │   └── linux/              # Linux packaging configurations
    ├── docs/                   # Documentation and guides
    ├── release-info.json       # Build metadata
    └── RELEASE_NOTES.md        # Release notes

EOF
}

# Check build prerequisites
check_prerequisites() {
    log_section "Checking Prerequisites"

    local missing_tools=()
    local warnings=()

    # Check essential tools
    for tool in git cargo rustc; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing essential tools: ${missing_tools[*]}"
        exit 1
    fi

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | awk '{print $2}')
    log_info "Rust version: $rust_version"

    # Check for Linux build dependencies
    if [[ "$PLATFORMS" == *"linux"* ]] || [ "$PLATFORMS" = "all" ]; then
        log_step "Checking Linux build dependencies..."

        for tool in pkg-config; do
            if ! command -v "$tool" &> /dev/null; then
                warnings+=("$tool (needed for Linux builds)")
            fi
        done

        # Check for GTK4 development files
        if ! pkg-config --exists gtk4 2>/dev/null; then
            warnings+=("GTK4 development files (needed for Linux GUI)")
        fi
    fi

    # Check for Android build dependencies
    if [[ "$PLATFORMS" == *"android"* ]] || [ "$PLATFORMS" = "all" ]; then
        log_step "Checking Android build dependencies..."

        if [ "$DOCKER_BUILDS" = true ]; then
            if ! command -v docker &> /dev/null; then
                log_error "Docker is required for Android builds"
                exit 1
            fi

            if ! docker info &> /dev/null; then
                log_error "Docker daemon is not running"
                exit 1
            fi
        else
            # Check for native Android build dependencies
            if [ -z "${ANDROID_NDK_HOME:-}" ] && [ -z "${NDK_HOME:-}" ]; then
                warnings+=("ANDROID_NDK_HOME (needed for native Android builds)")
            fi
        fi
    fi

    # Show warnings
    if [ ${#warnings[@]} -ne 0 ]; then
        log_warning "Missing optional dependencies:"
        for warning in "${warnings[@]}"; do
            log_warning "  - $warning"
        done
        echo
    fi

    log_success "Prerequisites check completed"
}

# Initialize release environment
init_release_environment() {
    log_section "Initializing Release Environment"

    # Clean if requested
    if [ "$CLEAN" = true ]; then
        log_step "Cleaning previous build artifacts..."
        rm -rf "$PROJECT_ROOT/target/release"
        rm -rf "$PROJECT_ROOT/target/debug"
        rm -rf "$PROJECT_ROOT/target/android"
        rm -rf "$PROJECT_ROOT/target/install"
        rm -rf "$RELEASE_DIR"

        # Clean Cargo cache
        cd "$PROJECT_ROOT"
        cargo clean

        log_success "Build artifacts cleaned"
    fi

    # Create release directory structure
    log_step "Creating release directory structure..."
    mkdir -p "$RELEASE_DIR"/{linux/{binaries,packages,install},android/{libraries,headers,integration},docs}

    # Copy documentation
    log_step "Copying documentation..."
    if [ -d "$PROJECT_ROOT/docs" ]; then
        cp -r "$PROJECT_ROOT/docs"/* "$RELEASE_DIR/docs/"
    fi

    # Copy packaging files
    log_step "Copying packaging files..."
    if [ -d "$PROJECT_ROOT/packaging" ]; then
        mkdir -p "$RELEASE_DIR/packaging"
        cp -r "$PROJECT_ROOT/packaging"/* "$RELEASE_DIR/packaging/"

        # Update PKGBUILD version and metadata to match current project version
        local pkgbuild_path="$RELEASE_DIR/packaging/arch/PKGBUILD"
        if [ -f "$pkgbuild_path" ]; then
            log_step "Updating PKGBUILD metadata for version $VERSION..."

            # Update version
            sed -i "s/^pkgver=.*/pkgver=$VERSION/" "$pkgbuild_path"

            # Update source URL to match version
            sed -i "s|^\(source=.*archive/\)v[^/]*\.tar\.gz|\1v$VERSION.tar.gz|" "$pkgbuild_path"

            # Reset pkgrel to 1 for new version
            sed -i "s/^pkgrel=.*/pkgrel=1/" "$pkgbuild_path"

            # Calculate and update checksums if requested
            if [ "$UPDATE_CHECKSUMS" = true ]; then
                log_step "Calculating SHA256 checksum for version $VERSION..."
                local source_url="https://github.com/ejangi/ziplock/archive/v$VERSION.tar.gz"
                local temp_file=$(mktemp)

                if curl -sL "$source_url" -o "$temp_file"; then
                    local new_checksum=$(sha256sum "$temp_file" | cut -d' ' -f1)
                    sed -i "s/^sha256sums=.*/sha256sums=('$new_checksum')/" "$pkgbuild_path"
                    rm -f "$temp_file"
                    log_success "PKGBUILD checksum updated: $new_checksum"
                else
                    rm -f "$temp_file"
                    log_warning "Failed to download source for checksum calculation"
                    if ! grep -q "# NOTE: sha256sums should be updated" "$pkgbuild_path"; then
                        sed -i "/^sha256sums=/i # NOTE: sha256sums should be updated when publishing to AUR" "$pkgbuild_path"
                    fi
                    log_warning "PKGBUILD checksum needs manual update for AUR publication"
                fi
            else
                # Add comment about checksum needing manual update
                if ! grep -q "# NOTE: sha256sums should be updated" "$pkgbuild_path"; then
                    sed -i "/^sha256sums=/i # NOTE: sha256sums should be updated when publishing to AUR" "$pkgbuild_path"
                fi
                log_warning "PKGBUILD checksum (sha256sums) needs manual update for AUR publication"
            fi

            log_success "PKGBUILD metadata updated for version $VERSION"
        fi
    fi

    # Copy important files
    for file in README.md LICENSE.md CHANGELOG.md; do
        if [ -f "$PROJECT_ROOT/$file" ]; then
            cp "$PROJECT_ROOT/$file" "$RELEASE_DIR/"
        fi
    done

    log_success "Release environment initialized"
}

# Build Linux components
build_linux() {
    log_header "Building Linux Components"

    cd "$PROJECT_ROOT"

    # Run Linux build script
    log_step "Building Linux binaries..."
    if [ "$VERBOSE" = true ]; then
        PROFILE=release ./scripts/build/build-linux.sh
    else
        PROFILE=release ./scripts/build/build-linux.sh > /dev/null 2>&1
    fi

    # Copy Linux binaries
    log_step "Copying Linux binaries..."
    if [ -d "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release" ]; then
        cp -r "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release"/* "$RELEASE_DIR/linux/binaries/" 2>/dev/null || true
    elif [ -d "$PROJECT_ROOT/target/release" ]; then
        cp -r "$PROJECT_ROOT/target/release"/* "$RELEASE_DIR/linux/binaries/" 2>/dev/null || true
    fi

    # Copy installation structure
    if [ -d "$PROJECT_ROOT/target/install" ]; then
        log_step "Copying installation structure..."
        cp -r "$PROJECT_ROOT/target/install"/* "$RELEASE_DIR/linux/install/"
    fi

    # Create packages if not skipped
    if [ "$SKIP_PACKAGES" = false ]; then
        log_step "Creating Linux packages..."

        # Create Debian package
        if command -v dpkg-deb &> /dev/null || command -v docker &> /dev/null; then
            if [ "$VERBOSE" = true ]; then
                ./scripts/build/package-deb.sh
            else
                ./scripts/build/package-deb.sh > /dev/null 2>&1
            fi

            # Copy packages
            find "$PROJECT_ROOT/target" -name "*.deb" -type f -exec cp {} "$RELEASE_DIR/linux/packages/" \; 2>/dev/null || true
        fi

        # Create Arch package if available
        if [ -f "./scripts/build/package-arch.sh" ]; then
            if [ "$VERBOSE" = true ]; then
                ./scripts/build/package-arch.sh 2>/dev/null || log_warning "Arch package creation failed"
            else
                ./scripts/build/package-arch.sh > /dev/null 2>&1 || log_warning "Arch package creation failed"
            fi

            # Copy Arch packages
            find "$PROJECT_ROOT/target" -name "*.pkg.tar.*" -type f -exec cp {} "$RELEASE_DIR/linux/packages/" \; 2>/dev/null || true
        fi
    fi

    # Run tests if not skipped
    if [ "$SKIP_TESTS" = false ]; then
        log_step "Running Linux tests..."
        if [ "$VERBOSE" = true ]; then
            cargo test --release
        else
            cargo test --release > /dev/null 2>&1
        fi
    fi

    log_success "Linux build completed"
}

# Build Android components
build_android() {
    log_header "Building Android Components"

    cd "$PROJECT_ROOT"

    if [ "$DOCKER_BUILDS" = true ]; then
        # Use Docker-based Android build
        log_step "Building Android libraries with Docker..."
        if [ "$VERBOSE" = true ]; then
            ./scripts/build/build-android-docker.sh build
        else
            ./scripts/build/build-android-docker.sh build > /dev/null 2>&1
        fi
    else
        # Use native Android build
        log_step "Building Android libraries natively..."
        if [ "$VERBOSE" = true ]; then
            ./scripts/build/build-mobile.sh -p android
        else
            ./scripts/build/build-mobile.sh -p android > /dev/null 2>&1
        fi
    fi

    # Copy Android libraries
    log_step "Organizing Android artifacts..."
    if [ -d "$PROJECT_ROOT/target/android/jniLibs" ]; then
        cp -r "$PROJECT_ROOT/target/android/jniLibs"/* "$RELEASE_DIR/android/libraries/"
    fi

    # Copy header files
    if [ -f "$PROJECT_ROOT/target/android/ziplock.h" ]; then
        cp "$PROJECT_ROOT/target/android/ziplock.h" "$RELEASE_DIR/android/headers/"
    fi

    # Create Android integration files
    log_step "Creating Android integration files..."

    # Create gradle dependency snippet
    cat > "$RELEASE_DIR/android/integration/build.gradle.snippet" << 'EOF'
// Add to your app's build.gradle dependencies section
android {
    // ... other configuration

    packagingOptions {
        pickFirst '**/libziplock_shared.so'
    }
}

dependencies {
    // Copy the libziplock_shared.so files to app/src/main/jniLibs/
    // The libraries will be automatically included in the APK
}
EOF

    # Create JNI loading example
    cat > "$RELEASE_DIR/android/integration/ZipLockJNI.java" << 'EOF'
// Example JNI wrapper for ZipLock Android integration
package com.ziplock;

public class ZipLockJNI {
    static {
        System.loadLibrary("ziplock_shared");
    }

    // Native method declarations - update based on your FFI interface
    public static native long createRepository(String path, String password);
    public static native boolean openRepository(String path, String password);
    public static native boolean saveRepository(long handle);
    public static native void closeRepository(long handle);

    // Add more native methods as needed
}
EOF

    # Run Android tests if not skipped
    if [ "$SKIP_TESTS" = false ]; then
        log_step "Running Android library tests..."
        if [ "$VERBOSE" = true ]; then
            ./scripts/build/test-android-integration.sh basic
        else
            ./scripts/build/test-android-integration.sh basic > /dev/null 2>&1
        fi
    fi

    log_success "Android build completed"
}

# Create release metadata
create_release_metadata() {
    log_section "Creating Release Metadata"

    # Create release info JSON
    local release_info="$RELEASE_DIR/release-info.json"
    cat > "$release_info" << EOF
{
  "version": "$VERSION",
  "build_timestamp": "$BUILD_TIMESTAMP",
  "git_commit": "$GIT_COMMIT",
  "git_branch": "$GIT_BRANCH",
  "platforms": "$PLATFORMS",
  "rust_version": "$(rustc --version)",
  "build_options": {
    "clean_build": $CLEAN,
    "parallel_builds": $PARALLEL_BUILDS,
    "docker_builds": $DOCKER_BUILDS,
    "tests_run": $([ "$SKIP_TESTS" = false ] && echo "true" || echo "false"),
    "packages_created": $([ "$SKIP_PACKAGES" = false ] && echo "true" || echo "false")
  },
  "artifacts": {
    "linux": {
      "binaries": $([ -d "$RELEASE_DIR/linux/binaries" ] && find "$RELEASE_DIR/linux/binaries" -type f | wc -l || echo 0),
      "packages": $([ -d "$RELEASE_DIR/linux/packages" ] && find "$RELEASE_DIR/linux/packages" -type f | wc -l || echo 0)
    },
    "android": {
      "architectures": $([ -d "$RELEASE_DIR/android/libraries" ] && find "$RELEASE_DIR/android/libraries" -name "*.so" | wc -l || echo 0),
      "headers": $([ -d "$RELEASE_DIR/android/headers" ] && find "$RELEASE_DIR/android/headers" -name "*.h" | wc -l || echo 0)
    }
  }
}
EOF

    # Extract changelog for this version
    local release_notes="$RELEASE_DIR/RELEASE_NOTES.md"
    echo "# ZipLock $VERSION Release Notes" > "$release_notes"
    echo >> "$release_notes"
    echo "**Build Date:** $BUILD_TIMESTAMP" >> "$release_notes"
    echo "**Git Commit:** $GIT_COMMIT" >> "$release_notes"
    echo "**Git Branch:** $GIT_BRANCH" >> "$release_notes"
    echo >> "$release_notes"

    # Try to extract changelog section
    if [ -f "$PROJECT_ROOT/CHANGELOG.md" ]; then
        log_step "Extracting changelog for version $VERSION..."

        # Look for version section in changelog
        local version_section
        version_section=$(awk "/^## \[?$VERSION\]?/,/^## \[?[0-9]/ { if (/^## \[?[0-9]/ && !/^## \[?$VERSION\]?/) exit; print }" "$PROJECT_ROOT/CHANGELOG.md" 2>/dev/null || echo "")

        if [ -n "$version_section" ]; then
            echo "$version_section" >> "$release_notes"
        else
            echo "## Changes" >> "$release_notes"
            echo >> "$release_notes"
            echo "Please see CHANGELOG.md for detailed changes." >> "$release_notes"
        fi
    else
        echo "## Changes" >> "$release_notes"
        echo >> "$release_notes"
        echo "Changelog not available." >> "$release_notes"
    fi

    echo >> "$release_notes"
    echo "## Artifacts Included" >> "$release_notes"
    echo >> "$release_notes"

    if [[ "$PLATFORMS" == *"linux"* ]] || [ "$PLATFORMS" = "all" ]; then
        echo "### Linux" >> "$release_notes"
        echo "- Desktop application binaries" >> "$release_notes"
        echo "- Debian packages (.deb)" >> "$release_notes"
        echo "- Arch Linux PKGBUILD and install scripts (version updated to $VERSION)" >> "$release_notes"
        echo "- Installation structure" >> "$release_notes"
        echo >> "$release_notes"
    fi

    if [[ "$PLATFORMS" == *"android"* ]] || [ "$PLATFORMS" = "all" ]; then
        echo "### Android" >> "$release_notes"
        echo "- Native libraries (.so) for all architectures" >> "$release_notes"
        echo "- C header files for FFI integration" >> "$release_notes"
        echo "- Integration examples and documentation" >> "$release_notes"
        echo >> "$release_notes"
    fi

    echo "## Installation" >> "$release_notes"
    echo >> "$release_notes"
    echo "### Linux" >> "$release_notes"
    echo "- **Debian/Ubuntu**: Install the .deb package from \`linux/packages/\`" >> "$release_notes"
    echo "- **Arch Linux**: Use the PKGBUILD from \`packaging/arch/\` (run \`makepkg -si\`)" >> "$release_notes"
    echo "  - Note: PKGBUILD version automatically updated to $VERSION" >> "$release_notes"
    echo "  - For AUR: Update sha256sums manually after building" >> "$release_notes"
    echo "- **Manual**: Use files from \`linux/install/\` directory" >> "$release_notes"
    echo >> "$release_notes"
    echo "### Android" >> "$release_notes"
    echo "- Copy libraries from \`android/libraries/\` to your app's \`jniLibs/\`" >> "$release_notes"
    echo "- Use header files from \`android/headers/\` for FFI integration" >> "$release_notes"
    echo "- See integration examples in \`android/integration/\`" >> "$release_notes"
    echo >> "$release_notes"
    echo "See the documentation in the \`docs/\` directory for detailed platform-specific instructions." >> "$release_notes"

    log_success "Release metadata created"
}

# Create release archive
create_release_archive() {
    log_section "Creating Release Archive"

    cd "$PROJECT_ROOT/target"

    local archive_name="ziplock-$VERSION-unified-release.tar.gz"
    log_step "Creating archive: $archive_name"

    tar -czf "$archive_name" -C unified-release .

    local archive_size
    archive_size=$(du -h "$archive_name" | cut -f1)

    log_success "Release archive created: $archive_name ($archive_size)"

    # Create checksums
    log_step "Creating checksums..."
    sha256sum "$archive_name" > "${archive_name}.sha256"
    md5sum "$archive_name" > "${archive_name}.md5"

    log_info "Archive location: $PROJECT_ROOT/target/$archive_name"
}

# Print build summary
print_build_summary() {
    log_header "Build Summary"

    echo -e "${CYAN}Version:${NC} $VERSION"
    echo -e "${CYAN}Platforms:${NC} $PLATFORMS"
    echo -e "${CYAN}Build Time:${NC} $BUILD_TIMESTAMP"
    echo -e "${CYAN}Git Commit:${NC} $GIT_COMMIT"
    echo -e "${CYAN}Git Branch:${NC} $GIT_BRANCH"
    echo

    # Show artifact counts
    if [[ "$PLATFORMS" == *"linux"* ]] || [ "$PLATFORMS" = "all" ]; then
        local linux_binaries=0
        local linux_packages=0

        [ -d "$RELEASE_DIR/linux/binaries" ] && linux_binaries=$(find "$RELEASE_DIR/linux/binaries" -type f | wc -l)
        [ -d "$RELEASE_DIR/linux/packages" ] && linux_packages=$(find "$RELEASE_DIR/linux/packages" -type f | wc -l)

        echo -e "${CYAN}Linux Artifacts:${NC}"
        echo "  - Binaries: $linux_binaries"
        echo "  - Packages: $linux_packages"
    fi

    if [[ "$PLATFORMS" == *"android"* ]] || [ "$PLATFORMS" = "all" ]; then
        local android_libs=0
        local android_headers=0

        [ -d "$RELEASE_DIR/android/libraries" ] && android_libs=$(find "$RELEASE_DIR/android/libraries" -name "*.so" | wc -l)
        [ -d "$RELEASE_DIR/android/headers" ] && android_headers=$(find "$RELEASE_DIR/android/headers" -name "*.h" | wc -l)

        echo -e "${CYAN}Android Artifacts:${NC}"
        echo "  - Libraries: $android_libs"
        echo "  - Headers: $android_headers"
    fi

    echo
    echo -e "${CYAN}Release Directory:${NC} $RELEASE_DIR"

    # Show directory structure
    echo
    echo -e "${CYAN}Directory Structure:${NC}"
    tree "$RELEASE_DIR" 2>/dev/null || find "$RELEASE_DIR" -type d | sed 's/^/  /'
}

# Main build orchestrator
main() {
    local start_time
    start_time=$(date +%s)

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--platforms)
                PLATFORMS="$2"
                shift 2
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-packages)
                SKIP_PACKAGES=true
                shift
                ;;
            --no-parallel)
                PARALLEL_BUILDS=false
                shift
                ;;
            --no-docker)
                DOCKER_BUILDS=false
                shift
                ;;
            --update-checksums)
                UPDATE_CHECKSUMS=true
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            -o|--output)
                RELEASE_DIR="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Set default platform
    if [ -z "$PLATFORMS" ]; then
        PLATFORMS="all"
    fi

    # Validate platforms
    case "$PLATFORMS" in
        "linux"|"android"|"all")
            ;;
        *)
            log_error "Invalid platforms: $PLATFORMS"
            log_error "Valid options: linux, android, all"
            exit 1
            ;;
    esac

    # Enable verbose mode if requested
    if [ "$VERBOSE" = true ]; then
        set -x
    fi

    # Start build process
    log_header "ZipLock Unified Release Build"
    log_info "Building ZipLock $VERSION for platforms: $PLATFORMS"

    check_prerequisites
    init_release_environment

    # Build platforms
    if [[ "$PLATFORMS" == *"linux"* ]] || [ "$PLATFORMS" = "all" ]; then
        build_linux
    fi

    if [[ "$PLATFORMS" == *"android"* ]] || [ "$PLATFORMS" = "all" ]; then
        build_android
    fi

    create_release_metadata
    create_release_archive

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    print_build_summary

    log_header "Build Completed Successfully!"
    log_success "Total build time: ${duration}s"
    log_info "Release ready for distribution: $PROJECT_ROOT/target/ziplock-$VERSION-unified-release.tar.gz"
}

# Run main function
main "$@"
