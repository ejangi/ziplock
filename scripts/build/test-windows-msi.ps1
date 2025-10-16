# ZipLock Windows MSI Testing Script
# Tests MSI creation and installation locally on Windows

param(
    [string]$Configuration = "release",
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$UseGNU = $false,
    [switch]$Clean = $false,
    [switch]$SkipBuild = $false,
    [switch]$TestInstall = $false,
    [switch]$TestUninstall = $false,
    [switch]$Verbose = $false
)

# Script configuration
$ErrorActionPreference = "Stop"
$VerbosePreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# Path configuration
$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$PackagingDir = Join-Path $ProjectRoot "packaging\windows"

# Auto-select GNU toolchain if MSVC fails or UseGNU is specified
if ($UseGNU -or $Target -eq "auto") {
    $Target = "x86_64-pc-windows-gnu"
    Write-Host "Using GNU toolchain: $Target" -ForegroundColor Yellow
}

$BuildDir = Join-Path $ProjectRoot "target\$Target\$Configuration"
$OutputDir = Join-Path $ProjectRoot "target\windows-test"
$TestResultsFile = Join-Path $OutputDir "test-results.txt"

Write-Host "ZipLock Windows MSI Testing Script" -ForegroundColor Green
Write-Host "===================================" -ForegroundColor Green
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Cyan
Write-Host "Output Directory: $OutputDir" -ForegroundColor Cyan

# Initialize test results (using script-scoped variable)
$script:TestResults = @()

function Write-TestResult {
    param(
        [string]$Test,
        [string]$Status,
        [string]$Message = ""
    )

    $Result = "$Test`: $Status"
    if ($Message) {
        $Result += " - $Message"
    }

    $script:TestResults += $Result

    $Color = switch ($Status) {
        "PASS" { "Green" }
        "FAIL" { "Red" }
        "WARN" { "Yellow" }
        "SKIP" { "Gray" }
        default { "White" }
    }

    Write-Host "[$Status] $Test" -ForegroundColor $Color
    if ($Message) {
        Write-Host "    $Message" -ForegroundColor Gray
    }
}

function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

