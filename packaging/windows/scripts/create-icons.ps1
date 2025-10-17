# ZipLock Windows Icon Generation Script
# Converts PNG assets to .ico format for Windows executable embedding

param(
    [string]$SourceDir = "",
    [string]$OutputDir = "",
    [switch]$Force = $false
)

# Script configuration
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))

# Default paths
if (-not $SourceDir) {
    $SourceDir = Join-Path $ProjectRoot "assets\icons"
}
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectRoot "packaging\windows\resources"
}

Write-Host "ZipLock Windows Icon Generation" -ForegroundColor Green
Write-Host "===============================" -ForegroundColor Green
Write-Host "Source Directory: $SourceDir" -ForegroundColor Cyan
Write-Host "Output Directory: $OutputDir" -ForegroundColor Cyan

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# Function to install ImageMagick if not present
function Install-ImageMagick {
    Write-Host "`nChecking for ImageMagick..." -ForegroundColor Blue

    if (Test-Command "magick") {
        Write-Host "ImageMagick found!" -ForegroundColor Green
        return $true
    }

    Write-Host "ImageMagick not found. Attempting to install..." -ForegroundColor Yellow

    # Try winget first
    if (Test-Command "winget") {
        try {
            Write-Host "Installing ImageMagick via winget..." -ForegroundColor Cyan
            & winget install --id ImageMagick.ImageMagick --silent --accept-package-agreements --accept-source-agreements

            # Refresh PATH
            $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")

            if (Test-Command "magick") {
                Write-Host "ImageMagick installed successfully!" -ForegroundColor Green
                return $true
            }
        }
        catch {
            Write-Host "Winget installation failed: $_" -ForegroundColor Yellow
        }
    }

    # Try chocolatey as fallback
    if (Test-Command "choco") {
        try {
            Write-Host "Installing ImageMagick via Chocolatey..." -ForegroundColor Cyan
            & choco install imagemagick -y

            # Refresh PATH
            $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")

            if (Test-Command "magick") {
                Write-Host "ImageMagick installed successfully!" -ForegroundColor Green
                return $true
            }
        }
        catch {
            Write-Host "Chocolatey installation failed: $_" -ForegroundColor Yellow
        }
    }

    Write-Host "Could not install ImageMagick automatically." -ForegroundColor Red
    Write-Host "Please install ImageMagick manually from: https://imagemagick.org/script/download.php#windows" -ForegroundColor Yellow
    Write-Host "Or install via winget: winget install ImageMagick.ImageMagick" -ForegroundColor Yellow
    Write-Host "Or install via chocolatey: choco install imagemagick" -ForegroundColor Yellow
    return $false
}

# Function to create .ico file from PNG
function New-IcoFromPng {
    param(
        [string]$PngPath,
        [string]$IcoPath,
        [int[]]$Sizes = @(16, 32, 48, 64, 128, 256)
    )

    Write-Host "Creating $IcoPath from $PngPath..." -ForegroundColor Cyan

    # Create temporary directory for resized images
    $TempDir = Join-Path $env:TEMP "ziplock-icon-temp-$(Get-Random)"
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

    try {
        # Generate different sizes
        $TempFiles = @()
        foreach ($Size in $Sizes) {
            $TempFile = Join-Path $TempDir "icon_$Size.png"
            $TempFiles += $TempFile

            Write-Host "  Generating ${Size}x${Size}..." -ForegroundColor Gray
            & magick "$PngPath" -resize "${Size}x${Size}" -background transparent -gravity center -extent "${Size}x${Size}" "$TempFile"

            if ($LASTEXITCODE -ne 0) {
                throw "Failed to resize to ${Size}x${Size}"
            }
        }

        # Create .ico file from all sizes
        Write-Host "  Combining into .ico file..." -ForegroundColor Gray
        $MagickArgs = @($TempFiles) + @("$IcoPath")
        & magick @MagickArgs

        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create .ico file"
        }

        # Verify the .ico file was created
        if (Test-Path $IcoPath) {
            $IcoInfo = Get-Item $IcoPath
            Write-Host "  ✅ Created: $($IcoInfo.Name) ($([math]::Round($IcoInfo.Length / 1KB, 1)) KB)" -ForegroundColor Green
        } else {
            throw ".ico file was not created"
        }
    }
    finally {
        # Cleanup temporary files
        if (Test-Path $TempDir) {
            Remove-Item $TempDir -Recurse -Force
        }
    }
}

# Main execution
try {
    # Verify source directory exists
    if (-not (Test-Path $SourceDir)) {
        Write-Error "Source directory not found: $SourceDir"
        exit 1
    }

    # Create output directory if it doesn't exist
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
        Write-Host "Created output directory: $OutputDir" -ForegroundColor Yellow
    }

    # Install ImageMagick if needed
    if (-not (Install-ImageMagick)) {
        exit 1
    }

    # Define source PNG files and their corresponding .ico outputs
    $IconConfigs = @(
        @{
            Source = "ziplock-icon-256.png"
            Output = "ziplock.ico"
            Sizes = @(16, 32, 48, 64, 128, 256)
            Description = "Main application icon"
        },
        @{
            Source = "ziplock-icon-128.png"
            Output = "ziplock-small.ico"
            Sizes = @(16, 32, 48, 64, 128)
            Description = "Small application icon"
        }
    )

    Write-Host "`nGenerating .ico files..." -ForegroundColor Blue

    $GeneratedCount = 0
    foreach ($Config in $IconConfigs) {
        $SourcePath = Join-Path $SourceDir $Config.Source
        $OutputPath = Join-Path $OutputDir $Config.Output

        if (-not (Test-Path $SourcePath)) {
            Write-Warning "Source file not found: $SourcePath"
            continue
        }

        # Check if output already exists and Force is not specified
        if ((Test-Path $OutputPath) -and -not $Force) {
            Write-Host "Skipping $($Config.Output) (already exists, use -Force to overwrite)" -ForegroundColor Yellow
            continue
        }

        Write-Host "`n$($Config.Description):" -ForegroundColor Magenta
        New-IcoFromPng -PngPath $SourcePath -IcoPath $OutputPath -Sizes $Config.Sizes
        $GeneratedCount++
    }

    Write-Host "`n=== Icon Generation Summary ===" -ForegroundColor Green
    Write-Host "Generated $GeneratedCount .ico files" -ForegroundColor Cyan
    Write-Host "Output directory: $OutputDir" -ForegroundColor Cyan

    # List generated files
    $IcoFiles = Get-ChildItem -Path $OutputDir -Filter "*.ico"
    if ($IcoFiles.Count -gt 0) {
        Write-Host "`nGenerated files:" -ForegroundColor Blue
        foreach ($File in $IcoFiles) {
            Write-Host "  - $($File.Name) ($([math]::Round($File.Length / 1KB, 1)) KB)" -ForegroundColor Gray
        }
    }

    Write-Host "`nNext steps:" -ForegroundColor Yellow
    Write-Host "1. Add build.rs script to embed icons in executable" -ForegroundColor White
    Write-Host "2. Update WiX installer to use .ico files" -ForegroundColor White
    Write-Host "3. Test Windows build with embedded icons" -ForegroundColor White

    Write-Host "`n✅ Icon generation completed successfully!" -ForegroundColor Green
}
catch {
    Write-Host "`n❌ Icon generation failed: $_" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
