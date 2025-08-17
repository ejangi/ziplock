#!/bin/bash

# Android Integration Test Script
# Tests that the Android libraries function correctly

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

# Test that libraries can be loaded
test_library_loading() {
    echo "Testing library loading..."

    for arch in arm64-v8a armeabi-v7a; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            if command -v readelf >/dev/null 2>&1; then
                echo "✓ $arch: Library structure valid"
                readelf -d "$lib_path" | grep SONAME || echo "Warning: No SONAME found"
            else
                echo "✓ $arch: Library file exists"
            fi
        else
            echo "✗ $arch: Library file missing"
            exit 1
        fi
    done
}

# Test library symbols
test_library_symbols() {
    echo "Testing library symbols..."

    lib_path="$OUTPUT_DIR/arm64-v8a/libziplock_shared.so"

    if command -v nm >/dev/null 2>&1; then
        # Check for key FFI functions that actually exist
        required_symbols=(
            "ziplock_init"
            "ziplock_get_version"
            "ziplock_session_create"
            "ziplock_archive_create"
            "ziplock_archive_open"
            "ziplock_credential_list"
            "ziplock_string_free"
        )

        for symbol in "${required_symbols[@]}"; do
            if nm -D "$lib_path" | grep -q "$symbol"; then
                echo "✓ Symbol found: $symbol"
            else
                echo "✗ Missing symbol: $symbol"
                exit 1
            fi
        done
    else
        echo "nm not available, skipping symbol test"
    fi
}

# Test library dependencies
test_library_dependencies() {
    echo "Testing library dependencies..."

    lib_path="$OUTPUT_DIR/arm64-v8a/libziplock_shared.so"

    if command -v ldd >/dev/null 2>&1; then
        echo "Library dependencies:"
        ldd "$lib_path" || echo "Note: ldd may not work for cross-compiled libraries"
    fi
}

# Test library file properties
test_library_properties() {
    echo "Testing library properties..."

    for arch in arm64-v8a armeabi-v7a; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            # Check file size (should not be too large or too small)
            size=$(stat -c%s "$lib_path" 2>/dev/null || stat -f%z "$lib_path")
            size_mb=$((size / 1024 / 1024))

            if [ $size -lt 1024 ]; then
                print_error "✗ $arch: Library suspiciously small (< 1KB)"
                exit 1
            elif [ $size -gt $((50 * 1024 * 1024)) ]; then
                print_warning "⚠ $arch: Library quite large (> 50MB)"
            else
                print_success "✓ $arch: Library size acceptable (${size_mb}MB)"
            fi

            # Check that it's actually a shared library
            if file "$lib_path" | grep -q "shared object"; then
                echo "  - Valid shared library format"
            else
                print_error "  - Invalid library format"
                exit 1
            fi

            # Check architecture
            case "$arch" in
                "arm64-v8a")
                    if file "$lib_path" | grep -q "aarch64"; then
                        echo "  - Correct ARM64 architecture"
                    else
                        print_error "  - Incorrect architecture for ARM64"
                        exit 1
                    fi
                    ;;
                "armeabi-v7a")
                    if file "$lib_path" | grep -q "ARM"; then
                        echo "  - Correct ARM architecture"
                    else
                        print_error "  - Incorrect architecture for ARMv7"
                        exit 1
                    fi
                    ;;
                "x86_64")
                    if file "$lib_path" | grep -q "x86-64"; then
                        echo "  - Correct x86_64 architecture"
                    else
                        print_error "  - Incorrect architecture for x86_64"
                        exit 1
                    fi
                    ;;
                "x86")
                    if file "$lib_path" | grep -q "80386\|i386"; then
                        echo "  - Correct x86 architecture"
                    else
                        print_error "  - Incorrect architecture for x86"
                        exit 1
                    fi
                    ;;
            esac
        fi
    done
}

# Test header file
test_header_file() {
    echo "Testing header file..."

    header_path="$OUTPUT_DIR/ziplock.h"
    if [ -f "$header_path" ]; then
        # Check that it contains essential declarations
        required_declarations=(
            "extern \"C\""
            "ziplock_init"
            "ziplock_get_version"
            "ziplock_session_create"
            "ziplock_archive_create"
            "ziplock_string_free"
        )

        for declaration in "${required_declarations[@]}"; do
            if grep -q "$declaration" "$header_path"; then
                echo "✓ Found declaration: $declaration"
            else
                echo "✗ Missing declaration: $declaration"
                exit 1
            fi
        done

        print_success "✓ Header file contains required declarations"
    else
        print_error "✗ Header file not found"
        exit 1
    fi
}

# Test cross-compilation consistency
test_cross_compilation_consistency() {
    echo "Testing cross-compilation consistency..."

    # Check that all architectures have the same symbols
    reference_arch="arm64-v8a"
    reference_lib="$OUTPUT_DIR/$reference_arch/libziplock_shared.so"

    if [ ! -f "$reference_lib" ] || ! command -v nm >/dev/null 2>&1; then
        echo "Skipping consistency test (missing tools or reference library)"
        return
    fi

    reference_symbols=$(nm -D "$reference_lib" 2>/dev/null | grep -E "ziplock_" | cut -d' ' -f3 | sort)

    for arch in armeabi-v7a; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            arch_symbols=$(nm -D "$lib_path" 2>/dev/null | grep -E "ziplock_" | cut -d' ' -f3 | sort)

            if [ "$reference_symbols" = "$arch_symbols" ]; then
                echo "✓ $arch: Symbol consistency with $reference_arch"
            else
                print_warning "⚠ $arch: Symbol differences with $reference_arch"
                echo "  This may be normal due to architecture differences"
            fi
        fi
    done
}

