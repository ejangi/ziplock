#!/bin/bash

# ZipLock Version Management Helper Script
# This script helps developers increment version numbers and update the changelog

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }

# Function to show usage
show_usage() {
    echo "Usage: $0 [patch|minor|major] [changelog-entry]"
    echo ""
    echo "Arguments:"
    echo "  patch|minor|major  - Type of version bump"
    echo "  changelog-entry    - Brief, user-friendly description of the change"
    echo ""
    echo "Examples:"
    echo "  $0 patch \"Fixed crash when opening large password files\""
    echo "  $0 minor \"Added CSV export functionality\""
    echo "  $0 major \"New encryption system (requires data migration)\""
    echo ""
    echo "This script will:"
    echo "  1. Increment the version in Cargo.toml files"
    echo "  2. Add the changelog entry to CHANGELOG.md"
    echo "  3. Show a summary of changes"
}

# Function to get current version from Cargo.toml
get_current_version() {
    if [ -f "Cargo.toml" ]; then
        grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/'
    else
        print_error "Cargo.toml not found in current directory"
        exit 1
    fi
}

# Function to increment version
increment_version() {
    local version="$1"
    local bump_type="$2"

    local major minor patch
    IFS='.' read -r major minor patch <<< "$version"

    case "$bump_type" in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            print_error "Invalid bump type: $bump_type"
            exit 1
            ;;
    esac

    echo "$major.$minor.$patch"
}

# Function to update version in Cargo.toml files
update_cargo_versions() {
    local new_version="$1"
    local updated_files=()

    # Find all Cargo.toml files
    while IFS= read -r -d '' file; do
        if grep -q '^version = ' "$file"; then
            print_info "Updating version in $file"
            sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$file"
            rm "$file.bak" 2>/dev/null || true
            updated_files+=("$file")
        fi
    done < <(find . -name "Cargo.toml" -print0)

    if [ ${#updated_files[@]} -eq 0 ]; then
        print_warning "No Cargo.toml files found with version field"
    else
        print_success "Updated version in ${#updated_files[@]} Cargo.toml file(s)"
    fi
}

# Function to update changelog
update_changelog() {
    local version="$1"
    local bump_type="$2"
    local entry="$3"
    local current_date=$(date +%Y-%m-%d)

    if [ ! -f "CHANGELOG.md" ]; then
        print_error "CHANGELOG.md not found"
        exit 1
    fi

    # Determine the section based on bump type
    local section
    case "$bump_type" in
        "major") section="Changed" ;;
        "minor") section="Added" ;;
        "patch") section="Fixed" ;;
    esac

    # Create a temporary file for the new changelog
    local temp_file=$(mktemp)
    local found_unreleased=false
    local added_entry=false

    while IFS= read -r line; do
        echo "$line" >> "$temp_file"

        # If we find the Unreleased section and haven't added our entry yet
        if [[ "$line" =~ ^##\ \[Unreleased\] ]] && [ "$found_unreleased" = false ]; then
            found_unreleased=true
            echo "" >> "$temp_file"
            echo "## [$version] - $current_date" >> "$temp_file"
            echo "" >> "$temp_file"
            echo "### $section" >> "$temp_file"
            echo "- $entry" >> "$temp_file"
            added_entry=true
        fi
    done < "CHANGELOG.md"

    if [ "$added_entry" = false ]; then
        print_error "Could not find [Unreleased] section in CHANGELOG.md"
        rm "$temp_file"
        exit 1
    fi

    # Replace the original file
    mv "$temp_file" "CHANGELOG.md"
    print_success "Added changelog entry for version $version"
}

# Function to show summary
show_summary() {
    local old_version="$1"
    local new_version="$2"
    local bump_type="$3"
    local entry="$4"

    echo ""
    print_success "Version Update Complete!"
    echo ""
    echo "  Previous version: $old_version"
    echo "  New version:      $new_version"
    echo "  Bump type:        $bump_type"
    echo "  Changelog entry:  $entry"
    echo ""
    print_info "Next steps:"
    echo "  1. Review the changes with: git diff"
    echo "  2. Test your changes thoroughly"
    echo "  3. Commit with: git add . && git commit -m \"Bump version to $new_version\""
    echo "  4. Create a release tag: git tag v$new_version"
    echo "  5. Push changes: git push && git push --tags"
}

# Main script
main() {
    # Check if we're in the project root
    if [ ! -f "Cargo.toml" ] || [ ! -f "CHANGELOG.md" ]; then
        print_error "This script must be run from the project root directory"
        print_error "Make sure both Cargo.toml and CHANGELOG.md exist"
        exit 1
    fi

    # Check arguments
    if [ $# -ne 2 ]; then
        show_usage
        exit 1
    fi

    local bump_type="$1"
    local changelog_entry="$2"

    # Validate bump type
    if [[ ! "$bump_type" =~ ^(patch|minor|major)$ ]]; then
        print_error "Invalid bump type: $bump_type"
        show_usage
        exit 1
    fi

    # Validate changelog entry
    if [ -z "$changelog_entry" ]; then
        print_error "Changelog entry cannot be empty"
        show_usage
        exit 1
    fi

    # Get current version and calculate new version
    local current_version
    current_version=$(get_current_version)
    local new_version
    new_version=$(increment_version "$current_version" "$bump_type")

    print_info "Current version: $current_version"
    print_info "New version: $new_version"
    print_info "Changelog entry: $changelog_entry"
    echo ""

    # Confirm with user
    read -p "Continue with version update? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Version update cancelled"
        exit 0
    fi

    # Update versions and changelog
    update_cargo_versions "$new_version"
    update_changelog "$new_version" "$bump_type" "$changelog_entry"

    # Show summary
    show_summary "$current_version" "$new_version" "$bump_type" "$changelog_entry"
}

# Run main function
main "$@"
