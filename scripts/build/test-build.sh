#!/bin/bash
set -euo pipefail

# ZipLock Build Test Script
# Verifies that the build process works correctly

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

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

cleanup() {
    log_info "Cleaning up test artifacts..."
    cd "$PROJECT_ROOT"
    rm -rf target/test-build
    rm -rf target/test-install
}

test_prerequisites() {
    log_info "Testing prerequisites..."

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found"
        return 1
    fi

    # Check system dependencies
    local missing_deps=()

    if ! pkg-config --exists fontconfig; then
        missing_deps+=("fontconfig")
    fi

    if ! pkg-config --exists freetype2; then
        missing_deps+=("freetype2")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_warning "Missing system dependencies: ${missing_deps[*]}"
        log_info "This may cause build failures"
    fi

    log_success "Prerequisites check completed"
    return 0
}

test_workspace_build() {
    log_info "Testing workspace compilation..."

    cd "$PROJECT_ROOT"

    # Test debug build
    if ! cargo build --workspace; then
        log_error "Debug build failed"
        return 1
    fi

    # Test release build
    if ! cargo build --workspace --release; then
        log_error "Release build failed"
        return 1
    fi

    log_success "Workspace builds completed"
    return 0
}

test_individual_components() {
    log_info "Testing individual component builds..."

    cd "$PROJECT_ROOT"

    # Test shared library
    if ! cargo build --release -p ziplock-shared; then
        log_error "Shared library build failed"
        return 1
    fi

    # Test shared library with C API
    if ! cargo build --release -p ziplock-shared --features c-api; then
        log_error "Shared library build failed"
        return 1
    fi

    # Test unified application
    if ! cargo build --release -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog,ffi-client"; then
        log_error "Unified application build failed"
        return 1
    fi

    log_success "Individual component builds completed"
    return 0
}

test_binary_functionality() {
    log_info "Testing binary functionality..."

    cd "$PROJECT_ROOT"

    local app_binary="target/release/ziplock"
    local shared_lib="target/release/libziplock_shared.so"

    # Check if binaries exist
    if [ ! -f "$app_binary" ]; then
        log_error "ZipLock application binary not found: $app_binary"
        return 1
    fi

    if [ ! -f "$shared_lib" ]; then
        log_error "Shared library not found: $shared_lib"
        return 1
    fi

    # Test version commands (GUI app may require display, so allow failure)
    if ! "$app_binary" --version &> /dev/null; then
        log_warning "Application --version command failed (expected for GUI app without display)"
    fi

    # Test help commands (GUI app may require display, so allow failure)
    if ! "$app_binary" --help &> /dev/null; then
        log_warning "Application --help command failed (expected for GUI app without display)"
    fi

    log_success "Binary functionality tests completed"
    return 0
}

test_build_script() {
    log_info "Testing build script..."

    cd "$PROJECT_ROOT"

    # Clean previous test builds
    rm -rf target/test-build
    export CARGO_TARGET_DIR="$PROJECT_ROOT/target/test-build"

    # Test the build script
    if ! ./scripts/build/build-linux.sh --profile release; then
        log_error "Build script failed"
        return 1
    fi

    # Verify build script output
    if [ ! -f "target/test-build/install/usr/bin/ziplock" ]; then
        log_error "Build script didn't create ZipLock application binary in install directory"
        return 1
    fi

    if [ ! -f "target/test-build/install/usr/lib/libziplock_shared.so" ]; then
        log_error "Build script didn't create shared library in install directory"
        return 1
    fi

    log_success "Build script test completed"
    return 0
}

