# Windows Packaging

This directory contains all the resources needed to build Windows packages for ZipLock Password Manager.

## Overview

ZipLock supports two Windows MSI installer configurations:

- **Enhanced MSI** (recommended) - Includes user feedback dialogs and better installation experience
- **Minimal MSI** (fallback) - Basic installer without custom actions

## Directory Structure

```
packaging/windows/
├── installer/              # WiX installer configurations
│   ├── ziplock-enhanced.wxs # Enhanced MSI with user feedback
│   ├── ziplock-minimal.wxs  # Basic MSI installer
│   ├── ziplock.wxs         # Legacy full configuration
│   └── license*.rtf        # License files for installer
├── resources/              # Icons and assets
│   ├── ziplock.ico         # Main application icon
│   ├── ziplock-small.ico   # Small icon variant
│   └── ziplock-large.ico   # Large icon variant
├── scripts/                # Build and utility scripts
│   ├── build-windows-enhanced.ps1    # Enhanced build script
│   ├── build-windows-simple.ps1      # Simple build script
│   ├── show-install-success.ps1      # Success dialog custom action
│   ├── show-install-failure.ps1      # Failure dialog custom action
│   ├── test-msi-installer.ps1        # MSI testing script
│   └── create-icons.py               # Icon generation script
└── README.md               # This file
```

## Quick Start

### Build Enhanced MSI (Recommended)

```powershell
# Build with user feedback dialogs
.\packaging\windows\scripts\build-windows-enhanced.ps1 -Version "1.0.0"

# This creates: target\ZipLock-1.0.0-x64-enhanced.msi
```

### Build Minimal MSI (Fallback)

```powershell
# Build basic installer without dialogs
.\packaging\windows\scripts\build-windows-enhanced.ps1 -Version "1.0.0" -UseMinimal

# This creates: target\ZipLock-1.0.0-x64-minimal.msi
```

### Test MSI Installer

```powershell
# Test installation (requires Administrator privileges)
.\packaging\windows\scripts\test-msi-installer.ps1

# Test specific MSI file
.\packaging\windows\scripts\test-msi-installer.ps1 -MsiPath "target\ZipLock-1.0.0-x64-enhanced.msi"

# Test both enhanced and minimal (if available)
.\packaging\windows\scripts\test-msi-installer.ps1 -TestBoth
```

## Enhanced MSI Features

### User Feedback Dialogs

The enhanced MSI includes PowerShell-based custom actions that provide user feedback:

1. **Installation Success Dialog**
   - Confirms successful installation
   - Shows launch instructions (Start Menu, Desktop shortcut)
   - Thanks the user for choosing ZipLock

2. **Installation Failure Dialog**
   - Explains the installation failed
   - Provides troubleshooting tips
   - Includes support link

### Custom Action Scripts

- `show-install-success.ps1` - Displays success message with installation instructions
- `show-install-failure.ps1` - Shows failure message with troubleshooting guidance

These scripts are included in the MSI and executed automatically during installation.

### Event Logging

Installation events are logged to the Windows Application Event Log:
- Success: Event ID 1001
- Failure: Event ID 1002

## Prerequisites

### Build Requirements

- **Rust toolchain** with `x86_64-pc-windows-msvc` target
- **.NET SDK** (for WiX Toolset)
- **WiX Toolset 4.0.4** (automatically installed by build scripts)
- **PowerShell 5.0+** (for enhanced MSI features)
- **Python 3.6+** (optional, for proper icon generation)

### Icon Generation

Icons are generated from PNG sources in `assets/icons/`:

1. **Python method** (preferred): Uses Pillow to create proper .ico files
2. **Fallback method**: Copies PNG files with .ico extension

```powershell
# Generate icons with Python (if available)
python packaging\windows\scripts\create-icons.py --force

# Icons are saved to packaging\windows\resources\
```

## Build Process

### Automated Build (CI/CD)

