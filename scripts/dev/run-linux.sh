#!/usr/bin/env bash

# ZipLock Launch Script
# This script launches the unified ZipLock application (FFI-based architecture)

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse command line arguments
NO_BUILD=false
DEBUG=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-build|-n)
            NO_BUILD=true
            shift
            ;;
        --debug|-d)
            DEBUG=true
            shift
            ;;
        --help|-h)
            echo "ZipLock Development Launcher (Unified FFI Architecture)"
            echo
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --no-build, -n    Skip building and run with existing binary"
            echo "  --debug, -d       Run with debug logging enabled"
            echo "  --help, -h        Show this help message"
            echo
            echo "This script builds and runs the unified ZipLock application."
            echo "No separate backend daemon is needed with the new FFI architecture."
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            echo "Usage: $0 [--no-build|-n] [--debug|-d] [--help|-h]"
            exit 1
            ;;
    esac
done

# Get script directory and project root (two levels up from scripts/dev)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${BLUE}🔐 ZipLock Development Launcher (Unified FFI)${NC}"
echo -e "${BLUE}=============================================${NC}"

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}👋 ZipLock closed${NC}"
    exit 0
}

# Set up signal handlers for graceful shutdown
trap cleanup SIGINT SIGTERM

# Define paths
FRONTEND_BIN="$PROJECT_ROOT/target/release/ziplock"
SHARED_LIB_DIR="$PROJECT_ROOT/target/release"

if [[ "$NO_BUILD" == "true" ]]; then
    echo -e "${YELLOW}⏭️  Skipping build (--no-build specified)${NC}"

    # Check if binary exists
    if [[ ! -f "$FRONTEND_BIN" ]]; then
        echo -e "${RED}❌ ZipLock binary not found: $FRONTEND_BIN${NC}"
        echo -e "${RED}   Run without --no-build to build first${NC}"
        exit 1
    fi

    # Check if shared library exists
    if [[ ! -f "$SHARED_LIB_DIR/libziplock_shared.so" ]] && [[ ! -f "$SHARED_LIB_DIR/libziplock_shared.dylib" ]]; then
        echo -e "${RED}❌ Shared library not found in: $SHARED_LIB_DIR${NC}"
        echo -e "${RED}   Run without --no-build to build first${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Using existing binaries${NC}"
else
    echo -e "${YELLOW}⚙️  Building ZipLock unified application...${NC}"
    cd "$PROJECT_ROOT"

    # Build shared library first
    echo -e "${BLUE}   Building shared library...${NC}"
    if cargo build --release -p ziplock-shared --features c-api; then
        echo -e "${GREEN}✅ Shared library built successfully${NC}"
    else
        echo -e "${RED}❌ Failed to build shared library${NC}"
        exit 1
    fi

    # Build the unified frontend application
    echo -e "${BLUE}   Building unified application...${NC}"
    if [[ -f "apps/linux/Cargo.toml" ]]; then
        if cargo build --release --bin ziplock --manifest-path apps/linux/Cargo.toml; then
            echo -e "${GREEN}✅ ZipLock application built successfully${NC}"
        else
            echo -e "${RED}❌ Failed to build ZipLock application${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ apps/linux/Cargo.toml not found!${NC}"
        echo -e "${RED}   Current directory: $(pwd)${NC}"
        exit 1
    fi
fi

# Set up environment for FFI
export LD_LIBRARY_PATH="$SHARED_LIB_DIR:${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="$SHARED_LIB_DIR:${DYLD_LIBRARY_PATH:-}"

if [[ "$DEBUG" == "true" ]]; then
    echo -e "${YELLOW}🐛 Enabling debug logging...${NC}"
    export RUST_LOG="debug"
    export ZIPLOCK_LOG_LEVEL="debug"
else
    export RUST_LOG="info"
    export ZIPLOCK_LOG_LEVEL="info"
fi

echo -e "${GREEN}🚀 Starting ZipLock (Unified FFI Architecture)...${NC}"
echo -e "${BLUE}   Binary: $FRONTEND_BIN${NC}"
echo -e "${BLUE}   Library path: $SHARED_LIB_DIR${NC}"
echo -e "${BLUE}   Log level: ${RUST_LOG}${NC}"

# Start the unified application
cd "$PROJECT_ROOT"
if [[ "$DEBUG" == "true" ]]; then
    echo -e "${YELLOW}   Running in debug mode with verbose output${NC}"
    exec "$FRONTEND_BIN" --verbose
else
    exec "$FRONTEND_BIN"
fi
