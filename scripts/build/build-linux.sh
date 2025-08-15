#!/bin/bash
set -euo pipefail

# ZipLock Linux Build Script
# Builds unified ZipLock application with FFI shared library for Linux distribution

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
PACKAGING_DIR="$PROJECT_ROOT/packaging/linux"

# Build configuration
RUST_TARGET_DIR="$BUILD_DIR"
PROFILE="${PROFILE:-release}"
TARGET_ARCH="${TARGET_ARCH:-x86_64-unknown-linux-gnu}"
VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')}"
CARGO_CMD="cargo"  # Will be set to correct path in check_dependencies

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
    log_info "Checking build dependencies..."

    # Check for Rust - try multiple locations
    if command -v cargo &> /dev/null; then
        CARGO_CMD="cargo"
    elif [ -f "/root/.cargo/bin/cargo" ]; then
        CARGO_CMD="/root/.cargo/bin/cargo"
        export PATH="/root/.cargo/bin:$PATH"
    elif [ -f "$HOME/.cargo/bin/cargo" ]; then
        CARGO_CMD="$HOME/.cargo/bin/cargo"
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        log_error "Rust/Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi

    log_info "Using cargo at: $(which cargo 2>/dev/null || echo $CARGO_CMD)"

    # Check for required system dependencies
    local missing_deps=()

    # GUI dependencies for iced
    if ! pkg-config --exists fontconfig; then
        missing_deps+=("libfontconfig1-dev")
    fi

    if ! pkg-config --exists freetype2; then
        missing_deps+=("libfreetype6-dev")
    fi

    if ! pkg-config --exists x11; then
        missing_deps+=("libx11-dev")
    fi

    if ! pkg-config --exists xft; then
        missing_deps+=("libxft-dev")
    fi

    # Check for 7z support
    if ! pkg-config --exists liblzma; then
        missing_deps+=("liblzma-dev")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing system dependencies: ${missing_deps[*]}"
        log_info "Install with: sudo apt-get install ${missing_deps[*]}"
        exit 1
    fi

    log_success "All dependencies satisfied"
}

setup_build_environment() {
    log_info "Setting up build environment..."

    cd "$PROJECT_ROOT"

    # Set Rust environment variables
    export CARGO_TARGET_DIR="$RUST_TARGET_DIR"
    export RUST_BACKTRACE=1

    # Create directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$PACKAGING_DIR"

    log_success "Build environment ready"
}

build_shared_library() {
    log_info "Building shared library with C API..."

    cd "$PROJECT_ROOT"

    # Map profile name to actual directory name
    local cargo_profile="$PROFILE"
    if [ "$PROFILE" = "dev" ]; then
        cargo_profile="debug"
    fi

    local shared_lib_dir="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile"
    local shared_lib_so="$shared_lib_dir/libziplock_shared.so"
    local shared_lib_dylib="$shared_lib_dir/libziplock_shared.dylib"

    # First attempt to build shared library
    log_info "First attempt: Building shared library..."
    $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-shared --features c-api

    # Verify the shared library was created
    if [ -f "$shared_lib_so" ] || [ -f "$shared_lib_dylib" ]; then
        log_success "Shared library built successfully on first attempt"
        if [ -f "$shared_lib_so" ]; then
            log_info "Shared library (.so): $(ls -la "$shared_lib_so")"
        fi
        if [ -f "$shared_lib_dylib" ]; then
            log_info "Shared library (.dylib): $(ls -la "$shared_lib_dylib")"
        fi
        return 0
    fi

    # If first attempt failed, try with clean build
    log_warning "Shared library not found after first attempt, trying clean build..."
    $CARGO_CMD clean -p ziplock-shared
    $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-shared --features c-api --verbose

    # Second verification
    if [ -f "$shared_lib_so" ] || [ -f "$shared_lib_dylib" ]; then
        log_success "Shared library built successfully on second attempt"
        if [ -f "$shared_lib_so" ]; then
            log_info "Shared library (.so): $(ls -la "$shared_lib_so")"
        fi
        if [ -f "$shared_lib_dylib" ]; then
            log_info "Shared library (.dylib): $(ls -la "$shared_lib_dylib")"
        fi
        return 0
    fi

    # Final attempt with explicit cdylib build
    log_warning "Shared library still not found, trying explicit cdylib build..."
    RUSTFLAGS="--cfg cdylib" $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-shared --features c-api --verbose

    # Final verification
    if [ -f "$shared_lib_so" ] || [ -f "$shared_lib_dylib" ]; then
        log_success "Shared library built successfully on final attempt"
        if [ -f "$shared_lib_so" ]; then
            log_info "Shared library (.so): $(ls -la "$shared_lib_so")"
        fi
        if [ -f "$shared_lib_dylib" ]; then
            log_info "Shared library (.dylib): $(ls -la "$shared_lib_dylib")"
        fi
        return 0
    fi

    # If all attempts failed, show debug information
    log_error "Shared library not found after all attempts at: $shared_lib_dir"
    log_info "Available files in target directory:"
    find "$RUST_TARGET_DIR" -name "libziplock*" -type f 2>/dev/null || log_warning "No libziplock files found"
    log_info "Available .so files in target directory:"
    find "$RUST_TARGET_DIR" -name "*.so" -type f 2>/dev/null | head -10 || log_warning "No .so files found"
    log_info "Cargo target directory contents:"
    ls -la "$shared_lib_dir/" 2>/dev/null || log_warning "Target directory not accessible"
    exit 1
}

