# ZipLock Windows Icon Generation Script (Pure .NET)
# Converts PNG assets to .ico format using .NET System.Drawing

param(
    [string]$SourceDir = "",
    [string]$OutputDir = "",
    [switch]$Force = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))

# Default paths
if (-not $SourceDir) {
    $SourceDir = Join-Path $ProjectRoot "assets\icons"
}
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectRoot "packaging\windows\resources"
}

Write-Host "ZipLock Windows Icon Generation (.NET)" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host "Source Directory: $SourceDir" -ForegroundColor Cyan
Write-Host "Output Directory: $OutputDir" -ForegroundColor Cyan

# Load required assemblies
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

# Function to create .ico from PNG using .NET
function New-IcoFromPngDotNet {
    param(
        [string]$PngPath,
        [string]$IcoPath,
        [int[]]$Sizes = @(16, 32, 48, 64, 128, 256)
    )

    Write-Host "Creating $([System.IO.Path]::GetFileName($IcoPath)) from $([System.IO.Path]::GetFileName($PngPath))..." -ForegroundColor Cyan

    try {
        # Load original image
        $originalBitmap = [System.Drawing.Image]::FromFile($PngPath)

        # Create ICO file stream
        $icoStream = [System.IO.File]::Create($IcoPath)

        # ICO header (6 bytes)
        $icoHeader = @(0, 0, 1, 0, $Sizes.Count, 0)
        $icoStream.Write($icoHeader, 0, 6)

        # Calculate offset for image data (starts after header + directory entries)
        $imageDataOffset = 6 + ($Sizes.Count * 16)

        $imageData = @()

        foreach ($size in $Sizes) {
            Write-Host "  Generating ${size}x${size}..." -ForegroundColor Gray

            # Resize image
            $resizedBitmap = New-Object System.Drawing.Bitmap($size, $size)
            $graphics = [System.Drawing.Graphics]::FromImage($resizedBitmap)
            $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $graphics.DrawImage($originalBitmap, 0, 0, $size, $size)
            $graphics.Dispose()

            # Convert to PNG bytes for storage in ICO
            $memoryStream = New-Object System.IO.MemoryStream
            $resizedBitmap.Save($memoryStream, [System.Drawing.Imaging.ImageFormat]::Png)
            $pngBytes = $memoryStream.ToArray()
            $memoryStream.Dispose()
            $resizedBitmap.Dispose()

            # ICO directory entry (16 bytes)
            $width = if ($size -eq 256) { 0 } else { $size }
            $height = if ($size -eq 256) { 0 } else { $size }

            $directoryEntry = @(
                $width,           # Width (0 = 256)
                $height,          # Height (0 = 256)
                0,                # Color count (0 = no palette)
                0,                # Reserved
                1, 0,             # Color planes
                32, 0,            # Bits per pixel
                $pngBytes.Length -band 0xFF, ($pngBytes.Length -shr 8) -band 0xFF,
                ($pngBytes.Length -shr 16) -band 0xFF, ($pngBytes.Length -shr 24) -band 0xFF,  # Data size
                $imageDataOffset -band 0xFF, ($imageDataOffset -shr 8) -band 0xFF,
                ($imageDataOffset -shr 16) -band 0xFF, ($imageDataOffset -shr 24) -band 0xFF   # Data offset
            )

            # Write directory entry
            $icoStream.Write($directoryEntry, 0, 16)

            # Store image data for later writing
            $imageData += ,@{
                Data = $pngBytes
                Offset = $imageDataOffset
            }

            $imageDataOffset += $pngBytes.Length
        }

        # Write all image data
        foreach ($imgData in $imageData) {
            $icoStream.Write($imgData.Data, 0, $imgData.Data.Length)
        }

        $icoStream.Close()
        $originalBitmap.Dispose()

        # Verify file creation
        if (Test-Path $IcoPath) {
            $icoInfo = Get-Item $IcoPath
            Write-Host "  ✅ Created: $($icoInfo.Name) ($([math]::Round($icoInfo.Length / 1KB, 1)) KB)" -ForegroundColor Green
            return $true
        } else {
            Write-Host "  ❌ Failed to create ICO file" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "  ❌ Error: $_" -ForegroundColor Red
        return $false
    }
}

# Function to create simple ICO using Icon.FromHandle (alternative method)
function New-SimpleIco {
    param(
        [string]$PngPath,
        [string]$IcoPath,
        [int]$Size = 32
    )

    try {
        Write-Host "Creating simple ICO: $([System.IO.Path]::GetFileName($IcoPath))" -ForegroundColor Yellow

        # Load and resize PNG
        $originalBitmap = [System.Drawing.Image]::FromFile($PngPath)
        $resizedBitmap = New-Object System.Drawing.Bitmap($Size, $Size)
        $graphics = [System.Drawing.Graphics]::FromImage($resizedBitmap)
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.DrawImage($originalBitmap, 0, 0, $Size, $Size)
        $graphics.Dispose()

        # Convert to icon and save
        $iconHandle = $resizedBitmap.GetHicon()
        $icon = [System.Drawing.Icon]::FromHandle($iconHandle)

        $fileStream = [System.IO.File]::Create($IcoPath)
        $icon.Save($fileStream)
        $fileStream.Close()

        # Cleanup
        $icon.Dispose()
        $resizedBitmap.Dispose()
        $originalBitmap.Dispose()

        if (Test-Path $IcoPath) {
            $icoInfo = Get-Item $IcoPath
            Write-Host "  ✅ Simple ICO created: $($icoInfo.Name) ($([math]::Round($icoInfo.Length / 1KB, 1)) KB)" -ForegroundColor Green
            return $true
        }

        return $false
    }
    catch {
        Write-Host "  ❌ Simple ICO creation failed: $_" -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    # Verify source directory exists
    if (-not (Test-Path $SourceDir)) {
        Write-Error "Source directory not found: $SourceDir"
        exit 1
    }

    # Create output directory if needed
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
        Write-Host "Created output directory: $OutputDir" -ForegroundColor Yellow
    }

    # Icon configurations
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
            Sizes = @(16, 32, 48, 64)
            Description = "Small application icon"
        }
    )

    Write-Host "`nGenerating ICO files..." -ForegroundColor Blue

    $GeneratedCount = 0
    foreach ($Config in $IconConfigs) {
        $SourcePath = Join-Path $SourceDir $Config.Source
        $OutputPath = Join-Path $OutputDir $Config.Output

        if (-not (Test-Path $SourcePath)) {
            Write-Warning "Source file not found: $SourcePath"
            continue
        }

        # Check if output exists and Force not specified
        if ((Test-Path $OutputPath) -and -not $Force) {
            Write-Host "Skipping $($Config.Output) (exists, use -Force to overwrite)" -ForegroundColor Yellow
            continue
        }

        Write-Host "`n$($Config.Description):" -ForegroundColor Magenta

        # Try advanced method first, fallback to simple method
        $success = New-IcoFromPngDotNet -PngPath $SourcePath -IcoPath $OutputPath -Sizes $Config.Sizes

        if (-not $success) {
            Write-Host "  Advanced method failed, trying simple method..." -ForegroundColor Yellow
            $success = New-SimpleIco -PngPath $SourcePath -IcoPath $OutputPath -Size 32
        }

        if ($success) {
            $GeneratedCount++
        }
    }

    # Summary
    Write-Host "`n=== Icon Generation Summary ===" -ForegroundColor Green
    Write-Host "Generated $GeneratedCount ICO files" -ForegroundColor Cyan
    Write-Host "Output directory: $OutputDir" -ForegroundColor Cyan

    # List generated files
    $IcoFiles = Get-ChildItem -Path $OutputDir -Filter "*.ico"
    if ($IcoFiles.Count -gt 0) {
        Write-Host "`nGenerated files:" -ForegroundColor Blue
        foreach ($File in $IcoFiles) {
            Write-Host "  - $($File.Name) ($([math]::Round($File.Length / 1KB, 1)) KB)" -ForegroundColor Gray
        }
    }

    if ($GeneratedCount -gt 0) {
        Write-Host "`nNext steps:" -ForegroundColor Yellow
        Write-Host "1. Build.rs script will embed these icons automatically" -ForegroundColor White
        Write-Host "2. Update WiX installer configuration" -ForegroundColor White
        Write-Host "3. Build Windows executable with: cargo build --release --target x86_64-pc-windows-msvc" -ForegroundColor White

        Write-Host "`n✅ Icon generation completed successfully!" -ForegroundColor Green
    } else {
        Write-Host "`n⚠️ No icons were generated" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "`n❌ Icon generation failed: $_" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
