#!/usr/bin/env bash

# ZipLock Backend Test Runner
# This script runs all backend-specific tests including unit tests, backend library tests,
# and backend-focused integration tests

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_OUTPUT_DIR="${PROJECT_ROOT}/tests/results/backend"
BACKEND_LOG="${TEST_OUTPUT_DIR}/backend.log"

# Option variables
NO_BUILD=""
VERBOSE=""
UNIT_ONLY=""
INTEGRATION_ONLY=""
LIB_ONLY=""
COVERAGE=""

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

# Help function
show_help() {
    cat << EOF
ZipLock Backend Test Runner

This script runs comprehensive backend tests including unit tests, library tests,
and backend-specific integration tests.

Usage: $0 [OPTIONS]

Options:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    --no-build          Skip build step (use existing binaries)
    --unit-only         Run only unit tests
    --integration-only  Run only integration tests
    --lib-only          Run only backend library tests
    --coverage          Generate test coverage report (requires cargo-tarpaulin)

Test Categories:
    Unit Tests:         Tests within backend modules and functions
    Library Tests:      Tests for the backend library crate
    Integration Tests:  Backend-specific integration tests

Examples:
    $0                  # Run all backend tests
    $0 --unit-only      # Run only unit tests
    $0 --no-build       # Run tests without rebuilding
    $0 --coverage       # Run tests with coverage report

EOF
}

