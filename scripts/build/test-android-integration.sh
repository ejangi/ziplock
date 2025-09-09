#!/bin/bash
set -euo pipefail

# ZipLock Android Integration Test Script
# Tests Android native libraries for functionality, performance, and security

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test configuration
TEST_OUTPUT_DIR="$PROJECT_ROOT/target/test-results"
ANDROID_LIBS_DIR="$PROJECT_ROOT/target/android"

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

# Show usage
show_usage() {
    cat << EOF
Usage: $0 <test_type> [options]

Test Android native libraries for various aspects.

TEST TYPES:
    basic          Basic functionality tests (library loading, symbols)
    performance    Performance and size analysis
    security       Security features analysis
    integration    Full integration tests
    all            Run all test types

OPTIONS:
    -v, --verbose    Enable verbose output
    -o, --output     Output directory for test results (default: target/test-results)

EXAMPLES:
    $0 basic                    # Run basic functionality tests
    $0 performance              # Run performance tests
    $0 security                 # Run security analysis
    $0 all -v                   # Run all tests with verbose output

EOF
}

# Check if required tools are available
check_dependencies() {
    local missing_tools=()

    # Check for basic tools
    for tool in file readelf objdump; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done

    # Check for Android-specific tools (optional)
    if ! command -v adb &> /dev/null; then
        log_warning "adb not found - device testing will be skipped"
    fi

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Install with: sudo apt-get install ${missing_tools[*]}"
        exit 1
    fi
}

# Initialize test environment
init_test_env() {
    log_info "Initializing test environment..."

    mkdir -p "$TEST_OUTPUT_DIR"

    # Check if Android libraries exist
    if [ ! -d "$ANDROID_LIBS_DIR" ]; then
        log_error "Android libraries not found at $ANDROID_LIBS_DIR"
        log_info "Run './scripts/build/build-android-docker.sh build' first"
        exit 1
    fi

    log_success "Test environment initialized"
}

# Basic functionality tests
test_basic_functionality() {
    log_info "Running basic functionality tests..."

    local test_results="$TEST_OUTPUT_DIR/basic-tests.txt"
    local failed_tests=0

    echo "=== ZipLock Android Basic Tests ===" > "$test_results"
    echo "Timestamp: $(date)" >> "$test_results"
    echo >> "$test_results"

    # Test each architecture
    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        echo "Testing $arch:" >> "$test_results"

        if [ ! -f "$lib_path" ]; then
            echo "  ✗ Library not found" >> "$test_results"
            log_warning "Library not found for $arch"
            ((failed_tests++))
            continue
        fi

        echo "  ✓ Library exists" >> "$test_results"

        # Check file type
        local file_info
        file_info=$(file "$lib_path")
        echo "  File type: $file_info" >> "$test_results"

        if [[ "$file_info" == *"shared object"* ]]; then
            echo "  ✓ Valid shared object" >> "$test_results"
        else
            echo "  ✗ Invalid file type" >> "$test_results"
            ((failed_tests++))
        fi

        # Check file size
        local file_size
        file_size=$(stat -c%s "$lib_path")
        echo "  Size: $file_size bytes ($(echo "scale=2; $file_size / 1024 / 1024" | bc -l) MB)" >> "$test_results"

        if [ "$file_size" -gt 100 ] && [ "$file_size" -lt $((50 * 1024 * 1024)) ]; then
            echo "  ✓ Reasonable file size" >> "$test_results"
        else
            echo "  ⚠ Unusual file size" >> "$test_results"
        fi

        # Check for ziplock symbols
        if readelf -s "$lib_path" 2>/dev/null | grep -q "ziplock_"; then
            echo "  ✓ Contains ziplock symbols" >> "$test_results"
        else
            echo "  ⚠ No ziplock symbols found" >> "$test_results"
        fi

        # Check for required sections
        local sections
        sections=$(readelf -S "$lib_path" 2>/dev/null | grep -E "\.(text|data|rodata)" | wc -l)
        if [ "$sections" -ge 3 ]; then
            echo "  ✓ Has required sections" >> "$test_results"
        else
            echo "  ⚠ Missing expected sections" >> "$test_results"
        fi

        # Check dependencies
        local deps
        deps=$(readelf -d "$lib_path" 2>/dev/null | grep "NEEDED" | wc -l)
        echo "  Dependencies: $deps" >> "$test_results"

        echo >> "$test_results"
    done

    # Test header file
    local header_path="$ANDROID_LIBS_DIR/ziplock.h"
    echo "Testing header file:" >> "$test_results"

    if [ -f "$header_path" ]; then
        echo "  ✓ Header file exists" >> "$test_results"

        local function_count
        function_count=$(grep -c "^[[:space:]]*[a-zA-Z].*(" "$header_path" 2>/dev/null || echo 0)
        echo "  Functions declared: $function_count" >> "$test_results"

        if [ "$function_count" -gt 0 ]; then
            echo "  ✓ Contains function declarations" >> "$test_results"
        else
            echo "  ⚠ No function declarations found" >> "$test_results"
        fi
    else
        echo "  ⚠ Header file not found" >> "$test_results"
    fi

    echo >> "$test_results"
    echo "=== Test Summary ===" >> "$test_results"
    echo "Failed tests: $failed_tests" >> "$test_results"

    if [ "$failed_tests" -eq 0 ]; then
        log_success "All basic functionality tests passed"
    else
        log_warning "$failed_tests basic functionality tests failed"
    fi

    log_info "Basic test results saved to: $test_results"
}

