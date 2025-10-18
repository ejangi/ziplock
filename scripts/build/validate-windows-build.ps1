# ZipLock Windows Build Validation Script
# Validates the complete Windows build process including MSI creation

param(
    [string]$Version = "",
    [switch]$CleanBuild = $false,
    [switch]$SkipTests = $false,
    [switch]$TestInstallation = $false,
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"
$VerbosePreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# Path configuration
$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$ValidationDir = Join-Path $ProjectRoot "target\validation"

Write-Host "ZipLock Windows Build Validation" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Initialize validation results
$script:ValidationResults = @()

function Add-ValidationResult {
    param(
        [string]$TestName,
        [string]$Status,
        [string]$Message = "",
        [string]$Details = ""
    )

    $script:ValidationResults += @{
        TestName = $TestName
        Status = $Status
        Message = $Message
        Details = $Details
        Timestamp = Get-Date
    }
}

function Write-Status {
    param(
        [string]$Message,
        [string]$Status = "INFO"
    )

    switch ($Status) {
        "PASS" { Write-Host "✅ $Message" -ForegroundColor Green }
        "FAIL" { Write-Host "❌ $Message" -ForegroundColor Red }
        "WARN" { Write-Host "⚠️  $Message" -ForegroundColor Yellow }
        "INFO" { Write-Host "ℹ️  $Message" -ForegroundColor Cyan }
        default { Write-Host $Message }
    }
}

function Test-BuildEnvironment {
    Write-Host "`n=== Testing Build Environment ===" -ForegroundColor Blue

    $allPassed = $true

    # Test Rust installation
    try {
        $rustVersion = cargo --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Status "Rust installed: $rustVersion" -Status "PASS"
            Add-ValidationResult -TestName "Rust Installation" -Status "PASS" -Message $rustVersion
        } else {
            throw "Cargo command failed"
        }
    } catch {
        Write-Status "Rust/Cargo not found or not working" -Status "FAIL"
        Add-ValidationResult -TestName "Rust Installation" -Status "FAIL" -Message "Rust/Cargo not available"
        $allPassed = $false
    }

    # Test Windows MSVC target
    try {
        $targets = rustup target list --installed 2>$null
        if ($targets -match "x86_64-pc-windows-msvc") {
            Write-Status "Windows MSVC target available" -Status "PASS"
            Add-ValidationResult -TestName "MSVC Target" -Status "PASS" -Message "x86_64-pc-windows-msvc installed"
        } else {
            Write-Status "Installing Windows MSVC target..." -Status "INFO"
            rustup target add x86_64-pc-windows-msvc
            if ($LASTEXITCODE -eq 0) {
                Write-Status "Windows MSVC target installed" -Status "PASS"
                Add-ValidationResult -TestName "MSVC Target" -Status "PASS" -Message "x86_64-pc-windows-msvc installed"
            } else {
                throw "Failed to install MSVC target"
            }
        }
    } catch {
        Write-Status "Failed to set up Windows MSVC target" -Status "FAIL"
        Add-ValidationResult -TestName "MSVC Target" -Status "FAIL" -Message $_.Exception.Message
        $allPassed = $false
    }

    # Test .NET SDK
    try {
        $dotnetVersion = dotnet --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Status ".NET SDK available: $dotnetVersion" -Status "PASS"
            Add-ValidationResult -TestName ".NET SDK" -Status "PASS" -Message $dotnetVersion
        } else {
            throw "dotnet command failed"
        }
    } catch {
        Write-Status ".NET SDK not available - MSI creation will be limited" -Status "WARN"
        Add-ValidationResult -TestName ".NET SDK" -Status "WARN" -Message "MSI creation may not work"
    }

    # Test WiX toolset
    try {
        $wixVersion = wix --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Status "WiX toolset available: $wixVersion" -Status "PASS"
            Add-ValidationResult -TestName "WiX Toolset" -Status "PASS" -Message $wixVersion
        } else {
            throw "WiX not available"
        }
    } catch {
        Write-Status "WiX toolset not available - attempting install..." -Status "WARN"
        try {
            dotnet tool install --global wix --version 4.0.4 2>$null
            $env:PATH += ";$env:USERPROFILE\.dotnet\tools"
            $wixVersion = wix --version 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Status "WiX toolset installed: $wixVersion" -Status "PASS"
                Add-ValidationResult -TestName "WiX Toolset" -Status "PASS" -Message "Installed $wixVersion"
            } else {
                throw "WiX installation failed"
            }
        } catch {
            Write-Status "Failed to install WiX toolset" -Status "FAIL"
            Add-ValidationResult -TestName "WiX Toolset" -Status "FAIL" -Message "Installation failed"
            $allPassed = $false
        }
    }

    return $allPassed
}

