package com.ziplock.temp_archive_test

import android.content.Context

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.ziplock.ffi.ZipLockNative
import com.ziplock.archive.EnhancedArchiveManager
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import org.junit.runner.RunWith
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import org.apache.commons.compress.archivers.sevenz.SevenZFile
import org.apache.commons.compress.archivers.sevenz.SevenZArchiveEntry
import kotlinx.coroutines.runBlocking
import android.net.Uri as AndroidUri
import java.util.Base64
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import kotlinx.serialization.decodeFromString

/**
 * Temporary Archive Creation Validation Test
 *
 * This test validates the proposed approach of using the shared Rust library
 * to create encrypted 7z archives in temporary storage, then moving them
 * to the final location using Android's native file operations.
 *
 * Test Flow:
 * 1. Create test credentials using Mobile FFI
 * 2. Use EnhancedArchiveManager with temporary archive approach
 * 3. Validate the archive is properly encrypted using shared library
 * 4. Move archive to final location using Android file operations
 * 5. Validate the moved archive can be opened and decrypted
 */
@RunWith(AndroidJUnit4::class)
class TempArchiveCreationTest {

    companion object {
        private const val TAG = "TempArchiveCreationTest"
        private const val TEST_PASSWORD = "SecureTestPassword123!@#"
        private const val TEST_ARCHIVE_NAME = "temp_test_archive.7z"
    }

    private lateinit var context: Context
    private lateinit var testDir: File
    private lateinit var tempDir: File
    private lateinit var enhancedArchiveManager: EnhancedArchiveManager

    @Before
    fun setUp() {
        context = InstrumentationRegistry.getInstrumentation().targetContext

        // Create isolated test directories
        testDir = File(context.cacheDir, "temp_archive_test_${System.currentTimeMillis()}")
        tempDir = File(testDir, "temp")
        testDir.mkdirs()
        tempDir.mkdirs()

        assertTrue("Test directory should be created", testDir.exists())
        assertTrue("Temp directory should be created", tempDir.exists())

        // Initialize enhanced archive manager
        enhancedArchiveManager = EnhancedArchiveManager(context)

        println("=== Temporary Archive Creation Test Setup ===")
        println("Test directory: ${testDir.absolutePath}")
        println("Temp directory: ${tempDir.absolutePath}")
    }

    @After
    fun tearDown() {
        try {
            // Clean up enhanced archive manager
            enhancedArchiveManager.cleanup()

            // Clean up test files
            if (testDir.exists()) {
                testDir.deleteRecursively()
            }
            println("=== Test cleanup complete ===")
        } catch (e: Exception) {
            println("Cleanup warning: ${e.message}")
        }
    }

    /**
     * Test 1: Basic FFI Connection and Shared Library Access
     */
    @Test
    fun test1_FFIConnection() {
        println("\n=== Test 1: FFI Connection ===")

        // Verify FFI is working
        val ffiResult = ZipLockNative.init()
        assertEquals("FFI should connect successfully", 0, ffiResult)

        println("✓ FFI connection successful")
    }

    /**
     * Test 2: Create Repository with Test Data
     */
    @Test
    fun test2_CreateRepositoryWithTestData() = runBlocking {
        println("\n=== Test 2: Create Repository with Test Data ===")

        // Create repository handle
        // Use ZipLockNative high-level API instead of direct FFI calls
        val success = ZipLockNative.createNewRepository()
        assertTrue("Repository should be created successfully", success)

        // Add test credential using high-level API
        val testCredential = ZipLockNative.Credential(
            id = "test-credential-id",
            title = "Test Login",
            credentialType = "login",
            fields = mapOf(
                "username" to ZipLockNative.FieldValue("testuser@example.com", "text", sensitive = false),
                "password" to ZipLockNative.FieldValue("TestPassword123!", "password", sensitive = true)
            ),
            tags = listOf("test", "temp-archive")
        )

        val addResult = ZipLockNative.addCredential(testCredential)
        assertNotNull("Credential should be added successfully", addResult)

        // Verify credential was added
        val credentials = ZipLockNative.listCredentials()
        assertTrue("Should have at least one credential", credentials.isNotEmpty())
        assertTrue("Should have Test Login credential", credentials.any { it.title == "Test Login" })

        println("✓ Repository created with test data")
        println("✓ Added credential: Test Login")
    }

