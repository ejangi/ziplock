#!/bin/bash

# Windows-specific archive persistence test for ZipLock
# Tests create/save/load cycle specifically on Windows to debug the metadata mismatch issue

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

echo -e "${BLUE}🪟 Windows Archive Persistence Test${NC}"
echo -e "${BLUE}===================================${NC}"

# Detect if we're running on Windows
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    echo -e "${GREEN}✅ Running on Windows platform${NC}"
    IS_WINDOWS=true
else
    echo -e "${YELLOW}⚠️  Not running on Windows, but proceeding with test${NC}"
    IS_WINDOWS=false
fi

# Test configuration
TEST_DIR="/tmp/ziplock_windows_test_$$"
if $IS_WINDOWS; then
    TEST_DIR="/c/temp/ziplock_windows_test_$$"
fi

TEST_ARCHIVE="$TEST_DIR/test_repo.7z"
TEST_PASSWORD="WindowsTest123!"

echo -e "${BLUE}🔧 Test configuration:${NC}"
echo -e "${BLUE}  Archive: $TEST_ARCHIVE${NC}"
echo -e "${BLUE}  Password: $TEST_PASSWORD${NC}"
echo -e "${BLUE}  Platform: $OSTYPE${NC}"

# Create test directory
echo -e "${YELLOW}📁 Setting up test environment...${NC}"
mkdir -p "$TEST_DIR"

# Enable detailed logging
export RUST_LOG="debug"
export RUST_BACKTRACE="1"

echo -e "${YELLOW}📝 Creating Windows archive persistence test program...${NC}"

# Create Cargo.toml for the test
# Convert PROJECT_ROOT to Windows-compatible path for Cargo (forward slashes work)
CARGO_PROJECT_ROOT=$(echo "$PROJECT_ROOT" | sed 's|^/c|C:|')

cat > "$TEST_DIR/Cargo.toml" << EOF
[package]
name = "windows-archive-persistence-test"
version = "0.1.0"
edition = "2021"

[dependencies]
ziplock-shared = { path = "$CARGO_PROJECT_ROOT/shared" }
anyhow = "1.0"
env_logger = "0.10"
log = "0.4"
uuid = "1.0"
serde_json = "1.0"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["fileapi", "handleapi", "winnt"] }
EOF

mkdir -p "$TEST_DIR/src"

# Create the main test program
cat > "$TEST_DIR/src/main.rs" << 'EOF'
use std::collections::HashMap;
use std::path::PathBuf;
use log::{info, error, warn};
use ziplock_shared::core::{
    DesktopFileProvider, UnifiedRepositoryManager, FileOperationProvider
};
use ziplock_shared::models::{CredentialRecord, CredentialField, FieldType};

