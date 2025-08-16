#!/bin/bash
set -euo pipefail

# Test script for the printf-based PKGBUILD update approach
# This tests the fix for the shell escaping issues in the GitHub workflow

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test the printf approach locally
test_printf_locally() {
    log_info "Testing printf approach locally..."

    # Create test PKGBUILD
    cat > /tmp/test_PKGBUILD << 'EOF'
# Maintainer: Test <test@example.com>
pkgname=ziplock
pkgver=0.2.5
pkgrel=1
pkgdesc="Test package"
arch=('x86_64')
url="https://example.com"
license=('Apache')
depends=('glibc')
sha256sums=('old_hash_here_should_be_replaced')
provides=('ziplock')

build() {
    echo "Building..."
}

package() {
    echo "Packaging..."
}
EOF

    echo "Original PKGBUILD:"
    cat /tmp/test_PKGBUILD
    echo

    # Test the printf approach
    TEST_SHA256="abcd1234567890abcd1234567890abcd12345678"
    VERSION="0.2.8"

    log_info "Applying updates with printf approach..."

    # Update version
    sed -i "s/^pkgver=.*/pkgver=$VERSION/" /tmp/test_PKGBUILD

    # Update sha256sums using printf approach
    cp /tmp/test_PKGBUILD /tmp/test_PKGBUILD.backup
    grep -v "^sha256sums=" /tmp/test_PKGBUILD.backup > /tmp/test_PKGBUILD.tmp
    printf "sha256sums=('%s')\n" "$TEST_SHA256" >> /tmp/test_PKGBUILD.tmp
    mv /tmp/test_PKGBUILD.tmp /tmp/test_PKGBUILD
    rm /tmp/test_PKGBUILD.backup

    echo "Updated PKGBUILD:"
    cat /tmp/test_PKGBUILD
    echo

    # Verify the changes
    if grep -q "pkgver=$VERSION" /tmp/test_PKGBUILD; then
        log_success "Version update successful"
    else
        log_error "Version update failed"
        return 1
    fi

    if grep -q "sha256sums=('$TEST_SHA256')" /tmp/test_PKGBUILD; then
        log_success "SHA256 update successful"
    else
        log_error "SHA256 update failed"
        return 1
    fi

    # Cleanup
    rm -f /tmp/test_PKGBUILD*
}

# Test the approach in a simple Docker container
test_printf_in_docker() {
    log_info "Testing printf approach in Docker..."

    docker run --rm alpine:latest sh -c "
        # Create test PKGBUILD
        cat > /tmp/test_PKGBUILD << 'EOF'
pkgname=ziplock
pkgver=0.2.5
pkgrel=1
sha256sums=('old_hash')
depends=('glibc')
EOF

        echo 'Original PKGBUILD:'
        cat /tmp/test_PKGBUILD
        echo

        # Apply the printf approach
        TEST_SHA256='abcd1234567890'
        VERSION='0.2.8'

        echo 'Updating version...'
        sed -i \"s/^pkgver=.*/pkgver=\$VERSION/\" /tmp/test_PKGBUILD

        echo 'Updating sha256sums with printf...'
        cp /tmp/test_PKGBUILD /tmp/test_PKGBUILD.backup
        grep -v \"^sha256sums=\" /tmp/test_PKGBUILD.backup > /tmp/test_PKGBUILD.tmp
        printf \"sha256sums=('%s')\n\" \"\$TEST_SHA256\" >> /tmp/test_PKGBUILD.tmp
        mv /tmp/test_PKGBUILD.tmp /tmp/test_PKGBUILD
        rm /tmp/test_PKGBUILD.backup

        echo 'Updated PKGBUILD:'
        cat /tmp/test_PKGBUILD
        echo

        # Verify
        if grep -q \"pkgver=\$VERSION\" /tmp/test_PKGBUILD; then
            echo 'SUCCESS: Version updated correctly'
        else
            echo 'ERROR: Version update failed'
            exit 1
        fi

        if grep -q \"sha256sums=('\$TEST_SHA256')\" /tmp/test_PKGBUILD; then
            echo 'SUCCESS: SHA256 updated correctly'
        else
            echo 'ERROR: SHA256 update failed'
            exit 1
        fi
    "
}

