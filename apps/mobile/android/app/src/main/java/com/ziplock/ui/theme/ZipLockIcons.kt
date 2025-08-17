package com.ziplock.ui.theme

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource

/**
 * ZipLock Icon System
 * Provides consistent iconography across the app, matching the Linux theme.rs implementation
 */
object ZipLockIcons {

    // Core UI Icons
    val Eye = Icons.Default.Visibility
    val EyeOff = Icons.Default.VisibilityOff
    val Alert = Icons.Default.Info
    val Check = Icons.Default.Check
    val Error = Icons.Default.Error
    val Warning = Icons.Default.Warning
    val Refresh = Icons.Default.Refresh
    val Plus = Icons.Default.Add
    val Settings = Icons.Default.Settings
    val Lock = Icons.Default.Lock
    val Close = Icons.Default.Close
    val Search = Icons.Default.Search
    val Menu = Icons.Default.Menu
    val ArrowBack = Icons.Default.ArrowBack
    val MoreVert = Icons.Default.MoreVert
    val Edit = Icons.Default.Edit
    val Delete = Icons.Default.Delete
    val Copy = Icons.Default.ContentCopy
    val Share = Icons.Default.Share
    val Download = Icons.Default.Download
    val Upload = Icons.Default.Upload
    val Folder = Icons.Default.Folder
    val FolderOpen = Icons.Default.FolderOpen
    val File = Icons.Default.InsertDriveFile

    // Credential Type Icons
    val CreditCard = Icons.Default.CreditCard
    val Note = Icons.Default.Note
    val User = Icons.Default.Person
    val Document = Icons.Default.Description
    val Bank = Icons.Default.AccountBalance
    val Wallet = Icons.Default.AccountBalanceWallet
    val Database = Icons.Default.Storage
    val License = Icons.Default.VpnKey
    val Web = Icons.Default.Language
    val Phone = Icons.Default.Phone
    val Email = Icons.Default.Email

    // Security Icons
    val Security = Icons.Default.Security
    val Shield = Icons.Default.Shield
    val Key = Icons.Default.VpnKey
    val Fingerprint = Icons.Default.Fingerprint
    val Password = Icons.Default.Password

    // Action Icons
    val Save = Icons.Default.Save
    val Cancel = Icons.Default.Cancel
    val Done = Icons.Default.Done
    val Send = Icons.Default.Send
    val Sync = Icons.Default.Sync
    val SyncProblem = Icons.Default.SyncProblem
    val CloudDownload = Icons.Default.CloudDownload
    val CloudUpload = Icons.Default.CloudUpload
    val CloudOff = Icons.Default.CloudOff

    // Navigation Icons
    val Home = Icons.Default.Home
    val Favorite = Icons.Default.Favorite
    val FavoriteBorder = Icons.Default.FavoriteBorder
    val Star = Icons.Default.Star
    val StarBorder = Icons.Default.StarBorder
    val Bookmark = Icons.Default.Bookmark
    val BookmarkBorder = Icons.Default.BookmarkBorder

    // Status Icons
    val Success = Icons.Default.CheckCircle
    val Info = Icons.Default.Info
    val ErrorCircle = Icons.Default.Error
    val WarningCircle = Icons.Default.Warning

    // File and Storage Icons
    val Archive = Icons.Default.Archive
    val Backup = Icons.Default.Backup
    val Restore = Icons.Default.Restore
    val Import = Icons.Default.GetApp
    val Export = Icons.Default.Publish
}

/**
 * Get icon for credential type
 * Matches the Linux implementation's credential type icon mapping
 */
@Composable
fun getCredentialTypeIcon(credentialType: String): ImageVector {
    return when (credentialType.lowercase()) {
        "login", "website", "web" -> ZipLockIcons.Web
        "credit_card", "card", "payment" -> ZipLockIcons.CreditCard
        "note", "secure_note" -> ZipLockIcons.Note
        "identity", "personal" -> ZipLockIcons.User
        "document" -> ZipLockIcons.Document
        "bank", "banking" -> ZipLockIcons.Bank
        "wallet", "crypto" -> ZipLockIcons.Wallet
        "database", "server" -> ZipLockIcons.Database
        "license", "software" -> ZipLockIcons.License
        "phone" -> ZipLockIcons.Phone
        "email" -> ZipLockIcons.Email
        else -> ZipLockIcons.Lock
    }
}

/**
 * Get validation icon based on state
 */
@Composable
fun getValidationIcon(isValid: Boolean, hasError: Boolean = false): ImageVector {
    return when {
        hasError -> ZipLockIcons.ErrorCircle
        isValid -> ZipLockIcons.Success
        else -> ZipLockIcons.WarningCircle
    }
}

/**
 * Get alert icon based on alert level
 */
@Composable
fun getAlertIcon(level: AlertLevel): ImageVector {
    return when (level) {
        AlertLevel.Error -> ZipLockIcons.ErrorCircle
        AlertLevel.Warning -> ZipLockIcons.WarningCircle
        AlertLevel.Success -> ZipLockIcons.Success
        AlertLevel.Info -> ZipLockIcons.Info
    }
}

/**
 * Alert levels matching the Linux theme implementation
 */
enum class AlertLevel {
    Error,
    Warning,
    Success,
    Info
}