    /**
     * Test 3: Proposed Temporary Archive Creation Flow
     *
     * This is the key test that validates our proposed approach:
     * 1. Create repository with data
     * 2. Get file map from Mobile FFI
     * 3. Use a new FFI function to create encrypted archive in temp location
     * 4. Validate archive encryption
     * 5. Move to final location using Android file operations
     */
    @Test
    fun test3_TemporaryArchiveCreationFlow() {
        runBlocking {
        println("\n=== Test 3: Temporary Archive Creation Flow ===")

        // Step 1: Create repository with test data
        // Use ZipLockNative high-level API
        val success = ZipLockNative.createNewRepository()
        assertTrue("Repository should be created successfully", success)

        // Add multiple test credentials for more realistic test
        val credentials = listOf(
            ZipLockNative.Credential(
                id = "gmail-account-id",
                title = "Gmail Account",
                credentialType = "login",
                fields = mapOf(
                    "username" to ZipLockNative.FieldValue("user@gmail.com", "text", sensitive = false),
                    "password" to ZipLockNative.FieldValue("GmailPassword123!", "password", sensitive = true)
                )
            ),
            ZipLockNative.Credential(
                id = "bank-account-id",
                title = "Bank Login",
                credentialType = "login",
                fields = mapOf(
                    "username" to ZipLockNative.FieldValue("bankuser", "text", sensitive = false),
                    "password" to ZipLockNative.FieldValue("SecureBankPass456$", "password", sensitive = true),
                    "account_number" to ZipLockNative.FieldValue("123456789", "text", sensitive = true)
                )
            )
        )

        for (credential in credentials) {
            val result = ZipLockNative.addCredential(credential)
            assertNotNull("Credential should be added", result)
        }

        println("✓ Created repository with ${credentials.size} test credentials")

        // Step 2: Get current repository state as file map (simulate this for now)
        val fileMapJson = """
        {
            "metadata.yml": "dmVyc2lvbjogIjEuMCIK",
            "credentials/gmail/record.yml": "aWQ6IGdtYWlsLWFjY291bnQtaWQK",
            "credentials/bank/record.yml": "aWQ6IGJhbmstYWNjb3VudC1pZAo="
        }
        """.trimIndent()

        println("✓ Using simulated file map for testing")
        println("File map preview: ${fileMapJson.take(200)}...")

        // Step 3: Use Enhanced Archive Manager to create encrypted archive using shared library
        println("✓ Using Enhanced Archive Manager for encrypted archive creation")

        // Parse the file map JSON to get the actual file map
        val json = Json { ignoreUnknownKeys = true }
        val fileMap = json.decodeFromString<Map<String, String>>(fileMapJson)

        val createResult = enhancedArchiveManager.createEncryptedArchive(
            fileMap = fileMap,
            password = TEST_PASSWORD
        )

        assertTrue("Archive creation should succeed", createResult.success)
        assertNotNull("Temporary archive path should be provided", createResult.tempFilePath)
        assertTrue("Archive should be encrypted", createResult.isEncrypted)

        val tempArchivePath = createResult.tempFilePath!!
        assertTrue("Temporary archive file should exist", File(tempArchivePath).exists())

        println("✓ Enhanced Archive Manager created encrypted archive at: $tempArchivePath")
        println("Files processed: ${createResult.filesProcessed}")
        println("Compression ratio: ${String.format("%.2f", createResult.compressionRatio)}")

        // Step 4: Test moving to final location
        val finalArchivePath = File(testDir, TEST_ARCHIVE_NAME).absolutePath
        val finalArchiveUri = AndroidUri.fromFile(File(finalArchivePath))

        val moveResult = enhancedArchiveManager.moveArchiveToDestination(
            tempArchivePath = tempArchivePath,
            destinationUri = finalArchiveUri
        )

        assertTrue("Archive move should succeed", moveResult.success)
        assertTrue("Final archive file should exist", File(finalArchivePath).exists())
        assertFalse("Temporary file should be removed", File(tempArchivePath).exists())

        println("✓ Successfully moved archive to final location: $finalArchivePath")
        println("Final archive size: ${moveResult.sizeBytes} bytes")

        // Clean up
        ZipLockNative.closeRepository()
        }
    }

    /**
     * Test 4: Validate Enhanced Archive Encryption
     *
     * This test demonstrates that the enhanced approach properly encrypts archives
     * using the shared library's sevenz-rust2 implementation.
     */
    @Test
    fun test4_ValidateEnhancedArchiveEncryption() {
        runBlocking {
        println("\n=== Test 4: Enhanced Archive Encryption Validation ===")

        // Create test content with sensitive data
        val sensitiveData = "MyVerySecretPassword123!@#"
        val testFileMap = mapOf(
            "credentials/test/record.yml" to Base64.getEncoder().encodeToString("""
                id: test-credential-id
                title: Super Secret Bank Account
                fields:
                  password:
                    value: $sensitiveData
                    sensitive: true
            """.trimIndent().toByteArray())
        )

        println("Creating encrypted archive with sensitive content...")
        println("Content to encrypt includes: '$sensitiveData'")

        // Use Enhanced Archive Manager to create encrypted archive
        val createResult = enhancedArchiveManager.createEncryptedArchive(
            fileMap = testFileMap,
            password = TEST_PASSWORD
        )

        assertTrue("Archive creation should succeed", createResult.success)
        assertNotNull("Temporary archive path should be provided", createResult.tempFilePath)
        assertTrue("Archive should be encrypted", createResult.isEncrypted)

        val tempArchiveFile = File(createResult.tempFilePath!!)
        assertTrue("Temporary archive file should exist", tempArchiveFile.exists())

        // Read the archive file as binary and check for plaintext
        val archiveBytes = tempArchiveFile.readBytes()
        val archiveString = String(archiveBytes, Charsets.ISO_8859_1)

        // Check that sensitive content is NOT visible in the archive
        val containsPlaintext = archiveString.contains(sensitiveData)
        assertFalse("Archive should NOT contain plaintext sensitive data", containsPlaintext)

        println("✅ CONFIRMED: Enhanced approach properly encrypts sensitive data")
        println("Plaintext found in archive: $containsPlaintext")
        println("Archive size: ${archiveBytes.size} bytes")

        // Clean up
        tempArchiveFile.delete()
        }
    }

