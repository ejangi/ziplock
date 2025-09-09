package com.ziplock.encryption

import org.apache.commons.compress.archivers.sevenz.SevenZOutputFile
import org.apache.commons.compress.archivers.sevenz.SevenZFile
import org.junit.Test
import org.junit.Assert.*
import java.io.File
import java.nio.file.Files

/**
 * Simple unit test to debug archive encryption issues
 *
 * This test directly uses Apache Commons Compress to validate that
 * encryption is working correctly at the library level.
 */
class EncryptionDebugTest {

    companion object {
        private const val TEST_PASSWORD = "TestPassword123!"
        private const val TEST_CONTENT = "This is sensitive content that should be encrypted"
        private const val TEST_FILENAME = "test_file.txt"
    }

    @Test
    fun testBasicSevenZEncryption() {
        val tempDir = Files.createTempDirectory("encryption_test").toFile()
        val archiveFile = File(tempDir, "test_encrypted.7z")

        try {
            // Create encrypted archive with password - this should automatically enable AES256 encryption
            SevenZOutputFile(archiveFile, TEST_PASSWORD.toCharArray()).use { output ->
                // Do NOT set any specific encryption methods - the constructor should handle this

                val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                entry.size = TEST_CONTENT.toByteArray().size.toLong()

                output.putArchiveEntry(entry)
                output.write(TEST_CONTENT.toByteArray())
                output.closeArchiveEntry()
            }

            assertTrue("Archive file should exist", archiveFile.exists())
            assertTrue("Archive file should have content", archiveFile.length() > 0)

            // Validate content is encrypted
            val archiveBytes = archiveFile.readBytes()
            val archiveString = String(archiveBytes, Charsets.ISO_8859_1)

            assertFalse(
                "Archive should not contain plaintext content",
                archiveString.contains(TEST_CONTENT)
            )

            // Test correct password decryption
            SevenZFile(archiveFile, TEST_PASSWORD.toCharArray()).use { sevenZFile ->
                val entry = sevenZFile.nextEntry
                assertNotNull("Should have one entry", entry)
                assertEquals("Filename should match", TEST_FILENAME, entry.name)

                val content = ByteArray(entry.size.toInt())
                sevenZFile.read(content)
                val extractedContent = String(content)

                assertEquals("Content should match after decryption", TEST_CONTENT, extractedContent)
            }

            // Test that reading content without password fails (7z allows reading headers without password)
            try {
                SevenZFile(archiveFile).use { sevenZFile ->
                    val entry = sevenZFile.nextEntry
                    assertNotNull("Should be able to read entry headers", entry)

                    // Try to read content - this should fail
                    val content = ByteArray(entry.size.toInt())
                    sevenZFile.read(content)
                    fail("Should fail when reading encrypted content without password")
                }
            } catch (e: Exception) {
                // Expected - encrypted content should fail without password
                assertTrue("Should indicate password needed",
                    e.message?.contains("password", ignoreCase = true) == true ||
                    e.message?.contains("encrypted", ignoreCase = true) == true)
            }

            // Test wrong password fails when reading content
            try {
                SevenZFile(archiveFile, "WrongPassword".toCharArray()).use { sevenZFile ->
                    val entry = sevenZFile.nextEntry
                    assertNotNull("Should be able to read entry headers with wrong password", entry)

                    // Try to read content - this should fail with wrong password
                    val content = ByteArray(entry.size.toInt())
                    sevenZFile.read(content)
                    fail("Should fail with wrong password when reading content")
                }
            } catch (e: Exception) {
                // Expected - wrong password should fail when reading content
                assertTrue("Should indicate decryption error",
                    e.message?.contains("corrupt", ignoreCase = true) == true ||
                    e.message?.contains("decrypt", ignoreCase = true) == true ||
                    e.message?.contains("auth", ignoreCase = true) == true)
            }

            println("✅ Basic SevenZ encryption test passed")

        } finally {
            // Cleanup
            tempDir.deleteRecursively()
        }
    }

