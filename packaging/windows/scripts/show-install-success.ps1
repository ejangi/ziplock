# ZipLock MSI Installer Success Message
# Custom action script to display installation success dialog

param(
    [string]$ProductName = "ZipLock Password Manager",
    [string]$Version = ""
)

$ErrorActionPreference = "SilentlyContinue"

try {
    # Load Windows Forms for message box
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing

    # Construct success message
    $title = "Installation Complete"
    $message = "$ProductName has been successfully installed!"

    if ($Version) {
        $message += "`n`nVersion: $Version"
    }

    $message += "`n`nYou can now launch ZipLock from:"
    $message += "`n• Start Menu > ZipLock"
    $message += "`n• Desktop shortcut (if selected)"
    $message += "`n`nThank you for choosing ZipLock!"

    # Show success dialog with OK button and Information icon
    [System.Windows.Forms.MessageBox]::Show(
        $message,
        $title,
        [System.Windows.Forms.MessageBoxButtons]::OK,
        [System.Windows.Forms.MessageBoxIcon]::Information
    )

    # Log success for debugging
    $logMessage = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'): ZipLock installation completed successfully"
    Write-Host $logMessage

    # Try to write to Windows event log (best effort)
    try {
        Write-EventLog -LogName "Application" -Source "ZipLock Installer" -EntryType Information -EventId 1001 -Message "ZipLock Password Manager installation completed successfully"
    } catch {
        # Silently ignore event log errors
    }

    exit 0
}
catch {
    # Fallback: simple console message if Windows Forms fails
    Write-Host "ZipLock Password Manager has been successfully installed!"
    Write-Host "You can now launch ZipLock from the Start Menu."
    exit 0
}
