#!/bin/bash

# ZipLock Android Docker Build Script
# Builds Android libraries using Docker containers with pre-configured NDK and toolchains

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Configuration
REGISTRY="${REGISTRY:-ghcr.io}"
IMAGE_NAME="${IMAGE_NAME:-ejangi/ziplock/android-builder}"
USE_REGISTRY="${USE_REGISTRY:-true}"
DOCKER_IMAGE="$REGISTRY/$IMAGE_NAME:latest"
LOCAL_IMAGE="ziplock-android-builder:local"

# Logging functions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

show_usage() {
    cat << EOF
Usage: $0 <command> [options]

Build Android libraries using Docker containers with pre-configured Android NDK.

COMMANDS:
    build [arch]       Build libraries (all architectures or specific: arm64, arm7, x86, x86_64)
    test              Run basic library tests
    clean             Clean build artifacts
    shell             Open interactive shell in Android builder container
    verify            Verify build environment and tools
    pull              Pull latest Android builder image

ENVIRONMENT VARIABLES:
    USE_REGISTRY   Use registry image if 'true', build local if 'false' (default: true)
    REGISTRY       Container registry to use (default: ghcr.io)
    IMAGE_NAME     Image name to use (default: ejangi/ziplock/android-builder)

Examples:
    $0 build                    # Build all Android architectures
    $0 build arm64              # Build only ARM64
    $0 test                     # Test built libraries
    $0 shell                    # Open interactive shell
    $0 verify                   # Verify build environment

Environment Setup:
    # Use registry image (default)
    $0 build

    # Force local image build
    USE_REGISTRY=false $0 build

    # Use custom registry
    REGISTRY=my-registry.com $0 build
EOF
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker is not running or not accessible"
        exit 1
    fi
}

# Pull or build Docker image
setup_image() {
    local image_to_use=""

    if [ "$USE_REGISTRY" = "true" ]; then
        log_info "Using registry image: $DOCKER_IMAGE" >&2

        # Try to pull the latest image
        if docker pull "$DOCKER_IMAGE" >&2 2>/dev/null; then
            log_success "Successfully pulled $DOCKER_IMAGE" >&2
            image_to_use="$DOCKER_IMAGE"
        else
            log_warning "Failed to pull registry image, falling back to local build" >&2
            USE_REGISTRY="false"
        fi
    fi

    if [ "$USE_REGISTRY" = "false" ]; then
        log_info "Building local Android builder image..." >&2

        # Build local image from Dockerfile
        if docker build -f ".github/docker/android-builder.Dockerfile" -t "$LOCAL_IMAGE" . >&2; then
            log_success "Successfully built local image: $LOCAL_IMAGE" >&2
            image_to_use="$LOCAL_IMAGE"
        else
            log_error "Failed to build local Android builder image" >&2
            exit 1
        fi
    fi

    echo "$image_to_use"
}

# Run Docker container with proper volume mounts
run_docker() {
    local image="$1"
    local cmd="$2"

    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        -u root \
        -e CARGO_TARGET_DIR=/workspace/target \
        -e RUSTFLAGS="-C link-arg=-static-libgcc -C link-arg=-Wl,--as-needed" \
        "$image" \
        bash -c "$cmd"
}

# Build Android libraries
build_android() {
    local target_arch="${1:-all}"
    local image

    log_info "Starting Android build process..."
    log_info "Target architecture: $target_arch"

    image=$(setup_image)

    # Map architecture names to Rust targets
    local targets=()
    case "$target_arch" in
        "all")
            targets=("aarch64-linux-android" "armv7-linux-androideabi" "x86_64-linux-android" "i686-linux-android")
            ;;
        "arm64")
            targets=("aarch64-linux-android")
            ;;
        "arm7"|"armv7")
            targets=("armv7-linux-androideabi")
            ;;
        "x86_64")
            targets=("x86_64-linux-android")
            ;;
        "x86")
            targets=("i686-linux-android")
            ;;
        *)
            log_error "Invalid architecture: $target_arch"
            log_error "Valid options: all, arm64, arm7, x86_64, x86"
            exit 1
            ;;
    esac

    # Create output directories
    log_info "Creating output directories..."
    run_docker "$image" "mkdir -p /workspace/target/android/jniLibs/{arm64-v8a,armeabi-v7a,x86_64,x86}"

    # Build for each target
    for target in "${targets[@]}"; do
        log_info "Building for target: $target"

        # Map target to Android architecture directory
        local arch_dir=""
        case "$target" in
            "aarch64-linux-android")
                arch_dir="arm64-v8a"
                ;;
            "armv7-linux-androideabi")
                arch_dir="armeabi-v7a"
                ;;
            "x86_64-linux-android")
                arch_dir="x86_64"
                ;;
            "i686-linux-android")
                arch_dir="x86"
                ;;
        esac

        # Build command
        local build_cmd="cd /workspace/shared && \
            RUSTFLAGS=\"-C link-arg=-static-libgcc -C link-arg=-Wl,--as-needed\" \
            cargo build --release --target $target --lib --features c-api && \
            cp /workspace/target/$target/release/libziplock_shared.so /workspace/target/android/jniLibs/$arch_dir/"

        if ! run_docker "$image" "$build_cmd"; then
            log_error "Build failed for target: $target"
            exit 1
        fi

        log_success "Successfully built for $target -> $arch_dir"
    done

    # Copy to Android app directory if it exists
    if [ -d "$PROJECT_ROOT/apps/mobile/android" ]; then
        log_info "Copying libraries to Android app jniLibs..."
        local copy_cmd="mkdir -p /workspace/apps/mobile/android/app/src/main/jniLibs && \
            cp -r /workspace/target/android/jniLibs/* /workspace/apps/mobile/android/app/src/main/jniLibs/"

        run_docker "$image" "$copy_cmd"
        log_success "Libraries copied to Android app"
    else
        log_warning "Android app directory not found at apps/mobile/android"
    fi

    # Generate header file
    log_info "Generating C header file..."
    local header_cmd="cd /workspace/shared && \
        cbindgen --config cbindgen.toml --crate ziplock-shared --output /workspace/target/android/ziplock.h || \
        echo '// Header generation requires cbindgen - install with: cargo install cbindgen' > /workspace/target/android/ziplock.h"

    run_docker "$image" "$header_cmd"

    # Create build info
    local info_cmd="cat > /workspace/target/android/build-info.json << 'EOL'
{
  \"timestamp\": \"$(date -u +\"%Y-%m-%dT%H:%M:%SZ\")\",
  \"target_arch\": \"$target_arch\",
  \"rust_version\": \"$(rustc --version)\",
  \"targets_built\": [$(printf '\"%s\",' ${targets[@]} | sed 's/,$//')]
}
EOL"

    run_docker "$image" "$info_cmd"

    log_success "Android build completed successfully!"
    log_info "Output directory: $PROJECT_ROOT/target/android/"
}

