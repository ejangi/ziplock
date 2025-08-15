#!/bin/bash
set -euo pipefail

# ZipLock Debian Package Creation Script
# Creates .deb packages for Ubuntu/Debian distributions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
PACKAGING_DIR="$PROJECT_ROOT/packaging/linux"

# Package configuration
PACKAGE_NAME="ziplock"
PACKAGE_VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | sed -n '1s/.*"\(.*\)".*/\1/p')}"
PACKAGE_ARCH="${PACKAGE_ARCH:-amd64}"
MAINTAINER="James Angus <james@ejangi.com>"
DESCRIPTION="A secure, portable password manager using encrypted 7z archives"
HOMEPAGE="https://github.com/ejangi/ziplock"

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
    log_info "Checking packaging dependencies..."

    local missing_deps=()

    if ! command -v dpkg-deb &> /dev/null; then
        missing_deps+=("dpkg-dev")
    fi

    if ! command -v fakeroot &> /dev/null; then
        missing_deps+=("fakeroot")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing packaging dependencies: ${missing_deps[*]}"
        log_info "Install with: sudo apt-get install ${missing_deps[*]}"
        exit 1
    fi

    log_success "Packaging dependencies satisfied"
}

verify_build() {
    log_info "Verifying build artifacts..."

    local install_dir="$BUILD_DIR/install"

    if [ ! -d "$install_dir" ]; then
        log_error "Installation directory not found: $install_dir"
        log_info "Run './scripts/build/build-linux.sh' first"
        exit 1
    fi

    # Check required files
    local required_files=(
        "$install_dir/usr/bin/ziplock"
        "$install_dir/etc/ziplock/config.yml"
        "$install_dir/usr/share/applications/ziplock.desktop"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Required file missing: $file"
            exit 1
        fi
    done

    # Special check for shared library (could be .so or .dylib)
    local shared_lib_found=false
    if [ -f "$install_dir/usr/lib/libziplock_shared.so" ]; then
        log_info "Found shared library (.so): $(ls -la "$install_dir/usr/lib/libziplock_shared.so")"
        shared_lib_found=true
    elif [ -f "$install_dir/usr/lib/libziplock_shared.dylib" ]; then
        log_info "Found shared library (.dylib): $(ls -la "$install_dir/usr/lib/libziplock_shared.dylib")"
        shared_lib_found=true
    fi

    if [ "$shared_lib_found" = false ]; then
        log_error "Required shared library missing: $install_dir/usr/lib/libziplock_shared.so"
        log_info "Available files in usr/lib:"
        ls -la "$install_dir/usr/lib/" 2>/dev/null || log_warning "usr/lib directory not found"
        log_info "Searching for any ziplock libraries:"
        find "$install_dir" -name "*ziplock*" -type f 2>/dev/null || log_warning "No ziplock files found"
        exit 1
    fi

    log_success "Build artifacts verified including shared library"
}

calculate_installed_size() {
    local install_dir="$BUILD_DIR/install"
    local size_kb=$(du -sk "$install_dir" | cut -f1)
    echo "$size_kb"
}

create_debian_control() {
    local deb_dir="$1"
    local installed_size="$2"

    mkdir -p "$deb_dir/DEBIAN"

    cat > "$deb_dir/DEBIAN/control" << EOF
Package: $PACKAGE_NAME
Version: $PACKAGE_VERSION
Section: utils
Priority: optional
Architecture: $PACKAGE_ARCH
Installed-Size: $installed_size
Depends: libc6, libfontconfig1, libfreetype6, libx11-6, libxft2, liblzma5
Recommends: gnome-keyring | kde-wallet-kf5
Suggests: firejail
Maintainer: $MAINTAINER
Description: $DESCRIPTION
 ZipLock provides a clean, modern interface for managing your passwords
 with strong encryption and cross-platform compatibility. Your encrypted
 password database is stored as a single 7z file that you can store
 anywhere - on your local drive, cloud storage, or USB drive.
 .
 Key features:
  * AES-256 encryption with Argon2 key derivation
  * Zero-knowledge architecture
  * Full-text search across credentials
  * TOTP two-factor authentication support
  * Browser integration capabilities
  * Cross-platform file sync compatibility
Homepage: $HOMEPAGE
EOF



    log_success "Created Debian control file"
}

create_postinst_script() {
    local deb_dir="$1"

    cat > "$deb_dir/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# ZipLock post-installation script

# Create ziplock user and group for the backend service
if ! getent group ziplock >/dev/null; then
    addgroup --system ziplock
fi

if ! getent passwd ziplock >/dev/null; then
    adduser --system --home /var/lib/ziplock --shell /usr/sbin/nologin \
            --gecos "ZipLock Backend Service" --ingroup ziplock ziplock
fi

# Set correct ownership and permissions
chown -R ziplock:ziplock /var/lib/ziplock
chmod 755 /var/lib/ziplock

# Set up configuration directory
chown root:ziplock /etc/ziplock
chmod 750 /etc/ziplock
chown root:ziplock /etc/ziplock/config.yml
chmod 640 /etc/ziplock/config.yml

# No service to enable - unified FFI architecture
echo "ZipLock unified application ready to use."
echo "No background service required with the new FFI architecture."

# Update desktop database (only if frontend is installed)
if [ -f /usr/share/applications/ziplock.desktop ]; then
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications
    fi
fi

# Update icon cache (only if frontend is installed)
if [ -f /usr/share/icons/hicolor/scalable/apps/ziplock.svg ]; then
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi
fi

echo "ZipLock has been installed successfully!"
echo "You can now launch ZipLock from your applications menu or run 'ziplock' in terminal."
echo "No background service is required - ZipLock uses a unified FFI architecture."

#DEBHELPER#

exit 0
EOF

    chmod 755 "$deb_dir/DEBIAN/postinst"
    log_success "Created post-installation script"
}

create_prerm_script() {
    local deb_dir="$1"

    cat > "$deb_dir/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e

# ZipLock pre-removal script

case "$1" in
    remove|upgrade|deconfigure)
        # No service to stop - unified FFI architecture
        echo "Preparing to remove ZipLock unified application..."
        ;;
esac
    failed-upgrade)
        ;;
    *)
        echo "prerm called with unknown argument \`$1'" >&2
        exit 1
        ;;
