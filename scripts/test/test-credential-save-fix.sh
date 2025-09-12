#!/usr/bin/env bash

# Test script to verify the credential saving fix
# This script tests that credentials can be saved without hanging

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

echo -e "${BLUE}🧪 Testing Credential Save Fix${NC}"
echo -e "${BLUE}===============================${NC}"

# Build the project first
echo -e "${YELLOW}⚙️  Building ZipLock shared library...${NC}"
cd "$PROJECT_ROOT"

if cargo build --release -p ziplock-shared --features c-api; then
    echo -e "${GREEN}✅ Shared library built successfully${NC}"
else
    echo -e "${RED}❌ Failed to build shared library${NC}"
    exit 1
fi

echo -e "${YELLOW}⚙️  Building ZipLock Linux app...${NC}"
if cargo build --release --bin ziplock --manifest-path apps/desktop/Cargo.toml; then
    echo -e "${GREEN}✅ Linux app built successfully${NC}"
else
    echo -e "${RED}❌ Failed to build Linux app${NC}"
    exit 1
fi

# Create a test archive for the test
TEST_DIR="/tmp/ziplock_test_$$"
TEST_ARCHIVE="$TEST_DIR/test_archive.7z"
TEST_PASSWORD="test123"

echo -e "${YELLOW}📁 Setting up test environment...${NC}"
mkdir -p "$TEST_DIR"

# Set up environment
export LD_LIBRARY_PATH="$PROJECT_ROOT/target/release:${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="$PROJECT_ROOT/target/release:${DYLD_LIBRARY_PATH:-}"
export RUST_LOG="debug"

echo -e "${BLUE}🔧 Test archive: $TEST_ARCHIVE${NC}"
echo -e "${BLUE}🔧 Test password: $TEST_PASSWORD${NC}"

# Test FFI functions directly using a simple test program
echo -e "${YELLOW}📝 Creating FFI test program...${NC}"

cat > "$TEST_DIR/test_ffi.c" << 'EOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <unistd.h>

// Function pointer types
typedef int (*init_func_t)(void);
typedef int (*create_archive_func_t)(const char*, const char*);
typedef int (*open_archive_func_t)(const char*, const char*);
typedef int (*create_credential_func_t)(const char*, const char*);
typedef int (*add_field_func_t)(int, const char*, const char*, int, int);
typedef int (*save_archive_func_t)(void);
typedef char* (*get_last_error_func_t)(void);
typedef void (*free_string_func_t)(char*);
typedef void (*cleanup_func_t)(void);