function Test-Prerequisites {
    Write-Host "`nTesting Prerequisites..." -ForegroundColor Blue

    # Add Rust to PATH if not already there
    $cargoPath = "$env:USERPROFILE\.cargo\bin"
    if ((Test-Path $cargoPath) -and ($env:PATH -notlike "*$cargoPath*")) {
        $env:PATH += ";$cargoPath"
        Write-Host "Added Rust to PATH: $cargoPath" -ForegroundColor Yellow
    }

    # Test Rust toolchain
    if (Test-Command "cargo") {
        try {
            $cargoVersion = cargo --version
            Write-TestResult "Cargo Available" "PASS" $cargoVersion
        }
        catch {
            Write-TestResult "Cargo Version Check" "FAIL" $_.Exception.Message
            return $false
        }
    }
    else {
        # Try direct path if command not found
        $cargoExe = "$env:USERPROFILE\.cargo\bin\cargo.exe"
        if (Test-Path $cargoExe) {
            try {
                $cargoVersion = & $cargoExe --version
                Write-TestResult "Cargo Available" "PASS" "$cargoVersion (direct path)"
                # Update aliases for the session
                Set-Alias -Name cargo -Value $cargoExe -Scope Script
                Set-Alias -Name rustup -Value "$env:USERPROFILE\.cargo\bin\rustup.exe" -Scope Script
            }
            catch {
                Write-TestResult "Cargo Version Check" "FAIL" $_.Exception.Message
                return $false
            }
        }
        else {
            Write-TestResult "Cargo Available" "FAIL" "Command not found"
            Write-Host "`nRust is not installed. Please install Rust from: https://rustup.rs/" -ForegroundColor Red
            Write-Host "After installation, restart PowerShell and run this script again." -ForegroundColor Yellow
            return $false
        }
    }

    # Test Rust target (only if cargo is available)
    $rustupCmd = if (Test-Command "rustup") { "rustup" } else { "$env:USERPROFILE\.cargo\bin\rustup.exe" }

    if ((Test-Command "rustup") -or (Test-Path "$env:USERPROFILE\.cargo\bin\rustup.exe")) {
        try {
            $installedTargets = & $rustupCmd target list --installed 2>$null
            if ($installedTargets -match $Target) {
                Write-TestResult "Rust Target $Target" "PASS" "Already installed"
            }
            else {
                Write-Host "Installing Rust target: $Target" -ForegroundColor Yellow
                & $rustupCmd target add $Target
                if ($LASTEXITCODE -eq 0) {
                    Write-TestResult "Rust Target $Target" "PASS" "Installed successfully"
                }
                else {
                    Write-TestResult "Rust Target $Target" "FAIL" "Installation failed"
                    return $false
                }
            }
        }
        catch {
            Write-TestResult "Rust Target Check" "FAIL" $_.Exception.Message
            return $false
        }
    }
    else {
        Write-TestResult "Rustup Available" "FAIL" "Command not found - Rust toolchain incomplete"
        return $false
    }

    # Test .NET (for WiX)
    if (Test-Command "dotnet") {
        try {
            $dotnetVersion = dotnet --version
            Write-TestResult ".NET Available" "PASS" "Version $dotnetVersion"
        }
        catch {
            Write-TestResult ".NET Version Check" "FAIL" $_.Exception.Message
        }
    }
    else {
        Write-TestResult ".NET Available" "WARN" "Command not found - MSI creation will be skipped"
        Write-Host "To install .NET SDK, visit: https://dotnet.microsoft.com/download" -ForegroundColor Yellow
    }

    # Test WiX toolset (only if .NET is available)
    if (Test-Command "dotnet") {
        if (Test-Command "wix") {
            try {
                $wixVersion = wix --version 2>$null
                Write-TestResult "WiX Toolset Available" "PASS" $wixVersion
            }
            catch {
                Write-Host "Installing WiX toolset..." -ForegroundColor Yellow
                try {
                    dotnet tool install --global wix --version 4.0.4
                    if ($LASTEXITCODE -eq 0) {
                        # Add .NET tools to PATH if not already there
                        $dotnetToolsPath = "$env:USERPROFILE\.dotnet\tools"
                        if ((Test-Path $dotnetToolsPath) -and ($env:PATH -notlike "*$dotnetToolsPath*")) {
                            $env:PATH += ";$dotnetToolsPath"
                        }
                        $wixVersion = wix --version 2>$null
                        Write-TestResult "WiX Toolset Installation" "PASS" $wixVersion
                    }
                    else {
                        Write-TestResult "WiX Toolset Installation" "WARN" "Installation failed - MSI creation will be skipped"
                    }
                }
                catch {
                    Write-TestResult "WiX Toolset Installation" "WARN" "Installation failed - MSI creation will be skipped"
                }
            }
        }
        else {
            Write-Host "Installing WiX toolset..." -ForegroundColor Yellow
            try {
                dotnet tool install --global wix --version 4.0.4
                if ($LASTEXITCODE -eq 0) {
                    # Update PATH for current session
                    $dotnetToolsPath = "$env:USERPROFILE\.dotnet\tools"
                    if ($env:PATH -notlike "*$dotnetToolsPath*") {
                        $env:PATH += ";$dotnetToolsPath"
                    }
                    $wixVersion = wix --version 2>$null
                    Write-TestResult "WiX Toolset Installation" "PASS" $wixVersion
                }
                else {
                    Write-TestResult "WiX Toolset Installation" "WARN" "Installation failed - MSI creation will be skipped"
                }
            }
            catch {
                Write-TestResult "WiX Toolset Installation" "WARN" "Installation failed - MSI creation will be skipped"
            }
        }
    }
    else {
        Write-TestResult "WiX Toolset" "SKIP" ".NET not available - MSI creation will be skipped"
    }

    # Test WiX UI Extension (only if WiX is available)
    if (Test-Command "wix") {
        try {
            $extensions = wix extension list 2>$null
            if ($extensions -match "WixToolset\.UI\.wixext" -and $extensions -notmatch "damaged") {
                Write-TestResult "WiX UI Extension" "PASS" "Already installed and working"
            }
            else {
                Write-Host "Installing WiX UI extension..." -ForegroundColor Yellow
                wix extension add WixToolset.UI.wixext 2>$null
                if ($LASTEXITCODE -eq 0) {
                    Write-TestResult "WiX UI Extension" "PASS" "Installed successfully"
                }
                else {
                    Write-TestResult "WiX UI Extension" "WARN" "Installation failed, will use fallback"
                }
            }
        }
        catch {
            Write-TestResult "WiX UI Extension Check" "WARN" "Check failed, will use fallback"
        }
    }
    else {
        Write-TestResult "WiX UI Extension" "SKIP" "WiX not available"
    }

    return $true
}

