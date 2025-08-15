#!/usr/bin/env bash

# ZipLock Pre-Push Checks - Quick validation before pushing
# This script runs the most common checks developers need before pushing code

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
    echo "Quick pre-push validation (format + clippy only, no tests)"
    echo ""
    echo "Options:"
    echo "  --fix             Automatically fix formatting and clippy issues"
    echo "  --full            Run full CI checks including tests"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Quick checks (format + clippy)"
    echo "  $0 --fix          # Fix issues automatically"
    echo "  $0 --full         # Run complete CI test suite"
    echo ""
    echo "This script is designed for fast pre-push validation."
    echo "For comprehensive testing, use: ./scripts/dev/run-ci-checks.sh"
}

# Parse command line arguments
FIX_MODE=false
FULL_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --fix)
            FIX_MODE=true
            shift
            ;;
        --full)
            FULL_MODE=true
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
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "apps/linux" ]] || [[ ! -d "shared" ]]; then
    print_error "This script must be run from the ZipLock project root directory"
    exit 1
fi

print_step "ZipLock Pre-Push Checks"
echo ""

# If full mode, delegate to comprehensive CI script
if [[ "$FULL_MODE" == "true" ]]; then
    print_step "Running full CI test suite..."
    if [[ "$FIX_MODE" == "true" ]]; then
        exec ./scripts/dev/run-ci-checks.sh --fix-format
    else
        exec ./scripts/dev/run-ci-checks.sh
    fi
fi

# Quick mode: format + clippy only
print_step "Running quick pre-push checks (format + clippy)..."
echo ""

# Step 1: Format check
if [[ "$FIX_MODE" == "true" ]]; then
    print_step "Fixing code formatting..."
    ./scripts/dev/run-format.sh --fix
else
    print_step "Checking code formatting..."
    ./scripts/dev/run-format.sh --check
fi
echo ""

# Step 2: Clippy check
if [[ "$FIX_MODE" == "true" ]]; then
    print_step "Running Clippy with fixes..."
    ./scripts/dev/run-clippy.sh --fix
else
    print_step "Running Clippy checks..."
    ./scripts/dev/run-clippy.sh
fi

echo ""
print_success "Quick pre-push checks completed! 🚀"

if [[ "$FIX_MODE" == "true" ]]; then
    echo ""
    print_step "Fixes have been applied. Review the changes:"
    echo "  git diff"
    echo "  git add ."
    echo "  git commit -m \"Fix formatting and clippy issues\""
    echo ""
fi

print_step "Your code should now pass GitHub CI checks"
print_warning "Note: This script skips tests for speed. Run with --full for complete validation."
echo ""
echo "Next steps:"
echo "  git add ."
echo "  git commit -m \"Your commit message\""
echo "  git push"
echo ""
echo "For comprehensive testing:"
echo "  ./scripts/dev/run-ci-checks.sh    # Full CI suite with tests"
echo ""