int main() {
    // Load the shared library
    void* lib = dlopen("./target/release/libziplock_shared.so", RTLD_LAZY);
    if (!lib) {
        printf("❌ Failed to load shared library: %s\n", dlerror());
        return 1;
    }

    // Load function pointers
    init_func_t ziplock_hybrid_init = (init_func_t)dlsym(lib, "ziplock_hybrid_init");
    create_archive_func_t ziplock_hybrid_create_archive = (create_archive_func_t)dlsym(lib, "ziplock_hybrid_create_archive");
    open_archive_func_t ziplock_hybrid_open_archive = (open_archive_func_t)dlsym(lib, "ziplock_hybrid_open_archive");
    create_credential_func_t ziplock_hybrid_credential_create = (create_credential_func_t)dlsym(lib, "ziplock_hybrid_credential_create");
    add_field_func_t ziplock_hybrid_credential_add_field = (add_field_func_t)dlsym(lib, "ziplock_hybrid_credential_add_field");
    save_archive_func_t ziplock_hybrid_save_archive = (save_archive_func_t)dlsym(lib, "ziplock_hybrid_save_archive");
    get_last_error_func_t ziplock_hybrid_get_last_error = (get_last_error_func_t)dlsym(lib, "ziplock_hybrid_get_last_error");
    free_string_func_t ziplock_hybrid_free_string = (free_string_func_t)dlsym(lib, "ziplock_hybrid_free_string");
    cleanup_func_t ziplock_hybrid_cleanup = (cleanup_func_t)dlsym(lib, "ziplock_hybrid_cleanup");

    if (!ziplock_hybrid_init || !ziplock_hybrid_create_archive || !ziplock_hybrid_open_archive ||
        !ziplock_hybrid_credential_create || !ziplock_hybrid_credential_add_field ||
        !ziplock_hybrid_save_archive || !ziplock_hybrid_get_last_error ||
        !ziplock_hybrid_free_string || !ziplock_hybrid_cleanup) {
        printf("❌ Failed to load required functions\n");
        dlclose(lib);
        return 1;
    }

    printf("✅ Loaded shared library successfully\n");

    // Test 1: Initialize
    printf("🔧 Initializing FFI...\n");
    if (ziplock_hybrid_init() != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to initialize: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        dlclose(lib);
        return 1;
    }
    printf("✅ FFI initialized successfully\n");

    // Test 2: Create archive
    printf("🔧 Creating test archive...\n");
    if (ziplock_hybrid_create_archive("/tmp/ziplock_test_ffi/test_archive.7z", "test123") != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to create archive: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        ziplock_hybrid_cleanup();
        dlclose(lib);
        return 1;
    }
    printf("✅ Archive created successfully\n");

    // Test 3: Open archive
    printf("🔧 Opening test archive...\n");
    if (ziplock_hybrid_open_archive("/tmp/ziplock_test_ffi/test_archive.7z", "test123") != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to open archive: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        ziplock_hybrid_cleanup();
        dlclose(lib);
        return 1;
    }
    printf("✅ Archive opened successfully\n");

    // Test 4: Create credential
    printf("🔧 Creating test credential...\n");
    int credential_id = ziplock_hybrid_credential_create("Test Login", "login");
    if (credential_id == 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to create credential: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        ziplock_hybrid_cleanup();
        dlclose(lib);
        return 1;
    }
    printf("✅ Credential created with ID: %d\n", credential_id);

    // Test 5: Add fields to credential
    printf("🔧 Adding fields to credential...\n");
    if (ziplock_hybrid_credential_add_field(credential_id, "username", "testuser", 4, 0) != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to add username field: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        ziplock_hybrid_cleanup();
        dlclose(lib);
        return 1;
    }

    if (ziplock_hybrid_credential_add_field(credential_id, "password", "testpass123", 1, 1) != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to add password field: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        ziplock_hybrid_cleanup();
        dlclose(lib);
        return 1;
    }
    printf("✅ Fields added successfully\n");

    // Test 6: Save archive (This is the critical test!)
    printf("🔧 Saving archive (testing the fix)...\n");

    // Set alarm for timeout
    alarm(35); // 35 seconds timeout (function has 30s internal timeout)

    int save_result = ziplock_hybrid_save_archive();

    alarm(0); // Cancel alarm

    if (save_result != 0) {
        char* error = ziplock_hybrid_get_last_error();
        printf("❌ Failed to save archive: %s\n", error ? error : "Unknown error");
        if (error) ziplock_hybrid_free_string(error);
        ziplock_hybrid_cleanup();
        dlclose(lib);
        return 1;
    }
    printf("✅ Archive saved successfully! Fix appears to work!\n");

    // Cleanup
    printf("🔧 Cleaning up...\n");
    ziplock_hybrid_cleanup();
    dlclose(lib);

    printf("🎉 All tests passed! Credential saving fix is working!\n");
    return 0;
}
EOF

# Compile the test program
echo -e "${YELLOW}🔨 Compiling FFI test program...${NC}"
cd "$PROJECT_ROOT"

if gcc -o "$TEST_DIR/test_ffi" "$TEST_DIR/test_ffi.c" -ldl; then
    echo -e "${GREEN}✅ Test program compiled successfully${NC}"
else
    echo -e "${RED}❌ Failed to compile test program${NC}"
    exit 1
fi

# Create test directory for FFI test
mkdir -p "/tmp/ziplock_test_ffi"

# Run the FFI test
echo -e "${YELLOW}🚀 Running FFI test (this tests the credential save fix directly)...${NC}"
cd "$PROJECT_ROOT"

if timeout 45s "$TEST_DIR/test_ffi"; then
    echo -e "${GREEN}🎉 SUCCESS: Credential saving fix is working!${NC}"
    echo -e "${GREEN}   The save operation completed without hanging.${NC}"
    EXIT_CODE=0
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${RED}❌ TIMEOUT: Credential saving still hangs (fix didn't work)${NC}"
    else
        echo -e "${RED}❌ FAILED: Test failed with exit code $EXIT_CODE${NC}"
    fi
fi

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up test files...${NC}"
rm -rf "$TEST_DIR" "/tmp/ziplock_test_ffi"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Credential save fix verification completed successfully!${NC}"
    echo -e "${GREEN}   You can now run the Linux app and save credentials without hanging.${NC}"
else
    echo -e "${RED}❌ Credential save fix verification failed.${NC}"
    echo -e "${RED}   The hanging issue may still exist.${NC}"
fi

exit $EXIT_CODE
