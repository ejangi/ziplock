#!/usr/bin/env bash

# ZipLock Clippy Check - Quick linting check
# This script runs only the Clippy checks from the GitHub workflow

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

print_error() {
    echo -e "${RED}✗${NC} ${1}"
}

# Function to show usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Run Clippy linting checks (same as GitHub CI)"
    echo ""
    echo "Options:"
    echo "  --fix             Run clippy with --fix to automatically fix issues"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Run clippy checks"
    echo "  $0 --fix          # Run clippy with automatic fixes"
    echo ""
}

# Parse command line arguments
FIX_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --fix)
            FIX_MODE=true
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

# Set clippy arguments
CLIPPY_ARGS="--all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code"
if [[ "$FIX_MODE" == "true" ]]; then
    CLIPPY_ARGS="--fix --allow-dirty --allow-staged ${CLIPPY_ARGS}"
    print_step "Running Clippy with automatic fixes..."
else
    print_step "Running Clippy linting checks..."
fi

echo ""

# Check if clippy is installed
if ! rustup component list --installed | grep -q "clippy"; then
    print_step "Installing clippy..."
    rustup component add clippy
fi

# Backend clippy check
print_step "Checking backend..."
if cargo clippy -p ziplock-backend $CLIPPY_ARGS; then
    print_success "Backend Clippy check passed"
else
    print_error "Backend Clippy check failed"
    exit 1
fi
echo ""

# Shared library clippy check
print_step "Checking shared library..."
if cargo clippy -p ziplock-shared $CLIPPY_ARGS; then
    print_success "Shared library Clippy check passed"
else
    print_error "Shared library Clippy check failed"
    exit 1
fi
echo ""

# Frontend clippy check
print_step "Checking frontend (iced-gui features only)..."
if cargo clippy -p ziplock-linux --no-default-features --features "iced-gui,wayland-support,file-dialog" $CLIPPY_ARGS; then
    print_success "Frontend Clippy check passed"
else
    print_error "Frontend Clippy check failed"
    exit 1
fi
echo ""

print_success "All Clippy checks passed! 🎉"

if [[ "$FIX_MODE" == "true" ]]; then
    echo ""
    print_step "Clippy fixes have been applied."
    print_step "Review the changes and commit them:"
    echo "  git diff"
    echo "  git add ."
    echo "  git commit -m \"Fix clippy warnings\""
fi
