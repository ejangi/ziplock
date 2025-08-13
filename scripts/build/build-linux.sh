#!/bin/bash
set -euo pipefail

# ZipLock Linux Build Script
# Builds both backend and frontend binaries for Linux distribution

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
    log_info "Building shared library..."

    cd "$PROJECT_ROOT"
    $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-shared

    log_success "Shared library built successfully"
}

build_backend() {
    log_info "Building backend service..."

    cd "$PROJECT_ROOT"
    $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-backend

    local backend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock-backend"
    if [ ! -f "$backend_binary" ]; then
        log_error "Backend binary not found at: $backend_binary"
        exit 1
    fi

    log_success "Backend service built successfully"
}

build_frontend() {
    log_info "Building frontend client..."

    cd "$PROJECT_ROOT"
    $CARGO_CMD build --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog"

    local frontend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock"
    if [ ! -f "$frontend_binary" ]; then
        log_error "Frontend binary not found at: $frontend_binary"
        exit 1
    fi

    log_success "Frontend client built successfully"
}

run_tests() {
    log_info "Running tests..."

    cd "$PROJECT_ROOT"
    # Test backend and shared libraries
    $CARGO_CMD test --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-backend
    $CARGO_CMD test --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-shared

    # Test frontend with iced-gui features only
    $CARGO_CMD test --profile "$PROFILE" --target "$TARGET_ARCH" -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog"

    log_success "All tests passed"
}

strip_binaries() {
    if [ "$PROFILE" = "release" ]; then
        log_info "Stripping debug symbols from binaries..."

        local backend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock-backend"
        local frontend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock"

        strip "$backend_binary" || log_warning "Failed to strip backend binary"
        strip "$frontend_binary" || log_warning "Failed to strip frontend binary"

        log_success "Binaries stripped"
    fi
}

verify_binaries() {
    log_info "Verifying built binaries..."

    local backend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock-backend"

    # Check if backend binary exists and is executable
    if [ ! -x "$backend_binary" ]; then
        log_error "Backend binary is not executable: $backend_binary"
        exit 1
    fi

    # Quick version check for backend
    log_info "Backend version: $("$backend_binary" --version 2>/dev/null || echo "Version check failed")"

    # Check backend binary size
    local backend_size=$(du -h "$backend_binary" | cut -f1)
    log_info "Binary sizes - Backend: $backend_size"

    # Check frontend binary
    local frontend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock"

    if [ ! -x "$frontend_binary" ]; then
        log_error "Frontend binary is not executable: $frontend_binary"
        exit 1
    fi

    # Frontend is a GUI application and cannot be run without a display
    # so we skip the version check and just verify it's executable
    log_info "Frontend binary verified (GUI application - version check skipped)"

    local frontend_size=$(du -h "$frontend_binary" | cut -f1)
    log_info "Frontend size: $frontend_size"

    log_success "Binary verification completed"
}

create_install_structure() {
    log_info "Creating installation structure..."

    local install_dir="$BUILD_DIR/install"
    rm -rf "$install_dir"
    mkdir -p "$install_dir"

    # Create directory structure
    mkdir -p "$install_dir/usr/bin"
    mkdir -p "$install_dir/lib/systemd/system"
    mkdir -p "$install_dir/etc/ziplock"
    mkdir -p "$install_dir/var/lib/ziplock"

    # Create GUI directories
    mkdir -p "$install_dir/usr/share/applications"
    mkdir -p "$install_dir/usr/share/icons/hicolor/scalable/apps"

    # Copy binaries
    local backend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock-backend"
    if [ -f "$backend_binary" ]; then
        cp "$backend_binary" "$install_dir/usr/bin/"
    else
        log_error "Backend binary not found: $backend_binary"
        exit 1
    fi

    local frontend_binary="$RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock"
    if [ -f "$frontend_binary" ]; then
        cp "$frontend_binary" "$install_dir/usr/bin/"
    else
        log_error "Frontend binary not found: $frontend_binary"
        exit 1
    fi

    # Copy desktop file and icon
    if [ -f "$PROJECT_ROOT/frontend/linux/resources/ziplock.desktop" ]; then
        cp "$PROJECT_ROOT/frontend/linux/resources/ziplock.desktop" "$install_dir/usr/share/applications/"
    else
        log_warning "Desktop file not found"
    fi

    # Copy icon
    if [ -f "$PROJECT_ROOT/frontend/linux/resources/icons/ziplock.svg" ]; then
        cp "$PROJECT_ROOT/frontend/linux/resources/icons/ziplock.svg" "$install_dir/usr/share/icons/hicolor/scalable/apps/"
    elif [ -f "$PROJECT_ROOT/assets/icons/ziplock-logo.svg" ]; then
        cp "$PROJECT_ROOT/assets/icons/ziplock-logo.svg" "$install_dir/usr/share/icons/hicolor/scalable/apps/ziplock.svg"
    else
        log_warning "No application icon found"
    fi

    # Copy systemd service
    if [ -f "$PACKAGING_DIR/ziplock-backend.service" ]; then
        cp "$PACKAGING_DIR/ziplock-backend.service" "$install_dir/lib/systemd/system/"
    else
        log_error "Systemd service file not found: $PACKAGING_DIR/ziplock-backend.service"
        exit 1
    fi

    # Create default config
    cat > "$install_dir/etc/ziplock/config.yml" << EOF
# ZipLock Configuration
storage:
  backup_count: 5
  auto_backup: true
  compression:
    level: 6
    solid: false
    multi_threaded: true

security:
  auto_lock_timeout: 900  # 15 minutes
  min_master_key_length: 12
  enforce_strong_master_key: true

backend:
  bind_address: "127.0.0.1:0"  # Random port
  log_level: "info"
  max_sessions: 10

ui:
  theme: "auto"  # auto, light, dark
  font_size: 14
  show_password_strength: true
EOF

    log_success "Installation structure created at: $install_dir"
}

display_build_summary() {
    log_success "Build completed successfully!"
    echo
    echo "Build Summary:"
    echo "=============="
    echo "Profile: $PROFILE"
    echo "Target: $TARGET_ARCH"
    echo "Version: $VERSION"
    echo
    echo "Binaries:"
    echo "  Backend: $RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock-backend"
    echo "  Frontend: $RUST_TARGET_DIR/$TARGET_ARCH/$PROFILE/ziplock"
    echo
    echo "Installation structure: $BUILD_DIR/install"
    echo
    echo "Next steps:"
    echo "  1. Run './scripts/build/package-deb.sh' to create .deb package"
    echo "  2. Or manually install with 'sudo cp -r $BUILD_DIR/install/* /'"
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
    build_backend
    build_frontend

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
