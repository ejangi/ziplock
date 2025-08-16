#!/bin/bash
set -euo pipefail

# ZipLock Containerized Local Testing Script
# Allows developers to test builds in the same containers used by CI

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Container image names
UBUNTU_IMAGE="ghcr.io/ejangi/ziplock/ubuntu-builder:latest"
ARCH_IMAGE="ghcr.io/ejangi/ziplock/arch-builder:latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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
    echo "Usage: $0 [OPTIONS] COMMAND"
    echo
    echo "Commands:"
    echo "  test           Run full test suite in Ubuntu container"
    echo "  build          Build binaries in Ubuntu container"
    echo "  package-deb    Create Debian package in Ubuntu container"
    echo "  package-arch   Create Arch package in Arch container"
    echo "  shell-ubuntu   Open interactive shell in Ubuntu container"
    echo "  shell-arch     Open interactive shell in Arch container"
    echo "  update-images  Pull latest container images"
    echo
    echo "Options:"
    echo "  --no-cache     Don't use Docker build cache"
    echo "  --clean        Clean target directory first"
    echo "  --help         Show this help message"
    echo
    echo "Examples:"
    echo "  $0 test                    # Run tests in Ubuntu container"
    echo "  $0 build --clean           # Clean build in Ubuntu container"
    echo "  $0 package-deb             # Create .deb package"
    echo "  $0 shell-ubuntu            # Interactive Ubuntu shell"
}

check_requirements() {
    log_info "Checking requirements..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi

    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "This script must be run from the ZipLock project directory."
        exit 1
    fi

    log_success "Requirements check passed"
}

update_images() {
    log_info "Updating container images..."

    log_info "Pulling Ubuntu builder image..."
    if ! docker pull "$UBUNTU_IMAGE" 2>/dev/null; then
        log_warning "Failed to pull Ubuntu image from registry. Building locally..."
        build_ubuntu_image_locally
    fi

    log_info "Pulling Arch builder image..."
    if ! docker pull "$ARCH_IMAGE" 2>/dev/null; then
        log_warning "Failed to pull Arch image from registry. Building locally..."
        build_arch_image_locally
    fi

    log_success "Image update completed"
}

build_ubuntu_image_locally() {
    log_info "Building Ubuntu builder image locally..."

    if [ ! -f ".github/docker/ubuntu-builder.Dockerfile" ]; then
        log_error "Ubuntu Dockerfile not found at .github/docker/ubuntu-builder.Dockerfile"
        exit 1
    fi

    docker build -f .github/docker/ubuntu-builder.Dockerfile -t "$UBUNTU_IMAGE" . || {
        log_error "Failed to build Ubuntu image locally"
        exit 1
    }

    log_success "Ubuntu builder image built successfully"
}

build_arch_image_locally() {
    log_info "Building Arch builder image locally..."

    # Create Arch Dockerfile if it doesn't exist
    mkdir -p .github/docker
    cat > .github/docker/arch-builder.Dockerfile << 'EOF'
FROM archlinux:latest

# Update system and install dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
      base-devel \
      rust \
      cargo \
      pkg-config \
      git \
      curl \
      file \
      fakeroot \
      openssl \
      xz \
      rsync \
      fontconfig \
      freetype2 \
      libx11 \
      libxft \
      glib2 \
      cairo \
      pango \
      gdk-pixbuf2 \
      atk \
      at-spi2-core \
      at-spi2-atk \
      gtk3 \
      gtk4 \
      libadwaita \
      && pacman -Scc --noconfirm

# Create non-root user for makepkg
RUN useradd -m -G wheel builder && \
    echo '%wheel ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# Set up environment
USER builder
WORKDIR /home/builder
ENV PATH="/home/builder/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version && cargo --version

CMD ["/bin/bash"]
EOF

    docker build -f .github/docker/arch-builder.Dockerfile -t "$ARCH_IMAGE" . || {
        log_error "Failed to build Arch image locally"
        exit 1
    }

    log_success "Arch builder image built successfully"
}

ensure_images_available() {
    log_info "Ensuring container images are available..."

    # Check if Ubuntu image exists locally
    if ! docker image inspect "$UBUNTU_IMAGE" >/dev/null 2>&1; then
        log_info "Ubuntu image not found locally, attempting to pull or build..."
        if ! docker pull "$UBUNTU_IMAGE" 2>/dev/null; then
            log_info "Pull failed, building Ubuntu image locally..."
            build_ubuntu_image_locally
        fi
    fi

    # Check if Arch image exists locally
    if ! docker image inspect "$ARCH_IMAGE" >/dev/null 2>&1; then
        log_info "Arch image not found locally, attempting to pull or build..."
        if ! docker pull "$ARCH_IMAGE" 2>/dev/null; then
            log_info "Pull failed, building Arch image locally..."
            build_arch_image_locally
        fi
    fi

    log_success "Container images are ready"
}

run_in_ubuntu() {
    local cmd="$1"
    local extra_args="${2:-}"

    log_info "Running command in Ubuntu container: $cmd"

    # Detect if we're in an interactive terminal
    local docker_flags="--rm"
    if [ -t 0 ] && [ -t 1 ]; then
        docker_flags="--rm -it"
    fi

    docker run $docker_flags \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        $extra_args \
        "$UBUNTU_IMAGE" \
        bash -c "$cmd"
}

