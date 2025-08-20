#!/usr/bin/env bash

# Direct ArchiveManager test to isolate the hanging issue
# This script tests the ArchiveManager directly without FFI to see if the issue is in the archive operations

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

echo -e "${BLUE}🧪 Direct ArchiveManager Test${NC}"
echo -e "${BLUE}=============================${NC}"

# Create a test directory
TEST_DIR="/tmp/ziplock_archive_test_$$"
TEST_ARCHIVE="$TEST_DIR/test_archive.7z"
TEST_PASSWORD="test123"

echo -e "${YELLOW}📁 Setting up test environment...${NC}"
mkdir -p "$TEST_DIR"

echo -e "${BLUE}🔧 Test archive: $TEST_ARCHIVE${NC}"
echo -e "${BLUE}🔧 Test password: $TEST_PASSWORD${NC}"

# Set up environment
export RUST_LOG="debug"

echo -e "${YELLOW}📝 Creating ArchiveManager test program...${NC}"

cat > "$TEST_DIR/Cargo.toml" << EOF
[package]
name = "archive-manager-test"
version = "0.1.0"
edition = "2021"

[dependencies]
ziplock-shared = { path = "${PROJECT_ROOT}/shared" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
EOF

mkdir -p "$TEST_DIR/src"

cat > "$TEST_DIR/src/main.rs" << 'EOF'
use std::path::PathBuf;
use std::collections::HashMap;
use ziplock_shared::archive::{ArchiveManager, ArchiveConfig};
use ziplock_shared::models::{CredentialRecord, CredentialField, FieldType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔧 Starting ArchiveManager test...");

    let test_archive = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "/tmp/test_archive.7z".to_string()));
    let test_password = "test123".to_string();

    println!("📁 Test archive: {:?}", test_archive);
    println!("🔑 Test password: {}", test_password);

    // Test 1: Create ArchiveManager
    println!("🔧 Creating ArchiveManager...");
    let config = ArchiveConfig::default();
    let manager = match ArchiveManager::new(config) {
        Ok(m) => {
            println!("✅ ArchiveManager created successfully");
            m
        },
        Err(e) => {
            println!("❌ Failed to create ArchiveManager: {}", e);
            return Err(e.into());
        }
    };

    // Test 2: Create archive (this might hang)
    println!("🔧 Creating archive (timeout in 30s)...");
    let create_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        manager.create_archive(test_archive.clone(), test_password.clone())
    ).await;

    match create_result {
        Ok(Ok(())) => {
            println!("✅ Archive created successfully");
        },
        Ok(Err(e)) => {
            println!("❌ Failed to create archive: {}", e);
            return Err(e.into());
        },
        Err(_) => {
            println!("❌ TIMEOUT: Archive creation timed out after 30 seconds");
            return Err(anyhow::anyhow!("Archive creation timeout"));
        }
    }

    // Test 3: Open archive
    println!("🔧 Opening archive (timeout in 30s)...");
    let open_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        manager.open_archive(test_archive.clone(), test_password.clone())
    ).await;

    match open_result {
        Ok(Ok(())) => {
            println!("✅ Archive opened successfully");
        },
        Ok(Err(e)) => {
            println!("❌ Failed to open archive: {}", e);
            return Err(e.into());
        },
        Err(_) => {
            println!("❌ TIMEOUT: Archive opening timed out after 30 seconds");
            return Err(anyhow::anyhow!("Archive opening timeout"));
        }
    }

    // Test 4: Create a test credential
    println!("🔧 Creating test credential...");
    let mut credential = CredentialRecord::new("Test Login".to_string(), "login".to_string());

    let mut fields = HashMap::new();
    fields.insert("username".to_string(), CredentialField {
        field_type: FieldType::Username,
        value: "testuser".to_string(),
        sensitive: false,
        label: Some("Username".to_string()),
        metadata: HashMap::new(),
    });

    fields.insert("password".to_string(), CredentialField {
        field_type: FieldType::Password,
        value: "testpass123".to_string(),
        sensitive: true,
        label: Some("Password".to_string()),
        metadata: HashMap::new(),
    });

    credential.fields = fields;

    // Test 5: Add credential to archive
    println!("🔧 Adding credential to archive (timeout in 30s)...");
    let add_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        manager.add_credential(credential.clone())
    ).await;

    match add_result {
        Ok(Ok(id)) => {
            println!("✅ Credential added successfully with ID: {}", id);
        },
        Ok(Err(e)) => {
            println!("❌ Failed to add credential: {}", e);
            return Err(e.into());
        },
        Err(_) => {
            println!("❌ TIMEOUT: Adding credential timed out after 30 seconds");
            return Err(anyhow::anyhow!("Add credential timeout"));
        }
    }

    // Test 6: Save archive (this is the critical test!)
    println!("🔧 Saving archive (timeout in 30s)...");
    let save_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        manager.save_archive()
    ).await;

    match save_result {
        Ok(Ok(())) => {
            println!("✅ Archive saved successfully! The ArchiveManager itself works.");
        },
        Ok(Err(e)) => {
            println!("❌ Failed to save archive: {}", e);
            return Err(e.into());
        },
        Err(_) => {
            println!("❌ TIMEOUT: Archive saving timed out after 30 seconds");
            println!("❌ This indicates the ArchiveManager itself has the hanging issue");
            return Err(anyhow::anyhow!("Archive saving timeout"));
        }
    }

    // Test 7: List credentials
    println!("🔧 Listing credentials...");
    let credentials = manager.list_credentials().await?;
    println!("✅ Found {} credentials in archive", credentials.len());

    println!("🎉 All ArchiveManager tests passed!");
    println!("✅ The issue is NOT in the ArchiveManager - it's in the FFI layer");

    Ok(())
}
EOF

# Build and run the test program
echo -e "${YELLOW}🔨 Building ArchiveManager test program...${NC}"
cd "$TEST_DIR"

if cargo build --release; then
    echo -e "${GREEN}✅ Test program built successfully${NC}"
else
    echo -e "${RED}❌ Failed to build test program${NC}"
    exit 1
fi

# Run the ArchiveManager test
echo -e "${YELLOW}🚀 Running ArchiveManager test...${NC}"

if timeout 120s ./target/release/archive-manager-test "$TEST_ARCHIVE"; then
    echo -e "${GREEN}🎉 SUCCESS: ArchiveManager works fine!${NC}"
    echo -e "${GREEN}   The hanging issue is in the FFI layer, not the ArchiveManager.${NC}"
    EXIT_CODE=0
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${RED}❌ TIMEOUT: ArchiveManager operations hang${NC}"
        echo -e "${RED}   The issue is in the ArchiveManager itself.${NC}"
    else
        echo -e "${RED}❌ FAILED: ArchiveManager test failed with exit code $EXIT_CODE${NC}"
    fi
fi

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up test files...${NC}"
cd "$PROJECT_ROOT"
rm -rf "$TEST_DIR"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ ArchiveManager test completed successfully!${NC}"
    echo -e "${BLUE}   Next: Focus on fixing the FFI thread/runtime issues.${NC}"
else
    echo -e "${RED}❌ ArchiveManager test failed.${NC}"
    echo -e "${RED}   The issue might be in the core archive operations.${NC}"
fi

exit $EXIT_CODE
