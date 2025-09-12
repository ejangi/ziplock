#!/bin/bash
set -euo pipefail

# Test script to replicate GitHub workflow Arch packaging locally
# This helps debug the shell escaping issues we're seeing in CI

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

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi

    log_success "Docker is available"
}

# Create mock installation structure (like the workflow does)
create_mock_install_structure() {
    log_info "Creating mock installation structure..."

    cd "$PROJECT_ROOT"

    # Clean up any existing mock structure
    rm -rf target/install
    rm -rf target/x86_64-unknown-linux-gnu/release

    # Create the directories
    mkdir -p target/x86_64-unknown-linux-gnu/release
    mkdir -p target/install/usr/bin
    mkdir -p target/install/usr/lib
    mkdir -p target/install/etc/ziplock
    mkdir -p target/install/usr/share/applications
    mkdir -p target/install/usr/share/icons/hicolor/scalable/apps

    # Create mock binaries
    echo "#!/bin/bash\necho 'Mock ZipLock binary'\n" > target/x86_64-unknown-linux-gnu/release/ziplock
    echo "/* Mock shared library */" > target/x86_64-unknown-linux-gnu/release/libziplock_shared.so

    # Copy to install structure
    cp target/x86_64-unknown-linux-gnu/release/ziplock target/install/usr/bin/
    cp target/x86_64-unknown-linux-gnu/release/libziplock_shared.so target/install/usr/lib/

    # Set permissions
    chmod +x target/install/usr/bin/ziplock
    chmod 644 target/install/usr/lib/libziplock_shared.so

    # Create config file
    cat > target/install/etc/ziplock/config.yml << 'EOF'
# ZipLock Configuration (Unified FFI Architecture)
storage:
  backup_count: 3
  auto_backup: true

security:
  auto_lock_timeout: 900
  min_master_key_length: 12

ui:
  theme: "auto"
  font_size: 14

logging:
  level: "info"
  file: null
EOF

    # Copy resource files if they exist
    if [ -f "packaging/linux/resources/ziplock.desktop" ]; then
        cp packaging/linux/resources/ziplock.desktop target/install/usr/share/applications/
    elif [ -f "apps/desktop/resources/ziplock.desktop" ]; then
        cp apps/desktop/resources/ziplock.desktop target/install/usr/share/applications/
    fi

    if [ -f "apps/desktop/resources/icons/ziplock.svg" ]; then
        cp apps/desktop/resources/icons/ziplock.svg target/install/usr/share/icons/hicolor/scalable/apps/
    elif [ -f "assets/icons/ziplock-logo.svg" ]; then
        cp assets/icons/ziplock-logo.svg target/install/usr/share/icons/hicolor/scalable/apps/ziplock.svg
    fi

    log_success "Mock installation structure created"
    find target/install -type f | sort
}

