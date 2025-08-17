#!/bin/bash

# Android Readiness Test Script
# Tests if the system is ready for Android compilation

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

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
WARNINGS=0

# Record test result
record_test() {
    local test_name="$1"
    local result="$2"
    local message="$3"

    if [ "$result" = "pass" ]; then
        print_success "✓ $test_name: $message"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    elif [ "$result" = "warn" ]; then
        print_warning "⚠ $test_name: $message"
        WARNINGS=$((WARNINGS + 1))
    else
        print_error "✗ $test_name: $message"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Test basic prerequisites
test_basic_prerequisites() {
    print_status "Testing basic prerequisites..."

    # Test Rust installation
    if command -v rustc >/dev/null 2>&1; then
        local rust_version=$(rustc --version)
        record_test "Rust compiler" "pass" "$rust_version"
    else
        record_test "Rust compiler" "fail" "rustc not found"
        return 1
    fi

    # Test Cargo installation
    if command -v cargo >/dev/null 2>&1; then
        local cargo_version=$(cargo --version)
        record_test "Cargo" "pass" "$cargo_version"
    else
        record_test "Cargo" "fail" "cargo not found"
        return 1
    fi

    # Test rustup installation
    if command -v rustup >/dev/null 2>&1; then
        local rustup_version=$(rustup --version | head -n 1)
        record_test "Rustup" "pass" "$rustup_version"
    else
        record_test "Rustup" "warn" "rustup not found (recommended for target management)"
    fi

    return 0
}

# Test Android targets availability
test_android_targets() {
    print_status "Testing Android targets..."

    if ! command -v rustup >/dev/null 2>&1; then
        record_test "Android targets" "warn" "rustup not available, skipping target check"
        return 0
    fi

    local targets=(
        "aarch64-linux-android"
        "armv7-linux-androideabi"
        "x86_64-linux-android"
        "i686-linux-android"
    )

    local available_targets=0
    local total_targets=${#targets[@]}

    for target in "${targets[@]}"; do
        if rustup target list --installed | grep -q "$target"; then
            record_test "Target $target" "pass" "installed"
            available_targets=$((available_targets + 1))
        else
            record_test "Target $target" "warn" "not installed (run: rustup target add $target)"
        fi
    done

    if [ $available_targets -eq $total_targets ]; then
        record_test "Android targets" "pass" "all targets available"
    elif [ $available_targets -gt 0 ]; then
        record_test "Android targets" "warn" "$available_targets/$total_targets targets available"
    else
        record_test "Android targets" "fail" "no Android targets installed"
    fi

    return 0
}

# Test host compilation
test_host_compilation() {
    print_status "Testing host compilation..."

    cd "$SHARED_DIR"

    # Test basic check
    if cargo check --features c-api >/dev/null 2>&1; then
        record_test "Host compilation" "pass" "cargo check passed"
    else
        record_test "Host compilation" "fail" "cargo check failed"
        return 1
    fi

    # Test FFI features specifically
    if cargo check --features c-api --no-default-features >/dev/null 2>&1; then
        record_test "FFI features" "pass" "c-api feature compiles"
    else
        record_test "FFI features" "fail" "c-api feature compilation failed"
        return 1
    fi

    return 0
}

# Test Android NDK availability
test_android_ndk() {
    print_status "Testing Android NDK..."

    # Check ANDROID_NDK_HOME
    if [ -n "$ANDROID_NDK_HOME" ] && [ -d "$ANDROID_NDK_HOME" ]; then
        record_test "NDK environment" "pass" "ANDROID_NDK_HOME set to $ANDROID_NDK_HOME"

        # Check for toolchain
        local toolchain_path="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
        if [ -d "$toolchain_path" ]; then
            record_test "NDK toolchain" "pass" "toolchain found at $toolchain_path"

            # Check for specific tools
            if [ -f "$toolchain_path/aarch64-linux-android21-clang" ]; then
                record_test "NDK compiler" "pass" "aarch64-linux-android21-clang available"
            else
                record_test "NDK compiler" "warn" "some NDK compilers may be missing"
            fi
        else
            record_test "NDK toolchain" "fail" "toolchain not found at expected location"
        fi
    else
        # Try to find NDK in common locations
        local ndk_found=false
        local ndk_paths=(
            "$HOME/Android/Sdk/ndk"
            "/opt/android-ndk"
            "/usr/local/android-ndk"
        )

        for ndk_path in "${ndk_paths[@]}"; do
            if [ -d "$ndk_path" ]; then
                local versions=$(ls "$ndk_path" 2>/dev/null | head -3)
                if [ -n "$versions" ]; then
                    record_test "NDK detection" "warn" "found at $ndk_path (set ANDROID_NDK_HOME)"
                    ndk_found=true
                    break
                fi
            fi
        done

        if [ "$ndk_found" = false ]; then
            record_test "Android NDK" "warn" "not found (required for native Android compilation)"
        fi
    fi

    return 0
}

# Test Docker availability
test_docker() {
    print_status "Testing Docker availability..."

    if command -v docker >/dev/null 2>&1; then
        if docker info >/dev/null 2>&1; then
            local docker_version=$(docker --version)
            record_test "Docker" "pass" "$docker_version"
        else
            record_test "Docker" "warn" "installed but not running or accessible"
        fi
    else
        record_test "Docker" "warn" "not installed (recommended for consistent builds)"
    fi

    return 0
}

# Test build scripts
test_build_scripts() {
    print_status "Testing build scripts..."

    local scripts=(
        "build-mobile.sh"
        "build-android-docker.sh"
        "test-android-compilation.sh"
        "test-android-integration.sh"
        "test-android-builder-image.sh"
    )

    for script in "${scripts[@]}"; do
        local script_path="$PROJECT_ROOT/scripts/build/$script"
        if [ -f "$script_path" ]; then
            if [ -x "$script_path" ]; then
                record_test "Script $script" "pass" "exists and executable"
            else
                record_test "Script $script" "warn" "exists but not executable"
            fi
        else
            record_test "Script $script" "warn" "not found"
        fi
    done

    return 0
}

# Test project structure
test_project_structure() {
    print_status "Testing project structure..."

    # Check shared library
    if [ -f "$SHARED_DIR/Cargo.toml" ]; then
        record_test "Shared library" "pass" "Cargo.toml found"
    else
        record_test "Shared library" "fail" "Cargo.toml not found"
        return 1
    fi

    # Check FFI module
    if [ -f "$SHARED_DIR/src/ffi.rs" ]; then
        record_test "FFI module" "pass" "ffi.rs found"
    else
        record_test "FFI module" "fail" "ffi.rs not found"
        return 1
    fi

    # Check C header
    if [ -f "$SHARED_DIR/include/ziplock.h" ]; then
        record_test "C header" "pass" "ziplock.h found"
    else
        record_test "C header" "fail" "ziplock.h not found"
        return 1
    fi

    # Check for c-api feature
    if grep -q 'c-api' "$SHARED_DIR/Cargo.toml"; then
        record_test "C API feature" "pass" "c-api feature defined"
    else
        record_test "C API feature" "warn" "c-api feature not found in Cargo.toml"
    fi

    return 0
}

# Provide recommendations
provide_recommendations() {
    echo ""
    print_status "Recommendations:"
    echo ""

    if [ $TESTS_FAILED -gt 0 ]; then
        echo "❌ Critical issues found - Android compilation will not work:"
        echo ""

        if ! command -v rustc >/dev/null 2>&1; then
            echo "  • Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        fi

        if [ ! -f "$SHARED_DIR/Cargo.toml" ]; then
            echo "  • Ensure you're in the correct ZipLock project directory"
        fi

        echo ""
    fi

    if [ $WARNINGS -gt 0 ]; then
        echo "⚠️  Recommended improvements:"
        echo ""

        if ! rustup target list --installed | grep -q "linux-android"; then
            echo "  • Install Android targets:"
            echo "    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android"
        fi

        if [ -z "$ANDROID_NDK_HOME" ]; then
            echo "  • For native builds, install Android NDK:"
            echo "    - Download from: https://developer.android.com/ndk/downloads"
            echo "    - Set ANDROID_NDK_HOME environment variable"
        fi

        if ! command -v docker >/dev/null 2>&1; then
            echo "  • Install Docker for consistent builds:"
            echo "    - Ubuntu/Debian: sudo apt-get install docker.io"
            echo "    - Or follow: https://docs.docker.com/engine/install/"
        fi

        echo ""
    fi

    if [ $TESTS_FAILED -eq 0 ]; then
        echo "✅ Ready for Android development!"
        echo ""
        echo "Next steps:"

        if [ -n "$ANDROID_NDK_HOME" ] && [ -d "$ANDROID_NDK_HOME" ]; then
            echo "  • Test native compilation: ./scripts/build/test-android-compilation.sh"
            echo "  • Build for Android: ./scripts/build/build-mobile.sh android"
        else
            echo "  • Test Docker build: ./scripts/build/build-android-docker.sh build"
        fi

        echo "  • Read documentation: docs/technical/android.md"
        echo ""
    fi
}

# Main function
main() {
    echo "Android Readiness Test"
    echo "====================="
    echo ""

    test_basic_prerequisites || true
    echo ""

    test_project_structure || true
    echo ""

    test_host_compilation || true
    echo ""

    test_android_targets || true
    echo ""

    test_android_ndk || true
    echo ""

    test_docker || true
    echo ""

    test_build_scripts || true
    echo ""

    # Summary
    echo "Test Summary"
    echo "============"
    echo "✅ Passed: $TESTS_PASSED"
    echo "⚠️  Warnings: $WARNINGS"
    echo "❌ Failed: $TESTS_FAILED"

    provide_recommendations

    # Exit with appropriate code
    if [ $TESTS_FAILED -gt 0 ]; then
        exit 1
    elif [ $WARNINGS -gt 0 ]; then
        exit 2
    else
        exit 0
    fi
}

main "$@"