# Performance tests
test_performance() {
    log_info "Running performance tests..."

    local test_results="$TEST_OUTPUT_DIR/performance-tests.txt"

    echo "=== ZipLock Android Performance Tests ===" > "$test_results"
    echo "Timestamp: $(date)" >> "$test_results"
    echo >> "$test_results"

    # Size analysis
    echo "Library Size Analysis:" >> "$test_results"
    echo "| Architecture | Size (bytes) | Size (MB) | Status |" >> "$test_results"
    echo "|--------------|--------------|-----------|--------|" >> "$test_results"

    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")
    local total_size=0

    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        if [ -f "$lib_path" ]; then
            local size_bytes
            size_bytes=$(stat -c%s "$lib_path")
            local size_mb
            size_mb=$(echo "scale=2; $size_bytes / 1024 / 1024" | bc -l)

            local status="Good"
            if [ "$size_bytes" -gt $((20 * 1024 * 1024)) ]; then
                status="Large"
            elif [ "$size_bytes" -lt 1024 ]; then
                status="Too small"
            fi

            echo "| $arch | $size_bytes | ${size_mb} MB | $status |" >> "$test_results"
            total_size=$((total_size + size_bytes))
        else
            echo "| $arch | - | - | Missing |" >> "$test_results"
        fi
    done

    local total_mb
    total_mb=$(echo "scale=2; $total_size / 1024 / 1024" | bc -l)
    echo "| **Total** | **$total_size** | **${total_mb} MB** | - |" >> "$test_results"
    echo >> "$test_results"

    # Symbol analysis
    echo "Symbol Analysis:" >> "$test_results"
    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        if [ -f "$lib_path" ]; then
            echo "$arch:" >> "$test_results"

            local total_symbols
            total_symbols=$(readelf -s "$lib_path" 2>/dev/null | wc -l)
            echo "  Total symbols: $total_symbols" >> "$test_results"

            local exported_symbols
            exported_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep -c " GLOBAL " || echo 0)
            echo "  Exported symbols: $exported_symbols" >> "$test_results"

            local ziplock_symbols
            ziplock_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep -c "ziplock_" || echo 0)
            echo "  ZipLock symbols: $ziplock_symbols" >> "$test_results"

            echo >> "$test_results"
        fi
    done

    log_success "Performance tests completed"
    log_info "Performance test results saved to: $test_results"
}

# Security tests
test_security() {
    log_info "Running security tests..."

    local test_results="$TEST_OUTPUT_DIR/security-tests.txt"

    echo "=== ZipLock Android Security Tests ===" > "$test_results"
    echo "Timestamp: $(date)" >> "$test_results"
    echo >> "$test_results"

    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        if [ ! -f "$lib_path" ]; then
            continue
        fi

        echo "Security Analysis for $arch:" >> "$test_results"

        # Check for stack protection
        if readelf -s "$lib_path" 2>/dev/null | grep -q "__stack_chk"; then
            echo "  ✓ Stack protection enabled" >> "$test_results"
        else
            echo "  ⚠ Stack protection not detected" >> "$test_results"
        fi

        # Check for position independent code
        if readelf -h "$lib_path" 2>/dev/null | grep -q "DYN"; then
            echo "  ✓ Position independent executable" >> "$test_results"
        else
            echo "  ⚠ Not position independent" >> "$test_results"
        fi

        # Check for RELRO (Read-Only Relocations)
        if readelf -l "$lib_path" 2>/dev/null | grep -q "GNU_RELRO"; then
            echo "  ✓ RELRO protection enabled" >> "$test_results"
        else
            echo "  ⚠ RELRO protection not found" >> "$test_results"
        fi

        # Check for executable stack
        if readelf -l "$lib_path" 2>/dev/null | grep "GNU_STACK" | grep -q "E"; then
            echo "  ⚠ Executable stack detected" >> "$test_results"
        else
            echo "  ✓ Non-executable stack" >> "$test_results"
        fi

        # Check for stripped symbols
        local symbol_count
        symbol_count=$(readelf -s "$lib_path" 2>/dev/null | wc -l)
        if [ "$symbol_count" -lt 50 ]; then
            echo "  ✓ Symbols stripped ($symbol_count symbols)" >> "$test_results"
        else
            echo "  ⚠ Many symbols present ($symbol_count symbols)" >> "$test_results"
        fi

        # Check for dangerous functions
        local dangerous_functions=("strcpy" "strcat" "sprintf" "gets")
        for func in "${dangerous_functions[@]}"; do
            if readelf -s "$lib_path" 2>/dev/null | grep -q "$func"; then
                echo "  ⚠ Uses potentially unsafe function: $func" >> "$test_results"
            fi
        done

        # Check for hardcoded strings (simple check)
        local strings_output
        strings_output=$(strings "$lib_path" 2>/dev/null | wc -l)
        echo "  String constants: $strings_output" >> "$test_results"

        echo >> "$test_results"
    done

    log_success "Security tests completed"
    log_info "Security test results saved to: $test_results"
}

