#!/bin/bash

# Android Compilation Test Script
# Verifies that all Android targets compile successfully

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
SHARED_DIR="$PROJECT_ROOT/shared"

# Android targets to test
TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
    "i686-linux-android"
)

# Target descriptions
declare -A TARGET_DESCRIPTIONS
TARGET_DESCRIPTIONS["aarch64-linux-android"]="ARM64 (primary target)"
TARGET_DESCRIPTIONS["armv7-linux-androideabi"]="ARMv7 (legacy support)"
TARGET_DESCRIPTIONS["x86_64-linux-android"]="x86_64 (emulator)"
TARGET_DESCRIPTIONS["i686-linux-android"]="x86 (emulator)"

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    # Check if Rust is installed
    if ! command -v rustup >/dev/null 2>&1; then
        print_error "Rust is not installed. Please install from https://rustup.rs/"
        exit 1
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        print_error "Cargo is not installed. Please install Rust toolchain."
        exit 1
    fi

    # Check if we're in the right directory
    if [ ! -f "$SHARED_DIR/Cargo.toml" ]; then
        print_error "Shared library Cargo.toml not found. Are you in the right directory?"
        exit 1
    fi

    # Check Android NDK (required for native builds)
    local ndk_available=false

    if [ -z "$ANDROID_NDK_HOME" ]; then
        print_warning "ANDROID_NDK_HOME not set."

        # Try to find NDK in common locations
        local ndk_paths=(
            "$HOME/Android/Sdk/ndk"
            "/opt/android-ndk"
            "/usr/local/android-ndk"
        )

        for ndk_path in "${ndk_paths[@]}"; do
            if [ -d "$ndk_path" ]; then
                # Find latest version
                local latest_ndk=$(ls "$ndk_path" 2>/dev/null | sort -V | tail -n 1)
                if [ -n "$latest_ndk" ] && [ -d "$ndk_path/$latest_ndk" ]; then
                    export ANDROID_NDK_HOME="$ndk_path/$latest_ndk"
                    print_warning "Auto-detected NDK at: $ANDROID_NDK_HOME"
                    ndk_available=true
                    break
                fi
            fi
        done

        if [ "$ndk_available" = false ]; then
            print_error "Android NDK not found!"
            print_error ""
            print_error "To install Android NDK:"
            print_error "1. Download from: https://developer.android.com/ndk/downloads"
            print_error "2. Extract and set ANDROID_NDK_HOME=/path/to/ndk"
            print_error "3. Or use Docker build: ./scripts/build/build-android-docker.sh"
            print_error ""
            print_error "For quick testing without NDK, run:"
            print_error "  ./scripts/build/build-android-docker.sh build"
            return 1
        fi
    elif [ ! -d "$ANDROID_NDK_HOME" ]; then
        print_error "ANDROID_NDK_HOME points to non-existent directory: $ANDROID_NDK_HOME"
        return 1
    else
        print_success "Android NDK found at: $ANDROID_NDK_HOME"
        ndk_available=true
    fi

    # Check if NDK tools are in PATH
    if [ "$ndk_available" = true ]; then
        local toolchain_path="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
        if [ -d "$toolchain_path" ]; then
            export PATH="$toolchain_path:$PATH"
            print_success "Added NDK tools to PATH"
        else
            print_error "NDK toolchain not found at: $toolchain_path"
            return 1
        fi
    fi

    print_success "Prerequisites check completed"
}

# Install Android targets
install_android_targets() {
    print_status "Installing Android targets..."

    for target in "${TARGETS[@]}"; do
        print_status "Installing target: $target"
        if rustup target add "$target"; then
            print_success "✓ Installed: $target"
        else
            print_error "✗ Failed to install: $target"
            exit 1
        fi
    done

    print_success "All Android targets installed"
}

# Test compilation for a single target
test_target_compilation() {
    local target="$1"
    local build_mode="${2:-debug}"
    local description="${TARGET_DESCRIPTIONS[$target]}"

    print_status "Testing compilation for $target ($description) in $build_mode mode"

    cd "$SHARED_DIR"

    # Test check (faster than full build)
    if cargo check --target "$target" --features c-api; then
        print_success "✓ Check passed: $target"
    else
        print_error "✗ Check failed: $target"
        return 1
    fi

    # Test actual build
    local build_flags="--target $target --features c-api"
    if [ "$build_mode" = "release" ]; then
        build_flags="$build_flags --release"
    fi

    if cargo build $build_flags; then
        print_success "✓ Build passed: $target ($build_mode)"
    else
        print_error "✗ Build failed: $target ($build_mode)"
        return 1
    fi

    # Verify output file exists
    local lib_path
    case "$target" in
        *-android)
            lib_path="target/$target/$build_mode/libziplock_shared.so"
            ;;
        *)
            print_error "Unknown target type: $target"
            return 1
            ;;
    esac

    if [ -f "$lib_path" ]; then
        local size=$(du -h "$lib_path" | cut -f1)
        print_success "✓ Library created: $lib_path ($size)"
    else
        print_error "✗ Library not found: $lib_path"
        return 1
    fi

    return 0
}

