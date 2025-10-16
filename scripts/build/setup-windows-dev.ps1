# ZipLock Windows Development Environment Setup
# This script sets up the complete development environment for building ZipLock on Windows

param(
    [switch]$InstallRust = $false,
    [switch]$InstallDotNet = $false,
    [switch]$InstallBuildTools = $false,
    [switch]$InstallAll = $false,
    [switch]$CheckOnly = $false
)

# Script configuration
$ErrorActionPreference = "Stop"
$VerbosePreference = "Continue"

Write-Host "ZipLock Windows Development Environment Setup" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Green

function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

function Test-VisualStudioBuildTools {
    # Check for Visual Studio Build Tools installation
    $vsWherePath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWherePath) {
        $vsInstances = & $vsWherePath -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
        return $vsInstances.Count -gt 0
    }

    # Alternative check for MSBuild and VC tools
    $msbuildPaths = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
        "${env:ProgramFiles}\MSBuild\Current\Bin\MSBuild.exe"
    )

    foreach ($path in $msbuildPaths) {
        if (Test-Path $path) {
            return $true
        }
    }

    return $false
}

function Check-Prerequisites {
    Write-Host "`n=== Checking Prerequisites ===" -ForegroundColor Blue

    $results = @{
        Rust = $false
        DotNet = $false
        BuildTools = $false
        Git = $false
    }

    # Check Git
    if (Test-Command "git") {
        $gitVersion = git --version
        Write-Host "✅ Git: $gitVersion" -ForegroundColor Green
        $results.Git = $true
    } else {
        Write-Host "❌ Git: Not installed" -ForegroundColor Red
        Write-Host "   Install from: https://git-scm.com/download/win" -ForegroundColor Yellow
    }

    # Check Rust
    $cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
    if ((Test-Command "cargo") -or (Test-Path $cargoPath)) {
        try {
            $cargoCmd = if (Test-Command "cargo") { "cargo" } else { $cargoPath }
            $cargoVersion = & $cargoCmd --version 2>$null
            Write-Host "✅ Rust: $cargoVersion" -ForegroundColor Green
            $results.Rust = $true

            # Check Rust targets
            $rustupCmd = if (Test-Command "rustup") { "rustup" } else { "$env:USERPROFILE\.cargo\bin\rustup.exe" }
            if (Test-Path $rustupCmd) {
                $targets = & $rustupCmd target list --installed 2>$null
                $hasMSVC = $targets -match "x86_64-pc-windows-msvc"
                $hasGNU = $targets -match "x86_64-pc-windows-gnu"

                Write-Host "   Targets:" -ForegroundColor Gray
                if ($hasMSVC) { Write-Host "   ✅ x86_64-pc-windows-msvc" -ForegroundColor Green }
                else { Write-Host "   ❌ x86_64-pc-windows-msvc" -ForegroundColor Red }

                if ($hasGNU) { Write-Host "   ✅ x86_64-pc-windows-gnu" -ForegroundColor Green }
                else { Write-Host "   ⚠️  x86_64-pc-windows-gnu" -ForegroundColor Yellow }
            }
        } catch {
            Write-Host "⚠️  Rust: Installed but not working properly" -ForegroundColor Yellow
        }
    } else {
        Write-Host "❌ Rust: Not installed" -ForegroundColor Red
        Write-Host "   Install from: https://rustup.rs/" -ForegroundColor Yellow
    }

    # Check .NET
    if (Test-Command "dotnet") {
        try {
            $dotnetVersion = dotnet --version 2>$null
            Write-Host "✅ .NET SDK: $dotnetVersion" -ForegroundColor Green
            $results.DotNet = $true

            # Check if WiX is installed
            $wixPath = "$env:USERPROFILE\.dotnet\tools\wix.exe"
            if ((Test-Command "wix") -or (Test-Path $wixPath)) {
                $wixCmd = if (Test-Command "wix") { "wix" } else { $wixPath }
                $wixVersion = & $wixCmd --version 2>$null
                Write-Host "   ✅ WiX: $wixVersion" -ForegroundColor Green
            } else {
                Write-Host "   ❌ WiX: Not installed" -ForegroundColor Red
            }
        } catch {
            Write-Host "⚠️  .NET SDK: Installed but not working properly" -ForegroundColor Yellow
        }
    } else {
        Write-Host "❌ .NET SDK: Not installed" -ForegroundColor Red
        Write-Host "   Install from: https://dotnet.microsoft.com/download" -ForegroundColor Yellow
    }

    # Check Visual Studio Build Tools
    if (Test-VisualStudioBuildTools) {
        Write-Host "✅ Visual Studio Build Tools: Installed" -ForegroundColor Green
        $results.BuildTools = $true
    } else {
        Write-Host "❌ Visual Studio Build Tools: Not installed" -ForegroundColor Red
        Write-Host "   Install from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Yellow
    }

    return $results
}

