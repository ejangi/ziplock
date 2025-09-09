package com.ziplock.temp_archive_test

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.ziplock.archive.EnhancedArchiveManager
import com.ziplock.ffi.ZipLockMobileFFI
import com.ziplock.ffi.ZipLockNative
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import kotlinx.serialization.decodeFromString
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import org.junit.runner.RunWith
import android.net.Uri
import java.io.File
import java.util.Base64

/**
 * Complete FFI-Based Archive Workflow Test
 *
 * This test validates the complete end-to-end workflow for creating password-protected
 * archives using the FFI approach with temporary files and SAF operations.
 *
 * Test Flow:
 * 1. Create test credentials using Mobile FFI
 * 2. Use EnhancedArchiveManager to create encrypted archive via FFI temp file approach
 * 3. Move archive to final location using Android file operations
 * 4. Extract the archive back using FFI extraction
 * 5. Validate the complete round-trip preserves all data
 *
 * This validates the complete architecture:
 * - FFI uses sevenz-rust2 for guaranteed proper encryption
 * - Android handles file operations and SAF integration
 * - No Apache Commons Compress encryption vulnerabilities
 * - Complete unified architecture compliance
 */
@RunWith(AndroidJUnit4::class)
class CompleteFFIWorkflowTest {

    companion object {
        private const val TAG = "CompleteFFIWorkflowTest"
        private const val TEST_PASSWORD = "SecureTestPassword123!@#"
        private const val TEST_ARCHIVE_NAME = "complete_ffi_test_archive.7z"
    }

    private lateinit var context: android.content.Context
    private lateinit var testDir: File
    private lateinit var tempDir: File
    private lateinit var enhancedArchiveManager: EnhancedArchiveManager
    private val json = Json { ignoreUnknownKeys = true; encodeDefaults = true }

    // Test data that will be used throughout the workflow
    private val testCredentials = listOf(
        ZipLockNative.Credential(
            id = "test-login-1",
            title = "Test Login Account",
            credentialType = "login",
            fields = mapOf(
                "username" to ZipLockNative.FieldValue("testuser1", "username", sensitive = false),
                "password" to ZipLockNative.FieldValue("TestPassword123!", "password", sensitive = true),
                "url" to ZipLockNative.FieldValue("https://example.com", "url", sensitive = false)
            ),
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis(),
            tags = listOf("test", "complete-ffi")
        ),
        ZipLockNative.Credential(
            id = "test-note-1",
            title = "Secure Test Note",
            credentialType = "note",
            fields = mapOf(
                "content" to ZipLockNative.FieldValue("This is a secure test note with sensitive data", "note", sensitive = true)
            ),
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis(),
            tags = listOf("test", "secure", "complete-ffi")
        )
    )

    @Before
    fun setUp() {
        println("\n=== Complete FFI Workflow Test Setup ===")
        context = InstrumentationRegistry.getInstrumentation().targetContext

        // Create isolated test directories
        testDir = File(context.cacheDir, "complete_ffi_test_${System.currentTimeMillis()}")
        tempDir = File(testDir, "temp")
        testDir.mkdirs()
        tempDir.mkdirs()

        // Initialize enhanced archive manager
        enhancedArchiveManager = EnhancedArchiveManager(context)

        // Initialize mobile FFI repository
        val initResult = ZipLockNative.createNewRepository()
        assertTrue("Repository initialization failed", initResult)

        println("Setup complete - Ready for FFI workflow testing")
    }

    @After
    fun tearDown() {
        println("\n=== Complete FFI Workflow Test Cleanup ===")

        try {
            // Clean up enhanced archive manager
            enhancedArchiveManager.cleanup()

            // Clean up FFI repository
            ZipLockNative.closeRepository()

            // Clean up test files
            testDir.deleteRecursively()
            println("Cleanup complete")
        } catch (e: Exception) {
            println("Cleanup warning: ${e.message}")
        }
    }