function Test-ProjectStructure {
    Write-Host "`n=== Validating Project Structure ===" -ForegroundColor Blue

    $allPassed = $true

    # Required files for Windows build
    $RequiredFiles = @(
        "Cargo.toml",
        "packaging\windows\installer\ziplock-minimal.wxs",
        "packaging\windows\installer\ziplock-enhanced.wxs",
        "packaging\windows\scripts\show-install-success.ps1",
        "packaging\windows\scripts\show-install-failure.ps1"
    )

    foreach ($file in $RequiredFiles) {
        $fullPath = Join-Path $ProjectRoot $file
        if (Test-Path $fullPath) {
            Write-Status "Found: $file" -Status "PASS"
            Add-ValidationResult -TestName "File: $file" -Status "PASS" -Message "File exists"
        } else {
            Write-Status "Missing: $file" -Status "FAIL"
            Add-ValidationResult -TestName "File: $file" -Status "FAIL" -Message "Required file missing"
            $allPassed = $false
        }
    }

    return $allPassed
}

function Test-Application Build {
    Write-Host "`n=== Testing Application Build ===" -ForegroundColor Blue

    try {
        Set-Location $ProjectRoot

        # Clean build if requested
        if ($CleanBuild) {
            Write-Status "Cleaning previous build..." -Status "INFO"
            cargo clean
        }

        # Set environment for Windows build
        $env:RUSTFLAGS = "-C target-feature=+crt-static"
        $env:ZIPLOCK_PRODUCTION = "1"

        Write-Status "Building Windows application..." -Status "INFO"

        # Build the application
        $buildOutput = cargo build --bin ziplock --target x86_64-pc-windows-msvc --release 2>&1

        if ($LASTEXITCODE -eq 0) {
            Write-Status "Application build successful" -Status "PASS"
            Add-ValidationResult -TestName "Application Build" -Status "PASS" -Message "Build completed successfully"

            # Verify binary exists and get info
            $binaryPath = Join-Path $ProjectRoot "target\x86_64-pc-windows-msvc\release\ziplock.exe"
            if (Test-Path $binaryPath) {
                $binaryInfo = Get-Item $binaryPath
                $sizeInMB = [math]::Round($binaryInfo.Length / 1MB, 2)
                Write-Status "Binary created: $sizeInMB MB" -Status "PASS"
                Add-ValidationResult -TestName "Binary Creation" -Status "PASS" -Message "Binary size: $sizeInMB MB"

                # Test binary execution
                try {
                    $versionOutput = & $binaryPath --version 2>&1
                    if ($LASTEXITCODE -eq 0) {
                        Write-Status "Binary execution test: $versionOutput" -Status "PASS"
                        Add-ValidationResult -TestName "Binary Execution" -Status "PASS" -Message $versionOutput
                    } else {
                        Write-Status "Binary execution failed (exit code: $LASTEXITCODE)" -Status "WARN"
                        Add-ValidationResult -TestName "Binary Execution" -Status "WARN" -Message "Failed with exit code $LASTEXITCODE"
                    }
                } catch {
                    Write-Status "Binary execution test failed: $($_.Exception.Message)" -Status "WARN"
                    Add-ValidationResult -TestName "Binary Execution" -Status "WARN" -Message $_.Exception.Message
                }

                return $true
            } else {
                Write-Status "Binary not found after build" -Status "FAIL"
                Add-ValidationResult -TestName "Binary Creation" -Status "FAIL" -Message "Binary file not found"
                return $false
            }
        } else {
            Write-Status "Application build failed" -Status "FAIL"
            Add-ValidationResult -TestName "Application Build" -Status "FAIL" -Message $buildOutput
            return $false
        }
    } catch {
        Write-Status "Build process exception: $($_.Exception.Message)" -Status "FAIL"
        Add-ValidationResult -TestName "Application Build" -Status "FAIL" -Message $_.Exception.Message
        return $false
    }
}

