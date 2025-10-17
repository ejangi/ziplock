# ZipLock Enhanced Windows Build Script
# Handles icon generation, executable building, and enhanced MSI packaging with user feedback

param(
    [string]$Version = "1.0.0",
    [string]$Configuration = "Release",
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipTests = $false,
    [switch]$SkipIcons = $false,
    [switch]$SkipMsi = $false,
    [switch]$Clean = $false,
    [switch]$Verbose = $false,
    [switch]$UseMinimal = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $ScriptDir))
$DesktopAppDir = Join-Path $ProjectRoot "apps\desktop"
$OutputDir = Join-Path $ProjectRoot "target"
$PackagingDir = Join-Path $ProjectRoot "packaging\windows"

Write-Host "ZipLock Enhanced Windows Build" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Green
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Cyan
if ($UseMinimal) {
    Write-Host "MSI Type: Minimal (no user feedback)" -ForegroundColor Yellow
} else {
    Write-Host "MSI Type: Enhanced (with user feedback)" -ForegroundColor Green
}
Write-Host ""

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

# Function to generate icons
function New-WindowsIcons {
    Write-Host "Generating Windows icons..." -ForegroundColor Blue

    $iconResourcesDir = Join-Path $PackagingDir "resources"

    # Ensure icon resources directory exists
    if (-not (Test-Path $iconResourcesDir)) {
        New-Item -ItemType Directory -Path $iconResourcesDir -Force | Out-Null
        Write-Host "Created icon resources directory: $iconResourcesDir" -ForegroundColor Yellow
    }

    # Try Python script first
    $pythonScript = Join-Path $PackagingDir "scripts\create-icons.py"
    if ((Test-Path $pythonScript) -and (Test-Command "python")) {
        try {
            Write-Host "Using Python script to generate proper .ico files..." -ForegroundColor Cyan
            python $pythonScript --force
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Icons generated successfully with Python script" -ForegroundColor Green
                return $true
            }
        }
        catch {
            Write-Host "Python icon generation failed: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    }

    # Fallback: copy PNG files as .ico
    Write-Host "Using fallback method (PNG to ICO copy)..." -ForegroundColor Yellow
    try {
        $assetsIconDir = Join-Path $ProjectRoot "assets\icons"

        if (Test-Path (Join-Path $assetsIconDir "ziplock-icon-256.png")) {
            Copy-Item (Join-Path $assetsIconDir "ziplock-icon-256.png") (Join-Path $iconResourcesDir "ziplock.ico") -Force
        }
        if (Test-Path (Join-Path $assetsIconDir "ziplock-icon-128.png")) {
            Copy-Item (Join-Path $assetsIconDir "ziplock-icon-128.png") (Join-Path $iconResourcesDir "ziplock-small.ico") -Force
        }
        if (Test-Path (Join-Path $assetsIconDir "ziplock-icon-512.png")) {
            Copy-Item (Join-Path $assetsIconDir "ziplock-icon-512.png") (Join-Path $iconResourcesDir "ziplock-large.ico") -Force
        }

        Write-Host "Fallback icon generation completed" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "Icon generation failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to verify prerequisites
function Test-Prerequisites {
    Write-Host "Verifying build prerequisites..." -ForegroundColor Blue

    $allGood = $true

    # Check Rust
    if (-not (Test-Command "cargo")) {
        Write-Host "Cargo (Rust) not found" -ForegroundColor Red
        $allGood = $false
    } else {
        Write-Host "Cargo found" -ForegroundColor Green
    }

    # Check PowerShell version for custom actions
    if (-not $UseMinimal) {
        $psVersion = $PSVersionTable.PSVersion.Major
        if ($psVersion -lt 5) {
            Write-Host "PowerShell 5.0 or later required for enhanced MSI features" -ForegroundColor Yellow
            Write-Host "Current version: $($PSVersionTable.PSVersion)" -ForegroundColor Gray
            Write-Host "Falling back to minimal MSI..." -ForegroundColor Yellow
            $script:UseMinimal = $true
        } else {
            Write-Host "PowerShell $psVersion detected - enhanced features available" -ForegroundColor Green
        }
    }

    return $allGood
}

# Function to build executable
function Build-Executable {
    Write-Host "Building Windows executable..." -ForegroundColor Blue

    try {
        Push-Location $DesktopAppDir

        # Clean if requested
        if ($Clean) {
            Write-Host "Cleaning previous builds..." -ForegroundColor Cyan
            cargo clean
        }

        # Run tests if not skipped
        if (-not $SkipTests) {
            Write-Host "Running tests..." -ForegroundColor Cyan
            cargo test --target $Target
            if ($LASTEXITCODE -ne 0) {
                throw "Tests failed"
            }
            Write-Host "Tests passed" -ForegroundColor Green
        }

        # Set environment variables for Windows build
        $env:RUSTFLAGS = "-C target-feature=+crt-static"

        # Build with specific configuration
        Write-Host "Building executable..." -ForegroundColor Cyan
        if ($Configuration -eq "Release") {
            cargo build --release --target $Target
        } else {
            cargo build --target $Target
        }

        if ($LASTEXITCODE -ne 0) {
            throw "Build failed with exit code $LASTEXITCODE"
        }

        $configPath = if ($Configuration -eq "Release") { "release" } else { "debug" }
        $exePath = Join-Path $ProjectRoot "target\$Target\$configPath\ziplock.exe"

        if (Test-Path $exePath) {
            $exeInfo = Get-Item $exePath
            Write-Host "Executable built successfully!" -ForegroundColor Green
            Write-Host "Path: $exePath" -ForegroundColor Gray
            Write-Host "Size: $([math]::Round($exeInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
            return $exePath
        } else {
            throw "Executable not found at expected location: $exePath"
        }
    }
    finally {
        Pop-Location
    }
}

# Function to create MSI installer
function New-MsiInstaller {
    param([string]$ExePath, [string]$Version)

    if ($SkipMsi) {
        Write-Host "Skipping MSI creation (--SkipMsi specified)" -ForegroundColor Yellow
        return $null
    }

    Write-Host "Creating MSI installer..." -ForegroundColor Blue

    # Install WiX if needed
    if (-not (Test-Command "wix")) {
        Write-Host "Installing WiX Toolset..." -ForegroundColor Cyan
        dotnet tool install --global wix --version 4.0.4

        # Update PATH for current session
        $toolsPath = "$env:USERPROFILE\.dotnet\tools"
        if ($env:PATH -notlike "*$toolsPath*") {
            $env:PATH = $env:PATH + ";$toolsPath"
        }
    }

    try {
        # Create staging directory
        $stagingDir = Join-Path $OutputDir "windows-package"
        if (Test-Path $stagingDir) {
            Remove-Item $stagingDir -Recurse -Force
        }
        New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null

        # Copy executable to staging
        Copy-Item $ExePath $stagingDir

        # Copy icon files to staging directory for WiX
        $iconResourcesDir = Join-Path $PackagingDir "resources"
        if (Test-Path $iconResourcesDir) {
            Write-Host "Copying icon files to staging directory..." -ForegroundColor Cyan
            $iconFiles = @("ziplock.ico", "ziplock-small.ico", "ziplock-large.ico")
            foreach ($iconFile in $iconFiles) {
                $iconPath = Join-Path $iconResourcesDir $iconFile
                if (Test-Path $iconPath) {
                    Copy-Item $iconPath $stagingDir
                    Write-Host "  Copied: $iconFile" -ForegroundColor Gray
                }
            }
        }

        # Copy custom action scripts if using enhanced MSI
        if (-not $UseMinimal) {
            Write-Host "Copying custom action scripts..." -ForegroundColor Cyan
            $scriptsDir = Join-Path $PackagingDir "scripts"
            $customActionScripts = @("show-install-success.ps1", "show-install-failure.ps1")
            foreach ($script in $customActionScripts) {
                $scriptPath = Join-Path $scriptsDir $script
                if (Test-Path $scriptPath) {
                    Copy-Item $scriptPath $stagingDir
                    Write-Host "  Copied: $script" -ForegroundColor Gray
                } else {
                    Write-Host "  Warning: Custom action script not found: $script" -ForegroundColor Yellow
                }
            }
        }

        # Determine which WiX configuration to use
        $wxsFile = if ($UseMinimal) { "ziplock-minimal.wxs" } else { "ziplock-enhanced.wxs" }
        $msiSuffix = if ($UseMinimal) { "minimal" } else { "enhanced" }

        # Verify icons are available
        $iconPath = Join-Path $PackagingDir "resources\ziplock.ico"
        if (-not (Test-Path $iconPath)) {
            Write-Host "Main icon not found, MSI may not have proper icons" -ForegroundColor Yellow
        }

        # Build MSI
        $installerDir = Join-Path $PackagingDir "installer"
        $wxsFilePath = Join-Path $installerDir $wxsFile
        $msiFile = Join-Path $OutputDir "ZipLock-$Version-x64-$msiSuffix.msi"

        if (-not (Test-Path $wxsFilePath)) {
            throw "WiX configuration file not found: $wxsFilePath"
        }

        Push-Location $installerDir

        Write-Host "Building MSI with WiX ($wxsFile)..." -ForegroundColor Cyan
        & wix build $wxsFile -define "SourceDir=$stagingDir" -define "Version=$Version" -out $msiFile

        if ($LASTEXITCODE -eq 0 -and (Test-Path $msiFile)) {
            $msiInfo = Get-Item $msiFile
            Write-Host "MSI installer created successfully!" -ForegroundColor Green
            Write-Host "Path: $msiFile" -ForegroundColor Gray
            Write-Host "Size: $([math]::Round($msiInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray

            if (-not $UseMinimal) {
                Write-Host "Features: Enhanced with user feedback dialogs" -ForegroundColor Green
            }

            return $msiFile
        } else {
            throw "MSI build failed with exit code $LASTEXITCODE"
        }
    }
    catch {
        Write-Host "MSI creation failed: $($_.Exception.Message)" -ForegroundColor Red

        # Fallback to minimal MSI if enhanced fails
        if (-not $UseMinimal) {
            Write-Host "Attempting fallback to minimal MSI..." -ForegroundColor Yellow
            $script:UseMinimal = $true
            return New-MsiInstaller $ExePath $Version
        }

        return $null
    }
    finally {
        Pop-Location
    }
}

# Function to test icon embedding
function Test-IconEmbedding {
    param([string]$ExePath)

    Write-Host "Testing icon embedding..." -ForegroundColor Blue

    try {
        # Load System.Drawing for icon extraction
        Add-Type -AssemblyName System.Drawing

        # Try to extract icon from executable
        $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($ExePath)

        if ($icon) {
            Write-Host "Icon extracted successfully!" -ForegroundColor Green
            Write-Host "Size: $($icon.Width)x$($icon.Height)" -ForegroundColor Gray
            $icon.Dispose()
            return $true
        } else {
            Write-Host "No icon could be extracted" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "Icon extraction failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to test custom action scripts
function Test-CustomActionScripts {
    if ($UseMinimal) {
        Write-Host "Skipping custom action tests (minimal MSI)" -ForegroundColor Gray
        return $true
    }

    Write-Host "Testing custom action scripts..." -ForegroundColor Blue

    $scriptsDir = Join-Path $PackagingDir "scripts"
    $scripts = @("show-install-success.ps1", "show-install-failure.ps1")
    $allGood = $true

    foreach ($script in $scripts) {
        $scriptPath = Join-Path $scriptsDir $script
        if (Test-Path $scriptPath) {
            Write-Host "  ✅ $script found" -ForegroundColor Green

            # Test script syntax
            try {
                $null = [System.Management.Automation.PSParser]::Tokenize((Get-Content $scriptPath -Raw), [ref]$null)
                Write-Host "    Syntax: OK" -ForegroundColor Gray
            }
            catch {
                Write-Host "    Syntax: ERROR - $($_.Exception.Message)" -ForegroundColor Red
                $allGood = $false
            }
        } else {
            Write-Host "  ❌ $script missing" -ForegroundColor Red
            $allGood = $false
        }
    }

    return $allGood
}

# Main execution
try {
    $startTime = Get-Date

    # Verify prerequisites
    if (-not (Test-Prerequisites)) {
        Write-Host "Prerequisites check failed" -ForegroundColor Red
        exit 1
    }

    # Generate icons
    if (-not $SkipIcons) {
        if (-not (New-WindowsIcons)) {
            Write-Host "Icon generation failed, but continuing..." -ForegroundColor Yellow
        }
    }

    # Test custom action scripts
    if (-not (Test-CustomActionScripts)) {
        Write-Host "Custom action scripts have issues, falling back to minimal MSI" -ForegroundColor Yellow
        $UseMinimal = $true
    }

    # Build executable
    $exePath = Build-Executable
    if (-not $exePath) {
        Write-Host "Build failed" -ForegroundColor Red
        exit 1
    }

    # Test icon embedding
    $iconTest = Test-IconEmbedding $exePath

    # Create MSI installer
    $msiPath = New-MsiInstaller $exePath $Version

    # Final summary
    $endTime = Get-Date
    $duration = $endTime - $startTime

    Write-Host ""
    Write-Host "Build Summary" -ForegroundColor Green
    Write-Host "=============" -ForegroundColor Green
    Write-Host "Version: $Version" -ForegroundColor Cyan
    Write-Host "Build Time: $($duration.ToString('mm\:ss'))" -ForegroundColor Cyan
    Write-Host "Executable: $exePath" -ForegroundColor Cyan
    if ($msiPath) {
        Write-Host "MSI Installer: $msiPath" -ForegroundColor Cyan
        $msiType = if ($UseMinimal) { "Minimal" } else { "Enhanced (with user feedback)" }
        Write-Host "MSI Type: $msiType" -ForegroundColor Cyan
    }

    $iconTestResult = if ($iconTest) { "PASSED" } else { "WARNING" }
    $iconTestColor = if ($iconTest) { "Green" } else { "Yellow" }
    Write-Host "Icon Test: $iconTestResult" -ForegroundColor $iconTestColor

    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "1. Test the executable: $exePath" -ForegroundColor Gray
    if ($msiPath) {
        Write-Host "2. Test MSI installation: $msiPath" -ForegroundColor Gray
        if (-not $UseMinimal) {
            Write-Host "3. Verify user feedback dialogs appear during installation" -ForegroundColor Gray
            Write-Host "4. Test both successful installation and rollback scenarios" -ForegroundColor Gray
        }
        Write-Host "5. Verify icons appear correctly in Windows Explorer, taskbar, and shortcuts" -ForegroundColor Gray
    }

    Write-Host ""
    Write-Host "Enhanced Windows build completed successfully!" -ForegroundColor Green

    exit 0
}
catch {
    Write-Host ""
    Write-Host "Build failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($Verbose) {
        Write-Host "Stack trace:" -ForegroundColor Gray
        Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    }
    exit 1
}