build_unified_application() {
    log_info "Building unified ZipLock application..."

    cd "$PROJECT_ROOT"
    $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog"

    # Map profile name to actual directory name
    local cargo_profile="$PROFILE"
    if [ "$PROFILE" = "dev" ]; then
        cargo_profile="debug"
    fi

    local app_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile/ziplock"
    if [ ! -f "$app_binary" ]; then
        log_error "ZipLock application binary not found at: $app_binary"
        exit 1
    fi

    log_success "Unified ZipLock application built successfully"
}

run_tests() {
    log_info "Running tests..."

    cd "$PROJECT_ROOT"
    # Test shared library with FFI features
    $CARGO_CMD test --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-shared --features c-api

    # Test unified application with FFI client
    $CARGO_CMD test --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog"

    log_success "All tests passed"
}

strip_binaries() {
    if [ "$PROFILE" = "release" ]; then
        log_info "Stripping debug symbols from binaries..."

        # Map profile name to actual directory name
        local cargo_profile="$PROFILE"
        if [ "$PROFILE" = "dev" ]; then
            cargo_profile="debug"
        fi

        local app_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile/ziplock"
        local shared_lib_dir="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile"

        strip "$app_binary" || log_warning "Failed to strip application binary"

        # Strip shared library if it exists
        if [ -f "$shared_lib_dir/libziplock_shared.so" ]; then
            strip "$shared_lib_dir/libziplock_shared.so" || log_warning "Failed to strip shared library (.so)"
        fi
        if [ -f "$shared_lib_dir/libziplock_shared.dylib" ]; then
            strip "$shared_lib_dir/libziplock_shared.dylib" || log_warning "Failed to strip shared library (.dylib)"
        fi

        log_success "Binaries stripped"
    fi
}

verify_binaries() {
    log_info "Verifying built binaries..."

    # Map profile name to actual directory name
    local cargo_profile="$PROFILE"
    if [ "$PROFILE" = "dev" ]; then
        cargo_profile="debug"
    fi

    local app_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile/ziplock"
    local shared_lib_dir="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile"

    # Check if application binary exists and is executable
    if [ ! -x "$app_binary" ]; then
        log_error "ZipLock application binary is not executable: $app_binary"
        exit 1
    fi

    # Check shared library exists
    if [ ! -f "$shared_lib_dir/libziplock_shared.so" ] && [ ! -f "$shared_lib_dir/libziplock_shared.dylib" ]; then
        log_error "Shared library not found in: $shared_lib_dir"
        exit 1
    fi

    # Application is a GUI and cannot be run without a display
    # so we skip the version check and just verify it's executable
    log_info "ZipLock application verified (GUI application - version check skipped)"

    local app_size=$(du -h "$app_binary" | cut -f1)
    log_info "Application size: $app_size"

    # Check shared library size
    if [ -f "$shared_lib_dir/libziplock_shared.so" ]; then
        local lib_size=$(du -h "$shared_lib_dir/libziplock_shared.so" | cut -f1)
        log_info "Shared library size (.so): $lib_size"
    fi
    if [ -f "$shared_lib_dir/libziplock_shared.dylib" ]; then
        local lib_size=$(du -h "$shared_lib_dir/libziplock_shared.dylib" | cut -f1)
        log_info "Shared library size (.dylib): $lib_size"
    fi

    log_success "Binary verification completed"
}

