# ZipLock Windows Icon Embedding Test Script
# Tests that icons are properly embedded in the Windows executable

param(
    [string]$ExePath = "",
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))

# Default executable path
if (-not $ExePath) {
    $ExePath = Join-Path $ProjectRoot "target\x86_64-pc-windows-msvc\release\ziplock.exe"
}

Write-Host "ZipLock Windows Icon Embedding Test" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green
Write-Host "Executable Path: $ExePath" -ForegroundColor Cyan
Write-Host "Verbose Mode: $Verbose" -ForegroundColor Cyan
Write-Host ""

# Function to test if executable exists
function Test-ExecutableExists {
    param([string]$Path)

    if (Test-Path $Path) {
        $fileInfo = Get-Item $Path
        Write-Host "✅ Executable found: $($fileInfo.Name)" -ForegroundColor Green
        Write-Host "   Size: $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
        Write-Host "   Created: $($fileInfo.CreationTime)" -ForegroundColor Gray
        return $true
    } else {
        Write-Host "❌ Executable not found: $Path" -ForegroundColor Red
        return $false
    }
}

# Function to extract and test icon resources
function Test-IconResources {
    param([string]$ExePath)

    Write-Host "🔍 Testing icon resources..." -ForegroundColor Blue

    try {
        # Load System.Drawing for icon extraction
        Add-Type -AssemblyName System.Drawing
        Add-Type -AssemblyName System.Windows.Forms

        # Try to extract icon from executable
        $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($ExePath)

        if ($icon) {
            Write-Host "✅ Icon extracted successfully!" -ForegroundColor Green
            Write-Host "   Size: $($icon.Width)x$($icon.Height)" -ForegroundColor Gray

            # Test icon properties
            if ($icon.Width -gt 0 -and $icon.Height -gt 0) {
                Write-Host "   Icon dimensions are valid" -ForegroundColor Green
            } else {
                Write-Host "   ⚠️ Icon has invalid dimensions" -ForegroundColor Yellow
            }

            $icon.Dispose()
            return $true
        } else {
            Write-Host "❌ No icon could be extracted" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "❌ Icon extraction failed: $_" -ForegroundColor Red
        return $false
    }
}

# Function to test Windows Explorer icon display
function Test-ExplorerIcon {
    param([string]$ExePath)

    Write-Host "🗂️ Testing Windows Explorer icon display..." -ForegroundColor Blue

    try {
        # Get file icon using Shell API
        $shell = New-Object -ComObject Shell.Application
        $folder = $shell.Namespace((Split-Path $ExePath))
        $file = $folder.ParseName((Split-Path $ExePath -Leaf))

        if ($file) {
            Write-Host "✅ File recognized by Windows Explorer" -ForegroundColor Green

            # Try to get icon information
            $iconLocation = $file.GetDetailsOf($null, 0)  # This might work differently
            Write-Host "   File accessible via Shell API" -ForegroundColor Gray

            # Cleanup COM object
            [System.Runtime.InteropServices.Marshal]::ReleaseComObject($shell) | Out-Null
            return $true
        } else {
            Write-Host "❌ File not recognized by Windows Explorer" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "⚠️ Windows Explorer test failed: $_" -ForegroundColor Yellow
        return $false
    }
}

# Function to test PE resource information
function Test-PEResources {
    param([string]$ExePath)

    Write-Host "📋 Testing PE resource information..." -ForegroundColor Blue

    try {
        # Get version info
        $versionInfo = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($ExePath)

        if ($versionInfo) {
            Write-Host "✅ Version information found:" -ForegroundColor Green
            Write-Host "   File Description: $($versionInfo.FileDescription)" -ForegroundColor Gray
            Write-Host "   Product Name: $($versionInfo.ProductName)" -ForegroundColor Gray
            Write-Host "   Company Name: $($versionInfo.CompanyName)" -ForegroundColor Gray
            Write-Host "   File Version: $($versionInfo.FileVersion)" -ForegroundColor Gray
            Write-Host "   Product Version: $($versionInfo.ProductVersion)" -ForegroundColor Gray

            # Check if custom properties are set
            $hasCustomInfo = $false
            if ($versionInfo.FileDescription -and $versionInfo.FileDescription -ne "") {
                $hasCustomInfo = $true
            }
            if ($versionInfo.ProductName -and $versionInfo.ProductName -ne "") {
                $hasCustomInfo = $true
            }

            if ($hasCustomInfo) {
                Write-Host "✅ Custom version information is embedded" -ForegroundColor Green
            } else {
                Write-Host "⚠️ No custom version information found" -ForegroundColor Yellow
            }

            return $true
        } else {
            Write-Host "❌ No version information found" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "❌ PE resource test failed: $_" -ForegroundColor Red
        return $false
    }
}

# Function to test taskbar icon display
function Test-TaskbarIcon {
    param([string]$ExePath)

    Write-Host "🖥️ Testing taskbar icon (requires manual verification)..." -ForegroundColor Blue

    Write-Host "⚠️ Manual test required:" -ForegroundColor Yellow
    Write-Host "   1. Run the executable: $ExePath" -ForegroundColor Gray
    Write-Host "   2. Check if the correct icon appears in the taskbar" -ForegroundColor Gray
    Write-Host "   3. Check if the icon appears in Alt+Tab switcher" -ForegroundColor Gray
    Write-Host "   4. Verify icon appears correctly in Windows Explorer" -ForegroundColor Gray

    return $true
}

# Function to save icon to file for visual inspection
function Save-IconForInspection {
    param([string]$ExePath)

    Write-Host "💾 Saving icon for visual inspection..." -ForegroundColor Blue

    try {
        Add-Type -AssemblyName System.Drawing

        $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($ExePath)
        if ($icon) {
            $outputDir = Join-Path $ProjectRoot "target"
            $iconPath = Join-Path $outputDir "extracted-icon.ico"

            $fileStream = [System.IO.File]::Create($iconPath)
            $icon.Save($fileStream)
            $fileStream.Close()
            $icon.Dispose()

            if (Test-Path $iconPath) {
                $iconInfo = Get-Item $iconPath
                Write-Host "✅ Icon saved for inspection: $iconPath" -ForegroundColor Green
                Write-Host "   Size: $([math]::Round($iconInfo.Length / 1KB, 1)) KB" -ForegroundColor Gray
                return $true
            }
        }

        Write-Host "❌ Could not save icon" -ForegroundColor Red
        return $false
    }
    catch {
        Write-Host "❌ Icon saving failed: $_" -ForegroundColor Red
        return $false
    }
}

# Main test execution
try {
    # Test 1: Check if executable exists
    if (-not (Test-ExecutableExists $ExePath)) {
        Write-Host ""
        Write-Host "❌ Cannot proceed - executable not found!" -ForegroundColor Red
        Write-Host "Build the Windows executable first:" -ForegroundColor Yellow
        Write-Host "   cd apps/desktop" -ForegroundColor Gray
        Write-Host "   cargo build --release --target x86_64-pc-windows-msvc" -ForegroundColor Gray
        exit 1
    }

    Write-Host ""

    # Test 2: Icon resource extraction
    $iconTest = Test-IconResources $ExePath
    Write-Host ""

    # Test 3: Windows Explorer recognition
    $explorerTest = Test-ExplorerIcon $ExePath
    Write-Host ""

    # Test 4: PE resource information
    $peTest = Test-PEResources $ExePath
    Write-Host ""

    # Test 5: Save icon for inspection
    if ($iconTest) {
        $saveTest = Save-IconForInspection $ExePath
        Write-Host ""
    }

    # Test 6: Manual taskbar test instructions
    Test-TaskbarIcon $ExePath
    Write-Host ""

    # Summary
    Write-Host "📊 Test Summary" -ForegroundColor Green
    Write-Host "===============" -ForegroundColor Green
    Write-Host "Icon Extraction: $(if ($iconTest) { '✅ PASS' } else { '❌ FAIL' })" -ForegroundColor $(if ($iconTest) { 'Green' } else { 'Red' })
    Write-Host "Explorer Recognition: $(if ($explorerTest) { '✅ PASS' } else { '⚠️ WARNING' })" -ForegroundColor $(if ($explorerTest) { 'Green' } else { 'Yellow' })
    Write-Host "PE Resources: $(if ($peTest) { '✅ PASS' } else { '❌ FAIL' })" -ForegroundColor $(if ($peTest) { 'Green' } else { 'Red' })

    $overallPass = $iconTest -and $peTest

    Write-Host ""
    if ($overallPass) {
        Write-Host "🎉 Icon embedding test PASSED!" -ForegroundColor Green
        Write-Host "The Windows executable has proper icon resources embedded." -ForegroundColor Green
    } else {
        Write-Host "❌ Icon embedding test FAILED!" -ForegroundColor Red
        Write-Host "The executable may be missing icon resources or version info." -ForegroundColor Red
        Write-Host ""
        Write-Host "Troubleshooting steps:" -ForegroundColor Yellow
        Write-Host "1. Ensure .ico files exist in packaging/windows/resources/" -ForegroundColor Gray
        Write-Host "2. Check that build.rs script is working correctly" -ForegroundColor Gray
        Write-Host "3. Verify embed-resource and winres dependencies" -ForegroundColor Gray
        Write-Host "4. Rebuild with: cargo clean && cargo build --release --target x86_64-pc-windows-msvc" -ForegroundColor Gray
    }

    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "1. Run the executable and verify the icon appears correctly" -ForegroundColor Gray
    Write-Host "2. Test MSI installer creation and verify installer icons" -ForegroundColor Gray
    Write-Host "3. Install the MSI and check Start Menu and Desktop shortcuts" -ForegroundColor Gray

    exit $(if ($overallPass) { 0 } else { 1 })
}
catch {
    Write-Host ""
    Write-Host "❌ Test script failed: $_" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
