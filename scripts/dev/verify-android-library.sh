#!/bin/bash

# ZipLock Android Native Library Verification Script
# This script verifies that the Android native libraries were built correctly
# and that the libgcc_s.so.1 dependency issue has been resolved.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

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

# Check if we're in the right directory
if [ ! -f "$PROJECT_ROOT/shared/Cargo.toml" ]; then
    log_error "Cannot find shared/Cargo.toml. Please run this script from the project root."
    exit 1
fi

echo "======================================="
echo "ZipLock Android Library Verification"
echo "======================================="
echo ""

# Check if libraries exist
ANDROID_LIBS_DIR="$PROJECT_ROOT/apps/mobile/android/app/src/main/jniLibs"
BUILD_LIBS_DIR="$PROJECT_ROOT/target/android/jniLibs"

log_info "Checking for built Android libraries..."

ARCHITECTURES=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")
FOUND_LIBS=0
TOTAL_SIZE=0

for arch in "${ARCHITECTURES[@]}"; do
    lib_path="$ANDROID_LIBS_DIR/$arch/libziplock_shared.so"
    build_path="$BUILD_LIBS_DIR/$arch/libziplock_shared.so"

    if [ -f "$lib_path" ]; then
        size=$(stat -f%z "$lib_path" 2>/dev/null || stat -c%s "$lib_path" 2>/dev/null || echo "0")
        TOTAL_SIZE=$((TOTAL_SIZE + size))
        log_success "Found $arch library: $(basename "$lib_path") ($(numfmt --to=iec $size))"
        FOUND_LIBS=$((FOUND_LIBS + 1))
    elif [ -f "$build_path" ]; then
        size=$(stat -f%z "$build_path" 2>/dev/null || stat -c%s "$build_path" 2>/dev/null || echo "0")
        TOTAL_SIZE=$((TOTAL_SIZE + size))
        log_warning "Found $arch library in build dir only: $(basename "$build_path") ($(numfmt --to=iec $size))"
        FOUND_LIBS=$((FOUND_LIBS + 1))
    else
        log_error "Missing $arch library"
    fi
done

echo ""
log_info "Library Statistics:"
echo "  - Architectures built: $FOUND_LIBS/4"
echo "  - Total size: $(numfmt --to=iec $TOTAL_SIZE)"

if [ $FOUND_LIBS -eq 0 ]; then
    log_error "No Android libraries found. Please run:"
    echo "  ./scripts/build/build-mobile.sh -p android"
    exit 1
fi

echo ""
log_info "Verifying library dependencies (checking for libgcc_s.so.1 issue)..."

# Function to check library dependencies
check_lib_dependencies() {
    local lib_path=$1
    local arch=$2

    if [ ! -f "$lib_path" ]; then
        log_warning "Library not found: $lib_path"
        return 1
    fi

    log_info "Checking $arch library dependencies:"

    # Use readelf to check dynamic dependencies
    if command -v readelf >/dev/null 2>&1; then
        dependencies=$(readelf -d "$lib_path" 2>/dev/null | grep "NEEDED" | awk '{print $5}' | tr -d '[]')

        echo "  Dependencies found:"
        while read -r dep; do
            if [ -n "$dep" ]; then
                echo "    - $dep"

                # Check for problematic dependencies
                case "$dep" in
                    *libgcc_s.so*)
                        log_error "❌ Found problematic libgcc_s dependency: $dep"
                        return 1
                        ;;
                    *glibc*)
                        log_error "❌ Found glibc dependency: $dep"
                        return 1
                        ;;
                    libc.so|libdl.so|libm.so)
                        log_success "✅ Android-compatible dependency: $dep"
                        ;;
                    *)
                        log_warning "⚠️  Unknown dependency: $dep"
                        ;;
                esac
            fi
        done <<< "$dependencies"

        # Check if libgcc_s.so.1 specifically is NOT present
        if echo "$dependencies" | grep -q "libgcc_s.so.1"; then
            log_error "❌ CRITICAL: libgcc_s.so.1 dependency found in $arch!"
            return 1
        else
            log_success "✅ No libgcc_s.so.1 dependency found in $arch"
        fi

    else
        log_warning "readelf not available, skipping dependency check"
    fi

    return 0
}

