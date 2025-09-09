#!/bin/bash

# ZipLock Archive Creation Flow Test Runner
# This script runs the comprehensive archive creation flow tests on Android

set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ANDROID_APP_DIR="$PROJECT_ROOT/apps/mobile/android"
TEST_CLASS="com.ziplock.integration.ArchiveCreationFlowTest"
LOG_DIR="$PROJECT_ROOT/test-logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
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

print_header() {
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE} ZipLock Archive Creation Flow Tests${NC}"
    echo -e "${BLUE}============================================${NC}"
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."

    # Check if we're in the right directory
    if [ ! -d "$ANDROID_APP_DIR" ]; then
        print_error "Android app directory not found: $ANDROID_APP_DIR"
        exit 1
    fi

    # Check for Android SDK
    if [ -z "$ANDROID_HOME" ] && [ -z "$ANDROID_SDK_ROOT" ]; then
        print_error "ANDROID_HOME or ANDROID_SDK_ROOT environment variable not set"
        exit 1
    fi

    # Check for adb
    if ! command -v adb &> /dev/null; then
        print_error "adb not found in PATH"
        exit 1
    fi

    # Check for connected devices/emulators
    DEVICE_COUNT=$(adb devices | grep -c "device$" || echo "0")
    if [ "$DEVICE_COUNT" -eq 0 ]; then
        print_error "No Android devices or emulators connected"
        print_info "Please connect a device or start an emulator and try again"
        exit 1
    fi

    print_success "Prerequisites check passed"
}

# Function to setup test environment
setup_test_environment() {
    print_info "Setting up test environment..."

    # Create log directory
    mkdir -p "$LOG_DIR"

    # Navigate to Android app directory
    cd "$ANDROID_APP_DIR"

    # Clean previous builds
    print_info "Cleaning previous builds..."
    ./gradlew clean > "$LOG_DIR/clean_${TIMESTAMP}.log" 2>&1

    # Build the app and test APKs
    print_info "Building app and test APKs..."
    ./gradlew assembleDebug assembleDebugAndroidTest > "$LOG_DIR/build_${TIMESTAMP}.log" 2>&1

    print_success "Test environment setup complete"
}

# Function to install APKs
install_apks() {
    print_info "Installing APKs on device..."

    # Install main app APK
    APP_APK=$(find app/build/outputs/apk/debug -name "*.apk" | head -1)
    if [ -n "$APP_APK" ] && [ -f "$APP_APK" ]; then
        print_info "Installing app APK: $APP_APK"
        adb install -r "$APP_APK" > "$LOG_DIR/install_app_${TIMESTAMP}.log" 2>&1
    else
        print_error "App APK not found"
        exit 1
    fi

    # Install test APK
    TEST_APK=$(find app/build/outputs/apk/androidTest/debug -name "*.apk" | head -1)
    if [ -n "$TEST_APK" ] && [ -f "$TEST_APK" ]; then
        print_info "Installing test APK: $TEST_APK"
        adb install -r "$TEST_APK" > "$LOG_DIR/install_test_${TIMESTAMP}.log" 2>&1
    else
        print_error "Test APK not found"
        exit 1
    fi

    print_success "APK installation complete"
}

# Function to run specific test method
run_test_method() {
    local test_method="$1"
    local log_file="$LOG_DIR/test_${test_method}_${TIMESTAMP}.log"

    print_info "Running test: $test_method"

    adb shell am instrument -w \
        -e class "${TEST_CLASS}#${test_method}" \
        com.ziplock.test/androidx.test.runner.AndroidJUnitRunner \
        > "$log_file" 2>&1

    # Check test result
    if grep -q "OK (" "$log_file"; then
        print_success "✅ $test_method PASSED"
        return 0
    else
        print_error "❌ $test_method FAILED"
        print_info "Check log file: $log_file"
        return 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_info "Running all archive creation flow tests..."

    local test_methods=(
        "testCompleteArchiveCreationWorkflow"
        "testPasswordValidationFlow"
        "testArchiveFormatAndEncryption"
        "testErrorHandlingAndEdgeCases"
        "testMemoryManagementAndPerformance"
        "testCrossPlatformCompatibility"
        "testLegacyCompatibility"
    )

    local passed_tests=0
    local total_tests=${#test_methods[@]}
    local failed_tests=()

    for test_method in "${test_methods[@]}"; do
        if run_test_method "$test_method"; then
            ((passed_tests++))
        else
            failed_tests+=("$test_method")
        fi
        echo # Add spacing between tests
    done

    # Print summary
    print_header
    echo -e "${BLUE}Test Results Summary${NC}"
    echo "==================="
    echo "Total Tests: $total_tests"
    echo "Passed: $passed_tests"
    echo "Failed: $((total_tests - passed_tests))"
    echo

    if [ ${#failed_tests[@]} -eq 0 ]; then
        print_success "🎉 All tests passed!"
        return 0
    else
        print_error "❌ Failed tests:"
        for failed_test in "${failed_tests[@]}"; do
            echo "  - $failed_test"
        done
        return 1
    fi
}

# Function to run quick smoke test
run_smoke_test() {
    print_info "Running smoke test (core workflow only)..."
    run_test_method "testCompleteArchiveCreationWorkflow"
}

# Function to clean up
cleanup() {
    print_info "Cleaning up test environment..."

    # Clear app data
    adb shell pm clear com.ziplock 2>/dev/null || true

    # Remove temporary files from device
    adb shell rm -rf /sdcard/Android/data/com.ziplock/cache/archive_flow_test_* 2>/dev/null || true

    print_success "Cleanup complete"
}

# Function to show device info
show_device_info() {
    print_info "Connected Android devices/emulators:"
    adb devices -l
    echo

    print_info "Device information:"
    adb shell getprop ro.build.version.release
    adb shell getprop ro.build.version.sdk
    adb shell getprop ro.product.model
    echo
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo
    echo "Options:"
    echo "  -a, --all           Run all archive creation flow tests"
    echo "  -s, --smoke         Run smoke test (core workflow only)"
    echo "  -t, --test METHOD   Run specific test method"
    echo "  -c, --clean         Clean up test environment"
    echo "  -i, --info          Show device information"
    echo "  -h, --help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0 --all                                    # Run all tests"
    echo "  $0 --smoke                                  # Run smoke test"
    echo "  $0 --test testPasswordValidationFlow       # Run specific test"
    echo "  $0 --clean                                  # Clean up only"
    echo
}

# Main execution
main() {
    print_header

    # Parse command line arguments
    case "$1" in
        -a|--all)
            show_device_info
            check_prerequisites
            setup_test_environment
            install_apks
            run_all_tests
            cleanup
            ;;
        -s|--smoke)
            show_device_info
            check_prerequisites
            setup_test_environment
            install_apks
            run_smoke_test
            cleanup
            ;;
        -t|--test)
            if [ -z "$2" ]; then
                print_error "Test method name required"
                show_usage
                exit 1
            fi
            show_device_info
            check_prerequisites
            setup_test_environment
            install_apks
            run_test_method "$2"
            cleanup
            ;;
        -c|--clean)
            cleanup
            ;;
        -i|--info)
            show_device_info
            ;;
        -h|--help|"")
            show_usage
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function with all arguments
main "$@"
