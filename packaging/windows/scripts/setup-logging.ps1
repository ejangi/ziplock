# ZipLock Windows Logging Setup Script
# Sets up Event Viewer logging and creates log directories for ZipLock

param(
    [switch]$Install = $false,
    [switch]$Uninstall = $false,
    [switch]$Status = $false,
    [string]$LogPath = "$env:CommonAppData\ZipLock\logs"
)

# Script configuration
$ErrorActionPreference = "Stop"
$EventSource = "ZipLock"
$EventLog = "Application"

# Colors for output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Success { param([string]$Message) Write-ColorOutput $Message "Green" }
function Write-Info { param([string]$Message) Write-ColorOutput $Message "Cyan" }
function Write-Warning { param([string]$Message) Write-ColorOutput $Message "Yellow" }
function Write-Error { param([string]$Message) Write-ColorOutput $Message "Red" }

# Check if running as Administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Register Event Viewer source
function Install-EventViewerSource {
    Write-Info "Registering ZipLock with Windows Event Viewer..."

    if (-not (Test-Administrator)) {
        Write-Warning "Administrator privileges required to register Event Viewer source"
        Write-Info "Event logging will fall back to file logging only"
        return $false
    }

    try {
        if (-not ([System.Diagnostics.EventLog]::SourceExists($EventSource))) {
            [System.Diagnostics.EventLog]::CreateEventSource($EventSource, $EventLog)
            Write-Success "Event Viewer source '$EventSource' registered successfully"
            Write-Info "ZipLock logs will appear in: Event Viewer > Windows Logs > Application"
            return $true
        } else {
            Write-Info "Event Viewer source '$EventSource' already exists"
            return $true
        }
    } catch {
        Write-Warning "Failed to register Event Viewer source: $_"
        return $false
    }
}

# Remove Event Viewer source
function Uninstall-EventViewerSource {
    Write-Info "Removing ZipLock from Windows Event Viewer..."

    if (-not (Test-Administrator)) {
        Write-Warning "Administrator privileges required to remove Event Viewer source"
        return $false
    }

    try {
        if ([System.Diagnostics.EventLog]::SourceExists($EventSource)) {
            [System.Diagnostics.EventLog]::DeleteEventSource($EventSource)
            Write-Success "Event Viewer source '$EventSource' removed successfully"
            return $true
        } else {
            Write-Info "Event Viewer source '$EventSource' does not exist"
            return $true
        }
    } catch {
        Write-Warning "Failed to remove Event Viewer source: $_"
        return $false
    }
}

# Create log directories
function Initialize-LogDirectories {
    Write-Info "Setting up log directories..."

    try {
        if (-not (Test-Path $LogPath)) {
            New-Item -ItemType Directory -Path $LogPath -Force | Out-Null
            Write-Success "Created log directory: $LogPath"
        } else {
            Write-Info "Log directory already exists: $LogPath"
        }

        # Set appropriate permissions
        $acl = Get-Acl $LogPath
        $accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
            "Users", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow"
        )
        $acl.SetAccessRule($accessRule)
        Set-Acl -Path $LogPath -AclObject $acl
        Write-Success "Set log directory permissions"

        return $true
    } catch {
        Write-Warning "Failed to create log directory: $_"
        return $false
    }
}

# Remove log directories
function Remove-LogDirectories {
    Write-Info "Removing log directories..."

    try {
        if (Test-Path $LogPath) {
            Remove-Item -Path $LogPath -Recurse -Force
            Write-Success "Removed log directory: $LogPath"
        } else {
            Write-Info "Log directory does not exist: $LogPath"
        }
        return $true
    } catch {
        Write-Warning "Failed to remove log directory: $_"
        return $false
    }
}

# Show current status
function Show-LoggingStatus {
    Write-Info "ZipLock Logging Status"
    Write-Info "======================"

    # Event Viewer status
    $eventSourceExists = [System.Diagnostics.EventLog]::SourceExists($EventSource)
    if ($eventSourceExists) {
        Write-Success "Event Viewer: Registered (Source: $EventSource in $EventLog log)"
    } else {
        Write-Warning "Event Viewer: Not registered"
    }

    # Log directory status
    if (Test-Path $LogPath) {
        $logFiles = Get-ChildItem $LogPath -Filter "*.log" -ErrorAction SilentlyContinue
        Write-Success "Log Directory: $LogPath"
        Write-Info "  Status: Exists"
        Write-Info "  Log files: $($logFiles.Count)"
        if ($logFiles.Count -gt 0) {
            Write-Info "  Recent logs:"
            $logFiles | Sort-Object LastWriteTime -Descending | Select-Object -First 3 | ForEach-Object {
                Write-Info "    - $($_.Name) ($([math]::Round($_.Length / 1KB, 1)) KB, $($_.LastWriteTime))"
            }
        }
    } else {
        Write-Warning "Log Directory: Does not exist ($LogPath)"
    }

    # Registry settings
    Write-Info "`nRegistry Settings:"
    $regPath = "HKLM:\SOFTWARE\ZipLock"
    if (Test-Path $regPath) {
        try {
            $logPathReg = Get-ItemProperty -Path $regPath -Name "LogPath" -ErrorAction SilentlyContinue
            if ($logPathReg) {
                Write-Info "  LogPath: $($logPathReg.LogPath)"
            }
        } catch {
            Write-Info "  LogPath: Not set"
        }
    } else {
        Write-Info "  ZipLock registry key: Not found"
    }

    Write-Info "`nHow to View Logs:"
    Write-Info "=================="
    Write-Info "1. Event Viewer: eventvwr.msc > Windows Logs > Application > Filter by Source '$EventSource'"
    Write-Info "2. Log Files: $LogPath"
    Write-Info "3. PowerShell: Get-EventLog -LogName Application -Source '$EventSource' -Newest 10"
}

