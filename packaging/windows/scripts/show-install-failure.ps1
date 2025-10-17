# ZipLock MSI Installer Failure Message
# Custom action script to display installation failure dialog

param(
    [string]$ProductName = "ZipLock Password Manager",
    [string]$Version = "",
    [string]$ErrorMessage = ""
)

$ErrorActionPreference = "SilentlyContinue"

try {
    # Load Windows Forms for message box
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing

    # Construct failure message
    $title = "Installation Failed"
    $message = "The installation of $ProductName was not completed successfully."

    if ($Version) {
        $message += "`n`nVersion: $Version"
    }

    if ($ErrorMessage) {
        $message += "`n`nError: $ErrorMessage"
    }

    $message += "`n`nPossible solutions:"
    $message += "`n• Run the installer as Administrator"
    $message += "`n• Ensure no antivirus software is blocking the installation"
    $message += "`n• Check available disk space"
    $message += "`n• Close any running instances of ZipLock"
    $message += "`n`nFor support, please visit: https://github.com/ejangi/ziplock"

    # Show error dialog with OK button and Error icon
    [System.Windows.Forms.MessageBox]::Show(
        $message,
        $title,
        [System.Windows.Forms.MessageBoxButtons]::OK,
        [System.Windows.Forms.MessageBoxIcon]::Error
    )

    # Log failure for debugging
    $logMessage = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'): ZipLock installation failed"
    if ($ErrorMessage) {
        $logMessage += " - Error: $ErrorMessage"
    }
    Write-Host $logMessage

    # Try to write to Windows event log (best effort)
    try {
        $eventMessage = "ZipLock Password Manager installation failed"
        if ($ErrorMessage) {
            $eventMessage += ": $ErrorMessage"
        }
        Write-EventLog -LogName "Application" -Source "ZipLock Installer" -EntryType Error -EventId 1002 -Message $eventMessage
    } catch {
        # Silently ignore event log errors
    }

    exit 1
}
catch {
    # Fallback: simple console message if Windows Forms fails
    Write-Host "ZipLock Password Manager installation failed."
    Write-Host "Please try running the installer as Administrator or check for system issues."
    Write-Host "For support, visit: https://github.com/ejangi/ziplock"
    exit 1
}
