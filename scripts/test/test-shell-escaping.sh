#!/bin/bash
set -euo pipefail

# Simple test script to debug shell escaping issues in Docker commands
# This isolates the problematic sed command that's causing syntax errors

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

# Test the sed command locally first
test_sed_locally() {
    log_info "Testing sed command locally..."

    # Create test PKGBUILD
    cat > /tmp/test_PKGBUILD << 'EOF'
pkgname=ziplock
pkgver=0.2.5
pkgrel=1
sha256sums=('old_hash_here')
EOF

    TEST_SHA256="abcd1234567890abcd1234567890abcd12345678"

    echo "Original file:"
    cat /tmp/test_PKGBUILD
    echo

    # Test different sed approaches
    log_info "Method 1: Direct substitution with single quotes"
    sed "s/^sha256sums=.*/sha256sums=('$TEST_SHA256')/" /tmp/test_PKGBUILD
    echo

    log_info "Method 2: Using double quotes"
    sed "s/^sha256sums=.*/sha256sums=(\"$TEST_SHA256\")/" /tmp/test_PKGBUILD
    echo

    log_info "Method 3: Using printf to avoid quotes in sed"
    sed "s/^sha256sums=.*/sha256sums=('PLACEHOLDER')/" /tmp/test_PKGBUILD | sed "s/PLACEHOLDER/$TEST_SHA256/"
    echo
}

# Test the command in Docker with minimal escaping
test_docker_simple() {
    log_info "Testing simple Docker command..."

    docker run --rm alpine:latest sh -c "
        echo 'Testing basic command...'
        TEST_VAR='hello_world'
        echo \"Variable value: \$TEST_VAR\"
    "
}

# Test the problematic sed command in Docker
test_docker_sed() {
    log_info "Testing sed command in Docker (this should work)..."

    docker run --rm alpine:latest sh -c "
        TEST_SHA256='abcd1234567890'
        echo 'sha256sums=(old_hash)' > /tmp/test
        echo 'Original:'
        cat /tmp/test
        echo 'Modified:'
        sed \"s/^sha256sums=.*/sha256sums=('\$TEST_SHA256')/\" /tmp/test
    "
}

# Test with the exact same escaping as the workflow
test_docker_workflow_style() {
    log_info "Testing with workflow-style nested commands..."

    docker run --rm alpine:latest sh -c "
        echo 'Outer shell command'
        sh -c '
            echo \"Inner shell command\"
            TEST_SHA256=\"abcd1234567890\"
            echo \"sha256sums=(old_hash)\" > /tmp/test
            echo \"Testing sed...\"
            sed \"s/^sha256sums=.*/sha256sums=('\"'\"'\$TEST_SHA256'\"'\"')/\" /tmp/test
        '
    "
}

# Test alternative approaches
test_alternatives() {
    log_info "Testing alternative approaches..."

    # Method 1: Use heredoc
    log_info "Method 1: Using heredoc"
    docker run --rm alpine:latest sh -c '
        TEST_SHA256="abcd1234567890"
        cat > /tmp/test << EOF
sha256sums=("$TEST_SHA256")
EOF
        cat /tmp/test
    '

    # Method 2: Use printf
    log_info "Method 2: Using printf"
    docker run --rm alpine:latest sh -c '
        TEST_SHA256="abcd1234567890"
        printf "sha256sums=(\"%s\")\n" "$TEST_SHA256" > /tmp/test
        cat /tmp/test
    '

    # Method 3: Use awk
    log_info "Method 3: Using awk"
    docker run --rm alpine:latest sh -c '
        TEST_SHA256="abcd1234567890"
        echo "sha256sums=(old_hash)" | awk -v sha="$TEST_SHA256" "{gsub(/sha256sums=.*/, \"sha256sums=(\\\"\" sha \"\\\")\"); print}"
    '
}

# Test the exact command from the GitHub workflow
test_exact_workflow_command() {
    log_info "Testing the exact command structure from GitHub workflow..."

    # This mimics the exact structure from the workflow
    docker run --rm alpine:latest bash -c "
        set -euo pipefail

        echo 'Starting nested command test...'

        # This is the structure that's failing
        bash -c '
            set -euo pipefail

            VERSION=\"0.2.8\"
            SHA256=\"abcd1234567890abcd1234567890abcd12345678\"

            echo \"Version: \$VERSION\"
            echo \"SHA256: \$SHA256\"

            # Create test PKGBUILD
            cat > /tmp/PKGBUILD << EOF
pkgname=ziplock
pkgver=0.2.5
sha256sums=(\"old_hash\")
EOF

            echo \"Original PKGBUILD:\"
            cat /tmp/PKGBUILD

            # This is the line that fails
            echo \"Updating with sed...\"
            sed \"s/^sha256sums=.*/sha256sums=('\"'\"'\$SHA256'\"'\"')/\" /tmp/PKGBUILD > /tmp/PKGBUILD.tmp
            mv /tmp/PKGBUILD.tmp /tmp/PKGBUILD

            echo \"Updated PKGBUILD:\"
            cat /tmp/PKGBUILD
        '
    "
}

# Main menu
main() {
    echo "Shell Escaping Test Suite"
    echo "========================="
    echo
    echo "This script tests various approaches to fix the shell escaping issues"
    echo "in the GitHub workflow Docker commands."
    echo

    echo "Available tests:"
    echo "1) Test sed command locally"
    echo "2) Test simple Docker command"
    echo "3) Test sed in Docker"
    echo "4) Test workflow-style nested commands"
    echo "5) Test alternative approaches"
    echo "6) Test exact workflow command (the failing one)"
    echo "7) Run all tests"

    read -p "Enter choice (1-7): " choice

    case $choice in
        1) test_sed_locally ;;
        2) test_docker_simple ;;
        3) test_docker_sed ;;
        4) test_docker_workflow_style ;;
        5) test_alternatives ;;
        6) test_exact_workflow_command ;;
        7)
            test_sed_locally
            echo "---"
            test_docker_simple
            echo "---"
            test_docker_sed
            echo "---"
            test_docker_workflow_style
            echo "---"
            test_alternatives
            echo "---"
            test_exact_workflow_command
            ;;
        *)
            log_error "Invalid choice"
            exit 1
            ;;
    esac

    log_success "Test completed!"
}

main "$@"
