#!/usr/bin/env bash

# ZipLock Format Check - Quick formatting check and fix
# This script runs the code formatting check from the GitHub workflow

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
    echo "Run code formatting check (same as GitHub CI)"
    echo ""
    echo "Options:"
    echo "  --fix             Automatically fix formatting issues"
    echo "  --check           Only check formatting (default)"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Check formatting"
    echo "  $0 --fix          # Fix formatting issues"
    echo "  $0 --check        # Check formatting (explicit)"
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
        --check)
            FIX_MODE=false
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

# Check if rustfmt is installed
if ! rustup component list --installed | grep -q "rustfmt"; then
    print_step "Installing rustfmt..."
    rustup component add rustfmt
fi

# Run formatting
if [[ "$FIX_MODE" == "true" ]]; then
    print_step "Fixing code formatting..."
    if cargo fmt --all; then
        print_success "Code formatting fixed"
        print_step "Review the changes:"
        echo "  git diff"
        echo "  git add ."
        echo "  git commit -m \"Fix code formatting\""
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
        echo ""
        print_step "To fix formatting issues, run:"
        echo "  $0 --fix"
        echo ""
        print_step "Or manually:"
        echo "  cargo fmt --all"
        exit 1
    fi
fi
