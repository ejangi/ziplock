#!/bin/bash
#
# Simplified Integration Test Runner for ZipLock
# This script runs integration tests with an existing backend service
# It assumes the backend is already running and ready
#

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
SOCKET_PATH="/tmp/ziplock.sock"

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

# Setup test environment
setup_test_environment() {
    print_status "Setting up test environment"
    mkdir -p "$TEST_OUTPUT_DIR"
    cd "$PROJECT_ROOT"
}

# Check if backend is running and ready
check_backend_ready() {
    print_status "Checking if backend is ready..."

    # Check process
    if ! pgrep -f "ziplock-backend" >/dev/null 2>&1; then
        print_error "No ziplock-backend process found"
        return 1
    fi

    # Check socket
    if [[ ! -S "$SOCKET_PATH" ]]; then
        print_error "Backend socket not found at $SOCKET_PATH"
        return 1
    fi

    # Test connectivity
    if command -v nc >/dev/null 2>&1; then
        local ping_request='{"request_id":"test","session_id":null,"request":{"Ping":{"client_info":"test-runner"}}}'
        local response
        response=$(echo "$ping_request" | timeout 3 nc -U "$SOCKET_PATH" 2>/dev/null || echo "")

        if [[ -n "$response" ]] && echo "$response" | grep -q '"request_id":"test"'; then
            print_success "Backend is running and responsive"
            return 0
        else
            print_error "Backend is not responding correctly"
            return 1
        fi
    else
        print_warning "Cannot test backend connectivity (nc not available), assuming it's ready"
        return 0
    fi
}

# Run unit tests
run_unit_tests() {
    print_status "Running unit tests..."

    local log_file="$TEST_OUTPUT_DIR/unit-tests.log"

    if cargo test --package ziplock-backend --lib 2>&1 | tee "$log_file"; then
        print_success "Unit tests completed successfully"
        return 0
    else
        print_error "Unit tests failed - check $log_file for details"
        return 1
    fi
}

# Run integration tests
run_integration_tests() {
    print_status "Running integration tests..."

    local log_file="$TEST_OUTPUT_DIR/integration-tests.log"

    # First verify our test files exist
    if [[ ! -f "tests/integration/simple_persistence_test.rs" ]]; then
        print_error "Integration test file not found: tests/integration/simple_persistence_test.rs"
        return 1
    fi

    # Run the simple persistence tests
    print_status "Running credential persistence tests..."
    if cargo test --package ziplock-backend --test integration -- --test-threads=1 2>&1 | tee "$log_file"; then
        print_success "Integration tests completed successfully"
        return 0
    else
        print_error "Integration tests failed - check $log_file for details"
        return 1
    fi
}

# Run backend storage tests (these test the actual persistence logic)
run_storage_tests() {
    print_status "Running storage layer tests..."

    local log_file="$TEST_OUTPUT_DIR/storage-tests.log"

    if cargo test --package ziplock-backend storage:: -- --test-threads=1 2>&1 | tee "$log_file"; then
        print_success "Storage tests completed successfully"
        return 0
    else
        print_error "Storage tests failed - check $log_file for details"
        return 1
    fi
}

