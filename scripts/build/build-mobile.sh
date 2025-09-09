#!/bin/bash
set -e

# ZipLock Mobile Build Script - Unified Architecture
# This script builds the mobile FFI library for Android and iOS platforms

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SHARED_DIR="$PROJECT_ROOT/shared"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Default values
PLATFORM=""
PROFILE="release"
VERBOSE=false
CLEAN=false
TARGET_DIR="$PROJECT_ROOT/target"
ANDROID_OUTPUT_DIR="$PROJECT_ROOT/target/android"
OUTPUT_DIR="$PROJECT_ROOT/target/build"

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build ZipLock mobile FFI library for Android and iOS platforms.

OPTIONS:
    -p, --platform PLATFORM    Target platform: android, ios, or all (default: all)
    -d, --debug                 Build debug profile (default: release)
    -c, --clean                 Clean build artifacts before building
    -v, --verbose               Enable verbose output
    -o, --output DIR           Output directory (default: $OUTPUT_DIR)

ANDROID BUILD LOCATIONS:
    Native libs are built to: $PROJECT_ROOT/target/android/jniLibs/
    Then copied to Android client: apps/mobile/android/app/src/main/jniLibs/
    -h, --help                 Show this help message

EXAMPLES:
    $0                         # Build for all platforms (release)
    $0 -p android              # Build for Android only
    $0 -p ios -d               # Build for iOS debug
    $0 -c -v                   # Clean build with verbose output

PLATFORMS:
    android    Build Android AAR with native libraries
    ios        Build iOS XCFramework with native libraries
    all        Build for all mobile platforms

The script will:
1. Build Rust mobile FFI library for target architectures
2. Place Android libraries in target/android/jniLibs/ by architecture
3. Copy libraries to Android client jniLibs folder
4. Package native libraries for platform consumption
5. Copy outputs to mobile-builds/ directory

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -d|--debug)
            PROFILE="debug"
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Set default platform if not specified
if [ -z "$PLATFORM" ]; then
    PLATFORM="all"
fi

# Validate platform
case $PLATFORM in
    android|ios|all)
        ;;
    *)
        log_error "Invalid platform: $PLATFORM. Must be 'android', 'ios', or 'all'"
        exit 1
        ;;
esac

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    log_error "Cargo is not installed. Please install Rust and Cargo."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "$SHARED_DIR/Cargo.toml" ]; then
    log_error "Cannot find shared/Cargo.toml. Please run this script from the project root."
    exit 1
fi

# Create output directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$ANDROID_OUTPUT_DIR"

log_info "Starting ZipLock mobile build"
log_info "Platform: $PLATFORM"
log_info "Profile: $PROFILE"
log_info "Output directory: $OUTPUT_DIR"

