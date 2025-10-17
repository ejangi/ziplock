#!/usr/bin/env python3
"""
Windows Archive Diagnostic Tool for ZipLock
============================================

This script examines a ZipLock .7z archive to diagnose the Windows-specific
issue where metadata claims 1 credential but 0 are found during loading.

The script will:
1. Extract the archive without using the ZipLock library
2. Examine the metadata.yml file
3. Look for credential files in the credentials/ directory
4. Compare expected vs actual file structure
5. Provide detailed diagnostic information

Usage:
    python debug-windows-archive.py <archive_path> [password]
"""

import sys
import os
import tempfile
import shutil
import subprocess
import yaml
import json
from pathlib import Path
from typing import Dict, List, Optional, Any


def print_header(text: str):
    """Print a formatted header."""
    print(f"\n{'=' * 60}")
    print(f" {text}")
    print(f"{'=' * 60}")


def print_info(text: str):
    """Print info message."""
    print(f"ℹ️  {text}")


def print_success(text: str):
    """Print success message."""
    print(f"✅ {text}")


def print_warning(text: str):
    """Print warning message."""
    print(f"⚠️  {text}")


def print_error(text: str):
    """Print error message."""
    print(f"❌ {text}")


def find_7z_executable() -> Optional[str]:
    """Find 7z executable on the system."""
    possible_paths = [
        "7z",
        "7za",
        "7zz",
        "C:\\Program Files\\7-Zip\\7z.exe",
        "C:\\Program Files (x86)\\7-Zip\\7z.exe",
    ]

    for path in possible_paths:
        try:
            result = subprocess.run([path], capture_output=True, text=True)
            if result.returncode != 9:  # 7z returns 9 when run without args
                continue
            print_success(f"Found 7z executable: {path}")
            return path
        except (subprocess.SubprocessError, FileNotFoundError):
            continue

    return None


def extract_archive(
    archive_path: str, extract_dir: str, password: Optional[str] = None
) -> bool:
    """Extract 7z archive using 7z command line tool."""
    seven_z = find_7z_executable()
    if not seven_z:
        print_error("7z executable not found. Please install 7-Zip.")
        print_info("Download from: https://www.7-zip.org/")
        return False

    cmd = [seven_z, "x", archive_path, f"-o{extract_dir}", "-y"]
    if password:
        cmd.append(f"-p{password}")

    print_info(f"Extracting archive: {archive_path}")
    print_info(
        f"Extract command: {' '.join(cmd[:-1])} -p[PASSWORD]"
        if password
        else " ".join(cmd)
    )

    try:
        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode == 0:
            print_success("Archive extracted successfully")
            return True
        else:
            print_error(f"Failed to extract archive (exit code: {result.returncode})")
            if result.stderr:
                print_error(f"Error output: {result.stderr}")
            if result.stdout:
                print_info(f"Output: {result.stdout}")
            return False

    except subprocess.SubprocessError as e:
        print_error(f"Failed to run 7z: {e}")
        return False


def list_directory_recursive(path: Path, prefix: str = "") -> List[str]:
    """List directory contents recursively."""
    items = []
    try:
        for item in sorted(path.iterdir()):
            relative_path = item.relative_to(
                path.parent if prefix == "" else path.parent.parent
            )
            if item.is_dir():
                items.append(f"{prefix}📁 {relative_path}/")
                items.extend(list_directory_recursive(item, prefix + "  "))
            else:
                size = item.stat().st_size
                items.append(f"{prefix}📄 {relative_path} ({size} bytes)")
    except PermissionError:
        items.append(f"{prefix}❌ Permission denied")
    except Exception as e:
        items.append(f"{prefix}❌ Error: {e}")

    return items


def examine_metadata(extract_dir: Path) -> Optional[Dict[str, Any]]:
    """Examine the metadata.yml file."""
    metadata_path = extract_dir / "metadata.yml"

    if not metadata_path.exists():
        print_error("metadata.yml file not found in archive")
        return None

    print_success(f"Found metadata.yml ({metadata_path.stat().st_size} bytes)")

    try:
        with open(metadata_path, "r", encoding="utf-8") as f:
            content = f.read()
            print_info("Metadata file content:")
            print("─" * 40)
            print(content)
            print("─" * 40)

        # Parse YAML
        metadata = yaml.safe_load(content)
        print_info("Parsed metadata:")
        for key, value in metadata.items():
            print(f"  {key}: {value}")

        return metadata

    except Exception as e:
        print_error(f"Failed to read metadata.yml: {e}")
        return None