fn main() -> anyhow::Result<()> {
    // Initialize logging with detailed output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .init();

    info!("🪟 Starting Windows Archive Persistence Test");

    let test_archive = PathBuf::from(
        std::env::args().nth(1).unwrap_or_else(|| "/tmp/test_repo.7z".to_string())
    );
    let test_password = std::env::args().nth(2).unwrap_or_else(|| "WindowsTest123!".to_string());

    info!("📁 Test archive: {:?}", test_archive);
    info!("🔑 Test password: {}", test_password);
    info!("💻 Platform: {}", std::env::consts::OS);
    info!("🏗️  Architecture: {}", std::env::consts::ARCH);

    // Log Windows-specific information if on Windows
    #[cfg(windows)]
    {
        info!("🪟 Windows-specific details:");
        if let Ok(temp_dir) = std::env::var("TEMP") {
            info!("  TEMP directory: {}", temp_dir);
        }
        if let Ok(tmp_dir) = std::env::var("TMP") {
            info!("  TMP directory: {}", tmp_dir);
        }
        info!("  std::env::temp_dir(): {:?}", std::env::temp_dir());
    }

    // Phase 1: Create repository and add credential
    info!("🔧 Phase 1: Creating repository and adding credential");

    let file_provider = DesktopFileProvider::new();
    let mut manager = UnifiedRepositoryManager::new(file_provider);

    info!("Creating repository at {:?}", test_archive);
    manager.create_repository(
        &test_archive.to_string_lossy(),
        &test_password
    )?;

    info!("✅ Repository created successfully");

    // Create a test credential
    info!("Creating test credential");
    let mut credential = CredentialRecord::new(
        "Windows Test Login".to_string(),
        "login".to_string()
    );

    let mut fields = HashMap::new();
    fields.insert("username".to_string(), CredentialField {
        field_type: FieldType::Username,
        value: "windows_test_user".to_string(),
        sensitive: false,
        label: Some("Username".to_string()),
        metadata: HashMap::new(),
    });

    fields.insert("password".to_string(), CredentialField {
        field_type: FieldType::Password,
        value: "super_secret_windows_password".to_string(),
        sensitive: true,
        label: Some("Password".to_string()),
        metadata: HashMap::new(),
    });

    fields.insert("url".to_string(), CredentialField {
        field_type: FieldType::Url,
        value: "https://windows-test.example.com".to_string(),
        sensitive: false,
        label: Some("URL".to_string()),
        metadata: HashMap::new(),
    });

    credential.fields = fields;
    let credential_id = credential.id.clone();

    info!("Adding credential with ID: {}", credential_id);
    manager.add_credential(credential)?;

    info!("✅ Credential added successfully");

    // Get stats before saving
    let stats_before_save = manager.get_stats()?;
    info!("📊 Stats before save: {} credentials", stats_before_save.credential_count);

    // Phase 2: Save repository
    info!("🔧 Phase 2: Saving repository");
    manager.save_repository()?;
    info!("✅ Repository saved successfully");

    // Phase 3: Close repository
    info!("🔧 Phase 3: Closing repository");
    manager.close_repository(false)?;
    info!("✅ Repository closed");

    // Verify archive file exists and has reasonable size
    if test_archive.exists() {
        let archive_size = std::fs::metadata(&test_archive)?.len();
        info!("📦 Archive file created: {} bytes", archive_size);

        if archive_size < 100 {
            error!("❌ Archive file is suspiciously small: {} bytes", archive_size);
            return Err(anyhow::anyhow!("Archive file too small"));
        }
    } else {
        error!("❌ Archive file was not created at {:?}", test_archive);
        return Err(anyhow::anyhow!("Archive file not found"));
    }

    // Phase 4: Create new manager and load repository
    info!("🔧 Phase 4: Loading repository in new manager");

    let file_provider2 = DesktopFileProvider::new();
    let mut manager2 = UnifiedRepositoryManager::new(file_provider2);

    info!("Opening repository from {:?}", test_archive);
    match manager2.open_repository(
        &test_archive.to_string_lossy(),
        &test_password
    ) {
        Ok(()) => {
            info!("✅ Repository opened successfully");
        },
        Err(e) => {
            error!("❌ Failed to open repository: {}", e);

            // Try to extract raw archive data for debugging
            info!("🔍 Attempting to debug archive contents...");
            debug_archive_contents(&test_archive, &test_password)?;

            return Err(anyhow::anyhow!("Failed to open repository: {}", e));
        }
    }

    // Phase 5: Verify loaded data
    info!("🔧 Phase 5: Verifying loaded data");

    let stats_after_load = manager2.get_stats()?;
    info!("📊 Stats after load: {} credentials", stats_after_load.credential_count);

    let credentials = manager2.list_credentials()?;
    info!("📋 Listed {} credentials after load", credentials.len());

    if credentials.is_empty() {
        error!("❌ No credentials found after loading repository");
        error!("   This indicates the Windows archive persistence issue!");
        return Err(anyhow::anyhow!("No credentials loaded - Windows persistence bug confirmed"));
    }

    // Verify our specific credential exists
    if let Some(loaded_credential) = credentials.iter().find(|c| c.id == credential_id) {
        info!("✅ Found our credential with ID: {}", credential_id);
        info!("   Name: {}", loaded_credential.title);
        info!("   Type: {}", loaded_credential.credential_type);
        info!("   Fields: {}", loaded_credential.fields.len());

        // Verify field contents
        for (field_name, field) in &loaded_credential.fields {
            info!("   Field '{}': {} (sensitive: {})",
                  field_name,
                  if field.sensitive { "[REDACTED]" } else { &field.value },
                  field.sensitive);
        }
    } else {
        error!("❌ Our test credential was not found after loading");
        error!("   Expected ID: {}", credential_id);
        error!("   Found credentials:");
        for cred in &credentials {
            error!("     - {} ({})", cred.title, cred.id);
        }
        return Err(anyhow::anyhow!("Test credential not found after loading"));
    }

    // Phase 6: Verify integrity
    info!("🔧 Phase 6: Running integrity verification");

    match manager2.verify_integrity() {
        Ok(issues) => {
            if issues.is_empty() {
                info!("✅ Repository integrity check passed - no issues found");
            } else {
                warn!("⚠️  Repository integrity issues found:");
                for issue in &issues {
                    warn!("   - {}", issue);
                }
                return Err(anyhow::anyhow!("Repository integrity issues found"));
            }
        },
        Err(e) => {
            error!("❌ Repository integrity check failed: {}", e);
            return Err(anyhow::anyhow!("Integrity check failed: {}", e));
        }
    }

    // Phase 7: Test round-trip modification
    info!("🔧 Phase 7: Testing round-trip modification");

    let mut modified_credential = credentials[0].clone();
    modified_credential.title = "Modified Windows Test Login".to_string();

    info!("Updating credential: {}", modified_credential.id);
    manager2.update_credential(modified_credential.clone())?;
    info!("✅ Credential updated successfully");

    info!("Saving modified repository");
    manager2.save_repository()?;
    info!("✅ Modified repository saved successfully");

    // Final verification
    let final_stats = manager2.get_stats()?;
    info!("📊 Final stats: {} credentials", final_stats.credential_count);

    manager2.close_repository(false)?;

    info!("🎉 Windows Archive Persistence Test completed successfully!");
    info!("✅ All phases passed - Windows archive persistence is working correctly");

    Ok(())
}

