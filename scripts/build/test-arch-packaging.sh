#!/bin/bash
set -euo pipefail

# ZipLock Arch Linux Packaging Test Script
# Tests the Arch packaging process in a containerized environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

check_dependencies() {
    log_info "Checking test dependencies..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is required for testing Arch packaging"
        exit 1
    fi

    log_success "Dependencies satisfied"
}

cleanup() {
    log_info "Cleaning up test environment..."

    # Remove test containers
    docker rm -f ziplock-arch-test 2>/dev/null || true

    # Remove test images if requested
    if [ "${CLEAN_IMAGES:-false}" = "true" ]; then
        docker rmi ziplock-arch-test-builder 2>/dev/null || true
    fi

    log_success "Cleanup completed"
}

# Set up cleanup trap
trap cleanup EXIT

create_test_dockerfile() {
    log_info "Creating Arch Linux test environment..."

    cat > "$BUILD_DIR/Dockerfile.arch-test" << 'EOF'
FROM archlinux:latest

# Update system and install dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
      base-devel \
      rust \
      cargo \
      pkg-config \
      fontconfig \
      freetype2 \
      libx11 \
      libxft \
      xz \
      gtk4 \
      libadwaita \
      git \
      curl \
      file \
      fakeroot \
      namcap \
      rsync \
      && pacman -Scc --noconfirm

# Create non-root user for makepkg
RUN useradd -m -G wheel builder && \
    echo '%wheel ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# Set up directories
RUN mkdir -p /workspace && chown builder:builder /workspace

USER builder
WORKDIR /workspace

# Verify environment
RUN echo "=== Arch Test Environment ===" && \
    echo "OS Release:" && cat /etc/os-release && \
    echo "Rust version:" && rustc --version && \
    echo "Cargo version:" && cargo --version && \
    echo "makepkg version:" && makepkg --version && \
    echo "=================================="
EOF

    log_success "Test Dockerfile created"
}

build_test_image() {
    log_info "Building Arch Linux test image..."

    cd "$BUILD_DIR"
    docker build -f Dockerfile.arch-test -t ziplock-arch-test-builder .

    log_success "Test image built successfully"
}

test_source_archive_creation() {
    log_info "Testing source archive creation..."

    # First, ensure we have built the project
    if [ ! -d "$BUILD_DIR/install" ]; then
        log_warning "Build artifacts not found, building project first..."
        cd "$PROJECT_ROOT"
        ./scripts/build/build-linux.sh --profile release
    fi

    # Test source archive creation
    cd "$PROJECT_ROOT"
    docker run --rm \
        -v "$PWD:/workspace" \
        -w /workspace \
        ziplock-arch-test-builder \
        bash -c "
            set -euo pipefail
            echo 'Testing source archive creation...'

            # Run the arch packaging script in source-only mode
            ./scripts/build/package-arch.sh --source-only

            # Verify source archive was created
            if [ ! -f target/ziplock-*.tar.gz ]; then
                echo 'ERROR: Source archive not created'
                exit 1
            fi

            # Verify SHA256 file was created
            if [ ! -f target/archive.sha256 ]; then
                echo 'ERROR: SHA256 checksum file not created'
                exit 1
            fi

            echo 'Source archive creation: PASSED'
        "

    log_success "Source archive creation test passed"
}

