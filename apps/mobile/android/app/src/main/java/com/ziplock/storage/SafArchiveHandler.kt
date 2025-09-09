package com.ziplock.storage

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import android.util.Log
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.documentfile.provider.DocumentFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.*

/**
 * Storage Access Framework (SAF) Archive Handler
 *
 * This class handles all Storage Access Framework operations for ZipLock archives.
 * It provides a unified interface for:
 * - Opening archive files from various storage locations
 * - Saving archive files to user-selected locations
 * - Managing recent files and permissions
 * - Creating new archive files
 *
 * This follows the unified architecture pattern where Android handles all
 * file I/O operations while the mobile FFI handles memory operations.
 */
class SafArchiveHandler(private val context: Context) {

    companion object {
        private const val TAG = "SafArchiveHandler"

        // MIME types for 7z archives
        const val MIME_TYPE_7Z = "application/x-7z-compressed"
        const val MIME_TYPE_ARCHIVE = "application/octet-stream"

        // File extensions
        const val EXTENSION_7Z = ".7z"
        const val EXTENSION_ZIPLOCK = ".ziplock"

        // Storage preferences
        private const val PREF_NAME = "ziplock_storage"
        private const val PREF_RECENT_ARCHIVES = "recent_archives"
        private const val PREF_LAST_SAVE_LOCATION = "last_save_location"
        private const val MAX_RECENT_FILES = 10

        // File size limits
        private const val MAX_ARCHIVE_SIZE = 500 * 1024 * 1024 // 500MB
    }

    /**
     * Result of file operation
     */
    sealed class FileOperationResult {
        data class Success(
            val uri: Uri,
            val displayName: String? = null,
            val size: Long? = null
        ) : FileOperationResult()

        data class Error(val message: String, val exception: Throwable? = null) : FileOperationResult()
        data class Cancelled(val reason: String = "User cancelled") : FileOperationResult()
    }

    /**
     * Archive file information
     */
    data class ArchiveInfo(
        val uri: Uri,
        val displayName: String,
        val size: Long,
        val lastModified: Long,
        val mimeType: String? = null,
        val isWritable: Boolean = false
    )

    /**
     * Recent archive file entry
     */
    @Serializable
    data class RecentArchive(
        val uri: String,
        val displayName: String,
        val lastOpened: Long,
        val size: Long = 0
    )

    private val preferences = context.getSharedPreferences(PREF_NAME, Context.MODE_PRIVATE)

