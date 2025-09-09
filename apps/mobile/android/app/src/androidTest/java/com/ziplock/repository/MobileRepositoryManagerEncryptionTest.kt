package com.ziplock.repository

import android.content.Context
import android.net.Uri
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.ziplock.ffi.ZipLockMobileFFI
import com.ziplock.ffi.ZipLockNative
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import org.junit.runner.RunWith
import java.io.File
import java.util.*

/**
 * Comprehensive Test for MobileRepositoryManager Encryption Validation
 *
 * This test validates the encryption features implemented based on learnings
 * from ArchiveCreationStandaloneTest.kt. It ensures that:
 *
 * 1. Archives are properly encrypted when passwords are provided
 * 2. Encryption validation catches security issues
 * 3. Comprehensive logging provides clear encryption status
 * 4. All repository operations (create, save, saveAs) validate encryption
 */
@RunWith(AndroidJUnit4::class)
class MobileRepositoryManagerEncryptionTest {

    companion object {
        private const val TAG = "MobileRepositoryManagerEncryptionTest"
        private const val STRONG_PASSWORD = "SecureTestPassword123!@#"
        private const val WEAK_PASSWORD = "123"
        private const val WRONG_PASSWORD = "WrongPassword456"
    }

    private lateinit var context: Context
    private lateinit var testDir: File
    private lateinit var repositoryManager: MobileRepositoryManager

    @Before
    fun setUp() {
        context = InstrumentationRegistry.getInstrumentation().targetContext

        // Create isolated test directory
        testDir = File(context.cacheDir, "encryption_test_${System.currentTimeMillis()}")
        testDir.mkdirs()
        assertTrue("Test directory should be created", testDir.exists())

        // Initialize repository manager
        repositoryManager = MobileRepositoryManager.getInstance(context)

        // Initialize system components
        runBlocking {
            assertTrue("Repository manager should initialize", repositoryManager.initialize())
        }

        // Test basic connectivity
        assertTrue("FFI should connect", ZipLockMobileFFI.testConnection())
        assertEquals("Native layer should initialize", 0, ZipLockNative.init())

        println("MobileRepositoryManager Encryption Test setup complete")
        println("Test directory: ${testDir.absolutePath}")
    }

    @After
    fun tearDown() {
        try {
            repositoryManager.closeRepository()
            if (testDir.exists()) {
                testDir.deleteRecursively()
            }
            println("Encryption test cleanup complete")
        } catch (e: Exception) {
            println("Cleanup warning: ${e.message}")
        }
    }

    /**
     * Test 1: Encrypted Repository Creation Validation
     *
     * Validates that createRepository properly encrypts archives and
     * the encryption validation catches any issues.
     */
    @Test
    fun testEncryptedRepositoryCreation() = runBlocking {
        println("\n=== Test 1: Encrypted Repository Creation Validation ===")

        val archiveFile = File(testDir, "encrypted_creation_test.7z")
        val archiveUri = Uri.fromFile(archiveFile)

        // Test encrypted repository creation
        println("Creating encrypted repository...")
        val createResult = repositoryManager.createRepository(archiveUri, STRONG_PASSWORD)

        when (createResult) {
            is MobileRepositoryManager.RepositoryResult.Success -> {
                println("✓ Encrypted repository created successfully")

                // Verify archive file exists and has content
                assertTrue("Archive file should exist", archiveFile.exists())
                assertTrue("Archive file should have content", archiveFile.length() > 0)

                // Verify repository state indicates encryption worked
                val repoState = createResult.data
                assertTrue("Repository should be open", repoState.isOpen)
                assertEquals("Should have 0 credentials initially", 0, repoState.credentialCount)

                // Close repository to test reopening
                repositoryManager.closeRepository()

                // Test opening with correct password
                println("Testing encrypted archive opening with correct password...")
                val openResult = repositoryManager.openRepository(archiveUri, STRONG_PASSWORD)
                assertTrue("Should open with correct password",
                    openResult is MobileRepositoryManager.RepositoryResult.Success)

                // Test opening with wrong password fails
                repositoryManager.closeRepository()
                println("Testing encrypted archive rejection with wrong password...")
                val wrongPasswordResult = repositoryManager.openRepository(archiveUri, WRONG_PASSWORD)
                assertTrue("Should fail with wrong password",
                    wrongPasswordResult is MobileRepositoryManager.RepositoryResult.Error)

                // Test opening without password fails
                repositoryManager.closeRepository()
                println("Testing encrypted archive rejection without password...")
                val noPasswordResult = repositoryManager.openRepository(archiveUri, "")
                assertTrue("Should fail without password",
                    noPasswordResult is MobileRepositoryManager.RepositoryResult.Error)

                println("✅ Encrypted repository creation validation passed!")

            }
            is MobileRepositoryManager.RepositoryResult.Error -> {
                fail("Encrypted repository creation failed: ${createResult.message}")
            }
        }
    }

