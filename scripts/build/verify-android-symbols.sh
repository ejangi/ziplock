#!/bin/bash

# Android Symbol Verification Script
# Verifies that Android libraries contain the expected FFI symbols

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
OUTPUT_DIR="$PROJECT_ROOT/target/android"

# Expected FFI symbols
EXPECTED_SYMBOLS=(
    # Core library functions
    "ziplock_init"
    "ziplock_shutdown"
    "ziplock_get_version"
    "ziplock_get_last_error"

    # Session management
    "ziplock_session_create"

    # Archive management
    "ziplock_archive_create"
    "ziplock_archive_open"
    "ziplock_archive_close"
    "ziplock_archive_save"
    "ziplock_is_archive_open"

    # Credential management
    "ziplock_credential_list"
    "ziplock_credential_list_free"

    # Memory management
    "ziplock_string_free"
)

# Optional symbols (may not be present in all builds)
OPTIONAL_SYMBOLS=(
    # Debug/testing functions
    "ziplock_test_echo"
    "ziplock_debug_logging"

    # Extended API functions
    "ziplock_credential_new"
    "ziplock_credential_from_template"
    "ziplock_credential_add_field"
    "ziplock_credential_get_field"
    "ziplock_credential_remove_field"
    "ziplock_credential_validate"

    # Password utilities
    "ziplock_password_generate"
    "ziplock_password_validate"

    # Validation functions
    "ziplock_email_validate"
    "ziplock_url_validate"

    # TOTP functions
    "ziplock_totp_generate"

    # Credit card utilities
    "ziplock_credit_card_format"
)

# Check if symbol extraction tools are available
check_tools() {
    print_status "Checking symbol extraction tools..."

    local tools_available=0

    if command -v nm >/dev/null 2>&1; then
        print_success "✓ nm available"
        tools_available=1
    fi

    if command -v objdump >/dev/null 2>&1; then
        print_success "✓ objdump available"
        tools_available=1
    fi

    if command -v readelf >/dev/null 2>&1; then
        print_success "✓ readelf available"
        tools_available=1
    fi

    if [ $tools_available -eq 0 ]; then
        print_error "No symbol extraction tools available (nm, objdump, readelf)"
        print_error "Please install binutils package"
        exit 1
    fi

    return 0
}

# Extract symbols from a library using available tools
extract_symbols() {
    local lib_path="$1"

    if command -v nm >/dev/null 2>&1; then
        # Use nm (preferred)
        nm -D "$lib_path" 2>/dev/null | grep -E "ziplock_" | cut -d' ' -f3 | sort
    elif command -v objdump >/dev/null 2>&1; then
        # Use objdump as fallback
        objdump -T "$lib_path" 2>/dev/null | grep -E "ziplock_" | awk '{print $NF}' | sort
    elif command -v readelf >/dev/null 2>&1; then
        # Use readelf as last resort
        readelf -Ws "$lib_path" 2>/dev/null | grep -E "ziplock_" | awk '{print $8}' | sort
    else
        print_error "No symbol extraction tool available"
        return 1
    fi
}