    @Test
    fun testUnencryptedArchive() {
        val tempDir = Files.createTempDirectory("encryption_test").toFile()
        val archiveFile = File(tempDir, "test_unencrypted.7z")

        try {
            // Create unencrypted archive (no password)
            SevenZOutputFile(archiveFile).use { output ->
                val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                entry.size = TEST_CONTENT.toByteArray().size.toLong()

                output.putArchiveEntry(entry)
                output.write(TEST_CONTENT.toByteArray())
                output.closeArchiveEntry()
            }

            assertTrue("Archive file should exist", archiveFile.exists())
            assertTrue("Archive file should have content", archiveFile.length() > 0)

            // Test opening without password works
            SevenZFile(archiveFile).use { sevenZFile ->
                val entry = sevenZFile.nextEntry
                assertNotNull("Should have one entry", entry)
                assertEquals("Filename should match", TEST_FILENAME, entry.name)

                val content = ByteArray(entry.size.toInt())
                sevenZFile.read(content)
                val extractedContent = String(content)

                assertEquals("Content should match", TEST_CONTENT, extractedContent)
            }

            println("✅ Unencrypted archive test passed")

        } finally {
            // Cleanup
            tempDir.deleteRecursively()
        }
    }

    @Test
    fun testPasswordValidation() {
        val passwords = listOf(
            "",
            "a",
            "short",
            "medium_length_password",
            "Very_Long_Password_With_Many_Characters",
            "Special!@#$%^&*()Characters",
            TEST_PASSWORD
        )

        for (password in passwords) {
            val tempDir = Files.createTempDirectory("password_test").toFile()
            val archiveFile = File(tempDir, "test_password.7z")

            try {
                println("Testing password: '${password.take(10)}...' (length: ${password.length})")

                if (password.isEmpty()) {
                    // Test unencrypted archive
                    SevenZOutputFile(archiveFile).use { output ->
                        val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                        entry.size = TEST_CONTENT.toByteArray().size.toLong()

                        output.putArchiveEntry(entry)
                        output.write(TEST_CONTENT.toByteArray())
                        output.closeArchiveEntry()
                    }

                    // Should open without password
                    SevenZFile(archiveFile).use { sevenZFile ->
                        val entry = sevenZFile.nextEntry
                        assertNotNull("Should extract entry with no password", entry)
                    }
                } else {
                    // Test encrypted archive
                    SevenZOutputFile(archiveFile, password.toCharArray()).use { output ->
                        val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                        entry.size = TEST_CONTENT.toByteArray().size.toLong()

                        output.putArchiveEntry(entry)
                        output.write(TEST_CONTENT.toByteArray())
                        output.closeArchiveEntry()
                    }

                    // Should open with correct password
                    SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                        val entry = sevenZFile.nextEntry
                        assertNotNull("Should extract entry with correct password", entry)
                    }

                    // Should fail when reading content without password
                    try {
                        SevenZFile(archiveFile).use { sevenZFile ->
                            val entry = sevenZFile.nextEntry
                            if (entry != null) {
                                val content = ByteArray(entry.size.toInt())
                                sevenZFile.read(content)
                                fail("Should fail when reading content without password")
                            }
                        }
                    } catch (e: Exception) {
                        // Expected - encrypted content needs password
                    }

                    // Should fail when reading content with wrong password
                    try {
                        SevenZFile(archiveFile, "wrong".toCharArray()).use { sevenZFile ->
                            val entry = sevenZFile.nextEntry
                            if (entry != null) {
                                // Try to read content to trigger authentication
                                val content = ByteArray(entry.size.toInt())
                                sevenZFile.read(content)
                                fail("Should fail with wrong password")
                            }
                        }
                    } catch (e: Exception) {
                        // Expected - wrong password should fail
                    }
                }

            } finally {
                tempDir.deleteRecursively()
            }
        }