# Generate simple test report
generate_test_report() {
    print_status "Generating test report..."

    local report_file="$TEST_OUTPUT_DIR/test-report.md"

    cat > "$report_file" << EOF
# ZipLock Integration Test Report (Existing Backend)

**Generated:** $(date)
**Backend Socket:** $SOCKET_PATH
**Backend PID:** $(pgrep -f "ziplock-backend" | head -1)

## Test Results

EOF

    # Check unit tests
    if [[ -f "$TEST_OUTPUT_DIR/unit-tests.log" ]] && grep -q "test result: ok" "$TEST_OUTPUT_DIR/unit-tests.log"; then
        echo "- ✅ Unit Tests: PASSED" >> "$report_file"
    else
        echo "- ❌ Unit Tests: FAILED" >> "$report_file"
    fi

    # Check storage tests
    if [[ -f "$TEST_OUTPUT_DIR/storage-tests.log" ]] && grep -q "test result: ok" "$TEST_OUTPUT_DIR/storage-tests.log"; then
        echo "- ✅ Storage Tests: PASSED" >> "$report_file"
    else
        echo "- ❌ Storage Tests: FAILED" >> "$report_file"
    fi

    # Check integration tests
    if [[ -f "$TEST_OUTPUT_DIR/integration-tests.log" ]] && grep -q "test result: ok" "$TEST_OUTPUT_DIR/integration-tests.log"; then
        echo "- ✅ Integration Tests: PASSED" >> "$report_file"
    else
        echo "- ❌ Integration Tests: FAILED" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

## Test Focus: Credential Persistence

This test run specifically validates:
- Credential creation and storage
- Credential updates and persistence
- Archive save/load cycles
- Data integrity across operations

## Logs
- Unit Tests: \`$TEST_OUTPUT_DIR/unit-tests.log\`
- Storage Tests: \`$TEST_OUTPUT_DIR/storage-tests.log\`
- Integration Tests: \`$TEST_OUTPUT_DIR/integration-tests.log\`

EOF

    print_success "Test report generated: $report_file"
}

# Show help
show_help() {
    cat << EOF
ZipLock Integration Test Runner (Existing Backend)

This script runs integration tests assuming the backend is already running.

Usage: $0 [OPTIONS]

Options:
    -h, --help              Show this help message
    -u, --unit-only         Run only unit tests
    -s, --storage-only      Run only storage tests
    -i, --integration-only  Run only integration tests
    -q, --quick             Run a quick subset of tests
    -v, --verbose           Enable verbose output

Prerequisites:
    - ZipLock backend must be running
    - Backend socket must be available at /tmp/ziplock.sock
    - Backend must be responsive to ping requests

Examples:
    $0                      # Run all tests
    $0 --unit-only          # Run only unit tests
    $0 --quick              # Run quick test subset
    $0 --verbose            # Run with verbose output

EOF
}

# Main execution
main() {
    local unit_only=false
    local storage_only=false
    local integration_only=false
    local quick_mode=false
    local verbose=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -u|--unit-only)
                unit_only=true
                shift
                ;;
            -s|--storage-only)
                storage_only=true
                shift
                ;;
            -i|--integration-only)
                integration_only=true
                shift
                ;;
            -q|--quick)
                quick_mode=true
                shift
                ;;
            -v|--verbose)
                verbose=true
                set -x
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    print_status "Starting ZipLock Integration Tests (Existing Backend Mode)"
    print_status "Project Root: $PROJECT_ROOT"

    # Setup
    setup_test_environment

    # Check backend
    if ! check_backend_ready; then
        print_error "Backend is not ready. Please ensure:"
        print_error "  1. Backend is running: cargo run --release --bin ziplock-backend"
        print_error "  2. Socket exists at: $SOCKET_PATH"
        print_error "  3. Backend is responsive to requests"
        exit 1
    fi

    # Run tests based on options
    local test_failures=0

    if [[ "$unit_only" == "true" ]]; then
        print_status "Running unit tests only..."
        if ! run_unit_tests; then
            ((test_failures++))
        fi
    elif [[ "$storage_only" == "true" ]]; then
        print_status "Running storage tests only..."
        if ! run_storage_tests; then
            ((test_failures++))
        fi
    elif [[ "$integration_only" == "true" ]]; then
        print_status "Running integration tests only..."
        if ! run_integration_tests; then
            ((test_failures++))
        fi
    elif [[ "$quick_mode" == "true" ]]; then
        print_status "Running quick test suite..."
        if ! run_storage_tests; then
            ((test_failures++))
        fi
    else
        print_status "Running full test suite..."

        if ! run_unit_tests; then
            ((test_failures++))
        fi

        if ! run_storage_tests; then
            ((test_failures++))
        fi

        if ! run_integration_tests; then
            ((test_failures++))
        fi
    fi

    # Generate report
    generate_test_report

    # Summary
    if [[ $test_failures -eq 0 ]]; then
        print_success "🎉 All tests passed! Credential persistence is working correctly."
        print_status "Test results: $TEST_OUTPUT_DIR"
        exit 0
    else
        print_error "❌ $test_failures test suite(s) failed"
        print_status "Check logs in $TEST_OUTPUT_DIR for details"
        print_status "Backend PID: $(pgrep -f "ziplock-backend" | head -1 || echo "not found")"
        exit 1
    fi
}

# Execute main function
main "$@"