function Install-RustToolchain {
    Write-Host "`n=== Installing Rust Toolchain ===" -ForegroundColor Blue

    if (Test-Command "cargo") {
        Write-Host "Rust is already installed, updating..." -ForegroundColor Yellow
        rustup update
    } else {
        Write-Host "Downloading Rust installer..." -ForegroundColor Cyan
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupInstaller = "rustup-init.exe"

        try {
            Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupInstaller -UseBasicParsing
            Write-Host "Installing Rust..." -ForegroundColor Cyan
            & .\$rustupInstaller -y --default-toolchain stable --default-host x86_64-pc-windows-msvc

            # Update PATH for current session
            $cargoPath = "$env:USERPROFILE\.cargo\bin"
            if ($env:PATH -notlike "*$cargoPath*") {
                $env:PATH += ";$cargoPath"
            }

            Remove-Item $rustupInstaller -Force
            Write-Host "✅ Rust installed successfully!" -ForegroundColor Green
        } catch {
            Write-Host "❌ Failed to install Rust: $_" -ForegroundColor Red
            return $false
        }
    }

    # Install required targets
    Write-Host "Installing Rust targets..." -ForegroundColor Cyan
    $rustupCmd = if (Test-Command "rustup") { "rustup" } else { "$env:USERPROFILE\.cargo\bin\rustup.exe" }

    & $rustupCmd target add x86_64-pc-windows-msvc
    & $rustupCmd target add x86_64-pc-windows-gnu

    Write-Host "✅ Rust targets installed!" -ForegroundColor Green
    return $true
}

function Install-DotNetSDK {
    Write-Host "`n=== Installing .NET SDK ===" -ForegroundColor Blue

    if (Test-Command "dotnet") {
        Write-Host ".NET SDK is already installed." -ForegroundColor Yellow
    } else {
        Write-Host "Downloading .NET SDK installer..." -ForegroundColor Cyan

        # Get the latest .NET 8 installer URL
        $dotnetUrl = "https://dotnetcli.azureedge.net/dotnet/Sdk/release/8.0.4xx/dotnet-sdk-win-x64.exe"
        $dotnetInstaller = "dotnet-sdk-installer.exe"

        try {
            Invoke-WebRequest -Uri $dotnetUrl -OutFile $dotnetInstaller -UseBasicParsing
            Write-Host "Installing .NET SDK..." -ForegroundColor Cyan
            Start-Process -FilePath ".\$dotnetInstaller" -ArgumentList "/quiet" -Wait

            Remove-Item $dotnetInstaller -Force
            Write-Host "✅ .NET SDK installed successfully!" -ForegroundColor Green
        } catch {
            Write-Host "❌ Failed to install .NET SDK: $_" -ForegroundColor Red
            Write-Host "Please download and install manually from: https://dotnet.microsoft.com/download" -ForegroundColor Yellow
            return $false
        }
    }

    # Install WiX toolset
    Write-Host "Installing WiX toolset..." -ForegroundColor Cyan
    try {
        dotnet tool install --global wix --version 4.0.4
        Write-Host "✅ WiX toolset installed!" -ForegroundColor Green

        # Add WiX UI extension
        Write-Host "Adding WiX UI extension..." -ForegroundColor Cyan
        $wixPath = "$env:USERPROFILE\.dotnet\tools\wix.exe"
        if (Test-Path $wixPath) {
            & $wixPath extension add WixToolset.UI.wixext
            Write-Host "✅ WiX UI extension added!" -ForegroundColor Green
        }
    } catch {
        Write-Host "⚠️  WiX installation had issues, but continuing..." -ForegroundColor Yellow
    }

    return $true
}