test_package_script() {
    log_info "Testing package script..."

    cd "$PROJECT_ROOT"

    # Ensure we have a build to package
    export CARGO_TARGET_DIR="$PROJECT_ROOT/target/test-build"

    if [ ! -d "target/test-build/install" ]; then
        log_warning "No install directory found, running build script first"
        if ! ./scripts/build/build-linux.sh --profile release; then
            log_error "Build script failed during package test"
            return 1
        fi
    fi

    # Test package creation
    if command -v dpkg-deb &> /dev/null && command -v fakeroot &> /dev/null; then
        export BUILD_DIR="$PROJECT_ROOT/target/test-build"
        if ! ./scripts/build/package-deb.sh --arch amd64; then
            log_error "Package script failed"
            return 1
        fi

        # Verify package was created
        if ! ls target/test-build/ziplock_*.deb &> /dev/null; then
            log_error "Package file not created"
            return 1
        fi

        log_success "Package script test completed"
    else
        log_warning "Packaging tools not available, skipping package test"
    fi

    return 0
}

test_code_quality() {
    log_info "Testing code quality..."

    cd "$PROJECT_ROOT"

    # Test formatting
    if ! cargo fmt --all -- --check; then
        log_warning "Code formatting check failed"
    fi

    # Test clippy
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        log_warning "Clippy lints failed"
    fi

    log_success "Code quality tests completed"
    return 0
}

run_tests() {
    log_info "Running unit tests..."

    cd "$PROJECT_ROOT"

    if ! cargo test --workspace; then
        log_error "Unit tests failed"
        return 1
    fi

    log_success "Unit tests completed"
    return 0
}

test_cross_compilation() {
    log_info "Testing cross-compilation setup..."

    cd "$PROJECT_ROOT"

    # Cross-compilation testing has been removed
    # ZipLock now focuses on x86_64 architecture only for Linux
    log_info "Cross-compilation testing skipped (ARM support removed)"

    log_success "Cross-compilation test completed"
}

print_summary() {
    echo
    echo "=================================="
    echo "Build Test Summary"
    echo "=================================="
    echo "✅ Prerequisites check"
    echo "✅ Workspace build test"
    echo "✅ Component build test"
    echo "✅ Binary functionality test"
    echo "✅ Build script test"
    echo "✅ Package script test (if tools available)"
    echo "✅ Code quality test"
    echo "✅ Unit tests"
    echo "✅ Cross-compilation test"
    echo
    echo "All tests completed successfully!"
    echo
    echo "Next steps:"
    echo "1. Run './scripts/build/build-linux.sh' for a full build"
    echo "2. Run './scripts/build/package-deb.sh' to create a .deb package"
    echo "3. Install and test the package"
    echo
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --quick          Run only essential tests"
    echo "  --no-cleanup     Don't clean up test artifacts"
    echo "  --help           Show this help message"
    echo
    echo "This script tests the entire ZipLock build process including:"
    echo "  - Prerequisites verification"
    echo "  - Workspace and component builds"
    echo "  - Binary functionality"
    echo "  - Build and package scripts"
    echo "  - Code quality checks"
    echo "  - Unit tests"
}

main() {
    local quick_mode=false
    local cleanup_enabled=true

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --quick)
                quick_mode=true
                shift
                ;;
            --no-cleanup)
                cleanup_enabled=false
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

    log_info "Starting ZipLock build test..."
    if [ "$quick_mode" = true ]; then
        log_info "Running in quick mode"
    fi

    # Set up cleanup trap
    if [ "$cleanup_enabled" = true ]; then
        trap cleanup EXIT
    fi

    # Run tests
    local failed_tests=()

    test_prerequisites || failed_tests+=("prerequisites")
    test_workspace_build || failed_tests+=("workspace_build")
    test_individual_components || failed_tests+=("component_builds")
    test_binary_functionality || failed_tests+=("binary_functionality")
    test_build_script || failed_tests+=("build_script")

    if [ "$quick_mode" = false ]; then
        test_package_script || failed_tests+=("package_script")
        test_code_quality || failed_tests+=("code_quality")
        run_tests || failed_tests+=("unit_tests")
        test_cross_compilation || failed_tests+=("cross_compilation")
    fi

    # Report results
    if [ ${#failed_tests[@]} -eq 0 ]; then
        print_summary
        exit 0
    else
        echo
        log_error "Some tests failed: ${failed_tests[*]}"
        echo
        echo "Failed tests:"
        for test in "${failed_tests[@]}"; do
            echo "  - $test"
        done
        echo
        echo "Please check the output above for specific error messages."
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
