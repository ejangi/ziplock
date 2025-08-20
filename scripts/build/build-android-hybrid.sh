#!/bin/bash

# ZipLock Android Hybrid Architecture Build Script
#
# This script builds the Android app using the new hybrid architecture:
# - Kotlin archive operations (Apache Commons Compress)
# - Rust FFI for data validation, crypto, and business logic
#
# This eliminates the sevenz_rust2 dependency that causes Android emulator crashes.

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SHARED_DIR="$PROJECT_ROOT/shared"
ANDROID_DIR="$PROJECT_ROOT/apps/mobile/android"

# Configuration
BUILD_TYPE="${1:-debug}"
ARCHITECTURES="${2:-arm64-v8a,armeabi-v7a,x86_64,x86}"
CLEAN="${3:-false}"

# Rust targets for Android
declare -A RUST_TARGETS=(
    ["arm64-v8a"]="aarch64-linux-android"
    ["armeabi-v7a"]="armv7-linux-androideabi"
    ["x86_64"]="x86_64-linux-android"
    ["x86"]="i686-linux-android"
)

# NDK configuration
ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_HOME/ndk/25.2.9519653}"
ANDROID_API_LEVEL="${ANDROID_API_LEVEL:-24}"

function log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

function log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

function log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

function log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

function check_dependencies() {
    log "Checking dependencies for hybrid build..."

    # Check Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi

    # Check cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed. Please install Rust toolchain."
        exit 1
    fi

    # Check Android SDK
    if [[ -z "${ANDROID_HOME:-}" ]]; then
        log_error "ANDROID_HOME is not set. Please set it to your Android SDK path."
        exit 1
    fi

    # Check Android NDK
    if [[ ! -d "$ANDROID_NDK_ROOT" ]]; then
        log_error "Android NDK not found at $ANDROID_NDK_ROOT"
        log_error "Please install NDK 25.2.9519653 or set ANDROID_NDK_ROOT"
        exit 1
    fi

    # Check Java
    if ! command -v javac &> /dev/null; then
        log_error "Java is not installed. Please install JDK 11 or higher."
        exit 1
    fi

    # Check Gradle
    if [[ ! -f "$ANDROID_DIR/gradlew" ]]; then
        log_error "Gradle wrapper not found in $ANDROID_DIR"
        exit 1
    fi

    log_success "All dependencies are available"
}

function setup_rust_environment() {
    log "Setting up Rust environment for hybrid build..."

    # Install required Rust targets
    IFS=',' read -ra ARCH_ARRAY <<< "$ARCHITECTURES"
    for arch in "${ARCH_ARRAY[@]}"; do
        rust_target="${RUST_TARGETS[$arch]}"
        log "Installing Rust target: $rust_target"
        rustup target add "$rust_target"
    done

    # Set up cargo config for Android NDK
    mkdir -p "$HOME/.cargo"
    cat > "$HOME/.cargo/config.toml" << EOF
[target.aarch64-linux-android]
ar = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android$ANDROID_API_LEVEL-clang"

[target.armv7-linux-androideabi]
ar = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi$ANDROID_API_LEVEL-clang"

[target.x86_64-linux-android]
ar = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android$ANDROID_API_LEVEL-clang"

[target.i686-linux-android]
ar = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android$ANDROID_API_LEVEL-clang"
EOF

    log_success "Rust environment configured"
}

function build_hybrid_shared_library() {
    log "Building hybrid shared library (Rust FFI only)..."

    cd "$SHARED_DIR"

    # Clean if requested
    if [[ "$CLEAN" == "true" ]]; then
        log "Cleaning previous build artifacts..."
        cargo clean
    fi

    # Build for each architecture
    IFS=',' read -ra ARCH_ARRAY <<< "$ARCHITECTURES"
    for arch in "${ARCH_ARRAY[@]}"; do
        rust_target="${RUST_TARGETS[$arch]}"
        log "Building for architecture: $arch (target: $rust_target)"

        # Set build flags for mobile optimization
        export RUSTFLAGS="-C prefer-dynamic -C link-arg=-Wl,--gc-sections"

        if [[ "$BUILD_TYPE" == "release" ]]; then
            cargo build --release --target "$rust_target" --features "c-api"
            RUST_BUILD_DIR="target/$rust_target/release"
        else
            cargo build --target "$rust_target" --features "c-api"
            RUST_BUILD_DIR="target/$rust_target/debug"
        fi

        # Create output directory
        OUTPUT_DIR="$ANDROID_DIR/app/src/main/jniLibs/$arch"
        mkdir -p "$OUTPUT_DIR"

        # Copy the built library
        cp "$RUST_BUILD_DIR/libziplock_shared.so" "$OUTPUT_DIR/"

        log_success "Built hybrid library for $arch"
    done

    # Copy hybrid header file
    cp "$SHARED_DIR/include/ziplock_hybrid.h" "$ANDROID_DIR/app/src/main/cpp/"

    log_success "Hybrid shared library build completed"
}

