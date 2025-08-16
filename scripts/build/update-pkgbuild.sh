#!/bin/bash
set -euo pipefail

# Simple script to update PKGBUILD version and sha256sums
# This avoids complex shell escaping issues in GitHub Actions

if [ $# -ne 2 ]; then
    echo "Usage: $0 <version> <sha256>"
    echo "Example: $0 0.2.8 abcd1234567890..."
    exit 1
fi

VERSION="$1"
SHA256="$2"
PKGBUILD_FILE="packaging/arch/PKGBUILD"

if [ ! -f "$PKGBUILD_FILE" ]; then
    echo "ERROR: PKGBUILD not found at $PKGBUILD_FILE"
    exit 1
fi

echo "Updating PKGBUILD with version: $VERSION"
echo "Updating PKGBUILD with SHA256: $SHA256"

# Update version
sed -i "s/^pkgver=.*/pkgver=$VERSION/" "$PKGBUILD_FILE"

# Update sha256sums by removing old line and adding new one
grep -v "^sha256sums=" "$PKGBUILD_FILE" > "$PKGBUILD_FILE.tmp"
echo "sha256sums=('$SHA256')" >> "$PKGBUILD_FILE.tmp"
mv "$PKGBUILD_FILE.tmp" "$PKGBUILD_FILE"

echo "PKGBUILD updated successfully"
echo "New version line: $(grep "^pkgver=" "$PKGBUILD_FILE")"
echo "New sha256sums line: $(grep "^sha256sums=" "$PKGBUILD_FILE")"