# Test Android libraries
test_android() {
    log_info "Testing Android libraries..."

    local image
    image=$(setup_image)

    # Check if libraries exist and have correct symbols
    local test_cmd="
        cd /workspace && \
        echo 'Testing Android library outputs...' && \
        for arch in arm64-v8a armeabi-v7a x86_64 x86; do
            lib_path=\"target/android/jniLibs/\$arch/libziplock_shared.so\"
            if [ -f \"\$lib_path\" ]; then
                echo \"✓ Found library: \$arch\"
                echo \"  Size: \$(stat -c%s \"\$lib_path\") bytes\"
                echo \"  Type: \$(file \"\$lib_path\" | cut -d: -f2-)\"

                # Check for key symbols
                if readelf -s \"\$lib_path\" | grep -q \"ziplock_\"; then
                    echo \"  ✓ Contains ziplock symbols\"
                else
                    echo \"  ⚠ No ziplock symbols found\"
                fi
            else
                echo \"✗ Missing library: \$arch\"
            fi
            echo
        done

        # Test header file
        if [ -f \"target/android/ziplock.h\" ]; then
            echo \"✓ Header file exists\"
            echo \"  Functions: \$(grep -c '^[[:space:]]*[a-zA-Z].*(' target/android/ziplock.h || echo 0)\"
        else
            echo \"✗ Header file missing\"
        fi
    "

    run_docker "$image" "$test_cmd"
    log_success "Android library testing completed"
}

# Clean build artifacts
clean_android() {
    log_info "Cleaning Android build artifacts..."

    rm -rf "$PROJECT_ROOT/target/android"
    rm -rf "$PROJECT_ROOT/target/aarch64-linux-android"
    rm -rf "$PROJECT_ROOT/target/armv7-linux-androideabi"
    rm -rf "$PROJECT_ROOT/target/x86_64-linux-android"
    rm -rf "$PROJECT_ROOT/target/i686-linux-android"

    if [ -d "$PROJECT_ROOT/apps/mobile/android/app/src/main/jniLibs" ]; then
        rm -rf "$PROJECT_ROOT/apps/mobile/android/app/src/main/jniLibs"
    fi

    log_success "Android build artifacts cleaned"
}

# Open interactive shell
open_shell() {
    log_info "Opening interactive shell in Android builder container..."

    local image
    image=$(setup_image)

    docker run -it --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        -u root \
        -e CARGO_TARGET_DIR=/workspace/target \
        "$image" \
        bash
}

# Verify build environment
verify_environment() {
    log_info "Verifying Android build environment..."

    local image
    image=$(setup_image)

    local verify_cmd="
        echo '=== Android Builder Environment Verification ==='
        echo
        echo 'System Information:'
        uname -a
        echo
        echo 'Rust Toolchain:'
        rustc --version
        cargo --version
        echo
        echo 'Android NDK:'
        echo \"NDK_HOME: \$ANDROID_NDK_HOME\"
        ls -la \$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/ | head -5
        echo
        echo 'Android Targets:'
        rustup target list --installed | grep android
        echo
        echo 'Cross-compilation Tools:'
        aarch64-linux-android21-clang --version | head -1
        armv7a-linux-androideabi21-clang --version | head -1
        x86_64-linux-android21-clang --version | head -1
        i686-linux-android21-clang --version | head -1
        echo
        echo 'Testing compilation for ARM64:'
        cd /workspace/shared
        cargo check --target aarch64-linux-android --features c-api
        echo
        echo '✓ Android build environment is ready!'
    "

    run_docker "$image" "$verify_cmd"
}

# Pull latest image
pull_image() {
    log_info "Pulling latest Android builder image..."

    if docker pull "$DOCKER_IMAGE"; then
        log_success "Successfully pulled $DOCKER_IMAGE"
    else
        log_error "Failed to pull $DOCKER_IMAGE"
        exit 1
    fi
}

# Main script logic
main() {
    check_docker

    if [ $# -eq 0 ]; then
        show_usage
        exit 1
    fi

    local command="$1"
    shift

    case "$command" in
        "build")
            build_android "$@"
            ;;
        "test")
            test_android
            ;;
        "clean")
            clean_android
            ;;
        "shell")
            open_shell
            ;;
        "verify")
            verify_environment
            ;;
        "pull")
            pull_image
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            echo
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
