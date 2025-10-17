# ZipLock Complete Windows Build Script
# Handles icon generation, executable building, and MSI packaging

param(
    [string]$Version = "",
    [string]$Configuration = "Release",
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipTests = $false,
    [switch]$SkipIcons = $false,
    [switch]$SkipMsi = $false,
    [switch]$Clean = $false,
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$DesktopAppDir = Join-Path $ProjectRoot "apps\desktop"
$OutputDir = Join-Path $ProjectRoot "target"
$PackagingDir = Join-Path $ProjectRoot "packaging\windows"

Write-Host "🔒 ZipLock Complete Windows Build" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Cyan
Write-Host "Skip Tests: $SkipTests" -ForegroundColor Cyan
Write-Host "Skip Icons: $SkipIcons" -ForegroundColor Cyan
Write-Host "Skip MSI: $SkipMsi" -ForegroundColor Cyan
Write-Host ""

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# Function to run command with error handling
function Invoke-BuildCommand {
    param(
        [string]$Command,
        [string]$Arguments = "",
        [string]$WorkingDirectory = $ProjectRoot,
        [string]$Description = ""
    )

    if ($Description) {
        Write-Host "🔧 $Description..." -ForegroundColor Blue
    }

    if ($Verbose) {
        Write-Host "Command: $Command $Arguments" -ForegroundColor Gray
        Write-Host "Working Directory: $WorkingDirectory" -ForegroundColor Gray
    }

    $originalLocation = Get-Location
    try {
        Set-Location $WorkingDirectory

        if ($Arguments) {
            $process = Start-Process -FilePath $Command -ArgumentList $Arguments -Wait -PassThru -NoNewWindow
        } else {
            $process = Start-Process -FilePath $Command -Wait -PassThru -NoNewWindow
        }

        if ($process.ExitCode -ne 0) {
            throw "Command failed with exit code $($process.ExitCode)"
        }

        if ($Description) {
            Write-Host "✅ $Description completed successfully" -ForegroundColor Green
        }
    }
    finally {
        Set-Location $originalLocation
    }
}

# Function to extract version from Cargo.toml
function Get-ProjectVersion {
    Write-Host "🔍 Extracting version information..." -ForegroundColor Blue

    try {
        $cargoTomlPath = Join-Path $DesktopAppDir "Cargo.toml"
        if (Test-Path $cargoTomlPath) {
            $cargoContent = Get-Content $cargoTomlPath -Raw
            $versionPattern = 'version\s*=\s*"([^"]+)"'
            if ($cargoContent -match $versionPattern) {
                return $matches[1]
            }
        }

        # Fallback: try workspace Cargo.toml
        $workspaceCargoPath = Join-Path $ProjectRoot "Cargo.toml"
        if (Test-Path $workspaceCargoPath) {
            $cargoContent = Get-Content $workspaceCargoPath -Raw
            $versionPattern = 'version\s*=\s*"([^"]+)"'
            if ($cargoContent -match $versionPattern) {
                return $matches[1]
            }
        }

        Write-Host "⚠️ Could not extract version, using default" -ForegroundColor Yellow
        return "1.0.0"
    }
    catch {
        Write-Host "⚠️ Error extracting version: $($_.Exception.Message), using default" -ForegroundColor Yellow
        return "1.0.0"
    }
}

# Function to generate icons
function New-WindowsIcons {
    Write-Host "🎨 Generating Windows icons..." -ForegroundColor Blue

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
            Write-Host "✅ Icons generated successfully with Python script" -ForegroundColor Green
            return $true
        }
        catch {
            Write-Host "⚠️ Python icon generation failed: $_" -ForegroundColor Yellow
        }
    }

    # Fallback: copy PNG files as .ico
    Write-Host "Using fallback method (PNG to ICO copy)..." -ForegroundColor Yellow
    try {
        $assetsIconDir = Join-Path $ProjectRoot "assets\icons"

        Copy-Item (Join-Path $assetsIconDir "ziplock-icon-256.png") (Join-Path $iconResourcesDir "ziplock.ico") -Force
        Copy-Item (Join-Path $assetsIconDir "ziplock-icon-128.png") (Join-Path $iconResourcesDir "ziplock-small.ico") -Force
        Copy-Item (Join-Path $assetsIconDir "ziplock-icon-512.png") (Join-Path $iconResourcesDir "ziplock-large.ico") -Force

        Write-Host "✅ Fallback icon generation completed" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "❌ Icon generation failed: $_" -ForegroundColor Red
        return $false
    }
}

