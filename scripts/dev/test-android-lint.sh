#!/bin/bash

# ZipLock Android Lint Testing Script
# Tests Android lint rules locally to catch issues before CI/CD

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ANDROID_DIR="$PROJECT_ROOT/apps/mobile/android"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

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

log_header() {
    echo -e "${BOLD}${BLUE}=== $1 ===${NC}"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 <command> [options]

Test Android lint issues locally to catch problems before CI/CD.

COMMANDS:
    quick              Run quick lint check (same as CI lintVitalRelease)
    full               Run comprehensive lint analysis
    baseline           Create lint baseline to ignore existing issues
    fix                Attempt to auto-fix lint issues
    check-xml          Validate XML files specifically
    report             Generate detailed lint report
    clean              Clean lint artifacts
    help               Show this help

EXAMPLES:
    $0 quick           # Quick lint check (CI equivalent)
    $0 full            # Full lint analysis with all checks
    $0 baseline        # Create baseline for existing issues
    $0 check-xml       # Validate backup/data extraction XML
    $0 report          # Generate HTML lint report

ENVIRONMENT VARIABLES:
    ANDROID_HOME       Path to Android SDK (auto-detected)
    JAVA_HOME          Path to Java (auto-detected)
    LINT_STRICT        Set to 'true' for strict mode (default: false)

The 'quick' command runs the same lint check that failed in CI.
EOF
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if we're in the right directory
    if [ ! -f "$ANDROID_DIR/gradlew" ]; then
        log_error "Android project not found at $ANDROID_DIR"
        log_error "Make sure you're running this from the project root"
        exit 1
    fi

    # Check Java
    if ! command -v java &> /dev/null; then
        log_error "Java not found in PATH"
        exit 1
    fi

    # Auto-detect JAVA_HOME if not set
    if [ -z "${JAVA_HOME:-}" ]; then
        if command -v java &> /dev/null; then
            JAVA_PATH=$(which java)
            # Get Java home by following symlinks and going up directories
            JAVA_HOME=$(dirname "$(dirname "$(readlink -f "$JAVA_PATH")")")
            export JAVA_HOME
            log_info "Auto-detected JAVA_HOME: $JAVA_HOME"
        fi
    else
        log_info "Using JAVA_HOME: $JAVA_HOME"
    fi

    # Auto-detect ANDROID_HOME if not set
    if [ -z "${ANDROID_HOME:-}" ]; then
        # Common Android SDK locations
        for sdk_path in "$HOME/Android/Sdk" "/opt/android-sdk" "/usr/local/lib/android/sdk"; do
            if [ -d "$sdk_path" ]; then
                export ANDROID_HOME="$sdk_path"
                export ANDROID_SDK_ROOT="$ANDROID_HOME"
                log_info "Auto-detected ANDROID_HOME: $ANDROID_HOME"
                break
            fi
        done

        if [ -z "${ANDROID_HOME:-}" ]; then
            log_warning "ANDROID_HOME not found - using containerized build may be needed"
        fi
    else
        log_info "Using ANDROID_HOME: $ANDROID_HOME"
        export ANDROID_SDK_ROOT="$ANDROID_HOME"
    fi

    # Make gradlew executable
    chmod +x "$ANDROID_DIR/gradlew"

    log_success "Prerequisites check completed"
}

# Run quick lint check (equivalent to CI)
run_quick_lint() {
    log_header "Running Quick Lint Check (CI Equivalent)"
    log_info "This runs the same lintVitalRelease task that failed in CI"

    cd "$ANDROID_DIR"

    # Clean previous build artifacts to ensure fresh check
    log_info "Cleaning previous build artifacts..."
    ./gradlew clean --quiet

    # Run the exact same command that failed in CI
    log_info "Running lintVitalRelease (this is what CI runs)..."

    local lint_exit_code=0
    ./gradlew lintVitalRelease --stacktrace || lint_exit_code=$?

    if [ $lint_exit_code -eq 0 ]; then
        log_success "✅ Quick lint check PASSED"
        log_success "The XML backup configuration fix resolved the CI issue!"
    else
        log_error "❌ Quick lint check FAILED with exit code $lint_exit_code"
        log_error "This means the CI will still fail"

        # Show lint results if available
        local lint_report="app/build/reports/lint-results-vitalRelease.html"
        if [ -f "$lint_report" ]; then
            log_info "Lint report available at: $lint_report"
            log_info "Open it in a browser to see detailed results"
        fi

        return $lint_exit_code
    fi
}

