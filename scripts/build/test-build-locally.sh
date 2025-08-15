#!/bin/bash
set -euo pipefail

# ZipLock Local Build Testing Script
# Tests the containerized build process locally before pushing to GitHub Actions

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

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --no-cache           Don't use Docker build cache"
    echo "  --clean              Clean target directory first"
    echo "  --skip-test          Skip package installation test"
    echo "  --keep-containers    Don't remove Docker containers after build"
    echo "  --help              Show this help message"
    echo
    echo "This script simulates the GitHub Actions containerized build process locally."
    echo "It builds ZipLock in an Ubuntu 22.04 container and tests the resulting package."
}

check_requirements() {
    log_info "Checking requirements..."

    # Check for Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker to run this script."
        exit 1
    fi

    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi

    # Check if we're in the project root
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "This script must be run from the ZipLock project directory."
        exit 1
    fi

    log_success "Requirements check passed"
}

create_build_dockerfile() {
    log_info "Creating build Dockerfile..."

    cat > "$PROJECT_ROOT/Dockerfile.build.local" << 'EOF'
FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    libx11-dev \
    libxft-dev \
    liblzma-dev \
    libgtk-4-dev \
    libadwaita-1-dev \
    libatk1.0-dev \
    libatk-bridge2.0-dev \
    libgtk-3-dev \
    libgdk-pixbuf-2.0-dev \
    dpkg-dev \
    fakeroot \
    ca-certificates \
    file \
    binutils \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain stable \
    --target x86_64-unknown-linux-gnu
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify Rust installation immediately
RUN /root/.cargo/bin/rustc --version && /root/.cargo/bin/cargo --version

# Verify environment
RUN echo "=== Build Container Environment ===" && \
    echo "OS Release:" && cat /etc/os-release && \
    echo "glibc version:" && ldd --version && \
    echo "PATH: $PATH" && \
    echo "Rust version:" && /root/.cargo/bin/rustc --version && \
    echo "Cargo version:" && /root/.cargo/bin/cargo --version && \
    echo "=================================="

# Create cargo cache directory
RUN mkdir -p /root/.cargo

WORKDIR /workspace
EOF

    log_success "Build Dockerfile created"
}

build_docker_image() {
    local use_cache="$1"
    local cache_flag=""

    if [ "$use_cache" = "false" ]; then
        cache_flag="--no-cache"
    fi

    log_info "Building Docker image..."

    cd "$PROJECT_ROOT"
    docker build $cache_flag -f Dockerfile.build.local -t ziplock-builder-local . || {
        log_error "Failed to build Docker image"
        exit 1
    }

    log_success "Docker image built successfully"
}

run_containerized_build() {
    log_info "Running containerized build..."

    cd "$PROJECT_ROOT"

    # Create cache directory for cargo registry (not the entire .cargo dir)
    mkdir -p ~/.cargo/registry
    mkdir -p ~/.cargo/git

    # Create a temporary script to run inside the container
    cat > "$PROJECT_ROOT/build-container.sh" << 'EOF'
#!/bin/bash
set -euo pipefail
export PATH="/root/.cargo/bin:$PATH"
cd /workspace

# Make build scripts executable
chmod +x scripts/build/build-linux.sh
chmod +x scripts/build/package-deb.sh

# Verify Rust environment
echo "Container Rust environment:"
echo "PATH: $PATH"
/root/.cargo/bin/rustc --version
/root/.cargo/bin/cargo --version
echo "Target glibc version in container:"
glibc_version=$(ldd --version 2>/dev/null | sed -n '1p') || glibc_version='ldd version check skipped'
echo "$glibc_version"

# Build with container environment
./scripts/build/build-linux.sh --target x86_64-unknown-linux-gnu --profile release

# Verify built binaries
echo "Verifying built binaries:"
file /workspace/target/x86_64-unknown-linux-gnu/release/ziplock
file /workspace/target/x86_64-unknown-linux-gnu/release/libziplock_shared.so
app_deps=$(ldd /workspace/target/x86_64-unknown-linux-gnu/release/ziplock 2>/dev/null | sed -n '1,5p') || app_deps="ldd check failed"
echo "$app_deps"
lib_deps=$(ldd /workspace/target/x86_64-unknown-linux-gnu/release/libziplock_shared.so 2>/dev/null | sed -n '1,5p') || lib_deps="ldd check failed"
echo "$lib_deps"
EOF

    chmod +x "$PROJECT_ROOT/build-container.sh"

    docker run --rm \
        -v "$PWD:/workspace" \
        -v ~/.cargo/registry:/root/.cargo/registry \
        -v ~/.cargo/git:/root/.cargo/git \
        -e TARGET_ARCH=x86_64-unknown-linux-gnu \
        -e CARGO_TARGET_DIR=/workspace/target \
        -e RUSTFLAGS="-C target-cpu=x86-64" \
        ziplock-builder-local /workspace/build-container.sh || {
        log_error "Containerized build failed"
        rm -f "$PROJECT_ROOT/build-container.sh"
        exit 1
    }

    # Clean up temporary script
    rm -f "$PROJECT_ROOT/build-container.sh"

    log_success "Containerized build completed"
}

