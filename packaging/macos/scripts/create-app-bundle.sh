#!/bin/bash
# ZipLock macOS App Bundle Creation Script
# Creates a proper .app bundle for macOS distribution

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PACKAGING_DIR="$PROJECT_ROOT/packaging/macos"
APP_BUNDLE_TEMPLATE="$PACKAGING_DIR/app-bundle/ZipLock.app"

# Default values
CONFIGURATION="release"
TARGET="x86_64-apple-darwin"
OUTPUT_DIR="$PROJECT_ROOT/target/macos-package"
SIGN_APP=false
SIGNING_IDENTITY=""
NOTARIZE=false
APPLE_ID=""
APP_PASSWORD=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Function to show usage
show_usage() {
    echo "ZipLock macOS App Bundle Creation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -c, --config CONFIG     Build configuration (debug|release) [default: release]"
    echo "  -t, --target TARGET     Rust target triple [default: x86_64-apple-darwin]"
    echo "  -o, --output DIR        Output directory [default: target/macos-package]"
    echo "  -s, --sign              Sign the app bundle"
    echo "  -i, --identity ID       Code signing identity"
    echo "  -n, --notarize          Notarize the app (requires --apple-id and --app-password)"
    echo "  --apple-id EMAIL        Apple ID for notarization"
    echo "  --app-password PASS     App-specific password for notarization"
    echo "  --skip-build            Skip building, use existing binary"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Create basic app bundle"
    echo "  $0 --sign --identity \"Developer ID Application: Your Name\""
    echo "  $0 --sign --notarize --apple-id your@email.com --app-password xxxx-xxxx-xxxx-xxxx"
}

# Parse command line arguments
SKIP_BUILD=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -c|--config)
            CONFIGURATION="$2"
            shift 2
            ;;
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -s|--sign)
            SIGN_APP=true
            shift
            ;;
        -i|--identity)
            SIGNING_IDENTITY="$2"
            shift 2
            ;;
        -n|--notarize)
            NOTARIZE=true
            shift
            ;;
        --apple-id)
            APPLE_ID="$2"
            shift 2
            ;;
        --app-password)
            APP_PASSWORD="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            ;;
    esac
done

# Validate configuration
if [[ "$CONFIGURATION" != "debug" && "$CONFIGURATION" != "release" ]]; then
    print_error "Configuration must be 'debug' or 'release'"
fi

# Set build paths
BUILD_DIR="$PROJECT_ROOT/target/$TARGET/$CONFIGURATION"
BINARY_PATH="$BUILD_DIR/ziplock"
APP_BUNDLE_PATH="$OUTPUT_DIR/ZipLock.app"

print_status "ZipLock macOS App Bundle Creation"
print_status "Configuration: $CONFIGURATION"
print_status "Target: $TARGET"
print_status "Output: $OUTPUT_DIR"
print_status "Project root: $PROJECT_ROOT"
print_status "Packaging dir: $PACKAGING_DIR"
print_status "App bundle template: $APP_BUNDLE_TEMPLATE"

# Create output directory with error checking
print_status "Creating output directory: $OUTPUT_DIR"
if ! mkdir -p "$OUTPUT_DIR"; then
    print_error "Failed to create output directory: $OUTPUT_DIR"
fi

# Verify prerequisites
print_status "Verifying prerequisites..."

if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust toolchain."
fi

if ! command -v rustup &> /dev/null; then
    print_error "rustup not found. Please install Rust toolchain."
fi

# Check if target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    print_warning "Installing Rust target: $TARGET"
    rustup target add "$TARGET"
fi

# Verify signing prerequisites if signing is enabled
if [[ "$SIGN_APP" == true ]]; then
    if [[ -z "$SIGNING_IDENTITY" ]]; then
        print_error "Signing identity required when --sign is specified"
    fi

    if ! command -v codesign &> /dev/null; then
        print_error "codesign not found. Xcode command line tools required for signing."
    fi

    # Verify signing identity exists
    if ! security find-identity -v -p codesigning | grep -q "$SIGNING_IDENTITY"; then
        print_error "Signing identity not found: $SIGNING_IDENTITY"
    fi
fi

# Verify notarization prerequisites
if [[ "$NOTARIZE" == true ]]; then
    if [[ -z "$APPLE_ID" || -z "$APP_PASSWORD" ]]; then
        print_error "Apple ID and app password required for notarization"
    fi

    if ! command -v xcrun &> /dev/null; then
        print_error "xcrun not found. Xcode required for notarization."
    fi
fi

# Build the application if not skipping
if [[ "$SKIP_BUILD" == false ]]; then
    print_status "Building ZipLock for macOS..."

    cd "$PROJECT_ROOT"

    # Build shared library
    print_status "Building shared library..."
    cargo build --package ziplock-shared --target "$TARGET" --profile "$CONFIGURATION"

    # Build desktop application
    print_status "Building desktop application..."
    cargo build --package ziplock-desktop --bin ziplock --target "$TARGET" --profile "$CONFIGURATION"

    print_success "Build completed successfully!"
else
    print_warning "Skipping build (using existing binaries)..."