function build_android_app() {
    log "Building Android app with hybrid architecture..."

    cd "$ANDROID_DIR"

    # Make gradlew executable
    chmod +x gradlew

    # Clean if requested
    if [[ "$CLEAN" == "true" ]]; then
        log "Cleaning Android build..."
        ./gradlew clean
    fi

    # Build the app
    if [[ "$BUILD_TYPE" == "release" ]]; then
        log "Building release APK..."
        ./gradlew assembleRelease
        APK_PATH="app/build/outputs/apk/release/app-release.apk"
    else
        log "Building debug APK..."
        ./gradlew assembleDebug
        APK_PATH="app/build/outputs/apk/debug/app-debug.apk"
    fi

    if [[ -f "$APK_PATH" ]]; then
        log_success "Android app built successfully: $APK_PATH"

        # Show APK info
        APK_SIZE=$(du -h "$APK_PATH" | cut -f1)
        log "APK size: $APK_SIZE"

        # Show architecture support
        log "Supported architectures:"
        unzip -l "$APK_PATH" | grep "lib/" | grep "\.so$" | awk '{print $4}' | sed 's/lib\///g' | sed 's/\/.*//g' | sort | uniq
    else
        log_error "Failed to build Android app"
        exit 1
    fi
}

function run_tests() {
    log "Running hybrid architecture tests..."

    cd "$ANDROID_DIR"

    # Run unit tests
    log "Running unit tests..."
    ./gradlew testDebugUnitTest

    # Run Rust FFI tests
    log "Running Rust FFI tests..."
    cd "$SHARED_DIR"
    cargo test --features "c-api"

    log_success "All tests passed"
}

function validate_hybrid_architecture() {
    log "Validating hybrid architecture implementation..."

    # Check that sevenz_rust2 is not included
    if grep -r "sevenz_rust2" "$ANDROID_DIR/app/src/" 2>/dev/null; then
        log_warning "Found sevenz_rust2 references in Android code - ensure they are removed"
    else
        log_success "✓ No sevenz_rust2 dependencies found in Android code"
    fi

    # Check that Apache Commons Compress is included
    if grep -q "commons-compress" "$ANDROID_DIR/app/build.gradle"; then
        log_success "✓ Apache Commons Compress dependency found"
    else
        log_error "Apache Commons Compress dependency not found in build.gradle"
        exit 1
    fi

    # Check that hybrid FFI is implemented
    if [[ -f "$SHARED_DIR/src/ffi_hybrid.rs" ]]; then
        log_success "✓ Hybrid FFI implementation found"
    else
        log_error "Hybrid FFI implementation not found"
        exit 1
    fi

    # Check that JNI bridge is implemented
    if [[ -f "$ANDROID_DIR/app/src/main/cpp/ziplock_hybrid_jni.cpp" ]]; then
        log_success "✓ JNI bridge implementation found"
    else
        log_error "JNI bridge implementation not found"
        exit 1
    fi

    # Check that Kotlin archive manager is implemented
    if [[ -f "$ANDROID_DIR/app/src/main/java/com/ziplock/archive/ArchiveManager.kt" ]]; then
        log_success "✓ Kotlin archive manager found"
    else
        log_error "Kotlin archive manager not found"
        exit 1
    fi

    log_success "Hybrid architecture validation completed"
}

function show_usage() {
    echo "Usage: $0 [BUILD_TYPE] [ARCHITECTURES] [CLEAN]"
    echo ""
    echo "Parameters:"
    echo "  BUILD_TYPE     - debug (default) or release"
    echo "  ARCHITECTURES  - Comma-separated list of Android ABIs"
    echo "                   Default: arm64-v8a,armeabi-v7a,x86_64,x86"
    echo "  CLEAN          - true to clean before building, false (default) otherwise"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Debug build, all architectures"
    echo "  $0 release                           # Release build, all architectures"
    echo "  $0 debug arm64-v8a,x86_64           # Debug build, specific architectures"
    echo "  $0 release arm64-v8a true           # Release build, ARM64 only, clean first"
    echo ""
    echo "Environment variables:"
    echo "  ANDROID_HOME      - Path to Android SDK"
    echo "  ANDROID_NDK_ROOT  - Path to Android NDK (default: \$ANDROID_HOME/ndk/25.2.9519653)"
    echo "  ANDROID_API_LEVEL - Android API level (default: 24)"
}

function main() {
    echo ""
    echo "=================================="
    echo "ZipLock Android Hybrid Build"
    echo "=================================="
    echo ""

    if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
        show_usage
        exit 0
    fi

    log "Starting hybrid architecture build..."
    log "Build type: $BUILD_TYPE"
    log "Architectures: $ARCHITECTURES"
    log "Clean build: $CLEAN"
    echo ""

    # Validate build parameters
    if [[ "$BUILD_TYPE" != "debug" && "$BUILD_TYPE" != "release" ]]; then
        log_error "Invalid build type: $BUILD_TYPE. Must be 'debug' or 'release'"
        exit 1
    fi

    # Run build steps
    check_dependencies
    validate_hybrid_architecture
    setup_rust_environment
    build_hybrid_shared_library
    build_android_app

    # Run tests if debug build
    if [[ "$BUILD_TYPE" == "debug" ]]; then
        run_tests
    fi

    echo ""
    log_success "Hybrid architecture build completed successfully!"
    echo ""
    echo "Key benefits of this hybrid approach:"
    echo "  ✓ No more Android emulator crashes (sevenz_rust2 removed)"
    echo "  ✓ Archive operations use stable Kotlin libraries"
    echo "  ✓ Data validation and crypto remain in proven Rust implementation"
    echo "  ✓ Better Android platform integration"
    echo "  ✓ Faster development and testing cycle"
    echo ""

    if [[ "$BUILD_TYPE" == "debug" ]]; then
        echo "Debug APK location: $ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
        echo ""
        echo "To install on device/emulator:"
        echo "  adb install -r $ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
    else
        echo "Release APK location: $ANDROID_DIR/app/build/outputs/apk/release/app-release.apk"
        echo ""
        echo "Remember to sign the APK before distribution!"
    fi
}

# Run main function with all arguments
main "$@"
