package com.ziplock.config

import android.content.Context
import android.content.SharedPreferences
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.io.File

/**
 * Android Configuration Manager
 *
 * Provides persistent storage for ZipLock app settings, particularly
 * the last opened archive path for seamless user experience.
 *
 * This mirrors the functionality of the Linux ConfigManager but uses
 * Android SharedPreferences for persistence.
 */
class AndroidConfigManager(private val context: Context) {

    companion object {
        private const val PREFS_NAME = "ziplock_config"
        private const val KEY_LAST_ARCHIVE_PATH = "last_archive_path"
        private const val KEY_LAST_ARCHIVE_ACCESSED = "last_archive_accessed"
        private const val KEY_SHOW_WIZARD_ON_STARTUP = "show_wizard_on_startup"
        private const val KEY_AUTO_LOCK_TIMEOUT = "auto_lock_timeout"
        private const val KEY_THEME = "theme"
    }

    private val sharedPreferences: SharedPreferences =
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    // Configuration state flows
    private val _lastArchivePath = MutableStateFlow<String?>(null)
    val lastArchivePath: StateFlow<String?> = _lastArchivePath.asStateFlow()

    private val _config = MutableStateFlow(AndroidConfig())
    val config: StateFlow<AndroidConfig> = _config.asStateFlow()

    init {
        loadConfiguration()
    }

    /**
     * Get the last opened archive path if it still exists and is accessible
     */
    fun getLastOpenedArchivePath(): String? {
        val lastPath = _lastArchivePath.value
        return if (lastPath != null && isFileAccessible(lastPath)) {
            lastPath
        } else {
            null
        }
    }

    /**
     * Check if there's a valid last opened archive that can be auto-opened
     */
    fun hasValidLastArchive(): Boolean {
        return getLastOpenedArchivePath() != null
    }

    /**
     * Save the archive path as the most recently accessed
     */
    fun setLastArchivePath(archivePath: String) {
        val currentTime = System.currentTimeMillis()

        sharedPreferences.edit()
            .putString(KEY_LAST_ARCHIVE_PATH, archivePath)
            .putLong(KEY_LAST_ARCHIVE_ACCESSED, currentTime)
            .apply()

        _lastArchivePath.value = archivePath

        // Update the config state
        _config.value = _config.value.copy(
            lastArchivePath = archivePath,
            lastArchiveAccessed = currentTime
        )
    }

    /**
     * Clear the saved last archive path
     */
    fun clearLastArchivePath() {
        sharedPreferences.edit()
            .remove(KEY_LAST_ARCHIVE_PATH)
            .remove(KEY_LAST_ARCHIVE_ACCESSED)
            .apply()

        _lastArchivePath.value = null
        _config.value = _config.value.copy(
            lastArchivePath = null,
            lastArchiveAccessed = 0L
        )
    }

    /**
     * Update UI settings
     */
    fun setShowWizardOnStartup(show: Boolean) {
        sharedPreferences.edit()
            .putBoolean(KEY_SHOW_WIZARD_ON_STARTUP, show)
            .apply()

        _config.value = _config.value.copy(showWizardOnStartup = show)
    }

    /**
     * Set auto-lock timeout in minutes
     */
    fun setAutoLockTimeout(timeoutMinutes: Int) {
        sharedPreferences.edit()
            .putInt(KEY_AUTO_LOCK_TIMEOUT, timeoutMinutes)
            .apply()

        _config.value = _config.value.copy(autoLockTimeoutMinutes = timeoutMinutes)
    }

    /**
     * Set the app theme
     */
    fun setTheme(theme: String) {
        sharedPreferences.edit()
            .putString(KEY_THEME, theme)
            .apply()

        _config.value = _config.value.copy(theme = theme)
    }

    /**
     * Check if the app should show the wizard on startup
     */
    fun shouldShowWizard(): Boolean {
        return _config.value.showWizardOnStartup && !hasValidLastArchive()
    }

    /**
     * Reset all configuration to defaults
     */
    fun resetToDefaults() {
        sharedPreferences.edit().clear().apply()
        loadConfiguration()
    }

    /**
     * Load configuration from SharedPreferences
     */
    private fun loadConfiguration() {
        val lastArchivePath = sharedPreferences.getString(KEY_LAST_ARCHIVE_PATH, null)
        val lastArchiveAccessed = sharedPreferences.getLong(KEY_LAST_ARCHIVE_ACCESSED, 0L)
        val showWizardOnStartup = sharedPreferences.getBoolean(KEY_SHOW_WIZARD_ON_STARTUP, true)
        val autoLockTimeout = sharedPreferences.getInt(KEY_AUTO_LOCK_TIMEOUT, 15)
        val theme = sharedPreferences.getString(KEY_THEME, "system")

        _lastArchivePath.value = lastArchivePath
        _config.value = AndroidConfig(
            lastArchivePath = lastArchivePath,
            lastArchiveAccessed = lastArchiveAccessed,
            showWizardOnStartup = showWizardOnStartup,
            autoLockTimeoutMinutes = autoLockTimeout,
            theme = theme ?: "system"
        )
    }

    /**
     * Check if a file path is accessible
     * Handles both regular file paths and Android content URIs
     */
    private fun isFileAccessible(path: String): Boolean {
        return try {
            when {
                path.startsWith("content://") -> {
                    // Handle Android content URIs
                    val uri = android.net.Uri.parse(path)
                    context.contentResolver.openInputStream(uri)?.use { true } ?: false
                }
                else -> {
                    // Handle regular file paths
                    File(path).exists() && File(path).canRead()
                }
            }
        } catch (e: Exception) {
            false
        }
    }
}

/**
 * Android-specific configuration data class
 */
data class AndroidConfig(
    val lastArchivePath: String? = null,
    val lastArchiveAccessed: Long = 0L,
    val showWizardOnStartup: Boolean = true,
    val autoLockTimeoutMinutes: Int = 15,
    val theme: String = "system"
)