esac

#DEBHELPER#

exit 0
EOF

    chmod 755 "$deb_dir/DEBIAN/prerm"
    log_success "Created pre-removal script"
}

create_postrm_script() {
    local deb_dir="$1"

    cat > "$deb_dir/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

# ZipLock post-removal script

case "$1" in
    purge)
        # Remove user and group on purge
        if getent passwd ziplock >/dev/null; then
            deluser --system ziplock || true
        fi

        if getent group ziplock >/dev/null; then
            delgroup --system ziplock || true
        fi

        # Remove state directories on purge
        rm -rf /var/lib/ziplock
        rm -rf /etc/ziplock

        # Update desktop database (if it exists)
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database /usr/share/applications 2>/dev/null || true
        fi

        # Update icon cache (if it exists)
        if command -v gtk-update-icon-cache >/dev/null 2>&1; then
            gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
        fi
        ;;
    remove|upgrade|failed-upgrade|abort-install|abort-upgrade|disappear)
        # No systemd service to reload - unified FFI architecture
        echo "ZipLock removal completed."
        ;;
    *)
        echo "postrm called with unknown argument \`$1'" >&2
        exit 1
        ;;
esac

#DEBHELPER#

exit 0
EOF

    chmod 755 "$deb_dir/DEBIAN/postrm"
    log_success "Created post-removal script"
}

create_copyright_file() {
    local deb_dir="$1"

    mkdir -p "$deb_dir/usr/share/doc/$PACKAGE_NAME"

    cat > "$deb_dir/usr/share/doc/$PACKAGE_NAME/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: ZipLock
Upstream-Contact: James Angus <james@ejangi.com>
Source: https://github.com/ejangi/ziplock

Files: *
Copyright: 2024 James Angus <james@ejangi.com>
License: Apache-2.0

License: Apache-2.0
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 .
 https://www.apache.org/licenses/LICENSE-2.0
 .
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 .
 On Debian systems, the complete text of the Apache version 2.0 license
 can be found in "/usr/share/common-licenses/Apache-2.0".
EOF

    log_success "Created copyright file"
}

create_changelog() {
    local deb_dir="$1"

    cat > "$deb_dir/usr/share/doc/$PACKAGE_NAME/changelog.Debian" << EOF
$PACKAGE_NAME ($PACKAGE_VERSION) stable; urgency=medium

  * Initial Debian package release
  * Secure password manager with AES-256 encryption
  * Backend service with systemd integration
  * GTK/Iced-based frontend application
  * Desktop integration with .desktop file

 -- $MAINTAINER  $(date -R)
EOF

    gzip -9 "$deb_dir/usr/share/doc/$PACKAGE_NAME/changelog.Debian"
    log_success "Created changelog"
}

