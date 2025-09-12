#!/bin/bash

# ZipLock Cross-Platform Desktop Build Script
# Builds ZipLock desktop application for Linux, Windows, and macOS

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default configuration
PROFILE="release"
SKIP_TESTS=false
PLATFORMS=()
PACKAGE_ONLY=false
SIGN_PACKAGES=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
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
    exit 1
}

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Show usage information
show_usage() {
    cat << EOF
ZipLock Cross-Platform Desktop Build Script

Usage: $0 [OPTIONS] [PLATFORMS...]

OPTIONS:
  -h, --help              Show this help message
  -p, --profile PROFILE   Build profile (debug|release) [default: release]
  -t, --skip-tests        Skip running tests
  -P, --package-only      Only create packages (skip building)
  -s, --sign              Sign packages where applicable
  --all                   Build for all supported platforms

PLATFORMS:
  linux                   Build for Linux (x86_64-unknown-linux-gnu)
  windows                 Build for Windows (x86_64-pc-windows-msvc)
  macos                   Build for macOS (x86_64-apple-darwin)

EXAMPLES:
  $0 linux                     # Build Linux version only
  $0 linux windows             # Build Linux and Windows versions
  $0 --all                     # Build for all platforms
  $0 --package-only linux      # Create Linux packages only (skip build)
  $0 --sign --all              # Build and sign packages for all platforms

ENVIRONMENT VARIABLES:
  CARGO_TARGET_DIR            Override cargo target directory
  ZIPLOCK_SIGNING_CERT        Path to code signing certificate (Windows)
  ZIPLOCK_APPLE_ID            Apple ID for macOS notarization
  ZIPLOCK_APP_PASSWORD        App password for macOS notarization

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        -t|--skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        -P|--package-only)
            PACKAGE_ONLY=true
            shift
            ;;
        -s|--sign)
            SIGN_PACKAGES=true
            shift
            ;;
        --all)
            PLATFORMS=(linux windows macos)
            shift
            ;;
        linux|windows|macos)
            PLATFORMS+=("$1")
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            ;;
    esac
done

# Default to current platform if no platforms specified
if [[ ${#PLATFORMS[@]} -eq 0 ]]; then
    case "$(uname -s)" in
        Linux*)     PLATFORMS=(linux) ;;
        Darwin*)    PLATFORMS=(macos) ;;
        CYGWIN*|MINGW32*|MSYS*|MINGW*) PLATFORMS=(windows) ;;
        *)          log_error "Unknown platform: $(uname -s)" ;;
    esac
fi

# Validate profile
if [[ "$PROFILE" != "debug" && "$PROFILE" != "release" ]]; then
    log_error "Profile must be 'debug' or 'release'"
fi

log_info "ZipLock Cross-Platform Desktop Build"
log_info "====================================="
log_info "Profile: $PROFILE"
log_info "Platforms: ${PLATFORMS[*]}"
log_info "Package only: $PACKAGE_ONLY"
log_info "Sign packages: $SIGN_PACKAGES"
echo

# Change to project root
cd "$PROJECT_ROOT"

# Function to check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."

    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
    fi

    if ! command -v rustup &> /dev/null; then
        log_error "Rustup not found. Please install Rust toolchain."
    fi

    # Check targets for each platform
    for platform in "${PLATFORMS[@]}"; do
        case $platform in
            linux)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            windows)
                TARGET="x86_64-pc-windows-msvc"
                # Check for Windows build tools on non-Windows systems
                if [[ "$(uname -s)" != "CYGWIN"* && "$(uname -s)" != "MINGW"* && "$(uname -s)" != "MSYS"* ]]; then
                    log_info "Cross-compiling for Windows from $(uname -s)"
                fi
                ;;
            macos)
                TARGET="x86_64-apple-darwin"
                # Check for macOS build tools on non-macOS systems
                if [[ "$(uname -s)" != "Darwin" ]]; then
                    log_info "Cross-compiling for macOS from $(uname -s)"
                fi
                ;;
        esac

        # Install target if not present
        if ! rustup target list --installed | grep -q "$TARGET"; then
            log_info "Installing Rust target: $TARGET"
            rustup target add "$TARGET"
        fi
    done

    log_success "Prerequisites checked"
}

# Function to run tests
run_tests() {
    if [[ "$SKIP_TESTS" == true ]]; then
        log_warning "Skipping tests"
        return
    fi

    log_step "Running tests..."
    cargo test --workspace --all-features --profile "$PROFILE"
    log_success "Tests completed"
}

