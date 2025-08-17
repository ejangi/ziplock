#!/bin/bash

# ZipLock Android Quick Test Build Script
# Builds and installs a debug APK for testing .7z file associations

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
cd "$ANDROID_DIR"

echo -e "${BLUE}=== ZipLock Android Quick Test Build ===${NC}"
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

# Check if we're in the right directory
if [ ! -f "$ANDROID_DIR/build.gradle" ]; then
    print_error "build.gradle not found. Android project directory not accessible."
    exit 1
fi

# Check if gradlew exists
if [ ! -f "$ANDROID_DIR/gradlew" ]; then
    print_error "gradlew not found. Android project directory not accessible."
    exit 1
fi

echo "1. Cleaning previous builds..."
./gradlew clean > /dev/null 2>&1
print_status "Previous builds cleaned"

echo ""
echo "2. Building debug APK..."
./gradlew assembleDebug
if [ $? -eq 0 ]; then
    print_status "Debug APK built successfully"
else
    print_error "Build failed. Check the output above for errors."
    exit 1
fi

echo ""
echo "3. Checking for connected devices..."
ADB_DEVICES=$(adb devices | grep -v "List of devices" | grep "device$" | wc -l)

if [ "$ADB_DEVICES" -eq 0 ]; then
    print_warning "No Android devices connected via ADB"
    print_info "Please connect your device and enable USB debugging, then run:"
    print_info "adb install app/build/outputs/apk/debug/app-debug.apk"
    exit 0
elif [ "$ADB_DEVICES" -gt 1 ]; then
    print_warning "Multiple devices connected. Please specify device with -s flag:"
    adb devices
    exit 0
fi

echo ""
echo "4. Installing APK to connected device..."
APK_PATH="app/build/outputs/apk/debug/app-debug.apk"

if [ ! -f "$APK_PATH" ]; then
    print_error "APK not found at: $APK_PATH"
    exit 1
fi

# Uninstall previous version if it exists
adb uninstall com.ziplock > /dev/null 2>&1 || true

# Install new version
adb install "$APK_PATH"
if [ $? -eq 0 ]; then
    print_status "APK installed successfully"
else
    print_error "Installation failed"
    exit 1
fi

echo ""
echo "5. Verifying intent filters..."
# Check if our app is registered for .7z files
INTENT_CHECK=$(adb shell "dumpsys package com.ziplock | grep -A 20 'Activity.*SplashActivity' | grep -c 'android.intent.action.VIEW'")
if [ "$INTENT_CHECK" -gt 0 ]; then
    print_status "Intent filters registered correctly"
else
    print_warning "Intent filters may not be registered correctly"
fi

echo ""
echo -e "${GREEN}=== Build and Installation Complete ===${NC}"
echo ""
echo "📱 Testing Instructions:"
echo ""
echo "1. Create or download a test .7z file to your device"
echo ""
echo "2. Open a file manager (Files by Google, Samsung My Files, etc.)"
echo ""
echo "3. Navigate to your .7z file and tap it"
echo ""
echo "4. Look for ZipLock in the 'Open with' dialog"
echo ""
echo "5. If ZipLock doesn't appear:"
echo "   • Try long-pressing the file and selecting 'Open with'"
echo "   • Try different file managers"
echo "   • Check that the file has .7z extension"
echo "   • Restart the device (sometimes needed for intent filters)"
echo ""
echo "🔍 Debugging:"
echo ""
echo "To check logcat for ZipLock messages:"
echo "adb logcat | grep -i ziplock"
echo ""
echo "To check intent filter registration:"
echo "adb shell dumpsys package com.ziplock | grep -A 30 'Activity.*SplashActivity'"
echo ""
echo "To manually test intent:"
echo "adb shell am start -a android.intent.action.VIEW -d 'file:///sdcard/Download/test.7z' com.ziplock/.SplashActivity"
echo ""
echo "📁 APK Location: apps/mobile/android/$APK_PATH"
echo "📱 Package Name: com.ziplock"
echo ""
print_info "If file association still doesn't work, try the troubleshooting steps in docs/technical/file-association.md"
