# ZipLock MSI Installer Test Script
# Tests both enhanced and minimal MSI installers in isolated environments

param(
    [string]$MsiPath = "",
    [string]$Version = "",
    [switch]$TestBoth = $false,
    [switch]$Uninstall = $false,
    [switch]$Verbose = $false,
    [switch]$Silent = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $ScriptDir))

Write-Host "ZipLock MSI Installer Test" -ForegroundColor Green
Write-Host "===========================" -ForegroundColor Green

# Function to check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Function to find MSI files
function Find-MsiFiles {
    $targetDir = Join-Path $ProjectRoot "target"
    $msiFiles = @()

    if (Test-Path $targetDir) {
        $msiFiles = Get-ChildItem -Path $targetDir -Filter "ZipLock-*.msi" | Sort-Object LastWriteTime -Descending
    }

    return $msiFiles
}

# Function to test MSI installation
function Test-MsiInstallation {
    param(
        [string]$MsiFilePath,
        [bool]$SilentMode = $false
    )

    $msiFileName = Split-Path -Leaf $MsiFilePath
    Write-Host "Testing MSI: $msiFileName" -ForegroundColor Cyan

    try {
        # Check if ZipLock is already installed
        $installed = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }
        if ($installed) {
            Write-Host "ZipLock is already installed. Uninstalling first..." -ForegroundColor Yellow
            foreach ($product in $installed) {
                $product.Uninstall() | Out-Null
            }
            Start-Sleep -Seconds 3
        }

        # Install MSI
        Write-Host "Installing $msiFileName..." -ForegroundColor Cyan
        $installArgs = @("/i", "`"$MsiFilePath`"")

        if ($SilentMode) {
            $installArgs += @("/quiet", "/norestart")
        } else {
            $installArgs += @("/passive", "/norestart")
        }

        if ($Verbose) {
            $installArgs += @("/L*v", "msi-install.log")
        }

        $process = Start-Process -FilePath "msiexec.exe" -ArgumentList $installArgs -Wait -PassThru

        if ($process.ExitCode -eq 0) {
            Write-Host "✅ Installation completed successfully" -ForegroundColor Green

            # Verify installation
            Start-Sleep -Seconds 2
            $newInstall = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }
            if ($newInstall) {
                Write-Host "✅ ZipLock found in installed programs" -ForegroundColor Green
                Write-Host "   Name: $($newInstall.Name)" -ForegroundColor Gray
                Write-Host "   Version: $($newInstall.Version)" -ForegroundColor Gray
                Write-Host "   Install Location: $($newInstall.InstallLocation)" -ForegroundColor Gray
            } else {
                Write-Host "⚠️  ZipLock not found in installed programs (may take time to register)" -ForegroundColor Yellow
            }

            # Check if executable exists
            $programFiles = ${env:ProgramFiles}
            $possiblePaths = @(
                "$programFiles\ZipLock\bin\ziplock.exe",
                "$programFiles\ZipLock\ziplock.exe"
            )

            $exeFound = $false
            foreach ($path in $possiblePaths) {
                if (Test-Path $path) {
                    Write-Host "✅ Executable found: $path" -ForegroundColor Green
                    $exeInfo = Get-Item $path
                    Write-Host "   Size: $([math]::Round($exeInfo.Length / (1024*1024), 2)) MB" -ForegroundColor Gray
                    Write-Host "   Version: $($exeInfo.VersionInfo.FileVersion)" -ForegroundColor Gray
                    $exeFound = $true
                    break
                }
            }

            if (-not $exeFound) {
                Write-Host "❌ Executable not found in expected locations" -ForegroundColor Red
                return $false
            }

            # Check Start Menu shortcuts
            $startMenuPaths = @(
                "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\ZipLock\ZipLock Password Manager.lnk",
                "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\ZipLock\ZipLock Password Manager.lnk"
            )

            $shortcutFound = $false
            foreach ($shortcutPath in $startMenuPaths) {
                if (Test-Path $shortcutPath) {
                    Write-Host "✅ Start Menu shortcut found: $shortcutPath" -ForegroundColor Green
                    $shortcutFound = $true
                    break
                }
            }

            if (-not $shortcutFound) {
                Write-Host "⚠️  Start Menu shortcut not found" -ForegroundColor Yellow
            }

            return $true

        } else {
            Write-Host "❌ Installation failed with exit code: $($process.ExitCode)" -ForegroundColor Red

            if ($Verbose -and (Test-Path "msi-install.log")) {
                Write-Host "Last few lines of install log:" -ForegroundColor Gray
                Get-Content "msi-install.log" -Tail 10 | ForEach-Object { Write-Host "   $_" -ForegroundColor Gray }
            }

            return $false
        }

    } catch {
        Write-Host "❌ Installation test failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to test MSI uninstallation
function Test-MsiUninstallation {
    Write-Host "Testing uninstallation..." -ForegroundColor Cyan

    try {
        $installed = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }

        if ($installed) {
            Write-Host "Uninstalling ZipLock..." -ForegroundColor Cyan
            foreach ($product in $installed) {
                Write-Host "Removing: $($product.Name)" -ForegroundColor Gray
                $result = $product.Uninstall()
                if ($result.ReturnValue -eq 0) {
                    Write-Host "✅ Uninstalled successfully" -ForegroundColor Green
                } else {
                    Write-Host "⚠️  Uninstall returned code: $($result.ReturnValue)" -ForegroundColor Yellow
                }
            }

            # Verify uninstallation
            Start-Sleep -Seconds 2
            $stillInstalled = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }
            if (-not $stillInstalled) {
                Write-Host "✅ ZipLock successfully removed from installed programs" -ForegroundColor Green
            } else {
                Write-Host "⚠️  ZipLock still appears in installed programs" -ForegroundColor Yellow
            }

            return $true
        } else {
            Write-Host "ℹ️  ZipLock is not currently installed" -ForegroundColor Blue
            return $true
        }

    } catch {
        Write-Host "❌ Uninstallation test failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to show test summary
function Show-TestSummary {
    param(
        [hashtable]$Results
    )

    Write-Host ""
    Write-Host "Test Summary" -ForegroundColor Green
    Write-Host "============" -ForegroundColor Green

    foreach ($test in $Results.Keys) {
        $result = $Results[$test]
        $status = if ($result) { "PASSED" } else { "FAILED" }
        $color = if ($result) { "Green" } else { "Red" }
        Write-Host "$test : $status" -ForegroundColor $color
    }

    $totalTests = $Results.Count
    $passedTests = ($Results.Values | Where-Object { $_ }).Count
    $failedTests = $totalTests - $passedTests

    Write-Host ""
    Write-Host "Total: $totalTests, Passed: $passedTests, Failed: $failedTests" -ForegroundColor Cyan

    if ($failedTests -eq 0) {
        Write-Host "All tests passed! ✅" -ForegroundColor Green
        return $true
    } else {
        Write-Host "Some tests failed! ❌" -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    # Check administrator privileges
    if (-not (Test-Administrator)) {
        Write-Host "This script requires administrator privileges to install/uninstall MSI packages." -ForegroundColor Red
        Write-Host "Please run as Administrator." -ForegroundColor Red
        exit 1
    }

    # Handle uninstall-only mode
    if ($Uninstall) {
        $uninstallResult = Test-MsiUninstallation
        exit $(if ($uninstallResult) { 0 } else { 1 })
    }

    # Find MSI files to test
    $msiFiles = @()

    if ($MsiPath -and (Test-Path $MsiPath)) {
        $msiFiles = @(Get-Item $MsiPath)
    } else {
        $msiFiles = Find-MsiFiles

        if ($msiFiles.Count -eq 0) {
            Write-Host "No MSI files found in target directory." -ForegroundColor Red
            Write-Host "Please build the MSI first or specify -MsiPath parameter." -ForegroundColor Red
            exit 1
        }

        if (-not $TestBoth) {
            # Use the most recent MSI by default
            $msiFiles = @($msiFiles[0])
        }
    }

    Write-Host "Found MSI files to test:" -ForegroundColor Cyan
    foreach ($msi in $msiFiles) {
        $msiInfo = Get-Item $msi.FullName
        Write-Host "  $($msi.Name) ($([math]::Round($msiInfo.Length / (1024*1024), 2)) MB)" -ForegroundColor Gray
    }
    Write-Host ""

    # Run tests
    $testResults = @{}

    foreach ($msiFile in $msiFiles) {
        $testName = "Install-$($msiFile.Name)"
        Write-Host "Running test: $testName" -ForegroundColor Yellow
        Write-Host "=================================" -ForegroundColor Yellow

        $installResult = Test-MsiInstallation -MsiFilePath $msiFile.FullName -SilentMode $Silent
        $testResults[$testName] = $installResult

        if ($installResult) {
            Write-Host ""
            Write-Host "Testing uninstallation..." -ForegroundColor Yellow
            $uninstallResult = Test-MsiUninstallation
            $testResults["Uninstall-$($msiFile.Name)"] = $uninstallResult
        }

        Write-Host ""
    }

    # Show final results
    $allPassed = Show-TestSummary -Results $testResults

    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Yellow
    if ($allPassed) {
        Write-Host "1. All tests passed - MSI installer is working correctly" -ForegroundColor Green
        Write-Host "2. Test the application by running it manually" -ForegroundColor Gray
        Write-Host "3. Verify user feedback dialogs appear during installation (if using enhanced MSI)" -ForegroundColor Gray
    } else {
        Write-Host "1. Review failed tests above" -ForegroundColor Red
        Write-Host "2. Check MSI build process for issues" -ForegroundColor Gray
        Write-Host "3. Verify all required files are in staging directory" -ForegroundColor Gray
        Write-Host "4. Check Windows Event Log for installation errors" -ForegroundColor Gray
    }

    exit $(if ($allPassed) { 0 } else { 1 })

} catch {
    Write-Host ""
    Write-Host "Test script failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($Verbose) {
        Write-Host "Stack trace:" -ForegroundColor Gray
        Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    }
    exit 1
}