test_pkgbuild_validity() {
    log_info "Testing PKGBUILD validity..."

    docker run --rm \
        -v "$PWD:/workspace" \
        -w /workspace \
        ziplock-arch-test-builder \
        bash -c "
            set -euo pipefail
            echo 'Testing PKGBUILD validity...'

            # Check if PKGBUILD exists
            if [ ! -f packaging/arch/PKGBUILD ]; then
                echo 'ERROR: PKGBUILD not found'
                exit 1
            fi

            # Check if install script exists
            if [ ! -f packaging/arch/ziplock.install ]; then
                echo 'ERROR: ziplock.install not found'
                exit 1
            fi

            # Validate PKGBUILD syntax
            cd packaging/arch
            if ! bash -n PKGBUILD; then
                echo 'ERROR: PKGBUILD has syntax errors'
                exit 1
            fi

            # Check required PKGBUILD variables
            source PKGBUILD

            if [ -z \"\${pkgname:-}\" ]; then
                echo 'ERROR: pkgname not defined in PKGBUILD'
                exit 1
            fi

            if [ -z \"\${pkgver:-}\" ]; then
                echo 'ERROR: pkgver not defined in PKGBUILD'
                exit 1
            fi

            if [ -z \"\${pkgdesc:-}\" ]; then
                echo 'ERROR: pkgdesc not defined in PKGBUILD'
                exit 1
            fi

            # Use namcap to check PKGBUILD
            namcap PKGBUILD || echo 'namcap warnings (non-fatal)'

            echo 'PKGBUILD validity: PASSED'
        "

    log_success "PKGBUILD validity test passed"
}

test_package_metadata() {
    log_info "Testing package metadata consistency..."

    # Get version from Cargo.toml
    local cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')

    # Check PKGBUILD version
    local pkgbuild_version=$(grep '^pkgver=' "$PROJECT_ROOT/packaging/arch/PKGBUILD" | sed 's/pkgver=//')

    if [ "$cargo_version" != "$pkgbuild_version" ]; then
        log_error "Version mismatch: Cargo.toml ($cargo_version) vs PKGBUILD ($pkgbuild_version)"
        log_error "PKGBUILD pkgver must match the workspace version in Cargo.toml"
        return 1
    else
        log_success "Version consistency check passed: $cargo_version"
    fi
}

test_pkgbuild_version_and_checksum() {
    log_info "Testing PKGBUILD version and SHA256 checksum accuracy..."

    # Get version from Cargo.toml
    local cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')

    # Check PKGBUILD version
    local pkgbuild_version=$(grep '^pkgver=' "$PROJECT_ROOT/packaging/arch/PKGBUILD" | sed 's/pkgver=//')

    # Check SHA256 in PKGBUILD
    local pkgbuild_sha=$(grep '^sha256sums=' "$PROJECT_ROOT/packaging/arch/PKGBUILD" | sed "s/sha256sums=('\\(.*\\)')/\\1/")

    log_info "Cargo.toml version: $cargo_version"
    log_info "PKGBUILD version: $pkgbuild_version"
    log_info "PKGBUILD SHA256: $pkgbuild_sha"

    # Test 1: Version must match
    if [ "$cargo_version" != "$pkgbuild_version" ]; then
        log_error "PKGBUILD version ($pkgbuild_version) does not match Cargo.toml version ($cargo_version)"
        return 1
    fi

    # Test 2: SHA256 must not be 'SKIP'
    if [ "$pkgbuild_sha" = "SKIP" ]; then
        log_error "PKGBUILD sha256sums is still set to 'SKIP' - must be a real checksum"
        return 1
    fi

    # Test 3: SHA256 must be valid format (64 hex characters)
    if ! echo "$pkgbuild_sha" | grep -qE '^[a-f0-9]{64}$'; then
        log_error "PKGBUILD sha256sums is not a valid SHA256 hash: $pkgbuild_sha"
        return 1
    fi

    # Test 4: If we have a source archive, verify the checksum matches
    local archive_file="$BUILD_DIR/ziplock-${cargo_version}.tar.gz"
    if [ -f "$archive_file" ]; then
        local actual_sha=$(sha256sum "$archive_file" | cut -d' ' -f1)
        log_info "Actual archive SHA256: $actual_sha"

        if [ "$pkgbuild_sha" != "$actual_sha" ]; then
            log_error "PKGBUILD SHA256 ($pkgbuild_sha) does not match actual archive SHA256 ($actual_sha)"
            log_error "Run './scripts/build/package-arch.sh --source-only' to generate correct checksum"
            return 1
        fi

        log_success "SHA256 checksum matches actual archive"
    else
        log_warning "Source archive not found at $archive_file - cannot verify checksum accuracy"
        log_info "Consider running './scripts/build/package-arch.sh --source-only' first"
    fi

    log_success "PKGBUILD version and checksum validation passed"
}

test_pkgbuild_source_url() {
    log_info "Testing PKGBUILD source URL consistency..."

    local cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')

    # Extract source URL from PKGBUILD
    local source_line=$(grep '^source=' "$PROJECT_ROOT/packaging/arch/PKGBUILD")

    # Check if source URL contains the correct version (either literal or variable)
    if echo "$source_line" | grep -q "v$cargo_version.tar.gz" || echo "$source_line" | grep -q 'v$pkgver.tar.gz'; then
        log_success "Source URL contains correct version reference"
    else
        log_error "Source URL does not contain correct version v$cargo_version or variable reference"
        log_error "Current source line: $source_line"
        return 1
    fi

    # Check URL format
    if echo "$source_line" | grep -qE 'github\.com/[^/]+/ziplock/archive/v.*\.tar\.gz'; then
        log_success "Source URL format is correct"
    else
        log_error "Source URL format appears incorrect"
        log_error "Expected format: https://github.com/USER/ziplock/archive/vVERSION.tar.gz"
        return 1
    fi
}

run_standalone_validation() {
    log_info "Running standalone PKGBUILD validation..."

    # Check if the standalone validation script exists
    local validation_script="$PROJECT_ROOT/scripts/build/test-pkgbuild-validation.sh"
    if [ ! -f "$validation_script" ]; then
        log_warning "Standalone validation script not found: $validation_script"
        return 0
    fi

    # Run the standalone validation script
    if "$validation_script"; then
        log_success "Standalone PKGBUILD validation passed"
    else
        log_error "Standalone PKGBUILD validation failed"
        log_info "Run '$validation_script' for detailed error information"
        return 1
    fi
}

test_dependencies() {
    log_info "Testing package dependencies..."

    docker run --rm \
        -v "$PWD:/workspace" \
        -w /workspace \
        ziplock-arch-test-builder \
        bash -c "
            set -euo pipefail
            echo 'Testing package dependencies...'

            cd packaging/arch
            source PKGBUILD

            echo 'Checking if dependencies are available...'
            for dep in \"\${depends[@]}\"; do
                # Remove version constraints for check
                dep_name=\$(echo \"\$dep\" | sed 's/[<>=].*//')
                if ! pacman -Si \"\$dep_name\" >/dev/null 2>&1; then
                    echo \"WARNING: Dependency \$dep_name not found in repositories\"
                else
                    echo \"✓ \$dep_name\"
                fi
            done

            echo 'Checking if makedepends are available...'
            for dep in \"\${makedepends[@]}\"; do
                dep_name=\$(echo \"\$dep\" | sed 's/[<>=].*//')
                if ! pacman -Si \"\$dep_name\" >/dev/null 2>&1; then
                    echo \"WARNING: Build dependency \$dep_name not found in repositories\"
                else
                    echo \"✓ \$dep_name\"
                fi
            done

            echo 'Dependency check: COMPLETED'
        "

    log_success "Dependency check completed"
}

run_full_build_test() {
    log_info "Running full build test (this may take several minutes)..."

    # Only run if explicitly requested
    if [ "${FULL_BUILD_TEST:-false}" != "true" ]; then
        log_warning "Skipping full build test (use --full-build to enable)"
        return 0
    fi

    docker run --rm \
        -v "$PWD:/workspace" \
        -w /workspace \
        ziplock-arch-test-builder \
        bash -c "
            set -euo pipefail
            echo 'Running full PKGBUILD build test...'

            # Copy source archive to a temporary directory
            mkdir -p /tmp/pkgbuild-test
            cp target/ziplock-*.tar.gz /tmp/pkgbuild-test/
            cp packaging/arch/PKGBUILD /tmp/pkgbuild-test/
            cp packaging/arch/ziplock.install /tmp/pkgbuild-test/

            cd /tmp/pkgbuild-test

            # Update PKGBUILD with current checksum
            sha256sum=\$(sha256sum ziplock-*.tar.gz | cut -d' ' -f1)
            sed -i \"s/sha256sums=.*/sha256sums=('\$sha256sum')/\" PKGBUILD

            # Attempt to build (may fail due to missing source URL, but will validate most things)
            if makepkg --nobuild --nodeps; then
                echo 'PKGBUILD preparation: PASSED'
            else
                echo 'PKGBUILD preparation: FAILED'
                exit 1
            fi

            echo 'Full build test: COMPLETED'
        "

    log_success "Full build test completed"
}

print_test_summary() {
    echo
    log_success "Arch Linux packaging test completed!"
    echo
    echo "Test Summary:"
    echo "============="
    echo "✓ Source archive creation"
    echo "✓ PKGBUILD validity"
    echo "✓ Package metadata consistency"
    echo "✓ PKGBUILD version and checksum validation"
    echo "✓ PKGBUILD source URL consistency"
    echo "✓ Standalone PKGBUILD validation"
    echo "✓ Dependency availability"
    if [ "${FULL_BUILD_TEST:-false}" = "true" ]; then
        echo "✓ Full build test"
    else
        echo "- Full build test (skipped)"
    fi
    echo
    echo "Next Steps:"
    echo "==========="
    echo "1. If version/checksum validation failed, update PKGBUILD:"
    echo "   ./scripts/build/package-arch.sh --source-only"
    echo "   # This will generate the correct archive and SHA256"
    echo "   # Then manually update packaging/arch/PKGBUILD with:"
    echo "   # - pkgver=<current_version>"
    echo "   # - sha256sums=('<generated_sha256>')"
    echo
    echo "   Or run the standalone validation for detailed guidance:"
    echo "   ./scripts/build/test-pkgbuild-validation.sh --fix-suggestions"
    echo
    echo "2. Test on real Arch Linux system:"
    echo "   cd packaging/arch && makepkg -si"
    echo
    echo "3. Submit to AUR (when ready):"
    echo "   ./scripts/build/package-arch.sh --source-only"
    echo "   # Then follow AUR submission process"
    echo
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --full-build        Run full makepkg build test (slow)"
    echo "  --clean-images      Remove Docker images after test"
    echo "  --help             Show this help message"
    echo
    echo "Environment Variables:"
    echo "  FULL_BUILD_TEST    Set to 'true' to enable full build test"
    echo "  CLEAN_IMAGES       Set to 'true' to clean Docker images"
}

main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --full-build)
                export FULL_BUILD_TEST=true
                shift
                ;;
            --clean-images)
                export CLEAN_IMAGES=true
                shift
                ;;
            --help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done

    log_info "Starting Arch Linux packaging tests..."

    check_dependencies
    create_test_dockerfile
    build_test_image
    test_source_archive_creation
    test_pkgbuild_validity
    test_package_metadata
    test_pkgbuild_version_and_checksum
    test_pkgbuild_source_url
    run_standalone_validation
    test_dependencies
    run_full_build_test
    print_test_summary
}

# Run main function with all arguments
main "$@"