    /**
     * Test 1: Complete FFI-Based Archive Creation and Extraction Workflow
     *
     * This is the comprehensive test that validates the entire approach:
     * - Uses FFI for all cryptographic operations (no Apache Commons Compress encryption)
     * - Tests temporary archive creation -> move -> extraction workflow
     * - Validates data integrity through complete round-trip
     */
    @Test
    fun test1_CompleteFFIWorkflow() {
        runBlocking {
            println("\n=== Test 1: Complete FFI-Based Archive Workflow ===")

            // Step 1: Add test credentials using FFI
            println("Step 1: Adding test credentials via FFI")
            for (credential in testCredentials) {
                val result = ZipLockNative.addCredential(credential)
                assertNotNull("Failed to add credential ${credential.id}", result)
            }
            println("✅ Added ${testCredentials.size} test credentials")

            // Step 2: Get repository state as file map
            println("Step 2: Serializing repository to file map")
            val fileMapBytes = ZipLockNative.getRepositoryAsFiles()
            assertNotNull("File map serialization failed", fileMapBytes)

            val fileMap = fileMapBytes!!.mapValues { (_, content) ->
                Base64.getEncoder().encodeToString(content)
            }
            assertTrue("File map should not be empty", fileMap.isNotEmpty())
            println("✅ Repository serialized to file map with ${fileMap.size} files")

            // Step 3: Create encrypted archive using FFI approach
            println("Step 3: Creating encrypted archive via FFI")
            val createResult = enhancedArchiveManager.createEncryptedArchive(
                fileMap = fileMap,
                password = TEST_PASSWORD
            )

            assertTrue("Archive creation should succeed", createResult.success)
            assertNotNull("Temporary archive path should be provided", createResult.tempFilePath)
            assertTrue("Archive should be marked as encrypted", createResult.isEncrypted)
            assertTrue("Archive size should be reasonable", createResult.compressedSizeBytes > 0)

            val tempArchivePath = createResult.tempFilePath!!
            println("✅ Encrypted archive created at: $tempArchivePath (${createResult.compressedSizeBytes} bytes)")

            // Step 4: Move archive to final location (simulating SAF operation)
            println("Step 4: Moving archive to final location")
            val finalArchivePath = File(testDir, TEST_ARCHIVE_NAME).absolutePath
            val finalArchiveUri = Uri.fromFile(File(finalArchivePath))

            val moveResult = enhancedArchiveManager.moveArchiveToDestination(
                tempArchivePath = tempArchivePath,
                destinationUri = finalArchiveUri
            )

            assertTrue("Archive move should succeed", moveResult.success)
            assertEquals("Final path should match", finalArchiveUri.toString(), moveResult.finalPath)
            assertTrue("Final archive should exist", File(finalArchivePath).exists())
            assertTrue("Temporary archive should be cleaned up", !File(tempArchivePath).exists())
            println("✅ Archive moved to final location: $finalArchivePath")

            // Step 5: Extract archive back using FFI
            println("Step 5: Extracting archive using FFI")
            val extractResult = enhancedArchiveManager.extractArchive(
                archiveUri = finalArchiveUri,
                password = TEST_PASSWORD
            )

            assertTrue("Archive extraction should succeed", extractResult.success)
            assertNotNull("Extracted file map should not be null", extractResult.fileMap)
            assertTrue("Archive should be detected as encrypted", extractResult.isEncrypted)
            assertTrue("Should have extracted files", extractResult.extractedFiles > 0)

            val extractedFileMap = extractResult.fileMap!!
            println("✅ Archive extracted: ${extractedFileMap.size} files, ${extractResult.totalSizeBytes} bytes")

            // Step 6: Verify data integrity through round-trip
            println("Step 6: Verifying data integrity")

            // Create new repository and load extracted data
            ZipLockNative.closeRepository()
            val verifyInitResult = ZipLockNative.createNewRepository()
            assertTrue("Verify repository initialization failed", verifyInitResult)

            val extractedFileMapBytes = extractedFileMap.mapValues { (_, base64) ->
                Base64.getDecoder().decode(base64)
            }
            val loadResult = ZipLockNative.loadRepositoryFromFiles(extractedFileMapBytes)
            assertTrue("Failed to load extracted data", loadResult)

            // Get credentials from verify repository
            val extractedCredentials = ZipLockNative.listCredentials()

            // Verify all credentials are preserved
            assertEquals("Should have same number of credentials", testCredentials.size, extractedCredentials.size)

            for (originalCred in testCredentials) {
                val extractedCred = extractedCredentials.find { it.id == originalCred.id }
                assertNotNull("Credential ${originalCred.id} should be preserved", extractedCred)

                assertEquals("Title should be preserved", originalCred.title, extractedCred!!.title)
                assertEquals("Type should be preserved", originalCred.credentialType, extractedCred.credentialType)
                assertEquals("Fields count should match", originalCred.fields.size, extractedCred.fields.size)

                // Verify field values (most important for security)
                for ((fieldName, originalField) in originalCred.fields) {
                    val extractedField = extractedCred.fields[fieldName]
                    assertNotNull("Field $fieldName should be preserved", extractedField)
                    assertEquals("Field value should match", originalField.value, extractedField!!.value)
                    assertEquals("Field sensitivity should match", originalField.sensitive, extractedField.sensitive)
                }
            }

            // Clean up verify repository
            ZipLockNative.closeRepository()
            println("✅ Data integrity verified - complete round-trip successful!")
        }
    }

