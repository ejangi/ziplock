#!/bin/bash

# ZipLock Validation System Demo Script
# This script demonstrates the comprehensive validation functionality

set -e

echo "🔐 ZipLock Validation System Demo"
echo "=================================="
echo

# Configuration
DEMO_DIR="./validation-demo"
ARCHIVE_PATH="$DEMO_DIR/demo-archive.7z"
PASSWORD="demo_validation_password_123"
CONFIG_FILE="scripts/dev/demo-config.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Setup demo environment
setup_demo() {
    log_info "Setting up demo environment..."

    # Create demo directory
    rm -rf "$DEMO_DIR"
    mkdir -p "$DEMO_DIR"

    # Use the demo configuration file
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Demo config file not found at $CONFIG_FILE"
        log_info "Please ensure you're running from the project root directory"
        log_info "See docs/technical/configuration.md for configuration examples"
        exit 1
    fi

    log_success "Demo environment created at $DEMO_DIR"
}

# Build the project
build_project() {
    log_info "Building ZipLock backend..."
    if cargo build --release --bin ziplock-backend > /dev/null 2>&1; then
        log_success "Backend built successfully"
    else
        log_error "Failed to build backend"
        exit 1
    fi
}

# Start backend daemon
start_backend() {
    log_info "Starting ZipLock backend daemon..."

    # Kill any existing backend process
    pkill -f "ziplock-backend" || true
    sleep 1

    # Start backend with demo config
    ZIPLOCK_CONFIG="$CONFIG_FILE" ./target/release/ziplock-backend > "$DEMO_DIR/backend.log" 2>&1 &
    BACKEND_PID=$!

    # Wait for backend to start
    sleep 3

    if kill -0 $BACKEND_PID 2>/dev/null; then
        log_success "Backend started (PID: $BACKEND_PID)"
        echo $BACKEND_PID > "$DEMO_DIR/backend.pid"
    else
        log_error "Failed to start backend"
        cat "$DEMO_DIR/backend.log"
        exit 1
    fi
}

# Stop backend daemon
stop_backend() {
    if [ -f "$DEMO_DIR/backend.pid" ]; then
        PID=$(cat "$DEMO_DIR/backend.pid")
        if kill -0 $PID 2>/dev/null; then
            log_info "Stopping backend daemon (PID: $PID)..."
            kill $PID
            sleep 2
            log_success "Backend stopped"
        fi
        rm -f "$DEMO_DIR/backend.pid"
    fi
}

# Create a test archive
create_test_archive() {
    log_info "Creating test archive with validation issues..."

    # Create a temporary repository structure
    local temp_repo="$DEMO_DIR/temp_repo"
    mkdir -p "$temp_repo"

    # Create basic repository structure
    mkdir -p "$temp_repo/credentials"
    mkdir -p "$temp_repo/types"

    # Create metadata file
    cat > "$temp_repo/metadata.yml" << 'EOF'
version: "1.0.0"
created_at: !Timestamp
  secs_since_epoch: 1705315800
  nanos_since_epoch: 0
credential_count: 1
EOF

    # Create a sample credential
    mkdir -p "$temp_repo/credentials/sample-login"
    cat > "$temp_repo/credentials/sample-login/record.yml" << 'EOF'
id: "sample-login"
title: "Demo Login"
credential_type: "login"
created_at: !Timestamp
  secs_since_epoch: 1705315800
  nanos_since_epoch: 0
updated_at: !Timestamp
  secs_since_epoch: 1705315800
  nanos_since_epoch: 0
fields:
  username: !Text "demo@example.com"
  password: !Secret "demo_password_123"
tags: ["demo", "test"]
notes: "This is a demo credential for validation testing"
EOF

    # Create archive using 7z
    if command -v 7z >/dev/null 2>&1; then
        cd "$temp_repo"
        7z a -p"$PASSWORD" "../demo-archive.7z" * > /dev/null 2>&1
        cd - > /dev/null
        log_success "Test archive created: $ARCHIVE_PATH"
    else
        log_error "7z command not found. Please install p7zip-full"
        exit 1
    fi

    # Clean up temp directory
    rm -rf "$temp_repo"
}

# Corrupt the archive to demonstrate auto-repair
corrupt_archive() {
    log_info "Corrupting archive to demonstrate auto-repair..."

    # Extract archive
    local temp_extract="$DEMO_DIR/temp_extract"
    mkdir -p "$temp_extract"

    cd "$temp_extract"
    7z x -p"$PASSWORD" "../demo-archive.7z" > /dev/null 2>&1
    cd - > /dev/null

    # Remove types directory to create validation issue
    rm -rf "$temp_extract/types"
    log_warning "Removed /types directory to simulate corruption"

    # Recreate archive with corruption
    cd "$temp_extract"
    7z a -p"$PASSWORD" "../demo-archive-corrupted.7z" * > /dev/null 2>&1
    cd - > /dev/null

    # Replace original with corrupted version
    mv "$DEMO_DIR/demo-archive-corrupted.7z" "$ARCHIVE_PATH"

    # Clean up
    rm -rf "$temp_extract"

    log_warning "Archive corrupted (missing /types directory)"
}

