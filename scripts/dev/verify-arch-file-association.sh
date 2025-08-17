#!/bin/bash
set -euo pipefail

# ZipLock Arch Linux File Association Verification Script
# Verifies that .7z file association is properly configured for Arch Linux

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

check_arch_system() {
    log_info "Checking if running on Arch Linux..."

    if [ -f /etc/arch-release ] || command -v pacman &> /dev/null; then
        log_success "Arch Linux system detected"
        return 0
    else
        log_warning "Not running on Arch Linux - some checks may be skipped"
        return 1
    fi
}

check_pkgbuild() {
    log_info "Checking PKGBUILD file..."

    local pkgbuild_file="$PROJECT_ROOT/packaging/arch/PKGBUILD"

    if ! check_file_exists "PKGBUILD file" "$pkgbuild_file"; then
        return 1
    fi

    # Check for shared-mime-info dependency
    if grep -q "shared-mime-info" "$pkgbuild_file"; then
        log_success "PKGBUILD includes shared-mime-info dependency"
    else
        log_error "PKGBUILD missing shared-mime-info dependency"
        VERIFICATION_PASSED=false
    fi

    # Check for desktop-file-utils in optdepends
    if grep -q "desktop-file-utils" "$pkgbuild_file"; then
        log_success "PKGBUILD includes desktop-file-utils in optdepends"
    else
        log_warning "PKGBUILD missing desktop-file-utils in optdepends"
    fi

    # Check if MIME type file installation is included
    if grep -q "ziplock.xml" "$pkgbuild_file"; then
        log_success "PKGBUILD installs MIME type definition"
    else
        log_error "PKGBUILD does not install MIME type definition"
        VERIFICATION_PASSED=false
    fi

    # Check if desktop file installation is included
    if grep -q "ziplock.desktop" "$pkgbuild_file"; then
        log_success "PKGBUILD installs desktop file"
    else
        log_error "PKGBUILD does not install desktop file"
        VERIFICATION_PASSED=false
    fi

    return 0
}

check_install_script() {
    log_info "Checking Arch install script..."

    local install_script="$PROJECT_ROOT/packaging/arch/ziplock.install"

    if ! check_file_exists "Install script" "$install_script"; then
        return 1
    fi

    # Check for MIME database updates in post_install
    if grep -A 20 "post_install()" "$install_script" | grep -q "update-mime-database"; then
        log_success "Install script updates MIME database in post_install"
    else
        log_error "Install script does not update MIME database in post_install"
        VERIFICATION_PASSED=false
    fi

    # Check for MIME database updates in post_upgrade
    if grep -A 20 "post_upgrade()" "$install_script" | grep -q "update-mime-database"; then
        log_success "Install script updates MIME database in post_upgrade"
    else
        log_error "Install script does not update MIME database in post_upgrade"
        VERIFICATION_PASSED=false
    fi

    # Check for MIME database updates in post_remove
    if grep -A 20 "post_remove()" "$install_script" | grep -q "update-mime-database"; then
        log_success "Install script updates MIME database in post_remove"
    else
        log_error "Install script does not update MIME database in post_remove"
        VERIFICATION_PASSED=false
    fi

    # Check for desktop database updates
    if grep -q "update-desktop-database" "$install_script"; then
        log_success "Install script updates desktop database"
    else
        log_error "Install script does not update desktop database"
        VERIFICATION_PASSED=false
    fi

    return 0
}

check_packaging_script() {
    log_info "Checking Arch packaging script..."

    local package_script="$PROJECT_ROOT/scripts/build/package-arch.sh"

    if ! check_file_exists "Packaging script" "$package_script"; then
        return 1
    fi

    # Check if MIME type file is verified in required files
    if grep -A 10 "required_files=" "$package_script" | grep -q "ziplock.xml"; then
        log_success "Packaging script verifies MIME type definition"
    else
        log_error "Packaging script does not verify MIME type definition"
        VERIFICATION_PASSED=false
    fi

    # Check if desktop file is verified
    if grep -A 10 "required_files=" "$package_script" | grep -q "ziplock.desktop"; then
        log_success "Packaging script verifies desktop file"
    else
        log_error "Packaging script does not verify desktop file"
        VERIFICATION_PASSED=false
    fi

    return 0
}

check_resource_files() {
    log_info "Checking resource files..."

    # Check desktop file
    local desktop_file="$PROJECT_ROOT/apps/linux/resources/ziplock.desktop"
    if ! check_file_exists "Desktop entry file" "$desktop_file"; then
        return 1
    fi

    # Check MIME type definition
    local mime_file="$PROJECT_ROOT/apps/linux/resources/mime/packages/ziplock.xml"
    if ! check_file_exists "MIME type definition file" "$mime_file"; then
        return 1
    fi

    return 0
}