function Test-MSICreation {
    Write-Host "`n=== Testing MSI Creation ===" -ForegroundColor Blue

    # Extract version if not provided
    if (-not $Version) {
        $cargoToml = Get-Content (Join-Path $ProjectRoot "Cargo.toml") -Raw
        if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
            $Version = $Matches[1]
        } else {
            $Version = "1.0.0"
        }
    }

    # Test using the validation script we created
    $msiTestScript = Join-Path $ScriptDir "test-msi-fix.ps1"
    if (Test-Path $msiTestScript) {
        try {
            Write-Status "Running MSI configuration test..." -Status "INFO"
            & $msiTestScript -Version $Version -TestBoth

            if ($LASTEXITCODE -eq 0) {
                Write-Status "MSI configuration test passed" -Status "PASS"
                Add-ValidationResult -TestName "MSI Configuration" -Status "PASS" -Message "Both minimal and enhanced MSI configs work"
                return $true
            } else {
                Write-Status "MSI configuration test failed" -Status "FAIL"
                Add-ValidationResult -TestName "MSI Configuration" -Status "FAIL" -Message "MSI test script failed"
                return $false
            }
        } catch {
            Write-Status "MSI test script exception: $($_.Exception.Message)" -Status "FAIL"
            Add-ValidationResult -TestName "MSI Configuration" -Status "FAIL" -Message $_.Exception.Message
            return $false
        }
    } else {
        Write-Status "MSI test script not found - using simple build script" -Status "WARN"

        # Fallback to simple build script
        $simpleBuildScript = Join-Path $ScriptDir "build-windows-simple.ps1"
        if (Test-Path $simpleBuildScript) {
            try {
                & $simpleBuildScript -Version $Version
                if ($LASTEXITCODE -eq 0) {
                    Write-Status "Simple MSI build successful" -Status "PASS"
                    Add-ValidationResult -TestName "MSI Creation" -Status "PASS" -Message "Simple build script succeeded"
                    return $true
                } else {
                    Write-Status "Simple MSI build failed" -Status "FAIL"
                    Add-ValidationResult -TestName "MSI Creation" -Status "FAIL" -Message "Simple build script failed"
                    return $false
                }
            } catch {
                Write-Status "Simple build script exception: $($_.Exception.Message)" -Status "FAIL"
                Add-ValidationResult -TestName "MSI Creation" -Status "FAIL" -Message $_.Exception.Message
                return $false
            }
        } else {
            Write-Status "No MSI build scripts found" -Status "FAIL"
            Add-ValidationResult -TestName "MSI Creation" -Status "FAIL" -Message "No MSI build scripts available"
            return $false
        }
    }
}

function Test-InstallationProcess {
    Write-Host "`n=== Testing Installation Process ===" -ForegroundColor Blue

    if (-not $TestInstallation) {
        Write-Status "Installation testing skipped (use -TestInstallation to enable)" -Status "INFO"
        Add-ValidationResult -TestName "Installation Test" -Status "SKIP" -Message "Skipped by user request"
        return $true
    }

    # Find the MSI file
    $msiFiles = Get-ChildItem "$ProjectRoot\target" -Filter "ZipLock-*.msi" -ErrorAction SilentlyContinue
    if ($msiFiles.Count -eq 0) {
        Write-Status "No MSI files found for installation testing" -Status "FAIL"
        Add-ValidationResult -TestName "Installation Test" -Status "FAIL" -Message "No MSI files found"
        return $false
    }

    $msiFile = $msiFiles[0].FullName
    Write-Status "Testing installation of: $($msiFiles[0].Name)" -Status "INFO"

    # Check if running as administrator
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

    if (-not $isAdmin) {
        Write-Status "Installation test requires administrator privileges" -Status "WARN"
        Add-ValidationResult -TestName "Installation Test" -Status "WARN" -Message "Admin privileges required"
        Write-Host "To test installation, run as administrator:" -ForegroundColor Yellow
        Write-Host "  msiexec /i `"$msiFile`" /qn /l*v install-test.log" -ForegroundColor Gray
        return $true
    }

    try {
        # Test installation in silent mode with logging
        $installLog = Join-Path $ValidationDir "install-test.log"
        Write-Status "Performing silent installation..." -Status "INFO"

        & msiexec /i "`"$msiFile`"" /qn /l*v "`"$installLog`""

        if ($LASTEXITCODE -eq 0) {
            Write-Status "Installation completed successfully" -Status "PASS"
            Add-ValidationResult -TestName "Installation Test" -Status "PASS" -Message "MSI installation succeeded"

            # Test uninstallation
            Write-Status "Testing uninstallation..." -Status "INFO"
            $uninstallLog = Join-Path $ValidationDir "uninstall-test.log"
            & msiexec /x "`"$msiFile`"" /qn /l*v "`"$uninstallLog`""

            if ($LASTEXITCODE -eq 0) {
                Write-Status "Uninstallation completed successfully" -Status "PASS"
                Add-ValidationResult -TestName "Uninstallation Test" -Status "PASS" -Message "MSI uninstallation succeeded"
                return $true
            } else {
                Write-Status "Uninstallation failed (exit code: $LASTEXITCODE)" -Status "FAIL"
                Add-ValidationResult -TestName "Uninstallation Test" -Status "FAIL" -Message "Exit code: $LASTEXITCODE"
                return $false
            }
        } else {
            Write-Status "Installation failed (exit code: $LASTEXITCODE)" -Status "FAIL"
            Add-ValidationResult -TestName "Installation Test" -Status "FAIL" -Message "Exit code: $LASTEXITCODE"

            # Check for error 2762 specifically
            if (Test-Path $installLog) {
                $logContent = Get-Content $installLog -Raw
                if ($logContent -match "2762") {
                    Write-Status "ERROR 2762 DETECTED - MSI fix did not work!" -Status "FAIL"
                    Add-ValidationResult -TestName "Error 2762 Check" -Status "FAIL" -Message "Error 2762 still occurring"
                }
            }
            return $false
        }
    } catch {
        Write-Status "Installation test exception: $($_.Exception.Message)" -Status "FAIL"
        Add-ValidationResult -TestName "Installation Test" -Status "FAIL" -Message $_.Exception.Message
        return $false
    }
}

