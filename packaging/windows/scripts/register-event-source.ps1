# Register ZipLock as Windows Event Log source
# This script registers ZipLock as a valid source for the Windows Application Event Log
# Requires administrator privileges

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("install", "uninstall")]
    [string]$Action = "install",

    [Parameter(Mandatory=$false)]
    [string]$SourceName = "ZipLock",

    [Parameter(Mandatory=$false)]
    [string]$LogName = "Application"
)

# Check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Register event source
function Register-EventSource {
    param(
        [string]$Source,
        [string]$Log
    )

    try {
        Write-Host "Registering Event Log source '$Source' in '$Log' log..."

        # Check if source already exists
        if ([System.Diagnostics.EventLog]::SourceExists($Source)) {
            Write-Host "Event source '$Source' already exists" -ForegroundColor Yellow
            return $true
        }

        # Create the event source
        New-EventLog -LogName $Log -Source $Source -ErrorAction Stop
        Write-Host "Event source '$Source' registered successfully" -ForegroundColor Green

        # Test by writing a registration event
        Write-EventLog -LogName $Log -Source $Source -EventId 1000 -EntryType Information -Message "ZipLock Event Log source registered successfully"
        Write-Host "Test event written to verify registration" -ForegroundColor Green

        return $true
    }
    catch {
        Write-Error "Failed to register event source '$Source': $_"
        return $false
    }
}

# Unregister event source
function Unregister-EventSource {
    param(
        [string]$Source
    )

    try {
        Write-Host "Removing Event Log source '$Source'..."

        # Check if source exists
        if (-not [System.Diagnostics.EventLog]::SourceExists($Source)) {
            Write-Host "Event source '$Source' does not exist" -ForegroundColor Yellow
            return $true
        }

        # Remove the event source
        Remove-EventLog -Source $Source -ErrorAction Stop
        Write-Host "Event source '$Source' removed successfully" -ForegroundColor Green

        return $true
    }
    catch {
        Write-Error "Failed to remove event source '$Source': $_"
        return $false
    }
}

# Registry-based registration (alternative method)
function Register-EventSourceRegistry {
    param(
        [string]$Source,
        [string]$Log
    )

    try {
        Write-Host "Registering Event Log source via registry..."

        $registryPath = "HKLM:\SYSTEM\CurrentControlSet\Services\EventLog\$Log\$Source"

        # Create registry key if it doesn't exist
        if (-not (Test-Path $registryPath)) {
            New-Item -Path $registryPath -Force | Out-Null
            Write-Host "Created registry key: $registryPath"
        }

        # Set EventMessageFile (optional - uses system default)
        $eventMessageFile = "%SystemRoot%\System32\EventCreate.exe"
        Set-ItemProperty -Path $registryPath -Name "EventMessageFile" -Value $eventMessageFile -Type ExpandString

        # Set TypesSupported (supports all event types)
        Set-ItemProperty -Path $registryPath -Name "TypesSupported" -Value 7 -Type DWord

        Write-Host "Registry entries created successfully" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Error "Failed to create registry entries: $_"
        return $false
    }
}

# Unregister via registry
function Unregister-EventSourceRegistry {
    param(
        [string]$Source,
        [string]$Log
    )

    try {
        Write-Host "Removing Event Log source from registry..."

        $registryPath = "HKLM:\SYSTEM\CurrentControlSet\Services\EventLog\$Log\$Source"

        if (Test-Path $registryPath) {
            Remove-Item -Path $registryPath -Recurse -Force
            Write-Host "Registry key removed: $registryPath" -ForegroundColor Green
        } else {
            Write-Host "Registry key not found: $registryPath" -ForegroundColor Yellow
        }

        return $true
    }
    catch {
        Write-Error "Failed to remove registry entries: $_"
        return $false
    }
}

# Main execution
Write-Host "ZipLock Event Log Source Registration" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

# Check administrator privileges
if (-not (Test-Administrator)) {
    Write-Error "This script requires administrator privileges. Please run as administrator."
    exit 1
}

Write-Host "Action: $Action"
Write-Host "Source: $SourceName"
Write-Host "Log: $LogName"
Write-Host ""

$success = $false

if ($Action -eq "install") {
    # Try PowerShell method first
    Write-Host "Attempting registration using PowerShell cmdlets..."
    $success = Register-EventSource -Source $SourceName -Log $LogName

    # If PowerShell method fails, try registry method
    if (-not $success) {
        Write-Host "PowerShell method failed, trying registry method..." -ForegroundColor Yellow
        $success = Register-EventSourceRegistry -Source $SourceName -Log $LogName
    }

    if ($success) {
        Write-Host ""
        Write-Host "Event Log source registration completed successfully!" -ForegroundColor Green
        Write-Host "ZipLock can now write events to the Windows Event Log." -ForegroundColor Green
        Write-Host ""
        Write-Host "You can view events in Event Viewer under:" -ForegroundColor Cyan
        Write-Host "  Windows Logs -> Application" -ForegroundColor Cyan
        Write-Host "  Filter by Source: $SourceName" -ForegroundColor Cyan
    }
}
elseif ($Action -eq "uninstall") {
    # Try PowerShell method first
    Write-Host "Attempting removal using PowerShell cmdlets..."
    $success = Unregister-EventSource -Source $SourceName

    # Also try registry cleanup
    Write-Host "Cleaning up registry entries..."
    $registrySuccess = Unregister-EventSourceRegistry -Source $SourceName -Log $LogName
    $success = $success -or $registrySuccess

    if ($success) {
        Write-Host ""
        Write-Host "Event Log source removal completed!" -ForegroundColor Green
    }
}

# Exit with appropriate code
if ($success) {
    Write-Host "Operation completed successfully." -ForegroundColor Green
    exit 0
} else {
    Write-Error "Operation failed. Check the error messages above."
    exit 1
}