build_package() {
    local deb_dir="$BUILD_DIR/deb"
    local package_file="$BUILD_DIR/${PACKAGE_NAME}_${PACKAGE_VERSION}_${PACKAGE_ARCH}.deb"

    log_info "Creating Debian package structure..."

    # Clean and create package directory
    rm -rf "$deb_dir"
    mkdir -p "$deb_dir"

    # Copy installation files
    cp -r "$BUILD_DIR/install"/* "$deb_dir/"

    # Calculate installed size
    local installed_size=$(calculate_installed_size)
    log_info "Calculated installed size: ${installed_size}KB"

    # Create Debian package files
    create_debian_control "$deb_dir" "$installed_size"
    create_postinst_script "$deb_dir"
    create_prerm_script "$deb_dir"
    create_postrm_script "$deb_dir"
    create_copyright_file "$deb_dir"
    create_changelog "$deb_dir"

    # Set correct permissions
    find "$deb_dir" -type f -name "*.so*" -exec chmod 644 {} \; 2>/dev/null || true
    find "$deb_dir" -type f -path "*/bin/*" -exec chmod 755 {} \;
    [ -f "$deb_dir/usr/share/applications/ziplock.desktop" ] && chmod 644 "$deb_dir/usr/share/applications/ziplock.desktop"
    [ -f "$deb_dir/usr/lib/libziplock_shared.so" ] && chmod 644 "$deb_dir/usr/lib/libziplock_shared.so"
    [ -f "$deb_dir/usr/lib/libziplock_shared.dylib" ] && chmod 644 "$deb_dir/usr/lib/libziplock_shared.dylib"

    # Verify shared library is present before building package
    log_info "Verifying shared library presence in package structure..."
    local shared_lib_in_package=false
    if [ -f "$deb_dir/usr/lib/libziplock_shared.so" ]; then
        log_info "Shared library (.so) found in package: $(ls -la "$deb_dir/usr/lib/libziplock_shared.so")"
        shared_lib_in_package=true
    elif [ -f "$deb_dir/usr/lib/libziplock_shared.dylib" ]; then
        log_info "Shared library (.dylib) found in package: $(ls -la "$deb_dir/usr/lib/libziplock_shared.dylib")"
        shared_lib_in_package=true
    fi

    if [ "$shared_lib_in_package" = false ]; then
        log_error "Shared library missing from package structure"
        log_info "Contents of package usr/lib directory:"
        ls -la "$deb_dir/usr/lib/" 2>/dev/null || log_warning "usr/lib directory not found in package"
        log_info "All files in package:"
        find "$deb_dir" -type f | head -20
        exit 1
    fi

    # Debug: Show package structure before building
    log_info "Package structure summary:"
    find "$deb_dir" -type f | wc -l | xargs -I {} echo "  Total files: {}"
    find "$deb_dir" -type d | wc -l | xargs -I {} echo "  Total directories: {}"

    log_info "Building Debian package..."

    # Build the package with better error handling and output capture
    local build_output
    if build_output=$(fakeroot dpkg-deb --build "$deb_dir" "$package_file" 2>&1); then
        log_info "Package build output: $build_output"
    else
        local exit_code=$?
        log_error "Package creation failed with exit code: $exit_code"
        log_error "Build output: $build_output"

        # Show debug info
        log_info "Debug: Contents of deb directory:"
        debug_contents=$(find "$deb_dir" -type f | sed -n '1,20p') || debug_contents="Could not list debug contents"
        echo "$debug_contents"
        log_info "Debug: DEBIAN control files:"
        ls -la "$deb_dir/DEBIAN/" || true
        exit 1
    fi

    if [ ! -f "$package_file" ]; then
        log_error "Package file was not created: $package_file"
        exit 1
    fi

    log_success "Debian package created: $package_file"

    # Verify package
    log_info "Verifying package..."
    if ! dpkg-deb --info "$package_file" 2>&1; then
        log_error "Package verification failed"
        exit 1
    fi

    # Show package contents safely without risking broken pipe
    log_info "Package contents (first 20 files):"
    package_contents=$(dpkg-deb --contents "$package_file" 2>/dev/null | sed -n '1,20p') || package_contents="Could not list package contents"
    echo "$package_contents"

    local package_size=$(du -h "$package_file" | cut -f1)
    log_success "Package verification completed - Size: $package_size"

    return 0
}

print_package_info() {
    local package_file="$BUILD_DIR/${PACKAGE_NAME}_${PACKAGE_VERSION}_${PACKAGE_ARCH}.deb"

    echo
    log_success "Debian package created successfully!"
    echo
    echo "Package Information:"
    echo "==================="
    echo "Name: $PACKAGE_NAME"
    echo "Version: $PACKAGE_VERSION"
    echo "Architecture: $PACKAGE_ARCH"
    echo "File: $package_file"
    echo "Size: $(du -h "$package_file" | cut -f1)"
    echo
    echo "Installation:"
    echo "  sudo dpkg -i $package_file"
    echo "  sudo apt-get install -f  # Fix any dependency issues"
    echo
    echo "Testing:"
    echo "  # Install in a clean environment (Docker recommended)"
    echo "  docker run -it --rm ubuntu:22.04 bash"
    echo "  # Copy .deb file and install"
    echo
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --arch ARCH          Package architecture [default: amd64]"
    echo "  --version VERSION    Override package version"
    echo "  --output-dir DIR     Output directory for .deb file [default: target/]"
    echo "  --help              Show this help message"
    echo
    echo "Environment Variables:"
    echo "  PACKAGE_ARCH        Override package architecture"
    echo "  VERSION            Override version string"
}

main() {
    local output_dir="$BUILD_DIR"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --arch)
                PACKAGE_ARCH="$2"
                shift 2
                ;;
            --version)
                PACKAGE_VERSION="$2"
                shift 2
                ;;
            --output-dir)
                output_dir="$2"
                shift 2
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

    log_info "Starting Debian package creation..."
    log_info "Package: $PACKAGE_NAME v$PACKAGE_VERSION ($PACKAGE_ARCH)"

    check_dependencies
    verify_build
    build_package
    print_package_info
}

# Run main function with all arguments
main "$@"