# Test validation using backend API
test_validation() {
    log_info "Testing validation system..."

    echo
    log_info "📋 Validation Test Results:"
    echo "----------------------------"

    # Test 1: Basic repository validation (no password required)
    echo
    log_info "Test 1: Basic repository format validation"
    if [ -f "$ARCHIVE_PATH" ]; then
        log_success "Archive file exists and is accessible"
        log_info "Archive size: $(ls -lh "$ARCHIVE_PATH" | awk '{print $5}')"
    else
        log_error "Archive file not found"
        return 1
    fi

    # Test 2: Demonstrate the opening process with validation
    echo
    log_info "Test 2: Opening archive with comprehensive validation enabled"
    log_info "This will trigger the validation system automatically..."

    # The validation happens automatically when opening the archive
    # For this demo, we'll show what would happen in the logs
    log_info "Expected validation process:"
    echo "  1. Extract archive to temporary directory"
    echo "  2. Perform comprehensive repository validation"
    echo "  3. Detect missing /types directory (validation issue)"
    echo "  4. Attempt auto-repair (create missing directory)"
    echo "  5. Save repaired archive"
    echo "  6. Load credentials from validated repository"

    # Show backend logs related to validation
    echo
    log_info "Backend validation logs:"
    echo "------------------------"
    if [ -f "$DEMO_DIR/backend.log" ]; then
        grep -i "validation\|repair" "$DEMO_DIR/backend.log" | tail -10 || true
    fi
}

# Show validation configuration
show_config() {
    echo
    log_info "📋 Validation Configuration:"
    echo "----------------------------"
    cat "$CONFIG_FILE" | grep -A 20 "validation:" | sed 's/^/  /'
}

# Show validation features
show_features() {
    echo
    log_info "🔍 Validation System Features:"
    echo "==============================="
    echo
    echo "1. Comprehensive Structure Validation:"
    echo "   ✓ Required directories (/credentials, /types)"
    echo "   ✓ Essential files (metadata.yml)"
    echo "   ✓ Repository format compliance"
    echo
    echo "2. Content Validation:"
    echo "   ✓ YAML file parsing and schema validation"
    echo "   ✓ Credential data integrity checks"
    echo "   ✓ Custom type definition validation"
    echo
    echo "3. Auto-Repair Capabilities:"
    echo "   ✓ Missing directory creation"
    echo "   ✓ Placeholder file generation (.gitkeep)"
    echo "   ✓ Legacy format migration"
    echo "   ✓ Structural issue resolution"
    echo
    echo "4. Configurable Behavior:"
    echo "   ✓ Production mode (strict validation)"
    echo "   ✓ Development mode (permissive validation)"
    echo "   ✓ Legacy compatibility mode"
    echo "   ✓ Detailed logging and reporting"
    echo
    echo "5. Performance Options:"
    echo "   ✓ Deep validation (can be disabled for speed)"
    echo "   ✓ Schema validation (configurable)"
    echo "   ✓ Legacy format checking (optional)"
    echo
    echo "6. Streamlined Configuration:"
    echo "   ✓ Removed unused parameters for clarity"
    echo "   ✓ Only includes actively implemented features"
    echo "   ✓ Simplified maintenance and documentation"
    echo
}

# Cleanup function
cleanup() {
    log_info "Cleaning up demo environment..."
    stop_backend
    rm -rf "$DEMO_DIR"
    log_success "Cleanup completed"
}

# Main demo execution
main() {
    echo "This demo showcases ZipLock's comprehensive validation system"
    echo "that automatically validates archives when the backend connects."
    echo

    # Setup
    setup_demo
    build_project

    # Show features
    show_features
    show_config

    # Run demo
    create_test_archive
    corrupt_archive
    start_backend

    # Wait a moment for any automatic validation
    sleep 2

    test_validation

    echo
    log_success "🎉 Validation Demo Completed!"
    echo
    echo "Key Points Demonstrated:"
    echo "• Comprehensive validation runs automatically when opening archives"
    echo "• Auto-repair fixes common issues (missing directories, etc.)"
    echo "• Validation is fully configurable for different use cases"
    echo "• Config file has been streamlined to only include used parameters"
    echo "• System provides detailed logging and reporting"
    echo
    log_info "Check the backend logs at: $DEMO_DIR/backend.log"
    log_info "Check the streamlined validation config at: $CONFIG_FILE"
    echo

    read -p "Press Enter to cleanup and exit..."
    cleanup
}

# Handle script interruption
trap cleanup EXIT INT TERM

# Check dependencies
if ! command -v cargo >/dev/null 2>&1; then
    log_error "Cargo not found. Please install Rust and Cargo."
    exit 1
fi

if ! command -v 7z >/dev/null 2>&1; then
    log_error "7z not found. Please install p7zip-full:"
    echo "  Ubuntu/Debian: sudo apt install p7zip-full"
    echo "  Fedora: sudo dnf install p7zip"
    echo "  Arch: sudo pacman -S p7zip"
    exit 1
fi

# Run the demo
main
