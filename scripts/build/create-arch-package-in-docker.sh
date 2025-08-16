#!/bin/bash
set -euo pipefail

# Comprehensive Arch package creation script for Docker environment
# This script runs inside the Docker container and handles all package creation

echo "Starting Arch package creation..."

# Validate that we're in the right environment
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Not in project root (Cargo.toml not found)"
    exit 1
fi

if [ ! -f "target/install/usr/bin/ziplock" ]; then
    echo "ERROR: Required binary not found in installation structure"
    exit 1
fi

echo "DEBUG: About to execute Arch package creation..."
echo "DEBUG: Current directory: $(pwd)"
echo "DEBUG: Available files:"
ls -la target/install/usr/bin/ || echo "No binaries found"

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "Creating Arch package for version: $VERSION"

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not extract version from Cargo.toml"
    exit 1
fi

# Create source archive manually
echo "Creating source archive..."
mkdir -p target/src
rm -rf target/src/*

# Copy project files excluding build artifacts
rsync -av \
  --exclude=".git/" \
  --exclude="target/" \
  --exclude="tests/results/" \
  --exclude="*.deb" \
  --exclude="*.pkg.tar.*" \
  --exclude=".DS_Store" \
  ./ target/src/ziplock-$VERSION/

# Create source archive
cd target/src
tar -czf "../ziplock-$VERSION.tar.gz" "ziplock-$VERSION/"
cd ../..

# Calculate SHA256
SHA256=$(sha256sum "target/ziplock-$VERSION.tar.gz" | cut -d' ' -f1)
echo "Source archive SHA256: $SHA256"

if [ -z "$SHA256" ]; then
    echo "ERROR: Could not calculate SHA256"
    exit 1
fi

# Update PKGBUILD with current version and SHA256 using our script
echo "DEBUG: Updating PKGBUILD using dedicated script..."
echo "DEBUG: Version: $VERSION"
echo "DEBUG: SHA256: $SHA256"

echo "DEBUG: Original PKGBUILD sha256sums line:"
grep "^sha256sums=" packaging/arch/PKGBUILD || echo "No sha256sums line found"

# Use dedicated script to avoid all shell escaping issues
chmod +x scripts/build/update-pkgbuild.sh
./scripts/build/update-pkgbuild.sh "$VERSION" "$SHA256"

echo "DEBUG: Updated PKGBUILD sha256sums line:"
grep "^sha256sums=" packaging/arch/PKGBUILD || echo "No sha256sums line found after update"

# Generate .SRCINFO from updated PKGBUILD
echo "Generating .SRCINFO..."
cd packaging/arch
makepkg --printsrcinfo > .SRCINFO

echo "Arch package creation completed successfully"

# Show final results
echo "Created files:"
echo "- Source archive: $(ls -la ../../target/ziplock-*.tar.gz)"
echo "- PKGBUILD version: $(grep '^pkgver=' PKGBUILD)"
echo "- PKGBUILD sha256sums: $(grep '^sha256sums=' PKGBUILD)"
echo "- .SRCINFO created: $([ -f .SRCINFO ] && echo 'YES' || echo 'NO')"