function Build-Application {
    if ($SkipBuild) {
        Write-Host "`nSkipping build (using existing binaries)..." -ForegroundColor Yellow
        Write-TestResult "Application Build" "SKIP" "Using existing binaries"
        return $true
    }

    Write-Host "`nBuilding Application..." -ForegroundColor Blue

    try {
        Set-Location $ProjectRoot

        # Determine cargo command
        $cargoCmd = if (Test-Command "cargo") { "cargo" } else { "$env:USERPROFILE\.cargo\bin\cargo.exe" }

        # Try building with current target first
        Write-Host "Building shared library with target: $Target" -ForegroundColor Cyan
        & $cargoCmd build --package ziplock-shared --target $Target --profile $Configuration
        if ($LASTEXITCODE -ne 0) {
            if ($Target -eq "x86_64-pc-windows-msvc") {
                Write-Host "MSVC build failed, trying GNU toolchain as fallback..." -ForegroundColor Yellow
                $script:Target = "x86_64-pc-windows-gnu"
                $script:BuildDir = Join-Path $ProjectRoot "target\$script:Target\$Configuration"

                Write-Host "Building shared library with GNU target: $script:Target" -ForegroundColor Cyan
                & $cargoCmd build --package ziplock-shared --target $script:Target --profile $Configuration
                if ($LASTEXITCODE -ne 0) {
                    Write-TestResult "Shared Library Build" "FAIL" "Both MSVC and GNU builds failed with exit code $LASTEXITCODE"
                    return $false
                }
                Write-TestResult "Shared Library Build" "PASS" "GNU fallback successful"
            } else {
                Write-TestResult "Shared Library Build" "FAIL" "Build failed with exit code $LASTEXITCODE"
                return $false
            }
        } else {
            Write-TestResult "Shared Library Build" "PASS"
        }

        # Build desktop application with the working target
        Write-Host "Building desktop application with target: $script:Target" -ForegroundColor Cyan
        & $cargoCmd build --package ziplock-desktop --bin ziplock --target $script:Target --profile $Configuration
        if ($LASTEXITCODE -ne 0) {
            Write-TestResult "Desktop Application Build" "FAIL" "Build failed with exit code $LASTEXITCODE"
            return $false
        }
        Write-TestResult "Desktop Application Build" "PASS"

        return $true
    }
    catch {
        Write-TestResult "Application Build" "FAIL" $_.Exception.Message
        return $false
    }
}