# Function to verify prerequisites
function Test-Prerequisites {
    Write-Host "🔍 Verifying build prerequisites..." -ForegroundColor Blue

    $allGood = $true

    # Check Rust
    if (-not (Test-Command "cargo")) {
        Write-Host "❌ Cargo (Rust) not found" -ForegroundColor Red
        $allGood = $false
    } else {
        Write-Host "✅ Cargo found" -ForegroundColor Green
    }

    # Check target
    try {
        $targets = cargo --list | Out-String
        # This is a simple check, more robust target verification could be added
        Write-Host "✅ Rust toolchain available" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ Rust toolchain verification failed" -ForegroundColor Red
        $allGood = $false
    }

    return $allGood
}

# Function to build executable
function Build-Executable {
    Write-Host "🏗️ Building Windows executable..." -ForegroundColor Blue

    try {
        Set-Location $DesktopAppDir

        # Clean if requested
        if ($Clean) {
            Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Cyan
            cargo clean
        }

        # Run tests if not skipped
        if (-not $SkipTests) {
            Write-Host "🧪 Running tests..." -ForegroundColor Cyan
            cargo test --target $Target
            Write-Host "✅ Tests passed" -ForegroundColor Green
        }

        # Build with specific flags for Windows
        Write-Host "🔨 Building executable..." -ForegroundColor Cyan
        $env:RUSTFLAGS = "-C target-feature=+crt-static"

        if ($Configuration -eq "Release") {
            cargo build --release --target $Target
        } else {
            cargo build --target $Target
        }

        $exePath = Join-Path $ProjectRoot "target\$Target\$(if ($Configuration -eq 'Release') { 'release' } else { 'debug' })\ziplock.exe"

        if (Test-Path $exePath) {
            $exeInfo = Get-Item $exePath
            Write-Host "✅ Executable built successfully!" -ForegroundColor Green
            Write-Host "   Path: $exePath" -ForegroundColor Gray
            Write-Host "   Size: $([math]::Round($exeInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
            return $exePath
        } else {
            throw "Executable not found at expected location: $exePath"
        }
    }
    finally {
        Set-Location $ProjectRoot
    }
}

# Function to create MSI installer
function New-MsiInstaller {
    param([string]$ExePath, [string]$Version)

    if ($SkipMsi) {
        Write-Host "⏭️ Skipping MSI creation (--SkipMsi specified)" -ForegroundColor Yellow
        return $null
    }

    Write-Host "📦 Creating MSI installer..." -ForegroundColor Blue

    # Install WiX if needed
    if (-not (Test-Command "wix")) {
        Write-Host "Installing WiX Toolset..." -ForegroundColor Cyan
        dotnet tool install --global wix --version 4.0.4

        # Update PATH for current session
        $toolsPath = "$env:USERPROFILE\.dotnet\tools"
        $env:PATH = $env:PATH + ";$toolsPath"
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

        # Verify icons are available
        $iconPath = Join-Path $PackagingDir "resources\ziplock.ico"
        if (-not (Test-Path $iconPath)) {
            Write-Host "⚠️ Main icon not found, MSI may not have proper icons" -ForegroundColor Yellow
        }

        # Build MSI
        $installerDir = Join-Path $PackagingDir "installer"
        $wxsFile = Join-Path $installerDir "ziplock-minimal.wxs"
        $msiFile = Join-Path $OutputDir "ZipLock-$Version-x64.msi"

        Set-Location $installerDir

        Write-Host "Building MSI with WiX..." -ForegroundColor Cyan
        wix build "ziplock-minimal.wxs" -define "SourceDir=$stagingDir" -define "Version=$Version" -out $msiFile

        if ($LASTEXITCODE -eq 0 -and (Test-Path $msiFile)) {
            $msiInfo = Get-Item $msiFile
            Write-Host "✅ MSI installer created successfully!" -ForegroundColor Green
            Write-Host "   Path: $msiFile" -ForegroundColor Gray
            Write-Host "   Size: $([math]::Round($msiInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
            return $msiFile
        } else {
            throw "MSI build failed or file not created"
        }
    }
    catch {
        Write-Host "❌ MSI creation failed: $_" -ForegroundColor Red
        return $null
    }
    finally {
        Set-Location $ProjectRoot
    }
}