function Install-VisualStudioBuildTools {
    Write-Host "`n=== Installing Visual Studio Build Tools ===" -ForegroundColor Blue

    if (Test-VisualStudioBuildTools) {
        Write-Host "Visual Studio Build Tools are already installed." -ForegroundColor Yellow
        return $true
    }

    Write-Host "Downloading Visual Studio Build Tools..." -ForegroundColor Cyan
    $buildToolsUrl = "https://aka.ms/vs/17/release/vs_buildtools.exe"
    $buildToolsInstaller = "vs_buildtools.exe"

    try {
        Invoke-WebRequest -Uri $buildToolsUrl -OutFile $buildToolsInstaller -UseBasicParsing

        Write-Host "Installing Visual Studio Build Tools (this may take several minutes)..." -ForegroundColor Cyan
        Write-Host "Components being installed:" -ForegroundColor Gray
        Write-Host "  - C++ build tools" -ForegroundColor Gray
        Write-Host "  - MSVC v143 compiler toolset" -ForegroundColor Gray
        Write-Host "  - Windows 10/11 SDK" -ForegroundColor Gray

        $installArgs = @(
            "--quiet",
            "--wait",
            "--add", "Microsoft.VisualStudio.Workload.VCTools",
            "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "--add", "Microsoft.VisualStudio.Component.Windows10SDK.20348"
        )

        Start-Process -FilePath ".\$buildToolsInstaller" -ArgumentList $installArgs -Wait

        Remove-Item $buildToolsInstaller -Force

        # Verify installation
        if (Test-VisualStudioBuildTools) {
            Write-Host "✅ Visual Studio Build Tools installed successfully!" -ForegroundColor Green
            return $true
        } else {
            Write-Host "⚠️  Installation completed but verification failed" -ForegroundColor Yellow
            Write-Host "You may need to restart your command prompt" -ForegroundColor Yellow
            return $false
        }
    } catch {
        Write-Host "❌ Failed to install Visual Studio Build Tools: $_" -ForegroundColor Red
        Write-Host "Please download and install manually from:" -ForegroundColor Yellow
        Write-Host "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Yellow
        return $false
    }
}

function Show-NextSteps {
    Write-Host "`n=== Next Steps ===" -ForegroundColor Blue
    Write-Host "1. Restart your PowerShell session to ensure PATH changes take effect" -ForegroundColor White
    Write-Host "2. Test the installation by running:" -ForegroundColor White
    Write-Host "   .\scripts\build\test-windows-msi.ps1" -ForegroundColor Cyan
    Write-Host "3. If tests pass, you can build ZipLock with:" -ForegroundColor White
    Write-Host "   cargo build --release" -ForegroundColor Cyan
    Write-Host "4. Create MSI installer with:" -ForegroundColor White
    Write-Host "   .\scripts\build\test-windows-msi.ps1 -CreateMsi" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "For troubleshooting, see:" -ForegroundColor White
    Write-Host "- docs/windows-msi-analysis.md" -ForegroundColor Cyan
    Write-Host "- docs/technical/build.md" -ForegroundColor Cyan
}

# Main execution
try {
    if ($CheckOnly) {
        Check-Prerequisites
        exit 0
    }

    $results = Check-Prerequisites

    if ($InstallAll) {
        $InstallRust = $true
        $InstallDotNet = $true
        $InstallBuildTools = $true
    }

    $success = $true

    if ($InstallRust -or (-not $results.Rust)) {
        if (-not (Install-RustToolchain)) {
            $success = $false
        }
    }

    if ($InstallDotNet -or (-not $results.DotNet)) {
        if (-not (Install-DotNetSDK)) {
            $success = $false
        }
    }

    if ($InstallBuildTools -or (-not $results.BuildTools)) {
        if (-not (Install-VisualStudioBuildTools)) {
            $success = $false
        }
    }

    Write-Host "`n=== Setup Summary ===" -ForegroundColor Green

    if ($success) {
        Write-Host "✅ Windows development environment setup completed!" -ForegroundColor Green
        Show-NextSteps
    } else {
        Write-Host "⚠️  Setup completed with some issues" -ForegroundColor Yellow
        Write-Host "Some components may need manual installation" -ForegroundColor Yellow
    }

    # Final verification
    Write-Host "`n=== Final Verification ===" -ForegroundColor Blue
    Check-Prerequisites

} catch {
    Write-Host "`n❌ Setup failed with error: $_" -ForegroundColor Red
    Write-Host "Please check the error messages above and try manual installation" -ForegroundColor Yellow
    exit 1
}

Write-Host "`nSetup script completed. Check the status messages above." -ForegroundColor Green