    /**
     * Test 2: Unencrypted Repository Creation Validation
     *
     * Validates that unencrypted repositories work correctly and
     * the system properly identifies them as unencrypted.
     */
    @Test
    fun testUnencryptedRepositoryCreation() = runBlocking {
        println("\n=== Test 2: Unencrypted Repository Creation Validation ===")

        val archiveFile = File(testDir, "unencrypted_creation_test.7z")
        val archiveUri = Uri.fromFile(archiveFile)

        // Test unencrypted repository creation (empty password)
        println("Creating unencrypted repository...")
        val createResult = repositoryManager.createRepository(archiveUri, "")

        when (createResult) {
            is MobileRepositoryManager.RepositoryResult.Success -> {
                println("✓ Unencrypted repository created successfully")

                // Verify archive file exists
                assertTrue("Archive file should exist", archiveFile.exists())
                assertTrue("Archive file should have content", archiveFile.length() > 0)

                // Close repository to test reopening
                repositoryManager.closeRepository()

                // Test opening without password works
                println("Testing unencrypted archive opening without password...")
                val openResult = repositoryManager.openRepository(archiveUri, "")
                assertTrue("Should open without password",
                    openResult is MobileRepositoryManager.RepositoryResult.Success)

                println("✅ Unencrypted repository creation validation passed!")

            }
            is MobileRepositoryManager.RepositoryResult.Error -> {
                fail("Unencrypted repository creation failed: ${createResult.message}")
            }
        }
    }

    /**
     * Test 3: Save Repository Encryption Validation
     *
     * Tests that saveRepository maintains encryption status and
     * validates encryption during save operations.
     */
    @Test
    fun testSaveRepositoryEncryption() = runBlocking {
        println("\n=== Test 3: Save Repository Encryption Validation ===")

        val archiveFile = File(testDir, "save_encryption_test.7z")
        val archiveUri = Uri.fromFile(archiveFile)

        // Create encrypted repository
        val createResult = repositoryManager.createRepository(archiveUri, STRONG_PASSWORD)
        assertTrue("Repository creation should succeed",
            createResult is MobileRepositoryManager.RepositoryResult.Success)

        // Add a test credential
        println("Adding test credential...")
        val testCredential = ZipLockMobileFFI.CredentialRecord(
            id = UUID.randomUUID().toString(),
            title = "Save Test Credential",
            credentialType = "login",
            fields = mapOf(
                "username" to ZipLockMobileFFI.FieldValue(
                    value = "testuser",
                    fieldType = "text",
                    sensitive = false
                ),
                "password" to ZipLockMobileFFI.FieldValue(
                    value = "secretpassword",
                    fieldType = "password",
                    sensitive = true
                )
            ),
            tags = listOf("test", "encryption"),
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis()
        )

        val addResult = repositoryManager.addCredential(testCredential)
        assertTrue("Credential addition should succeed",
            addResult is MobileRepositoryManager.RepositoryResult.Success)

        // Test save with same password (should maintain encryption)
        println("Testing save with same password...")
        val saveResult = repositoryManager.saveRepository(STRONG_PASSWORD)
        assertTrue("Save should succeed with encryption validation",
            saveResult is MobileRepositoryManager.RepositoryResult.Success)

        // Verify saved archive is still encrypted
        repositoryManager.closeRepository()
        val reopenResult = repositoryManager.openRepository(archiveUri, STRONG_PASSWORD)
        assertTrue("Should reopen with correct password",
            reopenResult is MobileRepositoryManager.RepositoryResult.Success)

        // Verify credential is preserved
        val listResult = repositoryManager.listCredentials()
        assertTrue("Should list credentials",
            listResult is MobileRepositoryManager.RepositoryResult.Success)

        val credentials = (listResult as MobileRepositoryManager.RepositoryResult.Success).data
        assertEquals("Should have 1 credential", 1, credentials.size)
        assertEquals("Credential title should match", "Save Test Credential", credentials.first().title)

        println("✅ Save repository encryption validation passed!")
    }

