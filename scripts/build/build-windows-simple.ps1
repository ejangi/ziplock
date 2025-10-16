# ZipLock Simplified Windows Build Script
# Builds Windows MSI installer with minimal configuration to avoid common issues

param(
    [string]$Configuration = "release",
    [string]$Version = "",
    [switch]$Clean = $false,
    [switch]$SkipBuild = $false,
    [switch]$TestInstall = $false,
    [switch]$Verbose = $false
)

# Script configuration
$ErrorActionPreference = "Stop"
$VerbosePreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# Path configuration
$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$PackagingDir = Join-Path $ProjectRoot "packaging\windows"
$BuildDir = Join-Path $ProjectRoot "target\x86_64-pc-windows-msvc\$Configuration"
$OutputDir = Join-Path $ProjectRoot "target\windows-build"

Write-Host "ZipLock Simplified Windows Build" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Cyan
Write-Host "Output Directory: $OutputDir" -ForegroundColor Cyan

# Extract version if not provided
if (-not $Version) {
    if (Test-Path "$ProjectRoot\Cargo.toml") {
        $CargoToml = Get-Content "$ProjectRoot\Cargo.toml" -Raw
        if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
            $Version = $Matches[1]
            Write-Host "Extracted version from Cargo.toml: $Version" -ForegroundColor Yellow
        } else {
            $Version = "1.0.0"
            Write-Host "Could not extract version, using default: $Version" -ForegroundColor Yellow
        }
    } else {
        $Version = "1.0.0"
        Write-Host "No Cargo.toml found, using default version: $Version" -ForegroundColor Yellow
    }
}

function Test-Prerequisites {
    Write-Host "`nChecking Prerequisites..." -ForegroundColor Blue

    # Check Rust
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: Rust/Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
        return $false
    }

    # Check target
    $installedTargets = rustup target list --installed
    if ($installedTargets -notmatch "x86_64-pc-windows-msvc") {
        Write-Host "Installing x86_64-pc-windows-msvc target..." -ForegroundColor Yellow
        rustup target add x86_64-pc-windows-msvc
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Failed to install Windows target" -ForegroundColor Red
            return $false
        }
    }

    # Check .NET (optional for MSI)
    if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) {
        Write-Host "WARNING: .NET not found. MSI creation will be skipped." -ForegroundColor Yellow
        Write-Host "To create MSI packages, install .NET SDK from https://dotnet.microsoft.com/download" -ForegroundColor Yellow
        return $true  # Continue without MSI capability
    }

    # Check/Install WiX (optional)
    if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
        Write-Host "Installing WiX toolset..." -ForegroundColor Yellow
        try {
            dotnet tool install --global wix --version 4.0.4 2>$null
            if ($LASTEXITCODE -eq 0) {
                # Update PATH for current session
                $dotnetToolsPath = "$env:USERPROFILE\.dotnet\tools"
                if ($env:PATH -notlike "*$dotnetToolsPath*") {
                    $env:PATH += ";$dotnetToolsPath"
                }
                Write-Host "WiX toolset installed successfully" -ForegroundColor Green
            }
        } catch {
            Write-Host "WARNING: Failed to install WiX toolset. MSI creation will be skipped." -ForegroundColor Yellow
        }
    }

    Write-Host "Prerequisites check completed" -ForegroundColor Green
    return $true
}