    /**
     * Create an intent to open an archive file
     * This should be used with an ActivityResultLauncher
     *
     * @return Intent for opening archive files
     */
    fun createOpenArchiveIntent(): Intent {
        return Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_MIME_TYPES, arrayOf(
                MIME_TYPE_7Z,
                MIME_TYPE_ARCHIVE,
                "application/zip" // Fallback for some file managers
            ))
            putExtra(Intent.EXTRA_TITLE, "Select ZipLock Archive")
        }
    }

    /**
     * Create an intent to save/create a new archive file
     * This should be used with an ActivityResultLauncher
     *
     * @param suggestedName Suggested filename for the archive
     * @return Intent for creating/saving archive files
     */
    fun createSaveArchiveIntent(suggestedName: String = generateDefaultArchiveName()): Intent {
        return Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = MIME_TYPE_7Z
            putExtra(Intent.EXTRA_TITLE, suggestedName)

            // Set initial location to last save location if available
            getLastSaveLocation()?.let { lastUri ->
                putExtra(DocumentsContract.EXTRA_INITIAL_URI, lastUri)
            }
        }
    }

    /**
     * Read archive data from URI
     *
     * @param archiveUri URI of the archive to read
     * @return ByteArray containing archive data, or null on error
     */
    suspend fun readArchiveData(archiveUri: Uri): ByteArray? = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Reading archive from: $archiveUri")

            context.contentResolver.openInputStream(archiveUri)?.use { inputStream ->
                val buffer = ByteArrayOutputStream()
                val data = ByteArray(8192)
                var totalBytes = 0L
                var bytesRead: Int

                while (inputStream.read(data).also { bytesRead = it } != -1) {
                    buffer.write(data, 0, bytesRead)
                    totalBytes += bytesRead

                    // Safety check to prevent memory exhaustion
                    if (totalBytes > MAX_ARCHIVE_SIZE) {
                        Log.e(TAG, "Archive too large: $totalBytes bytes")
                        return@withContext null
                    }
                }

                val result = buffer.toByteArray()
                Log.d(TAG, "Successfully read ${result.size} bytes")

                // Add to recent files
                addToRecentArchives(archiveUri, result.size.toLong())

                result
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to read archive data", e)
            null
        }
    }

    /**
     * Write archive data to URI
     *
     * @param archiveData ByteArray containing archive data
     * @param destinationUri URI where to write the archive
     * @return true if successful, false otherwise
     */
    suspend fun writeArchiveData(archiveData: ByteArray, destinationUri: Uri): Boolean = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Writing ${archiveData.size} bytes to: $destinationUri")

            context.contentResolver.openOutputStream(destinationUri, "wt")?.use { outputStream ->
                outputStream.write(archiveData)
                outputStream.flush()

                Log.d(TAG, "Successfully wrote archive data")

                // Update recent files and save location
                addToRecentArchives(destinationUri, archiveData.size.toLong())
                saveLastSaveLocation(destinationUri)

                true
            } ?: false
        } catch (e: Exception) {
            Log.e(TAG, "Failed to write archive data", e)
            false
        }
    }

    /**
     * Get information about an archive file
     *
     * @param archiveUri URI of the archive
     * @return ArchiveInfo or null if unable to get info
     */
    suspend fun getArchiveInfo(archiveUri: Uri): ArchiveInfo? = withContext(Dispatchers.IO) {
        try {
            when (archiveUri.scheme) {
                "file" -> {
                    // Handle file:// URIs directly
                    val file = File(archiveUri.path ?: return@withContext null)
                    if (!file.exists()) {
                        Log.w(TAG, "Archive file does not exist: $archiveUri")
                        return@withContext null
                    }

                    ArchiveInfo(
                        uri = archiveUri,
                        displayName = file.name,
                        size = file.length(),
                        lastModified = file.lastModified(),
                        mimeType = "application/x-7z-compressed",
                        isWritable = file.canWrite()
                    )
                }
                "content" -> {
                    // Handle content:// URIs via DocumentFile
                    val documentFile = DocumentFile.fromSingleUri(context, archiveUri)
                    if (documentFile == null || !documentFile.exists()) {
                        Log.w(TAG, "Archive file does not exist: $archiveUri")
                        return@withContext null
                    }

                    ArchiveInfo(
                        uri = archiveUri,
                        displayName = documentFile.name ?: "Unknown",
                        size = documentFile.length(),
                        lastModified = documentFile.lastModified(),
                        mimeType = documentFile.type,
                        isWritable = documentFile.canWrite()
                    )
                }
                else -> {
                    Log.w(TAG, "Unsupported URI scheme: ${archiveUri.scheme}")
                    return@withContext null
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get archive info for $archiveUri", e)
            null
        }
    }

    /**
     * Check if URI is still accessible
     *
     * @param uri URI to check
     * @return true if accessible, false otherwise
     */
    suspend fun isUriAccessible(uri: Uri): Boolean = withContext(Dispatchers.IO) {
        try {
            val documentFile = DocumentFile.fromSingleUri(context, uri)
            documentFile?.exists() == true
        } catch (e: Exception) {
            Log.w(TAG, "URI not accessible: $uri", e)
            false
        }
    }

    /**
     * Get list of recent archive files
     *
     * @return List of RecentArchive entries, filtered for accessible files
     */
    suspend fun getRecentArchives(): List<RecentArchive> = withContext(Dispatchers.IO) {
        try {
            val recentJson = preferences.getString(PREF_RECENT_ARCHIVES, "[]") ?: "[]"
            val recentList = kotlinx.serialization.json.Json.decodeFromString<List<RecentArchive>>(recentJson)

            // Filter out inaccessible files
            recentList.filter { recent ->
                try {
                    isUriAccessible(Uri.parse(recent.uri))
                } catch (e: Exception) {
                    false
                }
            }.sortedByDescending { it.lastOpened }

        } catch (e: Exception) {
            Log.e(TAG, "Failed to get recent archives", e)
            emptyList()
        }
    }

    /**
     * Clear recent archives list
     */
    fun clearRecentArchives() {
        preferences.edit().remove(PREF_RECENT_ARCHIVES).apply()
        Log.d(TAG, "Cleared recent archives list")
    }

    /**
     * Remove specific archive from recent list
     *
     * @param archiveUri URI to remove
     */
    suspend fun removeFromRecentArchives(archiveUri: Uri) {
        try {
            val current = getRecentArchives().toMutableList()
            current.removeAll { it.uri == archiveUri.toString() }

            val json = kotlinx.serialization.json.Json.encodeToString(kotlinx.serialization.builtins.ListSerializer(RecentArchive.serializer()), current)
            preferences.edit().putString(PREF_RECENT_ARCHIVES, json).apply()

            Log.d(TAG, "Removed from recent archives: $archiveUri")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to remove from recent archives", e)
        }
    }

    /**
     * Request persistent permissions for URI
     * This allows the app to access the file even after restart
     *
     * @param uri URI to request permissions for
     * @return true if permissions were granted
     */
    fun requestPersistentPermissions(uri: Uri): Boolean {
        return try {
            val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            context.contentResolver.takePersistableUriPermission(uri, flags)
            Log.d(TAG, "Granted persistent permissions for: $uri")
            true
        } catch (e: Exception) {
            Log.w(TAG, "Could not grant persistent permissions for: $uri", e)
            false
        }
    }

    /**
     * Release persistent permissions for URI
     *
     * @param uri URI to release permissions for
     */
    fun releasePersistentPermissions(uri: Uri) {
        try {
            val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            context.contentResolver.releasePersistableUriPermission(uri, flags)
            Log.d(TAG, "Released persistent permissions for: $uri")
        } catch (e: Exception) {
            Log.w(TAG, "Could not release persistent permissions for: $uri", e)
        }
    }

    /**
     * Get list of URIs with persistent permissions
     *
     * @return List of URIs with persistent access
     */
    fun getPersistedUris(): List<Uri> {
        return try {
            context.contentResolver.persistedUriPermissions.map { it.uri }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get persisted URIs", e)
            emptyList()
        }
    }

    /**
     * Add archive to recent files list
     */
    private fun addToRecentArchives(uri: Uri, size: Long) {
        try {
            val documentFile = DocumentFile.fromSingleUri(context, uri)
            val displayName = documentFile?.name ?: uri.lastPathSegment ?: "Unknown"

            val current = runCatching {
                val json = preferences.getString(PREF_RECENT_ARCHIVES, "[]") ?: "[]"
                kotlinx.serialization.json.Json.decodeFromString<List<RecentArchive>>(json).toMutableList()
            }.getOrElse { mutableListOf() }

            // Remove existing entry for this URI
            current.removeAll { it.uri == uri.toString() }

            // Add new entry at the beginning
            current.add(0, RecentArchive(
                uri = uri.toString(),
                displayName = displayName,
                lastOpened = System.currentTimeMillis(),
                size = size
            ))

            // Limit to max recent files
            while (current.size > MAX_RECENT_FILES) {
                current.removeAt(current.size - 1)
            }

            // Save back to preferences
            val json = kotlinx.serialization.json.Json.encodeToString(kotlinx.serialization.builtins.ListSerializer(RecentArchive.serializer()), current)
            preferences.edit().putString(PREF_RECENT_ARCHIVES, json).apply()

            Log.d(TAG, "Added to recent archives: $displayName")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to add to recent archives", e)
        }
    }

    /**
     * Save last save location for future use
     */
    private fun saveLastSaveLocation(uri: Uri) {
        try {
            // Get parent directory URI
            val documentFile = DocumentFile.fromSingleUri(context, uri)
            documentFile?.parentFile?.uri?.let { parentUri ->
                preferences.edit().putString(PREF_LAST_SAVE_LOCATION, parentUri.toString()).apply()
                Log.d(TAG, "Saved last save location: $parentUri")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to save last save location", e)
        }
    }

    /**
     * Get last save location URI
     */
    private fun getLastSaveLocation(): Uri? {
        return try {
            preferences.getString(PREF_LAST_SAVE_LOCATION, null)?.let { Uri.parse(it) }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get last save location", e)
            null
        }
    }

    /**
     * Generate a default archive filename with timestamp
     */
    private fun generateDefaultArchiveName(): String {
        val timestamp = SimpleDateFormat("yyyy-MM-dd_HH-mm-ss", Locale.getDefault())
            .format(Date())
        return "ziplock_archive_$timestamp$EXTENSION_7Z"
    }

    /**
     * Validate archive file
     *
     * @param uri URI of the archive to validate
     * @return true if the file appears to be a valid archive
     */
    suspend fun validateArchiveFile(uri: Uri): Boolean = withContext(Dispatchers.IO) {
        try {
            val archiveInfo = getArchiveInfo(uri) ?: return@withContext false

            // Check file size
            if (archiveInfo.size <= 0 || archiveInfo.size > MAX_ARCHIVE_SIZE) {
                Log.w(TAG, "Invalid archive size: ${archiveInfo.size}")
                return@withContext false
            }

            // Check file extension
            val fileName = archiveInfo.displayName.lowercase()
            if (!fileName.endsWith(EXTENSION_7Z) && !fileName.endsWith(EXTENSION_ZIPLOCK)) {
                Log.w(TAG, "Invalid archive extension: $fileName")
                return@withContext false
            }

            // Try to read a small portion to validate it's not corrupted
            context.contentResolver.openInputStream(uri)?.use { inputStream ->
                val header = ByteArray(6)
                val bytesRead = inputStream.read(header)

                // Check for 7z file signature: "7z" + BC + AF + 27 + 1C
                if (bytesRead >= 6 &&
                    header[0] == '7'.code.toByte() &&
                    header[1] == 'z'.code.toByte() &&
                    header[2] == 0xBC.toByte()) {
                    return@withContext true
                }
            }

            Log.w(TAG, "Archive validation failed - invalid header")
            false
        } catch (e: Exception) {
            Log.e(TAG, "Archive validation failed with exception", e)
            false
        }
    }
}
