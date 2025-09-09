package com.ziplock.repository

import android.content.Context
import android.net.Uri
import android.util.Log
import com.ziplock.archive.FileMapManager
import com.ziplock.archive.EnhancedArchiveManager
import com.ziplock.ffi.ZipLockMobileFFI
import com.ziplock.storage.SafArchiveHandler
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable

/**
 * Mobile Repository Manager for ZipLock Unified Architecture
 *
 * This class serves as the main integration point between:
 * - Mobile FFI (handles memory operations, validation, business logic)
 * - Native Archive Manager (handles 7z archive operations)
 * - Storage Access Framework (handles file I/O with user permission)
 * - File Map Manager (handles JSON exchange with FFI)
 *
 * Architecture flow:
 * 1. User selects archive file via SAF
 * 2. Native Archive Manager extracts 7z to file map
 * 3. File Map Manager converts to JSON for FFI
 * 4. Mobile FFI loads data into memory repository
 * 5. All credential operations happen via FFI
 * 6. When saving: reverse the process
 *
 * This follows the unified architecture pattern where mobile platforms handle
 * all file I/O natively while the shared library handles only memory operations.
 */
class MobileRepositoryManager private constructor(
    private val context: Context
) {
    companion object {
        private const val TAG = "MobileRepositoryManager"

        @Volatile
        private var instance: MobileRepositoryManager? = null

        /**
         * Get singleton instance of the repository manager
         */
        fun getInstance(context: Context): MobileRepositoryManager {
            return instance ?: synchronized(this) {
                instance ?: MobileRepositoryManager(context.applicationContext).also { instance = it }
            }
        }
    }

    // Core components
    private val mobileFFI = ZipLockMobileFFI
    private val archiveManager = EnhancedArchiveManager(context)
    private val fileMapManager = FileMapManager
    private val safHandler = SafArchiveHandler(context)

    // Repository state
    private var repositoryHandle: ZipLockMobileFFI.RepositoryHandle? = null
    private var currentArchiveUri: Uri? = null
    private var currentArchiveInfo: SafArchiveHandler.ArchiveInfo? = null
    private var currentArchivePassword: String? = null
    private var isRepositoryOpen = false

    /**
     * Repository state information
     */
    @Serializable
    data class RepositoryState(
        val isOpen: Boolean,
        val isModified: Boolean,
        val credentialCount: Int,
        val archiveName: String? = null,
        val archiveSize: Long? = null,
        val lastModified: Long? = null
    )

    /**
     * Result of repository operations
     */
    sealed class RepositoryResult<T> {
        data class Success<T>(val data: T) : RepositoryResult<T>()
        data class Error<T>(val message: String, val exception: Throwable? = null) : RepositoryResult<T>()
    }

    // EncryptionValidationResult removed - EnhancedArchiveManager handles validation internally

    /**
     * Initialize the repository manager
     * Should be called once during app startup
     *
     * @return true if initialization was successful
     */
    suspend fun initialize(): Boolean = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Initializing Mobile Repository Manager")

            // Test FFI connection
            if (!mobileFFI.testConnection()) {
                Log.e(TAG, "FFI connection test failed")
                return@withContext false
            }

            // Enhanced archive manager doesn't need explicit test (constructor validates FFI)
            Log.d(TAG, "Enhanced archive manager initialized")

            Log.d(TAG, "Mobile Repository Manager initialized successfully")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize Mobile Repository Manager", e)
            false
        }
    }

    /**
     * Create a new empty repository
     *
     * @return RepositoryResult with success/error status
     */
    suspend fun createNewRepository(): RepositoryResult<Unit> = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Creating new repository")

            // Close existing repository if open
            closeRepository()

            // Create new repository handle
            val handle = ZipLockMobileFFI.RepositoryHandle.create()
                ?: return@withContext RepositoryResult.Error("Failed to create repository handle")

            // Initialize the repository
            if (!handle.initialize()) {
                handle.close()
                return@withContext RepositoryResult.Error("Failed to initialize repository")
            }

            repositoryHandle = handle
            currentArchiveUri = null
            currentArchiveInfo = null
            currentArchivePassword = null
            isRepositoryOpen = true

            Log.d(TAG, "New repository created successfully")
            RepositoryResult.Success(Unit)

        } catch (e: Exception) {
            Log.e(TAG, "Failed to create new repository", e)
            RepositoryResult.Error("Failed to create repository: ${e.message}", e)
        }
    }

    /**
     * Open an existing repository from archive URI
     *
     * @param archiveUri URI of the archive file
     * @param password Password for the archive
     * @return RepositoryResult with success/error status
     */
    suspend fun openRepository(
        archiveUri: Uri,
        password: String
    ): RepositoryResult<RepositoryState> = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Opening repository from: $archiveUri")
            Log.d(TAG, "DEBUG: Opening with password length: ${password.length}")

            // Close existing repository if open
            closeRepository()

            // Add small delay to ensure cleanup is complete
            Thread.sleep(100)

            // Validate archive file
            if (!safHandler.validateArchiveFile(archiveUri)) {
                return@withContext RepositoryResult.Error("Invalid archive file")
            }

            // Get archive information
            val archiveInfo = safHandler.getArchiveInfo(archiveUri)
                ?: return@withContext RepositoryResult.Error("Cannot access archive file")

            // Extract archive to file map
            Log.d(TAG, "ENCRYPTION DEBUG: Opening with password length: ${password.length}")
            val extractResult = archiveManager.extractArchive(archiveUri, password)
            Log.d(TAG, "ENCRYPTION DEBUG: Extract result - success: ${extractResult.success}")

            if (!extractResult.success || extractResult.fileMap == null) {
                return@withContext RepositoryResult.Error(
                    extractResult.error ?: "Failed to extract archive"
                )
            }

            // Determine encryption status
            val isEncrypted = determineArchiveEncryptionStatus(archiveUri, password)

            // Log encryption status for opened archive
            Log.i(TAG, "✅ ARCHIVE OPENED SUCCESSFULLY:")
            Log.i(TAG, "  - Password provided: ${password.isNotEmpty()}")
            Log.i(TAG, "  - Archive encrypted: $isEncrypted")
            Log.i(TAG, "  - Archive size: ${archiveInfo.size} bytes")

            // Convert base64 file map back to ByteArray map for file map manager
            val fileMapBytes = extractResult.fileMap.mapValues { (_, base64Content) ->
                android.util.Base64.decode(base64Content, android.util.Base64.NO_WRAP)
            }

            // Normalize file map to add missing required fields
            val normalizedFileMap = fileMapManager.normalizeFileMap(fileMapBytes)
            Log.d(TAG, "Original file map keys: ${fileMapBytes.keys}")
            Log.d(TAG, "Normalized file map keys: ${normalizedFileMap.keys}")

            // Validate file map structure
            val validation = fileMapManager.validateFileMap(normalizedFileMap)
            if (!validation.isValid) {
                val issuesText = validation.issues.joinToString("; ")
                Log.w(TAG, "File map validation issues: $issuesText")

                // Provide user-friendly guidance for common issues
                val userFriendlyMessage = when {
                    validation.issues.any { it.contains("missing format field") } ->
                        "This archive appears to be missing required ZipLock repository metadata. Please ensure you're opening a valid ZipLock archive."
                    validation.issues.any { it.contains("Unknown file") } ->
                        "This archive contains files that don't belong in a ZipLock repository. It may have been created incorrectly or corrupted."
                    else -> "Repository validation issues: $issuesText"
                }

                // For now, continue anyway but with warning context
                Log.w(TAG, "User guidance: $userFriendlyMessage")
            }

            // Create fresh repository handle (DO NOT call initialize() - loadFromFiles will handle initialization)
            Log.d(TAG, "DEBUG: Creating fresh repository handle")
            val handle = ZipLockMobileFFI.RepositoryHandle.create()
                ?: return@withContext RepositoryResult.Error("Failed to create repository handle")

            Log.d(TAG, "DEBUG: Created repository handle: ${handle.getHandle()}")
            Log.d(TAG, "DEBUG: Repository is initialized before loading: ${handle.isInitialized()}")

            // Load data from normalized file map - this will initialize the repository internally
            // CRITICAL: Do NOT call handle.initialize() before this - loadFromFiles requires uninitialized state
            Log.d(TAG, "Loading files into FFI. File count: ${normalizedFileMap.size}")
            Log.d(TAG, "DEBUG: File map contents:")
            normalizedFileMap.forEach { (key, value) ->
                Log.d(TAG, "DEBUG:   $key -> ${value.size} bytes")
            }

            if (normalizedFileMap.containsKey("metadata.yml")) {
                val metadataContent = String(normalizedFileMap["metadata.yml"]!!)
                Log.d(TAG, "Loading metadata.yml content (first 200 chars): ${metadataContent.take(200)}")
            }

            Log.d(TAG, "DEBUG: About to call handle.loadFromFiles with ${normalizedFileMap.size} files")
            if (!handle.loadFromFiles(normalizedFileMap)) {
                Log.e(TAG, "DEBUG: handle.loadFromFiles returned false")
                handle.close()
                return@withContext RepositoryResult.Error("Failed to load repository data - the archive may not contain a valid ZipLock repository or may be corrupted. Please ensure you're opening a ZipLock archive (.7z) with the correct format.")
            }

            Log.d(TAG, "DEBUG: handle.loadFromFiles completed successfully")
            Log.d(TAG, "DEBUG: Repository is initialized after loading: ${handle.isInitialized()}")

            // Success - store state
            repositoryHandle = handle
            currentArchiveUri = archiveUri
            currentArchiveInfo = archiveInfo
            currentArchivePassword = password
            isRepositoryOpen = true

            // Request persistent permissions
            safHandler.requestPersistentPermissions(archiveUri)

            val repositoryState = RepositoryState(
                isOpen = true,
                isModified = false,
                credentialCount = validation.credentialCount,
                archiveName = archiveInfo.displayName,
                archiveSize = archiveInfo.size,
                lastModified = archiveInfo.lastModified
            )

            Log.i(TAG, "Repository opened successfully:")
            Log.i(TAG, "  - Credentials: ${validation.credentialCount}")
            Log.i(TAG, "  - Archive: ${archiveInfo.displayName}")
            Log.i(TAG, "  - Encrypted: $isEncrypted")
            Log.i(TAG, "  - Size: ${archiveInfo.size} bytes")
            RepositoryResult.Success(repositoryState)

        } catch (e: Exception) {
            Log.e(TAG, "Failed to open repository", e)
            RepositoryResult.Error("Failed to open repository: ${e.message}", e)
        }
    }

    /**
     * Save the current repository to its archive file
     *
     * @param password Password for the archive (if different from open password)
     * @return RepositoryResult with success/error status
     */
    suspend fun saveRepository(password: String? = null): RepositoryResult<Unit> = withContext(Dispatchers.IO) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            val archiveUri = currentArchiveUri
                ?: return@withContext RepositoryResult.Error("No archive file associated")

            Log.d(TAG, "Saving repository to: $archiveUri")

            // Get current repository data as file map
            val fileMapBytes = handle.serializeToFiles()
                ?: return@withContext RepositoryResult.Error("Failed to serialize repository data")

            // Convert ByteArray map to base64 string map for enhanced archive manager
            val fileMap = fileMapBytes.mapValues { (_, content) ->
                android.util.Base64.encodeToString(content, android.util.Base64.NO_WRAP)
            }

            // Create encrypted archive using enhanced approach (temp file + SAF move)
            val archivePassword = password ?: currentArchivePassword ?: ""  // Use provided password, stored password, or empty string
            Log.d(TAG, "ENHANCED SAVE: Using EnhancedArchiveManager with password length: ${archivePassword.length}")

            val createResult = archiveManager.createAndSaveArchive(fileMap, archivePassword, archiveUri)
            Log.d(TAG, "ENHANCED SAVE: Result - success: ${createResult.success}, encrypted: ${createResult.isEncrypted}")

            if (!createResult.success) {
                return@withContext RepositoryResult.Error(
                    createResult.error ?: "Failed to create encrypted archive"
                )
            }

            // Enhanced archive manager already validates encryption during creation
            Log.i(TAG, "✅ ENHANCED SAVE COMPLETED:")
            Log.i(TAG, "  - Password provided: ${archivePassword.isNotEmpty()}")
            Log.i(TAG, "  - Archive encrypted: ${createResult.isEncrypted}")
            Log.i(TAG, "  - Files processed: ${createResult.filesProcessed}")
            Log.i(TAG, "  - Size: ${createResult.compressedSizeBytes} bytes")
            Log.i(TAG, "  - Compression ratio: ${createResult.compressionRatio}")
            Log.i(TAG, "  - Final path: ${createResult.finalPath}")

            // Mark repository as saved
            handle.markSaved()

            // Update archive info
            currentArchiveInfo = safHandler.getArchiveInfo(archiveUri)

            Log.i(TAG, "Repository saved successfully:")
            Log.i(TAG, "  - Size: ${createResult.compressedSizeBytes} bytes")
            Log.i(TAG, "  - Compressed ratio: ${createResult.compressionRatio}")
            Log.i(TAG, "  - Encrypted: ${createResult.isEncrypted}")
            RepositoryResult.Success(Unit)

        } catch (e: Exception) {
            Log.e(TAG, "Failed to save repository", e)
            RepositoryResult.Error("Failed to save repository: ${e.message}", e)
        }
    }

    /**
     * Save the repository to a new archive file
     *
     * @param destinationUri URI where to save the archive
     * @param password Password for the new archive
     * @return RepositoryResult with success/error status
     */
    suspend fun saveRepositoryAs(
        destinationUri: Uri,
        password: String
    ): RepositoryResult<Unit> = withContext(Dispatchers.IO) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            Log.d(TAG, "Saving repository as: $destinationUri")

            // Get current repository data as file map
            val fileMapBytes = handle.serializeToFiles()
                ?: return@withContext RepositoryResult.Error("Failed to serialize repository data")

            // Convert ByteArray map to base64 string map for enhanced archive manager
            val fileMap = fileMapBytes.mapValues { (_, content) ->
                android.util.Base64.encodeToString(content, android.util.Base64.NO_WRAP)
            }

            // Create encrypted archive using enhanced approach (temp file + SAF move)
            Log.d(TAG, "ENHANCED SAVE-AS: Using EnhancedArchiveManager with password length: ${password.length}")

            val createResult = archiveManager.createAndSaveArchive(fileMap, password, destinationUri)
            Log.d(TAG, "ENHANCED SAVE-AS: Result - success: ${createResult.success}, encrypted: ${createResult.isEncrypted}")

            if (!createResult.success) {
                return@withContext RepositoryResult.Error(
                    createResult.error ?: "Failed to create encrypted archive"
                )
            }

            // Enhanced archive manager already validates encryption during creation
            Log.i(TAG, "✅ ENHANCED SAVE-AS COMPLETED:")
            Log.i(TAG, "  - Password provided: ${password.isNotEmpty()}")
            Log.i(TAG, "  - Archive encrypted: ${createResult.isEncrypted}")
            Log.i(TAG, "  - Files processed: ${createResult.filesProcessed}")
            Log.i(TAG, "  - Size: ${createResult.compressedSizeBytes} bytes")
            Log.i(TAG, "  - Compression ratio: ${createResult.compressionRatio}")
            Log.i(TAG, "  - Final path: ${createResult.finalPath}")

            // Update current archive URI and info
            currentArchiveUri = destinationUri
            currentArchiveInfo = safHandler.getArchiveInfo(destinationUri)

            // Mark repository as saved
            handle.markSaved()

            // Request persistent permissions for new location
            safHandler.requestPersistentPermissions(destinationUri)

            Log.i(TAG, "Repository saved as new file successfully")
            RepositoryResult.Success(Unit)

        } catch (e: Exception) {
            Log.e(TAG, "Failed to save repository as new file", e)
            RepositoryResult.Error("Failed to save as new file: ${e.message}", e)
        }
    }

    /**
     * Close the current repository
     */
    fun closeRepository() {
        Log.d(TAG, "Closing repository")
        Log.d(TAG, "DEBUG: Current repository handle: ${repositoryHandle?.getHandle()}")

        repositoryHandle?.let { handle ->
            Log.d(TAG, "DEBUG: Closing existing handle: ${handle.getHandle()}")
            handle.close()
            Log.d(TAG, "DEBUG: Handle closed")
        }

        repositoryHandle = null
        currentArchiveUri = null
        currentArchiveInfo = null
        currentArchivePassword = null
        isRepositoryOpen = false

        // Clean up temp files
        archiveManager.cleanup()

        Log.d(TAG, "Repository closed")
        Log.d(TAG, "DEBUG: Repository state reset")
    }

    /**
     * Add a new credential to the repository
     *
     * @param credential Credential to add
     * @return RepositoryResult with success/error status
     */
    suspend fun addCredential(
        credential: ZipLockMobileFFI.CredentialRecord
    ): RepositoryResult<Unit> = withContext(Dispatchers.Default) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            if (handle.addCredential(credential)) {
                Log.d(TAG, "Credential added: ${credential.title}")
                RepositoryResult.Success(Unit)
            } else {
                RepositoryResult.Error("Failed to add credential")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception adding credential", e)
            RepositoryResult.Error("Failed to add credential: ${e.message}", e)
        }
    }

    /**
     * Get a credential by ID
     *
     * @param credentialId ID of the credential to retrieve
     * @return RepositoryResult with credential or error
     */
    suspend fun getCredential(
        credentialId: String
    ): RepositoryResult<ZipLockMobileFFI.CredentialRecord> = withContext(Dispatchers.Default) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            val credential = handle.getCredential(credentialId)
                ?: return@withContext RepositoryResult.Error("Credential not found")

            Log.d(TAG, "Retrieved credential: ${credential.title}")
            RepositoryResult.Success(credential)
        } catch (e: Exception) {
            Log.e(TAG, "Exception getting credential", e)
            RepositoryResult.Error("Failed to get credential: ${e.message}", e)
        }
    }

    /**
     * Update an existing credential
     *
     * @param credential Updated credential
     * @return RepositoryResult with success/error status
     */
    suspend fun updateCredential(
        credential: ZipLockMobileFFI.CredentialRecord
    ): RepositoryResult<Unit> = withContext(Dispatchers.Default) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            if (handle.updateCredential(credential)) {
                Log.d(TAG, "Credential updated: ${credential.title}")
                RepositoryResult.Success(Unit)
            } else {
                RepositoryResult.Error("Failed to update credential")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception updating credential", e)
            RepositoryResult.Error("Failed to update credential: ${e.message}", e)
        }
    }

    /**
     * Delete a credential by ID
     *
     * @param credentialId ID of the credential to delete
     * @return RepositoryResult with success/error status
     */
    suspend fun deleteCredential(credentialId: String): RepositoryResult<Unit> = withContext(Dispatchers.Default) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            if (handle.deleteCredential(credentialId)) {
                Log.d(TAG, "Credential deleted: $credentialId")
                RepositoryResult.Success(Unit)
            } else {
                RepositoryResult.Error("Failed to delete credential")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception deleting credential", e)
            RepositoryResult.Error("Failed to delete credential: ${e.message}", e)
        }
    }

    /**
     * List all credentials in the repository
     *
     * @return RepositoryResult with list of credentials or error
     */
    suspend fun listCredentials(): RepositoryResult<List<ZipLockMobileFFI.CredentialRecord>> = withContext(Dispatchers.Default) {
        try {
            val handle = repositoryHandle
                ?: return@withContext RepositoryResult.Error("No repository open")

            val credentials = handle.listCredentials()
            Log.d(TAG, "Listed ${credentials.size} credentials")
            RepositoryResult.Success(credentials)
        } catch (e: Exception) {
            Log.e(TAG, "Exception listing credentials", e)
            RepositoryResult.Error("Failed to list credentials: ${e.message}", e)
        }
    }



    /**
     * Get SAF handler for file operations
     */
    fun getSafHandler(): SafArchiveHandler = safHandler

    /**
     * Check if a repository is currently open
     */
    fun isRepositoryOpen(): Boolean = isRepositoryOpen && repositoryHandle != null

    /**
     * Get current archive URI
     */
    fun getCurrentArchiveUri(): Uri? = currentArchiveUri

    /**
     * Get current archive info
     */
    fun getCurrentArchiveInfo(): SafArchiveHandler.ArchiveInfo? = currentArchiveInfo

    /**
     * Export repository to different format
     * (Future implementation for backup/export functionality)
     */
    suspend fun exportRepository(format: String): RepositoryResult<ByteArray> {
        // Placeholder for future export functionality
        return RepositoryResult.Error("Export functionality not yet implemented")
    }

    /**
     * Import repository from different format
     * (Future implementation for backup/import functionality)
     */
    suspend fun importRepository(data: ByteArray, format: String): RepositoryResult<Unit> {
        // Placeholder for future import functionality
        return RepositoryResult.Error("Import functionality not yet implemented")
    }

    /**
     * Create a new repository archive
     *
     * @param archiveUri URI where the new archive should be created
     * @param password Password to encrypt the archive
     * @return RepositoryResult with RepositoryState on success
     */
    suspend fun createRepository(
        archiveUri: Uri,
        password: String
    ): RepositoryResult<RepositoryState> = withContext(Dispatchers.IO) {
        try {
            Log.d(TAG, "Creating new repository at: $archiveUri")

            // CRITICAL DEBUG: Log encryption details
            Log.d(TAG, "ENCRYPTION DEBUG: Password provided: ${password.isNotEmpty()}")
            Log.d(TAG, "ENCRYPTION DEBUG: Password length: ${password.length}")
            if (password.isEmpty()) {
                Log.w(TAG, "⚠️ WARNING: Empty password - repository will be UNENCRYPTED!")
            } else {
                Log.d(TAG, "✓ Non-empty password - repository should be encrypted")
            }

            // Close existing repository if open
            closeRepository()

            // Create repository handle
            val handle = ZipLockMobileFFI.RepositoryHandle.create()
                ?: return@withContext RepositoryResult.Error("Failed to create repository handle")

            // Initialize empty repository
            if (!handle.initialize()) {
                handle.close()
                return@withContext RepositoryResult.Error("Failed to initialize repository")
            }

            // Get empty file map for new repository
            val fileMapBytes = handle.serializeToFiles()
                ?: return@withContext RepositoryResult.Error("Failed to serialize empty repository")

            // Convert ByteArray map to base64 string map for enhanced archive manager
            val fileMap = fileMapBytes.mapValues { (_, content) ->
                android.util.Base64.encodeToString(content, android.util.Base64.NO_WRAP)
            }

            // Create encrypted archive using enhanced approach (temp file + SAF move)
            Log.d(TAG, "ENHANCED CREATE: Using EnhancedArchiveManager with password length: ${password.length}")

            val createResult = archiveManager.createAndSaveArchive(fileMap, password, archiveUri)
            Log.d(TAG, "ENHANCED CREATE: Result - success: ${createResult.success}, encrypted: ${createResult.isEncrypted}")

            if (!createResult.success) {
                handle.close()
                return@withContext RepositoryResult.Error(
                    createResult.error ?: "Failed to create encrypted archive"
                )
            }

            // Enhanced archive manager already validates encryption during creation
            Log.i(TAG, "✅ ENHANCED CREATE COMPLETED:")
            Log.i(TAG, "  - Password provided: ${password.isNotEmpty()}")
            Log.i(TAG, "  - Archive encrypted: ${createResult.isEncrypted}")
            Log.i(TAG, "  - Files processed: ${createResult.filesProcessed}")
            Log.i(TAG, "  - Size: ${createResult.compressedSizeBytes} bytes")
            Log.i(TAG, "  - Compression ratio: ${createResult.compressionRatio}")
            Log.i(TAG, "  - Final path: ${createResult.finalPath}")

            // Get archive information
            val archiveInfo = safHandler.getArchiveInfo(archiveUri)
                ?: return@withContext RepositoryResult.Error("Failed to get archive info")

            // Success - store state
            repositoryHandle = handle
            currentArchiveUri = archiveUri
            currentArchiveInfo = archiveInfo
            currentArchivePassword = password
            isRepositoryOpen = true

            // Request persistent permissions
            safHandler.requestPersistentPermissions(archiveUri)

            val repositoryState = RepositoryState(
                isOpen = true,
                isModified = false,
                credentialCount = 0,
                archiveName = archiveInfo.displayName,
                archiveSize = archiveInfo.size,
                lastModified = archiveInfo.lastModified
            )

            Log.d(TAG, "Repository created successfully: ${archiveInfo.displayName}")
            RepositoryResult.Success(repositoryState)

        } catch (e: Exception) {
            Log.e(TAG, "Failed to create repository", e)
            RepositoryResult.Error("Failed to create repository: ${e.message}", e)
        }
    }

    /**
     * Get current repository state
     *
     * @return RepositoryResult with RepositoryState on success
     */
    suspend fun getRepositoryState(): RepositoryResult<RepositoryState> = withContext(Dispatchers.IO) {
        try {
            val handle = repositoryHandle
            val archiveUri = currentArchiveUri
            val archiveInfo = currentArchiveInfo

            if (handle != null && archiveUri != null && archiveInfo != null && isRepositoryOpen) {
                // Get current stats from repository
                val stats = handle.getStats()
                if (stats != null) {
                    RepositoryResult.Success(RepositoryState(
                        isOpen = true,
                        isModified = stats.isModified,
                        credentialCount = stats.credentialCount,
                        archiveName = archiveInfo.displayName,
                        archiveSize = archiveInfo.size,
                        lastModified = archiveInfo.lastModified
                    ))
                } else {
                    RepositoryResult.Success(RepositoryState(
                        isOpen = true,
                        isModified = false,
                        credentialCount = 0,
                        archiveName = archiveInfo.displayName,
                        archiveSize = archiveInfo.size,
                        lastModified = archiveInfo.lastModified
                    ))
                }
            } else {
                // Repository is not open
                RepositoryResult.Success(RepositoryState(
                    isOpen = false,
                    isModified = false,
                    credentialCount = 0,
                    archiveName = null,
                    archiveSize = null,
                    lastModified = null
                ))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting repository state", e)
            RepositoryResult.Error("Failed to get repository state: ${e.message}", e)
        }
    }

    // Old validation methods removed - EnhancedArchiveManager handles validation internally

    /**
     * Determine if an archive appears to be encrypted based on extraction behavior
     *
     * @param archiveUri The archive URI
     * @param password The password that was used successfully
     * @return True if the archive appears to be encrypted
     */
    private suspend fun determineArchiveEncryptionStatus(
        archiveUri: Uri,
        password: String
    ): Boolean = withContext(Dispatchers.IO) {
        try {
            // If no password was provided, assume unencrypted
            if (password.isEmpty()) {
                return@withContext false
            }

            // If a password was provided and extraction succeeded,
            // try extracting without password to see if it fails
            val emptyPasswordResult = archiveManager.extractArchive(archiveUri, "")

            // If extraction fails without password but succeeded with password,
            // the archive is likely encrypted
            return@withContext !emptyPasswordResult.success

        } catch (e: Exception) {
            // If there's an error, assume encrypted if password was provided
            return@withContext password.isNotEmpty()
        }
    }

    /**
     * Clean up resources when the manager is no longer needed
     */
    fun cleanup() {
        Log.d(TAG, "Cleaning up Mobile Repository Manager")
        closeRepository()
        archiveManager.cleanup()
    }
}
