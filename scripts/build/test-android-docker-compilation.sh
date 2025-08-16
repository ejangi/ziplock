#!/bin/bash

# ZipLock Android Docker Compilation Test
# Tests Android compilation in Docker to verify linking configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
DOCKER_FILE="$PROJECT_ROOT/.github/docker/android-builder.Dockerfile"
IMAGE_NAME="ziplock-android-test"

print_status "ZipLock Android Docker Compilation Test"
echo "========================================"

# Function to build test Docker image
build_test_image() {
    print_status "Building test Docker image..."

    if [ ! -f "$DOCKER_FILE" ]; then
        print_error "Docker file not found: $DOCKER_FILE"
        exit 1
    fi

    docker build -f "$DOCKER_FILE" -t "$IMAGE_NAME" "$PROJECT_ROOT"
    print_success "Test Docker image built successfully"
}

# Function to test basic Android compilation
test_basic_compilation() {
    print_status "Testing basic Android compilation..."

    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        "$IMAGE_NAME" \
        bash -c "
            set -e
            echo 'Testing basic Rust compilation for Android targets...'

            # Create a simple Rust project to test compilation
            mkdir -p /tmp/test-project
            cd /tmp/test-project

            # Initialize a simple Cargo project
            cargo init --name android-test --bin

            echo 'Testing aarch64-linux-android...'
            cargo build --target aarch64-linux-android --release
            file target/aarch64-linux-android/release/android-test

            echo 'Testing armv7-linux-androideabi...'
            cargo build --target armv7-linux-androideabi --release
            file target/armv7-linux-androideabi/release/android-test

            echo 'Testing x86_64-linux-android...'
            cargo build --target x86_64-linux-android --release
            file target/x86_64-linux-android/release/android-test

            echo 'Testing i686-linux-android...'
            cargo build --target i686-linux-android --release
            file target/i686-linux-android/release/android-test

            echo 'All basic compilations successful!'
        "

    print_success "Basic Android compilation test passed"
}

# Function to test ZipLock shared library compilation
test_ziplock_compilation() {
    print_status "Testing ZipLock shared library compilation..."

    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace/shared \
        "$IMAGE_NAME" \
        bash -c "
            set -e
            echo 'Testing ZipLock shared library compilation...'

            # Test compilation without building (faster check)
            echo 'Checking aarch64-linux-android...'
            cargo check --target aarch64-linux-android --features c-api

            echo 'Checking armv7-linux-androideabi...'
            cargo check --target armv7-linux-androideabi --features c-api

            echo 'Checking x86_64-linux-android...'
            cargo check --target x86_64-linux-android --features c-api

            echo 'Checking i686-linux-android...'
            cargo check --target i686-linux-android --features c-api

            echo 'All cargo check tests passed!'

            # Test actual build for ARM64 (most common Android target)
            echo 'Building for aarch64-linux-android...'
            cargo build --target aarch64-linux-android --features c-api --release

            # Verify the output
            if [ -f 'target/aarch64-linux-android/release/libziplock_shared.so' ]; then
                echo 'ARM64 library built successfully:'
                file target/aarch64-linux-android/release/libziplock_shared.so
                ls -lh target/aarch64-linux-android/release/libziplock_shared.so
            else
                echo 'ERROR: ARM64 library not found!'
                exit 1
            fi
        "

    print_success "ZipLock shared library compilation test passed"
}

# Function to verify symbols in the built library
test_library_symbols() {
    print_status "Testing library symbols..."

    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace/shared \
        "$IMAGE_NAME" \
        bash -c "
            set -e
            echo 'Verifying library symbols...'

            LIB_PATH='target/aarch64-linux-android/release/libziplock_shared.so'

            if [ ! -f \"\$LIB_PATH\" ]; then
                echo 'ERROR: Library not found at \$LIB_PATH'
                exit 1
            fi

            # Check if it's a valid shared library
            if ! file \"\$LIB_PATH\" | grep -q 'shared object'; then
                echo 'ERROR: Not a valid shared library'
                exit 1
            fi

            # Check for FFI symbols (if nm is available)
            if command -v nm >/dev/null 2>&1; then
                echo 'Checking for FFI symbols...'
                nm -D \"\$LIB_PATH\" 2>/dev/null | grep -E '(ziplock_|create_|destroy_)' | head -10 || echo 'No FFI symbols found (this might be normal)'
            fi

            # Check file size (should be reasonable)
            SIZE=\$(stat -c%s \"\$LIB_PATH\")
            echo \"Library size: \${SIZE} bytes\"

            if [ \"\$SIZE\" -lt 100000 ]; then
                echo 'WARNING: Library seems very small'
            elif [ \"\$SIZE\" -gt 50000000 ]; then
                echo 'WARNING: Library seems very large'
            else
                echo 'Library size looks reasonable'
            fi
        "

    print_success "Library symbols test passed"
}

# Function to clean up
cleanup() {
    print_status "Cleaning up..."
    docker rmi "$IMAGE_NAME" 2>/dev/null || true
    print_success "Cleanup completed"
}

# Function to run all tests
run_all_tests() {
    print_status "Running comprehensive Android compilation tests..."

    build_test_image
    test_basic_compilation
    test_ziplock_compilation
    test_library_symbols

    print_success "All Android compilation tests passed!"
    print_status "The Android build environment is working correctly."
}

# Usage
usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all        Run all tests (default)"
    echo "  basic      Test basic Android compilation only"
    echo "  ziplock    Test ZipLock library compilation only"
    echo "  symbols    Test library symbols only"
    echo "  clean      Clean up test artifacts"
    echo "  help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Run all tests"
    echo "  $0 basic        # Test basic compilation only"
    echo "  $0 clean        # Clean up"
}

# Main script logic
main() {
    local command="${1:-all}"

    case "$command" in
        "all")
            run_all_tests
            ;;
        "basic")
            build_test_image
            test_basic_compilation
            ;;
        "ziplock")
            build_test_image
            test_ziplock_compilation
            ;;
        "symbols")
            test_library_symbols
            ;;
        "clean")
            cleanup
            ;;
        "help"|"-h"|"--help")
            usage
            ;;
        *)
            print_error "Unknown command: $command"
            echo ""
            usage
            exit 1
            ;;
    esac
}

main "$@"