    /**
     * Test 5: Complete Archive Workflow Validation
     *
     * This test validates the complete enhanced archive creation workflow
     * including validation of proper encryption.
     */
    @Test
    fun test5_CompleteArchiveWorkflow() {
        runBlocking {
        println("\n=== Test 5: Complete Archive Workflow ===")

        // Create test file map with multiple files
        val testFileMap = mapOf(
            "metadata.yml" to Base64.getEncoder().encodeToString("""
                version: "1.0"
                created_at: ${System.currentTimeMillis()}
                credential_count: 2
            """.trimIndent().toByteArray()),

            "credentials/gmail/record.yml" to Base64.getEncoder().encodeToString("""
                id: gmail-account-id
                title: Gmail Account
                fields:
                  password:
                    value: SecretGmailPassword123!
                    sensitive: true
            """.trimIndent().toByteArray()),

            "credentials/bank/record.yml" to Base64.getEncoder().encodeToString("""
                id: bank-account-id
                title: Bank Account
                fields:
                  password:
                    value: SuperSecretBankPassword456!
                    sensitive: true
            """.trimIndent().toByteArray())
        )

        val finalArchivePath = File(testDir, "complete_workflow_test.7z")
        val finalArchiveUri = AndroidUri.fromFile(finalArchivePath)

        println("Creating archive with ${testFileMap.size} files using complete workflow...")

        // Test complete workflow: create + move to final destination
        val result = enhancedArchiveManager.createAndSaveArchive(
            fileMap = testFileMap,
            password = TEST_PASSWORD,
            destinationUri = finalArchiveUri
        )

        assertTrue("Complete workflow should succeed", result.success)
        assertTrue("Archive should be encrypted", result.isEncrypted)
        assertTrue("Final archive should exist", finalArchivePath.exists())
        assertEquals("Should process all files", testFileMap.size, result.filesProcessed)

        println("✅ Complete workflow successful")
        println("Final archive: ${finalArchivePath.absolutePath}")
        println("Archive size: ${result.compressedSizeBytes} bytes")
        println("Compression ratio: ${String.format("%.2f", result.compressionRatio)}")

        // Validate we can extract the archive back
        val extractResult = enhancedArchiveManager.extractArchive(
            archiveUri = finalArchiveUri,
            password = TEST_PASSWORD
        )

        assertTrue("Should be able to extract created archive", extractResult.success)
        assertTrue("Extracted archive should be marked as encrypted", extractResult.isEncrypted)
        assertEquals("Should extract all files", testFileMap.size, extractResult.extractedFiles)

        println("✅ Archive extraction validation passed")
        println("Extracted ${extractResult.extractedFiles} files")
        }
    }

    /**
     * Helper function to move file from temporary location to final location
     * This simulates the Android file operations that would be used.
     */
    private fun moveFileFromTempToFinal(tempPath: String, finalPath: String): Boolean {
        return try {
            val tempFile = File(tempPath)
            val finalFile = File(finalPath)

            // Ensure final directory exists
            finalFile.parentFile?.mkdirs()

            // Copy file content (simulating SAF operations)
            FileInputStream(tempFile).use { input ->
                FileOutputStream(finalFile).use { output ->
                    input.copyTo(output)
                }
            }

            // Remove temporary file
            tempFile.delete()

            true
        } catch (e: Exception) {
            println("File move failed: ${e.message}")
            false
        }
    }

    /**
     * Helper function to validate archive encryption
     */
    private fun validateArchiveEncryption(archivePath: String, password: String, expectedFiles: Set<String>): Boolean {
        return try {
            // This uses SevenZFile to validate the archive can be properly decrypted
            val archiveFile = File(archivePath)

            // Try to open with password
            SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                val foundFiles = mutableSetOf<String>()

                var entry: SevenZArchiveEntry? = sevenZFile.nextEntry
                while (entry != null) {
                    if (!entry.isDirectory) {
                        foundFiles.add(entry.name)

                        // Try to read some content to verify decryption works
                        val buffer = ByteArray(minOf(100, entry.size.toInt()))
                        sevenZFile.read(buffer)
                    }

                    entry = sevenZFile.nextEntry
                }

                // Verify all expected files are present
                val allPresent = expectedFiles.all { it in foundFiles }
                println("Archive validation: expected=${expectedFiles.size}, found=${foundFiles.size}, allPresent=$allPresent")
                allPresent
            }
        } catch (e: Exception) {
            println("Archive validation failed: ${e.message}")
            false
        }
    }
}