    /**
     * Test 2: FFI Archive Creation Performance and Security Validation
     */
    @Test
    fun test2_FFIArchiveSecurityValidation() {
        runBlocking {
            println("\n=== Test 2: FFI Archive Security Validation ===")

            // Create a repository with sensitive data
            val sensitiveData = "HIGHLY_SENSITIVE_PASSWORD_DATA_${System.currentTimeMillis()}"
            val sensitiveCredential = ZipLockNative.Credential(
                id = "sensitive-test",
                title = "Sensitive Test Data",
                credentialType = "login",
                fields = mapOf(
                    "password" to ZipLockNative.FieldValue(sensitiveData, "password", sensitive = true)
                ),
                createdAt = System.currentTimeMillis(),
                lastModified = System.currentTimeMillis(),
                tags = listOf("security-test")
            )

            // Add credential
            val addResult = ZipLockNative.addCredential(sensitiveCredential)
            assertNotNull("Failed to add sensitive credential", addResult)

            // Get file map
            val fileMapBytes = ZipLockNative.getRepositoryAsFiles()
            assertNotNull("File map serialization failed", fileMapBytes)
            val fileMap = fileMapBytes!!.mapValues { (_, content) ->
                Base64.getEncoder().encodeToString(content)
            }

            // Create encrypted archive
            val createResult = enhancedArchiveManager.createEncryptedArchive(
                fileMap = fileMap,
                password = TEST_PASSWORD
            )

            assertTrue("Archive creation should succeed", createResult.success)
            assertNotNull("Temporary archive path should be provided", createResult.tempFilePath)

            // Security validation: Archive should not contain plaintext sensitive data
            val archiveBytes = File(createResult.tempFilePath!!).readBytes()
            val archiveContent = String(archiveBytes, Charsets.ISO_8859_1)

            assertFalse("Archive should not contain plaintext sensitive data",
                archiveContent.contains(sensitiveData))

            println("✅ Security validation passed - sensitive data is properly encrypted")

            // Clean up
            File(createResult.tempFilePath).delete()
        }
    }

    /**
     * Test 3: FFI Error Handling and Edge Cases
     */
    @Test
    fun test3_FFIErrorHandling() {
        runBlocking {
            println("\n=== Test 3: FFI Error Handling ===")

            // Test 3a: Invalid password during creation
            val fileMapBytes = ZipLockNative.getRepositoryAsFiles()
            val fileMap = fileMapBytes!!.mapValues { (_, content) ->
                Base64.getEncoder().encodeToString(content)
            }

            val emptyPasswordResult = enhancedArchiveManager.createEncryptedArchive(
                fileMap = fileMap,
                password = "" // Empty password should fail
            )

            assertFalse("Empty password should fail", emptyPasswordResult.success)
            println("✅ Empty password properly rejected")

            // Test 3b: Invalid archive path during extraction
            val invalidExtractionResult = enhancedArchiveManager.extractArchive(
                archiveUri = Uri.fromFile(File("/nonexistent/path.7z")),
                password = TEST_PASSWORD
            )

            assertFalse("Invalid path should fail", invalidExtractionResult.success)
            println("✅ Invalid archive path properly handled")

            // Test 3c: Wrong password during extraction
            // First create a valid archive
            val validCreateResult = enhancedArchiveManager.createEncryptedArchive(
                fileMap = fileMap,
                password = TEST_PASSWORD
            )
            assertTrue("Valid archive creation should succeed", validCreateResult.success)

            val tempArchiveUri = Uri.fromFile(File(validCreateResult.tempFilePath!!))
            val wrongPasswordResult = enhancedArchiveManager.extractArchive(
                archiveUri = tempArchiveUri,
                password = "WrongPassword123"
            )

            assertFalse("Wrong password should fail extraction", wrongPasswordResult.success)
            println("✅ Wrong password properly rejected during extraction")

            // Clean up
            File(validCreateResult.tempFilePath).delete()
        }
    }

