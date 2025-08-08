#!/usr/bin/env bash

# ZipLock Launch Script
# This script launches both the backend service and frontend GUI for testing

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory and project root (one level up from scripts)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}🔐 ZipLock Development Launcher${NC}"
echo -e "${BLUE}=================================${NC}"

# Function to cleanup background processes on exit
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down ZipLock...${NC}"
    if [[ -n $BACKEND_PID ]]; then
        echo -e "${YELLOW}   Stopping backend (PID: $BACKEND_PID)${NC}"
        kill $BACKEND_PID 2>/dev/null || true
        wait $BACKEND_PID 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ Cleanup complete${NC}"
    exit 0
}

# Set up signal handlers for graceful shutdown
trap cleanup SIGINT SIGTERM EXIT

# Build binaries
BACKEND_BIN="$PROJECT_ROOT/target/release/ziplock-backend"
FRONTEND_BIN="$PROJECT_ROOT/target/release/ziplock"

echo -e "${YELLOW}⚙️  Building backend...${NC}"
cd "$PROJECT_ROOT"
cargo build --release --bin ziplock-backend
echo -e "${GREEN}✅ Backend built successfully${NC}"

echo -e "${YELLOW}⚙️  Building frontend...${NC}"
cd "$PROJECT_ROOT"
echo -e "${BLUE}   Current directory: $(pwd)${NC}"
echo -e "${BLUE}   Looking for: frontend/linux/Cargo.toml${NC}"
if [[ -f "frontend/linux/Cargo.toml" ]]; then
    echo -e "${BLUE}   Cargo.toml found, building...${NC}"
    cargo build --release --bin ziplock --manifest-path frontend/linux/Cargo.toml
    echo -e "${GREEN}✅ Frontend built successfully${NC}"
else
    echo -e "${RED}❌ frontend/linux/Cargo.toml not found!${NC}"
    echo -e "${RED}   Contents of current directory:${NC}"
    ls -la
    exit 1
fi

# Start backend in background with debug logging
echo -e "${GREEN}🚀 Starting ZipLock backend in debug mode...${NC}"
cd "$PROJECT_ROOT"
$BACKEND_BIN --foreground --debug &
BACKEND_PID=$!

# Give backend time to initialize
echo -e "${YELLOW}⏳ Waiting for backend to initialize...${NC}"
sleep 2

# Check if backend is still running
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Backend failed to start!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Backend started successfully (PID: $BACKEND_PID)${NC}"
echo -e "${GREEN}🖥️  Starting ZipLock frontend...${NC}"

# Start frontend in foreground
cd "$PROJECT_ROOT"
$FRONTEND_BIN

# If we reach here, frontend has exited
echo -e "${YELLOW}👋 Frontend closed${NC}"