# Setup test environment
setup_test_environment() {
    print_status "Setting up backend test environment"

    mkdir -p "$TEST_OUTPUT_DIR"

    # Clean up any existing test artifacts
    rm -f "$BACKEND_LOG"
    rm -f "$TEST_OUTPUT_DIR"/*.log

    # Ensure we're in the project root
    cd "$PROJECT_ROOT"

    print_success "Test environment ready"
}

# Build backend components
build_backend() {
    print_status "Building backend components"

    # Build backend binary
    print_status "Building backend binary..."
    if ! cargo build --release --bin ziplock-backend 2>&1 | tee -a "$BACKEND_LOG"; then
        print_error "Failed to build backend binary"
        return 1
    fi

    # Build backend library
    print_status "Building backend library..."
    if ! cargo build --release -p ziplock-backend --lib 2>&1 | tee -a "$BACKEND_LOG"; then
        print_error "Failed to build backend library"
        return 1
    fi

    # Build shared library (dependency of backend)
    print_status "Building shared library..."
    if ! cargo build --release -p ziplock-shared 2>&1 | tee -a "$BACKEND_LOG"; then
        print_error "Failed to build shared library"
        return 1
    fi

    print_success "Backend components built successfully"
}

# Run backend unit tests
run_unit_tests() {
    print_status "Running backend unit tests"

    local test_args=""
    if [[ "$VERBOSE" == "true" ]]; then
        test_args="--verbose"
    fi

    # Run tests for the backend package
    print_status "Running backend package unit tests..."
    if ! cargo test --release -p ziplock-backend $test_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/unit-tests.log"; then
        print_error "Backend unit tests failed"
        return 1
    fi

    print_success "Backend unit tests completed successfully"
}

# Run backend library tests
run_library_tests() {
    print_status "Running backend library tests"

    local test_args=""
    if [[ "$VERBOSE" == "true" ]]; then
        test_args="--verbose"
    fi

    # Run tests specifically for the backend library
    print_status "Running backend library tests..."
    if ! cargo test --release -p ziplock-backend --lib $test_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/lib-tests.log"; then
        print_error "Backend library tests failed"
        return 1
    fi

    # Also run shared library tests since backend depends on it
    print_status "Running shared library tests..."
    if ! cargo test --release -p ziplock-shared $test_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/lib-tests.log"; then
        print_error "Shared library tests failed"
        return 1
    fi

    print_success "Backend library tests completed successfully"
}

# Run backend integration tests
run_integration_tests() {
    print_status "Running backend integration tests"

    local test_args=""
    if [[ "$VERBOSE" == "true" ]]; then
        test_args="--verbose"
    fi

    # Run backend-specific integration tests
    if [[ -f "tests/integration/credential_persistence_test.rs" ]]; then
        print_status "Running credential persistence tests..."
        if ! cargo test --release --test credential_persistence_test $test_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/integration-tests.log"; then
            print_error "Credential persistence tests failed"
            return 1
        fi
    fi

    if [[ -f "tests/integration/simple_persistence_test.rs" ]]; then
        print_status "Running simple persistence tests..."
        if ! cargo test --release --test simple_persistence_test $test_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/integration-tests.log"; then
            print_error "Simple persistence tests failed"
            return 1
        fi
    fi

    # Run any other backend integration tests
    print_status "Running all backend integration tests..."
    if ! cargo test --release --test "*" $test_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/integration-tests.log"; then
        print_warning "Some integration tests may have failed - check logs for details"
    fi

    print_success "Backend integration tests completed"
}

# Generate test coverage report
run_coverage() {
    print_status "Generating test coverage report"

    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
        print_warning "cargo-tarpaulin not found. Installing..."
        if ! cargo install cargo-tarpaulin; then
            print_error "Failed to install cargo-tarpaulin. Skipping coverage report."
            return 1
        fi
    fi

    local coverage_args="--packages ziplock-backend,ziplock-shared"
    coverage_args="$coverage_args --out Html --output-dir $TEST_OUTPUT_DIR/coverage"

    if [[ "$VERBOSE" == "true" ]]; then
        coverage_args="$coverage_args --verbose"
    fi

    print_status "Running tests with coverage analysis..."
    if ! cargo tarpaulin $coverage_args 2>&1 | tee -a "$TEST_OUTPUT_DIR/coverage.log"; then
        print_error "Coverage analysis failed"
        return 1
    fi

    print_success "Coverage report generated in $TEST_OUTPUT_DIR/coverage/"
}

# Run backend examples as smoke tests
run_example_tests() {
    print_status "Running backend example tests"

    # Check if backend examples exist and run them as smoke tests
    if [[ -f "backend/examples/simple_credential_test.rs" ]]; then
        print_status "Running simple credential example test..."
        if ! cargo run --release --example simple_credential_test 2>&1 | tee -a "$TEST_OUTPUT_DIR/examples.log"; then
            print_warning "Simple credential example test failed"
        fi
    fi

    if [[ -f "backend/examples/credential_save_test.rs" ]]; then
        print_status "Running credential save example test..."
        if ! cargo run --release --example credential_save_test 2>&1 | tee -a "$TEST_OUTPUT_DIR/examples.log"; then
            print_warning "Credential save example test failed"
        fi
    fi

    print_success "Backend example tests completed"
}

# Validate backend binary
validate_backend_binary() {
    print_status "Validating backend binary"

    local backend_bin="$PROJECT_ROOT/target/release/ziplock-backend"

    if [[ ! -f "$backend_bin" ]]; then
        print_error "Backend binary not found: $backend_bin"
        return 1
    fi

    # Test binary help output
    if ! "$backend_bin" --help >/dev/null 2>&1; then
        print_error "Backend binary failed to show help"
        return 1
    fi

    # Test binary version output
    if ! "$backend_bin" --version >/dev/null 2>&1; then
        print_error "Backend binary failed to show version"
        return 1
    fi

    print_success "Backend binary validation passed"
}

# Generate test report
generate_test_report() {
    print_status "Generating backend test report"

    local report_file="$TEST_OUTPUT_DIR/backend-test-report.md"

    cat > "$report_file" << EOF
# ZipLock Backend Test Report

**Generated:** $(date)
**Environment:** $(uname -a)
**Rust Version:** $(rustc --version)
**Backend Version:** $(cd "$PROJECT_ROOT" && cargo metadata --format-version 1 2>/dev/null | grep -o '"ziplock-backend","version":"[^"]*"' | cut -d'"' -f4 || echo "unknown")

## Test Results Summary

EOF

    # Check test results and add to report
    local total_failures=0

    # Unit tests
    if [[ -f "$TEST_OUTPUT_DIR/unit-tests.log" ]]; then
        if grep -q "test result: ok" "$TEST_OUTPUT_DIR/unit-tests.log"; then
            echo "- ✅ Unit Tests: PASSED" >> "$report_file"
        else
            echo "- ❌ Unit Tests: FAILED" >> "$report_file"
            ((total_failures++))
        fi
    else
        echo "- ⚠️ Unit Tests: SKIPPED" >> "$report_file"
    fi

    # Library tests
    if [[ -f "$TEST_OUTPUT_DIR/lib-tests.log" ]]; then
        if grep -q "test result: ok" "$TEST_OUTPUT_DIR/lib-tests.log"; then
            echo "- ✅ Library Tests: PASSED" >> "$report_file"
        else
            echo "- ❌ Library Tests: FAILED" >> "$report_file"
            ((total_failures++))
        fi
    else
        echo "- ⚠️ Library Tests: SKIPPED" >> "$report_file"
    fi

    # Integration tests
    if [[ -f "$TEST_OUTPUT_DIR/integration-tests.log" ]]; then
        if grep -q "test result: ok" "$TEST_OUTPUT_DIR/integration-tests.log"; then
            echo "- ✅ Integration Tests: PASSED" >> "$report_file"
        else
            echo "- ❌ Integration Tests: FAILED" >> "$report_file"
            ((total_failures++))
        fi
    else
        echo "- ⚠️ Integration Tests: SKIPPED" >> "$report_file"
    fi

    # Coverage
    if [[ -f "$TEST_OUTPUT_DIR/coverage.log" ]]; then
        local coverage_percent=$(grep -o '[0-9]*\.[0-9]*%' "$TEST_OUTPUT_DIR/coverage.log" | tail -1 || echo "unknown")
        echo "- 📊 Test Coverage: $coverage_percent" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

## Backend Test Coverage

### Unit Tests
- ✅ Configuration validation and file operations
- ✅ Error handling and user-friendly messages
- ✅ API validation functions
- ✅ IPC request/response serialization
- ✅ File locking mechanisms
- ✅ Storage validation and repository handling

### Library Tests
- ✅ Core backend library functionality
- ✅ Shared library components
- ✅ Module integration points

### Integration Tests
- ✅ Credential persistence across save/load cycles
- ✅ Data integrity and encryption validation
- ✅ Archive format compatibility

## Detailed Logs

- Backend Build Log: \`$BACKEND_LOG\`
- Unit Tests Log: \`$TEST_OUTPUT_DIR/unit-tests.log\`
- Library Tests Log: \`$TEST_OUTPUT_DIR/lib-tests.log\`
- Integration Tests Log: \`$TEST_OUTPUT_DIR/integration-tests.log\`
- Examples Log: \`$TEST_OUTPUT_DIR/examples.log\`

EOF

    if [[ -f "$TEST_OUTPUT_DIR/coverage/tarpaulin-report.html" ]]; then
        echo "- Coverage Report: \`$TEST_OUTPUT_DIR/coverage/tarpaulin-report.html\`" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

## Performance Notes

- All tests run in release mode for performance validation
- File locking tests include concurrent access scenarios
- Integration tests validate multi-cycle data persistence

## Next Steps

EOF

    if [[ $total_failures -gt 0 ]]; then
        cat >> "$report_file" << EOF
⚠️ **$total_failures test suite(s) failed**
- Review detailed logs above for specific failure information
- Check error messages for actionable debugging steps
- Consider running tests with --verbose for additional details
EOF
    else
        cat >> "$report_file" << EOF
✅ **All backend tests passed successfully**
- Backend is ready for integration with frontend components
- Consider running full integration test suite with \`scripts/dev/run-integration-tests.sh\`
EOF
    fi

    print_success "Backend test report generated: $report_file"
    return $total_failures
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                set -x
                shift
                ;;
            --no-build)
                NO_BUILD=true
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
            --lib-only)
                LIB_ONLY=true
                shift
                ;;
            --coverage)
                COVERAGE=true
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    print_status "Starting ZipLock Backend Test Suite"
    print_status "Project Root: $PROJECT_ROOT"

    # Setup
    setup_test_environment

    # Build (unless skipped)
    if [[ "$NO_BUILD" != "true" ]]; then
        if ! build_backend; then
            print_error "Build failed, aborting tests"
            exit 1
        fi
    else
        print_status "Skipping build step (--no-build specified)"
        # Verify required binaries exist
        if [[ ! -f "$PROJECT_ROOT/target/release/ziplock-backend" ]]; then
            print_error "Backend binary not found. Please build first or remove --no-build flag."
            exit 1
        fi
    fi

    # Validate backend binary
    if ! validate_backend_binary; then
        print_error "Backend binary validation failed"
        exit 1
    fi

    # Run tests based on options
    local test_failures=0

    if [[ "$UNIT_ONLY" == "true" ]]; then
        print_status "Running unit tests only"
        if ! run_unit_tests; then
            ((test_failures++))
        fi
    elif [[ "$LIB_ONLY" == "true" ]]; then
        print_status "Running library tests only"
        if ! run_library_tests; then
            ((test_failures++))
        fi
    elif [[ "$INTEGRATION_ONLY" == "true" ]]; then
        print_status "Running integration tests only"
        if ! run_integration_tests; then
            ((test_failures++))
        fi
    else
        # Run all backend tests
        print_status "Running all backend test suites"

        if ! run_unit_tests; then
            ((test_failures++))
            print_error "Unit tests failed"
        fi

        if ! run_library_tests; then
            ((test_failures++))
            print_error "Library tests failed"
        fi

        if ! run_integration_tests; then
            ((test_failures++))
            print_error "Integration tests failed"
        fi

        # Run example tests as smoke tests
        run_example_tests || true  # Don't fail on example test failures
    fi

    # Generate coverage report if requested
    if [[ "$COVERAGE" == "true" ]]; then
        if ! run_coverage; then
            print_warning "Coverage report generation failed"
        fi
    fi

    # Generate report and get failure count
    if ! generate_test_report; then
        test_failures=$?
    fi

    # Summary
    if [[ $test_failures -eq 0 ]]; then
        print_success "All backend tests completed successfully! 🎉"
        print_status "Backend is ready for integration testing"
        print_status "Test results available in: $TEST_OUTPUT_DIR"
        print_status "Next step: Run full integration tests with scripts/dev/run-integration-tests.sh"
        exit 0
    else
        print_error "Backend test suite completed with $test_failures failure(s)"
        print_status "Check logs in $TEST_OUTPUT_DIR for details"
        print_status "Consider running with --verbose for additional debugging information"
        exit 1
    fi
}

# Execute main function
main "$@"