# Verify symbols in a single library
verify_library_symbols() {
    local lib_path="$1"
    local arch="$2"

    print_status "Verifying symbols in $arch library..."

    if [ ! -f "$lib_path" ]; then
        print_error "Library not found: $lib_path"
        return 1
    fi

    # Extract symbols
    local symbols
    symbols=$(extract_symbols "$lib_path")

    if [ -z "$symbols" ]; then
        print_error "No ziplock symbols found in $arch library"
        return 1
    fi

    # Check required symbols
    local missing_required=()
    local found_required=()

    for symbol in "${EXPECTED_SYMBOLS[@]}"; do
        if echo "$symbols" | grep -q "^$symbol$"; then
            found_required+=("$symbol")
        else
            missing_required+=("$symbol")
        fi
    done

    # Check optional symbols
    local found_optional=()

    for symbol in "${OPTIONAL_SYMBOLS[@]}"; do
        if echo "$symbols" | grep -q "^$symbol$"; then
            found_optional+=("$symbol")
        fi
    done

    # Report results
    echo "  Required symbols: ${#found_required[@]}/${#EXPECTED_SYMBOLS[@]} found"
    echo "  Optional symbols: ${#found_optional[@]}/${#OPTIONAL_SYMBOLS[@]} found"
    echo "  Total ziplock symbols: $(echo "$symbols" | wc -l)"

    # Show missing required symbols
    if [ ${#missing_required[@]} -gt 0 ]; then
        print_error "  Missing required symbols in $arch:"
        for symbol in "${missing_required[@]}"; do
            echo "    ✗ $symbol"
        done
        return 1
    else
        print_success "  ✓ All required symbols found in $arch"
    fi

    # Show found optional symbols (informational)
    if [ ${#found_optional[@]} -gt 0 ]; then
        echo "  Found optional symbols:"
        for symbol in "${found_optional[@]}"; do
            echo "    + $symbol"
        done
    fi

    return 0
}

# Verify symbols across all architectures
verify_all_symbols() {
    print_status "Verifying symbols in all Android libraries..."
    echo ""

    local failed_archs=()
    local successful_archs=()

    for arch in arm64-v8a armeabi-v7a x86_64 x86; do
        local lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"

        if [ -f "$lib_path" ]; then
            if verify_library_symbols "$lib_path" "$arch"; then
                successful_archs+=("$arch")
            else
                failed_archs+=("$arch")
            fi
            echo ""
        else
            print_warning "Library not found for $arch, skipping..."
            echo ""
        fi
    done

    # Summary
    echo "Symbol Verification Summary:"
    echo "============================"

    if [ ${#successful_archs[@]} -gt 0 ]; then
        echo ""
        echo "✅ Successful architectures:"
        for arch in "${successful_archs[@]}"; do
            echo "  ✓ $arch"
        done
    fi

    if [ ${#failed_archs[@]} -gt 0 ]; then
        echo ""
        echo "❌ Failed architectures:"
        for arch in "${failed_archs[@]}"; do
            echo "  ✗ $arch"
        done
        echo ""
        print_error "Some architectures have missing symbols"
        return 1
    fi

    echo ""
    print_success "All libraries contain required symbols!"
    return 0
}

# Compare symbols across architectures
compare_symbols_across_archs() {
    print_status "Comparing symbols across architectures..."

    local reference_arch="arm64-v8a"
    local reference_lib="$OUTPUT_DIR/$reference_arch/libziplock_shared.so"

    if [ ! -f "$reference_lib" ]; then
        print_warning "Reference library not found ($reference_arch), skipping comparison"
        return 0
    fi

    local reference_symbols
    reference_symbols=$(extract_symbols "$reference_lib")

    echo "Using $reference_arch as reference ($(echo "$reference_symbols" | wc -l) symbols)"
    echo ""

    for arch in armeabi-v7a x86_64 x86; do
        local lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"

        if [ -f "$lib_path" ]; then
            local arch_symbols
            arch_symbols=$(extract_symbols "$lib_path")

            echo "Comparing $arch with $reference_arch:"

            # Find differences
            local missing_in_arch
            local extra_in_arch

            missing_in_arch=$(comm -23 <(echo "$reference_symbols") <(echo "$arch_symbols"))
            extra_in_arch=$(comm -13 <(echo "$reference_symbols") <(echo "$arch_symbols"))

            if [ -z "$missing_in_arch" ] && [ -z "$extra_in_arch" ]; then
                print_success "  ✓ Identical symbol sets"
            else
                if [ -n "$missing_in_arch" ]; then
                    print_warning "  Missing in $arch:"
                    echo "$missing_in_arch" | sed 's/^/    - /'
                fi

                if [ -n "$extra_in_arch" ]; then
                    print_warning "  Extra in $arch:"
                    echo "$extra_in_arch" | sed 's/^/    + /'
                fi
            fi
            echo ""
        fi
    done
}

# Show detailed symbol information
show_symbol_details() {
    local arch="${1:-arm64-v8a}"
    local lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"

    print_status "Showing detailed symbol information for $arch..."

    if [ ! -f "$lib_path" ]; then
        print_error "Library not found: $lib_path"
        return 1
    fi

    local symbols
    symbols=$(extract_symbols "$lib_path")

    echo ""
    echo "All ziplock symbols in $arch:"
    echo "=============================="
    echo "$symbols" | sed 's/^/  /'

    echo ""
    echo "Symbol categories:"
    echo "=================="

    # Categorize symbols
    echo "Core functions:"
    echo "$symbols" | grep -E "(init|shutdown|version|error)" | sed 's/^/  /'

    echo ""
    echo "Archive functions:"
    echo "$symbols" | grep -E "archive" | sed 's/^/  /'

    echo ""
    echo "Credential functions:"
    echo "$symbols" | grep -E "credential" | sed 's/^/  /'

    echo ""
    echo "Utility functions:"
    echo "$symbols" | grep -E "(password|email|url|totp|credit)" | sed 's/^/  /'

    echo ""
    echo "Memory management:"
    echo "$symbols" | grep -E "(free|string)" | sed 's/^/  /'

    echo ""
    echo "Other functions:"
    echo "$symbols" | grep -vE "(init|shutdown|version|error|archive|credential|password|email|url|totp|credit|free|string)" | sed 's/^/  /'
}

# Usage
usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  verify         Verify symbols in all libraries (default)"
    echo "  compare        Compare symbols across architectures"
    echo "  details [ARCH] Show detailed symbol information"
    echo "  help           Show this help message"
    echo ""
    echo "Options for details:"
    echo "  ARCH           Architecture (arm64-v8a, armeabi-v7a, x86_64, x86)"
    echo ""
    echo "Examples:"
    echo "  $0                     # Verify all libraries"
    echo "  $0 verify              # Verify all libraries"
    echo "  $0 compare             # Compare symbols across architectures"
    echo "  $0 details             # Show details for arm64-v8a"
    echo "  $0 details armeabi-v7a # Show details for ARMv7"
}

# Main function
main() {
    local command="${1:-verify}"

    # Check if build output exists
    if [ ! -d "$OUTPUT_DIR" ]; then
        print_error "Build output directory not found: $OUTPUT_DIR"
        print_error "Please run the build first: ./scripts/build/build-android-docker.sh build"
        exit 1
    fi

    case "$command" in
        "verify")
            check_tools
            verify_all_symbols
            ;;
        "compare")
            check_tools
            compare_symbols_across_archs
            ;;
        "details")
            check_tools
            show_symbol_details "$2"
            ;;
        "help"|"-h"|"--help")
            usage
            ;;
        *)
            print_error "Unknown command: $command"
            echo ""
            usage
            exit 1
            ;;
    esac
}

main "$@"
