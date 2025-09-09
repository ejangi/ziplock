package com.ziplock.encryption

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.apache.commons.compress.archivers.sevenz.SevenZFile
import org.apache.commons.compress.archivers.sevenz.SevenZOutputFile
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import org.junit.runner.RunWith
import java.io.File

/**
 * Direct Apache Commons Compress Encryption Test
 *
 * This test directly uses Apache Commons Compress to validate that
 * encryption is working correctly at the library level and identify
 * why archives are not being encrypted in the app.
 */
@RunWith(AndroidJUnit4::class)
class ApacheCommonsCompressEncryptionTest {

    companion object {
        private const val TAG = "ApacheCommonsCompressEncryptionTest"
        private const val TEST_PASSWORD = "TestPassword123!@#"
        private const val TEST_CONTENT = "This is sensitive content that MUST be encrypted"
        private const val TEST_FILENAME = "sensitive_file.txt"
    }

    private lateinit var context: Context
    private lateinit var testDir: File

    @Before
    fun setUp() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        testDir = File(context.cacheDir, "commons_compress_test_${System.currentTimeMillis()}")
        testDir.mkdirs()
        assertTrue("Test directory should be created", testDir.exists())
        println("Apache Commons Compress Encryption Test setup complete")
        println("Test directory: ${testDir.absolutePath}")
    }

    @After
    fun tearDown() {
        try {
            if (testDir.exists()) {
                testDir.deleteRecursively()
            }
            println("Apache Commons Compress test cleanup complete")
        } catch (e: Exception) {
            println("Cleanup warning: ${e.message}")
        }
    }

    /**
     * Test 1: Basic SevenZ Encryption Validation
     *
     * Tests the most basic encryption scenario to see if Apache Commons Compress
     * is actually encrypting archives when passwords are provided.
     */
    @Test
    fun testBasicSevenZEncryption() {
        println("\n=== Test 1: Basic SevenZ Encryption Validation ===")

        val encryptedFile = File(testDir, "encrypted_test.7z")
        val unencryptedFile = File(testDir, "unencrypted_test.7z")

        // Step 1: Create encrypted archive
        println("Step 1: Creating ENCRYPTED archive with password...")
        try {
            SevenZOutputFile(encryptedFile, TEST_PASSWORD.toCharArray()).use { output ->
                println("  SevenZOutputFile created with password: ${TEST_PASSWORD.toCharArray().size} chars")

                val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                entry.size = TEST_CONTENT.toByteArray().size.toLong()
                println("  Archive entry created: ${entry.name}, size: ${entry.size}")

                output.putArchiveEntry(entry)
                output.write(TEST_CONTENT.toByteArray())
                output.closeArchiveEntry()
                println("  Content written to encrypted archive")
            }
            println("  ✅ Encrypted archive created successfully: ${encryptedFile.length()} bytes")
        } catch (e: Exception) {
            fail("Failed to create encrypted archive: ${e.message}")
        }

        // Step 2: Create unencrypted archive for comparison
        println("Step 2: Creating UNENCRYPTED archive without password...")
        try {
            SevenZOutputFile(unencryptedFile).use { output ->
                println("  SevenZOutputFile created WITHOUT password")

                val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                entry.size = TEST_CONTENT.toByteArray().size.toLong()
                println("  Archive entry created: ${entry.name}, size: ${entry.size}")

                output.putArchiveEntry(entry)
                output.write(TEST_CONTENT.toByteArray())
                output.closeArchiveEntry()
                println("  Content written to unencrypted archive")
            }
            println("  ✅ Unencrypted archive created successfully: ${unencryptedFile.length()} bytes")
        } catch (e: Exception) {
            fail("Failed to create unencrypted archive: ${e.message}")
        }

        // Step 3: CRITICAL - Compare archive contents
        println("Step 3: CRITICAL - Comparing archive binary contents...")
        val encryptedBytes = encryptedFile.readBytes()
        val unencryptedBytes = unencryptedFile.readBytes()

        println("  Encrypted archive size: ${encryptedBytes.size} bytes")
        println("  Unencrypted archive size: ${unencryptedBytes.size} bytes")

        // Check if archives have different content
        if (encryptedBytes.contentEquals(unencryptedBytes)) {
            fail("🚨 CRITICAL FAILURE: Encrypted and unencrypted archives are IDENTICAL! Encryption is NOT working!")
        } else {
            println("  ✅ Archives have different binary content - encryption appears to be working")
        }

        // Step 4: Check for plaintext content in encrypted archive
        println("Step 4: Checking encrypted archive for plaintext content...")
        val encryptedString = String(encryptedBytes, Charsets.ISO_8859_1)

        if (encryptedString.contains(TEST_CONTENT)) {
            fail("🚨 CRITICAL FAILURE: Encrypted archive contains plaintext content! Raw encryption is broken!")
        } else {
            println("  ✅ Encrypted archive does NOT contain plaintext content")
        }

        // Step 5: Test encrypted archive access
        println("Step 5: Testing encrypted archive access...")

        // Test with correct password
        println("  5a: Testing with CORRECT password...")
        try {
            SevenZFile(encryptedFile, TEST_PASSWORD.toCharArray()).use { sevenZFile ->
                val entry = sevenZFile.nextEntry
                assertNotNull("Should have one entry", entry)
                assertEquals("Filename should match", TEST_FILENAME, entry.name)

                val content = ByteArray(entry.size.toInt())
                sevenZFile.read(content)
                val extractedContent = String(content)

                assertEquals("Content should match after decryption", TEST_CONTENT, extractedContent)
                println("    ✅ Correct password successfully decrypted content")
            }
        } catch (e: Exception) {
            fail("Failed to decrypt with correct password: ${e.message}")
        }

        // Test without password (should fail)
        println("  5b: Testing without password (should fail)...")
        try {
            SevenZFile(encryptedFile).use { sevenZFile ->
                val entry = sevenZFile.nextEntry
                if (entry != null) {
                    // Try to read content - this should fail for encrypted archives
                    val content = ByteArray(entry.size.toInt())
                    sevenZFile.read(content)
                    fail("🚨 CRITICAL: Encrypted archive was readable WITHOUT password!")
                }
            }
        } catch (e: Exception) {
            println("    ✅ Archive correctly failed without password: ${e.message}")
        }

        // Test with wrong password (should fail)
        println("  5c: Testing with WRONG password (should fail)...")
        try {
            SevenZFile(encryptedFile, "WrongPassword".toCharArray()).use { sevenZFile ->
                val entry = sevenZFile.nextEntry
                if (entry != null) {
                    // Try to read content - this should fail with wrong password
                    val content = ByteArray(entry.size.toInt())
                    sevenZFile.read(content)
                    fail("🚨 CRITICAL: Encrypted archive was readable with WRONG password!")
                }
            }
        } catch (e: Exception) {
            println("    ✅ Archive correctly failed with wrong password: ${e.message}")
        }

        // Step 6: Test unencrypted archive access
        println("Step 6: Testing unencrypted archive access...")
        try {
            SevenZFile(unencryptedFile).use { sevenZFile ->
                val entry = sevenZFile.nextEntry
                assertNotNull("Should have one entry", entry)
                assertEquals("Filename should match", TEST_FILENAME, entry.name)

                val content = ByteArray(entry.size.toInt())
                sevenZFile.read(content)
                val extractedContent = String(content)

                assertEquals("Content should match", TEST_CONTENT, extractedContent)
                println("  ✅ Unencrypted archive accessible without password")
            }
        } catch (e: Exception) {
            fail("Failed to access unencrypted archive: ${e.message}")
        }

        println("✅ Basic SevenZ encryption validation PASSED!")
    }

    /**
     * Test 2: Advanced Encryption Configuration
     *
     * Tests different encryption configurations to see if there are specific
     * settings needed for proper encryption.
     */
    @Test
    fun testAdvancedEncryptionConfiguration() {
        println("\n=== Test 2: Advanced Encryption Configuration ===")

        val testFile = File(testDir, "advanced_encrypted_test.7z")

        println("Creating archive with explicit encryption settings...")
        try {
            SevenZOutputFile(testFile, TEST_PASSWORD.toCharArray()).use { output ->
                // Let's see what methods are available
                println("Available methods on SevenZOutputFile:")
                println("  - Class: ${output.javaClass.name}")

                // Create entry
                val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                entry.size = TEST_CONTENT.toByteArray().size.toLong()

                output.putArchiveEntry(entry)
                output.write(TEST_CONTENT.toByteArray())
                output.closeArchiveEntry()
            }

            assertTrue("Archive should be created", testFile.exists())
            assertTrue("Archive should have content", testFile.length() > 0)

            // Test if it's actually encrypted
            val archiveBytes = testFile.readBytes()
            val archiveString = String(archiveBytes, Charsets.ISO_8859_1)

            assertFalse("Archive should not contain plaintext",
                archiveString.contains(TEST_CONTENT))

            println("✅ Advanced encryption configuration test PASSED!")

        } catch (e: Exception) {
            fail("Advanced encryption test failed: ${e.message}")
        }
    }

    /**
     * Test 3: Debugging Library Version and Capabilities
     *
     * Outputs detailed information about the Apache Commons Compress library
     * to help debug encryption issues.
     */
    @Test
    fun testLibraryVersionAndCapabilities() {
        println("\n=== Test 3: Library Version and Capabilities ===")

        try {
            // Get package info
            val packageInfo = SevenZOutputFile::class.java.getPackage()
            println("Apache Commons Compress Package Info:")
            println("  Name: ${packageInfo?.name}")
            println("  Title: ${packageInfo?.implementationTitle}")
            println("  Version: ${packageInfo?.implementationVersion}")
            println("  Vendor: ${packageInfo?.implementationVendor}")

            // Test basic functionality
            val testFile = File(testDir, "capability_test.7z")

            println("Testing basic SevenZOutputFile capabilities...")
            SevenZOutputFile(testFile).use { output ->
                println("  SevenZOutputFile created successfully")

                // Get available methods
                val methods = output.javaClass.methods
                val relevantMethods = methods.filter {
                    it.name.contains("encrypt", ignoreCase = true) ||
                    it.name.contains("password", ignoreCase = true) ||
                    it.name.contains("method", ignoreCase = true) ||
                    it.name.contains("content", ignoreCase = true)
                }

                println("  Relevant methods found:")
                relevantMethods.forEach { method ->
                    println("    - ${method.name}(${method.parameterTypes.joinToString { it.simpleName }})")
                }
            }

            println("✅ Library version and capabilities test PASSED!")

        } catch (e: Exception) {
            fail("Library capabilities test failed: ${e.message}")
        }
    }

    /**
     * Test 4: Raw File Content Analysis
     *
     * Analyzes the raw file content to understand what's actually being
     * written to the archives.
     */
    @Test
    fun testRawFileContentAnalysis() {
        println("\n=== Test 4: Raw File Content Analysis ===")

        val encryptedFile = File(testDir, "content_analysis_encrypted.7z")
        val unencryptedFile = File(testDir, "content_analysis_unencrypted.7z")

        // Create both versions
        println("Creating archives for content analysis...")

        // Encrypted version
        SevenZOutputFile(encryptedFile, TEST_PASSWORD.toCharArray()).use { output ->
            val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
            entry.size = TEST_CONTENT.toByteArray().size.toLong()
            output.putArchiveEntry(entry)
            output.write(TEST_CONTENT.toByteArray())
            output.closeArchiveEntry()
        }

        // Unencrypted version
        SevenZOutputFile(unencryptedFile).use { output ->
            val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
            entry.size = TEST_CONTENT.toByteArray().size.toLong()
            output.putArchiveEntry(entry)
            output.write(TEST_CONTENT.toByteArray())
            output.closeArchiveEntry()
        }

        // Analyze content
        val encryptedBytes = encryptedFile.readBytes()
        val unencryptedBytes = unencryptedFile.readBytes()

        println("Raw content analysis:")
        println("  Encrypted file size: ${encryptedBytes.size}")
        println("  Unencrypted file size: ${unencryptedBytes.size}")

        // Look for 7z file signatures
        val encryptedHex = encryptedBytes.take(32).joinToString(" ") {
            "%02x".format(it.toInt() and 0xFF)
        }
        val unencryptedHex = unencryptedBytes.take(32).joinToString(" ") {
            "%02x".format(it.toInt() and 0xFF)
        }

        println("  Encrypted file header (hex): $encryptedHex")
        println("  Unencrypted file header (hex): $unencryptedHex")

        // 7z files should start with "37 7A BC AF 27 1C" (7z signature)
        val sevenZSignature = byteArrayOf(0x37, 0x7A, 0xBC.toByte(), 0xAF.toByte(), 0x27, 0x1C)

        val encryptedHasSignature = encryptedBytes.take(6).toByteArray().contentEquals(sevenZSignature)
        val unencryptedHasSignature = unencryptedBytes.take(6).toByteArray().contentEquals(sevenZSignature)

        println("  Encrypted has 7z signature: $encryptedHasSignature")
        println("  Unencrypted has 7z signature: $unencryptedHasSignature")

        // Check if test content appears in either file
        val encryptedString = String(encryptedBytes, Charsets.ISO_8859_1)
        val unencryptedString = String(unencryptedBytes, Charsets.ISO_8859_1)

        val encryptedContainsText = encryptedString.contains(TEST_CONTENT)
        val unencryptedContainsText = unencryptedString.contains(TEST_CONTENT)

        println("  Encrypted contains test text: $encryptedContainsText")
        println("  Unencrypted contains test text: $unencryptedContainsText")

        // Final validation
        if (encryptedContainsText) {
            fail("🚨 CRITICAL: Encrypted archive contains plaintext! This means encryption is NOT working!")
        }

        if (!unencryptedContainsText) {
            println("⚠️  WARNING: Unencrypted archive also doesn't contain plaintext - this might indicate compression is hiding the text")
        }

        println("✅ Raw file content analysis completed!")
    }
}