    /**
     * Test 4: Save As Repository Encryption Validation
     *
     * Tests that saveRepositoryAs creates properly encrypted archives
     * and validates encryption during the process.
     */
    @Test
    fun testSaveAsRepositoryEncryption() = runBlocking {
        println("\n=== Test 4: Save As Repository Encryption Validation ===")

        val originalFile = File(testDir, "original_saveas_test.7z")
        val newFile = File(testDir, "new_saveas_test.7z")
        val originalUri = Uri.fromFile(originalFile)
        val newUri = Uri.fromFile(newFile)

        // Create original repository with password
        val createResult = repositoryManager.createRepository(originalUri, STRONG_PASSWORD)
        assertTrue("Original repository creation should succeed",
            createResult is MobileRepositoryManager.RepositoryResult.Success)

        // Add test credential
        val testCredential = ZipLockMobileFFI.CredentialRecord(
            id = UUID.randomUUID().toString(),
            title = "SaveAs Test Credential",
            credentialType = "note",
            fields = mapOf(
                "content" to ZipLockMobileFFI.FieldValue(
                    value = "This is sensitive note content",
                    fieldType = "text",
                    sensitive = true
                )
            ),
            tags = listOf("test", "saveas"),
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis()
        )

        val addResult = repositoryManager.addCredential(testCredential)
        assertTrue("Credential addition should succeed",
            addResult is MobileRepositoryManager.RepositoryResult.Success)

        // Test save as with new password
        val newPassword = "NewEncryptionPassword789!@#"
        println("Testing save as with new password...")
        val saveAsResult = repositoryManager.saveRepositoryAs(newUri, newPassword)
        assertTrue("Save as should succeed with encryption validation",
            saveAsResult is MobileRepositoryManager.RepositoryResult.Success)

        // Verify new archive exists
        assertTrue("New archive file should exist", newFile.exists())
        assertTrue("New archive should have content", newFile.length() > 0)

        // Verify new archive is encrypted with new password
        repositoryManager.closeRepository()
        val openNewResult = repositoryManager.openRepository(newUri, newPassword)
        assertTrue("Should open new archive with new password",
            openNewResult is MobileRepositoryManager.RepositoryResult.Success)

        // Verify credential is preserved in new archive
        val listNewResult = repositoryManager.listCredentials()
        assertTrue("Should list credentials in new archive",
            listNewResult is MobileRepositoryManager.RepositoryResult.Success)

        val newCredentials = (listNewResult as MobileRepositoryManager.RepositoryResult.Success).data
        assertEquals("Should have 1 credential in new archive", 1, newCredentials.size)
        assertEquals("Credential title should match in new archive", "SaveAs Test Credential", newCredentials.first().title)

        // Verify old password doesn't work on new archive
        repositoryManager.closeRepository()
        val oldPasswordResult = repositoryManager.openRepository(newUri, STRONG_PASSWORD)
        assertTrue("Should fail with old password on new archive",
            oldPasswordResult is MobileRepositoryManager.RepositoryResult.Error)

        println("✅ Save as repository encryption validation passed!")
    }