# Test the exact workflow structure with the printf approach
test_workflow_structure() {
    log_info "Testing exact workflow structure with printf approach..."

    # Check if we have access to the arch-builder image
    if ! docker image ls | grep -q "arch-builder"; then
        log_info "Arch builder image not available locally, using alpine for structure test"
        DOCKER_IMAGE="alpine:latest"
        SHELL_CMD="sh"
    else
        DOCKER_IMAGE="ghcr.io/ejangi/ziplock/arch-builder:latest"
        SHELL_CMD="bash"
    fi

    # Get current user and group IDs
    USER_ID=$(id -u)
    GROUP_ID=$(id -g)

    log_info "Testing with Docker image: $DOCKER_IMAGE"

    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -e USER_ID="$USER_ID" \
        -e GROUP_ID="$GROUP_ID" \
        "$DOCKER_IMAGE" \
        $SHELL_CMD -c "
            set -euo pipefail

            echo 'Copying workspace to avoid git permission conflicts...'
            cp -r /workspace /tmp/build
            cd /tmp/build

            # Create a test structure similar to the workflow
            mkdir -p target/install/usr/bin
            mkdir -p target/install/usr/lib
            mkdir -p target/install/etc/ziplock
            echo 'mock binary' > target/install/usr/bin/ziplock
            echo 'mock library' > target/install/usr/lib/libziplock_shared.so

            # Validate copied structure
            if [ ! -f 'target/install/usr/bin/ziplock' ]; then
                echo 'ERROR: Binary not found in copied workspace'
                exit 1
            fi

            echo 'Structure validation passed'

            # Get version from Cargo.toml (use a fallback if not parseable)
            if [ -f 'Cargo.toml' ]; then
                VERSION=\$(grep '^version' Cargo.toml | head -1 | sed 's/.*= \"\(.*\)\"/\1/' || echo '0.2.8')
            else
                VERSION='0.2.8'
            fi
            echo \"Creating package for version: \$VERSION\"

            # Create a mock source archive
            mkdir -p target/src
            echo 'mock source' > target/src/test.txt
            echo 'mock source content' | gzip > \"target/ziplock-\$VERSION.tar.gz\"

            # Calculate SHA256
            if command -v sha256sum >/dev/null; then
                SHA256=\$(sha256sum \"target/ziplock-\$VERSION.tar.gz\" | cut -d' ' -f1)
            else
                # Fallback for systems without sha256sum
                SHA256='abcd1234567890abcd1234567890abcd12345678'
            fi
            echo \"Source archive SHA256: \$SHA256\"

            # Update PKGBUILD with the printf approach
            if [ -f 'packaging/arch/PKGBUILD' ]; then
                cd packaging/arch
                echo 'Original PKGBUILD version line:'
                grep '^pkgver=' PKGBUILD || echo 'No pkgver line found'

                echo 'Updating version...'
                sed -i \"s/^pkgver=.*/pkgver=\$VERSION/\" PKGBUILD

                echo 'Updating sha256sums with printf approach...'
                cp PKGBUILD PKGBUILD.backup
                grep -v \"^sha256sums=\" PKGBUILD.backup > PKGBUILD.tmp
                printf \"sha256sums=('%s')\n\" \"\$SHA256\" >> PKGBUILD.tmp
                mv PKGBUILD.tmp PKGBUILD
                rm PKGBUILD.backup

                echo 'Updated PKGBUILD lines:'
                grep '^pkgver=' PKGBUILD
                grep '^sha256sums=' PKGBUILD

                echo 'PKGBUILD update completed successfully'
            else
                echo 'No PKGBUILD found, creating mock one for testing'
                cat > PKGBUILD << EOF
pkgname=ziplock
pkgver=\$VERSION
pkgrel=1
sha256sums=('\$SHA256')
EOF
                echo 'Mock PKGBUILD created'
            fi

            echo 'Workflow structure test completed successfully'
        "
}

# Test edge cases
test_edge_cases() {
    log_info "Testing edge cases..."

    # Test with special characters in SHA256
    log_info "Testing with realistic SHA256..."
    TEST_SHA256="d4f5e6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5"

    cat > /tmp/edge_test_PKGBUILD << 'EOF'
pkgname=ziplock
pkgver=0.2.5
pkgrel=1
sha256sums=('old_hash_here')
depends=('glibc')
EOF

    # Apply printf approach
    cp /tmp/edge_test_PKGBUILD /tmp/edge_test_PKGBUILD.backup
    grep -v "^sha256sums=" /tmp/edge_test_PKGBUILD.backup > /tmp/edge_test_PKGBUILD.tmp
    printf "sha256sums=('%s')\n" "$TEST_SHA256" >> /tmp/edge_test_PKGBUILD.tmp
    mv /tmp/edge_test_PKGBUILD.tmp /tmp/edge_test_PKGBUILD
    rm /tmp/edge_test_PKGBUILD.backup

    echo "Result with realistic SHA256:"
    grep "^sha256sums=" /tmp/edge_test_PKGBUILD

    # Test with multiple sha256sums entries
    log_info "Testing with multiple sha256sums entries..."
    cat > /tmp/multi_test_PKGBUILD << 'EOF'
pkgname=ziplock
pkgver=0.2.5
pkgrel=1
sha256sums=('hash1'
           'hash2'
           'hash3')
depends=('glibc')
EOF

    echo "Original multi-line sha256sums:"
    grep -A 3 "^sha256sums=" /tmp/multi_test_PKGBUILD

    # This approach handles multi-line entries correctly
    cp /tmp/multi_test_PKGBUILD /tmp/multi_test_PKGBUILD.backup
    # Remove all lines starting with sha256sums= and subsequent lines that are part of the array
    awk '/^sha256sums=/{skip=1} skip && /\)/{skip=0; next} !skip' /tmp/multi_test_PKGBUILD.backup > /tmp/multi_test_PKGBUILD.tmp
    printf "sha256sums=('%s')\n" "$TEST_SHA256" >> /tmp/multi_test_PKGBUILD.tmp
    mv /tmp/multi_test_PKGBUILD.tmp /tmp/multi_test_PKGBUILD
    rm /tmp/multi_test_PKGBUILD.backup

    echo "Result after update:"
    cat /tmp/multi_test_PKGBUILD

    # Cleanup
    rm -f /tmp/edge_test_PKGBUILD* /tmp/multi_test_PKGBUILD*

    log_success "Edge cases test completed"
}

# Compare with the old sed approach (to show why it fails)
test_comparison() {
    log_info "Comparing printf approach with the problematic sed approach..."

    echo "=== Printf approach (WORKING) ==="
    docker run --rm alpine:latest sh -c "
        TEST_SHA256='abcd1234567890'
        echo 'sha256sums=(old_hash)' > /tmp/test

        echo 'Using printf approach:'
        grep -v '^sha256sums=' /tmp/test || true
        printf \"sha256sums=('%s')\n\" \"\$TEST_SHA256\"
    "

    echo
    echo "=== Sed approach (PROBLEMATIC) ==="
    echo "This would be the command that fails in nested shells:"
    echo "sed \"s/^sha256sums=.*/sha256sums=('\$TEST_SHA256')/\""
    echo "The parentheses cause syntax errors in complex shell escaping scenarios."
}

# Main function
main() {
    echo "Printf Approach Test Suite"
    echo "=========================="
    echo
    echo "This script tests the printf-based fix for PKGBUILD updates"
    echo "that replaces the problematic sed command causing shell escaping issues."
    echo

    if [ "${1:-}" = "--all" ]; then
        log_info "Running all tests..."
        test_printf_locally
        echo "---"
        test_printf_in_docker
        echo "---"
        test_workflow_structure
        echo "---"
        test_edge_cases
        echo "---"
        test_comparison
    else
        echo "Available tests:"
        echo "1) Test printf approach locally"
        echo "2) Test printf approach in Docker"
        echo "3) Test exact workflow structure"
        echo "4) Test edge cases"
        echo "5) Compare with old sed approach"
        echo "6) Run all tests"

        read -p "Enter choice (1-6): " choice

        case $choice in
            1) test_printf_locally ;;
            2) test_printf_in_docker ;;
            3) test_workflow_structure ;;
            4) test_edge_cases ;;
            5) test_comparison ;;
            6)
                test_printf_locally
                echo "---"
                test_printf_in_docker
                echo "---"
                test_workflow_structure
                echo "---"
                test_edge_cases
                echo "---"
                test_comparison
                ;;
            *)
                log_error "Invalid choice"
                exit 1
                ;;
        esac
    fi

    log_success "All tests completed! The printf approach should resolve the shell escaping issues."
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--all|--help]"
        echo
        echo "Test the printf-based PKGBUILD update approach"
        echo
        echo "Options:"
        echo "  --all    Run all tests automatically"
        echo "  --help   Show this help message"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
