package com.ziplock.helpers

import android.content.Context
import android.net.Uri
import android.util.Log
import com.ziplock.archive.EnhancedArchiveManager
import com.ziplock.repository.MobileRepositoryManager
import com.ziplock.storage.SafArchiveHandler
import com.ziplock.ffi.ZipLockMobileFFI
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

/**
 * Helper class for archive creation operations
 *
 * This class extracts the archive creation logic from ViewModels to make it more
 * testable and reusable. It provides a clean interface for creating encrypted
 * archives and handles the integration between the repository layer, archive
 * management, and storage access framework.
 */
class ArchiveCreationHelper(private val context: Context) {

    companion object {
        private const val TAG = "ArchiveCreationHelper"
    }

    private val repositoryManager: MobileRepositoryManager = MobileRepositoryManager.getInstance(context)
    private val archiveManager: EnhancedArchiveManager = EnhancedArchiveManager(context)
    private val safHandler: SafArchiveHandler = SafArchiveHandler(context)

    /**
     * Result of archive creation operation
     */
    data class CreationResult(
        val success: Boolean,
        val archivePath: String? = null,
        val archiveSize: Long = 0L,
        val error: String? = null,
        val isEncrypted: Boolean = false
    )

    /**
     * Configuration for archive creation
     */
    data class CreationConfig(
        val archiveName: String,
        val destinationUri: Uri,
        val password: String,
        val enableEncryption: Boolean = password.isNotEmpty(),
        val validateEncryption: Boolean = true
    )

    /**
     * Create a new encrypted archive repository
     *
     * This method follows the same pattern as the production code but provides
     * better error handling and validation for testing purposes.
     *
     * @param config Configuration for archive creation
     * @return CreationResult with success status and details
     */
    suspend fun createArchiveRepository(config: CreationConfig): CreationResult = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Creating archive repository: ${config.archiveName}")
            Log.d(TAG, "Encryption enabled: ${config.enableEncryption}")
            Log.d(TAG, "Password provided: ${config.password.isNotEmpty()}")

            // Ensure repository manager is initialized
            repositoryManager.initialize()

            // Close any existing repository
            repositoryManager.closeRepository()

            // Create working directory for the archive
            val workingDir = context.cacheDir.resolve("new_archives")
            workingDir.mkdirs()

            val workingFile = workingDir.resolve("${System.currentTimeMillis()}_${config.archiveName}.7z")
            val workingUri = Uri.fromFile(workingFile)

            // Create repository using the repository manager
            val createResult = repositoryManager.createRepository(workingUri, config.password)