check_system_integration() {
    log_info "Checking system integration (if ZipLock is installed)..."

    # Check if ZipLock is installed via pacman
    if command -v pacman &> /dev/null; then
        if pacman -Qi ziplock &> /dev/null; then
            log_success "ZipLock is installed via pacman"

            # Check if files are properly installed
            if [ -f "/usr/share/applications/ziplock.desktop" ]; then
                log_success "Desktop file is installed system-wide"
            else
                log_warning "Desktop file not found in system"
            fi

            if [ -f "/usr/share/mime/packages/ziplock.xml" ]; then
                log_success "MIME type definition is installed system-wide"
            else
                log_warning "MIME type definition not found in system"
            fi

            # Check if MIME type is registered
            if command -v xdg-mime &> /dev/null; then
                local mime_result
                if mime_result=$(xdg-mime query filetype test.7z 2>/dev/null); then
                    if [ "$mime_result" = "application/x-7z-compressed" ]; then
                        log_success "System correctly identifies .7z files"
                    else
                        log_warning "System identifies .7z files as: $mime_result"
                    fi
                fi
            fi
        else
            log_info "ZipLock not installed via pacman - skipping system checks"
        fi
    else
        log_warning "pacman not available - skipping system integration checks"
    fi

    return 0
}

test_package_build() {
    log_info "Testing package build (if makepkg is available)..."

    if ! command -v makepkg &> /dev/null; then
        log_warning "makepkg not available - skipping build test"
        return 0
    fi

    local pkg_dir="$PROJECT_ROOT/packaging/arch"
    if [ ! -f "$pkg_dir/PKGBUILD" ]; then
        log_warning "PKGBUILD not found - skipping build test"
        return 0
    fi

    # Test PKGBUILD syntax
    cd "$pkg_dir"
    if makepkg --printsrcinfo > /dev/null 2>&1; then
        log_success "PKGBUILD syntax is valid"
    else
        log_error "PKGBUILD syntax error detected"
        VERIFICATION_PASSED=false
    fi

    cd "$PROJECT_ROOT"
    return 0
}

print_manual_testing_instructions() {
    echo
    log_info "Manual Testing Instructions for Arch Linux:"
    echo "============================================="
    echo
    echo "1. Build and install the package:"
    echo "   ./scripts/build/package-arch.sh"
    echo "   sudo pacman -U target/ziplock-*.pkg.tar.xz"
    echo
    echo "2. Create a test .7z file:"
    echo "   7z a test.7z /etc/passwd"
    echo
    echo "3. Test file manager integration:"
    echo "   - Right-click the .7z file in your file manager"
    echo "   - Verify ZipLock appears in 'Open with...' menu"
    echo
    echo "4. Test command-line integration:"
    echo "   xdg-mime query filetype test.7z"
    echo "   xdg-mime query default application/x-7z-compressed"
    echo "   xdg-open test.7z"
    echo
    echo "5. Test in different environments:"
    echo "   - GNOME (Nautilus)"
    echo "   - KDE Plasma (Dolphin)"
    echo "   - XFCE (Thunar)"
    echo "   - i3/sway with file managers"
    echo
}

print_troubleshooting_tips() {
    echo
    log_info "Arch Linux Specific Troubleshooting:"
    echo "===================================="
    echo
    echo "If ZipLock doesn't appear in 'Open with...' menu:"
    echo
    echo "1. Update system databases:"
    echo "   sudo update-desktop-database"
    echo "   sudo update-mime-database /usr/share/mime"
    echo
    echo "2. Reinstall the package:"
    echo "   sudo pacman -R ziplock"
    echo "   sudo pacman -U target/ziplock-*.pkg.tar.xz"
    echo
    echo "3. Check package integrity:"
    echo "   pacman -Qkk ziplock"
    echo
    echo "4. Verify package contents:"
    echo "   pacman -Ql ziplock | grep -E '(desktop|mime)'"
    echo
    echo "5. Clear user caches:"
    echo "   rm -f ~/.local/share/mime/mimeinfo.cache"
    echo "   rm -f ~/.local/share/applications/mimeinfo.cache"
    echo
    echo "6. Check AUR build logs (if using AUR):"
    echo "   Check /tmp/makepkg-*/src/ziplock-*/makepkg.log"
    echo
    echo "7. Force MIME type association:"
    echo "   xdg-mime default ziplock.desktop application/x-7z-compressed"
    echo
}

print_aur_submission_notes() {
    echo
    log_info "AUR Submission Notes:"
    echo "===================="
    echo
    echo "When submitting to AUR, ensure:"
    echo "1. PKGBUILD version is updated correctly"
    echo "2. SHA256 checksum is current"
    echo "3. All dependencies are properly listed"
    echo "4. Install script handles all database updates"
    echo "5. .SRCINFO is regenerated:"
    echo "   makepkg --printsrcinfo > .SRCINFO"
    echo
}

main() {
    echo "ZipLock Arch Linux File Association Verification"
    echo "================================================"
    echo

    check_arch_system
    echo

    check_resource_files
    echo

    check_pkgbuild
    echo

    check_install_script
    echo

    check_packaging_script
    echo

    check_system_integration
    echo

    test_package_build
    echo

    if [ "$VERIFICATION_PASSED" = true ]; then
        log_success "All Arch Linux file association checks passed!"
        echo
        log_info "ZipLock should work correctly with .7z file associations on Arch Linux"
    else
        log_error "Some Arch Linux file association checks failed!"
        echo
        log_info "Please fix the issues above before building/installing the package"
        print_troubleshooting_tips
        exit 1
    fi

    print_manual_testing_instructions
    print_troubleshooting_tips
    print_aur_submission_notes
}

# Run main function
main "$@"