    /**
     * Test 4: Complete Workflow Integration Test
     *
     * This test simulates the complete real-world usage pattern:
     * 1. Mobile app creates/modifies credentials
     * 2. User saves to external storage using SAF
     * 3. User later opens the same archive
     * 4. All data is preserved and accessible
     */
    @Test
    fun test4_CompleteIntegrationWorkflow() {
        runBlocking {
            println("\n=== Test 4: Complete Integration Workflow ===")

            // Phase 1: Create and save archive (simulating "Save As" operation)
            println("Phase 1: Creating and saving archive")

            // Add multiple credentials with various field types
            val mixedCredentials = listOf(
                ZipLockNative.Credential(
                    id = "integration-login",
                    title = "Integration Test Login",
                    credentialType = "login",
                    fields = mapOf(
                        "username" to ZipLockNative.FieldValue("integration_user", "username"),
                        "password" to ZipLockNative.FieldValue("IntegrationPass123!", "password", sensitive = true),
                        "url" to ZipLockNative.FieldValue("https://integration-test.com", "url"),
                        "notes" to ZipLockNative.FieldValue("Integration test notes", "text")
                    ),
                    tags = listOf("integration", "web")
                ),
                ZipLockNative.Credential(
                    id = "integration-card",
                    title = "Integration Credit Card",
                    credentialType = "card",
                    fields = mapOf(
                        "number" to ZipLockNative.FieldValue("4111111111111111", "card_number", sensitive = true),
                        "expiry" to ZipLockNative.FieldValue("12/25", "expiry"),
                        "cvv" to ZipLockNative.FieldValue("123", "cvv", sensitive = true),
                        "holder" to ZipLockNative.FieldValue("Integration Test Holder", "text")
                    ),
                    tags = listOf("integration", "financial")
                )
            )

            for (credential in mixedCredentials) {
                val result = ZipLockNative.addCredential(credential)
                assertNotNull("Failed to add credential ${credential.id}", result)
            }

            // Save using complete workflow
            val fileMapBytes = ZipLockNative.getRepositoryAsFiles()
            val fileMap = fileMapBytes!!.mapValues { (_, content) ->
                Base64.getEncoder().encodeToString(content)
            }

            val finalArchivePath = File(testDir, "integration_test_archive.7z")
            val finalArchiveUri = Uri.fromFile(finalArchivePath)

            val saveResult = enhancedArchiveManager.createAndSaveArchive(
                fileMap = fileMap,
                password = TEST_PASSWORD,
                destinationUri = finalArchiveUri
            )

            assertTrue("Complete save workflow should succeed", saveResult.success)
            assertTrue("Final archive should exist", finalArchivePath.exists())
            println("✅ Phase 1 complete - Archive saved successfully")

            // Phase 2: Open and verify archive (simulating "Open" operation)
            println("Phase 2: Opening and verifying archive")

            val extractResult = enhancedArchiveManager.extractArchive(
                archiveUri = finalArchiveUri,
                password = TEST_PASSWORD
            )

            assertTrue("Archive opening should succeed", extractResult.success)
            assertNotNull("Extracted data should not be null", extractResult.fileMap)

            // Load into new repository to verify
            ZipLockNative.closeRepository()
            val openResult = ZipLockNative.createNewRepository()
            assertTrue("Open repository should succeed", openResult)

            val extractedFileMapBytes = extractResult.fileMap!!.mapValues { (_, base64) ->
                Base64.getDecoder().decode(base64)
            }
            val loadResult = ZipLockNative.loadRepositoryFromFiles(extractedFileMapBytes)
            assertTrue("Loading extracted data should succeed", loadResult)

            // Verify all credentials are accessible
            val loadedCredentials = ZipLockNative.listCredentials()

            assertEquals("Should have all credentials", mixedCredentials.size, loadedCredentials.size)

            for (originalCred in mixedCredentials) {
                val loadedCred = loadedCredentials.find { it.id == originalCred.id }
                assertNotNull("Credential ${originalCred.id} should be loaded", loadedCred)
                assertEquals("Credential title should match", originalCred.title, loadedCred!!.title)

                // Verify sensitive fields are properly preserved
                val sensitiveFields = originalCred.fields.filter { (_, field) -> field.sensitive }
                for ((fieldName, originalField) in sensitiveFields) {
                    val loadedField = loadedCred.fields[fieldName]
                    assertNotNull("Sensitive field $fieldName should be preserved", loadedField)
                    assertEquals("Sensitive field value should match", originalField.value, loadedField!!.value)
                    assertTrue("Field should still be marked as sensitive", loadedField.sensitive)
                }
            }

            ZipLockNative.closeRepository()
            println("✅ Phase 2 complete - All data verified after round-trip")

            println("🎉 Complete integration workflow test PASSED!")
        }
    }
}
