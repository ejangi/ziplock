#!/bin/bash
# ZipLock macOS Icon Set Creation Script
# Creates a proper .icns file from the SVG logo and existing PNG files

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
ASSETS_DIR="$PROJECT_ROOT/assets/icons"
MACOS_RESOURCES_DIR="$PROJECT_ROOT/packaging/macos/resources"
SVG_SOURCE="$ASSETS_DIR/ziplock-logo.svg"
OUTPUT_ICNS="$MACOS_RESOURCES_DIR/ziplock.icns"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
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
    echo "ZipLock macOS Icon Set Creation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -c, --clean             Clean up temporary files after creation"
    echo "  -k, --keep-iconset      Keep the .iconset directory after creating .icns"
    echo ""
    echo "This script creates a macOS .icns file from:"
    echo "  - SVG source: assets/icons/ziplock-logo.svg"
    echo "  - Existing PNGs: assets/icons/ziplock-icon-*.png"
    echo ""
    echo "Requirements:"
    echo "  - macOS with iconutil command"
    echo "  - rsvg-convert or Inkscape (for SVG conversion)"
    echo ""
}

# Parse command line arguments
CLEAN_TEMP=false
KEEP_ICONSET=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -c|--clean)
            CLEAN_TEMP=true
            shift
            ;;
        -k|--keep-iconset)
            KEEP_ICONSET=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            ;;
    esac
done

print_status "ZipLock macOS Icon Set Creation"
print_status "Project root: $PROJECT_ROOT"
print_status "SVG source: $SVG_SOURCE"
print_status "Output: $OUTPUT_ICNS"

# Verify prerequisites
print_status "Verifying prerequisites..."

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script requires macOS (iconutil command)"
fi

if ! command -v iconutil &> /dev/null; then
    print_error "iconutil not found. This script requires macOS."
fi

# Check for SVG conversion tool
SVG_CONVERTER=""
if command -v rsvg-convert &> /dev/null; then
    SVG_CONVERTER="rsvg-convert"
    print_status "Using rsvg-convert for SVG conversion"
elif command -v inkscape &> /dev/null; then
    SVG_CONVERTER="inkscape"
    print_status "Using Inkscape for SVG conversion"
else
    print_warning "No SVG converter found. Install librsvg or Inkscape for best results:"
    print_warning "  brew install librsvg"
    print_warning "  brew install inkscape"
    print_warning "Will attempt to use existing PNG files only."
fi

# Verify source files exist
if [[ ! -f "$SVG_SOURCE" ]]; then
    print_error "SVG source not found: $SVG_SOURCE"
fi

# Create resources directory if it doesn't exist
mkdir -p "$MACOS_RESOURCES_DIR"

# Create iconset directory
ICONSET_DIR="$MACOS_RESOURCES_DIR/ziplock.iconset"
if [[ -d "$ICONSET_DIR" ]]; then
    rm -rf "$ICONSET_DIR"
fi
mkdir -p "$ICONSET_DIR"

print_status "Creating iconset directory: $ICONSET_DIR"

# Function to convert SVG to PNG at specified size
convert_svg_to_png() {
    local size=$1
    local output_file=$2

    if [[ "$SVG_CONVERTER" == "rsvg-convert" ]]; then
        rsvg-convert -w "$size" -h "$size" "$SVG_SOURCE" -o "$output_file"
    elif [[ "$SVG_CONVERTER" == "inkscape" ]]; then
        inkscape --export-filename="$output_file" --export-width="$size" --export-height="$size" "$SVG_SOURCE" >/dev/null 2>&1
    else
        return 1
    fi
}

# Function to copy existing PNG if available, or convert from SVG
get_icon_at_size() {
    local size=$1
    local output_file=$2
    local existing_png="$ASSETS_DIR/ziplock-icon-$size.png"

    if [[ -f "$existing_png" ]]; then
        print_status "Using existing PNG: ziplock-icon-$size.png"
        cp "$existing_png" "$output_file"
        return 0
    elif [[ -n "$SVG_CONVERTER" ]]; then
        print_status "Converting SVG to ${size}x${size} PNG"
        convert_svg_to_png "$size" "$output_file"
        return 0
    else
        print_warning "Cannot create ${size}x${size} icon (no existing PNG, no SVG converter)"
        return 1
    fi
}

# Create all required icon sizes
print_status "Creating icon files..."

# Standard sizes for macOS iconsets
# Format: size:filename
ICON_SIZES=(
    "16:icon_16x16.png"
    "32:icon_16x16@2x.png"
    "32:icon_32x32.png"
    "64:icon_32x32@2x.png"
    "128:icon_128x128.png"
    "256:icon_128x128@2x.png"
    "256:icon_256x256.png"
    "512:icon_256x256@2x.png"
    "512:icon_512x512.png"
    "1024:icon_512x512@2x.png"
)

CREATED_COUNT=0
FAILED_COUNT=0

for entry in "${ICON_SIZES[@]}"; do
    IFS=':' read -r size filename <<< "$entry"
    output_path="$ICONSET_DIR/$filename"

    if get_icon_at_size "$size" "$output_path"; then
        ((CREATED_COUNT++))
    else
        ((FAILED_COUNT++))
    fi
done

print_status "Created $CREATED_COUNT icon files, $FAILED_COUNT failed"

# Verify we have at least some icons
if [[ $CREATED_COUNT -eq 0 ]]; then
    print_error "No icon files were created successfully"
fi

# Create the .icns file
print_status "Creating .icns file..."
if iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"; then
    print_success "Created macOS icon: $(basename "$OUTPUT_ICNS")"
else
    print_error "Failed to create .icns file"
fi

# Verify the .icns file was created
if [[ ! -f "$OUTPUT_ICNS" ]]; then
    print_error "Output .icns file not found: $OUTPUT_ICNS"
fi

# Get file size
ICNS_SIZE=$(du -h "$OUTPUT_ICNS" | cut -f1)
print_status "Icon file size: $ICNS_SIZE"

# Clean up iconset directory unless keeping it
if [[ "$KEEP_ICONSET" == false ]]; then
    rm -rf "$ICONSET_DIR"
    print_status "Cleaned up temporary iconset directory"
else
    print_status "Iconset directory kept at: $ICONSET_DIR"
fi

# Final verification - test the icon
print_status "Verifying icon file..."
if iconutil -V "$OUTPUT_ICNS" >/dev/null 2>&1; then
    print_success "Icon file verified successfully"
else
    print_warning "Icon verification failed (but file was created)"
fi

# Display summary
echo ""
print_success "===================="
print_success "Icon Set Created!"
print_success "===================="
echo -e "Location: $OUTPUT_ICNS"
echo -e "Size: $ICNS_SIZE"
echo -e "Icons created: $CREATED_COUNT/$((${#ICON_SIZES[@]}))"
[[ $FAILED_COUNT -gt 0 ]] && echo -e "Failed icons: $FAILED_COUNT"

echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "• The icon will be automatically used by create-app-bundle.sh"
echo "• Test the icon: qlmanage -p '$OUTPUT_ICNS'"
echo "• View icon info: iconutil -V '$OUTPUT_ICNS'"

if [[ $FAILED_COUNT -gt 0 ]]; then
    echo ""
    print_warning "Some icon sizes could not be created."
    print_warning "Install SVG conversion tools for complete iconset:"
    print_warning "  brew install librsvg"
    print_warning "  brew install inkscape"
fi

print_success "macOS iconset creation completed!"