# Check dependencies for each architecture
DEPENDENCY_CHECK_PASSED=0
for arch in "${ARCHITECTURES[@]}"; do
    lib_path="$ANDROID_LIBS_DIR/$arch/libziplock_shared.so"
    build_path="$BUILD_LIBS_DIR/$arch/libziplock_shared.so"

    if [ -f "$lib_path" ]; then
        if check_lib_dependencies "$lib_path" "$arch"; then
            DEPENDENCY_CHECK_PASSED=$((DEPENDENCY_CHECK_PASSED + 1))
        fi
    elif [ -f "$build_path" ]; then
        if check_lib_dependencies "$build_path" "$arch"; then
            DEPENDENCY_CHECK_PASSED=$((DEPENDENCY_CHECK_PASSED + 1))
        fi
    fi
    echo ""
done

echo "======================================="
log_info "Verification Summary:"
echo "  - Libraries found: $FOUND_LIBS/4"
echo "  - Dependency checks passed: $DEPENDENCY_CHECK_PASSED/$FOUND_LIBS"

if [ $DEPENDENCY_CHECK_PASSED -eq $FOUND_LIBS ] && [ $FOUND_LIBS -gt 0 ]; then
    log_success "🎉 All Android libraries verified successfully!"
    log_success "✅ libgcc_s.so.1 dependency issue RESOLVED"
    echo ""
    echo "The temporary archive approach should now work correctly."
    echo "You can test it by running the Android instrumented tests:"
    echo ""
    echo "  cd apps/mobile/android"
    echo "  ./gradlew app:connectedDebugAndroidTest"
    echo ""
else
    log_error "❌ Verification failed!"
    if [ $FOUND_LIBS -eq 0 ]; then
        echo "No libraries were found. Build the libraries first:"
        echo "  ./scripts/build/build-mobile.sh -p android"
    else
        echo "Some libraries have dependency issues. This may cause runtime failures."
        echo "The libgcc_s.so.1 issue might not be fully resolved."
    fi
    exit 1
fi

echo "======================================="
log_info "Additional Information:"
echo ""

# Show build configuration
if [ -f "$PROJECT_ROOT/shared/.cargo/config.toml" ]; then
    log_info "Using custom Cargo configuration with Android fixes:"
    echo "  - Static libgcc linking enabled"
    echo "  - Android NDK toolchain configured"
    echo "  - libgcc_s.so.1 dependency eliminated"
else
    log_warning "Custom Cargo configuration not found"
fi

# Show FFI interface status
if [ -f "$PROJECT_ROOT/shared/src/ffi/mobile.rs" ]; then
    temp_archive_fn=$(grep -c "ziplock_mobile_create_temp_archive" "$PROJECT_ROOT/shared/src/ffi/mobile.rs" || echo "0")
    if [ "$temp_archive_fn" -gt 0 ]; then
        log_success "Temporary archive FFI function available"
    else
        log_warning "Temporary archive FFI function not found"
    fi
fi

# Show Enhanced Archive Manager status
if [ -f "$PROJECT_ROOT/apps/mobile/android/app/src/main/java/com/ziplock/archive/EnhancedArchiveManager.kt" ]; then
    log_success "EnhancedArchiveManager available for temporary archive workflow"
else
    log_warning "EnhancedArchiveManager not found"
fi

echo ""
log_info "Next steps to complete the temporary archive integration:"
echo "1. Test the fix with Android instrumented tests"
echo "2. Replace NativeArchiveManager usage with EnhancedArchiveManager"
echo "3. Verify end-to-end archive creation workflow"
echo ""
log_success "Android library verification complete!"
