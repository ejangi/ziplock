package com.ziplock.ui.screens

import org.junit.Test
import org.junit.Assert.assertEquals

/**
 * Test for the extractUserFriendlyFileName function
 *
 * Verifies that the function correctly extracts user-friendly filenames
 * from various content URI formats and regular file paths.
 */
class ExtractFileNameTest {

    /**
     * Extract a user-friendly filename from a file path or content URI
     * (Copy of the function from RepositorySelectionScreen for testing)
     */
    private fun extractUserFriendlyFileName(path: String): String {
        return when {
            path.startsWith("content://") -> {
                // Handle Android content URIs
                try {
                    // First try to extract from the document ID part
                    val documentId = path.substringAfterLast("/")

                    // Decode URL encoding
                    val decoded = java.net.URLDecoder.decode(documentId, "UTF-8")

                    // Extract filename from various content URI formats
                    when {
                        // Format: "primary:Documents/filename.7z" or "1234:filename.7z"
                        decoded.contains(":") && decoded.contains("/") -> {
                            decoded.substringAfterLast("/")
                        }
                        // Format: "primary:filename.7z"
                        decoded.contains(":") -> {
                            decoded.substringAfterLast(":")
                        }
                        // Format: just "filename.7z"
                        decoded.contains(".") -> {
                            decoded
                        }
                        // Fallback
                        else -> "Selected Archive"
                    }
                } catch (e: Exception) {
                    // If decoding fails, try simple extraction
                    path.substringAfterLast("/").takeIf {
                        it.isNotEmpty() && it.contains(".")
                    } ?: "Selected Archive"
                }
            }
            else -> {
                // Handle regular file paths
                path.substringAfterLast("/")
            }
        }
    }

    @Test
    fun `test content URI with URL encoded path`() {
        // The exact format mentioned by the user
        val contentUri = "content://com.android.providers.media.documents/document/primary%3ADocuments%2FZipLock.7z"
        val result = extractUserFriendlyFileName(contentUri)
        assertEquals("ZipLock.7z", result)
    }

    @Test
    fun `test content URI with simple primary format`() {
        val contentUri = "content://com.android.providers.media.documents/document/primary%3AMyPasswords.7z"
        val result = extractUserFriendlyFileName(contentUri)
        assertEquals("MyPasswords.7z", result)
    }

    @Test
    fun `test content URI with nested folder structure`() {
        val contentUri = "content://com.android.providers.media.documents/document/primary%3ADownload%2FArchives%2FSecure.7z"
        val result = extractUserFriendlyFileName(contentUri)
        assertEquals("Secure.7z", result)
    }

    @Test
    fun `test regular file path`() {
        val filePath = "/storage/emulated/0/Documents/ZipLock.7z"
        val result = extractUserFriendlyFileName(filePath)
        assertEquals("ZipLock.7z", result)
    }

    @Test
    fun `test simple filename`() {
        val filePath = "MyArchive.7z"
        val result = extractUserFriendlyFileName(filePath)
        assertEquals("MyArchive.7z", result)
    }

    @Test
    fun `test content URI with numeric document ID`() {
        val contentUri = "content://com.google.android.apps.docs.files/document/1234567890"
        val result = extractUserFriendlyFileName(contentUri)
        assertEquals("Selected Archive", result) // Fallback for unrecognizable format
    }

    @Test
    fun `test Google Drive content URI`() {
        val contentUri = "content://com.google.android.apps.docs.files/document/acc%3D1%3Bdoc%3Dencoded_id_here"
        val result = extractUserFriendlyFileName(contentUri)
        // This should fallback to default since it doesn't contain filename
        assertEquals("Selected Archive", result)
    }

    @Test
    fun `test malformed content URI`() {
        val contentUri = "content://malformed/uri"
        val result = extractUserFriendlyFileName(contentUri)
        assertEquals("Selected Archive", result)
    }

    @Test
    fun `test empty path`() {
        val result = extractUserFriendlyFileName("")
        assertEquals("", result)
    }

    @Test
    fun `test Dropbox style content URI`() {
        val contentUri = "content://com.dropbox.android.documents/document/primary%3AArchives%2FWorkPasswords.7z"
        val result = extractUserFriendlyFileName(contentUri)
        assertEquals("WorkPasswords.7z", result)
    }
}
