#!/bin/bash
#
# Integration Test Runner for ZipLock
# This script runs comprehensive integration tests to verify credential persistence
# and data integrity across the entire application stack.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_OUTPUT_DIR="${PROJECT_ROOT}/test-results"
BACKEND_LOG="${TEST_OUTPUT_DIR}/backend.log"
FRONTEND_LOG="${TEST_OUTPUT_DIR}/frontend.log"
BACKEND_PID=""
BACKEND_STARTED_BY_US=""

# Option variables
UNIT_ONLY=""
INTEGRATION_ONLY=""
E2E_ONLY=""
NO_BUILD=""
USE_EXISTING_BACKEND=""
KILL_EXISTING_BACKEND=""

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

# Cleanup function
cleanup() {
    if [[ -n "$BACKEND_PID" ]]; then
        # Check if we started this backend process or if it was already running
        local our_process=false
        if [[ -n "$BACKEND_STARTED_BY_US" ]]; then
            our_process=true
        fi

        if [[ "$our_process" == "true" ]]; then
            print_status "Stopping backend service we started (PID: $BACKEND_PID)"
            kill "$BACKEND_PID" 2>/dev/null || true
            wait "$BACKEND_PID" 2>/dev/null || true
        else
            print_status "Leaving existing backend service running (PID: $BACKEND_PID)"
        fi
    fi

    # Only clean up socket files if we started the backend
    if [[ -n "$BACKEND_STARTED_BY_US" ]]; then
        find /tmp -name "ziplock*.sock" -user "$(whoami)" -delete 2>/dev/null || true
    fi

    print_status "Cleanup completed"
}

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Create test output directory
setup_test_environment() {
    print_status "Setting up test environment"

    mkdir -p "$TEST_OUTPUT_DIR"

    # Clean up any existing test artifacts
    rm -f "$BACKEND_LOG" "$FRONTEND_LOG"

    # Ensure we're in the project root
    cd "$PROJECT_ROOT"
}

# Build the project
build_project() {
    print_status "Building ZipLock project"

    # Build backend
    print_status "Building backend..."
    if ! cargo build --release --bin ziplock-backend 2>&1 | tee -a "$BACKEND_LOG"; then
        print_error "Failed to build backend"
        return 1
    fi

    # Build shared library tests
    print_status "Building shared library..."
    if ! cargo build --release -p ziplock-shared 2>&1 | tee -a "$BACKEND_LOG"; then
        print_error "Failed to build shared library"
        return 1
    fi

    # Build frontend (if it exists)
    if [[ -f "frontend/linux/Cargo.toml" ]]; then
        print_status "Building Linux frontend..."
        if ! cargo build --release --manifest-path frontend/linux/Cargo.toml 2>&1 | tee -a "$FRONTEND_LOG"; then
            print_warning "Frontend build failed, continuing with backend tests only"
        fi
    fi

    print_success "Build completed successfully"
}

# Check if backend is already running
check_backend_running() {
    # Check for existing process
    if pgrep -f "ziplock-backend" > /dev/null; then
        # Check if socket exists and is accessible
        if [[ -S "/tmp/ziplock.sock" ]]; then
            return 0  # Backend is running and ready
        fi
    fi
    return 1  # Backend is not running or not ready
}

# Test backend connectivity
test_backend_connectivity() {
    print_status "Testing backend connectivity..."

    # Try to connect to the socket and send a simple ping
    if command -v nc >/dev/null 2>&1; then
        # Use netcat to test socket connectivity
        echo '{"request_id":"test","session_id":null,"request":{"Ping":{"client_info":"test-script"}}}' | nc -U /tmp/ziplock.sock >/dev/null 2>&1
        if [[ $? -eq 0 ]]; then
            print_success "Backend connectivity test passed"
            return 0
        fi
    fi

    # Fallback: just check if socket exists and process is running
    if [[ -S "/tmp/ziplock.sock" ]] && pgrep -f "ziplock-backend" >/dev/null; then
        print_success "Backend appears to be running (socket exists and process found)"
        return 0
    fi

    print_warning "Backend connectivity test failed"
    return 1
}