create_install_structure() {
    log_info "Creating installation structure..."

    local install_dir="$BUILD_DIR/install"
    rm -rf "$install_dir"
    mkdir -p "$install_dir"

    # Create directory structure
    mkdir -p "$install_dir/usr/bin"
    mkdir -p "$install_dir/usr/lib"
    mkdir -p "$install_dir/etc/ziplock"
    mkdir -p "$install_dir/usr/share/applications"
    mkdir -p "$install_dir/usr/share/icons/hicolor/scalable/apps"

    # Map profile name to actual directory name
    local cargo_profile="$PROFILE"
    if [ "$PROFILE" = "dev" ]; then
        cargo_profile="debug"
    fi

    # Copy application binary
    local app_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile/ziplock"
    if [ -f "$app_binary" ]; then
        cp "$app_binary" "$install_dir/usr/bin/"
    else
        log_error "ZipLock application binary not found: $app_binary"
        exit 1
    fi

    # Copy shared library with robust verification
    local shared_lib_dir="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile"
    local shared_lib_copied=false

    log_info "Copying shared library from: $shared_lib_dir"

    if [ -f "$shared_lib_dir/libziplock_shared.so" ]; then
        log_info "Found .so shared library, copying..."
        cp "$shared_lib_dir/libziplock_shared.so" "$install_dir/usr/lib/"
        log_info "Copied shared library: $(ls -la "$install_dir/usr/lib/libziplock_shared.so")"
        shared_lib_copied=true
    elif [ -f "$shared_lib_dir/libziplock_shared.dylib" ]; then
        log_info "Found .dylib shared library, copying..."
        cp "$shared_lib_dir/libziplock_shared.dylib" "$install_dir/usr/lib/"
        log_info "Copied shared library: $(ls -la "$install_dir/usr/lib/libziplock_shared.dylib")"
        shared_lib_copied=true
    fi

    if [ "$shared_lib_copied" = false ]; then
        log_error "Shared library not found in: $shared_lib_dir"
        log_info "Available files in shared library directory:"
        ls -la "$shared_lib_dir/" 2>/dev/null || log_warning "Directory not accessible"
        log_info "Searching for shared library files:"
        find "$RUST_TARGET_DIR" -name "libziplock_shared.*" -type f 2>/dev/null || log_warning "No shared library files found"
        exit 1
    fi

    # Verify the copied shared library
    if [ -f "$install_dir/usr/lib/libziplock_shared.so" ]; then
        log_success "Shared library successfully installed: $(ls -la "$install_dir/usr/lib/libziplock_shared.so")"
    elif [ -f "$install_dir/usr/lib/libziplock_shared.dylib" ]; then
        log_success "Shared library successfully installed: $(ls -la "$install_dir/usr/lib/libziplock_shared.dylib")"
    else
        log_error "Shared library was not properly copied to install directory"
        exit 1
    fi

    # Copy desktop file and icon
    if [ -f "$PROJECT_ROOT/apps/linux/resources/ziplock.desktop" ]; then
        cp "$PROJECT_ROOT/apps/linux/resources/ziplock.desktop" "$install_dir/usr/share/applications/"
    else
        log_warning "Desktop file not found"
    fi

    # Copy icon
    if [ -f "$PROJECT_ROOT/apps/linux/resources/icons/ziplock.svg" ]; then
        cp "$PROJECT_ROOT/apps/linux/resources/icons/ziplock.svg" "$install_dir/usr/share/icons/hicolor/scalable/apps/"
    elif [ -f "$PROJECT_ROOT/assets/icons/ziplock-logo.svg" ]; then
        cp "$PROJECT_ROOT/assets/icons/ziplock-logo.svg" "$install_dir/usr/share/icons/hicolor/scalable/apps/ziplock.svg"
    else
        log_warning "No application icon found"
    fi

    # Create default config for unified application
    cat > "$install_dir/etc/ziplock/config.yml" << EOF
# ZipLock Configuration (Unified FFI Architecture)
storage:
  backup_count: 3
  auto_backup: true
  compression:
    level: 6
    solid: false
    multi_threaded: true

security:
  auto_lock_timeout: 900  # 15 minutes
  min_master_key_length: 12
  enforce_strong_master_key: true

ui:
  theme: "auto"  # auto, light, dark
  font_size: 14
  show_password_strength: true

logging:
  level: "info"  # debug, info, warn, error
  file: null     # null for console only, or path to log file
EOF

    log_success "Installation structure created at: $install_dir"
}