fi

# Verify binary exists
if [[ ! -f "$BINARY_PATH" ]]; then
    print_error "Binary not found at: $BINARY_PATH"
    print_status "Directory listing of build directory:"
    ls -la "$BUILD_DIR" || print_error "Cannot list build directory"
    exit 1
fi

# Remove existing app bundle
if [[ -d "$APP_BUNDLE_PATH" ]]; then
    print_status "Removing existing app bundle..."
    rm -rf "$APP_BUNDLE_PATH"
fi

# Copy app bundle template
print_status "Creating app bundle structure..."
print_status "Source template path: $APP_BUNDLE_TEMPLATE"
print_status "Target app bundle path: $APP_BUNDLE_PATH"

# Debug: Check if source exists and verify paths
print_status "Current working directory: $(pwd)"
print_status "Checking if source template exists..."

if [[ ! -d "$APP_BUNDLE_TEMPLATE" ]]; then
    print_error "Source app bundle template not found: $APP_BUNDLE_TEMPLATE"
    print_status "Contents of packaging directory:"
    ls -la "$PROJECT_ROOT/packaging/" 2>/dev/null || print_error "Cannot list packaging directory"
    print_status "Contents of packaging/macos directory:"
    ls -la "$PROJECT_ROOT/packaging/macos/" 2>/dev/null || print_error "Cannot list packaging/macos directory"
    print_status "Contents of packaging/macos/app-bundle directory:"
    ls -la "$PROJECT_ROOT/packaging/macos/app-bundle/" 2>/dev/null || print_error "Cannot list app-bundle directory"
    exit 1
fi

# Ensure output directory exists and is writable
if [[ ! -d "$OUTPUT_DIR" ]]; then
    print_status "Creating output directory: $OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR" || print_error "Failed to create output directory: $OUTPUT_DIR"
fi

print_status "Source template exists, copying..."
if ! cp -R "$APP_BUNDLE_TEMPLATE" "$APP_BUNDLE_PATH"; then
    print_error "Failed to copy app bundle template from $APP_BUNDLE_TEMPLATE to $APP_BUNDLE_PATH"
fi

print_status "Verifying app bundle was copied successfully..."
if [[ ! -d "$APP_BUNDLE_PATH" ]]; then
    print_error "App bundle was not created at: $APP_BUNDLE_PATH"
fi

# Copy the binary
print_status "Installing binary..."
cp "$BINARY_PATH" "$APP_BUNDLE_PATH/Contents/MacOS/ziplock"
chmod +x "$APP_BUNDLE_PATH/Contents/MacOS/ziplock"

# Process Info.plist template
print_status "Processing Info.plist..."
PLIST_TEMPLATE="$APP_BUNDLE_PATH/Contents/Info.plist.template"
PLIST_FILE="$APP_BUNDLE_PATH/Contents/Info.plist"

