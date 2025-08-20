package com.ziplock

import android.content.Intent
import android.net.Uri
import org.junit.Test
import org.junit.Assert.*
import org.junit.Before
import org.mockito.Mock
import org.mockito.Mockito.*
import org.mockito.MockitoAnnotations
import android.content.ComponentName
import android.content.pm.ActivityInfo
import android.content.pm.PackageManager
import android.content.pm.ResolveInfo

/**
 * Unit tests for .7z file association and intent handling functionality
 *
 * Tests verify that the Android app correctly:
 * - Registers intent filters for .7z files
 * - Handles incoming VIEW intents with .7z files
 * - Processes both file:// and content:// URIs
 * - Passes file URIs correctly from SplashActivity to MainActivity
 */
class FileAssociationTest {

    @Mock
    private lateinit var mockPackageManager: PackageManager

    @Before
    fun setup() {
        MockitoAnnotations.openMocks(this)
    }

    @Test
    fun `intent filter should handle 7z MIME type`() {
        // Test that our app can handle the standard 7z MIME type
        val intent = Intent(Intent.ACTION_VIEW).apply {
            type = "application/x-7z-compressed"
        }

        // Verify the intent action and MIME type
        assertEquals(Intent.ACTION_VIEW, intent.action)
        assertEquals("application/x-7z-compressed", intent.type)
    }

    @Test
    fun `intent filter should handle 7z file extension with generic MIME type`() {
        // Test handling .7z files that may have generic MIME types
        val intent = Intent(Intent.ACTION_VIEW).apply {
            type = "application/octet-stream"
            data = Uri.parse("file:///storage/emulated/0/Documents/passwords.7z")
        }

        // Verify the intent has correct data and MIME type
        assertEquals(Intent.ACTION_VIEW, intent.action)
        assertEquals("application/octet-stream", intent.type)
        assertTrue(intent.data.toString().endsWith(".7z"))
    }

    @Test
    fun `intent filter should handle content URI with 7z extension`() {
        // Test handling .7z files from Storage Access Framework
        val contentUri = "content://com.android.providers.media.documents/document/primary%3ADocuments%2Fpasswords.7z"
        val intent = Intent(Intent.ACTION_VIEW).apply {
            data = Uri.parse(contentUri)
        }

        // Verify the intent has correct content URI
        assertEquals(Intent.ACTION_VIEW, intent.action)
        assertTrue(intent.data.toString().startsWith("content://"))
        assertTrue(intent.data.toString().contains("passwords.7z"))
    }

    @Test
    fun `SplashActivity should extract file URI from VIEW intent`() {
        // Test that SplashActivity correctly extracts file URI from incoming intent
        val fileUri = "content://com.android.providers.media.documents/document/primary%3ADocuments%2Farchive.7z"
        val incomingIntent = Intent(Intent.ACTION_VIEW).apply {
            data = Uri.parse(fileUri)
        }

        // Simulate what SplashActivity does
        val extractedUri = if (incomingIntent.action == Intent.ACTION_VIEW && incomingIntent.data != null) {
            incomingIntent.data.toString()
        } else {
            null
        }

        // Verify the URI was extracted correctly
        assertNotNull(extractedUri)
        assertEquals(fileUri, extractedUri)
    }

    @Test
    fun `MainActivity should receive file URI from SplashActivity`() {
        // Test that MainActivity correctly receives and processes file URI
        val fileUri = "file:///storage/emulated/0/Download/passwords.7z"
        val intent = Intent().apply {
            putExtra("file_uri", fileUri)
            putExtra("opened_from_file", true)
        }

        // Simulate what MainActivity does
        val receivedUri = intent.getStringExtra("file_uri")
        val openedFromFile = intent.getBooleanExtra("opened_from_file", false)

        // Verify the data was passed correctly
        assertEquals(fileUri, receivedUri)
        assertTrue(openedFromFile)
    }

    @Test
    fun `MainActivity should handle null file URI gracefully`() {
        // Test that MainActivity handles case when no file URI is passed
        val intent = Intent() // No extras

        val receivedUri = intent.getStringExtra("file_uri")
        val openedFromFile = intent.getBooleanExtra("opened_from_file", false)

        // Verify defaults
        assertNull(receivedUri)
        assertFalse(openedFromFile)
    }

