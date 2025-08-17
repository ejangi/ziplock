#!/bin/bash

# ZipLock Android File Association Verification Script
#
# This script verifies that the Android app is properly configured to handle .7z files
# through intent filters and provides guidance for testing file association functionality.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANDROID_DIR="$SCRIPT_DIR/../../apps/mobile/android"
MANIFEST_FILE="$ANDROID_DIR/app/src/main/AndroidManifest.xml"

echo -e "${BLUE}=== ZipLock Android File Association Verification ===${NC}"
echo ""

# Function to print status messages
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Check if AndroidManifest.xml exists
echo "1. Checking AndroidManifest.xml..."
if [ ! -f "$MANIFEST_FILE" ]; then
    print_error "AndroidManifest.xml not found at: $MANIFEST_FILE"
    exit 1
fi
print_status "AndroidManifest.xml found"

echo ""
echo "2. Verifying intent filters for .7z file association..."

# Check for required intent filters
check_intent_filter() {
    local description="$1"
    local pattern="$2"

    if grep -q "$pattern" "$MANIFEST_FILE"; then
        print_status "$description"
        return 0
    else
        print_error "$description"
        return 1
    fi
}

# Track verification results
VERIFICATION_PASSED=true

# Check for ACTION_VIEW intent filters
if ! check_intent_filter "ACTION_VIEW intent action" 'android:name="android.intent.action.VIEW"'; then
    VERIFICATION_PASSED=false
fi

# Check for DEFAULT category
if ! check_intent_filter "DEFAULT category" 'android:name="android.intent.category.DEFAULT"'; then
    VERIFICATION_PASSED=false
fi

# Check for BROWSABLE category (optional for file associations)
if check_intent_filter "BROWSABLE category (optional)" 'android:name="android.intent.category.BROWSABLE"'; then
    print_info "BROWSABLE category found (good for web integration)"
else
    print_info "BROWSABLE category not found (not required for file associations)"
fi

# Check for 7z MIME type
if ! check_intent_filter "7z MIME type (application/x-7z-compressed)" 'android:mimeType="application/x-7z-compressed"'; then
    VERIFICATION_PASSED=false
fi

# Check for octet-stream MIME type with .7z pattern
if ! check_intent_filter "Generic MIME type with .7z pattern" 'android:mimeType="application/octet-stream"'; then
    VERIFICATION_PASSED=false
fi

# Check for file scheme
if ! check_intent_filter "File scheme support" 'android:scheme="file"'; then
    VERIFICATION_PASSED=false
fi

# Check for content scheme
if ! check_intent_filter "Content scheme support" 'android:scheme="content"'; then
    VERIFICATION_PASSED=false
fi

# Check for .7z path pattern
if ! check_intent_filter ".7z path pattern" '\.7z'; then
    VERIFICATION_PASSED=false
fi

echo ""
echo "3. Checking source code integration..."

# Check SplashActivity for intent handling
SPLASH_ACTIVITY="$ANDROID_DIR/app/src/main/java/com/ziplock/SplashActivity.kt"
if [ -f "$SPLASH_ACTIVITY" ]; then
    if grep -q "Intent.ACTION_VIEW" "$SPLASH_ACTIVITY" && grep -q "file_uri" "$SPLASH_ACTIVITY"; then
        print_status "SplashActivity handles incoming .7z file intents"
    else
        print_error "SplashActivity missing file intent handling"
        VERIFICATION_PASSED=false
    fi
else
    print_error "SplashActivity.kt not found"
    VERIFICATION_PASSED=false
fi

# Check MainActivity for parameter handling
MAIN_ACTIVITY="$ANDROID_DIR/app/src/main/java/com/ziplock/MainActivity.kt"
if [ -f "$MAIN_ACTIVITY" ]; then
    if grep -q "file_uri" "$MAIN_ACTIVITY" && grep -q "opened_from_file" "$MAIN_ACTIVITY"; then
        print_status "MainActivity processes file URI parameters"
    else
        print_error "MainActivity missing file URI parameter handling"
        VERIFICATION_PASSED=false
    fi
else
    print_error "MainActivity.kt not found"
    VERIFICATION_PASSED=false
fi

echo ""
echo "4. Checking test coverage..."

# Check for file association tests
TEST_FILE="$ANDROID_DIR/app/src/test/java/com/ziplock/FileAssociationTest.kt"
if [ -f "$TEST_FILE" ]; then
    print_status "File association tests found"
else
    print_warning "File association tests not found (recommended)"
fi

echo ""
echo "5. Verification Summary"
echo "========================"

if [ "$VERIFICATION_PASSED" = true ]; then
    print_status "All file association components verified successfully!"
    echo ""
    echo -e "${GREEN}Your ZipLock Android app is properly configured to handle .7z files.${NC}"
else
    print_error "Some file association components failed verification."
    echo ""
    echo -e "${RED}Please review and fix the issues above before testing.${NC}"
    exit 1
fi

echo ""
echo "6. Manual Testing Instructions"
echo "=============================="
echo ""
echo "To test .7z file association manually:"
echo ""
echo "1. Build and install the app:"
echo "   cd ../../apps/mobile/android"
echo "   ./gradlew assembleDebug"
echo "   adb install app/build/outputs/apk/debug/app-debug.apk"
echo ""
echo "2. Create or download a test .7z file to your device"
echo ""
echo "3. Open a file manager (Files by Google, Samsung My Files, etc.)"
echo ""
echo "4. Navigate to your .7z file and tap it"
echo ""
echo "5. Verify ZipLock appears in the 'Open with' dialog"
echo ""
echo "6. Select ZipLock and verify it opens to the repository selection screen"
echo ""
echo "7. Confirm the .7z file path is pre-filled correctly"
echo ""

echo "Additional test scenarios:"
echo "• Test with files from Google Drive, Dropbox, or OneDrive"
echo "• Test with email attachments (.7z files)"
echo "• Test with browser downloads"
echo "• Test with files shared from other apps"

echo ""
echo "For cloud storage testing:"
echo "• Upload a .7z file to Google Drive"
echo "• Open Google Drive app and tap the .7z file"
echo "• Verify ZipLock appears as an option"
echo "• Test opening and verify cloud storage handling works"

echo ""
echo "Troubleshooting:"
echo "• If ZipLock doesn't appear, try reinstalling the app"
echo "• Check that the .7z file has the correct extension"
echo "• Verify Android version compatibility (API 24+)"
echo "• Check logcat for any intent filter registration issues"

echo ""
print_info "For complete documentation, see: docs/technical/file-association.md"
print_info "For Android development setup, see: docs/technical/android.md"

echo ""
echo -e "${GREEN}File association verification completed successfully!${NC}"