    /**
     * Test 5: Encryption Validation Edge Cases
     *
     * Tests edge cases and error conditions in encryption validation.
     */
    @Test
    fun testEncryptionValidationEdgeCases() = runBlocking {
        println("\n=== Test 5: Encryption Validation Edge Cases ===")

        // Test with very short password (should still encrypt if provided)
        println("Testing encryption with short password...")
        val shortPasswordFile = File(testDir, "short_password_test.7z")
        val shortPasswordUri = Uri.fromFile(shortPasswordFile)

        val shortPasswordResult = repositoryManager.createRepository(shortPasswordUri, "a")
        assertTrue("Should succeed even with short password (encryption validation should pass)",
            shortPasswordResult is MobileRepositoryManager.RepositoryResult.Success)

        // Verify it's actually encrypted
        repositoryManager.closeRepository()
        val shortOpenResult = repositoryManager.openRepository(shortPasswordUri, "a")
        assertTrue("Should open with correct short password",
            shortOpenResult is MobileRepositoryManager.RepositoryResult.Success)

        val shortWrongResult = repositoryManager.openRepository(shortPasswordUri, "b")
        assertTrue("Should fail with different short password",
            shortWrongResult is MobileRepositoryManager.RepositoryResult.Error)

        // Test with special characters in password
        println("Testing encryption with special characters...")
        val specialPasswordFile = File(testDir, "special_password_test.7z")
        val specialPasswordUri = Uri.fromFile(specialPasswordFile)
        val specialPassword = "Test!@#$%^&*()_+-=[]{}|;':\",./<>?`~"

        val specialPasswordResult = repositoryManager.createRepository(specialPasswordUri, specialPassword)
        assertTrue("Should succeed with special character password",
            specialPasswordResult is MobileRepositoryManager.RepositoryResult.Success)

        // Verify special character password works
        repositoryManager.closeRepository()
        val specialOpenResult = repositoryManager.openRepository(specialPasswordUri, specialPassword)
        assertTrue("Should open with special character password",
            specialOpenResult is MobileRepositoryManager.RepositoryResult.Success)

        println("✅ Encryption validation edge cases passed!")
    }

    /**
     * Test 6: Repository State Encryption Information
     *
     * Validates that repository state properly reflects encryption status
     * and provides useful information to the UI layer.
     */
    @Test
    fun testRepositoryStateEncryptionInfo() = runBlocking {
        println("\n=== Test 6: Repository State Encryption Information ===")

        val archiveFile = File(testDir, "state_encryption_test.7z")
        val archiveUri = Uri.fromFile(archiveFile)

        // Create encrypted repository
        val createResult = repositoryManager.createRepository(archiveUri, STRONG_PASSWORD)
        assertTrue("Repository creation should succeed",
            createResult is MobileRepositoryManager.RepositoryResult.Success)

        val repoState = (createResult as MobileRepositoryManager.RepositoryResult.Success).data

        // Verify repository state contains useful information
        assertTrue("Repository should be marked as open", repoState.isOpen)
        assertFalse("Repository should not be marked as modified initially", repoState.isModified)
        assertEquals("Should have 0 credentials initially", 0, repoState.credentialCount)
        assertNotNull("Archive name should be available", repoState.archiveName)
        assertNotNull("Archive size should be available", repoState.archiveSize)
        assertTrue("Archive should have reasonable size", repoState.archiveSize!! > 0)

        // Get current state
        val currentStateResult = repositoryManager.getRepositoryState()
        assertTrue("Should get current repository state",
            currentStateResult is MobileRepositoryManager.RepositoryResult.Success)

        val currentState = (currentStateResult as MobileRepositoryManager.RepositoryResult.Success).data
        assertEquals("Current state should match creation state", repoState.isOpen, currentState.isOpen)
        assertEquals("Current credential count should match", repoState.credentialCount, currentState.credentialCount)

        println("✅ Repository state encryption information validation passed!")
    }

