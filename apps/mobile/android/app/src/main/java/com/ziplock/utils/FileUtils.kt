package com.ziplock.utils

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.OpenableColumns
import androidx.documentfile.provider.DocumentFile
import java.io.File
import java.io.FileOutputStream
import java.io.IOException

/**
 * Utility class for handling file operations, particularly converting content URIs
 * to usable file paths for the native library.
 */
object FileUtils {

    /**
     * Convert a content URI to a real file path that can be used by the native library.
     * For content URIs that don't have direct file paths, this creates a copy in the
     * app's private directory.
     *
     * @param context Android context
     * @param uri The content URI to convert
     * @param fileName Desired filename (with extension)
     * @return Real file path that the native library can use
     * @throws IOException if file operations fail
     */
    fun getUsableFilePath(context: Context, uri: Uri, fileName: String): String {
        // If it's already a file URI, extract the path directly
        if (uri.scheme == "file") {
            return uri.path ?: throw IOException("Invalid file URI: $uri")
        }

        // For content URIs, always copy to private directory
        // The native library cannot handle content URIs directly, and direct file path
        // access may fail due to Android scoped storage restrictions
        if (uri.scheme == "content") {
            println("FileUtils: Converting content URI to file path: $uri")
            println("FileUtils: URI authority: ${uri.authority}")
            println("FileUtils: URI path: ${uri.path}")

            // Check if we can read from the content URI first
            try {
                context.contentResolver.openInputStream(uri)?.use { stream ->
                    println("FileUtils: Successfully opened input stream for content URI")
                    val available = stream.available()
                    println("FileUtils: Available bytes: $available")
                }
            } catch (e: Exception) {
                println("FileUtils: ERROR - Cannot open input stream: ${e.message}")
                throw IOException("Cannot access file at URI: $uri. Error: ${e.message}")
            }

            // Always copy content URI to private cache file to avoid permission issues
            // This ensures the native library can access the file regardless of scoped storage
            println("FileUtils: Copying content URI to private cache")
            val result = copyContentUriToPrivateFile(context, uri, fileName)
            println("FileUtils: Successfully copied to private file: $result")
            return result
        }

        throw IOException("Unsupported URI scheme: ${uri.scheme}")
    }