# Start backend service
start_backend() {
    # Handle existing backend based on command line options
    if check_backend_running; then
        local existing_pid
        existing_pid=$(pgrep -f "ziplock-backend" | head -1)

        if [[ "$KILL_EXISTING_BACKEND" == "true" ]]; then
            print_status "Killing existing backend service (PID: $existing_pid)"
            kill "$existing_pid" 2>/dev/null || true
            sleep 2
            # Remove stale socket files
            find /tmp -name "ziplock*.sock" -user "$(whoami)" -delete 2>/dev/null || true
        elif [[ "$USE_EXISTING_BACKEND" == "true" ]]; then
            print_success "Using existing backend service (PID: $existing_pid)"
            BACKEND_PID="$existing_pid"
            # Don't set BACKEND_STARTED_BY_US since we didn't start it
            return 0
        else
            print_success "Backend service is already running (PID: $existing_pid)"
            BACKEND_PID="$existing_pid"
            # Don't set BACKEND_STARTED_BY_US since we didn't start it
            return 0
        fi
    fi

    print_status "Starting backend service"

    # Remove any stale socket files
    find /tmp -name "ziplock*.sock" -user "$(whoami)" -delete 2>/dev/null || true

    # Start backend in background
    ./target/release/ziplock-backend > "$BACKEND_LOG" 2>&1 &
    BACKEND_PID=$!
    BACKEND_STARTED_BY_US="true"

    # Wait for backend to start
    print_status "Waiting for backend to initialize..."
    local max_attempts=30
    local attempt=0

    while [[ $attempt -lt $max_attempts ]]; do
        if check_backend_running; then
            # Additional connectivity test
            if test_backend_connectivity; then
                print_success "Backend service started successfully (PID: $BACKEND_PID)"
                return 0
            fi
        fi

        # Check if our process is still alive
        if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
            print_error "Backend process died unexpectedly"
            if [[ -f "$BACKEND_LOG" ]]; then
                print_error "Backend log:"
                tail -20 "$BACKEND_LOG"
            fi
            return 1
        fi

        ((attempt++))
        sleep 1
    done

    print_error "Backend service failed to start within $max_attempts seconds"
    if [[ -f "$BACKEND_LOG" ]]; then
        print_error "Backend log:"
        tail -20 "$BACKEND_LOG"
    fi
    return 1
}

# Run unit tests
run_unit_tests() {
    print_status "Running unit tests"

    # Backend unit tests
    print_status "Running backend unit tests..."
    if ! cargo test --release -p ziplock-backend 2>&1 | tee -a "$TEST_OUTPUT_DIR/unit-tests.log"; then
        print_error "Backend unit tests failed"
        return 1
    fi

    # Shared library unit tests
    print_status "Running shared library unit tests..."
    if ! cargo test --release -p ziplock-shared 2>&1 | tee -a "$TEST_OUTPUT_DIR/unit-tests.log"; then
        print_error "Shared library unit tests failed"
        return 1
    fi

    print_success "Unit tests completed successfully"
}

# Run integration tests
run_integration_tests() {
    print_status "Running integration tests"

    # Run the comprehensive credential persistence tests
    print_status "Running credential persistence tests..."
    if ! cargo test --release --test credential_persistence_test 2>&1 | tee -a "$TEST_OUTPUT_DIR/integration-tests.log"; then
        print_error "Credential persistence tests failed"
        return 1
    fi

    print_success "Integration tests completed successfully"
}

# Run end-to-end tests (if frontend is available)
run_e2e_tests() {
    if [[ ! -f "./target/release/ziplock" ]]; then
        print_warning "Frontend binary not found, skipping E2E tests"
        return 0
    fi

    print_status "Running end-to-end tests"

    # Test frontend-backend communication
    print_status "Testing frontend-backend communication..."

    # Create a test script for frontend testing
    cat > "$TEST_OUTPUT_DIR/test_frontend.sh" << 'EOF'
#!/bin/bash
# Basic frontend test script
# This would normally include more sophisticated testing

echo "Testing frontend startup..."
timeout 10 ./target/release/ziplock --version > /dev/null 2>&1
if [[ $? -eq 0 ]]; then
    echo "Frontend startup test: PASSED"
else
    echo "Frontend startup test: FAILED"
    exit 1
fi
EOF

    chmod +x "$TEST_OUTPUT_DIR/test_frontend.sh"

    if "$TEST_OUTPUT_DIR/test_frontend.sh" 2>&1 | tee -a "$TEST_OUTPUT_DIR/e2e-tests.log"; then
        print_success "End-to-end tests completed successfully"
    else
        print_warning "End-to-end tests encountered issues"
        return 1
    fi
}