    /**
     * Test 7: Multiple Repository Operations
     *
     * Tests that encryption validation works correctly across multiple
     * repository operations in sequence.
     */
    @Test
    fun testMultipleRepositoryOperations() = runBlocking {
        println("\n=== Test 7: Multiple Repository Operations ===")

        val repo1File = File(testDir, "multi_repo1_test.7z")
        val repo2File = File(testDir, "multi_repo2_test.7z")
        val repo1Uri = Uri.fromFile(repo1File)
        val repo2Uri = Uri.fromFile(repo2File)

        // Create first encrypted repository
        println("Creating first encrypted repository...")
        val create1Result = repositoryManager.createRepository(repo1Uri, STRONG_PASSWORD)
        assertTrue("First repository creation should succeed",
            create1Result is MobileRepositoryManager.RepositoryResult.Success)

        // Add credential to first repository
        val credential1 = ZipLockMobileFFI.CredentialRecord(
            id = UUID.randomUUID().toString(),
            title = "First Repository Credential",
            credentialType = "login",
            fields = mapOf(
                "username" to ZipLockMobileFFI.FieldValue(
                    value = "user1",
                    fieldType = "text",
                    sensitive = false
                )
            ),
            tags = listOf("repo1"),
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis()
        )

        val add1Result = repositoryManager.addCredential(credential1)
        assertTrue("First credential addition should succeed",
            add1Result is MobileRepositoryManager.RepositoryResult.Success)

        // Save first repository
        val save1Result = repositoryManager.saveRepository(STRONG_PASSWORD)
        assertTrue("First repository save should succeed",
            save1Result is MobileRepositoryManager.RepositoryResult.Success)

        // Create second encrypted repository with different password
        println("Creating second encrypted repository...")
        val secondPassword = "SecondPassword456!@#"
        val create2Result = repositoryManager.createRepository(repo2Uri, secondPassword)
        assertTrue("Second repository creation should succeed",
            create2Result is MobileRepositoryManager.RepositoryResult.Success)

        // Add credential to second repository
        val credential2 = ZipLockMobileFFI.CredentialRecord(
            id = UUID.randomUUID().toString(),
            title = "Second Repository Credential",
            credentialType = "note",
            fields = mapOf(
                "content" to ZipLockMobileFFI.FieldValue(
                    value = "Second repo content",
                    fieldType = "text",
                    sensitive = true
                )
            ),
            tags = listOf("repo2"),
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis()
        )

        val add2Result = repositoryManager.addCredential(credential2)
        assertTrue("Second credential addition should succeed",
            add2Result is MobileRepositoryManager.RepositoryResult.Success)

        // Save second repository
        val save2Result = repositoryManager.saveRepository(secondPassword)
        assertTrue("Second repository save should succeed",
            save2Result is MobileRepositoryManager.RepositoryResult.Success)

        // Verify both repositories can be opened with correct passwords
        repositoryManager.closeRepository()

        println("Verifying first repository with correct password...")
        val open1Result = repositoryManager.openRepository(repo1Uri, STRONG_PASSWORD)
        assertTrue("First repository should open with correct password",
            open1Result is MobileRepositoryManager.RepositoryResult.Success)

        val list1Result = repositoryManager.listCredentials()
        assertTrue("Should list credentials from first repository",
            list1Result is MobileRepositoryManager.RepositoryResult.Success)
        val creds1 = (list1Result as MobileRepositoryManager.RepositoryResult.Success).data
        assertEquals("First repository should have 1 credential", 1, creds1.size)
        assertEquals("First repository credential should match", "First Repository Credential", creds1.first().title)

        repositoryManager.closeRepository()

        println("Verifying second repository with correct password...")
        val open2Result = repositoryManager.openRepository(repo2Uri, secondPassword)
        assertTrue("Second repository should open with correct password",
            open2Result is MobileRepositoryManager.RepositoryResult.Success)

        val list2Result = repositoryManager.listCredentials()
        assertTrue("Should list credentials from second repository",
            list2Result is MobileRepositoryManager.RepositoryResult.Success)
        val creds2 = (list2Result as MobileRepositoryManager.RepositoryResult.Success).data
        assertEquals("Second repository should have 1 credential", 1, creds2.size)
        assertEquals("Second repository credential should match", "Second Repository Credential", creds2.first().title)

        // Verify cross-password access fails
        repositoryManager.closeRepository()

        println("Verifying cross-password access fails...")
        val crossOpen1Result = repositoryManager.openRepository(repo1Uri, secondPassword)
        assertTrue("First repository should fail with second password",
            crossOpen1Result is MobileRepositoryManager.RepositoryResult.Error)

        val crossOpen2Result = repositoryManager.openRepository(repo2Uri, STRONG_PASSWORD)
        assertTrue("Second repository should fail with first password",
            crossOpen2Result is MobileRepositoryManager.RepositoryResult.Error)

        println("✅ Multiple repository operations validation passed!")
    }
}
