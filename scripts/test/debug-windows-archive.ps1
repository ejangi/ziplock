# ZipLock Windows Archive Diagnostic Script
# PowerShell version for debugging the metadata/credential mismatch issue

param(
    [Parameter(Mandatory=$true)]
    [string]$ArchivePath,
    [string]$Password = ""
)

# Colors for output
$colors = @{
    Info = "Cyan"
    Success = "Green"
    Warning = "Yellow"
    Error = "Red"
    Header = "Magenta"
}

function Write-Header {
    param([string]$Text)
    Write-Host ""
    Write-Host ("=" * 60) -ForegroundColor $colors.Header
    Write-Host " $Text" -ForegroundColor $colors.Header
    Write-Host ("=" * 60) -ForegroundColor $colors.Header
}

function Write-Info {
    param([string]$Text)
    Write-Host "ℹ️  $Text" -ForegroundColor $colors.Info
}

function Write-Success {
    param([string]$Text)
    Write-Host "✅ $Text" -ForegroundColor $colors.Success
}

function Write-Warning {
    param([string]$Text)
    Write-Host "⚠️  $Text" -ForegroundColor $colors.Warning
}

function Write-Error {
    param([string]$Text)
    Write-Host "❌ $Text" -ForegroundColor $colors.Error
}

function Test-7ZipAvailable {
    $possible_paths = @(
        "7z",
        "7za",
        "7zz",
        "C:\Program Files\7-Zip\7z.exe",
        "C:\Program Files (x86)\7-Zip\7z.exe"
    )

    foreach ($path in $possible_paths) {
        try {
            $result = & $path 2>$null
            if ($LASTEXITCODE -eq 0 -or $LASTEXITCODE -eq 9) {
                Write-Success "Found 7z executable: $path"
                return $path
            }
        } catch {
            continue
        }
    }

    return $null
}

function Extract-Archive {
    param(
        [string]$ArchivePath,
        [string]$ExtractDir,
        [string]$Password,
        [string]$SevenZipPath
    )

    $args = @("x", $ArchivePath, "-o$ExtractDir", "-y")
    if ($Password) {
        $args += "-p$Password"
    }

    Write-Info "Extracting archive: $ArchivePath"
    Write-Info "Extract directory: $ExtractDir"

    try {
        $result = & $SevenZipPath $args 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Archive extracted successfully"
            return $true
        } else {
            Write-Error "Failed to extract archive (exit code: $LASTEXITCODE)"
            Write-Error "Output: $result"
            return $false
        }
    } catch {
        Write-Error "Failed to run 7z: $_"
        return $false
    }
}

function Get-DirectoryListing {
    param(
        [string]$Path,
        [string]$Prefix = ""
    )

    $items = @()
    try {
        $children = Get-ChildItem -Path $Path | Sort-Object Name
        foreach ($item in $children) {
            $relativePath = $item.Name
            if ($item.PSIsContainer) {
                $items += "$Prefix" + "DIR: $relativePath/"
                $items += Get-DirectoryListing -Path $item.FullName -Prefix "$Prefix  "
            } else {
                $size = $item.Length
                $items += "$Prefix" + "FILE: $relativePath ($size bytes)"
            }
        }
    } catch {
        $items += "$Prefix" + "ERROR: Error reading directory: $($_.Exception.Message)"
    }

    return $items
}

function Read-YamlContent {
    param([string]$FilePath)

    if (-not (Test-Path $FilePath)) {
        return $null
    }

    try {
        $content = Get-Content -Path $FilePath -Raw -Encoding UTF8
        return $content
    } catch {
        Write-Error "Failed to read file $FilePath`: $($_.Exception.Message)"
        return $null
    }
}

function Parse-SimpleYaml {
    param([string]$Content)

    $result = @{}
    $lines = $Content -split "`n"

    foreach ($line in $lines) {
        $line = $line.Trim()
        if ($line -and -not $line.StartsWith("#")) {
            if ($line -match "^([^:]+):(.*)$") {
                $key = $matches[1].Trim()
                $value = $matches[2].Trim()

                # Handle quoted strings
                if ($value -match '^"(.*)"$' -or $value -match "^'(.*)'$") {
                    $value = $matches[1]
                }

                # Try to convert numbers
                if ($value -match "^\d+$") {
                    $value = [int]$value
                }

                $result[$key] = $value
            }
        }
    }

    return $result
}

