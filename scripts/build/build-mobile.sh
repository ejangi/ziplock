#!/bin/bash

# ZipLock Mobile Library Build Script
# This script builds the ZipLock shared library for iOS and Android platforms

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SHARED_DIR="$PROJECT_ROOT/shared"
OUTPUT_DIR="$PROJECT_ROOT/mobile-builds"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install Rust targets
install_rust_targets() {
    print_status "Installing Rust targets..."

    # iOS targets
    rustup target add aarch64-apple-ios || true
    rustup target add x86_64-apple-ios || true
    rustup target add aarch64-apple-ios-sim || true

    # Android targets
    rustup target add aarch64-linux-android || true
    rustup target add armv7-linux-androideabi || true
    rustup target add x86_64-linux-android || true
    rustup target add i686-linux-android || true

    print_success "Rust targets installed"
}

# Function to build for iOS
build_ios() {
    print_status "Building for iOS..."

    cd "$SHARED_DIR"

    # Create output directories
    mkdir -p "$OUTPUT_DIR/ios/device"
    mkdir -p "$OUTPUT_DIR/ios/simulator"
    mkdir -p "$OUTPUT_DIR/ios/xcframework"

    # Build for iOS device (ARM64)
    print_status "Building for iOS device (ARM64)..."
    cargo build --release --target aarch64-apple-ios --features c-api
    cp "target/aarch64-apple-ios/release/libziplock_shared.a" "$OUTPUT_DIR/ios/device/"

    # Build for iOS simulator (x86_64)
    print_status "Building for iOS simulator (x86_64)..."
    cargo build --release --target x86_64-apple-ios --features c-api

    # Build for iOS simulator (ARM64)
    print_status "Building for iOS simulator (ARM64)..."
    cargo build --release --target aarch64-apple-ios-sim --features c-api

    # Create universal library for simulator
    print_status "Creating universal library for iOS simulator..."
    if command_exists lipo; then
        lipo -create \
            "target/x86_64-apple-ios/release/libziplock_shared.a" \
            "target/aarch64-apple-ios-sim/release/libziplock_shared.a" \
            -output "$OUTPUT_DIR/ios/simulator/libziplock_shared.a"
    else
        print_warning "lipo not found, copying ARM64 simulator library only"
        cp "target/aarch64-apple-ios-sim/release/libziplock_shared.a" "$OUTPUT_DIR/ios/simulator/"
    fi

    # Create XCFramework if xcodebuild is available
    if command_exists xcodebuild; then
        print_status "Creating XCFramework..."
        rm -rf "$OUTPUT_DIR/ios/xcframework/ZipLockCore.xcframework"
        xcodebuild -create-xcframework \
            -library "$OUTPUT_DIR/ios/device/libziplock_shared.a" \
            -headers "$SHARED_DIR/include" \
            -library "$OUTPUT_DIR/ios/simulator/libziplock_shared.a" \
            -headers "$SHARED_DIR/include" \
            -output "$OUTPUT_DIR/ios/xcframework/ZipLockCore.xcframework"
        print_success "XCFramework created at $OUTPUT_DIR/ios/xcframework/ZipLockCore.xcframework"
    else
        print_warning "xcodebuild not found, skipping XCFramework creation"
    fi

    # Copy header file
    cp "$SHARED_DIR/include/ziplock.h" "$OUTPUT_DIR/ios/"

    print_success "iOS build completed"
}