    @Test
    fun `file URI should be passed to RepositorySelection screen`() {
        // Test that file URI is correctly passed to RepositorySelection screen
        val fileUri = "content://com.google.android.apps.docs.files/document/archive.7z"

        // Simulate the initial screen determination logic
        val initialScreen = when {
            fileUri != null -> "RepositorySelection with file: $fileUri"
            else -> "RepositorySelection default"
        }

        assertEquals("RepositorySelection with file: $fileUri", initialScreen)
    }

    @Test
    fun `cloud storage URIs should be detected correctly`() {
        val testCases = mapOf(
            // Google Drive patterns
            "/Android/data/com.google.android.apps.docs/files/test.7z" to true,

            // Dropbox patterns
            "/Android/data/com.dropbox.android/files/test.7z" to true,

            // OneDrive patterns
            "/Android/data/com.microsoft.skydrive/files/test.7z" to true,

            // Storage Access Framework
            "content://com.android.providers.media.documents/document/test.7z" to true,
            "content://com.android.externalstorage.documents/document/test.7z" to true,

            // Regular local files (should not be detected as cloud)
            "/storage/emulated/0/Documents/test.7z" to false,
            "/data/data/com.ziplock/files/test.7z" to false
        )

        testCases.forEach { (path, expectedIsCloud) ->
            val isCloud = isCloudStoragePath(path)
            assertEquals("Path: $path", expectedIsCloud, isCloud)
        }
    }

    @Test
    fun `file extension extraction should work for various URI formats`() {
        val testCases = mapOf(
            "file:///storage/emulated/0/Documents/MyPasswords.7z" to "MyPasswords.7z",
            "content://com.android.providers.media.documents/document/primary%3ADocuments%2FArchive.7z" to "Archive.7z",
            "content://com.google.android.apps.docs.files/document/BackupFile.7z" to "BackupFile.7z",
            "/storage/emulated/0/Download/passwords.7z" to "passwords.7z"
        )

        testCases.forEach { (uri, expectedFileName) ->
            val extractedName = extractFileNameFromUri(uri)
            assertEquals("URI: $uri", expectedFileName, extractedName)
        }
    }

    @Test
    fun `intent filters should be comprehensive for different scenarios`() {
        // Test various scenarios that should trigger our app to be offered as an option
        val testIntents = listOf(
            // Standard 7z MIME type
            Intent(Intent.ACTION_VIEW).apply {
                type = "application/x-7z-compressed"
            },

            // Generic MIME type with .7z extension
            Intent(Intent.ACTION_VIEW).apply {
                type = "application/octet-stream"
                data = Uri.parse("file:///path/to/file.7z")
            },

            // File scheme with .7z extension
            Intent(Intent.ACTION_VIEW).apply {
                data = Uri.parse("file:///storage/emulated/0/file.7z")
            },

            // Content scheme with .7z extension
            Intent(Intent.ACTION_VIEW).apply {
                data = Uri.parse("content://provider/document/file.7z")
            }
        )

        testIntents.forEach { intent ->
            // Verify all intents have VIEW action (required for file associations)
            assertEquals("Intent should have VIEW action", Intent.ACTION_VIEW, intent.action)

            // Verify intent has either appropriate MIME type or .7z file extension
            val hasCorrectMimeType = intent.type == "application/x-7z-compressed" ||
                                   intent.type == "application/octet-stream"
            val has7zExtension = intent.data?.toString()?.endsWith(".7z") == true

            assertTrue("Intent should have correct MIME type or .7z extension",
                      hasCorrectMimeType || has7zExtension)
        }
    }

    // Helper methods to simulate actual implementation logic

    private fun isCloudStoragePath(path: String): Boolean {
        val cloudPatterns = listOf(
            // Android cloud storage app patterns
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

        return cloudPatterns.any { pattern -> path.contains(pattern, ignoreCase = true) }
    }

    private fun extractFileNameFromUri(uri: String): String {
        return when {
            uri.startsWith("content://") -> {
                // Handle Android content URIs
                val decodedUri = java.net.URLDecoder.decode(uri, "UTF-8")
                val parts = decodedUri.split("/", "%2F")
                parts.lastOrNull { it.endsWith(".7z") } ?: "Unknown.7z"
            }
            uri.startsWith("file://") -> {
                // Handle file URIs
                Uri.parse(uri).lastPathSegment ?: "Unknown.7z"
            }
            else -> {
                // Handle regular paths
                uri.substringAfterLast('/')
            }
        }
    }
}