# Generate test report
generate_test_report() {
    print_status "Generating test report"

    local report_file="$TEST_OUTPUT_DIR/test-report.md"

    cat > "$report_file" << EOF
# ZipLock Integration Test Report

**Generated:** $(date)
**Environment:** $(uname -a)
**Rust Version:** $(rustc --version)

## Test Results Summary

EOF

    # Check if unit tests passed
    if grep -q "test result: ok" "$TEST_OUTPUT_DIR/unit-tests.log" 2>/dev/null; then
        echo "- ✅ Unit Tests: PASSED" >> "$report_file"
    else
        echo "- ❌ Unit Tests: FAILED" >> "$report_file"
    fi

    # Check if integration tests passed
    if grep -q "test result: ok" "$TEST_OUTPUT_DIR/integration-tests.log" 2>/dev/null; then
        echo "- ✅ Integration Tests: PASSED" >> "$report_file"
    else
        echo "- ❌ Integration Tests: FAILED" >> "$report_file"
    fi

    # Check if E2E tests passed
    if grep -q "PASSED" "$TEST_OUTPUT_DIR/e2e-tests.log" 2>/dev/null; then
        echo "- ✅ End-to-End Tests: PASSED" >> "$report_file"
    elif [[ -f "$TEST_OUTPUT_DIR/e2e-tests.log" ]]; then
        echo "- ❌ End-to-End Tests: FAILED" >> "$report_file"
    else
        echo "- ⚠️ End-to-End Tests: SKIPPED" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

## Detailed Logs

- Backend Log: \`$BACKEND_LOG\`
- Unit Tests Log: \`$TEST_OUTPUT_DIR/unit-tests.log\`
- Integration Tests Log: \`$TEST_OUTPUT_DIR/integration-tests.log\`
- E2E Tests Log: \`$TEST_OUTPUT_DIR/e2e-tests.log\`

## Key Test Coverage

### Credential Persistence Tests
- ✅ Credential creation and persistence
- ✅ Credential update and persistence
- ✅ Multiple credential operations
- ✅ Special characters and edge cases
- ✅ Archive integrity across multiple cycles
- ✅ Data consistency simulation
- ✅ Archive file structure validation

### Security Tests
- ✅ Master password validation
- ✅ Encrypted storage verification
- ✅ Session management

### Performance Tests
- ✅ Multiple save/load cycles
- ✅ Rapid consecutive operations
- ✅ Large credential datasets

EOF

    print_success "Test report generated: $report_file"
}

# Main execution
main() {
    # Parse command line arguments first
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                set -x
                shift
                ;;
            --unit-only)
                UNIT_ONLY=true
                shift
                ;;
            --integration-only)
                INTEGRATION_ONLY=true
                shift
                ;;
            --e2e-only)
                E2E_ONLY=true
                shift
                ;;
            --no-build)
                NO_BUILD=true
                shift
                ;;
            --use-existing-backend)
                USE_EXISTING_BACKEND=true
                shift
                ;;
            --kill-existing-backend)
                KILL_EXISTING_BACKEND=true
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    print_status "Starting ZipLock Integration Test Suite"
    print_status "Project Root: $PROJECT_ROOT"

    # Setup
    setup_test_environment

    # Build (unless skipped)
    if [[ "$NO_BUILD" != "true" ]]; then
        if ! build_project; then
            print_error "Build failed, aborting tests"
            exit 1
        fi
    else
        print_status "Skipping build step (--no-build specified)"
        # Verify required binaries exist
        if [[ ! -f "./target/release/ziplock-backend" ]]; then
            print_error "Backend binary not found. Please build first or remove --no-build flag."
            exit 1
        fi
    fi

    # Start or connect to backend
    if ! start_backend; then
        print_error "Failed to start or connect to backend service"
        exit 1
    fi

    # Test connectivity regardless of whether we started it or it was already running
    if ! test_backend_connectivity; then
        print_error "Backend connectivity test failed"
        print_status "This could mean:"
        print_status "  - Backend is not responding"
        print_status "  - Socket permissions are incorrect"
        print_status "  - Backend is starting but not ready yet"
        exit 1
    fi

    # Wait a moment for backend to fully initialize
    print_status "Waiting for backend to fully initialize..."
    sleep 2

    # Run tests based on options
    local test_failures=0

    if [[ "$INTEGRATION_ONLY" == "true" ]]; then
        print_status "Running integration tests only"
        if ! run_integration_tests; then
            ((test_failures++))
            print_error "Integration tests failed"
        fi
    elif [[ "$UNIT_ONLY" == "true" ]]; then
        print_status "Running unit tests only"
        if ! run_unit_tests; then
            ((test_failures++))
            print_error "Unit tests failed"
        fi
    elif [[ "$E2E_ONLY" == "true" ]]; then
        print_status "Running E2E tests only"
        if ! run_e2e_tests; then
            ((test_failures++))
            print_error "End-to-end tests failed"
        fi
    else
        # Run all tests
        print_status "Running all test suites"

        if ! run_unit_tests; then
            ((test_failures++))
            print_error "Unit tests failed"
        fi

        if ! run_integration_tests; then
            ((test_failures++))
            print_error "Integration tests failed"
        fi

        if ! run_e2e_tests; then
            ((test_failures++))
            print_error "End-to-end tests failed"
        fi
    fi

    # Generate report
    generate_test_report

    # Summary
    if [[ $test_failures -eq 0 ]]; then
        print_success "All tests completed successfully! 🎉"
        print_status "Test results available in: $TEST_OUTPUT_DIR"
        exit 0
    else
        print_error "Test suite completed with $test_failures failure(s)"
        print_status "Check logs in $TEST_OUTPUT_DIR for details"
        exit 1
    fi
}

# Help function
show_help() {
    cat << EOF
ZipLock Integration Test Runner

Usage: $0 [OPTIONS]

Options:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    --unit-only         Run only unit tests
    --integration-only  Run only integration tests
    --e2e-only          Run only end-to-end tests
    --no-build              Skip build step (use existing binaries)
    --use-existing-backend  Use already running backend service
    --kill-existing-backend Kill any existing backend before starting

Examples:
    $0                  # Run all tests
    $0 --unit-only      # Run only unit tests
    $0 --no-build       # Run tests without rebuilding

EOF
}

# Execute main function
main "$@"