function Test-BinaryOutput {
    Write-Host "`nTesting Binary Output..." -ForegroundColor Blue

    $BinaryPath = Join-Path $script:BuildDir "ziplock.exe"

    if (!(Test-Path $BinaryPath)) {
        Write-TestResult "Binary Exists" "FAIL" "Not found at $BinaryPath"
        return $false
    }

    $BinaryInfo = Get-Item $BinaryPath
    Write-TestResult "Binary Exists" "PASS" "Size: $([math]::Round($BinaryInfo.Length / 1MB, 2)) MB"

    # Test binary execution
    try {
        $VersionOutput = & $BinaryPath --version 2>&1 | Out-String
        if ($LASTEXITCODE -eq 0) {
            Write-TestResult "Binary Execution" "PASS" $VersionOutput.Trim()
        }
        else {
            Write-TestResult "Binary Execution" "FAIL" "Exit code: $LASTEXITCODE, Output: $VersionOutput"
        }
    }
    catch {
        Write-TestResult "Binary Execution" "FAIL" $_.Exception.Message
    }

    # Test binary dependencies
    try {
        $Dependencies = dumpbin /dependents $BinaryPath 2>$null | Select-String "\.dll"
        $DepCount = ($Dependencies | Measure-Object).Count
        Write-TestResult "Binary Dependencies" "PASS" "$DepCount dependencies found"
        Write-Verbose "Dependencies: $($Dependencies -join ', ')"
    }
    catch {
        Write-TestResult "Binary Dependencies" "WARN" "Could not analyze dependencies"
    }

    return $true
}

function Prepare-PackageFiles {
    Write-Host "`nPreparing Package Files..." -ForegroundColor Blue

    # Clean output directory if requested
    if ($Clean -and (Test-Path $OutputDir)) {
        Remove-Item $OutputDir -Recurse -Force
        Write-TestResult "Clean Output Directory" "PASS"
    }

    # Create output directory
    if (!(Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
        Write-TestResult "Create Output Directory" "PASS"
    }

    # Copy binary
    $BinaryPath = Join-Path $script:BuildDir "ziplock.exe"
    $OutputBinary = Join-Path $OutputDir "ziplock.exe"

    try {
        Copy-Item $BinaryPath $OutputBinary -Force
        Write-TestResult "Copy Binary" "PASS" "Copied to $OutputBinary"
    }
    catch {
        Write-TestResult "Copy Binary" "FAIL" $_.Exception.Message
        return $false
    }

    # Download VC++ Redistributable
    $VCRedistUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    $VCRedistPath = Join-Path $OutputDir "vc_redist.x64.exe"

    try {
        Write-Host "Downloading VC++ Redistributable..." -ForegroundColor Cyan
        Invoke-WebRequest -Uri $VCRedistUrl -OutFile $VCRedistPath -UseBasicParsing -TimeoutSec 30
        $VCRedistInfo = Get-Item $VCRedistPath
        Write-TestResult "Download VC++ Redistributable" "PASS" "Size: $([math]::Round($VCRedistInfo.Length / 1MB, 2)) MB"
    }
    catch {
        Write-TestResult "Download VC++ Redistributable" "WARN" "Download failed: $($_.Exception.Message)"
        # Create dummy file to prevent MSI build failure
        Set-Content -Path $VCRedistPath -Value "Dummy VC++ Redistributable for testing"
        Write-TestResult "Create Dummy VC++ Redistributable" "PASS" "Created for testing purposes"
    }

    # Create license.rtf file (missing from current setup)
    $LicenseRtfPath = Join-Path $OutputDir "license.rtf"
    $LicenseMdPath = Join-Path $ProjectRoot "LICENSE.md"

    if (Test-Path $LicenseMdPath) {
        try {
            # Convert LICENSE.md to basic RTF format
            $licenseContent = Get-Content $LicenseMdPath -Raw
            $rtfContent = @"
{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}
\f0\fs24 $($licenseContent -replace "`n", "\par`n")
}
"@
            Set-Content -Path $LicenseRtfPath -Value $rtfContent -Encoding ASCII
            Write-TestResult "Create License RTF" "PASS" "Converted from LICENSE.md"
        }
        catch {
            Write-TestResult "Create License RTF" "FAIL" $_.Exception.Message
        }
    }
    else {
        # Create minimal license file
        $minimalLicense = @"
{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}
\f0\fs24 ZipLock Password Manager\par
\par
This software is provided as-is for testing purposes.\par
}
"@
        Set-Content -Path $LicenseRtfPath -Value $minimalLicense -Encoding ASCII
        Write-TestResult "Create License RTF" "PASS" "Created minimal license for testing"
    }

    return $true
}

