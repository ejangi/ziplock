#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ZipLock Windows Icon Generation Script
Converts PNG assets to .ico format for Windows executable embedding

This script uses PIL (Pillow) to convert PNG files to Windows .ico format
with multiple icon sizes embedded in a single .ico file.
"""

import os
import sys
import argparse
from pathlib import Path

# Set console encoding to UTF-8 for Windows
if sys.platform == "win32":
    import codecs

    sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())
    sys.stderr = codecs.getwriter("utf-8")(sys.stderr.detach())

try:
    from PIL import Image

    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False
    Image = None


def install_pillow():
    """Attempt to install Pillow if not available."""
    print("Pillow (PIL) not found. Attempting to install...")
    try:
        import subprocess

        result = subprocess.run(
            [sys.executable, "-m", "pip", "install", "Pillow"],
            capture_output=True,
            text=True,
            check=True,
        )
        print("SUCCESS: Pillow installed successfully!")
        print(result.stdout)
        return True
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Failed to install Pillow: {e}")
        print(f"Error output: {e.stderr}")
        return False
    except Exception as e:
        print(f"ERROR: Unexpected error installing Pillow: {e}")
        return False


def create_ico_from_png(png_path, ico_path, sizes=None):
    """
    Create a .ico file from a PNG with multiple sizes.

    Args:
        png_path (Path): Source PNG file
        ico_path (Path): Output ICO file
        sizes (list): List of sizes to include (default: [16, 32, 48, 64, 128, 256])
    """
    if sizes is None:
        sizes = [16, 32, 48, 64, 128, 256]

    print(f"Creating {ico_path.name} from {png_path.name}...")

    try:
        # Load the source PNG
        with Image.open(png_path) as img:
            # Convert to RGBA if not already
            if img.mode != "RGBA":
                img = img.convert("RGBA")

            # Create list of resized images
            icon_images = []
            for size in sizes:
                print(f"   Generating {size}x{size}...")
                resized = img.resize((size, size), Image.Resampling.LANCZOS)
                icon_images.append(resized)

            # Save as ICO with all sizes
            print(f"   Saving .ico file...")
            icon_images[0].save(
                ico_path,
                format="ICO",
                sizes=[(size, size) for size in sizes],
                append_images=icon_images[1:],
            )

            # Verify the file was created
            if ico_path.exists():
                size_kb = ico_path.stat().st_size / 1024
                print(f"   SUCCESS: Created: {ico_path.name} ({size_kb:.1f} KB)")
                return True
            else:
                print(f"   ERROR: Failed to create {ico_path.name}")
                return False

    except Exception as e:
        print(f"   ERROR: Error processing {png_path.name}: {e}")
        return False


def main():
    """Main execution function."""
    parser = argparse.ArgumentParser(
        description="Convert ZipLock PNG icons to Windows ICO format"
    )
    parser.add_argument(
        "--source-dir", type=str, help="Source directory containing PNG files"
    )
    parser.add_argument("--output-dir", type=str, help="Output directory for ICO files")
    parser.add_argument(
        "--force", action="store_true", help="Overwrite existing ICO files"
    )

    args = parser.parse_args()

    # Determine paths
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent

    source_dir = (
        Path(args.source_dir) if args.source_dir else project_root / "assets" / "icons"
    )
    output_dir = (
        Path(args.output_dir)
        if args.output_dir
        else project_root / "packaging" / "windows" / "resources"
    )

    print("ZipLock Windows Icon Generation")
    print("=" * 31)
    print(f"📂 Source Directory: {source_dir}")
    print(f"📁 Output Directory: {output_dir}")
    print(f"Force Overwrite: {args.force}")
    print()

    # Check if Pillow is available
    global PIL_AVAILABLE
    if not PIL_AVAILABLE:
        if not install_pillow():
            print("ERROR: Cannot proceed without Pillow. Please install it manually:")
            print("   pip install Pillow")
            sys.exit(1)
        else:
            # Re-import after installation
            try:
                from PIL import Image

                PIL_AVAILABLE = True
                globals()["Image"] = Image
            except ImportError:
                print("ERROR: Pillow installation succeeded but import still fails")
                sys.exit(1)

    # Verify source directory exists
    if not source_dir.exists():
        print(f"ERROR: Source directory not found: {source_dir}")
        sys.exit(1)

    # Create output directory if needed
    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"Output directory ready: {output_dir}")
    print()

    # Icon generation configurations
    icon_configs = [
        {
            "source": "ziplock-icon-256.png",
            "output": "ziplock.ico",
            "sizes": [16, 32, 48, 64, 128, 256],
            "description": "Main application icon",
        },
        {
            "source": "ziplock-icon-128.png",
            "output": "ziplock-small.ico",
            "sizes": [16, 32, 48, 64, 128],
            "description": "Small application icon (fallback)",
        },
        {
            "source": "ziplock-icon-512.png",
            "output": "ziplock-large.ico",
            "sizes": [16, 32, 48, 64, 128, 256],
            "description": "Large application icon (high-res displays)",
        },
    ]

    print("Generating ICO files...")
    print()

    generated_count = 0
    failed_count = 0

    for config in icon_configs:
        source_path = source_dir / config["source"]
        output_path = output_dir / config["output"]

        if not source_path.exists():
            print(f"WARNING: Source file not found: {source_path.name}")
            failed_count += 1
            continue

        # Check if output already exists and force is not specified
        if output_path.exists() and not args.force:
            print(
                f"Skipping {config['output']} (already exists, use --force to overwrite)"
            )
            continue

        print(f"{config['description']}:")
        if create_ico_from_png(source_path, output_path, config["sizes"]):
            generated_count += 1
        else:
            failed_count += 1
        print()

    # Summary
    print("Icon Generation Summary")
    print("=" * 23)
    print(f"Generated: {generated_count} files")
    print(f"Failed: {failed_count} files")
    print(f"Output directory: {output_dir}")
    print()

    # List generated files
    ico_files = list(output_dir.glob("*.ico"))
    if ico_files:
        print("Generated ICO files:")
        for ico_file in sorted(ico_files):
            size_kb = ico_file.stat().st_size / 1024
            print(f"   - {ico_file.name} ({size_kb:.1f} KB)")
    print()

    if generated_count > 0:
        print("Next steps:")
        print("   1. The build.rs script will automatically embed these icons")
        print("   2. Update WiX installer to reference the .ico files")
        print("   3. Build the Windows executable to test icon embedding")
        print()
        print("Icon generation completed successfully!")
    else:
        print("WARNING: No icons were generated. Check source files and try again.")
        if failed_count > 0:
            sys.exit(1)


if __name__ == "__main__":
    main()