        println("✅ Password validation test passed")
    }

    @Test
    fun testDetailedEncryptionDebug() {
        val tempDir = Files.createTempDirectory("encryption_debug").toFile()
        val archiveFile = File(tempDir, "debug_encrypted.7z")

        try {
            println("=== DETAILED ENCRYPTION DEBUG ===")

            // Create encrypted archive
            println("Creating encrypted archive with password...")
            SevenZOutputFile(archiveFile, TEST_PASSWORD.toCharArray()).use { output ->
                println("SevenZOutputFile created with password: ${TEST_PASSWORD.toCharArray().size} chars")

                val entry = output.createArchiveEntry(File(TEST_FILENAME), TEST_FILENAME)
                entry.size = TEST_CONTENT.toByteArray().size.toLong()
                println("Archive entry created: ${entry.name}, size: ${entry.size}")

                output.putArchiveEntry(entry)
                output.write(TEST_CONTENT.toByteArray())
                output.closeArchiveEntry()
                println("Content written to archive")
            }

            println("Archive created. File size: ${archiveFile.length()} bytes")

            // Read raw archive bytes and check content
            val archiveBytes = archiveFile.readBytes()
            val archiveString = String(archiveBytes, Charsets.ISO_8859_1)

            println("Checking if sensitive content is visible in raw archive...")
            val containsPlaintext = archiveString.contains(TEST_CONTENT)
            println("Contains plaintext content: $containsPlaintext")

            if (containsPlaintext) {
                println("⚠️  WARNING: Archive contains plaintext content!")
                val index = archiveString.indexOf(TEST_CONTENT)
                println("Found at position: $index")
                val context = archiveString.substring(maxOf(0, index - 50), minOf(archiveString.length, index + TEST_CONTENT.length + 50))
                println("Context: ${context.map { if (it.isLetterOrDigit() || it.isWhitespace()) it else '?' }.joinToString("")}")
            }

            // Try to open with no password
            println("Attempting to open archive with NO password...")
            try {
                SevenZFile(archiveFile).use { sevenZFile ->
                    val entry = sevenZFile.nextEntry
                    if (entry != null) {
                        println("⚠️  CRITICAL: Archive opened WITHOUT password!")
                        println("Attempting to read content to verify encryption...")
                        val content = ByteArray(entry.size.toInt())
                        sevenZFile.read(content)
                        val extractedContent = String(content)
                        println("Extracted content: '$extractedContent'")
                        println("Content matches: ${extractedContent == TEST_CONTENT}")
                        fail("Archive content should not be readable without password")
                    } else {
                        println("No entries found without password")
                        fail("Archive should fail to open without password")
                    }
                }
            } catch (e: Exception) {
                println("✅ Good: Archive failed to open without password: ${e.message}")
            }

            // Try with correct password
            println("Attempting to open archive with CORRECT password...")
            try {
                SevenZFile(archiveFile, TEST_PASSWORD.toCharArray()).use { sevenZFile ->
                    val entry = sevenZFile.nextEntry
                    if (entry != null) {
                        println("✅ Archive opened with correct password")
                        val content = ByteArray(entry.size.toInt())
                        sevenZFile.read(content)
                        val extractedContent = String(content)
                        println("Extracted content matches: ${extractedContent == TEST_CONTENT}")
                    } else {
                        println("❌ No entry found in archive")
                    }
                }
            } catch (e: Exception) {
                println("❌ Failed to open with correct password: ${e.message}")
                throw e
            }

            // Try with wrong password
            println("Attempting to open archive with WRONG password...")
            try {
                SevenZFile(archiveFile, "WrongPassword123".toCharArray()).use { sevenZFile ->
                    val entry = sevenZFile.nextEntry
                    if (entry != null) {
                        println("Entry found with wrong password, trying to read content...")
                        val content = ByteArray(entry.size.toInt())
                        sevenZFile.read(content)
                        val extractedContent = String(content)
                        println("❌ CRITICAL: Content readable with wrong password: '$extractedContent'")
                        fail("Archive content should not be readable with wrong password")
                    } else {
                        println("No entries found with wrong password")
                        fail("Archive should fail completely with wrong password")
                    }
                }
            } catch (e: Exception) {
                println("✅ Good: Wrong password failed: ${e.message}")
            }

        } finally {
            tempDir.deleteRecursively()
        }
    }
}