# Create logging configuration file
function Create-LoggingConfig {
    $configPath = "$LogPath\logging-config.json"
    Write-Info "Creating logging configuration: $configPath"

    $config = @{
        version = "1.0"
        logging = @{
            console = @{
                enabled = $true
                level = "INFO"
                format = "%Y-%m-%d %H:%M:%S [%level] %message"
            }
            file = @{
                enabled = $true
                level = "DEBUG"
                path = "$LogPath\ziplock.log"
                max_size = "10MB"
                max_files = 5
                format = "%Y-%m-%d %H:%M:%S.%3f [%level] [%thread] %message"
            }
            event_viewer = @{
                enabled = $eventSourceExists
                source = $EventSource
                log = $EventLog
                level = "WARN"
            }
        }
        paths = @{
            log_directory = $LogPath
            config_directory = "$env:CommonAppData\ZipLock"
        }
    } | ConvertTo-Json -Depth 4

    try {
        Set-Content -Path $configPath -Value $config -Encoding UTF8
        Write-Success "Created logging configuration file"
        return $true
    } catch {
        Write-Warning "Failed to create logging configuration: $_"
        return $false
    }
}

# Show usage information
function Show-Usage {
    Write-Info "ZipLock Windows Logging Setup"
    Write-Info "=============================="
    Write-Info ""
    Write-Info "This script sets up logging for ZipLock on Windows systems."
    Write-Info ""
    Write-Info "Usage:"
    Write-Info "  .\setup-logging.ps1 -Install    # Install logging (run as Administrator for Event Viewer)"
    Write-Info "  .\setup-logging.ps1 -Uninstall  # Remove logging setup"
    Write-Info "  .\setup-logging.ps1 -Status     # Show current logging status"
    Write-Info ""
    Write-Info "Parameters:"
    Write-Info "  -LogPath <path>  # Custom log directory (default: $env:CommonAppData\ZipLock\logs)"
    Write-Info ""
    Write-Info "Examples:"
    Write-Info "  .\setup-logging.ps1 -Install -LogPath 'C:\MyLogs\ZipLock'"
    Write-Info "  .\setup-logging.ps1 -Status"
    Write-Info ""
    Write-Info "Logging Locations:"
    Write-Info "  • Event Viewer: Windows Logs > Application (Source: ZipLock)"
    Write-Info "  • Log Files: $LogPath"
    Write-Info "  • Configuration: $LogPath\logging-config.json"
}

# Main execution
function Main {
    Write-Info "ZipLock Windows Logging Setup v1.0"
    Write-Info "===================================="

    # Show status if no parameters
    if (-not $Install -and -not $Uninstall -and -not $Status) {
        Show-Usage
        return
    }

    if ($Status) {
        Show-LoggingStatus
        return
    }

    if ($Install) {
        Write-Info "Installing ZipLock logging..."
        Write-Info "Log Path: $LogPath"
        Write-Info "Administrator: $(Test-Administrator)"
        Write-Info ""

        $success = $true

        # Create log directories
        if (-not (Initialize-LogDirectories)) {
            $success = $false
        }

        # Register Event Viewer source
        $eventSuccess = Install-EventViewerSource
        if (-not $eventSuccess) {
            Write-Info "Continuing without Event Viewer integration..."
        }

        # Create configuration
        if (-not (Create-LoggingConfig)) {
            $success = $false
        }

        if ($success) {
            Write-Success "`nLogging setup completed successfully!"
            Write-Info "Logs will be written to: $LogPath"
            if ($eventSuccess) {
                Write-Info "Event Viewer integration: Enabled"
            } else {
                Write-Info "Event Viewer integration: Disabled (run as Administrator to enable)"
            }
        } else {
            Write-Error "`nLogging setup completed with errors."
        }
    }

    if ($Uninstall) {
        Write-Info "Uninstalling ZipLock logging..."

        $success = $true

        # Remove Event Viewer source
        if (-not (Uninstall-EventViewerSource)) {
            $success = $false
        }

        # Ask before removing log files
        $response = Read-Host "Remove log files? (y/N)"
        if ($response -eq "y" -or $response -eq "Y") {
            if (-not (Remove-LogDirectories)) {
                $success = $false
            }
        }

        if ($success) {
            Write-Success "`nLogging uninstall completed successfully!"
        } else {
            Write-Error "`nLogging uninstall completed with errors."
        }
    }

    Write-Info "`nFor more information, run with -Status parameter"
}

# Execute main function
Main
