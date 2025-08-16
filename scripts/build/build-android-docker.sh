#!/bin/bash

# ZipLock Android Docker Build Script
# Builds Android libraries in a consistent Docker environment

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
IMAGE_NAME="ziplock-android-builder"
OUTPUT_DIR="$PROJECT_ROOT/android-builds"

# Function to build Docker image
build_image() {
    print_status "Building Android builder Docker image..."

    if [ ! -f "$DOCKER_FILE" ]; then
        print_error "Docker file not found: $DOCKER_FILE"
        exit 1
    fi

    docker build -f "$DOCKER_FILE" -t "$IMAGE_NAME" "$PROJECT_ROOT"
    print_success "Docker image built successfully"
}

# Function to run Android build in container
run_build() {
    local targets="${1:-all}"

    print_status "Running Android build in container..."

    # Create output directory
    mkdir -p "$OUTPUT_DIR"

    # Run build in container
    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -v "$OUTPUT_DIR:/output" \
        -w /workspace \
        "$IMAGE_NAME" \
        bash -c "
            set -e
            echo 'Building ZipLock Android libraries...'

            cd shared

            # Build for specified targets
            if [ '$targets' = 'all' ] || [ '$targets' = 'arm64' ]; then
                echo 'Building for ARM64...'
                cargo build --release --target aarch64-linux-android --features c-api

                mkdir -p /output/arm64-v8a
                cp /workspace/target/aarch64-linux-android/release/libziplock_shared.so /output/arm64-v8a/
            fi

            if [ '$targets' = 'all' ] || [ '$targets' = 'armv7' ]; then
                echo 'Building for ARMv7...'
                cargo build --release --target armv7-linux-androideabi --features c-api
                mkdir -p /output/armeabi-v7a
                cp /workspace/target/armv7-linux-androideabi/release/libziplock_shared.so /output/armeabi-v7a/
            fi

            if [ '$targets' = 'all' ] || [ '$targets' = 'x86_64' ]; then
                echo 'Building for x86_64...'
                cargo build --release --target x86_64-linux-android --features c-api
                mkdir -p /output/x86_64
                cp /workspace/target/x86_64-linux-android/release/libziplock_shared.so /output/x86_64/
            fi

            if [ '$targets' = 'all' ] || [ '$targets' = 'x86' ]; then
                echo 'Building for x86...'
                cargo build --release --target i686-linux-android --features c-api
                mkdir -p /output/x86
                cp /workspace/target/i686-linux-android/release/libziplock_shared.so /output/x86/
            fi

            # Copy header file
            cp /workspace/shared/include/ziplock.h /output/

            echo 'Android build completed successfully!'
        "

    print_success "Android libraries built successfully"
    print_status "Output directory: $OUTPUT_DIR"
}

# Function to test the built libraries
test_libraries() {
    print_status "Testing built libraries..."

    for arch in arm64-v8a armeabi-v7a x86_64 x86; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            size=$(du -h "$lib_path" | cut -f1)
            print_success "✓ $arch: $size"

            # Verify it's a valid shared library
            if file "$lib_path" | grep -q "shared object"; then
                echo "  - Valid shared library format"
            else
                print_error "  - Invalid library format"
            fi
        else
            print_error "✗ $arch: Library not found"
        fi
    done

    # Test header file
    if [ -f "$OUTPUT_DIR/ziplock.h" ]; then
        print_success "✓ Header file: $(du -h "$OUTPUT_DIR/ziplock.h" | cut -f1)"
    else
        print_error "✗ Header file not found"
    fi
}

# Clean up
clean() {
    print_status "Cleaning up..."
    rm -rf "$OUTPUT_DIR"
    docker rmi "$IMAGE_NAME" 2>/dev/null || true
    print_success "Clean completed"
}

# Usage
usage() {
    echo "Usage: $0 [COMMAND] [TARGET]"
    echo ""
    echo "Commands:"
    echo "  build [TARGET]    Build Android libraries (default: all)"
    echo "  test              Test built libraries"
    echo "  clean             Clean up build artifacts and Docker image"
    echo "  help              Show this help message"
    echo ""
    echo "Targets:"
    echo "  all               Build for all architectures (default)"
    echo "  arm64             Build for ARM64 only"
    echo "  armv7             Build for ARMv7 only"
    echo "  x86_64            Build for x86_64 only"
    echo "  x86               Build for x86 only"
    echo ""
    echo "Examples:"
    echo "  $0 build                 # Build for all architectures"
    echo "  $0 build arm64           # Build for ARM64 only"
    echo "  $0 test                  # Test built libraries"
}

# Main script logic
main() {
    local command="${1:-build}"
    local target="${2:-all}"

    case "$command" in
        "build")
            build_image
            run_build "$target"
            test_libraries
            ;;
        "test")
            test_libraries
            ;;
        "clean")
            clean
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
