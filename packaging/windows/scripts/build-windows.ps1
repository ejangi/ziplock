# ZipLock Windows Build Script
# Builds the ZipLock desktop application for Windows x64

param(
    [string]$Configuration = "release",
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipBuild = $false,
    [switch]$CreateMsi = $false,
    [switch]$Sign = $false,
    [string]$SigningCert = "",
    [string]$OutputDir = "target\windows-package"
)

# Script configuration
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$PackagingDir = Join-Path $ProjectRoot "packaging\windows"
$BuildDir = Join-Path $ProjectRoot "target\$Target\$Configuration"
$AppDir = Join-Path $ProjectRoot "apps\desktop"

Write-Host "ZipLock Windows Build Script" -ForegroundColor Green
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Cyan

# Ensure output directory exists
if (!(Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-Host "Created output directory: $OutputDir" -ForegroundColor Yellow
}

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# Verify prerequisites
Write-Host "`nVerifying prerequisites..." -ForegroundColor Blue

if (!(Test-Command "cargo")) {
    Write-Error "Cargo not found. Please install Rust toolchain."
    exit 1
}

if (!(Test-Command "rustup")) {
    Write-Error "Rustup not found. Please install Rust toolchain."
    exit 1
}

# Check if target is installed
$installedTargets = rustup target list --installed
if ($installedTargets -notmatch $Target) {
    Write-Host "Installing Rust target: $Target" -ForegroundColor Yellow
    rustup target add $Target
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install Rust target: $Target"
        exit 1
    }
}

# Build the application
if (-not $SkipBuild) {
    Write-Host "`nBuilding ZipLock for Windows..." -ForegroundColor Blue

    Set-Location $ProjectRoot

    # Build shared library first
    Write-Host "Building shared library..." -ForegroundColor Cyan
    cargo build --package ziplock-shared --target $Target --profile $Configuration
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to build shared library"
        exit 1
    }

    # Build desktop application
    Write-Host "Building desktop application..." -ForegroundColor Cyan
    cargo build --package ziplock-desktop --bin ziplock --target $Target --profile $Configuration
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to build desktop application"
        exit 1
    }

    Write-Host "Build completed successfully!" -ForegroundColor Green
} else {
    Write-Host "`nSkipping build (using existing binaries)..." -ForegroundColor Yellow
}

# Verify binary exists
$BinaryPath = Join-Path $BuildDir "ziplock.exe"
if (!(Test-Path $BinaryPath)) {
    Write-Error "Binary not found at: $BinaryPath"
    Write-Error "Make sure the build completed successfully."
    exit 1
}

# Copy binary to output directory
$OutputBinary = Join-Path $OutputDir "ziplock.exe"
Copy-Item $BinaryPath $OutputBinary -Force
Write-Host "Copied binary to: $OutputBinary" -ForegroundColor Green

# Copy resources
Write-Host "`nCopying Windows resources..." -ForegroundColor Blue
$ResourcesDir = Join-Path $PackagingDir "resources"
if (Test-Path $ResourcesDir) {
    Copy-Item "$ResourcesDir\*" $OutputDir -Recurse -Force
    Write-Host "Copied resources from: $ResourcesDir" -ForegroundColor Green
}

# Copy license file
$LicensePath = Join-Path $ProjectRoot "LICENSE.md"
if (Test-Path $LicensePath) {
    Copy-Item $LicensePath (Join-Path $OutputDir "LICENSE.txt") -Force
    Write-Host "Copied license file" -ForegroundColor Green
}

# Copy README
$ReadmePath = Join-Path $ProjectRoot "README.md"
if (Test-Path $ReadmePath) {
    Copy-Item $ReadmePath (Join-Path $OutputDir "README.txt") -Force
    Write-Host "Copied README file" -ForegroundColor Green
}

# Sign binary if requested
if ($Sign -and $SigningCert) {
    Write-Host "`nSigning binary..." -ForegroundColor Blue

    if (!(Test-Command "signtool")) {
        Write-Error "signtool.exe not found. Please install Windows SDK."
        exit 1
    }

    & signtool sign /f $SigningCert /t http://timestamp.sectigo.com /v $OutputBinary
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to sign binary"
        exit 1
    }

    Write-Host "Binary signed successfully!" -ForegroundColor Green
}

# Create MSI installer if requested
if ($CreateMsi) {
    Write-Host "`nCreating MSI installer..." -ForegroundColor Blue

    $CreateMsiScript = Join-Path $PackagingDir "scripts\create-msi.ps1"
    if (!(Test-Path $CreateMsiScript)) {
        Write-Error "create-msi.ps1 script not found at: $CreateMsiScript"
        exit 1
    }

    & $CreateMsiScript -SourceDir $OutputDir -OutputDir $OutputDir -Sign:$Sign -SigningCert $SigningCert
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to create MSI installer"
        exit 1
    }
}

# Display build information
Write-Host "`nBuild Summary" -ForegroundColor Green
Write-Host "==============" -ForegroundColor Green
Write-Host "Configuration: $Configuration" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Cyan
Write-Host "Output Directory: $OutputDir" -ForegroundColor Cyan
Write-Host "Binary: $OutputBinary" -ForegroundColor Cyan

if (Test-Path $OutputBinary) {
    $BinaryInfo = Get-Item $OutputBinary
    Write-Host "Binary Size: $([math]::Round($BinaryInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan
    Write-Host "Binary Modified: $($BinaryInfo.LastWriteTime)" -ForegroundColor Cyan
}

Write-Host "`nWindows build completed successfully!" -ForegroundColor Green

# Test basic functionality
Write-Host "`nTesting binary..." -ForegroundColor Blue
$TestOutput = & $OutputBinary --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "Binary test successful: $TestOutput" -ForegroundColor Green
} else {
    Write-Warning "Binary test failed, but build completed"
    Write-Host "Test output: $TestOutput" -ForegroundColor Yellow
}

Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "- Test the binary: $OutputBinary" -ForegroundColor White
Write-Host "- Create MSI installer: .\build-windows.ps1 -CreateMsi" -ForegroundColor White
Write-Host "- Sign for distribution: .\build-windows.ps1 -Sign -SigningCert `"path\to\cert.pfx`"" -ForegroundColor White
