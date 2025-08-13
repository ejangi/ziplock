#!/usr/bin/env bash

# ZipLock CI Checks - Run GitHub workflow checks locally
# This script replicates the "Test Suite" job from the GitHub workflow

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}==>${NC} ${1}"
}

print_success() {
    echo -e "${GREEN}✓${NC} ${1}"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} ${1}"
}

print_error() {
    echo -e "${RED}✗${NC} ${1}"
}

# Function to show usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Run GitHub CI checks locally before pushing"
    echo ""
    echo "Options:"
    echo "  --skip-format     Skip formatting check"
    echo "  --skip-clippy     Skip Clippy linting"
    echo "  --skip-tests      Skip running tests"
    echo "  --fix-format      Automatically fix formatting issues"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                         # Run all checks"
    echo "  $0 --fix-format            # Fix formatting and run all checks"
    echo "  $0 --skip-tests            # Run format and clippy checks only"
    echo ""
}

# Parse command line arguments
SKIP_FORMAT=false
SKIP_CLIPPY=false
SKIP_TESTS=false
FIX_FORMAT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-format)
            SKIP_FORMAT=true
            shift
            ;;
        --skip-clippy)
            SKIP_CLIPPY=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --fix-format)
            FIX_FORMAT=true
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check if we're in the project root
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "frontend/linux" ]] || [[ ! -d "backend" ]]; then
    print_error "This script must be run from the ZipLock project root directory"
    exit 1
fi

print_step "Starting ZipLock CI checks..."
echo ""

# Check for required system dependencies
print_step "Checking system dependencies..."

# Check if pkg-config can find GTK4 (needed for compilation)
if ! pkg-config --exists gtk4 >/dev/null 2>&1; then
    print_warning "GTK4 not found via pkg-config"
    print_warning "You may need to install GTK4 development libraries:"
    print_warning "  Ubuntu/Debian: sudo apt-get install libgtk-4-dev libadwaita-1-dev"
    print_warning "  Fedora: sudo dnf install gtk4-devel libadwaita-devel"
    print_warning "  Arch: sudo pacman -S gtk4 libadwaita"
    echo ""
fi

# Check if Rust toolchain has required components
if ! rustup component list --installed | grep -q "rustfmt"; then
    print_warning "rustfmt not installed, installing..."
    rustup component add rustfmt
fi

if ! rustup component list --installed | grep -q "clippy"; then
    print_warning "clippy not installed, installing..."
    rustup component add clippy
fi

print_success "System dependencies check complete"
echo ""

# Step 1: Formatting check
if [[ "$SKIP_FORMAT" == "false" ]]; then
    if [[ "$FIX_FORMAT" == "true" ]]; then
        print_step "Fixing code formatting..."
        if cargo fmt --all; then
            print_success "Code formatting fixed"
        else
            print_error "Failed to fix code formatting"
            exit 1
        fi
    else
        print_step "Checking code formatting..."
        if cargo fmt --all -- --check; then
            print_success "Code formatting is correct"
        else
            print_error "Code formatting issues found"
            print_warning "Run with --fix-format to automatically fix formatting issues"
            exit 1
        fi
    fi
    echo ""
else
    print_warning "Skipping formatting check"
    echo ""
fi

# Step 2: Clippy linting
if [[ "$SKIP_CLIPPY" == "false" ]]; then
    print_step "Running Clippy linting..."

    # Backend clippy check
    print_step "  Checking backend..."
    if cargo clippy -p ziplock-backend --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code; then
        print_success "Backend Clippy check passed"
    else
        print_error "Backend Clippy check failed"
        exit 1
    fi

    # Shared library clippy check
    print_step "  Checking shared library..."
    if cargo clippy -p ziplock-shared --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code; then
        print_success "Shared library Clippy check passed"
    else
        print_error "Shared library Clippy check failed"
        exit 1
    fi

    # Frontend clippy check
    print_step "  Checking frontend (iced-gui features only)..."
    if cargo clippy -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog" --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code; then
        print_success "Frontend Clippy check passed"
    else
        print_error "Frontend Clippy check failed"
        exit 1
    fi

    print_success "All Clippy checks passed"
    echo ""
else
    print_warning "Skipping Clippy linting"
    echo ""
fi

# Step 3: Run tests
if [[ "$SKIP_TESTS" == "false" ]]; then
    print_step "Running tests..."

    # Backend tests
    print_step "  Testing backend..."
    if cargo test --verbose -p ziplock-backend; then
        print_success "Backend tests passed"
    else
        print_error "Backend tests failed"
        exit 1
    fi

    # Shared library tests
    print_step "  Testing shared library..."
    if cargo test --verbose -p ziplock-shared; then
        print_success "Shared library tests passed"
    else
        print_error "Shared library tests failed"
        exit 1
    fi

    # Frontend tests
    print_step "  Testing frontend (iced-gui features only)..."
    if cargo test --verbose -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog"; then
        print_success "Frontend tests passed"
    else
        print_error "Frontend tests failed"
        exit 1
    fi

    print_success "All tests passed"
    echo ""
else
    print_warning "Skipping tests"
    echo ""
fi

# Final summary
echo ""
print_success "All CI checks completed successfully! 🎉"
print_step "Your code is ready to push to GitHub"

# Show helpful commands
echo ""
echo "Next steps:"
echo "  git add ."
echo "  git commit -m \"Your commit message\""
echo "  git push"
echo ""
echo "Or to run individual checks:"
echo "  ./scripts/dev/run-ci-checks.sh --skip-tests     # Format and clippy only"
echo "  ./scripts/dev/run-ci-checks.sh --fix-format     # Auto-fix formatting"
echo ""
