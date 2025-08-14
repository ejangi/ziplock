#!/bin/bash
set -euo pipefail

# ZipLock Arch Linux Package Creation Script
# Creates .pkg.tar.xz packages for Arch Linux

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
PACKAGING_DIR="$PROJECT_ROOT/packaging/arch"

# Package configuration
PACKAGE_NAME="ziplock"
PACKAGE_VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')}"
PACKAGE_ARCH="${PACKAGE_ARCH:-x86_64}"
MAINTAINER="James Angus <james@ejangi.com>"
DESCRIPTION="A secure, portable password manager using encrypted 7z archives"
HOMEPAGE="https://github.com/ejangi/ziplock"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

check_dependencies() {
    log_info "Checking Arch packaging dependencies..."

    local missing_deps=()

    if ! command -v makepkg &> /dev/null; then
        missing_deps+=("base-devel")
    fi

    if ! command -v fakeroot &> /dev/null; then
        missing_deps+=("fakeroot")
    fi

    if ! command -v pacman &> /dev/null; then
        log_error "This script must be run on Arch Linux or an Arch-based distribution"
        exit 1
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing packaging dependencies: ${missing_deps[*]}"
        log_info "Install with: sudo pacman -S ${missing_deps[*]}"
        exit 1
    fi

    log_success "Packaging dependencies satisfied"
}