fn debug_archive_contents(archive_path: &PathBuf, password: &str) -> anyhow::Result<()> {
    info!("🔍 Debugging archive contents at {:?}", archive_path);

    let file_provider = DesktopFileProvider::new();

    // Read raw archive data
    let archive_data = std::fs::read(archive_path)?;
    info!("📦 Archive file size: {} bytes", archive_data.len());

    // Try to extract and examine contents
    match file_provider.extract_archive(&archive_data, password) {
        Ok(file_map) => {
            info!("✅ Archive extraction successful - {} files found:", file_map.len());
            for (path, content) in &file_map {
                info!("   - {}: {} bytes", path, content.len());

                // If it's a YAML file, try to show content (truncated)
                if path.ends_with(".yml") || path.ends_with(".yaml") {
                    let content_str = String::from_utf8_lossy(content);
                    let preview = if content_str.len() > 200 {
                        format!("{}...", &content_str[..200])
                    } else {
                        content_str.to_string()
                    };
                    info!("     Content preview: {}", preview.replace('\n', "\\n"));
                }
            }

            // Check specifically for metadata and credential files
            if let Some(metadata_content) = file_map.get("metadata.yml") {
                let metadata_str = String::from_utf8_lossy(metadata_content);
                info!("🗂️  Metadata content: {}", metadata_str);
            } else {
                warn!("⚠️  No metadata.yml file found in archive");
            }

            let credential_files: Vec<_> = file_map.keys()
                .filter(|k| k.starts_with("credentials/") && k.ends_with("/record.yml"))
                .collect();
            info!("📋 Found {} credential files", credential_files.len());
            for cred_file in credential_files {
                info!("   - {}", cred_file);
            }
        },
        Err(e) => {
            error!("❌ Failed to extract archive for debugging: {}", e);

            // Try without password in case it's a password issue
            if !password.is_empty() {
                info!("🔍 Trying extraction without password...");
                match file_provider.extract_archive(&archive_data, "") {
                    Ok(file_map) => {
                        warn!("⚠️  Archive extracted without password - password issue detected!");
                        info!("   {} files found without password", file_map.len());
                    },
                    Err(e2) => {
                        error!("❌ Also failed without password: {}", e2);
                    }
                }
            }
        }
    }

    Ok(())
}
EOF

# Build the test program
echo -e "${YELLOW}🔨 Building Windows test program...${NC}"
cd "$TEST_DIR"

if cargo build --release; then
    echo -e "${GREEN}✅ Test program built successfully${NC}"
else
    echo -e "${RED}❌ Failed to build test program${NC}"
    exit 1
fi

# Run the test
echo -e "${YELLOW}🚀 Running Windows archive persistence test...${NC}"
echo -e "${BLUE}============================================${NC}"

EXIT_CODE=0

if ./target/release/windows-archive-persistence-test "$TEST_ARCHIVE" "$TEST_PASSWORD"; then
    echo -e "${BLUE}============================================${NC}"
    echo -e "${GREEN}🎉 SUCCESS: Windows archive persistence test passed!${NC}"
    echo -e "${GREEN}   Archive creation and loading work correctly on Windows${NC}"
else
    EXIT_CODE=$?
    echo -e "${BLUE}============================================${NC}"
    echo -e "${RED}❌ FAILED: Windows archive persistence test failed${NC}"
    echo -e "${RED}   Exit code: $EXIT_CODE${NC}"

    # Show detailed error information
    if [ -f "$TEST_ARCHIVE" ]; then
        echo -e "${YELLOW}📦 Archive file exists: $(ls -la "$TEST_ARCHIVE")${NC}"

        # Try to get more information about the archive
        if command -v file >/dev/null 2>&1; then
            echo -e "${YELLOW}📋 File type: $(file "$TEST_ARCHIVE")${NC}"
        fi
    else
        echo -e "${RED}📦 Archive file was not created${NC}"
    fi

    echo -e "${YELLOW}🔍 Check the logs above for detailed error information${NC}"
fi

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up test files...${NC}"
cd "$PROJECT_ROOT"
rm -rf "$TEST_DIR"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Windows archive persistence test completed successfully!${NC}"
else
    echo -e "${RED}❌ Windows archive persistence test failed - review logs for debugging${NC}"
fi

exit $EXIT_CODE