# Run comprehensive lint analysis
run_full_lint() {
    log_header "Running Full Lint Analysis"

    cd "$ANDROID_DIR"

    # Clean first
    ./gradlew clean --quiet

    log_info "Running comprehensive lint check..."
    ./gradlew lint --stacktrace

    # Show results
    local results_dir="app/build/reports"
    if [ -d "$results_dir" ]; then
        log_info "Lint reports generated:"
        find "$results_dir" -name "lint-results*" -type f | while read -r file; do
            log_info "  - $(basename "$file"): $file"
        done

        # Show HTML report if available
        local html_report="$results_dir/lint-results-debug.html"
        if [ -f "$html_report" ]; then
            log_success "📊 Open HTML report: file://$html_report"
        fi
    fi
}

# Create lint baseline
create_baseline() {
    log_header "Creating Lint Baseline"
    log_info "This creates a baseline to ignore existing issues"

    cd "$ANDROID_DIR"

    # Add baseline configuration to build.gradle if not present
    local build_gradle="app/build.gradle"
    if ! grep -q "baseline.*lint-baseline.xml" "$build_gradle"; then
        log_info "Adding lint baseline configuration to build.gradle..."

        # Create a backup
        cp "$build_gradle" "$build_gradle.backup"

        # Add lint configuration
        local lint_config='
    lint {
        baseline = file("lint-baseline.xml")
        abortOnError false
    }'

        # Insert after android { block
        if grep -q "android {" "$build_gradle"; then
            sed -i '/android {/a\'"$lint_config" "$build_gradle"
            log_success "Added lint baseline configuration"
        else
            log_warning "Could not automatically add lint configuration"
            log_info "Please add this to your app/build.gradle android block:"
            echo -e "${YELLOW}$lint_config${NC}"
        fi
    fi

    # Run baseline creation
    log_info "Creating lint baseline..."
    ./gradlew updateLintBaseline

    if [ -f "app/lint-baseline.xml" ]; then
        log_success "✅ Lint baseline created at app/lint-baseline.xml"
        log_info "This file contains $(grep -c '<issue' app/lint-baseline.xml) baseline issues"
    else
        log_error "❌ Failed to create lint baseline"
    fi
}

# Attempt to auto-fix lint issues
auto_fix_lint() {
    log_header "Attempting Auto-Fix of Lint Issues"

    cd "$ANDROID_DIR"

    log_info "Running lint with auto-fix where possible..."

    # Some lint issues can be auto-fixed
    ./gradlew lintFix --stacktrace || true

    log_info "Auto-fix completed - please review changes"
    log_warning "Note: Not all lint issues can be auto-fixed"
}

# Validate XML files specifically
check_xml_files() {
    log_header "Validating XML Configuration Files"

    local xml_files=(
        "$ANDROID_DIR/app/src/main/res/xml/backup_rules.xml"
        "$ANDROID_DIR/app/src/main/res/xml/data_extraction_rules.xml"
    )

    for xml_file in "${xml_files[@]}"; do
        if [ -f "$xml_file" ]; then
            log_info "Checking $(basename "$xml_file")..."

            # Basic XML syntax check
            if xmllint --noout "$xml_file" 2>/dev/null; then
                log_success "✅ $(basename "$xml_file") - Valid XML syntax"
            else
                log_error "❌ $(basename "$xml_file") - Invalid XML syntax"
                xmllint "$xml_file" || true
                continue
            fi

            # Check for common backup rule issues
            if [[ "$(basename "$xml_file")" == *"backup"* ]]; then
                log_info "Checking backup rules compliance..."

                # Check for exclude without include pattern
                if grep -q "<exclude" "$xml_file" && ! grep -q "<include" "$xml_file"; then
                    log_warning "⚠️  Found exclude rules without include rules"
                    log_info "This can cause FullBackupContent lint errors"
                fi

                # Show content summary
                local includes=$(grep -c "<include" "$xml_file" 2>/dev/null || echo "0")
                local excludes=$(grep -c "<exclude" "$xml_file" 2>/dev/null || echo "0")
                log_info "  - Include rules: $includes"
                log_info "  - Exclude rules: $excludes"
            fi
        else
            log_warning "⚠️  XML file not found: $(basename "$xml_file")"
        fi
    done
}

