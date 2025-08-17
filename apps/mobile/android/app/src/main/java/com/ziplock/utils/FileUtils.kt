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

        // For content URIs, we need to create a copy in app's private directory
        if (uri.scheme == "content") {
            return copyContentUriToPrivateFile(context, uri, fileName)
        }

        throw IOException("Unsupported URI scheme: ${uri.scheme}")
    }

    /**
     * Copy content from a content URI to a file in the app's private directory.
     * This ensures the native library can work with a real file path.
     */
    private fun copyContentUriToPrivateFile(context: Context, uri: Uri, fileName: String): String {
        // Create a unique filename to avoid conflicts
        val timestamp = System.currentTimeMillis()
        val uniqueFileName = "${timestamp}_$fileName"

        // Create file in app's private cache directory
        val privateFile = File(context.cacheDir, "archives/$uniqueFileName")

        // Ensure parent directory exists
        privateFile.parentFile?.mkdirs()

        // Copy content from URI to private file
        context.contentResolver.openInputStream(uri)?.use { inputStream ->
            FileOutputStream(privateFile).use { outputStream ->
                inputStream.copyTo(outputStream)
            }
        } ?: throw IOException("Failed to open input stream for URI: $uri")

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
     * Check if a URI represents a cloud storage location.
     * Uses patterns from the cloud storage implementation.
     */
    fun isCloudStorageUri(uri: Uri): Boolean {
        val uriString = uri.toString()

        val cloudPatterns = listOf(
            // Android cloud storage patterns
            "/Android/data/com.google.android.apps.docs/",
            "/Android/data/com.dropbox.android/",
            "/Android/data/com.microsoft.skydrive/",
            "/Android/data/com.box.android/",
            "/Android/data/com.nextcloud.client/",

            // Storage Access Framework patterns
            "content://com.android.providers.media.documents/",
            "content://com.android.externalstorage.documents/",

            // Generic cloud indicators
            "/cloud/", "/sync/", "/googledrive/", "/dropbox/", "/onedrive/"
        )

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
}

/**
 * Information about a writable archive location.
 */
data class WritableArchiveInfo(
    val workingPath: String,
    val finalDestinationUri: Uri?,
    val needsCopyBack: Boolean
)
