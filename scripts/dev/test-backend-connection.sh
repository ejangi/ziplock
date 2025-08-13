#!/bin/bash
#
# Simple Backend Connectivity Test Script
# This script tests if the ZipLock backend is running and responsive

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOCKET_PATH="/tmp/ziplock.sock"
MAX_WAIT_TIME=10
VERBOSE=false

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Show help
show_help() {
    cat << EOF
ZipLock Backend Connectivity Test

Usage: $0 [OPTIONS]

Options:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -w, --wait TIME     Wait up to TIME seconds for backend (default: $MAX_WAIT_TIME)
    -s, --socket PATH   Use custom socket path (default: $SOCKET_PATH)

Examples:
    $0                  # Quick connectivity test
    $0 -v               # Verbose test with debug info
    $0 -w 30            # Wait up to 30 seconds for backend
    $0 -s /custom/path  # Use custom socket path

EOF
}

# Check if backend process is running
check_backend_process() {
    print_verbose "Checking for ziplock-backend process..."

    if pgrep -f "ziplock-backend" >/dev/null 2>&1; then
        local pid
        pid=$(pgrep -f "ziplock-backend" | head -1)
        print_success "Backend process found (PID: $pid)"
        return 0
    else
        print_error "No ziplock-backend process found"
        return 1
    fi
}

# Check if socket file exists and is accessible
check_socket_file() {
    print_verbose "Checking socket file: $SOCKET_PATH"

    if [[ ! -e "$SOCKET_PATH" ]]; then
        print_error "Socket file does not exist: $SOCKET_PATH"
        return 1
    fi

    if [[ ! -S "$SOCKET_PATH" ]]; then
        print_error "Socket file is not a valid socket: $SOCKET_PATH"
        return 1
    fi

    if [[ ! -r "$SOCKET_PATH" || ! -w "$SOCKET_PATH" ]]; then
        print_error "Socket file is not readable/writable: $SOCKET_PATH"
        return 1
    fi

    print_success "Socket file is valid and accessible"
    return 0
}

# Test actual connectivity to backend
test_backend_connectivity() {
    print_verbose "Testing backend connectivity via socket..."

    # Create a simple ping request
    local ping_request='{"request_id":"connectivity-test","session_id":null,"request":{"Ping":{"client_info":"connectivity-test-script"}}}'

    # Try to send ping and get response
    if command -v nc >/dev/null 2>&1; then
        print_verbose "Using netcat for connectivity test..."

        local response
        response=$(echo "$ping_request" | timeout 5 nc -U "$SOCKET_PATH" 2>/dev/null || echo "")

        if [[ -n "$response" ]]; then
            print_verbose "Received response: $response"

            # Check if response contains expected fields
            if echo "$response" | grep -q '"request_id":"connectivity-test"' && echo "$response" | grep -q '"result"'; then
                print_success "Backend is responding correctly to requests"
                return 0
            else
                print_warning "Backend responded but with unexpected format"
                print_verbose "Response: $response"
                return 1
            fi
        else
            print_error "No response received from backend"
            return 1
        fi
    else
        print_warning "netcat not available, using basic socket check only"
        # Just verify we can connect to the socket
        if timeout 3 bash -c "</dev/tcp/localhost/0 2>/dev/null || test -S '$SOCKET_PATH'"; then
            print_success "Socket is connectable (basic test)"
            return 0
        else
            print_error "Cannot connect to socket"
            return 1
        fi
    fi
}

# Wait for backend to become ready
wait_for_backend() {
    print_status "Waiting up to $MAX_WAIT_TIME seconds for backend to become ready..."

    local attempt=0
    while [[ $attempt -lt $MAX_WAIT_TIME ]]; do
        print_verbose "Attempt $((attempt + 1))/$MAX_WAIT_TIME"

        if check_backend_process && check_socket_file && test_backend_connectivity; then
            print_success "Backend is ready and responsive!"
            return 0
        fi

        ((attempt++))
        sleep 1
    done

    print_error "Backend did not become ready within $MAX_WAIT_TIME seconds"
    return 1
}

# Main connectivity test
run_connectivity_test() {
    print_status "Running ZipLock backend connectivity test..."
    print_verbose "Socket path: $SOCKET_PATH"
    print_verbose "Max wait time: $MAX_WAIT_TIME seconds"

    local test_failures=0

    # Test 1: Check if backend process is running
    print_status "Test 1: Backend process check"
    if ! check_backend_process; then
        ((test_failures++))
        print_error "❌ Backend process test failed"
    else
        print_success "✅ Backend process test passed"
    fi

    # Test 2: Check socket file
    print_status "Test 2: Socket file check"
    if ! check_socket_file; then
        ((test_failures++))
        print_error "❌ Socket file test failed"
    else
        print_success "✅ Socket file test passed"
    fi

    # Test 3: Test actual connectivity
    print_status "Test 3: Backend connectivity test"
    if ! test_backend_connectivity; then
        ((test_failures++))
        print_error "❌ Backend connectivity test failed"
    else
        print_success "✅ Backend connectivity test passed"
    fi

    # Summary
    echo
    print_status "Test Summary:"
    if [[ $test_failures -eq 0 ]]; then
        print_success "🎉 All connectivity tests passed! Backend is ready for integration tests."
        return 0
    else
        print_error "❌ $test_failures test(s) failed. Backend may not be ready."

        # Provide helpful suggestions
        echo
        print_status "Troubleshooting suggestions:"
        if ! pgrep -f "ziplock-backend" >/dev/null; then
            print_status "• Start the backend: cargo run --release --bin ziplock-backend"
        fi
        if [[ ! -S "$SOCKET_PATH" ]]; then
            print_status "• Check if backend has permission to create socket files in /tmp"
            print_status "• Verify backend configuration and socket path"
        fi
        print_status "• Check backend logs for error messages"
        print_status "• Ensure no firewall or permission issues"

        return 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -w|--wait)
            MAX_WAIT_TIME="$2"
            shift 2
            ;;
        -s|--socket)
            SOCKET_PATH="$2"
            shift 2
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate arguments
if ! [[ "$MAX_WAIT_TIME" =~ ^[0-9]+$ ]] || [[ "$MAX_WAIT_TIME" -lt 1 ]]; then
    print_error "Invalid wait time: $MAX_WAIT_TIME (must be a positive integer)"
    exit 1
fi

# Run the test
if [[ "$MAX_WAIT_TIME" -gt 1 ]]; then
    # If wait time is specified, use wait mode
    wait_for_backend
else
    # Otherwise just run immediate test
    run_connectivity_test
fi
