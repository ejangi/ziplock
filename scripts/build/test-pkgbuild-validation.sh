#!/bin/bash
set -euo pipefail

# ZipLock PKGBUILD Validation Test Script
# Tests PKGBUILD version and checksum validation without Docker

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
PACKAGING_DIR="$PROJECT_ROOT/packaging/arch"

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

# Test that the PKGBUILD file exists and is readable
test_pkgbuild_exists() {
    log_info "Testing PKGBUILD file existence..."

    if [ ! -f "$PACKAGING_DIR/PKGBUILD" ]; then
        log_error "PKGBUILD file not found at $PACKAGING_DIR/PKGBUILD"
        return 1
    fi

    if [ ! -r "$PACKAGING_DIR/PKGBUILD" ]; then
        log_error "PKGBUILD file is not readable"
        return 1
    fi

    log_success "PKGBUILD file exists and is readable"
}

# Test PKGBUILD syntax
test_pkgbuild_syntax() {
    log_info "Testing PKGBUILD syntax..."

    if ! bash -n "$PACKAGING_DIR/PKGBUILD"; then
        log_error "PKGBUILD has syntax errors"
        return 1
    fi

    log_success "PKGBUILD syntax is valid"
}

# Test that required PKGBUILD variables are defined
test_pkgbuild_variables() {
    log_info "Testing required PKGBUILD variables..."

    # Source the PKGBUILD in a subshell to extract variables
    local pkgbuild_vars
    if ! pkgbuild_vars=$(cd "$PACKAGING_DIR" && bash -c 'source PKGBUILD && echo "pkgname=$pkgname" && echo "pkgver=$pkgver" && echo "pkgrel=$pkgrel" && echo "pkgdesc=$pkgdesc"'); then
        log_error "Failed to source PKGBUILD"
        return 1
    fi

    # Check each required variable
    local errors=0
    if ! echo "$pkgbuild_vars" | grep -q "pkgname=ziplock"; then
        log_error "pkgname is not set to 'ziplock'"
        errors=$((errors + 1))
    fi

    if ! echo "$pkgbuild_vars" | grep -q "pkgver="; then
        log_error "pkgver is not defined"
        errors=$((errors + 1))
    fi

    if ! echo "$pkgbuild_vars" | grep -q "pkgrel="; then
        log_error "pkgrel is not defined"
        errors=$((errors + 1))
    fi

    if ! echo "$pkgbuild_vars" | grep -q "pkgdesc="; then
        log_error "pkgdesc is not defined"
        errors=$((errors + 1))
    fi

    if [ $errors -gt 0 ]; then
        return 1
    fi

    log_success "Required PKGBUILD variables are defined"
}

# Test version consistency between Cargo.toml and PKGBUILD
test_version_consistency() {
    log_info "Testing version consistency..."

    # Get version from Cargo.toml
    local cargo_version
    if ! cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p'); then
        log_error "Failed to extract version from Cargo.toml"
        return 1
    fi

    if [ -z "$cargo_version" ]; then
        log_error "No version found in Cargo.toml"
        return 1
    fi

    # Get version from PKGBUILD
    local pkgbuild_version
    if ! pkgbuild_version=$(grep '^pkgver=' "$PACKAGING_DIR/PKGBUILD" | sed 's/pkgver=//'); then
        log_error "Failed to extract pkgver from PKGBUILD"
        return 1
    fi

    if [ -z "$pkgbuild_version" ]; then
        log_error "No pkgver found in PKGBUILD"
        return 1
    fi

    log_info "Cargo.toml version: $cargo_version"
    log_info "PKGBUILD version: $pkgbuild_version"

    if [ "$cargo_version" != "$pkgbuild_version" ]; then
        log_error "Version mismatch: Cargo.toml ($cargo_version) vs PKGBUILD ($pkgbuild_version)"
        log_error "PKGBUILD pkgver must match the workspace version in Cargo.toml"
        return 1
    fi

    log_success "Version consistency check passed: $cargo_version"
}

