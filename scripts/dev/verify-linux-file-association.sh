#!/bin/bash
set -euo pipefail

# ZipLock Linux File Association Verification Script
# Verifies that .7z file association is properly configured for Linux

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

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Global verification status
VERIFICATION_PASSED=true

check_file_exists() {
    local description="$1"
    local file_path="$2"

    if [ -f "$file_path" ]; then
        log_success "$description: Found"
        return 0
    else
        log_error "$description: Missing - $file_path"
        VERIFICATION_PASSED=false
        return 1
    fi
}

check_desktop_file() {
    log_info "Checking desktop entry file..."

    local desktop_file="$PROJECT_ROOT/apps/linux/resources/ziplock.desktop"

    if ! check_file_exists "Desktop entry file" "$desktop_file"; then
        return 1
    fi

    # Check for MIME type association
    if grep -q "MimeType=.*application/x-7z-compressed" "$desktop_file"; then
        log_success "Desktop file contains .7z MIME type association"
    else
        log_error "Desktop file missing .7z MIME type association"
        log_info "Expected: MimeType=application/x-7z-compressed;"
        VERIFICATION_PASSED=false
    fi

    # Check other required fields
    local required_fields=("Type=Application" "Name=" "Exec=" "Icon=")

    for field in "${required_fields[@]}"; do
        if grep -q "^$field" "$desktop_file"; then
            log_success "Desktop file contains required field: $field"
        else
            log_error "Desktop file missing required field: $field"
            VERIFICATION_PASSED=false
        fi
    done

    return 0
}

check_mime_type_definition() {
    log_info "Checking MIME type definition file..."

    local mime_file="$PROJECT_ROOT/apps/linux/resources/mime/packages/ziplock.xml"

    if ! check_file_exists "MIME type definition file" "$mime_file"; then
        return 1
    fi

    # Check for proper MIME type definition
    if grep -q '<mime-type type="application/x-7z-compressed">' "$mime_file"; then
        log_success "MIME definition contains correct MIME type"
    else
        log_error "MIME definition missing or incorrect MIME type"
        VERIFICATION_PASSED=false
    fi

    # Check for .7z file glob patterns
    if grep -q '<glob pattern="\*.7z"/>' "$mime_file"; then
        log_success "MIME definition contains .7z glob pattern"
    else
        log_error "MIME definition missing .7z glob pattern"
        VERIFICATION_PASSED=false
    fi

    # Check for magic bytes
    if grep -q '<magic priority="50">' "$mime_file"; then
        log_success "MIME definition contains magic byte detection"
    else
        log_warning "MIME definition missing magic byte detection (optional)"
    fi

    return 0
}

check_build_script_integration() {
    log_info "Checking build script integration..."

    local build_script="$PROJECT_ROOT/scripts/build/build-linux.sh"

    if ! check_file_exists "Build script" "$build_script"; then
        return 1
    fi

    # Check if desktop file is copied
    if grep -q "ziplock.desktop" "$build_script"; then
        log_success "Build script copies desktop file"
    else
        log_error "Build script does not copy desktop file"
        VERIFICATION_PASSED=false
    fi

    # Check if MIME type file is copied
    if grep -q "mime/packages/ziplock.xml" "$build_script"; then
        log_success "Build script copies MIME type definition"
    else
        log_error "Build script does not copy MIME type definition"
        VERIFICATION_PASSED=false
    fi

    return 0
}

check_packaging_script_integration() {
    log_info "Checking packaging script integration..."

    local package_script="$PROJECT_ROOT/scripts/build/package-deb.sh"

    if ! check_file_exists "Packaging script" "$package_script"; then
        return 1
    fi

    # Check if MIME database is updated in post-install
    if grep -q "update-mime-database" "$package_script"; then
        log_success "Packaging script updates MIME database"
    else
        log_error "Packaging script does not update MIME database"
        VERIFICATION_PASSED=false
    fi

    # Check if desktop database is updated
    if grep -q "update-desktop-database" "$package_script"; then
        log_success "Packaging script updates desktop database"
    else
        log_error "Packaging script does not update desktop database"
        VERIFICATION_PASSED=false
    fi

    # Check if shared-mime-info is a dependency
    if grep -q "shared-mime-info" "$package_script"; then
        log_success "Packaging script includes shared-mime-info dependency"
    else
        log_warning "Packaging script missing shared-mime-info dependency"
        log_info "This may cause MIME database updates to fail on some systems"
    fi

    return 0
}