run_in_arch() {
    local cmd="$1"
    local extra_args="${2:-}"

    log_info "Running command in Arch container: $cmd"

    # Detect if we're in an interactive terminal
    local docker_flags="--rm"
    if [ -t 0 ] && [ -t 1 ]; then
        docker_flags="--rm -it"
    fi

    docker run $docker_flags \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        $extra_args \
        "$ARCH_IMAGE" \
        bash -c "
            sudo chown -R builder:builder /workspace
            cd /workspace
            $cmd
        "
}

clean_target() {
    if [ "$CLEAN" = "true" ]; then
        log_info "Cleaning target directory..."
        rm -rf "$PROJECT_ROOT/target"
    fi
}

run_tests() {
    log_info "Running test suite in Ubuntu container..."
    ensure_images_available

    run_in_ubuntu "
        set -euo pipefail

        # Install dependencies if needed
        if ! pkg-config --exists gtk4; then
            echo 'GTK4 not found - this should not happen in the builder image'
            exit 1
        fi

        # Run formatting check
        echo 'Checking code formatting...'
        cargo fmt --all -- --check

        # Run clippy
        echo 'Running clippy...'
        cargo clippy -p ziplock-shared --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code
        cargo clippy -p ziplock-linux --no-default-features --features 'iced-gui,wayland-support,file-dialog' --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code

        # Run tests
        echo 'Running tests...'
        cargo test --verbose -p ziplock-shared
        cargo test --verbose -p ziplock-linux --no-default-features --features 'iced-gui,wayland-support,file-dialog'

        echo 'All tests passed!'
    "
}

build_binaries() {
    log_info "Building binaries in Ubuntu container..."
    ensure_images_available

    run_in_ubuntu "
        set -euo pipefail

        echo 'Building shared library...'
        cargo build --release --target x86_64-unknown-linux-gnu -p ziplock-shared --features c-api

        echo 'Building unified application...'
        cargo build --release --target x86_64-unknown-linux-gnu -p ziplock-linux --no-default-features --features 'iced-gui,wayland-support,file-dialog'

        echo 'Verifying built binaries...'
        ls -la target/x86_64-unknown-linux-gnu/release/ziplock
        ls -la target/x86_64-unknown-linux-gnu/release/libziplock_shared.so

        echo 'Build completed successfully!'
    "
}

package_debian() {
    log_info "Creating Debian package in Ubuntu container..."
    ensure_images_available

    # Make sure binaries exist
    if [ ! -f "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/ziplock" ]; then
        log_info "Binaries not found, building first..."
        build_binaries
    fi

    run_in_ubuntu "
        set -euo pipefail

        chmod +x scripts/build/package-deb.sh
        ./scripts/build/package-deb.sh --arch amd64

        echo 'Debian package created:'
        ls -la target/ziplock_*_amd64.deb
    "
}

package_arch() {
    log_info "Creating Arch package in Arch container..."
    ensure_images_available

    # Make sure binaries exist
    if [ ! -f "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/ziplock" ]; then
        log_info "Binaries not found, building first..."
        build_binaries
    fi

    run_in_arch "
        set -euo pipefail

        chmod +x scripts/build/package-arch.sh
        ./scripts/build/package-arch.sh --source-only

        echo 'Arch package created:'
        ls -la target/ziplock-*.tar.gz
        ls -la packaging/arch/PKGBUILD
    "
}

interactive_shell_ubuntu() {
    log_info "Opening interactive shell in Ubuntu container..."
    log_info "You're now in the same environment as CI builds"
    log_info "Project is mounted at /workspace"
    ensure_images_available

    # Force interactive mode for shell
    docker run --rm -it \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        "$UBUNTU_IMAGE" \
        bash
}

interactive_shell_arch() {
    log_info "Opening interactive shell in Arch container..."
    log_info "You're now in the same environment as CI builds"
    log_info "Project is mounted at /workspace"
    ensure_images_available

    # Force interactive mode for shell
    docker run --rm -it \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        "$ARCH_IMAGE" \
        bash -c "
            sudo chown -R builder:builder /workspace
            cd /workspace
            bash
        "
}

# Parse arguments
NO_CACHE=false
CLEAN=false
COMMAND=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        test|build|package-deb|package-arch|shell-ubuntu|shell-arch|update-images)
            COMMAND="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

if [ -z "$COMMAND" ]; then
    log_error "No command specified"
    print_usage
    exit 1
fi

# Main execution
cd "$PROJECT_ROOT"

check_requirements
clean_target

case "$COMMAND" in
    test)
        run_tests
        ;;
    build)
        build_binaries
        ;;
    package-deb)
        package_debian
        ;;
    package-arch)
        package_arch
        ;;
    shell-ubuntu)
        interactive_shell_ubuntu
        ;;
    shell-arch)
        interactive_shell_arch
        ;;
    update-images)
        update_images
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        exit 1
        ;;
esac

log_success "Command '$COMMAND' completed successfully!"