# Test SHA256 checksum format and validity
test_sha256_checksum() {
    log_info "Testing SHA256 checksum..."

    # Extract SHA256 from PKGBUILD
    local pkgbuild_sha
    if ! pkgbuild_sha=$(grep '^sha256sums=' "$PACKAGING_DIR/PKGBUILD" | sed "s/sha256sums=('\\(.*\\)')/\\1/"); then
        log_error "Failed to extract sha256sums from PKGBUILD"
        return 1
    fi

    if [ -z "$pkgbuild_sha" ]; then
        log_error "No sha256sums found in PKGBUILD"
        return 1
    fi

    log_info "PKGBUILD SHA256: $pkgbuild_sha"

    # Test 1: SHA256 must not be 'SKIP'
    if [ "$pkgbuild_sha" = "SKIP" ]; then
        log_error "PKGBUILD sha256sums is still set to 'SKIP' - must be a real checksum"
        log_error "Run './scripts/build/package-arch.sh --source-only' to generate correct checksum"
        return 1
    fi

    # Test 2: SHA256 must be valid format (64 hex characters)
    if ! echo "$pkgbuild_sha" | grep -qE '^[a-f0-9]{64}$'; then
        log_error "PKGBUILD sha256sums is not a valid SHA256 hash: $pkgbuild_sha"
        log_error "Expected: 64 lowercase hexadecimal characters"
        return 1
    fi

    log_success "SHA256 checksum format is valid"
}

# Test if checksum matches existing source archive (if available)
test_checksum_accuracy() {
    log_info "Testing checksum accuracy against source archive..."

    # Get version from Cargo.toml
    local cargo_version
    cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')

    # Check if source archive exists
    local archive_file="$BUILD_DIR/ziplock-${cargo_version}.tar.gz"
    if [ ! -f "$archive_file" ]; then
        log_warning "Source archive not found at $archive_file"
        log_info "Cannot verify checksum accuracy. Run './scripts/build/package-arch.sh --source-only' to create archive"
        return 0
    fi

    # Get SHA256 from PKGBUILD
    local pkgbuild_sha
    pkgbuild_sha=$(grep '^sha256sums=' "$PACKAGING_DIR/PKGBUILD" | sed "s/sha256sums=('\\(.*\\)')/\\1/")

    # Calculate actual SHA256 of the archive
    local actual_sha
    if ! actual_sha=$(sha256sum "$archive_file" | cut -d' ' -f1); then
        log_error "Failed to calculate SHA256 of archive"
        return 1
    fi

    log_info "Archive SHA256: $actual_sha"
    log_info "PKGBUILD SHA256: $pkgbuild_sha"

    if [ "$pkgbuild_sha" != "$actual_sha" ]; then
        log_error "PKGBUILD SHA256 ($pkgbuild_sha) does not match actual archive SHA256 ($actual_sha)"
        log_error "Run './scripts/build/package-arch.sh --source-only' to generate correct checksum"
        return 1
    fi

    log_success "SHA256 checksum matches actual archive"
}