function Test-MSICreation {
    Write-Host "`nTesting MSI Creation..." -ForegroundColor Blue

    $InstallerDir = Join-Path $PackagingDir "installer"
    $WxsFile = Join-Path $InstallerDir "ziplock.wxs"

    if (!(Test-Path $WxsFile)) {
        Write-TestResult "WXS File Exists" "FAIL" "Not found at $WxsFile"
        return $false
    }
    Write-TestResult "WXS File Exists" "PASS"

    # Test with UI extension first
    $Version = "1.0.0-test"
    $MsiPath = Join-Path $OutputDir "ZipLock-$Version-x64.msi"

    try {
        Set-Location $InstallerDir

        # Check if UI extension is available
        $extensions = wix extension list 2>$null
        $hasUIExtension = ($extensions -match "WixToolset\.UI\.wixext" -and $extensions -notmatch "damaged")

        if ($hasUIExtension) {
            Write-Host "Building MSI with UI extension..." -ForegroundColor Cyan
            wix build ziplock.wxs -ext WixToolset.UI.wixext -define "SourceDir=$OutputDir" -define "Version=$Version" -out $MsiPath -v 2>&1 | Write-Verbose

            if ($LASTEXITCODE -eq 0 -and (Test-Path $MsiPath)) {
                $MsiInfo = Get-Item $MsiPath
                Write-TestResult "MSI Creation (Full)" "PASS" "Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB"
                return $true
            }
            else {
                Write-TestResult "MSI Creation (Full)" "FAIL" "Build failed, trying fallback..."
            }
        }

        # Fallback: Create minimal MSI without UI extension
        Write-Host "Creating fallback MSI without UI extension..." -ForegroundColor Yellow

        $FallbackWxs = @'
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="ZipLock Password Manager"
           Language="1033"
           Version="$(var.Version)"
           Manufacturer="ZipLock Project"
           UpgradeCode="12345678-1234-1234-1234-123456789012"
           InstallerVersion="500"
           Compressed="yes"
           Scope="perMachine">

    <SummaryInformation Description="ZipLock Password Manager - Secure password management" />
    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <Media Id="1" Cabinet="media1.cab" EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="ZipLock Password Manager" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
      <ComponentRef Id="ApplicationShortcut" />
    </Feature>

    <StandardDirectory Id="ProgramFiles64Folder">
      <Directory Id="INSTALLFOLDER" Name="ZipLock">
        <Directory Id="BINDIR" Name="bin" />
      </Directory>
    </StandardDirectory>

    <StandardDirectory Id="ProgramMenuFolder">
      <Directory Id="ApplicationProgramsFolder" Name="ZipLock" />
    </StandardDirectory>

    <ComponentGroup Id="ProductComponents" Directory="BINDIR">
      <Component Id="ZipLockExecutable">
        <File Id="ZipLockExe" Source="$(var.SourceDir)\ziplock.exe" Checksum="yes" />
      </Component>
    </ComponentGroup>

    <Component Id="ApplicationShortcut" Directory="ApplicationProgramsFolder">
      <Shortcut Id="ApplicationStartMenuShortcut"
                Name="ZipLock Password Manager"
                Target="[#ZipLockExe]"
                WorkingDirectory="BINDIR"
                Description="Secure password management" />
      <RemoveFolder Id="CleanUpShortCut" Directory="ApplicationProgramsFolder" On="uninstall" />
      <RegistryValue Root="HKCU" Key="Software\ZipLock" Name="installed" Type="integer" Value="1" />
    </Component>

    <Property Id="ARPPRODUCTICON" Value="[#ZipLockExe]" />
    <Property Id="ARPNOREPAIR" Value="1" />
    <Property Id="ARPNOMODIFY" Value="1" />

  </Package>
</Wix>
'@

        $FallbackWxsPath = "ziplock-test.wxs"
        Set-Content $FallbackWxsPath $FallbackWxs -Encoding UTF8

        $FallbackMsiPath = Join-Path $OutputDir "ZipLock-$Version-minimal.msi"
        wix build $FallbackWxsPath -define "SourceDir=$OutputDir" -define "Version=$Version" -out $FallbackMsiPath -v 2>&1 | Write-Verbose

        if ($LASTEXITCODE -eq 0 -and (Test-Path $FallbackMsiPath)) {
            $MsiInfo = Get-Item $FallbackMsiPath
            Write-TestResult "MSI Creation (Minimal)" "PASS" "Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB"

            # Copy minimal MSI as the main output for testing
            Copy-Item $FallbackMsiPath $MsiPath -Force
            return $true
        }
        else {
            Write-TestResult "MSI Creation (Minimal)" "FAIL" "Fallback build also failed"
            return $false
        }
    }
    catch {
        Write-TestResult "MSI Creation" "FAIL" $_.Exception.Message
        return $false
    }
    finally {
        Set-Location $ProjectRoot
    }
}