The GitHub Actions workflow automatically:
1. Builds the Windows executable with static linking
2. Generates icons from PNG sources
3. Creates enhanced MSI with user feedback
4. Falls back to minimal MSI if enhanced version fails
5. Uploads MSI as build artifact

### Local Build Steps

1. **Build executable**:
   ```powershell
   cd apps\desktop
   $env:RUSTFLAGS = "-C target-feature=+crt-static"
   cargo build --release --target x86_64-pc-windows-msvc
   ```

2. **Generate icons**:
   ```powershell
   python packaging\windows\scripts\create-icons.py --force
   ```

3. **Create MSI installer**:
   ```powershell
   .\packaging\windows\scripts\build-windows-enhanced.ps1 -Version "1.0.0"
   ```

### Manual MSI Creation

```powershell
# Install WiX Toolset
dotnet tool install --global wix --version 4.0.4

# Create staging directory
New-Item -ItemType Directory -Path "target\windows-package" -Force
Copy-Item "target\x86_64-pc-windows-msvc\release\ziplock.exe" "target\windows-package\"

# Copy icons and custom action scripts
Copy-Item "packaging\windows\resources\*.ico" "target\windows-package\"
Copy-Item "packaging\windows\scripts\show-install-*.ps1" "target\windows-package\"

# Build enhanced MSI
cd packaging\windows\installer
wix build "ziplock-enhanced.wxs" -define "SourceDir=..\..\..\target\windows-package" -define "Version=1.0.0" -out "..\..\..\target\ZipLock-1.0.0-x64-enhanced.msi"
```

## Testing

### Installation Testing

```powershell
# Test MSI installation (requires Admin privileges)
.\packaging\windows\scripts\test-msi-installer.ps1

# The test script will:
# 1. Install the MSI
# 2. Verify executable and shortcuts are created
# 3. Test uninstallation
# 4. Check for user feedback dialogs (enhanced MSI only)
```

### Verification Checklist

After installation, verify:
- [ ] Executable installed to `Program Files\ZipLock\bin\ziplock.exe`
- [ ] Start Menu shortcut created: "ZipLock Password Manager"
- [ ] Desktop shortcut created (if selected during install)
- [ ] Icons display correctly in all locations
- [ ] Application appears in Add/Remove Programs
- [ ] Success dialog appeared (enhanced MSI)
- [ ] Application launches successfully

## Troubleshooting

### Common Issues

1. **WiX build fails**:
   - Ensure WiX 4.0.4 is installed: `wix --version`
   - Check that all source files exist in staging directory
   - Verify icon files are present

2. **Icons not displaying**:
   - Run icon generation script: `python create-icons.py --force`
   - Check that .ico files exist in `packaging\windows\resources\`
   - Verify icons are copied to staging directory

3. **Custom actions fail**:
   - Ensure PowerShell 5.0+ is available
   - Check custom action scripts exist and have valid syntax
   - Review Windows Event Log for installation errors
   - Falls back to minimal MSI automatically

4. **Installation fails**:
   - Run installer as Administrator
   - Check available disk space
   - Close any running instances of ZipLock
   - Disable antivirus temporarily during installation

### Getting Help

- Check build logs for detailed error messages
- Review Windows Event Log (Application) for installation events
- Use verbose MSI logging: `msiexec /i installer.msi /L*v install.log`
- Create issues on GitHub: https://github.com/ejangi/ziplock/issues

## Contributing

When modifying Windows packaging:

1. Test both enhanced and minimal MSI configurations
2. Verify custom action scripts work correctly
3. Test installation and uninstallation thoroughly
4. Update documentation for any new features
5. Ensure backward compatibility with older Windows versions

### Adding New Custom Actions

To add new custom actions:

1. Create PowerShell script in `scripts/` directory
2. Add script to `ziplock-enhanced.wxs` configuration
3. Update build scripts to copy the new script
4. Test thoroughly with the MSI test script
5. Document the new feature in this README