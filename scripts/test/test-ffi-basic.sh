#!/usr/bin/env bash

# Basic FFI test to isolate the hanging issue
# This script tests just the core FFI functions without complex operations

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${BLUE}🧪 Basic FFI Test${NC}"
echo -e "${BLUE}=================${NC}"

# Build the project first
echo -e "${YELLOW}⚙️  Building ZipLock shared library...${NC}"
cd "$PROJECT_ROOT"

if cargo build --release -p ziplock-shared --features c-api; then
    echo -e "${GREEN}✅ Shared library built successfully${NC}"
else
    echo -e "${RED}❌ Failed to build shared library${NC}"
    exit 1
fi

# Create a test directory
TEST_DIR="/tmp/ziplock_basic_test_$$"
mkdir -p "$TEST_DIR"

# Set up environment
export LD_LIBRARY_PATH="$PROJECT_ROOT/target/release:${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="$PROJECT_ROOT/target/release:${DYLD_LIBRARY_PATH:-}"
export RUST_LOG="debug"

echo -e "${YELLOW}📝 Creating basic FFI test program...${NC}"

cat > "$TEST_DIR/test_basic_ffi.c" << 'EOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <unistd.h>
#include <signal.h>

// Function pointer types
typedef int (*init_func_t)(void);
typedef char* (*get_last_error_func_t)(void);
typedef void (*free_string_func_t)(char*);
typedef void (*cleanup_func_t)(void);
typedef char* (*get_version_func_t)(void);
typedef int (*test_echo_func_t)(const char*);

void timeout_handler(int sig) {
    printf("❌ TIMEOUT: Function call took too longer than 10 seconds\n");
    exit(124);
}

int main() {
    // Set up timeout handler
    signal(SIGALRM, timeout_handler);

    printf("🔧 Loading shared library...\n");
    // Load the shared library
    void* lib = dlopen("./target/release/libziplock_shared.so", RTLD_LAZY);
    if (!lib) {
        printf("❌ Failed to load shared library: %s\n", dlerror());
        return 1;
    }

    // Load function pointers
    init_func_t ziplock_hybrid_init = (init_func_t)dlsym(lib, "ziplock_hybrid_init");
    get_last_error_func_t ziplock_hybrid_get_last_error = (get_last_error_func_t)dlsym(lib, "ziplock_hybrid_get_last_error");
    free_string_func_t ziplock_hybrid_free_string = (free_string_func_t)dlsym(lib, "ziplock_hybrid_free_string");
    cleanup_func_t ziplock_hybrid_cleanup = (cleanup_func_t)dlsym(lib, "ziplock_hybrid_cleanup");
    get_version_func_t ziplock_hybrid_get_version = (get_version_func_t)dlsym(lib, "ziplock_hybrid_get_version");
    test_echo_func_t ziplock_hybrid_test_echo = (test_echo_func_t)dlsym(lib, "ziplock_hybrid_test_echo");

    if (!ziplock_hybrid_init || !ziplock_hybrid_get_last_error ||
        !ziplock_hybrid_free_string || !ziplock_hybrid_cleanup) {
        printf("❌ Failed to load required functions\n");
        dlclose(lib);
        return 1;
    }

    printf("✅ Loaded shared library successfully\n");

    // Test 1: Get version (should be immediate)
    if (ziplock_hybrid_get_version) {
        printf("🔧 Testing version function...\n");
        alarm(5);
        char* version = ziplock_hybrid_get_version();
        alarm(0);
        if (version) {
            printf("✅ Version: %s\n", version);
            ziplock_hybrid_free_string(version);
        } else {
            printf("⚠️ Version function returned NULL\n");
        }
    }

    // Test 2: Test echo (should be immediate)
    if (ziplock_hybrid_test_echo) {
        printf("🔧 Testing echo function...\n");
        alarm(5);
        int echo_result = ziplock_hybrid_test_echo("test");
        alarm(0);
        if (echo_result == 0) {
            printf("✅ Echo test passed\n");
        } else {
            printf("⚠️ Echo test failed\n");
        }
    }

    // Test 3: Initialize (this might be where it hangs)
    printf("🔧 Testing initialization...\n");
    alarm(10);
    int init_result = ziplock_hybrid_init();
    alarm(0);

    if (init_result != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to initialize: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        dlclose(lib);
        return 1;
    }
    printf("✅ FFI initialized successfully\n");

    // Test 4: Cleanup
    printf("🔧 Testing cleanup...\n");
    alarm(5);
    ziplock_hybrid_cleanup();
    alarm(0);
    printf("✅ Cleanup completed\n");

    dlclose(lib);
    printf("🎉 All basic tests passed!\n");
    return 0;
}
EOF

# Compile the test program
echo -e "${YELLOW}🔨 Compiling basic FFI test program...${NC}"
cd "$PROJECT_ROOT"

if gcc -o "$TEST_DIR/test_basic_ffi" "$TEST_DIR/test_basic_ffi.c" -ldl; then
    echo -e "${GREEN}✅ Test program compiled successfully${NC}"
else
    echo -e "${RED}❌ Failed to compile test program${NC}"
    exit 1
fi

# Run the basic FFI test
echo -e "${YELLOW}🚀 Running basic FFI test...${NC}"
cd "$PROJECT_ROOT"

if timeout 30s "$TEST_DIR/test_basic_ffi"; then
    echo -e "${GREEN}🎉 SUCCESS: Basic FFI functions work!${NC}"
    EXIT_CODE=0
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${RED}❌ TIMEOUT: Basic FFI test hangs${NC}"
    else
        echo -e "${RED}❌ FAILED: Basic FFI test failed with exit code $EXIT_CODE${NC}"
    fi
fi

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up test files...${NC}"
rm -rf "$TEST_DIR"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Basic FFI test completed successfully!${NC}"
    echo -e "${BLUE}   The issue might be in specific archive operations.${NC}"
else
    echo -e "${RED}❌ Basic FFI test failed.${NC}"
    echo -e "${RED}   The issue might be in FFI initialization itself.${NC}"
fi

exit $EXIT_CODE