# Test source URL format and version consistency
test_source_url() {
    log_info "Testing source URL format and version consistency..."

    # Get version from Cargo.toml
    local cargo_version
    cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')

    # Extract source URL from PKGBUILD
    local source_line
    if ! source_line=$(grep '^source=' "$PACKAGING_DIR/PKGBUILD"); then
        log_error "No source line found in PKGBUILD"
        return 1
    fi

    log_info "Source line: $source_line"

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

# Test install script exists
test_install_script() {
    log_info "Testing install script existence..."

    if [ ! -f "$PACKAGING_DIR/ziplock.install" ]; then
        log_warning "Install script not found at $PACKAGING_DIR/ziplock.install"
        log_info "This is optional but recommended for systemd services"
        return 0
    fi

    if [ ! -r "$PACKAGING_DIR/ziplock.install" ]; then
        log_error "Install script is not readable"
        return 1
    fi

    log_success "Install script exists and is readable"
}

# Generate suggestions for fixing issues
print_fix_suggestions() {
    echo
    log_info "Fix suggestions:"
    echo "================"
    echo
    echo "1. To update PKGBUILD with correct version and checksum:"
    echo "   ./scripts/build/package-arch.sh --source-only"
    echo "   # This generates the source archive and calculates SHA256"
    echo
    echo "2. To manually update PKGBUILD:"
    echo "   # Update version:"
    echo "   sed -i 's/^pkgver=.*/pkgver=$(grep '^version' Cargo.toml | sed -n '1s/.*\"\(.*\)\".*/\1/p')/' packaging/arch/PKGBUILD"
    echo
    echo "   # Update checksum (after creating source archive):"
    echo "   sha256sum target/ziplock-*.tar.gz | cut -d' ' -f1"
    echo "   # Then manually edit packaging/arch/PKGBUILD to set sha256sums=('NEW_CHECKSUM')"
    echo
    echo "3. To test the package build:"
    echo "   cd packaging/arch && makepkg -si  # (requires Arch Linux)"
    echo
    echo "4. To run full packaging tests:"
    echo "   ./scripts/build/test-arch-packaging.sh"
    echo
}

# Print test summary
print_test_summary() {
    local total_tests=7
    local passed_tests=$1
    local failed_tests=$((total_tests - passed_tests))

    echo
    if [ $failed_tests -eq 0 ]; then
        log_success "All PKGBUILD validation tests passed! ($passed_tests/$total_tests)"
        echo
        echo "✅ PKGBUILD file exists and is readable"
        echo "✅ PKGBUILD syntax is valid"
        echo "✅ Required variables are defined"
        echo "✅ Version consistency with Cargo.toml"
        echo "✅ SHA256 checksum format is valid"
        echo "✅ SHA256 checksum accuracy (if archive exists)"
        echo "✅ Source URL format and version consistency"
        echo
        log_success "PKGBUILD is ready for Arch Linux packaging!"
    else
        log_error "Some PKGBUILD validation tests failed ($failed_tests/$total_tests failed)"
        print_fix_suggestions
    fi
}

# Print usage information
print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Validates the Arch Linux PKGBUILD file for correctness."
    echo
    echo "Options:"
    echo "  --help             Show this help message"
    echo "  --fix-suggestions  Show fix suggestions even if tests pass"
    echo
    echo "This script validates:"
    echo "  - PKGBUILD file existence and syntax"
    echo "  - Required variables (pkgname, pkgver, pkgrel, pkgdesc)"
    echo "  - Version consistency with Cargo.toml"
    echo "  - SHA256 checksum format and validity"
    echo "  - Source URL format and version consistency"
    echo "  - Install script existence"
    echo
}

# Main function
main() {
    local show_suggestions=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help)
                print_usage
                exit 0
                ;;
            --fix-suggestions)
                show_suggestions=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done

    log_info "Starting PKGBUILD validation tests..."
    echo

    local passed_tests=0
    local test_failed=false

    # Run tests
    if test_pkgbuild_exists; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    if test_pkgbuild_syntax; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    if test_pkgbuild_variables; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    if test_version_consistency; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    if test_sha256_checksum; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    if test_checksum_accuracy; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    if test_source_url; then
        passed_tests=$((passed_tests + 1))
    else
        test_failed=true
    fi

    # Install script test is optional
    test_install_script

    # Print summary
    print_test_summary $passed_tests

    # Show suggestions if requested or if tests failed
    if [ "$show_suggestions" = true ] && [ "$test_failed" = false ]; then
        print_fix_suggestions
    fi

    # Exit with appropriate code
    if [ "$test_failed" = true ]; then
        exit 1
    else
        exit 0
    fi
}

# Run main function with all arguments
main "$@"