# Test the exact command that's failing in GitHub Actions
test_arch_packaging() {
    log_info "Testing Arch packaging process..."

    cd "$PROJECT_ROOT"

    # Get current user and group IDs (like the workflow)
    USER_ID=$(id -u)
    GROUP_ID=$(id -g)

    log_info "Using USER_ID=$USER_ID, GROUP_ID=$GROUP_ID"

    # Run the exact Docker command from the workflow
    log_info "Running Docker command (this is where the error occurs)..."

    set -x  # Enable command tracing to see exactly what's being executed

    docker run --rm \
        -v "$PWD:/workspace" \
        -e USER_ID="$USER_ID" \
        -e GROUP_ID="$GROUP_ID" \
        ghcr.io/ejangi/ziplock/arch-builder:latest \
        bash -c "
            set -euo pipefail

            # Copy workspace to avoid git permission conflicts
            echo 'Copying workspace to avoid git permission conflicts...'
            # Use rsync to skip permission-denied files (like Cargo lock files)
            rsync -av \
                --exclude='target/debug/incremental/' \
                --exclude='target/debug/deps/' \
                --exclude='target/debug/build/' \
                --exclude='target/release/incremental/' \
                --exclude='target/release/deps/' \
                --exclude='target/release/build/' \
                --exclude='target/.rustc_info.json' \
                --exclude='target/CACHEDIR.TAG' \
                --exclude='.git/' \
                /workspace/ /tmp/build/
            cd /tmp/build

            # Validate copied structure
            if [ ! -f 'target/install/usr/bin/ziplock' ]; then
                echo 'ERROR: Binary not found in copied workspace'
                exit 1
            fi

            # Change ownership for build process
            sudo chown -R builder:builder /tmp/build

            # Create Arch package with proper PKGBUILD and .SRCINFO
            sudo -u builder bash -c '
                set -euo pipefail
                cd /tmp/build

                # Get version from Cargo.toml
                VERSION=\$(grep \"^version\" Cargo.toml | head -1 | sed \"s/.*= \\\"\\(.*\\)\\\"/\\1/\")
                echo \"Creating Arch package for version: \$VERSION\"

                # Create source archive manually
                mkdir -p target/src
                rm -rf target/src/*

                # Copy project files excluding build artifacts
                rsync -av \
                  --exclude=\".git/\" \
                  --exclude=\"target/\" \
                  --exclude=\"tests/results/\" \
                  --exclude=\"*.deb\" \
                  --exclude=\"*.pkg.tar.*\" \
                  --exclude=\".DS_Store\" \
                  ./ target/src/ziplock-\$VERSION/

                # Create source archive
                cd target/src
                tar -czf \"../ziplock-\$VERSION.tar.gz\" \"ziplock-\$VERSION/\"
                cd ../..

                # Calculate SHA256
                SHA256=\$(sha256sum \"target/ziplock-\$VERSION.tar.gz\" | cut -d\" \" -f1)
                echo \"Source archive SHA256: \$SHA256\"

                # Update PKGBUILD with current version and SHA256
                cd packaging/arch
                sed -i \"s/^pkgver=.*/pkgver=\$VERSION/\" PKGBUILD

                # Update sha256sums using printf approach to avoid escaping issues
                cp PKGBUILD PKGBUILD.backup
                grep -v \"^sha256sums=\" PKGBUILD.backup > PKGBUILD.tmp
                printf \"sha256sums=('%s')\n\" \"\$SHA256\" >> PKGBUILD.tmp
                mv PKGBUILD.tmp PKGBUILD
                rm PKGBUILD.backup

                # Generate .SRCINFO from updated PKGBUILD
                echo \"Generating .SRCINFO...\"
                makepkg --printsrcinfo > .SRCINFO

                echo \"Arch package creation completed successfully\"
            '

            echo 'Docker command completed successfully'
        " || {
            log_error "Docker command failed"
            exit 1
        }

    set +x  # Disable command tracing

    log_success "Arch packaging test completed"
}

# Test a simplified version to isolate the issue
test_simplified_commands() {
    log_info "Testing simplified commands to isolate the issue..."

    cd "$PROJECT_ROOT"

    # Test the sed command that's likely causing the issue
    log_info "Testing sed command in isolation..."

    # Create a test PKGBUILD
    cat > /tmp/test_PKGBUILD << 'EOF'
pkgname=ziplock
pkgver=0.2.5
pkgrel=1
sha256sums=('old_hash_here')
EOF

    # Test the sed command that's failing
    TEST_SHA256="abcd1234567890"

    log_info "Original PKGBUILD:"
    cat /tmp/test_PKGBUILD

    log_info "Testing sed command..."
    sed "s/^sha256sums=.*/sha256sums=('$TEST_SHA256')/" /tmp/test_PKGBUILD > /tmp/test_PKGBUILD.tmp
    mv /tmp/test_PKGBUILD.tmp /tmp/test_PKGBUILD

    log_info "Modified PKGBUILD:"
    cat /tmp/test_PKGBUILD

    # Now test it inside Docker
    log_info "Testing same command inside Docker..."
    docker run --rm \
        -v "$PWD:/workspace" \
        ghcr.io/ejangi/ziplock/arch-builder:latest \
        bash -c "
            TEST_SHA256='abcd1234567890'
            echo 'Testing sed command inside Docker...'
            echo 'sha256sums=(old_hash)' > /tmp/test
            sed \"s/^sha256sums=.*/sha256sums=('\$TEST_SHA256')/\" /tmp/test
        "
}

# Main function
main() {
    log_info "Starting local Arch packaging test..."

    echo "This script replicates the GitHub workflow Arch packaging process locally"
    echo "to help debug the shell escaping issues."
    echo

    # Check prerequisites
    check_docker

    # Create mock structure
    create_mock_install_structure

    echo
    log_info "Choose test to run:"
    echo "1) Full Docker test (replicates exact workflow command)"
    echo "2) Simplified test (isolate the sed command issue)"
    echo "3) Both tests"

    read -p "Enter choice (1-3): " choice

    case $choice in
        1)
            test_arch_packaging
            ;;
        2)
            test_simplified_commands
            ;;
        3)
            test_simplified_commands
            echo
            test_arch_packaging
            ;;
        *)
            log_error "Invalid choice"
            exit 1
            ;;
    esac

    log_success "Local test completed!"
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--help]"
        echo
        echo "This script replicates the GitHub workflow Arch packaging process locally"
        echo "to help debug shell escaping issues."
        echo
        echo "Prerequisites:"
        echo "  - Docker installed and running"
        echo "  - Internet access to pull the arch-builder image"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