            when (createResult) {
                is MobileRepositoryManager.RepositoryResult.Success -> {
                    Log.d(TAG, "Repository created successfully")

                    // Validate encryption if requested
                    if (config.validateEncryption && config.enableEncryption) {
                        val encryptionValid = validateArchiveEncryption(workingFile, config.password)
                        if (!encryptionValid) {
                            return@withContext CreationResult(
                                success = false,
                                error = "Archive encryption validation failed"
                            )
                        }
                        Log.d(TAG, "Archive encryption validated successfully")
                    }

                    // Copy to final destination if different from working location
                    val finalArchiveSize = if (workingUri != config.destinationUri) {
                        copyArchiveToDestination(workingFile, config.destinationUri)
                    } else {
                        workingFile.length()
                    }

                    if (finalArchiveSize <= 0) {
                        return@withContext CreationResult(
                            success = false,
                            error = "Failed to copy archive to destination"
                        )
                    }

                    CreationResult(
                        success = true,
                        archivePath = config.destinationUri.toString(),
                        archiveSize = finalArchiveSize,
                        isEncrypted = config.enableEncryption
                    )
                }
                is MobileRepositoryManager.RepositoryResult.Error -> {
                    Log.e(TAG, "Repository creation failed: ${createResult.message}")
                    CreationResult(
                        success = false,
                        error = createResult.message
                    )
                }
            }

        } catch (e: Exception) {
            Log.e(TAG, "Archive creation failed with exception", e)
            CreationResult(
                success = false,
                error = "Archive creation failed: ${e.message}"
            )
        }
    }

    /**
     * Create archive from file map (for testing purposes)
     *
     * @param fileMap Map of file paths to content
     * @param password Password for encryption
     * @return CreationResult with archive data
     */
    suspend fun createArchiveFromFiles(
        fileMap: Map<String, ByteArray>,
        password: String
    ): CreationResult = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Creating archive from ${fileMap.size} files")
            Log.d(TAG, "Password provided: ${password.isNotEmpty()}")

            // Convert ByteArray map to Base64 string map for EnhancedArchiveManager
            val base64FileMap = fileMap.mapValues { (_, content) ->
                android.util.Base64.encodeToString(content, android.util.Base64.NO_WRAP)
            }

            // Create temporary archive (test helper - doesn't need final destination)
            val tempFile = File.createTempFile("test_archive", ".7z", context.cacheDir)
            val tempUri = android.net.Uri.fromFile(tempFile)

            try {
                val createResult = archiveManager.createAndSaveArchive(base64FileMap, password, tempUri)

                if (createResult.success) {
                    val archiveSize = tempFile.length()

                    CreationResult(
                        success = true,
                        archivePath = tempFile.absolutePath,
                        archiveSize = archiveSize,
                        isEncrypted = createResult.isEncrypted,
                        error = null
                    )
                } else {
                    CreationResult(
                        success = false,
                        error = createResult.error ?: "Archive creation failed"
                    )
                }
            } finally {
                // Clean up temp file for test
                tempFile.delete()
            }

        } catch (e: Exception) {
            Log.e(TAG, "Archive creation from files failed", e)
            CreationResult(
                success = false,
                error = "Archive creation failed: ${e.message}"
            )
        }
    }

    /**
     * Test archive decryption with password
     *
     * @param archiveData Archive byte data
     * @param password Password to test
     * @return True if password can decrypt the archive
     */
    suspend fun testArchiveDecryption(
        archiveData: ByteArray,
        password: String
    ): Boolean = withContext(Dispatchers.IO) {
        try {
            // Write to temporary file
            val tempFile = context.cacheDir.resolve("test_decrypt_${System.currentTimeMillis()}.7z")
            tempFile.writeBytes(archiveData)

            try {
                // Try to extract using EnhancedArchiveManager
                val extractResult = archiveManager.extractArchive(Uri.fromFile(tempFile), password)
                return@withContext extractResult.success
            } finally {
                // Clean up temp file
                tempFile.delete()
            }

        } catch (e: Exception) {
            Log.e(TAG, "Archive decryption test failed", e)
            return@withContext false
        }
    }

    /**
     * Validate that an archive file is properly encrypted
     *
     * @param archiveFile File to validate
     * @param correctPassword The password that should work
     * @return True if archive appears to be encrypted
     */
    private suspend fun validateArchiveEncryption(
        archiveFile: File,
        correctPassword: String
    ): Boolean = withContext(Dispatchers.IO) {
        try {
            val archiveUri = Uri.fromFile(archiveFile)

            // Test 1: Correct password should work
            val correctResult = archiveManager.extractArchive(archiveUri, correctPassword)
            if (!correctResult.success) {
                Log.w(TAG, "Encryption validation failed: correct password doesn't work")
                return@withContext false
            }

            // Test 2: Wrong password should fail (if password is not empty)
            if (correctPassword.isNotEmpty()) {
                val wrongPassword = correctPassword + "_wrong"
                val wrongResult = archiveManager.extractArchive(archiveUri, wrongPassword)
                if (wrongResult.success) {
                    Log.w(TAG, "Encryption validation failed: wrong password works")
                    return@withContext false
                }

                // Test 3: Empty password should fail
                val emptyResult = archiveManager.extractArchive(archiveUri, "")
                if (emptyResult.success) {
                    Log.w(TAG, "Encryption validation failed: empty password works on encrypted archive")
                    return@withContext false
                }
            }

            return@withContext true

        } catch (e: Exception) {
            Log.e(TAG, "Archive encryption validation failed", e)
            return@withContext false
        }
    }

    /**
     * Validate that archive content is actually encrypted (not plaintext)
     *
     * @param archiveData Archive byte data
     * @param originalFiles Original file content to check for
     * @return True if content appears encrypted
     */
    private fun validateArchiveContentEncryption(
        archiveData: ByteArray,
        originalFiles: Map<String, ByteArray>
    ): Boolean {
        try {
            val archiveString = String(archiveData, Charsets.ISO_8859_1)

            // Check that original text content is not visible in the archive
            for ((path, content) in originalFiles) {
                val contentString = String(content, Charsets.UTF_8)

                // Skip binary or very short content
                if (contentString.length < 10) continue

                // Look for recognizable patterns from the content
                val lines = contentString.lines().filter { it.trim().length > 5 }
                for (line in lines.take(5)) { // Check first 5 significant lines
                    val cleanLine = line.trim()
                    if (cleanLine.isNotEmpty() && archiveString.contains(cleanLine)) {
                        Log.w(TAG, "Found unencrypted content in archive: '$cleanLine' from file: $path")
                        return false
                    }
                }
            }

            return true

        } catch (e: Exception) {
            Log.e(TAG, "Content encryption validation failed", e)
            return false
        }
    }

    /**
     * Copy archive from working location to final destination
     *
     * @param sourceFile Source archive file
     * @param destinationUri Destination URI
     * @return Size of copied archive, or 0 if failed
     */
    private suspend fun copyArchiveToDestination(
        sourceFile: File,
        destinationUri: Uri
    ): Long = withContext(Dispatchers.IO) {
        try {
            val archiveData = sourceFile.readBytes()
            val success = safHandler.writeArchiveData(archiveData, destinationUri)

            // Clean up source file
            sourceFile.delete()

            if (success) {
                archiveData.size.toLong()
            } else {
                0L
            }

        } catch (e: Exception) {
            Log.e(TAG, "Failed to copy archive to destination", e)
            0L
        }
    }
}