if [[ -f "$PLIST_TEMPLATE" ]]; then
    # Get version from Cargo.toml
    VERSION=$(grep "^version" "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
    BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    BUILD_OS=$(sw_vers -buildVersion 2>/dev/null || echo "Unknown")
    GIT_COMMIT=$(cd "$PROJECT_ROOT" && git rev-parse --short HEAD 2>/dev/null || echo "Unknown")

    # Replace template variables
    sed -e "s/{{VERSION}}/$VERSION/g" \
        -e "s/{{BUILD_DATE}}/$BUILD_DATE/g" \
        -e "s/{{BUILD_OS}}/$BUILD_OS/g" \
        -e "s/{{GIT_COMMIT}}/$GIT_COMMIT/g" \
        "$PLIST_TEMPLATE" > "$PLIST_FILE"

    # Remove template file
    rm "$PLIST_TEMPLATE"

    print_success "Info.plist created (version: $VERSION)"
else
    print_warning "Info.plist.template not found"
fi

# Create and copy icons
ICONSET_SOURCE="$PACKAGING_DIR/resources/ziplock.iconset"
ICON_SOURCE="$PACKAGING_DIR/resources/ziplock.icns"

# Create .icns from iconset if available and we're on macOS
if [[ -d "$ICONSET_SOURCE" && "$OSTYPE" == "darwin"* ]] && command -v iconutil &> /dev/null; then
    if [[ ! -f "$ICON_SOURCE" || "$ICONSET_SOURCE" -nt "$ICON_SOURCE" ]]; then
        print_status "Creating .icns file from iconset..."
        if iconutil -c icns "$ICONSET_SOURCE" -o "$ICON_SOURCE"; then
            print_success "Created .icns file from iconset"
        else
            print_warning "Failed to create .icns file from iconset"
        fi
    fi
fi

# Copy icon if it exists
if [[ -f "$ICON_SOURCE" ]]; then
    cp "$ICON_SOURCE" "$APP_BUNDLE_PATH/Contents/Resources/"
    print_success "App icon installed"
else
    print_warning "App icon not found at: $ICON_SOURCE"
    if [[ -d "$ICONSET_SOURCE" ]]; then
        print_warning "Iconset found but couldn't create .icns (requires macOS with iconutil)"
    fi
fi

# Copy additional resources
RESOURCES_DIR="$PACKAGING_DIR/resources"
if [[ -d "$RESOURCES_DIR" ]]; then
    for file in "$RESOURCES_DIR"/*; do
        if [[ -f "$file" && "$file" != *.icns ]]; then
            cp "$file" "$APP_BUNDLE_PATH/Contents/Resources/"
        fi
    done
    print_success "Additional resources copied"
fi

# Copy license and documentation
if [[ -f "$PROJECT_ROOT/LICENSE.md" ]]; then
    cp "$PROJECT_ROOT/LICENSE.md" "$APP_BUNDLE_PATH/Contents/Resources/LICENSE.txt"
fi

if [[ -f "$PROJECT_ROOT/README.md" ]]; then
    cp "$PROJECT_ROOT/README.md" "$APP_BUNDLE_PATH/Contents/Resources/README.txt"
fi

# Set proper permissions
print_status "Setting permissions..."
find "$APP_BUNDLE_PATH" -type f -exec chmod 644 {} \;
find "$APP_BUNDLE_PATH" -type d -exec chmod 755 {} \;
chmod +x "$APP_BUNDLE_PATH/Contents/MacOS/ziplock"

# Code signing
if [[ "$SIGN_APP" == true ]]; then
    print_status "Code signing app bundle..."

    # Sign the binary first
    codesign --force --verify --verbose --sign "$SIGNING_IDENTITY" \
        --options runtime \
        --entitlements "$PACKAGING_DIR/resources/entitlements.plist" \
        "$APP_BUNDLE_PATH/Contents/MacOS/ziplock" 2>/dev/null || \
    codesign --force --verify --verbose --sign "$SIGNING_IDENTITY" \
        --options runtime \
        "$APP_BUNDLE_PATH/Contents/MacOS/ziplock"

    # Sign the entire app bundle
    codesign --force --verify --verbose --sign "$SIGNING_IDENTITY" \
        --options runtime \
        "$APP_BUNDLE_PATH"

    # Verify signing
    codesign --verify --verbose=4 "$APP_BUNDLE_PATH"

    print_success "App bundle signed with: $SIGNING_IDENTITY"
fi

# Notarization
if [[ "$NOTARIZE" == true ]]; then
    print_status "Submitting for notarization..."

    # Create a ZIP for notarization
    NOTARIZE_ZIP="$OUTPUT_DIR/ZipLock-notarize.zip"
    (cd "$OUTPUT_DIR" && zip -r "$(basename "$NOTARIZE_ZIP")" "ZipLock.app")

    # Submit for notarization
    xcrun notarytool submit "$NOTARIZE_ZIP" \
        --apple-id "$APPLE_ID" \
        --password "$APP_PASSWORD" \
        --team-id "$(security find-identity -v -p codesigning | grep "$SIGNING_IDENTITY" | head -1 | sed 's/.*(\([A-Z0-9]*\)).*/\1/')" \
        --wait

    # Staple the ticket to the app
    xcrun stapler staple "$APP_BUNDLE_PATH"

    # Clean up
    rm "$NOTARIZE_ZIP"

    print_success "App bundle notarized and stapled"
fi

# Final verification
print_status "Verifying app bundle..."

# Check bundle structure
if [[ ! -f "$APP_BUNDLE_PATH/Contents/Info.plist" ]]; then
    print_error "Invalid app bundle: Info.plist missing"
fi

if [[ ! -x "$APP_BUNDLE_PATH/Contents/MacOS/ziplock" ]]; then
    print_error "Invalid app bundle: executable missing or not executable"
fi

# Test the binary
BINARY_VERSION=$("$APP_BUNDLE_PATH/Contents/MacOS/ziplock" --version 2>&1 | head -1 || echo "Failed to get version")
print_success "Binary test: $BINARY_VERSION"

# Calculate bundle size
BUNDLE_SIZE=$(du -sh "$APP_BUNDLE_PATH" | cut -f1)

# Display summary
echo ""
print_success "===================="
print_success "App Bundle Created!"
print_success "===================="
echo -e "${CYAN}Location:${NC} $APP_BUNDLE_PATH"
echo -e "${CYAN}Size:${NC} $BUNDLE_SIZE"
echo -e "${CYAN}Target:${NC} $TARGET"
echo -e "${CYAN}Configuration:${NC} $CONFIGURATION"
[[ "$SIGN_APP" == true ]] && echo -e "${CYAN}Signed:${NC} Yes ($SIGNING_IDENTITY)"
[[ "$NOTARIZE" == true ]] && echo -e "${CYAN}Notarized:${NC} Yes"

echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "• Test the app: open '$APP_BUNDLE_PATH'"
echo "• Create DMG: ./create-dmg.sh --app-bundle '$APP_BUNDLE_PATH'"
[[ "$SIGN_APP" == false ]] && echo "• For distribution: re-run with --sign"
[[ "$NOTARIZE" == false && "$SIGN_APP" == true ]] && echo "• For App Store/Gatekeeper: re-run with --notarize"

print_success "macOS app bundle creation completed!"