    /**
     * Try to get a real file path from a content URI without copying.
     * Returns null if the content URI doesn't correspond to a real file.
     */
    private fun getRealPathFromContentUri(context: Context, uri: Uri): String? {
        return try {
            when {
                // Handle external storage documents
                DocumentsContract.isDocumentUri(context, uri) -> {
                    when (uri.authority) {
                        "com.android.externalstorage.documents" -> {
                            val docId = DocumentsContract.getDocumentId(uri)
                            val split = docId.split(":")
                            if (split.size >= 2) {
                                val type = split[0]
                                val path = split[1]
                                when (type) {
                                    "primary" -> "/storage/emulated/0/$path"
                                    else -> "/storage/$type/$path"
                                }
                            } else null
                        }
                        "com.android.providers.downloads.documents" -> {
                            // For downloads, try to resolve the real path
                            resolveDownloadsPath(context, uri)
                        }
                        "com.android.providers.media.documents" -> {
                            // For media documents, try to resolve real path
                            resolveMediaPath(context, uri)
                        }
                        else -> null
                    }
                }
                // Handle regular content URIs by querying for _data column
                else -> {
                    context.contentResolver.query(uri, arrayOf("_data"), null, null, null)?.use { cursor ->
                        if (cursor.moveToFirst()) {
                            val columnIndex = cursor.getColumnIndex("_data")
                            if (columnIndex >= 0) cursor.getString(columnIndex) else null
                        } else null
                    }
                }
            }
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Resolve real file path for downloads documents.
     */
    private fun resolveDownloadsPath(context: Context, uri: Uri): String? {
        return try {
            context.contentResolver.query(uri, arrayOf("_data"), null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val columnIndex = cursor.getColumnIndex("_data")
                    if (columnIndex >= 0) cursor.getString(columnIndex) else null
                } else null
            }
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Resolve real file path for media documents.
     */
    private fun resolveMediaPath(context: Context, uri: Uri): String? {
        return try {
            val docId = DocumentsContract.getDocumentId(uri)
            val split = docId.split(":")
            if (split.size >= 2) {
                val type = split[0]
                val id = split[1]

                val contentUri = when (type) {
                    "image" -> MediaStore.Images.Media.EXTERNAL_CONTENT_URI
                    "video" -> MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                    "audio" -> MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
                    else -> MediaStore.Files.getContentUri("external")
                }

                context.contentResolver.query(
                    contentUri,
                    arrayOf("_data"),
                    "_id=?",
                    arrayOf(id),
                    null
                )?.use { cursor ->
                    if (cursor.moveToFirst()) {
                        val columnIndex = cursor.getColumnIndex("_data")
                        if (columnIndex >= 0) cursor.getString(columnIndex) else null
                    } else null
                }
            } else null
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Copy content from a content URI to a file in the app's private directory.
     * This ensures the native library can work with a real file path.
     * Only used when we cannot get a direct file path from the content URI.
     */
    private fun copyContentUriToPrivateFile(context: Context, uri: Uri, fileName: String): String {
        println("FileUtils: Starting copy operation for URI: $uri")

        // Create a unique filename to avoid conflicts
        val timestamp = System.currentTimeMillis()
        val uniqueFileName = "${timestamp}_$fileName"
        println("FileUtils: Generated unique filename: $uniqueFileName")

        // Create file in app's private cache directory
        val privateFile = File(context.cacheDir, "archives/$uniqueFileName")
        println("FileUtils: Target private file path: ${privateFile.absolutePath}")

        // Ensure parent directory exists
        val parentCreated = privateFile.parentFile?.mkdirs() ?: false
        println("FileUtils: Parent directory created/exists: $parentCreated")
        println("FileUtils: Parent directory path: ${privateFile.parentFile?.absolutePath}")

        // Verify parent directory is writable
        privateFile.parentFile?.let { parent ->
            if (!parent.canWrite()) {
                throw IOException("Cannot write to cache directory: ${parent.absolutePath}")
            }
            println("FileUtils: Parent directory is writable")
        }

        // Copy content from URI to private file
        var bytesCopied = 0L
        try {
            context.contentResolver.openInputStream(uri)?.use { inputStream ->
                FileOutputStream(privateFile).use { outputStream ->
                    bytesCopied = inputStream.copyTo(outputStream)
                }
            } ?: throw IOException("Failed to open input stream for URI: $uri")

            println("FileUtils: Successfully copied $bytesCopied bytes")
            println("FileUtils: Final file size: ${privateFile.length()} bytes")
            println("FileUtils: File exists: ${privateFile.exists()}")
            println("FileUtils: File readable: ${privateFile.canRead()}")

            if (!privateFile.exists()) {
                throw IOException("File was not created successfully")
            }

            if (privateFile.length() == 0L) {
                throw IOException("Copied file is empty")
            }

        } catch (e: Exception) {
            println("FileUtils: ERROR during copy operation: ${e.message}")
            // Clean up partial file
            if (privateFile.exists()) {
                privateFile.delete()
                println("FileUtils: Cleaned up partial file")
            }
            throw IOException("Failed to copy file from URI: $uri. Error: ${e.message}")
        }

        return privateFile.absolutePath
    }

    /**
     * Get a writable file path for creating a new archive.
     * For content URIs pointing to directories, this creates a path in app's private directory
     * and sets up for later copying back to the original location.
     */
    fun getWritableArchivePath(context: Context, destinationUri: Uri, archiveName: String): WritableArchiveInfo {
        val fileName = if (archiveName.endsWith(".7z")) archiveName else "$archiveName.7z"

        when (destinationUri.scheme) {
            "file" -> {
                // Direct file system access
                val destinationPath = destinationUri.path ?: throw IOException("Invalid file URI")
                val archivePath = File(destinationPath, fileName).absolutePath
                return WritableArchiveInfo(
                    workingPath = archivePath,
                    finalDestinationUri = null,
                    needsCopyBack = false
                )
            }

            "content" -> {
                // Content URI - work in private directory and copy back later
                val timestamp = System.currentTimeMillis()
                val uniqueFileName = "${timestamp}_$fileName"
                val privateFile = File(context.cacheDir, "new_archives/$uniqueFileName")

                // Ensure parent directory exists
                privateFile.parentFile?.mkdirs()

                return WritableArchiveInfo(
                    workingPath = privateFile.absolutePath,
                    finalDestinationUri = constructChildDocumentUri(destinationUri, fileName),
                    needsCopyBack = true
                )
            }

            else -> throw IOException("Unsupported URI scheme: ${destinationUri.scheme}")
        }
    }

    /**
     * Copy a file from the working location back to the final destination URI.
     * Used when creating archives in content URI locations.
     */
    fun copyBackToDestination(context: Context, workingFilePath: String, destinationUri: Uri): Boolean {
        return try {
            val workingFile = File(workingFilePath)
            if (!workingFile.exists()) {
                return false
            }

            // Create the document at the destination
            val documentFile = DocumentFile.fromTreeUri(context, destinationUri)
            val fileName = workingFile.name.substringAfter("_") // Remove timestamp prefix
            val outputDocument = documentFile?.createFile("application/x-7z-compressed", fileName.removeSuffix(".7z"))

            outputDocument?.let { doc ->
                context.contentResolver.openOutputStream(doc.uri)?.use { outputStream ->
                    workingFile.inputStream().use { inputStream ->
                        inputStream.copyTo(outputStream)
                    }
                }

                // Clean up working file
                workingFile.delete()
                true
            } ?: false
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Construct a child document URI for a file within a directory tree URI.
     */
    private fun constructChildDocumentUri(treeUri: Uri, fileName: String): Uri {
        val documentId = DocumentsContract.getTreeDocumentId(treeUri)
        val childDocumentId = "$documentId/$fileName"

        return DocumentsContract.buildDocumentUriUsingTree(treeUri, childDocumentId)
    }

    /**
     * Get a human-readable display name for a URI.
     */
    fun getDisplayName(context: Context, uri: Uri): String? {
        if (uri.scheme == "content") {
            context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (nameIndex >= 0) {
                        return cursor.getString(nameIndex)
                    }
                }
            }
        }
        return uri.lastPathSegment
    }

    /**
     * Debug utility to check if a file can be accessed and is not locked by another process.
     * This helps diagnose file locking issues before attempting to open archives.
     */
    fun checkFileAccessibility(filePath: String): FileAccessibilityInfo {
        // For content URIs, use Android SAF if available
        if (filePath.startsWith("content://")) {
            return checkContentUriAccessibility(filePath)
        }

        val file = File(filePath)

        return try {
            val exists = file.exists()
            val canRead = file.canRead()
            val canWrite = file.canWrite()
            val size = if (exists) file.length() else 0
            val lastModified = if (exists) file.lastModified() else 0

            // Check for lock file
            val lockFile = File("$filePath.lock")
            val hasLockFile = lockFile.exists()
            val lockFileContent = if (hasLockFile) {
                try { lockFile.readText().trim() } catch (e: Exception) { "unreadable" }
            } else null

            // Try to open file for reading to test accessibility
            val canOpenForRead = try {
                file.inputStream().use { true }
            } catch (e: Exception) {
                false
            }

            // Try to create a test lock file to see if we can write to the directory
            val canCreateLockFile = try {
                val testLockFile = File("${filePath}.test_lock")
                testLockFile.writeText("test")
                val success = testLockFile.exists()
                testLockFile.delete()
                success
            } catch (e: Exception) {
                false
            }

            FileAccessibilityInfo(
                exists = exists,
                canRead = canRead,
                canWrite = canWrite,
                canOpenForRead = canOpenForRead,
                canCreateLockFile = canCreateLockFile,
                size = size,
                lastModified = lastModified,
                hasLockFile = hasLockFile,
                lockFileContent = lockFileContent,
                error = null
            )
        } catch (e: Exception) {
            FileAccessibilityInfo(
                exists = false,
                canRead = false,
                canWrite = false,
                canOpenForRead = false,
                canCreateLockFile = false,
                size = 0,
                lastModified = 0,
                hasLockFile = false,
                lockFileContent = null,
                error = e.message
            )
        }
    }

    /**
     * Check accessibility of a content URI using Android SAF
     */
    private fun checkContentUriAccessibility(contentUri: String): FileAccessibilityInfo {
        return if (isAndroidSafAvailable()) {
            try {
                // Test if we can access the content URI through Android SAF
                val testResult = com.ziplock.ffi.ZipLockNative.testAndroidSaf()

                FileAccessibilityInfo(
                    exists = testResult,
                    canRead = testResult,
                    canWrite = testResult, // Assume writable if readable via SAF
                    canOpenForRead = testResult,
                    canCreateLockFile = false, // Not applicable for content URIs
                    size = 0, // Size will be determined by SAF
                    lastModified = 0, // Not available through basic SAF test
                    hasLockFile = false, // Not applicable for content URIs
                    lockFileContent = null,
                    error = if (!testResult) "Content URI cannot be accessed through Android SAF" else null
                )
            } catch (e: Exception) {
                FileAccessibilityInfo(
                    exists = false,
                    canRead = false,
                    canWrite = false,
                    canOpenForRead = false,
                    canCreateLockFile = false,
                    size = 0,
                    lastModified = 0,
                    hasLockFile = false,
                    lockFileContent = null,
                    error = "SAF accessibility check failed: ${e.message}"
                )
            }
        } else {
            FileAccessibilityInfo(
                exists = false,
                canRead = false,
                canWrite = false,
                canOpenForRead = false,
                canCreateLockFile = false,
                size = 0,
                lastModified = 0,
                hasLockFile = false,
                lockFileContent = null,
                error = "Android SAF not available"
            )
        }
    }

    /**
     * Check if Android SAF is available in the native library
     */
    private fun isAndroidSafAvailable(): Boolean {
        return try {
            // Check if Android SAF is available through the native library
            // This checks if the SAF callbacks have been properly initialized
            com.ziplock.ffi.ZipLockNative.isAndroidSafAvailable()
        } catch (e: Exception) {
            println("FileUtils: Exception checking SAF availability: ${e.message}")
            false
        }
    }

    /**
     * Check if a URI represents a cloud storage location that requires special handling.
     * Uses patterns from the cloud storage implementation but excludes regular local file access.
     */
    fun isCloudStorageUri(uri: Uri): Boolean {
        val uriString = uri.toString()

        val cloudPatterns = listOf(
            // Android cloud storage app patterns (actual cloud storage apps)
            "/Android/data/com.google.android.apps.docs/",
            "/Android/data/com.dropbox.android/",
            "/Android/data/com.microsoft.skydrive/",
            "/Android/data/com.box.android/",
            "/Android/data/com.nextcloud.client/",

            // Generic cloud indicators in paths
            "/cloud/", "/sync/", "/googledrive/", "/dropbox/", "/onedrive/"
        )

        // Exclude regular Storage Access Framework URIs that can be resolved to real paths
        val excludePatterns = listOf(
            "content://com.android.externalstorage.documents/",
            "content://com.android.providers.downloads.documents/",
            "content://com.android.providers.media.documents/"
        )

        // Don't treat regular SAF URIs as cloud storage
        if (excludePatterns.any { pattern -> uriString.contains(pattern, ignoreCase = true) }) {
            return false
        }

        return cloudPatterns.any { pattern ->
            uriString.contains(pattern, ignoreCase = true)
        }
    }

    /**
     * Clean up temporary files created for archive operations.
     */
    fun cleanupTempFiles(context: Context) {
        try {
            File(context.cacheDir, "archives").deleteRecursively()
            File(context.cacheDir, "new_archives").deleteRecursively()
        } catch (e: Exception) {
            // Ignore cleanup errors
        }
    }

    /**
     * Manual cleanup of stale lock files for testing purposes.
     * This function attempts to remove .7z.lock files that may be preventing file access.
     */
    fun cleanupStaleLockFiles(archivePath: String): Boolean {
        return try {
            val archiveFile = File(archivePath)
            val lockFile = File("${archivePath}.lock")

            if (lockFile.exists()) {
                val content = lockFile.readText().trim()
                if (content == "ziplock") {
                    val deleted = lockFile.delete()
                    if (deleted) {
                        android.util.Log.d("FileUtils", "Successfully removed stale lock file: ${lockFile.absolutePath}")
                    } else {
                        android.util.Log.w("FileUtils", "Failed to delete lock file: ${lockFile.absolutePath}")
                    }
                    deleted
                } else {
                    android.util.Log.d("FileUtils", "Lock file exists but content is not 'ziplock': $content")
                    false
                }
            } else {
                android.util.Log.d("FileUtils", "No lock file found for: $archivePath")
                true
            }
        } catch (e: Exception) {
            android.util.Log.e("FileUtils", "Error cleaning up lock file for $archivePath: ${e.message}")
            false
        }
    }
}

/**
 * Information about a writable archive location.
 */
data class WritableArchiveInfo(
    val workingPath: String,
    val finalDestinationUri: Uri?,
    val needsCopyBack: Boolean
)

/**
 * Information about file accessibility and potential locking issues.
 */
data class FileAccessibilityInfo(
    val exists: Boolean,
    val canRead: Boolean,
    val canWrite: Boolean,
    val canOpenForRead: Boolean,
    val canCreateLockFile: Boolean,
    val size: Long,
    val lastModified: Long,
    val hasLockFile: Boolean,
    val lockFileContent: String?,
    val error: String?
) {
    val isAccessible: Boolean
        get() = exists && canRead && canOpenForRead && !hasLockFile

    val diagnosticInfo: String
        get() = buildString {
            appendLine("File Accessibility Diagnostic:")
            appendLine("  Path exists: $exists")
            appendLine("  Can read: $canRead")
            appendLine("  Can write: $canWrite")
            appendLine("  Can open for read: $canOpenForRead")
            appendLine("  Can create lock file: $canCreateLockFile")
            appendLine("  Size: $size bytes")
            appendLine("  Last modified: $lastModified")
            appendLine("  Has lock file: $hasLockFile")
            if (lockFileContent != null) {
                appendLine("  Lock file content: '$lockFileContent'")
            }
            if (error != null) {
                appendLine("  Error: $error")
            }
            appendLine("  Overall accessible: $isAccessible")
        }
}