function Test-MSIProperties {
    Write-Host "`nTesting MSI Properties..." -ForegroundColor Blue

    $Version = "1.0.0-test"
    $MsiPath = Join-Path $OutputDir "ZipLock-$Version-x64.msi"

    if (!(Test-Path $MsiPath)) {
        Write-TestResult "MSI File Exists" "FAIL" "MSI not found"
        return $false
    }

    $MsiInfo = Get-Item $MsiPath
    Write-TestResult "MSI File Exists" "PASS" "Size: $([math]::Round($MsiInfo.Length / 1MB, 2)) MB"

    # Test MSI properties using Windows Installer COM API
    try {
        $Installer = New-Object -ComObject WindowsInstaller.Installer
        $Database = $Installer.OpenDatabase($MsiPath, 0)

        # Test ProductName
        $View = $Database.OpenView("SELECT Value FROM Property WHERE Property='ProductName'")
        $View.Execute()
        $Record = $View.Fetch()
        if ($Record) {
            $ProductName = $Record.StringData(1)
            Write-TestResult "MSI Product Name" "PASS" $ProductName
        } else {
            Write-TestResult "MSI Product Name" "FAIL" "Property not found"
        }

        # Test ProductVersion
        $View = $Database.OpenView("SELECT Value FROM Property WHERE Property='ProductVersion'")
        $View.Execute()
        $Record = $View.Fetch()
        if ($Record) {
            $ProductVersion = $Record.StringData(1)
            Write-TestResult "MSI Product Version" "PASS" $ProductVersion
        } else {
            Write-TestResult "MSI Product Version" "WARN" "Property not found"
        }

        # Cleanup COM objects
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($Database) | Out-Null
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($Installer) | Out-Null

        return $true
    }
    catch {
        Write-TestResult "MSI Properties Check" "WARN" "Could not read MSI properties: $($_.Exception.Message)"
        return $true # Don't fail the entire test for this
    }
}