display_build_summary() {
    log_success "Build completed successfully!"
    echo
    echo "Build Summary (Unified FFI Architecture):"
    echo "========================================="
    echo "Profile: $PROFILE"
    echo "Target: $TARGET_ARCH"
    echo "Version: $VERSION"
    echo

    # Map profile name to actual directory name
    local cargo_profile="$PROFILE"
    if [ "$PROFILE" = "dev" ]; then
        cargo_profile="debug"
    fi

    echo "Binaries:"
    echo "  Application: $RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile/ziplock"
    local shared_lib_dir="$RUST_TARGET_DIR/$TARGET_ARCH/$cargo_profile"
    if [ -f "$shared_lib_dir/libziplock_shared.so" ]; then
        echo "  Shared Library: $shared_lib_dir/libziplock_shared.so"
    elif [ -f "$shared_lib_dir/libziplock_shared.dylib" ]; then
        echo "  Shared Library: $shared_lib_dir/libziplock_shared.dylib"
    fi
    echo
    echo "Installation structure: $BUILD_DIR/install"
    echo
    echo "Next steps:"
    echo "  1. Run './scripts/build/package-deb.sh' to create .deb package"
    echo "  2. Or manually install with 'sudo cp -r $BUILD_DIR/install/* /'"
    echo "  3. Run with: LD_LIBRARY_PATH=/usr/lib ziplock"
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --profile PROFILE     Build profile (dev|release) [default: release]"
    echo "  --target TARGET       Target architecture [default: x86_64-unknown-linux-gnu]"
    echo "  --skip-tests         Skip running tests"
    echo "  --no-strip           Don't strip debug symbols"
    echo "  --clean              Clean build directory first"
    echo "  --help               Show this help message"
    echo
    echo "Environment Variables:"
    echo "  VERSION              Override version string"
    echo "  CARGO_TARGET_DIR     Override cargo target directory"
}

main() {
    local skip_tests=false
    local no_strip=false
    local clean=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --profile)
                PROFILE="$2"
                shift 2
                ;;
            --target)
                TARGET_ARCH="$2"
                shift 2
                ;;

            --skip-tests)
                skip_tests=true
                shift
                ;;
            --no-strip)
                no_strip=true
                shift
                ;;
            --clean)
                clean=true
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

    # Validate profile
    if [[ "$PROFILE" != "dev" && "$PROFILE" != "release" ]]; then
        log_error "Invalid profile: $PROFILE. Must be 'dev' or 'release'"
        exit 1
    fi

    log_info "Starting ZipLock Linux build..."
    log_info "Profile: $PROFILE, Target: $TARGET_ARCH, Version: $VERSION"

    # Clean if requested
    if [ "$clean" = true ]; then
        log_info "Cleaning build directory..."
        rm -rf "$BUILD_DIR"
    fi

    # Run build steps
    check_dependencies
    setup_build_environment
    build_shared_library
    build_unified_application

    if [ "$skip_tests" = false ]; then
        run_tests
    fi

    if [ "$no_strip" = false ]; then
        strip_binaries
    fi

    verify_binaries
    create_install_structure
    display_build_summary
}

# Run main function with all arguments
main "$@"
