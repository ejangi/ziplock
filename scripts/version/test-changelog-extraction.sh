#!/bin/bash

# Test script for changelog extraction logic
# This script tests the same logic used in the GitHub workflow

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }

# Test the changelog extraction logic
test_changelog_extraction() {
    local version="$1"

    if [ ! -f "CHANGELOG.md" ]; then
        print_error "CHANGELOG.md not found"
        return 1
    fi

    print_info "Testing changelog extraction for version: $version"

    # Extract the version section from CHANGELOG.md (same logic as workflow)
    CHANGELOG_SECTION=""
    if grep -q "## \[$version\]" CHANGELOG.md; then
        # Extract from version header to next version header or end of file
        CHANGELOG_SECTION=$(awk "/^## \[$version\]/{flag=1; next} /^## \[/{flag=0} flag" CHANGELOG.md)
        print_success "Found specific version section for [$version]"
    elif grep -q "## \[Unreleased\]" CHANGELOG.md; then
        # If no specific version found, use Unreleased section
        CHANGELOG_SECTION=$(awk "/^## \[Unreleased\]/{flag=1; next} /^## \[/{flag=0} flag" CHANGELOG.md)
        print_warning "Version [$version] not found, using [Unreleased] section"
    else
        print_error "No changelog section found"
        return 1
    fi

    # Clean up and format the changelog section
    if [ -n "$CHANGELOG_SECTION" ]; then
        # Remove empty lines at the beginning and end, format for release notes
        CHANGELOG_SECTION=$(echo "$CHANGELOG_SECTION" | sed '/^$/d' | sed 's/^### /## /')

        echo ""
        print_success "Extracted changelog content:"
        echo "----------------------------------------"
        echo "$CHANGELOG_SECTION"
        echo "----------------------------------------"
    else
        print_error "No changelog entries found for this version."
        return 1
    fi
}

# Test with different scenarios
main() {
    if [ ! -f "CHANGELOG.md" ]; then
        print_error "This script must be run from the project root directory"
        print_error "CHANGELOG.md not found"
        exit 1
    fi

    echo "Testing changelog extraction logic..."
    echo ""

    # Test 1: Extract unreleased section (most common case)
    print_info "Test 1: Extracting [Unreleased] section"
    test_changelog_extraction "999.999.999"  # Non-existent version to force Unreleased
    echo ""

    # Test 2: Extract existing version if available
    if grep -q "## \[0.1.0\]" CHANGELOG.md; then
        print_info "Test 2: Extracting [0.1.0] section"
        test_changelog_extraction "0.1.0"
        echo ""
    fi

    # Test 3: Show the raw sections available
    print_info "Available sections in CHANGELOG.md:"
    grep "^## \[" CHANGELOG.md || print_warning "No version sections found"

    print_success "Changelog extraction test completed!"
}

main "$@"