function Test-MSIInstallation {
    if (-not $TestInstall) {
        Write-TestResult "MSI Installation Test" "SKIP" "Not requested"
        return $true
    }

    Write-Host "`nTesting MSI Installation..." -ForegroundColor Blue
    Write-Warning "This will actually install ZipLock on your system!"

    $Version = "1.0.0-test"
    $MsiPath = Join-Path $OutputDir "ZipLock-$Version-x64.msi"

    if (!(Test-Path $MsiPath)) {
        Write-TestResult "MSI Installation Test" "FAIL" "MSI file not found"
        return $false
    }

    # Check if running as administrator
    $IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

    if (-not $IsAdmin) {
        Write-TestResult "MSI Installation Test" "WARN" "Not running as administrator, installation may fail"
    }

    try {
        Write-Host "Installing MSI (this may take a moment)..." -ForegroundColor Cyan
        $InstallProcess = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i", "`"$MsiPath`"", "/quiet", "/norestart" -Wait -PassThru

        if ($InstallProcess.ExitCode -eq 0) {
            Write-TestResult "MSI Installation" "PASS" "Installed successfully"

            # Test if executable exists in Program Files
            $InstallPath = "C:\Program Files\ZipLock\bin\ziplock.exe"
            if (Test-Path $InstallPath) {
                Write-TestResult "Installation Files Check" "PASS" "Executable found at $InstallPath"

                # Test installed executable
                try {
                    $InstalledVersion = & $InstallPath --version 2>&1
                    if ($LASTEXITCODE -eq 0) {
                        Write-TestResult "Installed Binary Test" "PASS" $InstalledVersion
                    } else {
                        Write-TestResult "Installed Binary Test" "FAIL" "Exit code: $LASTEXITCODE"
                    }
                } catch {
                    Write-TestResult "Installed Binary Test" "FAIL" $_.Exception.Message
                }
            } else {
                Write-TestResult "Installation Files Check" "FAIL" "Executable not found at expected location"
            }

            return $true
        } else {
            Write-TestResult "MSI Installation" "FAIL" "Exit code: $($InstallProcess.ExitCode)"
            return $false
        }
    }
    catch {
        Write-TestResult "MSI Installation" "FAIL" $_.Exception.Message
        return $false
    }
}

function Test-MSIUninstallation {
    if (-not $TestUninstall) {
        Write-TestResult "MSI Uninstallation Test" "SKIP" "Not requested"
        return $true
    }

    Write-Host "`nTesting MSI Uninstallation..." -ForegroundColor Blue

    try {
        # Find installed product by name
        $InstalledProducts = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*ZipLock*" }

        if ($InstalledProducts) {
            foreach ($Product in $InstalledProducts) {
                Write-Host "Uninstalling: $($Product.Name)" -ForegroundColor Cyan
                $UninstallResult = $Product.Uninstall()

                if ($UninstallResult.ReturnValue -eq 0) {
                    Write-TestResult "MSI Uninstallation" "PASS" "Product uninstalled: $($Product.Name)"
                } else {
                    Write-TestResult "MSI Uninstallation" "FAIL" "Uninstall failed with code: $($UninstallResult.ReturnValue)"
                }
            }

            # Verify files are removed
            $InstallPath = "C:\Program Files\ZipLock"
            if (Test-Path $InstallPath) {
                Write-TestResult "Uninstallation Cleanup" "WARN" "Installation directory still exists"
            } else {
                Write-TestResult "Uninstallation Cleanup" "PASS" "Installation directory removed"
            }

            return $true
        } else {
            Write-TestResult "MSI Uninstallation Test" "SKIP" "No ZipLock installation found"
            return $true
        }
    }
    catch {
        Write-TestResult "MSI Uninstallation" "FAIL" $_.Exception.Message
        return $false
    }
}

function Save-TestResults {
    Write-Host "`nSaving Test Results..." -ForegroundColor Blue

    try {
        $ResultsContent = @()
        $ResultsContent += "ZipLock Windows MSI Test Results"
        $ResultsContent += "Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
        $ResultsContent += "Target: $Target"
        $ResultsContent += "Configuration: $Configuration"
        $ResultsContent += ""
        $ResultsContent += "Test Results:"
        $ResultsContent += "============="
        $script:TestResults | ForEach-Object { $ResultsContent += $_ }

        Set-Content -Path $TestResultsFile -Value ($ResultsContent -join "`n") -Encoding UTF8
        Write-TestResult "Save Test Results" "PASS" "Saved to $TestResultsFile"
    }
    catch {
        Write-TestResult "Save Test Results" "FAIL" $_.Exception.Message
    }
}

# Main execution
Write-Host "`nStarting Windows MSI Test Suite..." -ForegroundColor Green

