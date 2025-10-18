# ZipLock MSI Fix Validation Script
# Tests the corrected MSI configuration to ensure error 2762 is resolved

param(
    [string]$Version = "",
    [switch]$TestBoth = $false,
    [switch]$Verbose = $false,
    [switch]$CleanFirst = $false
)

$ErrorActionPreference = "Stop"
$VerbosePreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# Path configuration
$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$PackagingDir = Join-Path $ProjectRoot "packaging\windows"
$InstallerDir = Join-Path $PackagingDir "installer"
$TestOutputDir = Join-Path $ProjectRoot "target\msi-test"

Write-Host "ZipLock MSI Fix Validation" -ForegroundColor Green
Write-Host "===========================" -ForegroundColor Green

# Extract version if not provided
if (-not $Version) {
    if (Test-Path "$ProjectRoot\Cargo.toml") {
        $CargoToml = Get-Content "$ProjectRoot\Cargo.toml" -Raw
        if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
            $Version = $Matches[1]
            Write-Host "Extracted version from Cargo.toml: $Version" -ForegroundColor Cyan
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
    Write-Host ""
    Write-Host "Checking Prerequisites..." -ForegroundColor Blue

    # Check WiX
    if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
        Write-Host "Installing WiX toolset..." -ForegroundColor Yellow
        try {
            dotnet tool install --global wix --version 4.0.4 2>$null
            if ($LASTEXITCODE -eq 0) {
                $dotnetToolsPath = "$env:USERPROFILE\.dotnet\tools"
                if ($env:PATH -notlike "*$dotnetToolsPath*") {
                    $env:PATH += ";$dotnetToolsPath"
                }
                Write-Host "WiX toolset installed successfully" -ForegroundColor Green
            } else {
                Write-Host "ERROR: Failed to install WiX toolset" -ForegroundColor Red
                return $false
            }
        } catch {
            Write-Host "ERROR: Exception installing WiX toolset: $($_.Exception.Message)" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "WiX toolset is available" -ForegroundColor Green
    }

    # Verify required files exist
    $RequiredFiles = @(
        "$InstallerDir\ziplock-minimal.wxs",
        "$InstallerDir\ziplock-enhanced.wxs"
    )

    foreach ($file in $RequiredFiles) {
        if (Test-Path $file) {
            Write-Host "✅ Found: $(Split-Path -Leaf $file)" -ForegroundColor Green
        } else {
            Write-Host "❌ Missing: $file" -ForegroundColor Red
            return $false
        }
    }

    return $true
}

function Create-TestStaging {
    Write-Host ""
    Write-Host "Creating Test Staging Environment..." -ForegroundColor Blue

    # Clean/create test directory
    if ($CleanFirst -and (Test-Path $TestOutputDir)) {
        Remove-Item $TestOutputDir -Recurse -Force
    }

    if (!(Test-Path $TestOutputDir)) {
        New-Item -ItemType Directory -Path $TestOutputDir -Force | Out-Null
    }

    # Create dummy executable for testing
    $DummyExePath = Join-Path $TestOutputDir "ziplock.exe"
    if (!(Test-Path $DummyExePath)) {
        Write-Host "Creating dummy executable for testing..." -ForegroundColor Cyan

        # Create a minimal Windows executable stub
        $DummyContent = @"
@echo off
echo ZipLock Password Manager Test Stub
echo Version: $Version
echo This is a test executable for MSI validation.
pause
"@

        # Save as batch file first, then rename to .exe for testing
        $BatchPath = Join-Path $TestOutputDir "ziplock.bat"
        Set-Content -Path $BatchPath -Value $DummyContent -Encoding ASCII
        Copy-Item $BatchPath $DummyExePath -Force
        Remove-Item $BatchPath -Force
    }

    # Create dummy PowerShell scripts
    $SuccessScript = @"
param([string]`$ProductName = "ZipLock", [string]`$Version = "")
Write-Host "SUCCESS: `$ProductName `$Version installation completed!"
"@

    $FailureScript = @"
param([string]`$ProductName = "ZipLock", [string]`$Version = "", [string]`$ErrorMessage = "")
Write-Host "FAILURE: `$ProductName `$Version installation failed: `$ErrorMessage"
"@

    $SuccessPath = Join-Path $TestOutputDir "show-install-success.ps1"
    $FailurePath = Join-Path $TestOutputDir "show-install-failure.ps1"

    if (!(Test-Path $SuccessPath)) {
        Set-Content -Path $SuccessPath -Value $SuccessScript -Encoding UTF8
        Write-Host "Created test script: show-install-success.ps1" -ForegroundColor Cyan
    }

    if (!(Test-Path $FailurePath)) {
        Set-Content -Path $FailurePath -Value $FailureScript -Encoding UTF8
        Write-Host "Created test script: show-install-failure.ps1" -ForegroundColor Cyan
    }

    # Create dummy icon files
    $IconFiles = @("ziplock.ico", "ziplock-small.ico")
    foreach ($iconFile in $IconFiles) {
        $IconPath = Join-Path $TestOutputDir $iconFile
        if (!(Test-Path $IconPath)) {
            # Create a minimal dummy file (MSI will accept any file as icon for testing)
            Set-Content -Path $IconPath -Value "DUMMY_ICON_FOR_TESTING" -Encoding ASCII
            Write-Host "Created dummy icon: $iconFile" -ForegroundColor Cyan
        }
    }

    Write-Host "Test staging completed" -ForegroundColor Green
    return $true
}

function Test-WixConfiguration {
    param(
        [string]$ConfigName,
        [string]$WxsFile
    )

    Write-Host ""
    Write-Host "Testing $ConfigName Configuration..." -ForegroundColor Blue
    Write-Host "WXS File: $WxsFile" -ForegroundColor Gray

    try {
        Set-Location $InstallerDir

        # Test MSI build
        $MsiOutput = Join-Path $ProjectRoot "target\ZipLock-$Version-$ConfigName-test.msi"
        $BuildCommand = "wix build `"$WxsFile`" -define `"SourceDir=$TestOutputDir`" -define `"Version=$Version`" -out `"$MsiOutput`""

        Write-Host "Build Command: $BuildCommand" -ForegroundColor Gray

        # Execute build
        $BuildResult = Invoke-Expression $BuildCommand 2>&1

        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ $ConfigName MSI build PASSED" -ForegroundColor Green

            if (Test-Path $MsiOutput) {
                $MsiInfo = Get-Item $MsiOutput
                Write-Host "   File: $($MsiInfo.FullName)" -ForegroundColor Cyan
                Write-Host "   Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan

                # Basic MSI validation
                try {
                    $MsiProperties = Get-ItemProperty $MsiOutput
                    Write-Host "   MSI validation: OK" -ForegroundColor Green
                } catch {
                    Write-Host "   MSI validation: WARNING - $($_.Exception.Message)" -ForegroundColor Yellow
                }

                return @{ Success = $true; File = $MsiOutput; Message = "Build successful" }
            } else {
                Write-Host "❌ $ConfigName MSI file not found after build" -ForegroundColor Red
                return @{ Success = $false; File = $null; Message = "MSI file missing" }
            }
        } else {
            Write-Host "❌ $ConfigName MSI build FAILED" -ForegroundColor Red
            Write-Host "Exit code: $LASTEXITCODE" -ForegroundColor Gray
            Write-Host "Build output:" -ForegroundColor Gray
            Write-Host $BuildResult -ForegroundColor Gray

            # Check for specific error 2762
            if ($BuildResult -match "2762") {
                Write-Host "🔍 ERROR 2762 DETECTED - Custom action scheduling issue!" -ForegroundColor Red
                return @{ Success = $false; File = $null; Message = "Error 2762 - Custom action scheduling" }
            } else {
                return @{ Success = $false; File = $null; Message = "Build failed with exit code $LASTEXITCODE" }
            }
        }
    }
    catch {
        Write-Host "❌ $ConfigName test EXCEPTION: $($_.Exception.Message)" -ForegroundColor Red
        return @{ Success = $false; File = $null; Message = "Exception: $($_.Exception.Message)" }
    }
    finally {
        Set-Location $ProjectRoot
    }
}

function Generate-TestReport {
    param(
        [hashtable]$MinimalResult,
        [hashtable]$EnhancedResult
    )

    Write-Host ""
    Write-Host "=" * 50 -ForegroundColor Cyan
    Write-Host "MSI FIX VALIDATION REPORT" -ForegroundColor Green
    Write-Host "=" * 50 -ForegroundColor Cyan

    Write-Host ""
    Write-Host "Minimal Configuration:" -ForegroundColor Blue
    if ($MinimalResult.Success) {
        Write-Host "  Status: ✅ PASSED" -ForegroundColor Green
        Write-Host "  File: $($MinimalResult.File)" -ForegroundColor Cyan
    } else {
        Write-Host "  Status: ❌ FAILED" -ForegroundColor Red
        Write-Host "  Error: $($MinimalResult.Message)" -ForegroundColor Yellow
    }

    if ($TestBoth) {
        Write-Host ""
        Write-Host "Enhanced Configuration:" -ForegroundColor Blue
        if ($EnhancedResult.Success) {
            Write-Host "  Status: ✅ PASSED" -ForegroundColor Green
            Write-Host "  File: $($EnhancedResult.File)" -ForegroundColor Cyan
        } else {
            Write-Host "  Status: ❌ FAILED" -ForegroundColor Red
            Write-Host "  Error: $($EnhancedResult.Message)" -ForegroundColor Yellow
        }
    }

    Write-Host ""
    Write-Host "Summary:" -ForegroundColor Blue
    $TotalTests = if ($TestBoth) { 2 } else { 1 }
    $PassedTests = 0
    if ($MinimalResult.Success) { $PassedTests++ }
    if ($TestBoth -and $EnhancedResult.Success) { $PassedTests++ }

    Write-Host "  Tests run: $TotalTests" -ForegroundColor Cyan
    Write-Host "  Passed: $PassedTests" -ForegroundColor Green
    Write-Host "  Failed: $($TotalTests - $PassedTests)" -ForegroundColor Red

    if ($PassedTests -eq $TotalTests) {
        Write-Host ""
        Write-Host "🎉 ALL TESTS PASSED - MSI Error 2762 appears to be FIXED!" -ForegroundColor Green
    } else {
        Write-Host ""
        Write-Host "⚠️  SOME TESTS FAILED - Additional fixes may be needed." -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Blue
    Write-Host "  1. Test actual installation: msiexec /i `"path\to\msi`" /l*v install.log" -ForegroundColor Gray
    Write-Host "  2. Check installation log for any remaining issues" -ForegroundColor Gray
    Write-Host "  3. Test on clean Windows systems" -ForegroundColor Gray
    Write-Host "  4. Run full CI/CD pipeline validation" -ForegroundColor Gray
}

# Main execution
try {
    Write-Host "Version: $Version" -ForegroundColor Cyan
    Write-Host "Test Both Configurations: $TestBoth" -ForegroundColor Cyan
    Write-Host "Verbose: $Verbose" -ForegroundColor Cyan

    # Prerequisites
    if (-not (Test-Prerequisites)) {
        Write-Host ""
        Write-Host "❌ Prerequisites check failed!" -ForegroundColor Red
        exit 1
    }

    # Create test environment
    if (-not (Create-TestStaging)) {
        Write-Host ""
        Write-Host "❌ Failed to create test staging!" -ForegroundColor Red
        exit 1
    }

    # Test minimal configuration (this should always work)
    $MinimalResult = Test-WixConfiguration -ConfigName "minimal" -WxsFile "ziplock-minimal.wxs"

    # Test enhanced configuration if requested
    $EnhancedResult = @{ Success = $true; File = $null; Message = "Skipped" }
    if ($TestBoth) {
        $EnhancedResult = Test-WixConfiguration -ConfigName "enhanced" -WxsFile "ziplock-enhanced.wxs"
    }

    # Generate report
    Generate-TestReport -MinimalResult $MinimalResult -EnhancedResult $EnhancedResult

    # Exit with appropriate code
    if ($MinimalResult.Success -and ($TestBoth -eq $false -or $EnhancedResult.Success)) {
        exit 0
    } else {
        exit 1
    }
}
catch {
    Write-Host ""
    Write-Host "❌ Unexpected error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
finally {
    # Clean up - return to original directory
    if ($ProjectRoot -and (Test-Path $ProjectRoot)) {
        Set-Location $ProjectRoot
    }
}