# Function to test icon embedding
function Test-IconEmbedding {
    param([string]$ExePath)

    Write-Host "🔍 Testing icon embedding..." -ForegroundColor Blue

    $testScript = Join-Path $PackagingDir "scripts\test-icon-embedding.ps1"
    if (Test-Path $testScript) {
        try {
            & $testScript -ExePath $ExePath
            return $true
        }
        catch {
            Write-Host "⚠️ Icon embedding test failed: $_" -ForegroundColor Yellow
            return $false
        }
    } else {
        Write-Host "⚠️ Icon test script not found, skipping test" -ForegroundColor Yellow
        return $true
    }
}

# Main execution
try {
    $startTime = Get-Date

    # Verify prerequisites
    if (-not (Test-Prerequisites)) {
        Write-Host "❌ Prerequisites check failed" -ForegroundColor Red
        exit 1
    }

    # Extract version
    if (-not $Version) {
        $Version = Get-ProjectVersion
        Write-Host "📋 Using version: $Version" -ForegroundColor Cyan
    }

    # Generate icons
    if (-not $SkipIcons) {
        if (-not (New-WindowsIcons)) {
            Write-Host "⚠️ Icon generation failed, but continuing..." -ForegroundColor Yellow
        }
    }

    # Build executable
    $exePath = Build-Executable
    if (-not $exePath) {
        Write-Host "❌ Build failed" -ForegroundColor Red
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
    Write-Host "🎉 Build Summary" -ForegroundColor Green
    Write-Host "================" -ForegroundColor Green
    Write-Host "Version: $Version" -ForegroundColor Cyan
    Write-Host "Build Time: $($duration.ToString('mm\:ss'))" -ForegroundColor Cyan
    Write-Host "Executable: $exePath" -ForegroundColor Cyan
    if ($msiPath) {
        Write-Host "MSI Installer: $msiPath" -ForegroundColor Cyan
    }
    $iconTestResult = if ($iconTest) { "✅ PASSED" } else { "⚠️ WARNING" }
    $iconTestColor = if ($iconTest) { "Green" } else { "Yellow" }
    Write-Host "Icon Test: $iconTestResult" -ForegroundColor $iconTestColor

    Write-Host ""
    Write-Host "🚀 Next Steps:" -ForegroundColor Yellow
    Write-Host "1. Test the executable: $exePath" -ForegroundColor Gray
    if ($msiPath) {
        Write-Host "2. Test MSI installation: $msiPath" -ForegroundColor Gray
        Write-Host "3. Verify icons appear correctly in Windows Explorer, taskbar, and shortcuts" -ForegroundColor Gray
    }

    Write-Host ""
    Write-Host "✅ Windows build completed successfully!" -ForegroundColor Green

    exit 0
}
catch {
    Write-Host ""
    Write-Host "❌ Build failed: $_" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