# Integration tests
test_integration() {
    log_info "Running integration tests..."

    local test_results="$TEST_OUTPUT_DIR/integration-tests.txt"

    echo "=== ZipLock Android Integration Tests ===" > "$test_results"
    echo "Timestamp: $(date)" >> "$test_results"
    echo >> "$test_results"

    # Test Android app integration
    local android_app_dir="$PROJECT_ROOT/apps/mobile/android"
    if [ -d "$android_app_dir" ]; then
        echo "Android App Integration:" >> "$test_results"

        local jnilibs_dir="$android_app_dir/app/src/main/jniLibs"
        if [ -d "$jnilibs_dir" ]; then
            echo "  ✓ jniLibs directory exists" >> "$test_results"

            # Check if libraries are copied
            local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")
            for arch in "${architectures[@]}"; do
                if [ -f "$jnilibs_dir/$arch/libziplock_shared.so" ]; then
                    echo "  ✓ $arch library copied to app" >> "$test_results"
                else
                    echo "  ✗ $arch library missing from app" >> "$test_results"
                fi
            done
        else
            echo "  ⚠ jniLibs directory not found in Android app" >> "$test_results"
        fi

        # Check for build.gradle
        if [ -f "$android_app_dir/app/build.gradle" ]; then
            echo "  ✓ Android build.gradle exists" >> "$test_results"
        else
            echo "  ⚠ Android build.gradle not found" >> "$test_results"
        fi

    else
        echo "Android App Integration:" >> "$test_results"
        echo "  ⚠ Android app directory not found" >> "$test_results"
    fi

    echo >> "$test_results"

    # Test build artifacts
    echo "Build Artifacts:" >> "$test_results"

    if [ -f "$ANDROID_LIBS_DIR/build-info.json" ]; then
        echo "  ✓ Build info exists" >> "$test_results"
        echo "  Build info:" >> "$test_results"
        sed 's/^/    /' "$ANDROID_LIBS_DIR/build-info.json" >> "$test_results"
    else
        echo "  ⚠ Build info missing" >> "$test_results"
    fi

    if [ -f "$ANDROID_LIBS_DIR/ziplock.h" ]; then
        echo "  ✓ C header file exists" >> "$test_results"
    else
        echo "  ⚠ C header file missing" >> "$test_results"
    fi

    echo >> "$test_results"

    log_success "Integration tests completed"
    log_info "Integration test results saved to: $test_results"
}

# Run all tests
run_all_tests() {
    log_info "Running all Android tests..."

    test_basic_functionality
    test_performance
    test_security
    test_integration

    # Create summary report
    local summary_report="$TEST_OUTPUT_DIR/test-summary.txt"

    echo "=== ZipLock Android Test Summary ===" > "$summary_report"
    echo "Timestamp: $(date)" >> "$summary_report"
    echo >> "$summary_report"

    echo "Test Results:" >> "$summary_report"
    echo "- Basic functionality: $([ -f "$TEST_OUTPUT_DIR/basic-tests.txt" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$summary_report"
    echo "- Performance analysis: $([ -f "$TEST_OUTPUT_DIR/performance-tests.txt" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$summary_report"
    echo "- Security analysis: $([ -f "$TEST_OUTPUT_DIR/security-tests.txt" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$summary_report"
    echo "- Integration tests: $([ -f "$TEST_OUTPUT_DIR/integration-tests.txt" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$summary_report"
    echo >> "$summary_report"

    echo "Output Files:" >> "$summary_report"
    for file in "$TEST_OUTPUT_DIR"/*.txt; do
        if [ -f "$file" ]; then
            echo "- $(basename "$file")" >> "$summary_report"
        fi
    done

    log_success "All tests completed"
    log_info "Summary report: $summary_report"
}

# Main function
main() {
    local test_type=""
    local verbose=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--verbose)
                verbose=true
                shift
                ;;
            -o|--output)
                TEST_OUTPUT_DIR="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                if [ -z "$test_type" ]; then
                    test_type="$1"
                else
                    log_error "Unknown option: $1"
                    show_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done

    if [ -z "$test_type" ]; then
        log_error "Test type is required"
        show_usage
        exit 1
    fi

    # Set verbose output
    if [ "$verbose" = true ]; then
        set -x
    fi

    # Check dependencies and initialize
    check_dependencies
    init_test_env

    # Run requested tests
    case "$test_type" in
        "basic")
            test_basic_functionality
            ;;
        "performance")
            test_performance
            ;;
        "security")
            test_security
            ;;
        "integration")
            test_integration
            ;;
        "all")
            run_all_tests
            ;;
        *)
            log_error "Invalid test type: $test_type"
            show_usage
            exit 1
            ;;
    esac

    log_info "Test results available in: $TEST_OUTPUT_DIR"
}

# Run main function
main "$@"