function Generate-ValidationReport {
    Write-Host "`n" -NoNewline
    Write-Host "=" * 60 -ForegroundColor Cyan
    Write-Host "WINDOWS BUILD VALIDATION REPORT" -ForegroundColor Green
    Write-Host "=" * 60 -ForegroundColor Cyan

    # Summary statistics
    $totalTests = $script:ValidationResults.Count
    $passedTests = ($script:ValidationResults | Where-Object { $_.Status -eq "PASS" }).Count
    $failedTests = ($script:ValidationResults | Where-Object { $_.Status -eq "FAIL" }).Count
    $warnedTests = ($script:ValidationResults | Where-Object { $_.Status -eq "WARN" }).Count
    $skippedTests = ($script:ValidationResults | Where-Object { $_.Status -eq "SKIP" }).Count

    Write-Host "`nSummary:" -ForegroundColor Blue
    Write-Host "  Total Tests: $totalTests" -ForegroundColor Cyan
    Write-Host "  Passed: $passedTests" -ForegroundColor Green
    Write-Host "  Failed: $failedTests" -ForegroundColor Red
    Write-Host "  Warnings: $warnedTests" -ForegroundColor Yellow
    Write-Host "  Skipped: $skippedTests" -ForegroundColor Gray

    # Detailed results
    Write-Host "`nDetailed Results:" -ForegroundColor Blue
    foreach ($result in $script:ValidationResults) {
        $status = switch ($result.Status) {
            "PASS" { "✅" }
            "FAIL" { "❌" }
            "WARN" { "⚠️" }
            "SKIP" { "⏭️" }
            default { "ℹ️" }
        }

        Write-Host "  $status $($result.TestName): $($result.Message)" -ForegroundColor Gray
        if ($result.Details) {
            Write-Host "     Details: $($result.Details)" -ForegroundColor DarkGray
        }
    }

    # Overall result
    Write-Host "`nOverall Result:" -ForegroundColor Blue
    if ($failedTests -eq 0) {
        Write-Host "🎉 ALL CRITICAL TESTS PASSED!" -ForegroundColor Green
        Write-Host "The Windows build process is working correctly." -ForegroundColor Green
        if ($warnedTests -gt 0) {
            Write-Host "Note: $warnedTests warnings were found - review above for details." -ForegroundColor Yellow
        }
        return $true
    } else {
        Write-Host "❌ $failedTests CRITICAL TESTS FAILED!" -ForegroundColor Red
        Write-Host "The Windows build process has issues that need to be resolved." -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    # Create validation directory
    if (!(Test-Path $ValidationDir)) {
        New-Item -ItemType Directory -Path $ValidationDir -Force | Out-Null
    }

    Write-Host "Validation Directory: $ValidationDir" -ForegroundColor Cyan
    Write-Host "Clean Build: $CleanBuild" -ForegroundColor Cyan
    Write-Host "Test Installation: $TestInstallation" -ForegroundColor Cyan

    $allTestsPassed = $true

    # Run validation tests
    if (-not (Test-BuildEnvironment)) { $allTestsPassed = $false }
    if (-not (Test-ProjectStructure)) { $allTestsPassed = $false }
    if (-not (Test-ApplicationBuild)) { $allTestsPassed = $false }
    if (-not (Test-MSICreation)) { $allTestsPassed = $false }
    if (-not (Test-InstallationProcess)) { $allTestsPassed = $false }

    # Generate final report
    $reportResult = Generate-ValidationReport

    # Save detailed results
    $resultFile = Join-Path $ValidationDir "validation-results.json"
    $script:ValidationResults | ConvertTo-Json -Depth 3 | Out-File $resultFile -Encoding UTF8
    Write-Host "`nDetailed results saved to: $resultFile" -ForegroundColor Cyan

    # Exit with appropriate code
    if ($reportResult -and $allTestsPassed) {
        Write-Host "`n🚀 Windows build validation completed successfully!" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "`n⚠️ Windows build validation completed with issues." -ForegroundColor Yellow
        exit 1
    }
}
catch {
    Write-Host "`n❌ Validation script failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
finally {
    # Return to project root
    if ($ProjectRoot -and (Test-Path $ProjectRoot)) {
        Set-Location $ProjectRoot
    }
}
