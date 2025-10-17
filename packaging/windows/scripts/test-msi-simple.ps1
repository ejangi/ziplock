# ZipLock Simple MSI Installer Test Script
# Simplified version for testing MSI installation functionality

param(
    [string]$MsiPath = "",
    [switch]$Uninstall = $false
)

$ErrorActionPreference = "Stop"

Write-Host "ZipLock Simple MSI Test" -ForegroundColor Green
Write-Host "========================" -ForegroundColor Green

# Function to check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Check administrator privileges
if (-not (Test-Administrator)) {
    Write-Host "This script requires administrator privileges to install/uninstall MSI packages." -ForegroundColor Red
    Write-Host "Please run as Administrator." -ForegroundColor Red
    exit 1
}

# Handle uninstall-only mode
if ($Uninstall) {
    Write-Host "Uninstalling ZipLock..." -ForegroundColor Cyan
    try {
        $installed = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }
        if ($installed) {
            foreach ($product in $installed) {
                Write-Host "Removing: $($product.Name)" -ForegroundColor Gray
                $result = $product.Uninstall()
                if ($result.ReturnValue -eq 0) {
                    Write-Host "✅ Uninstalled successfully" -ForegroundColor Green
                } else {
                    Write-Host "⚠️ Uninstall returned code: $($result.ReturnValue)" -ForegroundColor Yellow
                }
            }
        } else {
            Write-Host "ℹ️ ZipLock is not currently installed" -ForegroundColor Blue
        }
        exit 0
    } catch {
        Write-Host "❌ Uninstallation failed: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

# Find MSI file if not specified
if (-not $MsiPath -or -not (Test-Path $MsiPath)) {
    Write-Host "Looking for MSI files..." -ForegroundColor Cyan
    $targetDir = "target"
    if (Test-Path $targetDir) {
        $msiFiles = Get-ChildItem -Path $targetDir -Filter "ZipLock-*.msi" | Sort-Object LastWriteTime -Descending
        if ($msiFiles.Count -gt 0) {
            $MsiPath = $msiFiles[0].FullName
            Write-Host "Found MSI: $($msiFiles[0].Name)" -ForegroundColor Green
        } else {
            Write-Host "❌ No MSI files found in target directory" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "❌ Target directory not found" -ForegroundColor Red
        exit 1
    }
}

if (-not (Test-Path $MsiPath)) {
    Write-Host "❌ MSI file not found: $MsiPath" -ForegroundColor Red
    exit 1
}

$msiFileName = Split-Path -Leaf $MsiPath
$msiInfo = Get-Item $MsiPath
$msiSizeMB = [math]::Round($msiInfo.Length / (1024*1024), 2)

Write-Host ""
Write-Host "Testing MSI: $msiFileName ($msiSizeMB MB)" -ForegroundColor Cyan

try {
    # Check if ZipLock is already installed
    Write-Host "Checking for existing installation..." -ForegroundColor Gray
    $installed = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }
    if ($installed) {
        Write-Host "ZipLock is already installed. Uninstalling first..." -ForegroundColor Yellow
        foreach ($product in $installed) {
            $product.Uninstall() | Out-Null
        }
        Start-Sleep -Seconds 3
    }

    # Install MSI
    Write-Host "Installing MSI..." -ForegroundColor Cyan
    $installArgs = @("/i", "`"$MsiPath`"", "/quiet", "/norestart")

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
        } else {
            Write-Host "⚠️ ZipLock not found in installed programs (may take time to register)" -ForegroundColor Yellow
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
                $exeSizeMB = [math]::Round($exeInfo.Length / (1024*1024), 2)
Write-Host "   Size: $exeSizeMB MB" -ForegroundColor Gray
                Write-Host "   Version: $($exeInfo.VersionInfo.FileVersion)" -ForegroundColor Gray
                $exeFound = $true
                break
            }
        }

        if (-not $exeFound) {
            Write-Host "❌ Executable not found in expected locations" -ForegroundColor Red
            return $false
        }

        Write-Host ""
        Write-Host "Installation Test: PASSED ✅" -ForegroundColor Green

        # Test uninstallation
        Write-Host "Testing uninstallation..." -ForegroundColor Cyan
        $installed = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }
        if ($installed) {
            foreach ($product in $installed) {
                $result = $product.Uninstall()
                if ($result.ReturnValue -eq 0) {
                    Write-Host "✅ Uninstalled successfully" -ForegroundColor Green
                } else {
                    Write-Host "⚠️ Uninstall returned code: $($result.ReturnValue)" -ForegroundColor Yellow
                }
            }
        }

        Write-Host ""
        Write-Host "All tests completed successfully! ✅" -ForegroundColor Green

    } else {
        Write-Host "❌ Installation failed with exit code: $($process.ExitCode)" -ForegroundColor Red

        # Common exit codes
        switch ($process.ExitCode) {
            1602 { Write-Host "   User cancelled installation" -ForegroundColor Gray }
            1603 { Write-Host "   Fatal error during installation" -ForegroundColor Gray }
            1618 { Write-Host "   Another installation is in progress" -ForegroundColor Gray }
            1633 { Write-Host "   Platform not supported" -ForegroundColor Gray }
            default { Write-Host "   Unknown error code" -ForegroundColor Gray }
        }

        exit 1
    }

} catch {
    Write-Host "❌ Installation test failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "1. The enhanced MSI should have shown user feedback dialogs" -ForegroundColor Gray
Write-Host "2. Check Windows Event Log for installation events" -ForegroundColor Gray
Write-Host "3. Test the application manually to ensure it works correctly" -ForegroundColor Gray

exit 0
