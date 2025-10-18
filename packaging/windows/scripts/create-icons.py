#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ZipLock Windows Icon Creation Script
====================================

Creates high-resolution Windows ICO files from PNG sources with multiple embedded sizes.

This script generates proper multi-resolution ICO files using a manual ICO file format
approach to ensure Windows displays crisp icons at all sizes and DPI settings.

Features:
- Converts high-resolution PNG files to ICO format
- Embeds multiple icon sizes (16px to 512px) in a single ICO file
- Optimized for Windows 10/11 and high-DPI displays
- Uses PNG data directly in ICO containers for best quality
- Creates multiple ICO variants for different Windows contexts

Usage:
    python3 create-icons.py

The script automatically finds PNG sources in assets/icons/ and outputs ICO files
to packaging/windows/resources/ which are then embedded by the build script.
"""

import os
import sys
import struct
from pathlib import Path

try:
    from PIL import Image

    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False
    Image = None


def create_ico_header(num_images):
    """Create ICO file header"""
    return struct.pack("<HHH", 0, 1, num_images)  # Reserved, Type=1 (ICO), Count


def create_ico_entry(width, height, size, offset):
    """Create an ICO directory entry"""
    # If size is >= 256, we need to use 0 in the width/height fields
    w = width if width < 256 else 0
    h = height if height < 256 else 0

    return struct.pack(
        "<BBBBHHII",
        w,  # Width (0 if >= 256)
        h,  # Height (0 if >= 256)
        0,  # Color count (0 for 24-bit+)
        0,  # Reserved
        1,  # Color planes
        32,  # Bits per pixel
        size,  # Image data size
        offset,
    )  # Offset to image data


def png_to_ico_data(png_path, sizes):
    """Convert PNG to multiple sizes and return ICO-compatible data"""
    with Image.open(png_path) as img:
        if img.mode != "RGBA":
            img = img.convert("RGBA")

        ico_images = []
        for size in sizes:
            # Resize with high quality
            resized = img.resize((size, size), Image.Resampling.LANCZOS)

            # Convert to PNG bytes (ICO can contain PNG data directly)
            from io import BytesIO

            png_buffer = BytesIO()
            resized.save(png_buffer, format="PNG")
            png_data = png_buffer.getvalue()

            ico_images.append(
                {"width": size, "height": size, "data": png_data, "size": len(png_data)}
            )

        return ico_images


def create_ico_file(png_path, ico_path, sizes=None):
    """Create an ICO file from a PNG with multiple sizes"""
    if sizes is None:
        sizes = [16, 24, 32, 48, 64, 128, 256]

    print(f"Creating {ico_path.name} from {png_path.name}...")
    print(f"  Target sizes: {', '.join(map(str, sizes))} pixels")

    try:
        # Generate image data for all sizes
        ico_images = png_to_ico_data(png_path, sizes)

        # Calculate offsets
        header_size = 6  # ICO header
        entry_size = 16  # Each directory entry
        entries_size = len(ico_images) * entry_size
        offset = header_size + entries_size

        for img in ico_images:
            img["offset"] = offset
            offset += img["size"]

        # Write ICO file
        with open(ico_path, "wb") as f:
            # Write header
            f.write(create_ico_header(len(ico_images)))

            # Write directory entries
            for img in ico_images:
                entry = create_ico_entry(
                    img["width"], img["height"], img["size"], img["offset"]
                )
                f.write(entry)

            # Write image data
            for img in ico_images:
                f.write(img["data"])

        # Verify file was created
        if ico_path.exists():
            size_kb = ico_path.stat().st_size / 1024
            print(f"  ✅ SUCCESS: Created {ico_path.name} ({size_kb:.1f} KB)")
            print(f"     Contains {len(sizes)} size variants")
            return True
        else:
            print(f"  ❌ ERROR: Failed to create {ico_path.name}")
            return False

    except Exception as e:
        print(f"  ❌ ERROR: {e}")
        return False


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Create high-resolution Windows ICO files"
    )
    parser.add_argument(
        "--install-deps",
        action="store_true",
        help="Automatically install Pillow if missing",
    )
    args = parser.parse_args()

    # Check if Pillow is available
    global PIL_AVAILABLE, Image
    if not PIL_AVAILABLE:
        if args.install_deps:
            print("Installing Pillow...")
            try:
                import subprocess

                subprocess.check_call(
                    [sys.executable, "-m", "pip", "install", "Pillow"]
                )
                from PIL import Image

                PIL_AVAILABLE = True
                print("✅ Pillow installed successfully!")
            except Exception as e:
                print(f"❌ Failed to install Pillow: {e}")
                print("Please install manually: pip install Pillow")
                return
        else:
            print("WARNING: Pillow not found. Cannot create proper ICO files.")
            print("Please install Pillow with: pip install Pillow")
            print("Or run with --install-deps to install automatically")
            print()
            print(
                "For CI/CD environments, the workflow should handle this with fallback icons."
            )
            print("Exiting gracefully - fallback method will be used.")
            return  # Exit gracefully without error

    # Determine paths
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent

    source_dir = project_root / "assets" / "icons"
    output_dir = project_root / "packaging" / "windows" / "resources"

    print("ZipLock High-Resolution ICO Creation")
    print("====================================")
    print(f"Source: {source_dir}")
    print(f"Output: {output_dir}")
    print()

    # Ensure output directory exists
    output_dir.mkdir(parents=True, exist_ok=True)

    # Icon configurations with optimized size ranges
    configs = [
        {
            "source": "ziplock-icon-512.png",
            "output": "ziplock.ico",
            "sizes": [16, 20, 24, 32, 48, 64, 96, 128, 256, 512],
            "desc": "Main application icon (full range)",
        },
        {
            "source": "ziplock-icon-256.png",
            "output": "ziplock-standard.ico",
            "sizes": [16, 20, 24, 32, 48, 64, 96, 128, 256],
            "desc": "Standard icon (up to 256px)",
        },
        {
            "source": "ziplock-icon-128.png",
            "output": "ziplock-small.ico",
            "sizes": [16, 20, 24, 32, 48, 64, 96, 128],
            "desc": "Small icon variants",
        },
        {
            "source": "ziplock-icon-256.png",
            "output": "ziplock-taskbar.ico",
            "sizes": [16, 20, 24, 32, 48, 64],
            "desc": "Taskbar optimized",
        },
    ]

    generated = 0
    failed = 0

    for config in configs:
        source_path = source_dir / config["source"]
        output_path = output_dir / config["output"]

        if not source_path.exists():
            print(f"❌ Source not found: {config['source']}")
            failed += 1
            continue

        print(f"📄 {config['desc']}:")
        if create_ico_file(source_path, output_path, config["sizes"]):
            generated += 1
        else:
            failed += 1
        print()

    print("Summary")
    print("=======")
    print(f"Generated: {generated}")
    print(f"Failed: {failed}")

    if generated > 0:
        print()
        print("✅ High-resolution ICO files created successfully!")
        print()
        print("Technical improvements achieved:")
        print("• Multi-resolution ICO files (16px to 512px) for all Windows contexts")
        print("• High-quality PNG-based icon data for crisp display")
        print("• Optimized for high-DPI displays and modern Windows versions")
        print("• Icons are automatically embedded by build.rs during compilation")
        print()
        print("Next steps:")
        print("1. Build the Windows executable:")
        print("   cargo build --release")
        print("2. Verify icon quality in Windows Explorer, taskbar, and Alt+Tab")
        print("3. Test on different DPI settings and screen resolutions")


if __name__ == "__main__":
    main()