check_system_integration() {
    log_info "Checking system integration (if ZipLock is installed)..."

    # Check if ZipLock is installed
    if ! command -v ziplock &> /dev/null; then
        log_warning "ZipLock not installed - skipping system integration checks"
        return 0
    fi

    # Check if desktop file is installed
    if [ -f "/usr/share/applications/ziplock.desktop" ]; then
        log_success "Desktop file is installed system-wide"

        # Check if MIME type is registered
        if xdg-mime query default application/x-7z-compressed 2>/dev/null | grep -q ziplock; then
            log_success "ZipLock is registered as default handler for .7z files"
        else
            log_info "ZipLock is not the default handler for .7z files (this is normal)"
            log_info "Users can still select ZipLock from 'Open with...' menu"
        fi
    else
        log_warning "Desktop file not found in system applications directory"
    fi

    # Check if MIME type definition is installed
    if [ -f "/usr/share/mime/packages/ziplock.xml" ]; then
        log_success "MIME type definition is installed system-wide"
    else
        log_warning "MIME type definition not found in system MIME packages"
    fi

    return 0
}

test_file_association() {
    log_info "Testing file association (if available)..."

    # Only test if we have a display and required tools
    if [ -z "${DISPLAY:-}" ]; then
        log_warning "No display available - skipping interactive tests"
        return 0
    fi

    if ! command -v xdg-mime &> /dev/null; then
        log_warning "xdg-mime not available - skipping association tests"
        return 0
    fi

    # Test MIME type query
    local mime_result
    if mime_result=$(xdg-mime query filetype test.7z 2>/dev/null); then
        if [ "$mime_result" = "application/x-7z-compressed" ]; then
            log_success "System correctly identifies .7z files as application/x-7z-compressed"
        else
            log_warning "System identifies .7z files as: $mime_result"
            log_info "Expected: application/x-7z-compressed"
        fi
    else
        log_warning "Could not query MIME type for .7z files"
    fi

    return 0
}

print_manual_testing_instructions() {
    echo
    log_info "Manual Testing Instructions:"
    echo "============================"
    echo
    echo "1. Create a test .7z file:"
    echo "   7z a test.7z /etc/passwd"
    echo
    echo "2. Right-click the .7z file in your file manager"
    echo "   - You should see ZipLock in the 'Open with...' menu"
    echo
    echo "3. Test command-line association:"
    echo "   xdg-mime query default application/x-7z-compressed"
    echo "   xdg-open test.7z"
    echo
    echo "4. Test from different file managers:"
    echo "   - Nautilus (GNOME Files)"
    echo "   - Dolphin (KDE)"
    echo "   - Thunar (XFCE)"
    echo "   - PCManFM (LXDE)"
    echo
    echo "5. Test double-click behavior:"
    echo "   - Double-click should either open with ZipLock"
    echo "   - Or show 'Open with...' dialog including ZipLock"
    echo
}

print_troubleshooting_tips() {
    echo
    log_info "Troubleshooting Tips:"
    echo "===================="
    echo
    echo "If ZipLock doesn't appear in 'Open with...' menu:"
    echo
    echo "1. Update system databases:"
    echo "   sudo update-desktop-database"
    echo "   sudo update-mime-database /usr/share/mime"
    echo
    echo "2. Clear user MIME cache:"
    echo "   rm -f ~/.local/share/mime/mimeinfo.cache"
    echo "   rm -f ~/.local/share/applications/mimeinfo.cache"
    echo
    echo "3. Restart file manager:"
    echo "   killall nautilus && nautilus &"
    echo "   # Or for KDE: killall dolphin && dolphin &"
    echo
    echo "4. Check desktop file syntax:"
    echo "   desktop-file-validate /usr/share/applications/ziplock.desktop"
    echo
    echo "5. Verify MIME type definition:"
    echo "   xmllint --noout /usr/share/mime/packages/ziplock.xml"
    echo
    echo "6. Force association (if needed):"
    echo "   xdg-mime default ziplock.desktop application/x-7z-compressed"
    echo
}

main() {
    echo "ZipLock Linux File Association Verification"
    echo "==========================================="
    echo

    check_desktop_file
    echo

    check_mime_type_definition
    echo

    check_build_script_integration
    echo

    check_packaging_script_integration
    echo

    check_system_integration
    echo

    test_file_association
    echo

    if [ "$VERIFICATION_PASSED" = true ]; then
        log_success "All file association checks passed!"
        echo
        log_info "ZipLock should appear in 'Open with...' menus for .7z files"
    else
        log_error "Some file association checks failed!"
        echo
        log_info "Please fix the issues above before testing file associations"
        print_troubleshooting_tips
        exit 1
    fi

    print_manual_testing_instructions
    print_troubleshooting_tips
}

# Run main function
main "$@"