# Function to setup Android NDK configuration
setup_android_ndk() {
    if [ -z "$ANDROID_NDK_HOME" ]; then
        # Try to find NDK in common locations
        if [ -d "$HOME/Android/Sdk/ndk" ]; then
            # Find the latest NDK version
            NDK_VERSION=$(ls "$HOME/Android/Sdk/ndk" | sort -V | tail -n 1)
            export ANDROID_NDK_HOME="$HOME/Android/Sdk/ndk/$NDK_VERSION"
            print_status "Found Android NDK at $ANDROID_NDK_HOME"
        elif [ -d "/opt/android-ndk" ]; then
            export ANDROID_NDK_HOME="/opt/android-ndk"
            print_status "Found Android NDK at $ANDROID_NDK_HOME"
        else
            print_error "Android NDK not found. Please set ANDROID_NDK_HOME environment variable"
            print_error "Download from: https://developer.android.com/ndk/downloads"
            return 1
        fi
    fi

    # Determine host OS for toolchain path
    case "$(uname -s)" in
        Linux*)     HOST_TAG=linux-x86_64;;
        Darwin*)    HOST_TAG=darwin-x86_64;;
        MINGW*)     HOST_TAG=windows-x86_64;;
        *)          print_error "Unsupported host OS: $(uname -s)"; return 1;;
    esac

    TOOLCHAIN_PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG"

    if [ ! -d "$TOOLCHAIN_PATH" ]; then
        print_error "Android NDK toolchain not found at $TOOLCHAIN_PATH"
        return 1
    fi

    export PATH="$TOOLCHAIN_PATH/bin:$PATH"

    # Create cargo config for Android cross-compilation
    mkdir -p "$HOME/.cargo"

    cat >> "$HOME/.cargo/config.toml" << EOF

# ZipLock Android build configuration
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android21-clang"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi21-clang"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android21-clang"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android21-clang"
EOF

    print_success "Android NDK configuration completed"
}

# Function to build for Android
build_android() {
    print_status "Building for Android..."

    setup_android_ndk || return 1

    cd "$SHARED_DIR"

    # Create output directories
    mkdir -p "$OUTPUT_DIR/android/arm64-v8a"
    mkdir -p "$OUTPUT_DIR/android/armeabi-v7a"
    mkdir -p "$OUTPUT_DIR/android/x86_64"
    mkdir -p "$OUTPUT_DIR/android/x86"

    # Build for ARM64
    print_status "Building for Android ARM64..."
    cargo build --release --target aarch64-linux-android --features c-api
    cp "target/aarch64-linux-android/release/libziplock_shared.so" "$OUTPUT_DIR/android/arm64-v8a/"

    # Build for ARMv7
    print_status "Building for Android ARMv7..."
    cargo build --release --target armv7-linux-androideabi --features c-api
    cp "target/armv7-linux-androideabi/release/libziplock_shared.so" "$OUTPUT_DIR/android/armeabi-v7a/"

    # Build for x86_64
    print_status "Building for Android x86_64..."
    cargo build --release --target x86_64-linux-android --features c-api
    cp "target/x86_64-linux-android/release/libziplock_shared.so" "$OUTPUT_DIR/android/x86_64/"

    # Build for x86
    print_status "Building for Android x86..."
    cargo build --release --target i686-linux-android --features c-api
    cp "target/i686-linux-android/release/libziplock_shared.so" "$OUTPUT_DIR/android/x86/"

    # Copy header file
    cp "$SHARED_DIR/include/ziplock.h" "$OUTPUT_DIR/android/"

    # Create Android project structure
    mkdir -p "$OUTPUT_DIR/android/jniLibs/arm64-v8a"
    mkdir -p "$OUTPUT_DIR/android/jniLibs/armeabi-v7a"
    mkdir -p "$OUTPUT_DIR/android/jniLibs/x86_64"
    mkdir -p "$OUTPUT_DIR/android/jniLibs/x86"

    cp "$OUTPUT_DIR/android/arm64-v8a/libziplock_shared.so" "$OUTPUT_DIR/android/jniLibs/arm64-v8a/"
    cp "$OUTPUT_DIR/android/armeabi-v7a/libziplock_shared.so" "$OUTPUT_DIR/android/jniLibs/armeabi-v7a/"
    cp "$OUTPUT_DIR/android/x86_64/libziplock_shared.so" "$OUTPUT_DIR/android/jniLibs/x86_64/"
    cp "$OUTPUT_DIR/android/x86/libziplock_shared.so" "$OUTPUT_DIR/android/jniLibs/x86/"

    print_success "Android build completed"
}