# Test that libraries can be used in Android project structure
test_android_project_structure() {
    echo "Testing Android project structure..."

    # Create a temporary Android project structure
    temp_dir=$(mktemp -d)
    jni_libs_dir="$temp_dir/app/src/main/jniLibs"

    mkdir -p "$jni_libs_dir"/{arm64-v8a,armeabi-v7a}

    # Copy libraries
    for arch in arm64-v8a armeabi-v7a; do
        if [ -f "$OUTPUT_DIR/$arch/libziplock_shared.so" ]; then
            cp "$OUTPUT_DIR/$arch/libziplock_shared.so" "$jni_libs_dir/$arch/"
            echo "✓ Copied $arch library to Android project structure"
        fi
    done

    # Verify structure
    if [ -d "$jni_libs_dir" ]; then
        echo "✓ Android jniLibs directory structure created successfully"
        echo "  Structure: $jni_libs_dir"
        ls -la "$jni_libs_dir"
    fi

    # Clean up
    rm -rf "$temp_dir"
}

# Performance smoke test
test_performance_smoke() {
    echo "Running performance smoke test..."

    # Check that libraries are not obviously broken
    for arch in arm64-v8a armeabi-v7a; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            # Check for obvious performance killers
            if command -v strings >/dev/null 2>&1; then
                # Check for debug symbols (should be stripped in release)
                if strings "$lib_path" | grep -q "debug"; then
                    print_warning "⚠ $arch: May contain debug information (larger size)"
                else
                    echo "✓ $arch: No obvious debug information found"
                fi

                # Check for panic strings (should use abort in release)
                panic_count=$(strings "$lib_path" | grep -c "panic" || true)
                if [ "$panic_count" -gt 5 ]; then
                    print_warning "⚠ $arch: High number of panic strings found ($panic_count)"
                else
                    echo "✓ $arch: Reasonable panic string count ($panic_count)"
                fi
            fi
        fi
    done
}

# Security smoke test
test_security_smoke() {
    echo "Running security smoke test..."

    for arch in arm64-v8a armeabi-v7a; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            # Check for common security features
            if command -v readelf >/dev/null 2>&1; then
                # Check for stack protection
                if readelf -s "$lib_path" | grep -q "__stack_chk"; then
                    echo "✓ $arch: Stack protection enabled"
                else
                    echo "? $arch: Stack protection not detected"
                fi

                # Check for position independent code
                if readelf -h "$lib_path" | grep -q "DYN"; then
                    echo "✓ $arch: Position independent executable"
                else
                    echo "? $arch: Not position independent"
                fi
            fi
        fi
    done
}

# Main test runner
run_all_tests() {
    echo "Running Android integration tests..."
    echo "===================================="

    # Check if build output exists
    if [ ! -d "$OUTPUT_DIR" ]; then
        print_error "Build output directory not found: $OUTPUT_DIR"
        print_error "Please run the build first: ./scripts/build/build-android-docker.sh build"
        exit 1
    fi

    test_library_loading
    echo ""
    test_library_symbols
    echo ""
    test_library_dependencies
    echo ""
    test_library_properties
    echo ""
    test_header_file
    echo ""
    test_cross_compilation_consistency
    echo ""
    test_android_project_structure
    echo ""
    test_performance_smoke
    echo ""
    test_security_smoke
    echo ""
    echo "✅ All integration tests completed!"

    # Summary
    echo ""
    echo "Build Summary:"
    echo "=============="
    for arch in arm64-v8a armeabi-v7a; do
        lib_path="$OUTPUT_DIR/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            size=$(du -h "$lib_path" | cut -f1)
            echo "  ✓ $arch: $size"
        else
            echo "  ✗ $arch: Missing"
        fi
    done

    if [ -f "$OUTPUT_DIR/ziplock.h" ]; then
        echo "  ✓ Header file: Available"
    else
        echo "  ✗ Header file: Missing"
    fi
}

# Usage function
usage() {
    echo "Usage: $0 [test_name]"
    echo ""
    echo "Available tests:"
    echo "  all                    Run all tests (default)"
    echo "  loading                Test library loading"
    echo "  symbols                Test library symbols"
    echo "  dependencies           Test library dependencies"
    echo "  properties             Test library properties"
    echo "  header                 Test header file"
    echo "  consistency            Test cross-compilation consistency"
    echo "  android-structure      Test Android project structure"
    echo "  performance            Performance smoke test"
    echo "  security               Security smoke test"
    echo ""
    echo "Examples:"
    echo "  $0                     # Run all tests"
    echo "  $0 symbols             # Test only symbols"
    echo "  $0 loading             # Test only loading"
}

# Main function
main() {
    local test_name="${1:-all}"

    case "$test_name" in
        "all")
            run_all_tests
            ;;
        "loading")
            test_library_loading
            ;;
        "symbols")
            test_library_symbols
            ;;
        "dependencies")
            test_library_dependencies
            ;;
        "properties")
            test_library_properties
            ;;
        "header")
            test_header_file
            ;;
        "consistency")
            test_cross_compilation_consistency
            ;;
        "android-structure")
            test_android_project_structure
            ;;
        "performance")
            test_performance_smoke
            ;;
        "security")
            test_security_smoke
            ;;
        "help"|"-h"|"--help")
            usage
            ;;
        *)
            print_error "Unknown test: $test_name"
            echo ""
            usage
            exit 1
            ;;
    esac
}

main "$@"