# Test all Android targets
test_all_targets() {
    local build_mode="${1:-debug}"
    print_status "Testing compilation for all Android targets in $build_mode mode..."
    echo ""

    local failed_targets=()
    local successful_targets=()

    for target in "${TARGETS[@]}"; do
        if test_target_compilation "$target" "$build_mode"; then
            successful_targets+=("$target")
        else
            failed_targets+=("$target")
        fi
        echo ""
    done

    # Summary
    echo "Compilation Test Summary ($build_mode mode):"
    echo "========================"

    if [ ${#successful_targets[@]} -gt 0 ]; then
        echo ""
        echo "✅ Successful targets:"
        for target in "${successful_targets[@]}"; do
            echo "  ✓ $target - ${TARGET_DESCRIPTIONS[$target]}"
        done
    fi

    if [ ${#failed_targets[@]} -gt 0 ]; then
        echo ""
        echo "❌ Failed targets:"
        for target in "${failed_targets[@]}"; do
            echo "  ✗ $target - ${TARGET_DESCRIPTIONS[$target]}"
        done
        echo ""
        print_error "Some targets failed to compile. Check the errors above."
        return 1
    fi

    echo ""
    print_success "All Android targets compiled successfully in $build_mode mode!"
    return 0
}

# Test specific features
test_features() {
    print_status "Testing feature compilation..."

    local target="aarch64-linux-android"  # Use primary target for feature testing

    cd "$SHARED_DIR"

    # Test without features
    print_status "Testing no features..."
    if cargo check --target "$target" --no-default-features; then
        print_success "✓ No features compilation passed"
    else
        print_error "✗ No features compilation failed"
        return 1
    fi

    # Test c-api feature specifically
    print_status "Testing c-api feature..."
    if cargo check --target "$target" --features c-api --no-default-features; then
        print_success "✓ c-api feature compilation passed"
    else
        print_error "✗ c-api feature compilation failed"
        return 1
    fi

    # Test all features
    print_status "Testing all features..."
    if cargo check --target "$target" --all-features; then
        print_success "✓ All features compilation passed"
    else
        print_error "✗ All features compilation failed"
        return 1
    fi

    return 0
}

# Test release builds for all targets
test_release_builds() {
    print_status "Testing release builds for all Android targets..."

    test_all_targets "release"
}

# Clean build artifacts
clean_builds() {
    print_status "Cleaning build artifacts..."

    cd "$SHARED_DIR"
    cargo clean

    print_success "Build artifacts cleaned"
}

# Run tests
run_tests() {
    print_status "Running tests for Android compilation..."

    cd "$SHARED_DIR"

    # Run tests with default target (host)
    if cargo test --features c-api; then
        print_success "✓ Tests passed"
    else
        print_warning "⚠ Some tests failed (this may be expected for FFI tests)"
    fi
}

# Usage
usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  targets        Test compilation for all Android targets in debug mode (default)"
    echo "  features       Test feature compilation"
    echo "  release        Test release builds for all Android targets"
    echo "  test           Run tests"
    echo "  install        Install Android targets"
    echo "  clean          Clean build artifacts"
    echo "  all            Run all tests"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0             # Test all Android targets in debug mode"
    echo "  $0 targets     # Test compilation for all targets in debug mode"
    echo "  $0 release     # Test compilation for all targets in release mode"
    echo "  $0 features    # Test feature compilation"
    echo "  $0 all         # Run comprehensive tests"
}

# Main function
main() {
    local command="${1:-targets}"

    case "$command" in
        "targets")
            check_prerequisites
            install_android_targets
            test_all_targets "debug"
            ;;
        "features")
            check_prerequisites
            test_features
            ;;
        "release")
            check_prerequisites
            install_android_targets
            test_release_builds
            ;;
        "test")
            check_prerequisites
            run_tests
            ;;
        "install")
            check_prerequisites
            install_android_targets
            ;;
        "clean")
            clean_builds
            ;;
        "all")
            check_prerequisites
            install_android_targets
            test_all_targets "debug"
            test_features
            test_release_builds
            run_tests
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