# Clean if requested
if [ "$CLEAN" = true ]; then
    log_info "Cleaning build artifacts..."
    cd "$SHARED_DIR"
    cargo clean
    rm -rf "$OUTPUT_DIR"/*
    rm -rf "$ANDROID_OUTPUT_DIR"/*
    mkdir -p "$OUTPUT_DIR"
    mkdir -p "$ANDROID_OUTPUT_DIR"
    log_success "Clean completed"
fi

# Build flags
BUILD_FLAGS=""
if [ "$PROFILE" = "release" ]; then
    BUILD_FLAGS="--release"
fi

if [ "$VERBOSE" = true ]; then
    BUILD_FLAGS="$BUILD_FLAGS --verbose"
fi

# Function to build for Android
build_android() {
    log_info "Building for Android..."

    # Check for Android NDK
    if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$NDK_HOME" ]; then
        log_warning "ANDROID_NDK_HOME not set. Trying to find NDK..."
        # Try common locations
        for ndk_path in "$HOME/Android/Sdk/ndk"/* "/usr/local/lib/android/sdk/ndk"/* "/opt/android-ndk"*; do
            if [ -d "$ndk_path" ]; then
                export ANDROID_NDK_HOME="$ndk_path"
                log_info "Found NDK at: $ANDROID_NDK_HOME"
                break
            fi
        done

        if [ -z "$ANDROID_NDK_HOME" ]; then
            log_error "Android NDK not found. Please set ANDROID_NDK_HOME or install Android NDK."
            return 1
        fi
    fi

    # Android target architectures
    ANDROID_TARGETS=(
        "aarch64-linux-android"    # ARM64
        "armv7-linux-androideabi"  # ARM7
        "i686-linux-android"       # x86
        "x86_64-linux-android"     # x86_64
    )

    # Install targets if needed
    for target in "${ANDROID_TARGETS[@]}"; do
        if ! rustup target list --installed | grep -q "$target"; then
            log_info "Installing Rust target: $target"
            rustup target add "$target"
        fi
    done

    cd "$SHARED_DIR"

    # Build for each Android architecture
    for target in "${ANDROID_TARGETS[@]}"; do
        log_info "Building for Android target: $target"

        # Set up environment for cross-compilation with libgcc fix
        case $target in
            aarch64-linux-android)
                export CC_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
                export AR_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
                export CFLAGS_aarch64_linux_android="-static-libgcc"
                export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
                arch_dir="arm64-v8a"
                ;;
            armv7-linux-androideabi)
                export CC_armv7_linux_androideabi="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang"
                export AR_armv7_linux_androideabi="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
                export CFLAGS_armv7_linux_androideabi="-static-libgcc"
                export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang"
                arch_dir="armeabi-v7a"
                ;;
            i686-linux-android)
                export CC_i686_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android21-clang"
                export AR_i686_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
                export CFLAGS_i686_linux_android="-static-libgcc"
                export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android21-clang"
                arch_dir="x86"
                ;;
            x86_64-linux-android)
                export CC_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"
                export AR_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
                export CFLAGS_x86_64_linux_android="-static-libgcc"
                export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"
                arch_dir="x86_64"
                ;;
        esac

        # Set additional environment variables to force static linking and avoid libgcc_s.so.1
        export RUSTFLAGS="-C link-arg=-static-libgcc -C link-arg=-Wl,--as-needed"

        # Build the library with Android-specific configuration
        log_info "Building with static libgcc to avoid libgcc_s.so.1 dependency"
        RUSTFLAGS="$RUSTFLAGS -C target-feature=+crt-static" cargo build --target "$target" $BUILD_FLAGS --lib

        # Copy the built library to the target/android directory
        android_lib_dir="$ANDROID_OUTPUT_DIR/jniLibs/$arch_dir"
        mkdir -p "$android_lib_dir"

        if [ "$PROFILE" = "release" ]; then
            cp "$TARGET_DIR/$target/release/libziplock_shared.so" "$android_lib_dir/"
        else
            cp "$TARGET_DIR/$target/debug/libziplock_shared.so" "$android_lib_dir/"
        fi

        # Also copy to the legacy output directory for compatibility
        legacy_lib_dir="$OUTPUT_DIR/android/jniLibs/$arch_dir"
        mkdir -p "$legacy_lib_dir"
        cp "$android_lib_dir/libziplock_shared.so" "$legacy_lib_dir/"

        log_success "Built for Android target: $target -> $arch_dir"
    done

    # Copy to Android app directory
    ANDROID_APP_JNILIBS="$PROJECT_ROOT/apps/mobile/android/app/src/main/jniLibs"
    if [ -d "$PROJECT_ROOT/apps/mobile/android" ]; then
        log_info "Copying libraries to Android client jniLibs..."

        # Ensure the jniLibs directory exists
        mkdir -p "$ANDROID_APP_JNILIBS"

        # Copy each architecture's libraries
        for arch in arm64-v8a armeabi-v7a x86 x86_64; do
            if [ -f "$ANDROID_OUTPUT_DIR/jniLibs/$arch/libziplock_shared.so" ]; then
                mkdir -p "$ANDROID_APP_JNILIBS/$arch"
                cp "$ANDROID_OUTPUT_DIR/jniLibs/$arch/libziplock_shared.so" "$ANDROID_APP_JNILIBS/$arch/"
                log_success "Copied $arch library to Android client"
            fi
        done

        log_success "All libraries copied to Android client"
    else
        log_warning "Android client directory not found at $PROJECT_ROOT/apps/mobile/android"
        log_info "Libraries available in: $ANDROID_OUTPUT_DIR/jniLibs/"
    fi

    log_success "Android build completed"
}

# Function to build for iOS
build_ios() {
    log_info "Building for iOS..."

    # Check if we're on macOS
    if [ "$(uname)" != "Darwin" ]; then
        log_warning "iOS builds are only supported on macOS. Skipping iOS build."
        return 0
    fi

    # Check for Xcode command line tools
    if ! command -v xcodebuild &> /dev/null; then
        log_error "Xcode command line tools not found. Please install Xcode."
        return 1
    fi

    # iOS target architectures
    IOS_TARGETS=(
        "aarch64-apple-ios"        # iOS ARM64 (device)
        "x86_64-apple-ios"         # iOS x86_64 (simulator)
        "aarch64-apple-ios-sim"    # iOS ARM64 (simulator)
    )

    # Install targets if needed
    for target in "${IOS_TARGETS[@]}"; do
        if ! rustup target list --installed | grep -q "$target"; then
            log_info "Installing Rust target: $target"
            rustup target add "$target"
        fi
    done

    cd "$SHARED_DIR"

    # Build for each iOS architecture
    IOS_LIBS=()
    for target in "${IOS_TARGETS[@]}"; do
        log_info "Building for iOS target: $target"

        cargo build --target "$target" $BUILD_FLAGS --lib

        if [ "$PROFILE" = "release" ]; then
            IOS_LIBS+=("$TARGET_DIR/$target/release/libziplock_shared.a")
        else
            IOS_LIBS+=("$TARGET_DIR/$target/debug/libziplock_shared.a")
        fi

        log_success "Built for iOS target: $target"
    done

    # Create universal library using lipo
    IOS_OUTPUT_DIR="$OUTPUT_DIR/ios"
    mkdir -p "$IOS_OUTPUT_DIR"

    log_info "Creating universal iOS library..."
    lipo -create "${IOS_LIBS[@]}" -output "$IOS_OUTPUT_DIR/libziplock_shared.a"

    # Copy to iOS app directory if it exists
    IOS_APP_DIR="$PROJECT_ROOT/apps/mobile/ios"
    if [ -d "$IOS_APP_DIR" ]; then
        log_info "Copying library to iOS app directory..."
        cp "$IOS_OUTPUT_DIR/libziplock_shared.a" "$IOS_APP_DIR/"
        log_success "Library copied to iOS app"
    fi

    log_success "iOS build completed"
}

# Main build logic
cd "$SHARED_DIR"

case $PLATFORM in
    android)
        build_android
        ;;
    ios)
        build_ios
        ;;
    all)
        build_android
        build_ios
        ;;
esac

# Generate build info
BUILD_INFO_FILE="$OUTPUT_DIR/build-info.json"
cat > "$BUILD_INFO_FILE" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "platform": "$PLATFORM",
  "profile": "$PROFILE",
  "rust_version": "$(rustc --version)",
  "git_commit": "$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')"
}
EOF

log_success "Build completed successfully!"
log_info "Output directory: $OUTPUT_DIR"
log_info "Build info: $BUILD_INFO_FILE"

# Show what was built
echo
log_info "Built artifacts:"
find "$OUTPUT_DIR" -name "*.so" -o -name "*.a" | while read -r lib; do
    echo "  $lib"
done