def examine_credentials(extract_dir: Path, expected_count: int) -> List[Dict[str, Any]]:
    """Examine credential files in the credentials directory."""
    credentials_dir = extract_dir / "credentials"

    if not credentials_dir.exists():
        print_error("credentials/ directory not found in archive")
        print_warning(
            f"Expected {expected_count} credentials but credentials directory is missing"
        )
        return []

    print_success(f"Found credentials/ directory")

    # List all items in credentials directory
    credential_folders = []
    for item in credentials_dir.iterdir():
        if item.is_dir():
            credential_folders.append(item)
        else:
            print_warning(f"Unexpected file in credentials directory: {item.name}")

    print_info(f"Found {len(credential_folders)} credential folders")
    for folder in credential_folders:
        print(f"  📁 {folder.name}/")

    # Examine each credential folder
    credentials = []
    for folder in credential_folders:
        record_file = folder / "record.yml"
        if record_file.exists():
            print_success(
                f"Found record.yml in {folder.name}/ ({record_file.stat().st_size} bytes)"
            )

            try:
                with open(record_file, "r", encoding="utf-8") as f:
                    content = f.read()
                    credential = yaml.safe_load(content)
                    credentials.append(credential)

                    print_info(f"Credential {folder.name}:")
                    print(f"    ID: {credential.get('id', 'MISSING')}")
                    print(f"    Name: {credential.get('name', 'MISSING')}")
                    print(f"    Type: {credential.get('credential_type', 'MISSING')}")
                    print(f"    Fields: {len(credential.get('fields', {}))}")

            except Exception as e:
                print_error(f"Failed to parse {folder.name}/record.yml: {e}")
        else:
            print_error(f"Missing record.yml in {folder.name}/")

    return credentials


def diagnose_mismatch(metadata: Dict[str, Any], credentials: List[Dict[str, Any]]):
    """Diagnose the metadata vs credential count mismatch."""
    expected_count = metadata.get("credential_count", 0)
    actual_count = len(credentials)

    print_header("DIAGNOSIS")

    print(f"Expected credentials (from metadata): {expected_count}")
    print(f"Actual credentials found: {actual_count}")

    if expected_count == actual_count:
        print_success("✅ Credential count matches - no mismatch detected")
        print_info(
            "The issue might be in the ZipLock loading logic, not the archive format"
        )
    else:
        print_error(
            f"❌ MISMATCH DETECTED: Expected {expected_count}, found {actual_count}"
        )

        if actual_count == 0 and expected_count > 0:
            print_error(
                "This is the exact issue reported: metadata claims credentials exist but none are found"
            )
            print_info("Possible causes:")
            print("  1. Credential files were not written during archive creation")
            print("  2. Credential files were written with wrong paths/names")
            print("  3. 7z compression failed to include credential files")
            print("  4. Windows path handling issues during file creation")

        elif actual_count > expected_count:
            print_warning(
                "More credentials found than expected - metadata not updated properly"
            )

        else:
            print_warning(
                f"Some credentials missing: {expected_count - actual_count} not found"
            )


def main():
    """Main function."""
    print_header("ZipLock Windows Archive Diagnostic Tool")

    if len(sys.argv) < 2:
        print("Usage: python debug-windows-archive.py <archive_path> [password]")
        sys.exit(1)

    archive_path = sys.argv[1]
    password = sys.argv[2] if len(sys.argv) > 2 else None

    # Validate archive file
    if not os.path.exists(archive_path):
        print_error(f"Archive file not found: {archive_path}")
        sys.exit(1)

    archive_size = os.path.getsize(archive_path)
    print_info(f"Archive: {archive_path}")
    print_info(f"Size: {archive_size} bytes")
    print_info(f"Password: {'[PROVIDED]' if password else '[NONE]'}")

    # Check if it's a valid 7z file
    with open(archive_path, "rb") as f:
        header = f.read(6)
        if header[:2] != b"7z":
            print_error("File does not appear to be a 7z archive")
            print_info(f"Header bytes: {header.hex()}")
            sys.exit(1)

    print_success("File appears to be a valid 7z archive")

    # Create temporary directory for extraction
    with tempfile.TemporaryDirectory(prefix="ziplock_debug_") as temp_dir:
        extract_dir = Path(temp_dir)
        print_info(f"Temporary extraction directory: {extract_dir}")

        # Extract archive
        if not extract_archive(archive_path, str(extract_dir), password):
            print_error("Failed to extract archive")
            sys.exit(1)

        # List extracted contents
        print_header("EXTRACTED CONTENTS")
        contents = list_directory_recursive(extract_dir, "")
        for item in contents:
            print(item)

        # Examine metadata
        print_header("METADATA EXAMINATION")
        metadata = examine_metadata(extract_dir)
        if not metadata:
            sys.exit(1)

        # Examine credentials
        print_header("CREDENTIAL EXAMINATION")
        expected_count = metadata.get("credential_count", 0)
        credentials = examine_credentials(extract_dir, expected_count)

        # Diagnose the issue
        diagnose_mismatch(metadata, credentials)

        # Final recommendations
        print_header("RECOMMENDATIONS")
        if metadata.get("credential_count", 0) > len(credentials):
            print("🔧 DEBUGGING STEPS:")
            print("1. Add detailed logging to DesktopFileProvider::create_archive()")
            print("2. Verify temp directory contents before 7z compression")
            print("3. Check if credential files are actually written to temp dir")
            print("4. Test with simpler paths (no nested directories)")
            print("5. Compare Linux vs Windows temp directory behavior")

            print("\n🛠️  POTENTIAL FIXES:")
            print("1. Use absolute paths when creating temp files")
            print("2. Add Windows-specific path normalization")
            print("3. Verify temp file permissions on Windows")
            print("4. Add file existence checks after each write operation")
            print("5. Use a flatter directory structure for Windows")
        else:
            print("✅ Archive structure appears correct")
            print("🔍 The issue might be in the ZipLock loading/parsing logic")


if __name__ == "__main__":
    main()
