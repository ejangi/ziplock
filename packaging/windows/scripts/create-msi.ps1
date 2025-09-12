# ZipLock MSI Installer Creation Script
# Creates MSI installer using WiX Toolset v4

param(
    [Parameter(Mandatory=$true)]
    [string]$SourceDir,

    [string]$OutputDir = ".",
    [string]$Version = "1.0.0",
    [switch]$Sign = $false,
    [string]$SigningCert = ""
)

# Script configuration
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$InstallerDir = Join-Path $ProjectRoot "packaging\windows\installer"
$WxsFile = Join-Path $InstallerDir "ziplock.wxs"

Write-Host "ZipLock MSI Installer Creation" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Green
Write-Host "Source Directory: $SourceDir" -ForegroundColor Cyan
Write-Host "Output Directory: $OutputDir" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Cyan
Write-Host "WXS File: $WxsFile" -ForegroundColor Cyan

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# Verify prerequisites
Write-Host "`nVerifying prerequisites..." -ForegroundColor Blue

# Check for WiX toolset
if (!(Test-Command "wix")) {
    Write-Host "Installing WiX Toolset..." -ForegroundColor Yellow
    try {
        dotnet tool install --global wix --version 4.0.4
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to install WiX toolset"
            exit 1
        }
    }
    catch {
        Write-Error "Failed to install WiX toolset: $_"
        exit 1
    }
}

# Add WiX UI extension
Write-Host "Adding WiX UI extension..." -ForegroundColor Yellow
wix extension add WixToolset.UI.wixext
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Failed to add WiX UI extension, but continuing..."
}

# Verify source directory exists
if (!(Test-Path $SourceDir)) {
    Write-Error "Source directory not found: $SourceDir"
    exit 1
}

# Verify WXS file exists
if (!(Test-Path $WxsFile)) {
    Write-Error "WiX source file not found: $WxsFile"
    exit 1
}

# Verify binary exists in source directory
$BinaryPath = Join-Path $SourceDir "ziplock.exe"
if (!(Test-Path $BinaryPath)) {
    Write-Error "ZipLock binary not found at: $BinaryPath"
    exit 1
}

# Create output directory if it doesn't exist
if (!(Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-Host "Created output directory: $OutputDir" -ForegroundColor Yellow
}

# Extract version from binary if not provided
if ($Version -eq "1.0.0") {
    try {
        $VersionInfo = Get-ItemProperty $BinaryPath | Select-Object -ExpandProperty VersionInfo
        if ($VersionInfo.FileVersion) {
            $Version = $VersionInfo.FileVersion
            Write-Host "Extracted version from binary: $Version" -ForegroundColor Yellow
        }
    }
    catch {
        Write-Warning "Could not extract version from binary, using default: $Version"
    }
}

# Build MSI installer
Write-Host "`nBuilding MSI installer..." -ForegroundColor Blue

$MsiFileName = "ZipLock-$Version-x64.msi"
$MsiPath = Join-Path $OutputDir $MsiFileName

# Change to installer directory for relative paths
Push-Location $InstallerDir

try {
    Write-Host "Running WiX build command..." -ForegroundColor Cyan
    Write-Host "Command: wix build ziplock.wxs -ext WixToolset.UI.wixext -define SourceDir=$SourceDir -define Version=$Version -out $MsiPath" -ForegroundColor Gray

    wix build ziplock.wxs -ext WixToolset.UI.wixext -define "SourceDir=$SourceDir" -define "Version=$Version" -out $MsiPath

    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX build failed with exit code: $LASTEXITCODE"
        exit 1
    }
}
finally {
    Pop-Location
}

# Verify MSI was created
if (!(Test-Path $MsiPath)) {
    Write-Error "MSI installer was not created at: $MsiPath"
    exit 1
}

$MsiInfo = Get-Item $MsiPath
Write-Host "MSI installer created successfully!" -ForegroundColor Green
Write-Host "File: $MsiPath" -ForegroundColor Cyan
Write-Host "Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan
Write-Host "Created: $($MsiInfo.CreationTime)" -ForegroundColor Cyan

# Sign MSI if requested
if ($Sign -and $SigningCert) {
    Write-Host "`nSigning MSI installer..." -ForegroundColor Blue

    if (!(Test-Command "signtool")) {
        Write-Error "signtool.exe not found. Please install Windows SDK."
        exit 1
    }

    if (!(Test-Path $SigningCert)) {
        Write-Error "Signing certificate not found: $SigningCert"
        exit 1
    }

    & signtool sign /f $SigningCert /t http://timestamp.sectigo.com /v $MsiPath
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to sign MSI installer"
        exit 1
    }

    Write-Host "MSI installer signed successfully!" -ForegroundColor Green
}

# Test MSI basic properties
Write-Host "`nTesting MSI installer..." -ForegroundColor Blue
try {
    # Try to get MSI properties using Windows Installer API
    $WindowsInstaller = New-Object -ComObject WindowsInstaller.Installer
    $Database = $WindowsInstaller.GetType().InvokeMember("OpenDatabase", "InvokeMethod", $null, $WindowsInstaller, @($MsiPath, 0))

    # Get ProductName
    $View = $Database.GetType().InvokeMember("OpenView", "InvokeMethod", $null, $Database, @("SELECT Value FROM Property WHERE Property='ProductName'"))
    $View.GetType().InvokeMember("Execute", "InvokeMethod", $null, $View, $null)
    $Record = $View.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $View, $null)
    if ($Record) {
        $ProductName = $Record.GetType().InvokeMember("StringData", "GetProperty", $null, $Record, 1)
        Write-Host "Product Name: $ProductName" -ForegroundColor Green
    }

    # Cleanup COM objects
    [System.Runtime.InteropServices.Marshal]::ReleaseComObject($Database) | Out-Null
    [System.Runtime.InteropServices.Marshal]::ReleaseComObject($WindowsInstaller) | Out-Null
}
catch {
    Write-Warning "Could not test MSI properties, but file was created successfully"
}

Write-Host "`nMSI Creation Summary" -ForegroundColor Green
Write-Host "====================" -ForegroundColor Green
Write-Host "MSI File: $MsiPath" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Cyan
Write-Host "Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan
Write-Host "Signed: $(if ($Sign -and $SigningCert) { 'Yes' } else { 'No' })" -ForegroundColor Cyan

Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "- Test installation: Start-Process '$MsiPath' -ArgumentList '/quiet' -Wait" -ForegroundColor White
Write-Host "- Manual install: Double-click '$MsiPath'" -ForegroundColor White
Write-Host "- Verify installation: Check 'C:\Program Files\ZipLock\'" -ForegroundColor White

Write-Host "`nMSI installer creation completed successfully!" -ForegroundColor Green
