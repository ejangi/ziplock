package com.ziplock.archive

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import org.junit.runner.RunWith
import java.io.File

/**
 * Direct Test for NativeArchiveManager Encryption Functionality
 *
 * This test directly validates the NativeArchiveManager's encryption capabilities
 * to ensure archives are properly encrypted when passwords are provided.
 */
@RunWith(AndroidJUnit4::class)
class NativeArchiveManagerEncryptionTest {

    companion object {
        private const val TAG = "NativeArchiveManagerEncryptionTest"
        private const val STRONG_PASSWORD = "TestPassword123!@#"
        private const val WRONG_PASSWORD = "WrongPassword456"
    }

    private lateinit var context: Context
    private lateinit var testDir: File
    private lateinit var archiveManager: NativeArchiveManager

    @Before
    fun setUp() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        testDir = File(context.cacheDir, "archive_encryption_test_${System.currentTimeMillis()}")
        testDir.mkdirs()
        assertTrue("Test directory should be created", testDir.exists())

        archiveManager = NativeArchiveManager(context)

        println("NativeArchiveManager Encryption Test setup complete")
        println("Test directory: ${testDir.absolutePath}")
    }

    @After
    fun tearDown() {
        try {
            if (testDir.exists()) {
                testDir.deleteRecursively()
            }
            println("Archive encryption test cleanup complete")
        } catch (e: Exception) {
            println("Cleanup warning: ${e.message}")
        }
    }

    /**
     * Test 1: Direct Encryption Validation
     *
     * Tests that the NativeArchiveManager correctly encrypts archives
     * when passwords are provided.
     */
    @Test
    fun testDirectEncryptionValidation() {
        println("\n=== Test 1: Direct Encryption Validation ===")

        // Create test content
        val testContent = """
            version: "1.0"
            format: "memory-v1"
            created_at: ${System.currentTimeMillis() / 1000}
            last_modified: ${System.currentTimeMillis() / 1000}
            credential_count: 1
            structure_version: "1.0"
            generator: "ziplock-unified"
        """.trimIndent()

        val sensitiveContent = "This is sensitive credential data that should be encrypted"

        val fileMap = mapOf(
            "metadata.yml" to testContent.toByteArray(Charsets.UTF_8),
            "credentials/test.json" to sensitiveContent.toByteArray(Charsets.UTF_8)
        )

        // Test encrypted archive creation
        println("Creating encrypted archive...")
        val encryptedResult = archiveManager.createArchive(fileMap, STRONG_PASSWORD)

        // Validate creation succeeded
        assertTrue("Encrypted archive creation should succeed", encryptedResult.success)
        assertNotNull("Archive data should be present", encryptedResult.archiveData)
        assertTrue("Archive should be marked as encrypted", encryptedResult.isEncrypted)
        assertTrue("Archive should have reasonable size", encryptedResult.archiveData!!.size > 0)

        println("✅ Encrypted archive created successfully")
        println("  - Size: ${encryptedResult.archiveData!!.size} bytes")
        println("  - Encrypted: ${encryptedResult.isEncrypted}")
        println("  - Compression ratio: ${encryptedResult.compressionRatio}")

        // Test unencrypted archive creation for comparison
        println("Creating unencrypted archive for comparison...")
        val unencryptedResult = archiveManager.createArchive(fileMap, "")

        assertTrue("Unencrypted archive creation should succeed", unencryptedResult.success)
        assertNotNull("Unencrypted archive data should be present", unencryptedResult.archiveData)
        assertFalse("Archive should NOT be marked as encrypted", unencryptedResult.isEncrypted)

        // CRITICAL: Verify archives have different content
        println("Verifying archives have different content...")
        val encryptedBytes = encryptedResult.archiveData!!
        val unencryptedBytes = unencryptedResult.archiveData!!

        assertFalse("Encrypted and unencrypted archives should have different content",
            encryptedBytes.contentEquals(unencryptedBytes))

        println("✅ Archives have different content - encryption is working")

        // Verify sensitive content is not visible in encrypted archive
        println("Checking encrypted archive for plaintext content...")
        val encryptedString = String(encryptedBytes, Charsets.ISO_8859_1)
        assertFalse("Sensitive content should not be visible in encrypted archive",
            encryptedString.contains(sensitiveContent))
        assertFalse("Metadata content should not be visible in encrypted archive",
            encryptedString.contains("memory-v1"))

        println("✅ Encrypted archive does not contain plaintext content")

        // Test extraction with correct password
        println("Testing extraction with correct password...")
        val extractResult = archiveManager.extractArchiveFromBytes(encryptedBytes, STRONG_PASSWORD)

        assertTrue("Extraction with correct password should succeed", extractResult.success)
        assertNotNull("File map should be present", extractResult.fileMap)
        assertEquals("Should extract correct number of files", 2, extractResult.fileMap!!.size)

        // Verify extracted content matches original
        assertTrue("Should contain metadata file", extractResult.fileMap!!.containsKey("metadata.yml"))
        assertTrue("Should contain credentials file", extractResult.fileMap!!.containsKey("credentials/test.json"))

        val extractedSensitive = String(extractResult.fileMap!!["credentials/test.json"]!!, Charsets.UTF_8)
        assertEquals("Extracted sensitive content should match original", sensitiveContent, extractedSensitive)

        println("✅ Extraction with correct password successful")

        // Test extraction with wrong password fails
        println("Testing extraction with wrong password...")
        val wrongPasswordResult = archiveManager.extractArchiveFromBytes(encryptedBytes, WRONG_PASSWORD)

        assertFalse("Extraction with wrong password should fail", wrongPasswordResult.success)
        assertNotNull("Should have error message", wrongPasswordResult.error)

        println("✅ Extraction with wrong password correctly failed")

        // Test extraction without password fails
        println("Testing extraction without password...")
        val noPasswordResult = archiveManager.extractArchiveFromBytes(encryptedBytes, "")

        assertFalse("Extraction without password should fail", noPasswordResult.success)
        assertNotNull("Should have error message", noPasswordResult.error)

        println("✅ Extraction without password correctly failed")

        println("✅ Direct encryption validation test PASSED!")
    }

    /**
     * Test 2: Multiple Password Lengths
     *
     * Tests encryption with various password lengths and complexities.
     */
    @Test
    fun testMultiplePasswordLengths() {
        println("\n=== Test 2: Multiple Password Lengths ===")

        val testContent = "Test content for password length validation"
        val fileMap = mapOf("test.txt" to testContent.toByteArray(Charsets.UTF_8))

        val passwords = listOf(
            "a", // Very short
            "short", // Short
            "medium_length_password", // Medium
            "Very_Long_Complex_Password_With_Numbers_123_And_Symbols_!@#", // Very long
            "特殊字符密码测试", // Unicode characters
            STRONG_PASSWORD // Original test password
        )

        for (password in passwords) {
            println("Testing password: '${password.take(10)}...' (length: ${password.length})")

            // Create encrypted archive
            val createResult = archiveManager.createArchive(fileMap, password)

            assertTrue("Archive creation should succeed with password length ${password.length}",
                createResult.success)
            assertTrue("Archive should be marked as encrypted", createResult.isEncrypted)

            // Test extraction
            val extractResult = archiveManager.extractArchiveFromBytes(
                createResult.archiveData!!, password)

            assertTrue("Extraction should succeed with correct password", extractResult.success)
            assertEquals("Should extract 1 file", 1, extractResult.fileMap!!.size)

            val extractedContent = String(extractResult.fileMap!!["test.txt"]!!, Charsets.UTF_8)
            assertEquals("Extracted content should match", testContent, extractedContent)

            // Test wrong password fails
            val wrongResult = archiveManager.extractArchiveFromBytes(
                createResult.archiveData!!, password + "_wrong")
            assertFalse("Wrong password should fail", wrongResult.success)

            println("  ✅ Password length ${password.length} works correctly")
        }

        println("✅ Multiple password lengths test PASSED!")
    }

    /**
     * Test 3: Large Content Encryption
     *
     * Tests that encryption works correctly with larger amounts of data.
     */
    @Test
    fun testLargeContentEncryption() {
        println("\n=== Test 3: Large Content Encryption ===")

        // Create larger test content
        val largeContent = StringBuilder()
        repeat(1000) { i ->
            largeContent.append("This is line $i of sensitive data that should be properly encrypted. ")
            largeContent.append("It contains credentials, passwords, and other sensitive information. ")
        }

        val largeString = largeContent.toString()
        val fileMap = mapOf(
            "large_file.txt" to largeString.toByteArray(Charsets.UTF_8),
            "metadata.yml" to "version: 1.0\ndata: sensitive".toByteArray(Charsets.UTF_8)
        )

        println("Creating encrypted archive with ${largeString.length} bytes of content...")

        val createResult = archiveManager.createArchive(fileMap, STRONG_PASSWORD)

        assertTrue("Large content archive creation should succeed", createResult.success)
        assertTrue("Archive should be marked as encrypted", createResult.isEncrypted)
        assertNotNull("Archive data should be present", createResult.archiveData)

        val archiveData = createResult.archiveData!!
        val archiveSize = archiveData.size
        println("  - Archive size: $archiveSize bytes")
        println("  - Original size: ${largeString.length + 25} bytes")
        println("  - Compression ratio: ${createResult.compressionRatio}")

        // Verify no plaintext content is visible in archive
        val archiveString = String(archiveData, Charsets.ISO_8859_1)

        // Check for sample content that should not be visible
        val sampleLines = listOf(
            "This is line 100 of sensitive data",
            "This is line 500 of sensitive data",
            "credentials, passwords, and other sensitive",
            "version: 1.0"
        )

        for (sampleLine in sampleLines) {
            assertFalse("Archive should not contain plaintext: '$sampleLine'",
                archiveString.contains(sampleLine))
        }

        println("✅ Large content properly encrypted (no plaintext found)")

        // Test extraction
        val extractResult = archiveManager.extractArchiveFromBytes(archiveData, STRONG_PASSWORD)

        assertTrue("Large content extraction should succeed", extractResult.success)
        assertEquals("Should extract 2 files", 2, extractResult.fileMap!!.size)

        val extractedLarge = String(extractResult.fileMap!!["large_file.txt"]!!, Charsets.UTF_8)
        assertEquals("Extracted large content should match", largeString, extractedLarge)

        println("✅ Large content encryption test PASSED!")
    }

    /**
     * Test 4: Edge Cases and Error Handling
     *
     * Tests various edge cases and error conditions.
     */
    @Test
    fun testEdgeCasesAndErrorHandling() {
        println("\n=== Test 4: Edge Cases and Error Handling ===")

        // Test empty file map
        println("Testing empty file map...")
        val emptyResult = archiveManager.createArchive(emptyMap(), STRONG_PASSWORD)
        assertTrue("Empty archive creation should succeed", emptyResult.success)
        assertTrue("Empty archive should still be encrypted", emptyResult.isEncrypted)

        // Test extraction of empty encrypted archive
        val extractEmptyResult = archiveManager.extractArchiveFromBytes(emptyResult.archiveData!!, STRONG_PASSWORD)
        assertTrue("Empty archive extraction should succeed", extractEmptyResult.success)
        assertEquals("Empty archive should have no files", 0, extractEmptyResult.fileMap!!.size)

        // Test very small content
        println("Testing very small content...")
        val smallFileMap = mapOf("tiny.txt" to "x".toByteArray())
        val smallResult = archiveManager.createArchive(smallFileMap, STRONG_PASSWORD)
        assertTrue("Small content encryption should succeed", smallResult.success)
        assertTrue("Small archive should be encrypted", smallResult.isEncrypted)

        // Test extraction
        val extractSmallResult = archiveManager.extractArchiveFromBytes(smallResult.archiveData!!, STRONG_PASSWORD)
        assertTrue("Small content extraction should succeed", extractSmallResult.success)
        assertEquals("Should extract small content correctly", "x",
            String(extractSmallResult.fileMap!!["tiny.txt"]!!))

        println("✅ Edge cases and error handling test PASSED!")
    }
}