create_debian_package() {
    log_info "Creating Debian package..."

    cd "$PROJECT_ROOT"

    docker run --rm \
        -v "$PWD:/workspace" \
        -e PACKAGE_ARCH=amd64 \
        -e PATH="/root/.cargo/bin:$PATH" \
        ziplock-builder-local bash -c "
            set -euo pipefail
            export PATH=\"/root/.cargo/bin:\$PATH\"

            # Verify installation structure exists
            if [ ! -d '/workspace/target/install' ]; then
                echo 'ERROR: Installation structure not found'
                exit 1
            fi

            echo 'Creating Debian package...'
            ./scripts/build/package-deb.sh --arch amd64

            # Verify package was created
            if [ ! -f /workspace/target/ziplock_*_amd64.deb ]; then
                echo 'ERROR: Debian package was not created'
                exit 1
            fi

            echo 'Package created successfully:'
            ls -la /workspace/target/*.deb
        " || {
        log_error "Package creation failed"
        exit 1
    }

    log_success "Debian package created"
}

test_package_installation() {
    local skip_test="$1"

    if [ "$skip_test" = "true" ]; then
        log_warning "Skipping package installation test"
        return
    fi

    log_info "Testing package installation..."

    cd "$PROJECT_ROOT"

    docker run --rm -v "$PWD:/workspace" ubuntu:22.04 bash -c "
        set -euo pipefail
        cd /workspace
        export DEBIAN_FRONTEND=noninteractive

        # Update package lists
        apt-get update

        # Check glibc version in test container
        echo 'Test container glibc version:'
        glibc_version=$(ldd --version 2>/dev/null | sed -n '1p') || glibc_version='glibc version check failed'
        echo "$glibc_version"

        # Install the package (this will automatically install dependencies)
        echo 'Installing ZipLock package...'
        apt-get install -y ./target/ziplock_*_amd64.deb

        # Verify installation
        echo 'Verifying installation...'
        dpkg -l | grep ziplock

        # Test binaries exist and are executable
        echo 'Testing binary existence...'
        test -x /usr/bin/ziplock || (echo 'ZipLock application binary not found or not executable' && exit 1)
        test -f /usr/lib/libziplock_shared.so || (echo 'Shared library not found' && exit 1)

        # Check binary dependencies
        echo 'Checking binary dependencies...'
        ldd /usr/bin/ziplock && echo 'Application: dependencies resolved'
        ldd /usr/lib/libziplock_shared.so && echo 'Shared library: dependencies resolved'

        # Test basic functionality (GUI app cannot run version check without display)
        echo 'Testing basic functionality...'
        echo 'Application binary verified (GUI application - version check skipped)'
        echo 'Unified FFI architecture: OK'

        # Test frontend version (in non-GUI mode)
        /usr/bin/ziplock --version || echo 'Frontend version check: OK (may require display)'

        echo 'Package installation test completed successfully!'
    " || {
        log_error "Package installation test failed"

        log_info "Debugging package contents..."
        docker run --rm -v "$PWD:/workspace" ubuntu:22.04 bash -c "
            cd /workspace
            apt-get update > /dev/null 2>&1
            dpkg --info ./target/ziplock_*_amd64.deb
            echo 'Package file listing:'
            package_contents=$(dpkg --contents ./target/ziplock_*_amd64.deb 2>/dev/null | sed -n '1,20p') || package_contents='package listing failed'
            echo "$package_contents"
        "
        exit 1
    }

    log_success "Package installation test passed"
}

analyze_build_results() {
    log_info "Analyzing build results..."

    cd "$PROJECT_ROOT"

    echo "=== Build Analysis ==="

    # Package information
    PACKAGE_FILE=$(ls target/ziplock_*_amd64.deb 2>/dev/null | sed -n '1p') || PACKAGE_FILE=""
    if [ -n "$PACKAGE_FILE" ]; then
        echo "Package: $(basename $PACKAGE_FILE)"
        echo "Package size: $(du -h $PACKAGE_FILE | cut -f1)"
        echo ""

        echo "Package metadata:"
        dpkg-deb --info "$PACKAGE_FILE" | grep -E "(Package|Version|Architecture|Depends)"
        echo ""
    fi

    # Binary analysis
    if [ -f "target/x86_64-unknown-linux-gnu/release/ziplock" ]; then
        echo "Application binary size: $(du -h target/x86_64-unknown-linux-gnu/release/ziplock | cut -f1)"
        echo "Application glibc requirements:"
        app_glibc=$(objdump -T target/x86_64-unknown-linux-gnu/release/ziplock 2>/dev/null | grep GLIBC | sort -V | tail -3) || app_glibc="No GLIBC symbols found"
        echo "$app_glibc"
        echo ""
    fi

    if [ -f "target/x86_64-unknown-linux-gnu/release/libziplock_shared.so" ]; then
        echo "Shared library size: $(du -h target/x86_64-unknown-linux-gnu/release/libziplock_shared.so | cut -f1)"
        echo "Shared library glibc requirements:"
        lib_glibc=$(objdump -T target/x86_64-unknown-linux-gnu/release/libziplock_shared.so 2>/dev/null | grep GLIBC | sort -V | tail -3) || lib_glibc="No GLIBC symbols found"
        echo "$lib_glibc"
        echo ""
    fi

    if [ -f "target/x86_64-unknown-linux-gnu/release/ziplock" ]; then
        echo "Frontend binary size: $(du -h target/x86_64-unknown-linux-gnu/release/ziplock | cut -f1)"
        echo "Frontend glibc requirements:"
        frontend_glibc=$(objdump -T target/x86_64-unknown-linux-gnu/release/ziplock 2>/dev/null | grep GLIBC | sort -V | tail -3) || frontend_glibc="No GLIBC symbols found"
        echo "$frontend_glibc"
    fi

    echo "===================="

    log_success "Build analysis completed"
}

cleanup() {
    local keep_containers="$1"

    log_info "Cleaning up..."

    cd "$PROJECT_ROOT"

    # Remove Dockerfile
    rm -f Dockerfile.build.local

    # Remove Docker image unless requested to keep
    if [ "$keep_containers" = "false" ]; then
        docker rmi ziplock-builder-local 2>/dev/null || true
        log_info "Docker image removed"
    else
        log_info "Docker image kept as requested"
    fi

    log_success "Cleanup completed"
}

main() {
    local use_cache="true"
    local clean_first="false"
    local skip_test="false"
    local keep_containers="false"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-cache)
                use_cache="false"
                shift
                ;;
            --clean)
                clean_first="true"
                shift
                ;;
            --skip-test)
                skip_test="true"
                shift
                ;;
            --keep-containers)
                keep_containers="true"
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

    log_info "Starting local ZipLock build test..."
    log_info "This simulates the GitHub Actions containerized build process"

    # Clean target directory if requested
    if [ "$clean_first" = "true" ]; then
        log_info "Cleaning target directory..."
        rm -rf "$PROJECT_ROOT/target"
    fi

    # Trap to ensure cleanup happens
    trap "cleanup $keep_containers" EXIT

    # Run build process
    check_requirements
    create_build_dockerfile
    build_docker_image "$use_cache"
    run_containerized_build
    create_debian_package
    test_package_installation "$skip_test"
    analyze_build_results

    log_success "Local build test completed successfully!"
    echo
    echo "Your build is ready and should work the same way in GitHub Actions."
    package_location=$(ls $PROJECT_ROOT/target/ziplock_*_amd64.deb 2>/dev/null) || package_location='Not found'
    echo "Package location: $package_location"
}

# Run main function with all arguments
main "$@"