# Function to clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."

    cd "$SHARED_DIR"
    cargo clean

    rm -rf "$OUTPUT_DIR"

    print_success "Clean completed"
}

# Function to run tests
test_build() {
    print_status "Running tests..."

    cd "$SHARED_DIR"
    cargo test --features c-api

    print_success "Tests completed"
}

# Function to display usage
usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  ios        Build for iOS platforms"
    echo "  android    Build for Android platforms"
    echo "  all        Build for all platforms (default)"
    echo "  clean      Clean build artifacts"
    echo "  test       Run tests"
    echo "  setup      Install required Rust targets"
    echo "  help       Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  ANDROID_NDK_HOME   Path to Android NDK (auto-detected if not set)"
    echo ""
    echo "Examples:"
    echo "  $0 ios             # Build only for iOS"
    echo "  $0 android         # Build only for Android"
    echo "  $0 all             # Build for all platforms"
    echo "  $0 clean           # Clean all build artifacts"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    # Check if Rust is installed
    if ! command_exists rustup; then
        print_error "Rust is not installed. Please install from https://rustup.rs/"
        exit 1
    fi

    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust toolchain."
        exit 1
    fi

    # Check if we're in the right directory
    if [ ! -f "$SHARED_DIR/Cargo.toml" ]; then
        print_error "Shared library Cargo.toml not found. Are you in the right directory?"
        exit 1
    fi

    print_success "Prerequisites check passed"
}

# Function to create build summary
create_summary() {
    print_status "Build Summary:"
    echo ""

    if [ -d "$OUTPUT_DIR" ]; then
        echo "Output directory: $OUTPUT_DIR"
        echo ""

        # iOS summary
        if [ -d "$OUTPUT_DIR/ios" ]; then
            echo "iOS Libraries:"
            if [ -f "$OUTPUT_DIR/ios/device/libziplock_shared.a" ]; then
                SIZE=$(du -h "$OUTPUT_DIR/ios/device/libziplock_shared.a" | cut -f1)
                echo "  ✓ Device (ARM64): $SIZE"
            fi
            if [ -f "$OUTPUT_DIR/ios/simulator/libziplock_shared.a" ]; then
                SIZE=$(du -h "$OUTPUT_DIR/ios/simulator/libziplock_shared.a" | cut -f1)
                echo "  ✓ Simulator (Universal): $SIZE"
            fi
            if [ -d "$OUTPUT_DIR/ios/xcframework/ZipLockCore.xcframework" ]; then
                echo "  ✓ XCFramework: Available"
            fi
            echo ""
        fi

        # Android summary
        if [ -d "$OUTPUT_DIR/android" ]; then
            echo "Android Libraries:"
            for arch in arm64-v8a armeabi-v7a x86_64 x86; do
                if [ -f "$OUTPUT_DIR/android/$arch/libziplock_shared.so" ]; then
                    SIZE=$(du -h "$OUTPUT_DIR/android/$arch/libziplock_shared.so" | cut -f1)
                    echo "  ✓ $arch: $SIZE"
                fi
            done
            echo ""
        fi

        echo "Header file: include/ziplock.h"
        echo "Documentation: docs/mobile-integration.md"
    else
        echo "No build artifacts found."
    fi
}

# Main script logic
main() {
    local command="${1:-all}"

    case "$command" in
        "ios")
            check_prerequisites
            install_rust_targets
            build_ios
            create_summary
            ;;
        "android")
            check_prerequisites
            install_rust_targets
            build_android
            create_summary
            ;;
        "all")
            check_prerequisites
            install_rust_targets
            build_ios
            build_android
            create_summary
            ;;
        "clean")
            clean
            ;;
        "test")
            check_prerequisites
            test_build
            ;;
        "setup")
            check_prerequisites
            install_rust_targets
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

# Execute main function with all arguments
main "$@"