# Function to build for a specific platform
build_platform() {
    local platform=$1
    local target=""
    local features=""

    case $platform in
        linux)
            target="x86_64-unknown-linux-gnu"
            features="iced-gui,file-dialog"
            ;;
        windows)
            target="x86_64-pc-windows-msvc"
            features="iced-gui,file-dialog,file-associations"
            ;;
        macos)
            target="x86_64-apple-darwin"
            features="iced-gui,file-dialog,file-associations"
            ;;
        *)
            log_error "Unknown platform: $platform"
            ;;
    esac

    log_step "Building for $platform ($target)..."

    # Build shared library first
    log_info "  Building shared library..."
    cargo build --package ziplock-shared --target "$target" --profile "$PROFILE"

    # Build desktop application
    log_info "  Building desktop application..."
    cargo build --package ziplock-desktop --bin ziplock --target "$target" --profile "$PROFILE" --features "$features"

    # Create platform-specific output directory
    local output_dir="target/desktop-builds/$platform"
    mkdir -p "$output_dir"

    # Copy binary
    case $platform in
        windows)
            cp "target/$target/$PROFILE/ziplock.exe" "$output_dir/"
            ;;
        *)
            cp "target/$target/$PROFILE/ziplock" "$output_dir/"
            ;;
    esac

    log_success "Built $platform successfully"
}

# Function to create packages
create_packages() {
    local platform=$1

    log_step "Creating $platform packages..."

    case $platform in
        linux)
            if [[ -f "packaging/linux/scripts/build-deb.sh" ]]; then
                chmod +x "packaging/linux/scripts/build-deb.sh"
                ./packaging/linux/scripts/build-deb.sh
            else
                log_warning "Linux packaging script not found, using cargo-deb"
                if command -v cargo-deb &> /dev/null; then
                    cd apps/desktop
                    cargo deb --profile "$PROFILE"
                    cd ../..
                else
                    log_warning "cargo-deb not installed, skipping Debian package creation"
                fi
            fi
            ;;
        windows)
            if [[ -f "packaging/windows/scripts/build-windows.ps1" ]]; then
                if command -v pwsh &> /dev/null; then
                    local sign_arg=""
                    if [[ "$SIGN_PACKAGES" == true && -n "${ZIPLOCK_SIGNING_CERT:-}" ]]; then
                        sign_arg="-Sign -SigningCert '$ZIPLOCK_SIGNING_CERT'"
                    fi
                    pwsh -File "packaging/windows/scripts/build-windows.ps1" -CreateMsi $sign_arg
                else
                    log_warning "PowerShell not available, skipping Windows MSI creation"
                fi
            else
                log_warning "Windows packaging script not found"
            fi
            ;;
        macos)
            if [[ -f "packaging/macos/scripts/create-app-bundle.sh" ]]; then
                chmod +x "packaging/macos/scripts/create-app-bundle.sh"
                local sign_args=""
                if [[ "$SIGN_PACKAGES" == true ]]; then
                    sign_args="--sign"
                    if [[ -n "${ZIPLOCK_SIGNING_IDENTITY:-}" ]]; then
                        sign_args="$sign_args --identity '$ZIPLOCK_SIGNING_IDENTITY'"
                    fi
                    if [[ -n "${ZIPLOCK_APPLE_ID:-}" && -n "${ZIPLOCK_APP_PASSWORD:-}" ]]; then
                        sign_args="$sign_args --notarize --apple-id '$ZIPLOCK_APPLE_ID' --app-password '$ZIPLOCK_APP_PASSWORD'"
                    fi
                fi
                ./packaging/macos/scripts/create-app-bundle.sh --config "$PROFILE" $sign_args
            else
                log_warning "macOS packaging script not found"
            fi
            ;;
    esac

    log_success "Created $platform packages"
}

# Main execution flow
main() {
    check_prerequisites

    if [[ "$PACKAGE_ONLY" == false ]]; then
        run_tests

        # Build for each platform
        for platform in "${PLATFORMS[@]}"; do
            build_platform "$platform"
        done
    fi

    # Create packages for each platform
    for platform in "${PLATFORMS[@]}"; do
        create_packages "$platform"
    done

    echo
    log_success "Cross-platform desktop build completed!"
    log_info "Platforms built: ${PLATFORMS[*]}"
    log_info "Profile: $PROFILE"

    echo
    log_info "Build artifacts:"
    for platform in "${PLATFORMS[@]}"; do
        echo "  $platform: target/desktop-builds/$platform/"
    done

    if [[ "$PACKAGE_ONLY" == false ]]; then
        echo
        log_info "Next steps:"
        echo "  • Test binaries: ./target/desktop-builds/<platform>/ziplock"
        echo "  • Install packages: See packaging/<platform>/ directories"
        echo "  • Development: ./scripts/dev/run-desktop.sh"
    fi
}

# Execute main function
main "$@"