function Build-Application {
    if ($SkipBuild) {
        Write-Host "`nSkipping build (using existing binaries)..." -ForegroundColor Yellow

        # Check if binary exists
        $BinaryPath = Join-Path $script:BuildDir "ziplock.exe"
        if (!(Test-Path $BinaryPath)) {
            Write-Host "ERROR: No existing binary found at $BinaryPath" -ForegroundColor Red
            Write-Host "Run without -SkipBuild to build the application first" -ForegroundColor Yellow
            return $false
        }

        $BinaryInfo = Get-Item $BinaryPath
        Write-Host "Using existing binary: $([math]::Round($BinaryInfo.Length / 1MB, 2)) MB" -ForegroundColor Green
        return $true
    }

    Write-Host "`nBuilding Application..." -ForegroundColor Blue

    try {
        Set-Location $ProjectRoot

        # Clean if requested
        if ($Clean) {
            Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
            cargo clean
        }

        # Build desktop application
        Write-Host "Building ZipLock desktop application..." -ForegroundColor Cyan
        cargo build --bin ziplock --target x86_64-pc-windows-msvc --profile $Configuration

        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
            return $false
        }

        # Verify binary exists
        $BinaryPath = Join-Path $script:BuildDir "ziplock.exe"
        if (!(Test-Path $BinaryPath)) {
            Write-Host "ERROR: Binary not found at $BinaryPath" -ForegroundColor Red
            return $false
        }

        $BinaryInfo = Get-Item $BinaryPath
        Write-Host "Build successful! Binary size: $([math]::Round($BinaryInfo.Length / 1MB, 2)) MB" -ForegroundColor Green

        # Test binary execution
        try {
            $VersionOutput = & $BinaryPath --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Binary test: $VersionOutput" -ForegroundColor Green
            }
        } catch {
            Write-Host "WARNING: Binary test failed, but continuing..." -ForegroundColor Yellow
        }

        return $true
    }
    catch {
        Write-Host "ERROR: Build failed - $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Prepare-PackageFiles {
    Write-Host "`nPreparing Package Files..." -ForegroundColor Blue

    # Clean/create output directory
    if ($Clean -and (Test-Path $OutputDir)) {
        Remove-Item $OutputDir -Recurse -Force
    }

    if (!(Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    }

    # Copy binary
    $BinaryPath = Join-Path $BuildDir "ziplock.exe"
    $OutputBinary = Join-Path $OutputDir "ziplock.exe"

    try {
        Copy-Item $BinaryPath $OutputBinary -Force
        Write-Host "Copied binary to package directory" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "ERROR: Failed to copy binary - $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Create-MSI {
    Write-Host "`nCreating MSI Installer..." -ForegroundColor Blue

    # Check if WiX is available
    if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
        Write-Host "WiX not available, skipping MSI creation" -ForegroundColor Yellow
        return $true  # Not an error, just skip
    }

    try {
        $InstallerDir = Join-Path $PackagingDir "installer"
        $WxsFile = Join-Path $InstallerDir "ziplock-minimal.wxs"
        $LicenseFile = Join-Path $InstallerDir "license-simple.rtf"
        $MsiOutput = Join-Path $ProjectRoot "target\ZipLock-$Version-x64.msi"

        # Verify required files exist
        if (!(Test-Path $WxsFile)) {
            Write-Host "ERROR: WXS file not found: $WxsFile" -ForegroundColor Red
            return $false
        }

        if (!(Test-Path $LicenseFile)) {
            Write-Host "WARNING: License file not found: $LicenseFile" -ForegroundColor Yellow
        }

        # Change to installer directory
        Set-Location $InstallerDir

        # Build MSI using minimal configuration
        Write-Host "Building MSI with minimal configuration..." -ForegroundColor Cyan
        Write-Host "Command: wix build ziplock-minimal.wxs -define SourceDir=$OutputDir -define Version=$Version -out $MsiOutput" -ForegroundColor Gray

        & wix build "ziplock-minimal.wxs" -define "SourceDir=$OutputDir" -define "Version=$Version" -out $MsiOutput

        if ($LASTEXITCODE -eq 0) {
            $MsiInfo = Get-Item $MsiOutput
            Write-Host "MSI created successfully!" -ForegroundColor Green
            Write-Host "File: $($MsiInfo.FullName)" -ForegroundColor Cyan
            Write-Host "Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan
            return $true
        } else {
            Write-Host "ERROR: MSI build failed with exit code $LASTEXITCODE" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "ERROR: MSI creation failed - $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Test-MSIInstallation {
    Write-Host "`nTesting MSI Installation..." -ForegroundColor Blue

    $MsiFile = Join-Path $ProjectRoot "target\ZipLock-$Version-x64.msi"

    if (!(Test-Path $MsiFile)) {
        Write-Host "ERROR: MSI file not found: $MsiFile" -ForegroundColor Red
        return $false
    }

    try {
        # Test MSI properties (basic validation)
        Write-Host "Validating MSI package..." -ForegroundColor Cyan
        $MsiInfo = Get-Item $MsiFile

        if ($MsiInfo.Length -lt 1MB) {
            Write-Host "WARNING: MSI file seems too small ($([math]::Round($MsiInfo.Length / 1KB, 0)) KB)" -ForegroundColor Yellow
        } else {
            Write-Host "MSI validation passed: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB" -ForegroundColor Green
        }

        # Note: Actual installation testing would require admin rights and could interfere with system
        Write-Host "For full installation testing, run as administrator:" -ForegroundColor Yellow
        Write-Host "  msiexec /i `"$MsiFile`" /qn /l*v install.log" -ForegroundColor Gray
        Write-Host "  msiexec /x `"$MsiFile`" /qn /l*v uninstall.log" -ForegroundColor Gray

        return $true
    }
    catch {
        Write-Host "ERROR: MSI validation failed - $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    $success = $true

    # Prerequisites
    if (-not (Test-Prerequisites)) {
        $success = $false
    }

    # Build
    if ($success -and -not $SkipBuild -and -not (Build-Application)) {
        $success = $false
    }

    # Package
    if ($success -and -not (Prepare-PackageFiles)) {
        $success = $false
    }

    # MSI Creation
    if ($success -and -not (Create-MSI)) {
        $success = $false
    }

    # Test Installation
    if ($success -and $TestInstall -and -not (Test-MSIInstallation)) {
        $success = $false
    }

    # Summary
    Write-Host "`n" -NoNewline
    if ($success) {
        Write-Host "✅ Build completed successfully!" -ForegroundColor Green
        Write-Host "`nOutput files:" -ForegroundColor Cyan

        $OutputBinary = Join-Path $OutputDir "ziplock.exe"
        if (Test-Path $OutputBinary) {
            $BinaryInfo = Get-Item $OutputBinary
            Write-Host "  Binary: $OutputBinary ($([math]::Round($BinaryInfo.Length / 1MB, 2)) MB)" -ForegroundColor Gray
        }

        $MsiFile = Join-Path $ProjectRoot "target\ZipLock-$Version-x64.msi"
        if (Test-Path $MsiFile) {
            $MsiInfo = Get-Item $MsiFile
            Write-Host "  MSI:    $MsiFile ($([math]::Round($MsiInfo.Length / 1MB, 2)) MB)" -ForegroundColor Gray
        }

        Write-Host "`nTo test the installer:" -ForegroundColor Yellow
        Write-Host "  .\scripts\build\build-windows-simple.ps1 -TestInstall" -ForegroundColor Gray
    } else {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        Write-Host "`nCommon solutions:" -ForegroundColor Yellow
        Write-Host "  1. Install Visual Studio Build Tools with C++ workload" -ForegroundColor Gray
        Write-Host "  2. Install .NET SDK 8.0+ from https://dotnet.microsoft.com/download" -ForegroundColor Gray
        Write-Host "  3. Run: rustup target add x86_64-pc-windows-msvc" -ForegroundColor Gray
        Write-Host "  4. Check Windows Defender isn't blocking file operations" -ForegroundColor Gray
        exit 1
    }
}
catch {
    Write-Host "`n❌ Unexpected error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
finally {
    # Return to original directory
    if ($ProjectRoot -and (Test-Path $ProjectRoot)) {
        Set-Location $ProjectRoot
    }
}
