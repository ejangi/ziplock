package com.ziplock

import android.content.Context
import android.net.Uri
import com.ziplock.utils.FileUtils
import com.ziplock.utils.WritableArchiveInfo
import org.junit.Test
import org.junit.Assert.*
import org.mockito.Mockito.*
import java.io.File

/**
 * Unit tests for FileUtils content URI handling functionality
 */
class FileUtilsTest {

    @Test
    fun `isCloudStorageUri should detect Google Drive content URIs`() {
        val googleDriveUri = Uri.parse("content://com.android.providers.media.documents/tree/primary%3AGoogleDrive")
        assertTrue(FileUtils.isCloudStorageUri(googleDriveUri))
    }

    @Test
    fun `isCloudStorageUri should detect external storage documents`() {
        val externalStorageUri = Uri.parse("content://com.android.externalstorage.documents/tree/primary%3ADocuments")
        assertTrue(FileUtils.isCloudStorageUri(externalStorageUri))
    }

    @Test
    fun `isCloudStorageUri should detect Android cloud storage paths`() {
        val dropboxUri = Uri.parse("file:///Android/data/com.dropbox.android/files/test.7z")
        assertTrue(FileUtils.isCloudStorageUri(dropboxUri))
    }

    @Test
    fun `isCloudStorageUri should not detect local file URIs`() {
        val localUri = Uri.parse("file:///storage/emulated/0/Download/test.7z")
        assertFalse(FileUtils.isCloudStorageUri(localUri))
    }

    @Test
    fun `isCloudStorageUri should be case insensitive`() {
        val mixedCaseUri = Uri.parse("file:///android/data/com.DROPBOX.android/files/test.7z")
        assertTrue(FileUtils.isCloudStorageUri(mixedCaseUri))
    }

    @Test
    fun `getWritableArchivePath should handle file URIs directly`() {
        val mockContext = mock(Context::class.java)
        val fileUri = Uri.parse("file:///storage/emulated/0/Documents")

        val result = FileUtils.getWritableArchivePath(mockContext, fileUri, "TestArchive")

        assertEquals("/storage/emulated/0/Documents/TestArchive.7z", result.workingPath)
        assertNull(result.finalDestinationUri)
        assertFalse(result.needsCopyBack)
    }

    @Test
    fun `getWritableArchivePath should add 7z extension if missing`() {
        val mockContext = mock(Context::class.java)
        val fileUri = Uri.parse("file:///storage/emulated/0/Documents")

        val result = FileUtils.getWritableArchivePath(mockContext, fileUri, "TestArchive")

        assertTrue(result.workingPath.endsWith(".7z"))
    }

    @Test
    fun `getWritableArchivePath should not add extension if already present`() {
        val mockContext = mock(Context::class.java)
        val fileUri = Uri.parse("file:///storage/emulated/0/Documents")

        val result = FileUtils.getWritableArchivePath(mockContext, fileUri, "TestArchive.7z")

        assertEquals("/storage/emulated/0/Documents/TestArchive.7z", result.workingPath)
        assertFalse(result.workingPath.endsWith(".7z.7z"))
    }

    @Test
    fun `getWritableArchivePath should handle content URIs with copy back`() {
        val mockContext = mock(Context::class.java)
        val cacheDir = mock(File::class.java)
        `when`(mockContext.cacheDir).thenReturn(cacheDir)
        `when`(cacheDir.absolutePath).thenReturn("/data/data/com.ziplock/cache")

        val contentUri = Uri.parse("content://com.android.externalstorage.documents/tree/primary%3ADocuments")

        val result = FileUtils.getWritableArchivePath(mockContext, contentUri, "TestArchive")

        assertTrue(result.workingPath.contains("/cache/new_archives/"))
        assertTrue(result.workingPath.endsWith("_TestArchive.7z"))
        assertNotNull(result.finalDestinationUri)
        assertTrue(result.needsCopyBack)
    }

    @Test(expected = java.io.IOException::class)
    fun `getWritableArchivePath should throw exception for unsupported scheme`() {
        val mockContext = mock(Context::class.java)
        val unsupportedUri = Uri.parse("ftp://example.com/test")

        FileUtils.getWritableArchivePath(mockContext, unsupportedUri, "TestArchive")
    }

    @Test
    fun `getDisplayName should handle content URIs`() {
        val mockContext = mock(Context::class.java)
        val contentUri = Uri.parse("content://com.android.providers.media.documents/document/test")

        // Since we can't easily mock ContentResolver in unit tests,
        // we test that the method doesn't crash and returns a fallback
        val result = FileUtils.getDisplayName(mockContext, contentUri)

        // Should return the last path segment as fallback
        assertEquals("test", result)
    }

    @Test
    fun `getDisplayName should handle file URIs`() {
        val mockContext = mock(Context::class.java)
        val fileUri = Uri.parse("file:///storage/emulated/0/Documents/test.txt")

        val result = FileUtils.getDisplayName(mockContext, fileUri)

        assertEquals("test.txt", result)
    }

    @Test
    fun `getUsableFilePath should handle file URIs directly`() {
        val mockContext = mock(Context::class.java)
        val fileUri = Uri.parse("file:///storage/emulated/0/test.7z")

        val result = FileUtils.getUsableFilePath(mockContext, fileUri, "test.7z")

        assertEquals("/storage/emulated/0/test.7z", result)
    }

    @Test(expected = java.io.IOException::class)
    fun `getUsableFilePath should throw exception for invalid file URI`() {
        val mockContext = mock(Context::class.java)
        val invalidUri = Uri.parse("file://")

        FileUtils.getUsableFilePath(mockContext, invalidUri, "test.7z")
    }

    @Test(expected = java.io.IOException::class)
    fun `getUsableFilePath should throw exception for unsupported scheme`() {
        val mockContext = mock(Context::class.java)
        val unsupportedUri = Uri.parse("http://example.com/test.7z")

        FileUtils.getUsableFilePath(mockContext, unsupportedUri, "test.7z")
    }
}
