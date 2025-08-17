#!/bin/bash

# ZipLock Android Builder Image Test Script
# Tests the Android builder Docker image functionality

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
REGISTRY_IMAGE="ghcr.io/ejangi/ziplock/android-builder:latest"
LOCAL_IMAGE="ziplock-android-builder"
TEST_OUTPUT_DIR="$PROJECT_ROOT/test-android-output"

# Test functions
test_registry_image() {
    print_status "Testing registry image: $REGISTRY_IMAGE"

    if docker pull "$REGISTRY_IMAGE" 2>/dev/null; then
        print_success "Successfully pulled registry image"

        # Test basic functionality
        print_status "Testing basic tools in registry image..."
        docker run --rm "$REGISTRY_IMAGE" bash -c "
            rustc --version &&
            cargo --version &&
            aarch64-linux-android21-clang --version &&
            echo 'Registry image tools verified successfully'
        "
        print_success "Registry image tools test passed"
        return 0
    else
        print_error "Failed to pull registry image"
        return 1
    fi
}

test_local_image() {
    print_status "Testing local image: $LOCAL_IMAGE"

    if docker image inspect "$LOCAL_IMAGE" >/dev/null 2>&1; then
        print_success "Local image found"

        # Test basic functionality
        print_status "Testing basic tools in local image..."
        docker run --rm "$LOCAL_IMAGE" bash -c "
            rustc --version &&
            cargo --version &&
            aarch64-linux-android21-clang --version &&
            echo 'Local image tools verified successfully'
        "
        print_success "Local image tools test passed"
        return 0
    else
        print_error "Local image not found"
        return 1
    fi
}

test_build_capability() {
    local image_name="$1"
    print_status "Testing Android build capability with image: $image_name"

    # Create test output directory
    mkdir -p "$TEST_OUTPUT_DIR"

    # Test building a simple Android library
    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -v "$TEST_OUTPUT_DIR:/output" \
        -w /workspace \
        "$image_name" \
        bash -c "
            set -e
            echo 'Testing Android compilation...'

            # Test if we can compile for Android targets
            cd shared

            # Test ARM64 compilation
            echo 'Testing ARM64 compilation...'
            cargo check --target aarch64-linux-android --features c-api
            echo 'ARM64 compilation check passed'

            # Test ARMv7 compilation
            echo 'Testing ARMv7 compilation...'
            cargo check --target armv7-linux-androideabi --features c-api
            echo 'ARMv7 compilation check passed'

            # Test x86_64 compilation
            echo 'Testing x86_64 compilation...'
            cargo check --target x86_64-linux-android --features c-api
            echo 'x86_64 compilation check passed'

            # Test x86 compilation
            echo 'Testing x86 compilation...'
            cargo check --target i686-linux-android --features c-api
            echo 'x86 compilation check passed'

            echo 'All Android targets compilation tests passed!'
        "

    if [ $? -eq 0 ]; then
        print_success "Android build capability test passed"
        return 0
    else
        print_error "Android build capability test failed"
        return 1
    fi
}

test_environment_variables() {
    local image_name="$1"
    print_status "Testing environment variables in image: $image_name"

    docker run --rm "$image_name" bash -c "
        echo 'Testing environment variables...'

        # Check ANDROID_NDK_HOME
        if [ -z \"\$ANDROID_NDK_HOME\" ]; then
            echo 'ERROR: ANDROID_NDK_HOME not set'
            exit 1
        fi
        echo \"ANDROID_NDK_HOME: \$ANDROID_NDK_HOME\"

        # Check NDK_ROOT
        if [ -z \"\$NDK_ROOT\" ]; then
            echo 'ERROR: NDK_ROOT not set'
            exit 1
        fi
        echo \"NDK_ROOT: \$NDK_ROOT\"

        # Check ANDROID_API_LEVEL
        if [ -z \"\$ANDROID_API_LEVEL\" ]; then
            echo 'ERROR: ANDROID_API_LEVEL not set'
            exit 1
        fi
        echo \"ANDROID_API_LEVEL: \$ANDROID_API_LEVEL\"

        # Check PATH includes NDK tools
        if ! which aarch64-linux-android21-clang >/dev/null 2>&1; then
            echo 'ERROR: NDK tools not in PATH'
            exit 1
        fi
        echo 'NDK tools found in PATH'

        # Check Cargo config
        if [ ! -f /root/.cargo/config.toml ]; then
            echo 'ERROR: Cargo config not found'
            exit 1
        fi
        echo 'Cargo config found'

        echo 'All environment variables test passed!'
    "

    if [ $? -eq 0 ]; then
        print_success "Environment variables test passed"
        return 0
    else
        print_error "Environment variables test failed"
        return 1
    fi
}

clean_test_output() {
    print_status "Cleaning test output..."
    rm -rf "$TEST_OUTPUT_DIR"
    print_success "Test output cleaned"
}

# Test scenarios
test_registry() {
    print_status "=== Testing Registry Image ==="

    if test_registry_image; then
        test_environment_variables "$REGISTRY_IMAGE"
        test_build_capability "$REGISTRY_IMAGE"
        print_success "Registry image tests completed successfully"
        return 0
    else
        print_error "Registry image tests failed"
        return 1
    fi
}

test_local() {
    print_status "=== Testing Local Image ==="

    if test_local_image; then
        test_environment_variables "$LOCAL_IMAGE"
        test_build_capability "$LOCAL_IMAGE"
        print_success "Local image tests completed successfully"
        return 0
    else
        print_error "Local image tests failed"
        return 1
    fi
}

test_both() {
    print_status "=== Testing Both Registry and Local Images ==="

    local registry_result=0
    local local_result=0

    test_registry || registry_result=1
    echo ""
    test_local || local_result=1

    if [ $registry_result -eq 0 ] && [ $local_result -eq 0 ]; then
        print_success "Both registry and local image tests passed"
        return 0
    elif [ $registry_result -eq 0 ]; then
        print_warning "Registry image tests passed, local image tests failed"
        return 1
    elif [ $local_result -eq 0 ]; then
        print_warning "Local image tests passed, registry image tests failed"
        return 1
    else
        print_error "Both registry and local image tests failed"
        return 1
    fi
}

# Usage
usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  registry          Test registry image only"
    echo "  local             Test local image only"
    echo "  both              Test both registry and local images"
    echo "  clean             Clean test output directory"
    echo "  help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 registry       # Test registry image"
    echo "  $0 local          # Test local image"
    echo "  $0 both           # Test both images"
    echo "  $0 clean          # Clean up test files"
}

# Main script logic
main() {
    local command="${1:-both}"

    case "$command" in
        "registry")
            test_registry
            ;;
        "local")
            test_local
            ;;
        "both")
            test_both
            ;;
        "clean")
            clean_test_output
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

# Run main function with all arguments
main "$@"