function Examine-Metadata {
    param([string]$ExtractDir)

    $metadataPath = Join-Path $ExtractDir "metadata.yml"

    if (-not (Test-Path $metadataPath)) {
        Write-Error "metadata.yml file not found in archive"
        return $null
    }

    $fileInfo = Get-Item $metadataPath
    Write-Success "Found metadata.yml ($($fileInfo.Length) bytes total)"

    $content = Read-YamlContent -FilePath $metadataPath
    if (-not $content) {
        return $null
    }

    Write-Info "Metadata file content:"
    Write-Host ("-" * 40)
    Write-Host $content
    Write-Host ("-" * 40)

    $metadata = Parse-SimpleYaml -Content $content
    Write-Info "Parsed metadata:"
    foreach ($key in $metadata.Keys) {
        Write-Host "  ${key}: $($metadata[$key])"
    }

    return $metadata
}

function Examine-Credentials {
    param(
        [string]$ExtractDir,
        [int]$ExpectedCount
    )

    $credentialsDir = Join-Path $ExtractDir "credentials"

    if (-not (Test-Path $credentialsDir)) {
        Write-Error "credentials/ directory not found in archive"
        Write-Warning "Expected $ExpectedCount credentials but credentials directory is missing"
        return @()
    }

    Write-Success "Found credentials/ directory"

    # List credential folders
    $credentialFolders = @()
    try {
        $items = Get-ChildItem -Path $credentialsDir
        foreach ($item in $items) {
            if ($item.PSIsContainer) {
                $credentialFolders += $item
            } else {
                Write-Warning "Unexpected file in credentials directory: $($item.Name)"
            }
        }
    } catch {
        Write-Error "Failed to read credentials directory: $_"
        return @()
    }

    Write-Info "Found $($credentialFolders.Count) credential folders"
    foreach ($folder in $credentialFolders) {
        Write-Host "  DIR: $($folder.Name)/"
    }

    # Examine each credential folder
    $credentials = @()
    foreach ($folder in $credentialFolders) {
        $recordFile = Join-Path $folder.FullName "record.yml"
        if (Test-Path $recordFile) {
            $fileInfo = Get-Item $recordFile
            Write-Success "Found record.yml in $($folder.Name)/ ($($fileInfo.Length) bytes total)"

            $content = Read-YamlContent -FilePath $recordFile
            if ($content) {
                $credential = Parse-SimpleYaml -Content $content
                $credentials += $credential

                Write-Info "Credential $($folder.Name):"
                $id = if ($credential.id) { $credential.id } else { 'MISSING' }
                $name = if ($credential.name) { $credential.name } else { 'MISSING' }
                $type = if ($credential.credential_type) { $credential.credential_type } else { 'MISSING' }
                Write-Host "    ID: $id"
                Write-Host "    Name: $name"
                Write-Host "    Type: $type"

                # Count fields (this is approximate since we're doing simple YAML parsing)
                $fieldCount = ($content -split "`n" | Where-Object { $_ -match "^\s+\w+:" }).Count
                Write-Host "    Fields: ~$fieldCount"
            }
        } else {
            Write-Error "Missing record.yml in $($folder.Name)/"
        }
    }

    return $credentials
}

function Diagnose-Mismatch {
    param(
        [hashtable]$Metadata,
        [array]$Credentials
    )

    Write-Header "DIAGNOSIS"

    $expectedCount = if ($Metadata.credential_count) { $Metadata.credential_count } else { 0 }
    $actualCount = $Credentials.Count

    Write-Host "Expected credentials (from metadata): $expectedCount"
    Write-Host "Actual credentials found: $actualCount"

    if ($expectedCount -eq $actualCount) {
        Write-Success "✅ Credential count matches - no mismatch detected"
        Write-Info "The issue might be in the ZipLock loading logic, not the archive format"
    } else {
        Write-Error "❌ MISMATCH DETECTED: Expected $expectedCount, found $actualCount"

        if ($actualCount -eq 0 -and $expectedCount -gt 0) {
            Write-Error "This is the exact issue reported: metadata claims credentials exist but none are found"
            Write-Info "Possible causes:"
            Write-Host "  1. Credential files were not written during archive creation"
            Write-Host "  2. Credential files were written with wrong paths/names"
            Write-Host "  3. 7z compression failed to include credential files"
            Write-Host "  4. Windows path handling issues during file creation"
        } elseif ($actualCount -gt $expectedCount) {
            Write-Warning "More credentials found than expected - metadata not updated properly"
        } else {
            Write-Warning "Some credentials missing: $($expectedCount - $actualCount) not found"
        }
    }
}