verify_build() {
    log_info "Verifying build artifacts..."

    local install_dir="$BUILD_DIR/install"

    if [ ! -d "$install_dir" ]; then
        log_error "Installation directory not found: $install_dir"
        log_info "Run './scripts/build/build-linux.sh' first"
        exit 1
    fi

    # Check required files
    local required_files=(
        "$install_dir/usr/bin/ziplock-backend"
        "$install_dir/usr/bin/ziplock"
        "$install_dir/lib/systemd/system/ziplock-backend.service"
        "$install_dir/etc/ziplock/config.yml"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Required file missing: $file"
            exit 1
        fi
    done

    log_success "Build artifacts verified"
}

create_source_archive() {
    log_info "Creating source archive for package..."

    local src_dir="$BUILD_DIR/src"
    local archive_name="${PACKAGE_NAME}-${PACKAGE_VERSION}.tar.gz"
    local archive_path="$BUILD_DIR/$archive_name"

    # Clean and create source directory
    rm -rf "$src_dir"
    mkdir -p "$src_dir"

    # Copy project files (excluding build artifacts and git)
    rsync -av \
        --exclude='.git/' \
        --exclude='target/' \
        --exclude='test-results/' \
        --exclude='*.deb' \
        --exclude='*.pkg.tar.*' \
        --exclude='.DS_Store' \
        --exclude='node_modules/' \
        "$PROJECT_ROOT/" "$src_dir/${PACKAGE_NAME}-${PACKAGE_VERSION}/"

    # Create archive
    cd "$src_dir"
    tar -czf "$archive_path" "${PACKAGE_NAME}-${PACKAGE_VERSION}/"
    cd "$PROJECT_ROOT"

    # Calculate SHA256
    local sha256sum=$(sha256sum "$archive_path" | cut -d' ' -f1)
    log_info "Archive SHA256: $sha256sum"

    # Store for later use
    echo "$sha256sum" > "$BUILD_DIR/archive.sha256"

    log_success "Source archive created: $archive_path"
}

create_pkgbuild() {
    local pkg_dir="$BUILD_DIR/pkg"
    local sha256sum=$(cat "$BUILD_DIR/archive.sha256" 2>/dev/null || echo "SKIP")

    log_info "Creating PKGBUILD for package build..."

    # Clean and create package directory
    rm -rf "$pkg_dir"
    mkdir -p "$pkg_dir"

    # Copy PKGBUILD and install script
    cp "$PACKAGING_DIR/PKGBUILD" "$pkg_dir/"
    cp "$PACKAGING_DIR/ziplock.install" "$pkg_dir/"

    # Update PKGBUILD with current version and checksum
    sed -i "s/^pkgver=.*/pkgver=$PACKAGE_VERSION/" "$pkg_dir/PKGBUILD"
    sed -i "s/^sha256sums=.*/sha256sums=('$sha256sum')/" "$pkg_dir/PKGBUILD"

    log_success "PKGBUILD created in: $pkg_dir"
}

build_package() {
    local pkg_dir="$BUILD_DIR/pkg"
    local archive_name="${PACKAGE_NAME}-${PACKAGE_VERSION}.tar.gz"

    log_info "Building Arch package..."

    # Copy source archive to package directory
    cp "$BUILD_DIR/$archive_name" "$pkg_dir/"

    cd "$pkg_dir"

    # Set build environment
    export PKGDEST="$BUILD_DIR"
    export SRCDEST="$pkg_dir"
    export LOGDEST="$BUILD_DIR/logs"
    export BUILDDIR="$BUILD_DIR/makepkg"

    # Create log directory
    mkdir -p "$LOGDEST"

    # Build package with makepkg
    local makepkg_output
    if makepkg_output=$(makepkg --force --clean --syncdeps --noconfirm 2>&1); then
        log_info "Package build completed successfully"
    else
        local exit_code=$?
        log_error "Package build failed with exit code: $exit_code"
        log_error "Build output: $makepkg_output"
        exit 1
    fi

    cd "$PROJECT_ROOT"

    # Find the created package
    local package_file=$(find "$BUILD_DIR" -name "${PACKAGE_NAME}-${PACKAGE_VERSION}-*.pkg.tar.*" -type f | head -n1)

    if [ ! -f "$package_file" ]; then
        log_error "Package file was not created"
        exit 1
    fi

    log_success "Arch package created: $package_file"

    # Verify package
    log_info "Verifying package..."
    if ! pacman -Qip "$package_file" 2>&1; then
        log_error "Package verification failed"
        exit 1
    fi

    # Show package contents (first 20 files)
    log_info "Package contents (first 20 files):"
    local package_contents=$(pacman -Qlp "$package_file" 2>/dev/null | sed -n '1,20p') || package_contents="Could not list package contents"
    echo "$package_contents"

    local package_size=$(du -h "$package_file" | cut -f1)
    log_success "Package verification completed - Size: $package_size"

    return 0
}

test_package_installation() {
    log_info "Testing package installation (dry run)..."

    local package_file=$(find "$BUILD_DIR" -name "${PACKAGE_NAME}-${PACKAGE_VERSION}-*.pkg.tar.*" -type f | head -n1)

    if [ ! -f "$package_file" ]; then
        log_error "Package file not found for testing"
        return 1
    fi

    # Check if package can be read
    if ! pacman -Qip "$package_file" >/dev/null 2>&1; then
        log_error "Package is corrupted or invalid"
        return 1
    fi

    # Check package dependencies
    log_info "Checking package dependencies..."
    local deps_output
    if deps_output=$(pacman -Qip "$package_file" | grep "Depends On" 2>/dev/null); then
        echo "$deps_output"
    fi

    log_success "Package installation test passed"
}

print_package_info() {
    local package_file=$(find "$BUILD_DIR" -name "${PACKAGE_NAME}-${PACKAGE_VERSION}-*.pkg.tar.*" -type f | head -n1)

    echo
    log_success "Arch Linux package created successfully!"
    echo
    echo "Package Information:"
    echo "==================="
    echo "Name: $PACKAGE_NAME"
    echo "Version: $PACKAGE_VERSION"
    echo "Architecture: $PACKAGE_ARCH"
    echo "File: $package_file"
    echo "Size: $(du -h "$package_file" | cut -f1)"
    echo
    echo "Installation:"
    echo "  sudo pacman -U $package_file"
    echo
    echo "Testing in clean environment:"
    echo "  # Test in Arch Linux container"
    echo "  docker run -it --rm archlinux:latest bash"
    echo "  # Copy .pkg.tar.xz file and install"
    echo
    echo "Publishing to AUR:"
    echo "  1. Fork the AUR package repository"
    echo "  2. Update PKGBUILD with new version and checksum"
    echo "  3. Test build with 'makepkg -si'"
    echo "  4. Submit to AUR"
    echo
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --arch ARCH          Package architecture [default: x86_64]"
    echo "  --version VERSION    Override package version"
    echo "  --skip-tests         Skip package installation tests"
    echo "  --source-only        Only create source archive (for AUR)"
    echo "  --help              Show this help message"
    echo
    echo "Environment Variables:"
    echo "  PACKAGE_ARCH        Override package architecture"
    echo "  VERSION            Override version string"
}

main() {
    local skip_tests=false
    local source_only=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --arch)
                PACKAGE_ARCH="$2"
                shift 2
                ;;
            --version)
                PACKAGE_VERSION="$2"
                shift 2
                ;;
            --skip-tests)
                skip_tests=true
                shift
                ;;
            --source-only)
                source_only=true
                shift
                ;;
            --help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done

    log_info "Starting Arch Linux package creation..."
    log_info "Package: $PACKAGE_NAME v$PACKAGE_VERSION ($PACKAGE_ARCH)"

    check_dependencies

    if [ "$source_only" = true ]; then
        log_info "Creating source-only package for AUR..."
        create_source_archive
        log_success "Source archive created for AUR submission"
        echo "Archive location: $BUILD_DIR/${PACKAGE_NAME}-${PACKAGE_VERSION}.tar.gz"
        echo "SHA256: $(cat "$BUILD_DIR/archive.sha256")"
        exit 0
    fi

    verify_build
    create_source_archive
    create_pkgbuild
    build_package

    if [ "$skip_tests" = false ]; then
        test_package_installation
    fi

    print_package_info
}

# Run main function with all arguments
main "$@"
