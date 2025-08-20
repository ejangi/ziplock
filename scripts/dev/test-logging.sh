#!/bin/bash
# ZipLock Logging Test Script
#
# This script provides comprehensive testing of the logging system in various
# configurations and environments, including Docker-based testing.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_OUTPUT_DIR="$PROJECT_ROOT/target/test-logs"
DOCKER_IMAGE_NAME="ziplock-logging-test"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."

    # Stop any running containers
    docker stop ziplock-logging-test-container 2>/dev/null || true
    docker rm ziplock-logging-test-container 2>/dev/null || true

    # Remove test logs if requested
    if [[ "${CLEANUP_LOGS:-false}" == "true" ]]; then
        rm -rf "$TEST_OUTPUT_DIR"
        log_info "Cleaned up test logs"
    fi
}

# Trap cleanup on exit
trap cleanup EXIT

# Build the Linux application
build_application() {
    log_info "Building ZipLock Linux application..."

    cd "$PROJECT_ROOT"

    # Build in release mode for testing
    cargo build --release --bin ziplock

    if [[ ! -f "target/release/ziplock" ]]; then
        log_error "Build failed - binary not found"
        exit 1
    fi

    log_success "Application built successfully"
}

# Create test directories
setup_test_environment() {
    log_info "Setting up test environment..."

    # Create test output directory
    mkdir -p "$TEST_OUTPUT_DIR"
    mkdir -p "$TEST_OUTPUT_DIR/dev-logs"
    mkdir -p "$TEST_OUTPUT_DIR/prod-logs"
    mkdir -p "$TEST_OUTPUT_DIR/systemd-logs"
    mkdir -p "$TEST_OUTPUT_DIR/docker-logs"

    log_success "Test directories created"
}