# Main script execution
Write-Header "ZipLock Windows Archive Diagnostic Tool"

# Validate inputs
if (-not (Test-Path $ArchivePath)) {
    Write-Error "Archive file not found: $ArchivePath"
    exit 1
}

$archiveInfo = Get-Item $ArchivePath
Write-Info "Archive: $ArchivePath"
Write-Info "Size: $($archiveInfo.Length) bytes"
Write-Info "Password: $(if ($Password) { '[PROVIDED]' } else { '[NONE]' })"

# Check if it's a 7z file
try {
    $bytes = [System.IO.File]::ReadAllBytes($ArchivePath)
    if ($bytes.Length -lt 6 -or $bytes[0] -ne 0x37 -or $bytes[1] -ne 0x7A) {
        Write-Error "File does not appear to be a 7z archive"
        $headerHex = ($bytes[0..5] | ForEach-Object { $_.ToString("X2") }) -join " "
        Write-Info "Header bytes: $headerHex"
        exit 1
    }
} catch {
    Write-Error "Failed to read archive file: $_"
    exit 1
}

Write-Success "File appears to be a valid 7z archive"

# Find 7z executable
$sevenZip = Test-7ZipAvailable
if (-not $sevenZip) {
    Write-Error "7z executable not found. Please install 7-Zip."
    Write-Info "Download from: https://www.7-zip.org/"
    exit 1
}

# Create temporary directory for extraction
$tempDir = Join-Path $env:TEMP "ziplock_debug_$(Get-Random)"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
Write-Info "Temporary extraction directory: $tempDir"

try {
    # Extract archive
    if (-not (Extract-Archive -ArchivePath $ArchivePath -ExtractDir $tempDir -Password $Password -SevenZipPath $sevenZip)) {
        Write-Error "Failed to extract archive"
        exit 1
    }

    # List extracted contents
    Write-Header "EXTRACTED CONTENTS"
    $contents = Get-DirectoryListing -Path $tempDir
    foreach ($item in $contents) {
        Write-Host $item
    }

    # Examine metadata
    Write-Header "METADATA EXAMINATION"
    $metadata = Examine-Metadata -ExtractDir $tempDir
    if (-not $metadata) {
        exit 1
    }

    # Examine credentials
    Write-Header "CREDENTIAL EXAMINATION"
    $expectedCount = if ($metadata.credential_count) { $metadata.credential_count } else { 0 }
    $credentials = Examine-Credentials -ExtractDir $tempDir -ExpectedCount $expectedCount

    # Diagnose the issue
    Diagnose-Mismatch -Metadata $metadata -Credentials $credentials

    # Final recommendations
    Write-Header "RECOMMENDATIONS"
    $metadataCount = if ($metadata.credential_count) { $metadata.credential_count } else { 0 }
    if ($metadataCount -gt $credentials.Count) {
        Write-Host "DEBUGGING STEPS:" -ForegroundColor Yellow
        Write-Host "1. Add detailed logging to DesktopFileProvider::create_archive()"
        Write-Host "2. Verify temp directory contents before 7z compression"
        Write-Host "3. Check if credential files are actually written to temp dir"
        Write-Host "4. Test with simpler paths (no nested directories)"
        Write-Host "5. Compare Linux vs Windows temp directory behavior"

        Write-Host "`nPOTENTIAL FIXES:" -ForegroundColor Yellow
        Write-Host "1. Use absolute paths when creating temp files"
        Write-Host "2. Add Windows-specific path normalization"
        Write-Host "3. Verify temp file permissions on Windows"
        Write-Host "4. Add file existence checks after each write operation"
        Write-Host "5. Use a flatter directory structure for Windows"
    } else {
        Write-Success "Archive structure appears correct"
        Write-Info "The issue might be in the ZipLock loading/parsing logic"
    }

} finally {
    # Cleanup
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Info "Cleaned up temporary directory"
    }
}