# Generate detailed lint report
generate_report() {
    log_header "Generating Detailed Lint Report"

    cd "$ANDROID_DIR"

    ./gradlew clean --quiet
    ./gradlew lintDebug --stacktrace || true

    local report_dir="app/build/reports"
    local output_dir="$PROJECT_ROOT/target/lint-reports"
    mkdir -p "$output_dir"

    if [ -d "$report_dir" ]; then
        # Copy reports to target directory
        cp -r "$report_dir"/* "$output_dir/" 2>/dev/null || true

        log_success "📊 Lint reports copied to: $output_dir"

        # List available reports
        find "$output_dir" -name "lint-results*" -type f | while read -r file; do
            local file_size=$(stat -c%s "$file")
            log_info "  - $(basename "$file") (${file_size} bytes): $file"
        done

        # Open HTML report if available
        local html_report="$output_dir/lint-results-debug.html"
        if [ -f "$html_report" ]; then
            log_success "🌐 Open in browser: file://$html_report"
        fi
    else
        log_warning "No lint reports found"
    fi
}

# Clean lint artifacts
clean_lint() {
    log_header "Cleaning Lint Artifacts"

    cd "$ANDROID_DIR"

    # Clean Gradle build
    ./gradlew clean --quiet

    # Remove lint reports
    rm -rf app/build/reports/lint*
    rm -rf "$PROJECT_ROOT/target/lint-reports"

    # Remove baseline if requested
    if [ -f "app/lint-baseline.xml" ]; then
        read -p "Remove lint baseline file? [y/N]: " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -f "app/lint-baseline.xml"
            log_success "Removed lint baseline"
        fi
    fi

    log_success "✅ Lint artifacts cleaned"
}

# Containerized lint check (fallback for missing Android SDK)
run_containerized_lint() {
    log_header "Running Containerized Lint Check"
    log_info "Using Docker container with pre-configured Android environment"

    if ! command -v docker &> /dev/null; then
        log_error "Docker not available for containerized testing"
        exit 1
    fi

    # Use the build script to set up container and run lint
    local build_script="$PROJECT_ROOT/scripts/build/build-android-docker.sh"
    if [ -f "$build_script" ]; then
        log_info "Building libraries first to ensure everything is set up..."
        "$build_script" build

        log_info "Running lint in container..."
        # Create a custom command to run lint in the container
        local container_cmd="cd /workspace/apps/mobile/android && ./gradlew lintVitalRelease --stacktrace"

        docker run --rm \
            -v "$PROJECT_ROOT:/workspace" \
            -w /workspace \
            ghcr.io/ejangi/ziplock/android-builder:latest \
            bash -c "$container_cmd"
    else
        log_error "Build script not found: $build_script"
        exit 1
    fi
}

# Main function
main() {
    if [ $# -eq 0 ]; then
        show_usage
        exit 1
    fi

    local command="$1"
    shift

    case "$command" in
        "quick")
            check_prerequisites
            run_quick_lint
            ;;
        "full")
            check_prerequisites
            run_full_lint
            ;;
        "baseline")
            check_prerequisites
            create_baseline
            ;;
        "fix")
            check_prerequisites
            auto_fix_lint
            ;;
        "check-xml")
            check_xml_files
            ;;
        "report")
            check_prerequisites
            generate_report
            ;;
        "clean")
            check_prerequisites
            clean_lint
            ;;
        "container")
            run_containerized_lint
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            echo
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