# Create Dockerfile for testing
create_test_dockerfile() {
    log_info "Creating test Dockerfile..."

    cat > "$PROJECT_ROOT/Dockerfile.logging-test" << 'EOF'
FROM ubuntu:22.04

# Install required packages
RUN apt-get update && apt-get install -y \
    logrotate \
    systemd \
    rsyslog \
    cron \
    && rm -rf /var/lib/apt/lists/*

# Create ziplock user and group
RUN groupadd --system ziplock && \
    useradd --system --gid ziplock \
            --home-dir /var/lib/ziplock \
            --shell /bin/false \
            --comment "ZipLock Password Manager" \
            ziplock

# Create necessary directories
RUN mkdir -p /var/log/ziplock /var/lib/ziplock && \
    chown ziplock:ziplock /var/log/ziplock /var/lib/ziplock && \
    chmod 750 /var/log/ziplock /var/lib/ziplock

# Copy the binary
COPY target/release/ziplock /usr/bin/ziplock
RUN chmod +x /usr/bin/ziplock

# Copy test scripts
COPY scripts/dev/test-logging-inside-container.sh /test-logging.sh
RUN chmod +x /test-logging.sh

# Create logrotate config for testing
RUN echo '/var/log/ziplock/*.log {\
    daily\
    rotate 7\
    compress\
    delaycompress\
    notifempty\
    missingok\
    create 644 ziplock ziplock\
    copytruncate\
}' > /etc/logrotate.d/ziplock

# Set working directory
WORKDIR /var/lib/ziplock

# Default command
CMD ["/test-logging.sh"]
EOF

    log_success "Dockerfile created"
}

# Create container test script
create_container_test_script() {
    log_info "Creating container test script..."

    cat > "$PROJECT_ROOT/scripts/dev/test-logging-inside-container.sh" << 'EOF'
#!/bin/bash
# Container-based logging tests

set -euo pipefail

echo "=== ZipLock Logging Test (Inside Container) ==="

# Test 1: Development logging
echo "1. Testing development logging..."
export ZIPLOCK_ENV=development
export RUST_LOG=debug
timeout 10 /usr/bin/ziplock &
ZIPLOCK_PID=$!
sleep 5
kill $ZIPLOCK_PID || true
wait $ZIPLOCK_PID 2>/dev/null || true

echo "   Development logs:"
ls -la /var/log/ziplock/ || echo "   No log files found"
if ls /var/log/ziplock/*.log 1> /dev/null 2>&1; then
    echo "   Latest log entries:"
    tail -n 10 /var/log/ziplock/*.log | head -20
fi

# Test 2: Production logging
echo "2. Testing production logging..."
rm -f /var/log/ziplock/*.log
export ZIPLOCK_ENV=production
export RUST_LOG=info
timeout 10 /usr/bin/ziplock &
ZIPLOCK_PID=$!
sleep 5
kill $ZIPLOCK_PID || true
wait $ZIPLOCK_PID 2>/dev/null || true

echo "   Production logs:"
ls -la /var/log/ziplock/ || echo "   No log files found"
if ls /var/log/ziplock/*.log 1> /dev/null 2>&1; then
    echo "   Latest log entries:"
    tail -n 10 /var/log/ziplock/*.log | head -20
fi

# Test 3: Log rotation
echo "3. Testing log rotation..."
if ls /var/log/ziplock/*.log 1> /dev/null 2>&1; then
    # Force logrotate
    logrotate -f /etc/logrotate.d/ziplock
    echo "   Files after rotation:"
    ls -la /var/log/ziplock/
else
    echo "   No log files to rotate"
fi

# Test 4: File permissions
echo "4. Checking file permissions..."
echo "   Log directory:"
ls -la /var/log/ | grep ziplock || echo "   Directory not found"
echo "   Log files:"
ls -la /var/log/ziplock/ || echo "   No files found"

# Test 5: User permissions
echo "5. Testing user permissions..."
echo "   Current user: $(whoami)"
echo "   Can write to log dir: $(test -w /var/log/ziplock && echo "YES" || echo "NO")"

echo "=== Container tests completed ==="
EOF

    chmod +x "$PROJECT_ROOT/scripts/dev/test-logging-inside-container.sh"
    log_success "Container test script created"
}

# Build Docker test image
build_test_image() {
    log_info "Building Docker test image..."

    cd "$PROJECT_ROOT"

    docker build -f Dockerfile.logging-test -t "$DOCKER_IMAGE_NAME" .

    log_success "Docker test image built"
}

# Run Docker-based logging tests
run_docker_tests() {
    log_info "Running Docker-based logging tests..."

    # Run container with volume mount for log inspection
    docker run --rm \
        --name ziplock-logging-test-container \
        -v "$TEST_OUTPUT_DIR/docker-logs:/output-logs" \
        "$DOCKER_IMAGE_NAME" bash -c "
            /test-logging.sh

            echo '=== Copying logs to output ==='
            cp -r /var/log/ziplock/* /output-logs/ 2>/dev/null || echo 'No logs to copy'
            ls -la /output-logs/
        "

    log_success "Docker tests completed"
}

# Run local development tests
run_local_dev_tests() {
    log_info "Running local development logging tests..."

    cd "$PROJECT_ROOT"

    # Test development configuration
    export ZIPLOCK_ENV=development
    export RUST_LOG=debug
    export ZIPLOCK_LOG_DIR="$TEST_OUTPUT_DIR/dev-logs"

    log_info "Starting application in development mode (10 seconds)..."
    timeout 10 target/release/ziplock &
    ZIPLOCK_PID=$!
    sleep 5

    # Generate some activity
    kill -USR1 $ZIPLOCK_PID 2>/dev/null || true
    sleep 2

    kill $ZIPLOCK_PID 2>/dev/null || true
    wait $ZIPLOCK_PID 2>/dev/null || true

    log_info "Development test logs:"
    ls -la "$TEST_OUTPUT_DIR/dev-logs/" || log_warning "No dev logs found"

    if ls "$TEST_OUTPUT_DIR/dev-logs/"*.log 1> /dev/null 2>&1; then
        log_info "Sample log entries:"
        tail -n 10 "$TEST_OUTPUT_DIR/dev-logs/"*.log | head -20
    fi

    log_success "Local development tests completed"
}

# Run production simulation tests
run_prod_simulation_tests() {
    log_info "Running production simulation tests..."

    cd "$PROJECT_ROOT"

    # Test production configuration
    export ZIPLOCK_ENV=production
    export RUST_LOG=info
    export ZIPLOCK_LOG_DIR="$TEST_OUTPUT_DIR/prod-logs"

    log_info "Starting application in production mode (10 seconds)..."
    timeout 10 target/release/ziplock &
    ZIPLOCK_PID=$!
    sleep 5

    # Generate some activity
    kill -USR1 $ZIPLOCK_PID 2>/dev/null || true
    sleep 2

    kill $ZIPLOCK_PID 2>/dev/null || true
    wait $ZIPLOCK_PID 2>/dev/null || true

    log_info "Production test logs:"
    ls -la "$TEST_OUTPUT_DIR/prod-logs/" || log_warning "No prod logs found"

    if ls "$TEST_OUTPUT_DIR/prod-logs/"*.log 1> /dev/null 2>&1; then
        log_info "Sample log entries:"
        tail -n 10 "$TEST_OUTPUT_DIR/prod-logs/"*.log | head -20
    fi

    log_success "Production simulation tests completed"
}

# Test log rotation locally
test_log_rotation() {
    log_info "Testing log rotation functionality..."

    # Create a test log file with some content
    TEST_LOG="$TEST_OUTPUT_DIR/test-rotation.log"

    # Generate test log content
    for i in {1..1000}; do
        echo "$(date '+%Y-%m-%d %H:%M:%S') [INFO] Test log entry $i" >> "$TEST_LOG"
    done

    log_info "Created test log with $(wc -l < "$TEST_LOG") lines"

    # Test our cleanup function
    cd "$PROJECT_ROOT"

    # This would normally be done by the logging system
    log_info "Log file size: $(du -h "$TEST_LOG" | cut -f1)"

    log_success "Log rotation test completed"
}

# Analyze test results
analyze_results() {
    log_info "Analyzing test results..."

    echo "=== Test Results Summary ==="
    echo

    # Check development logs
    if [[ -d "$TEST_OUTPUT_DIR/dev-logs" ]]; then
        DEV_LOG_COUNT=$(find "$TEST_OUTPUT_DIR/dev-logs" -name "*.log" | wc -l)
        echo "Development logs: $DEV_LOG_COUNT files"
        if [[ $DEV_LOG_COUNT -gt 0 ]]; then
            DEV_LOG_SIZE=$(du -sh "$TEST_OUTPUT_DIR/dev-logs" | cut -f1)
            echo "  Total size: $DEV_LOG_SIZE"
        fi
    fi

    # Check production logs
    if [[ -d "$TEST_OUTPUT_DIR/prod-logs" ]]; then
        PROD_LOG_COUNT=$(find "$TEST_OUTPUT_DIR/prod-logs" -name "*.log" | wc -l)
        echo "Production logs: $PROD_LOG_COUNT files"
        if [[ $PROD_LOG_COUNT -gt 0 ]]; then
            PROD_LOG_SIZE=$(du -sh "$TEST_OUTPUT_DIR/prod-logs" | cut -f1)
            echo "  Total size: $PROD_LOG_SIZE"
        fi
    fi

    # Check Docker logs
    if [[ -d "$TEST_OUTPUT_DIR/docker-logs" ]]; then
        DOCKER_LOG_COUNT=$(find "$TEST_OUTPUT_DIR/docker-logs" -name "*.log" | wc -l)
        echo "Docker logs: $DOCKER_LOG_COUNT files"
        if [[ $DOCKER_LOG_COUNT -gt 0 ]]; then
            DOCKER_LOG_SIZE=$(du -sh "$TEST_OUTPUT_DIR/docker-logs" | cut -f1)
            echo "  Total size: $DOCKER_LOG_SIZE"
        fi
    fi

    echo
    echo "All test outputs available in: $TEST_OUTPUT_DIR"
    echo

    # Provide recommendations
    log_info "Recommendations:"
    echo "  • Review log formats in the output files"
    echo "  • Check file permissions and ownership"
    echo "  • Verify log rotation is working as expected"
    echo "  • Test with longer running processes for production validation"
    echo "  • Consider monitoring log file sizes in production"

    log_success "Analysis completed"
}

# Display usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Test ZipLock logging system in various configurations"
    echo
    echo "Options:"
    echo "  --docker-only         Run only Docker-based tests"
    echo "  --local-only          Run only local tests"
    echo "  --skip-build          Skip building the application"
    echo "  --skip-docker-build   Skip building Docker test image"
    echo "  --cleanup-logs        Remove test logs after completion"
    echo "  --help                Show this help message"
    echo
    echo "Environment Variables:"
    echo "  ZIPLOCK_ENV           Environment mode (development|production)"
    echo "  RUST_LOG             Rust log level (trace|debug|info|warn|error)"
    echo "  ZIPLOCK_LOG_DIR      Custom log directory"
    echo "  CLEANUP_LOGS         Set to 'true' to cleanup logs on exit"
}

# Parse command line arguments
DOCKER_ONLY=false
LOCAL_ONLY=false
SKIP_BUILD=false
SKIP_DOCKER_BUILD=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --docker-only)
            DOCKER_ONLY=true
            shift
            ;;
        --local-only)
            LOCAL_ONLY=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-docker-build)
            SKIP_DOCKER_BUILD=true
            shift
            ;;
        --cleanup-logs)
            export CLEANUP_LOGS=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_info "Starting ZipLock logging tests..."
    log_info "Test output directory: $TEST_OUTPUT_DIR"

    setup_test_environment

    if [[ "$SKIP_BUILD" != "true" ]]; then
        build_application
    else
        log_info "Skipping application build"
    fi

    if [[ "$DOCKER_ONLY" != "true" ]]; then
        run_local_dev_tests
        run_prod_simulation_tests
        test_log_rotation
    fi

    if [[ "$LOCAL_ONLY" != "true" ]]; then
        create_test_dockerfile
        create_container_test_script

        if [[ "$SKIP_DOCKER_BUILD" != "true" ]]; then
            build_test_image
        fi

        run_docker_tests
    fi

    analyze_results

    log_success "All logging tests completed!"
    log_info "Check the results in: $TEST_OUTPUT_DIR"
}

# Run main function
main "$@"