try {
    # Run all tests
    $success = $true

    $prereqsPassed = Test-Prerequisites
    $FailCount = ($script:TestResults | Where-Object { $_ -match ": FAIL" }).Count

    if ($FailCount -gt 0) {
        Write-Host "`nCritical prerequisites failed. Cannot continue with build tests." -ForegroundColor Red
        Write-Host "Please install missing dependencies and run again." -ForegroundColor Yellow
        Save-TestResults
        exit 1
    }

    # Check if we can do MSI tests
    $WarnCount = ($script:TestResults | Where-Object { $_ -match ": WARN" }).Count
    $CanCreateMSI = (Test-Command "wix") -and (Test-Command "dotnet")

    if ($WarnCount -gt 0 -and -not $CanCreateMSI) {
        Write-Host "`nSome tools are missing but basic build testing can continue." -ForegroundColor Yellow
        Write-Host "MSI creation tests will be skipped." -ForegroundColor Yellow
    }

    $success = $prereqsPassed -and $success
    $success = Build-Application -and $success
    $success = Test-BinaryOutput -and $success
    $success = Prepare-PackageFiles -and $success

    # Only run MSI tests if we have the tools
    if ($CanCreateMSI) {
        $success = Test-MSICreation -and $success
        $success = Test-MSIProperties -and $success
        $success = Test-MSIInstallation -and $success
        $success = Test-MSIUninstallation -and $success
    }
    else {
        Write-TestResult "MSI Creation Test" "SKIP" ".NET or WiX not available"
        Write-TestResult "MSI Properties Test" "SKIP" ".NET or WiX not available"
        Write-TestResult "MSI Installation Test" "SKIP" ".NET or WiX not available"
        Write-TestResult "MSI Uninstallation Test" "SKIP" ".NET or WiX not available"
    }

    # Save results
    Save-TestResults

    # Summary
    Write-Host ("="*50) -ForegroundColor Green
    Write-Host "TEST SUMMARY" -ForegroundColor Green
    Write-Host ("="*50) -ForegroundColor Green

    $PassCount = ($script:TestResults | Where-Object { $_ -match ": PASS" }).Count
    $FailCount = ($script:TestResults | Where-Object { $_ -match ": FAIL" }).Count
    $WarnCount = ($script:TestResults | Where-Object { $_ -match ": WARN" }).Count
    $SkipCount = ($script:TestResults | Where-Object { $_ -match ": SKIP" }).Count

    if ($script:TestResults.Count -eq 0) {
        Write-Host "No test results recorded - script may have failed early" -ForegroundColor Red
        exit 1
    }

    Write-Host "Passed: $PassCount" -ForegroundColor Green
    Write-Host "Failed: $FailCount" -ForegroundColor Red
    Write-Host "Warnings: $WarnCount" -ForegroundColor Yellow
    Write-Host "Skipped: $SkipCount" -ForegroundColor Gray
    Write-Host "Total: $($script:TestResults.Count)" -ForegroundColor Cyan

    Write-Host "`nOutput Directory: $OutputDir" -ForegroundColor Cyan
    Write-Host "Test Results: $TestResultsFile" -ForegroundColor Cyan

    if ($FailCount -eq 0 -and $PassCount -gt 0) {
        Write-Host "`nAll critical tests passed! ✅" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "`nSome tests failed! ❌" -ForegroundColor Red
        Write-Host "Check the output above for details." -ForegroundColor Yellow
        if ($FailCount -gt 0) {
            Write-Host "Failed tests ($FailCount) need to be addressed before MSI can be created successfully." -ForegroundColor Red
        }
        exit 1
    }
}
catch {
    Write-Host "`nUnexpected error during testing: $_" -ForegroundColor Red
    exit 1
}
finally {
    # Cleanup
    if (Test-Path "$ProjectRoot\packaging\windows\installer\ziplock-test.wxs") {
        Remove-Item "$ProjectRoot\packaging\windows\installer\ziplock-test.wxs" -ErrorAction SilentlyContinue
    }
}
