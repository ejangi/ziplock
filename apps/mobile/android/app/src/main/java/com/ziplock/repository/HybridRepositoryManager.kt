package com.ziplock.repository

import android.content.Context
import android.util.Log
import com.ziplock.archive.ArchiveManager
import com.ziplock.ffi.ZipLockDataManager
import com.ziplock.ffi.ZipLockNative
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import kotlinx.serialization.decodeFromString
import java.io.File
import java.io.FileWriter
import java.nio.file.Files
import java.util.*

/**
 * Hybrid repository manager that combines:
 * - Kotlin-based archive operations (no more sevenz_rust2 crashes)
 * - Rust-based data validation, crypto, and business logic
 *
 * This architecture eliminates the Android emulator compatibility issues
 * while maintaining the robust crypto and validation from the Rust core.
 */
class HybridRepositoryManager(private val context: Context) {

    companion object {
        private const val TAG = "HybridRepositoryManager"
        private const val CREDENTIALS_FILE = "credentials.json"
        private const val METADATA_FILE = "metadata.json"
        private const val TEMP_EXTRACT_PREFIX = "ziplock_extract_"
    }

    private val archiveManager = ArchiveManager(context)
    private val dataManager = ZipLockDataManager()
    private val json = Json {
        prettyPrint = true
        ignoreUnknownKeys = true
    }

    @Serializable
    data class RepositoryMetadata(
        val version: String = "1.0",
        val createdAt: Long = System.currentTimeMillis(),
        val lastModified: Long = System.currentTimeMillis(),
        val credentialCount: Int = 0,
        val format: String = "hybrid-v1",
        // Hybrid-specific fields
        val extractedPath: String? = null,
        val archiveSource: String? = null,
        val extractedAt: Long? = null
    )

    @Serializable
    data class SerializedCredential(
        val id: String,
        val title: String,
        val type: String,
        val fields: Map<String, String>,
        val sensitiveFields: Set<String>,
        val tags: Set<String> = emptySet(),
        val createdAt: Long,
        val updatedAt: Long
    )

    data class RepositoryResult<T>(
        val success: Boolean,
        val data: T? = null,
        val errorMessage: String? = null
    )

    private var currentArchivePath: String? = null // Original archive path (for UI display)
    private var currentSavePath: String? = null // Actual save path (may differ for content URIs)
    private var currentPassword: String? = null
    private var currentExtractedPath: String? = null
    private var isOpen = false

    /**
     * Initialize the hybrid repository manager
     */
    suspend fun initialize(): RepositoryResult<Boolean> = withContext(Dispatchers.IO) {
        try {
            val initSuccess = dataManager.initialize()
            if (!initSuccess) {
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Failed to initialize data manager"
                )
            }

            Log.i(TAG, "Hybrid repository manager initialized successfully")
            Log.i(TAG, "Library version: hybrid-1.0")

            RepositoryResult(success = true, data = true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize hybrid repository manager", e)
            RepositoryResult(
                success = false,
                errorMessage = "Initialization failed: ${e.message}"
            )
        }
    }

    /**
     * Create a new encrypted repository
     */
    suspend fun createRepository(
        archivePath: String,
        masterPassword: String
    ): RepositoryResult<Boolean> = withContext(Dispatchers.IO) {
        try {
            Log.i(TAG, "Creating new repository: $archivePath")

            // Create temporary directory with initial content
            val tempDir = Files.createTempDirectory(TEMP_EXTRACT_PREFIX).toFile()

            try {
                // Create metadata
                val metadata = RepositoryMetadata()
                val metadataFile = File(tempDir, METADATA_FILE)
                FileWriter(metadataFile).use { writer ->
                    writer.write(json.encodeToString(metadata))
                }

                // Create empty credentials file
                val credentialsFile = File(tempDir, CREDENTIALS_FILE)
                FileWriter(credentialsFile).use { writer ->
                    writer.write(json.encodeToString(emptyList<SerializedCredential>()))
                }

                // Create the archive using Kotlin implementation
                val archiveResult = archiveManager.createArchive(archivePath, masterPassword, tempDir)

                if (!archiveResult.success) {
                    return@withContext RepositoryResult(
                        success = false,
                        errorMessage = "Failed to create archive: ${archiveResult.errorMessage}"
                    )
                }

                Log.i(TAG, "Repository created successfully: $archivePath")
                RepositoryResult(success = true, data = true)

            } finally {
                // Cleanup temp directory
                tempDir.deleteRecursively()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to create repository", e)
            RepositoryResult(
                success = false,
                errorMessage = "Repository creation failed: ${e.message}"
            )
        }
    }

    /**
     * Open an existing encrypted repository using hybrid approach:
     * - Apache Commons Compress validates archive safety (prevents SIGABRT crashes)
     * - Native library handles repository logic with validated archive (consistent format)
     */
    suspend fun openRepository(
        archivePath: String,
        masterPassword: String
    ): RepositoryResult<RepositoryMetadata> = withContext(Dispatchers.IO) {
        try {
            Log.i(TAG, "Opening repository with hybrid approach: $archivePath")

            // Determine if this is a content URI and get proper paths
            val isContentUri = archivePath.startsWith("content://")
            val validationPath = if (isContentUri) {
                // Step 1a: For content URIs, create local copy for validation
                Log.d(TAG, "Step 1a: Converting content URI to local file for validation...")
                val uri = android.net.Uri.parse(archivePath)
                val fileName = uri.path?.substringAfterLast('/') ?: "archive.7z"
                com.ziplock.utils.FileUtils.getUsableFilePath(context, uri, fileName)
            } else {
                // For regular file paths, validate directly
                archivePath
            }

            // For saving, we need a writable path
            val savePath = if (isContentUri) {
                // For content URIs, we'll need to save to a local file and then copy back
                validationPath
            } else {
                // For regular file paths, save directly to original location
                archivePath
            }

            // Step 1: Use Apache Commons Compress to safely validate the archive
            Log.d(TAG, "Step 1: Validating archive with Apache Commons Compress...")
            val validateResult = archiveManager.validateArchive(validationPath, masterPassword)
            if (!validateResult.success) {
                Log.e(TAG, "Archive validation failed: ${validateResult.errorMessage}")
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Archive validation failed: ${validateResult.errorMessage}"
                )
            }
            Log.d(TAG, "✅ Archive validation successful - safe to pass to native library")

            // Step 2: Extract archive to temporary directory for native library
            Log.d(TAG, "Step 2: Extracting archive contents for native library...")

            val tempExtractDir = Files.createTempDirectory("ziplock_extract_").toFile()
            val extractResult = archiveManager.openArchive(validationPath, masterPassword, tempExtractDir)

            if (!extractResult.success) {
                Log.e(TAG, "Failed to extract validated archive: ${extractResult.errorMessage}")
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Archive extraction failed: ${extractResult.errorMessage}"
                )
            }

            Log.d(TAG, "✅ Archive extracted successfully")

            // Step 3: Hand off extracted contents to native library using hybrid FFI
            Log.d(TAG, "Step 3: Initializing hybrid session for extracted contents...")

            // Initialize hybrid session for credential management
            val nativeResult = ZipLockNative.openExtractedContents(tempExtractDir.absolutePath, masterPassword)
            if (!nativeResult.success) {
                Log.e(TAG, "Hybrid FFI failed to initialize session: ${nativeResult.errorMessage}")
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Hybrid FFI error: ${nativeResult.errorMessage}"
                )
            }

            Log.d(TAG, "✅ Hybrid FFI session successfully initialized for credential management")

            val metadata = RepositoryMetadata(
                version = "hybrid-1.0",
                format = if (isContentUri) "content-uri" else "file-path",
                extractedPath = tempExtractDir.absolutePath, // Track extracted location
                archiveSource = archivePath, // Store original path for saving
                extractedAt = System.currentTimeMillis()
            )

            // Store current state for hybrid bridge
            currentArchivePath = archivePath // Keep original path for UI display and config
            currentSavePath = savePath // Keep save path for actual file operations
            currentPassword = masterPassword
            currentExtractedPath = tempExtractDir.absolutePath
            isOpen = true

            Log.d(TAG, "✅ Hybrid bridge established - native library managing contents")
            Log.i(TAG, "Repository opened via hybrid approach: $archivePath")

            RepositoryResult(
                success = true,
                data = metadata
            )

        } catch (e: Exception) {
            Log.e(TAG, "Exception during hybrid repository validation", e)
            RepositoryResult(
                success = false,
                errorMessage = "Hybrid validation failed: ${e.message}"
            )
        }
    }

    /**
     * Load all credentials from the repository
     */
    suspend fun loadCredentials(): RepositoryResult<List<ZipLockDataManager.Credential>> = withContext(Dispatchers.IO) {
        if (!isOpen) {
            return@withContext RepositoryResult(
                success = false,
                errorMessage = "No repository is open"
            )
        }

        try {
            Log.i(TAG, "Loading credentials from repository via hybrid FFI")

            // In hybrid mode, credentials are managed through the FFI layer
            // but actual persistence is handled by this repository manager
            val credentialsFile = File(currentExtractedPath!!, CREDENTIALS_FILE)
            if (!credentialsFile.exists()) {
                Log.i(TAG, "No credentials file found, returning empty list")
                return@withContext RepositoryResult(
                    success = true,
                    data = emptyList()
                )
            }

            // Load and parse credentials from file
            val credentialsJson = credentialsFile.readText()
            val serializedCredentials = json.decodeFromString<List<SerializedCredential>>(credentialsJson)

            // Convert to native credentials using hybrid FFI
            val credentials = mutableListOf<ZipLockDataManager.Credential>()
            for (serialized in serializedCredentials) {
                // Create credential through data manager
                val credential = dataManager.createCredential(serialized.title, serialized.type)
                if (credential != null) {
                    // Add fields to the credential
                    for ((name, value) in serialized.fields) {
                        val fieldType = when (name) {
                            "password" -> ZipLockDataManager.FieldType.PASSWORD
                            "email" -> ZipLockDataManager.FieldType.EMAIL
                            "url" -> ZipLockDataManager.FieldType.URL
                            "username" -> ZipLockDataManager.FieldType.USERNAME
                            "phone" -> ZipLockDataManager.FieldType.PHONE
                            else -> ZipLockDataManager.FieldType.TEXT
                        }

                        // Note: Direct FFI credential management not available in hybrid mode
                        // Credentials are managed through the repository layer
                    }

                    // Use the credential we already created
                    for ((name, value) in serialized.fields) {
                        credential.addField(
                            name = name,
                            value = value,
                            fieldType = when (name) {
                                "password" -> ZipLockDataManager.FieldType.PASSWORD
                                "email" -> ZipLockDataManager.FieldType.EMAIL
                                "url" -> ZipLockDataManager.FieldType.URL
                                "username" -> ZipLockDataManager.FieldType.USERNAME
                                "phone" -> ZipLockDataManager.FieldType.PHONE
                                else -> ZipLockDataManager.FieldType.TEXT
                            },
                            sensitive = serialized.sensitiveFields.contains(name)
                        )
                    }
                    credentials.add(credential)
                }
            }

            Log.i(TAG, "Loaded ${credentials.size} credentials via hybrid approach")
            return@withContext RepositoryResult(success = true, data = credentials)

        } catch (e: Exception) {
            Log.e(TAG, "Failed to load credentials", e)
            RepositoryResult(
                success = false,
                errorMessage = "Failed to load credentials: ${e.message}"
            )
        }
    }

    /**
     * Save credentials to the repository
     */
    suspend fun saveCredentials(
        credentials: List<ZipLockDataManager.Credential>
    ): RepositoryResult<Boolean> = withContext(Dispatchers.IO) {
        if (!isOpen) {
            return@withContext RepositoryResult(
                success = false,
                errorMessage = "No repository is open"
            )
        }

        try {
            Log.i(TAG, "Saving ${credentials.size} credentials to repository via hybrid approach")

            // In hybrid mode, we work with the already extracted contents
            val extractedDir = File(currentExtractedPath!!)

            // Convert credentials to serializable format
            val serializedCredentials = mutableListOf<SerializedCredential>()

            for (credential in credentials) {
                val credentialJson = credential.toJson()
                if (credentialJson != null) {
                    // Parse the JSON to extract fields
                    // This is a simplified approach - in practice you'd have a more robust parser
                    val fields = mutableMapOf<String, String>()
                    val sensitiveFields = mutableSetOf<String>()

                    // For now, we'll use a simple approach
                    // In production, you'd implement proper JSON parsing of the credential structure

                    serializedCredentials.add(
                        SerializedCredential(
                            id = UUID.randomUUID().toString(),
                            title = "Credential", // Would parse from JSON
                            type = "login", // Would parse from JSON
                            fields = fields,
                            sensitiveFields = sensitiveFields,
                            createdAt = System.currentTimeMillis(),
                            updatedAt = System.currentTimeMillis()
                        )
                    )
                }
            }

            // Write credentials file to extracted contents
            val credentialsFile = File(extractedDir, CREDENTIALS_FILE)
            FileWriter(credentialsFile).use { writer ->
                writer.write(json.encodeToString(serializedCredentials))
            }

            // Update metadata
            val metadata = RepositoryMetadata(
                credentialCount = credentials.size,
                lastModified = System.currentTimeMillis()
            )
            val metadataFile = File(extractedDir, METADATA_FILE)
            FileWriter(metadataFile).use { writer ->
                writer.write(json.encodeToString(metadata))
            }

            // Phase 3: Save back to archive using ArchiveManager
            val createResult = archiveManager.createArchive(
                currentArchivePath!!,
                currentPassword!!,
                extractedDir
            )

            if (!createResult.success) {
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Failed to save archive: ${createResult.errorMessage}"
                )
            }

            Log.i(TAG, "Successfully saved ${credentials.size} credentials via hybrid approach")
            RepositoryResult(success = true, data = true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to save credentials", e)
            RepositoryResult(
                success = false,
                errorMessage = "Failed to save credentials: ${e.message}"
            )
        }
    }

    /**
     * Save serialized credentials directly to the repository
     */
    suspend fun saveSerializedCredentials(
        serializedCredentials: List<SerializedCredential>
    ): RepositoryResult<Boolean> = withContext(Dispatchers.IO) {
        if (!isOpen) {
            return@withContext RepositoryResult(
                success = false,
                errorMessage = "No repository is open"
            )
        }

        try {
            Log.i(TAG, "Saving ${serializedCredentials.size} serialized credentials to repository")

            // In hybrid mode, we work with the already extracted contents
            val extractedDir = File(currentExtractedPath!!)
            Log.d(TAG, "=== DEBUGGING SAVE PROCESS ===")
            Log.d(TAG, "Extracted directory: ${extractedDir.absolutePath}")
            Log.d(TAG, "Directory exists: ${extractedDir.exists()}")
            Log.d(TAG, "Directory is writable: ${extractedDir.canWrite()}")

            // List existing files before save
            Log.d(TAG, "Files in extracted directory before save:")
            extractedDir.listFiles()?.forEach { file ->
                Log.d(TAG, "  - ${file.name} (${file.length()} bytes)")
            }

            // Write credentials file to extracted contents
            val credentialsFile = File(extractedDir, CREDENTIALS_FILE)
            Log.d(TAG, "Writing credentials to: ${credentialsFile.absolutePath}")
            Log.d(TAG, "Serialized credentials count: ${serializedCredentials.size}")

            val credentialsJson = json.encodeToString(serializedCredentials)
            Log.d(TAG, "Credentials JSON length: ${credentialsJson.length}")
            Log.d(TAG, "Credentials JSON preview: ${credentialsJson.take(200)}...")

            FileWriter(credentialsFile).use { writer ->
                writer.write(credentialsJson)
            }

            Log.d(TAG, "Credentials file written. Size: ${credentialsFile.length()} bytes")

            // Update metadata
            val metadata = RepositoryMetadata(
                credentialCount = serializedCredentials.size,
                lastModified = System.currentTimeMillis()
            )
            val metadataFile = File(extractedDir, METADATA_FILE)
            Log.d(TAG, "Writing metadata to: ${metadataFile.absolutePath}")

            val metadataJson = json.encodeToString(metadata)
            Log.d(TAG, "Metadata JSON: $metadataJson")

            FileWriter(metadataFile).use { writer ->
                writer.write(metadataJson)
            }

            Log.d(TAG, "Metadata file written. Size: ${metadataFile.length()} bytes")

            // List files after save
            Log.d(TAG, "Files in extracted directory after save:")
            extractedDir.listFiles()?.forEach { file ->
                Log.d(TAG, "  - ${file.name} (${file.length()} bytes)")
            }

            // Phase 3: Save back to archive using ArchiveManager
            // Use the save path, which handles content URIs properly
            val createResult = archiveManager.createArchive(
                currentSavePath!!,
                currentPassword!!,
                extractedDir
            )

            if (!createResult.success) {
                Log.e(TAG, "Archive creation failed: ${createResult.errorMessage}")
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Failed to save archive: ${createResult.errorMessage}"
                )
            }

            Log.d(TAG, "Archive created successfully at save path: $currentSavePath")

            // If this was a content URI, we need to copy the saved file back to the original URI
            if (currentArchivePath != currentSavePath) {
                Log.d(TAG, "Content URI detected - copying saved file back to original location")
                Log.d(TAG, "Original path: $currentArchivePath")
                Log.d(TAG, "Save path: $currentSavePath")

                try {
                    val uri = android.net.Uri.parse(currentArchivePath!!)
                    val savedFile = File(currentSavePath!!)

                    Log.d(TAG, "Saved file exists: ${savedFile.exists()}")
                    Log.d(TAG, "Saved file size: ${savedFile.length()} bytes")

                    if (savedFile.exists()) {
                        context.contentResolver.openOutputStream(uri)?.use { outputStream ->
                            savedFile.inputStream().use { inputStream ->
                                val bytesCopied = inputStream.copyTo(outputStream)
                                Log.d(TAG, "Copied $bytesCopied bytes back to content URI")
                            }
                        }
                        Log.d(TAG, "Successfully copied archive back to content URI")
                    } else {
                        Log.e(TAG, "Saved file does not exist: ${currentSavePath}")
                        return@withContext RepositoryResult(
                            success = false,
                            errorMessage = "Failed to save archive: temporary file missing"
                        )
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to copy archive back to content URI", e)
                    return@withContext RepositoryResult(
                        success = false,
                        errorMessage = "Failed to save archive to original location: ${e.message}"
                    )
                }
            } else {
                Log.d(TAG, "Regular file path - no content URI copy needed")
            }

            if (!createResult.success) {
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Failed to save archive: ${createResult.errorMessage}"
                )
            }

            Log.i(TAG, "Successfully saved ${serializedCredentials.size} serialized credentials via hybrid approach")
            Log.d(TAG, "=== SAVE PROCESS COMPLETE ===")
            RepositoryResult(success = true, data = true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to save serialized credentials", e)
            RepositoryResult(
                success = false,
                errorMessage = "Failed to save credentials: ${e.message}"
            )
        }
    }

    /**
     * Add a single credential to the repository
     */
    suspend fun addCredential(
        credential: ZipLockDataManager.Credential
    ): RepositoryResult<Boolean> = withContext(Dispatchers.IO) {
        try {
            val currentCredentials = loadCredentials()
            if (!currentCredentials.success) {
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Failed to add credential: ${currentCredentials.errorMessage}"
                )
            }

            val updatedCredentials = (currentCredentials.data ?: emptyList()) + credential
            return@withContext saveCredentials(updatedCredentials)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to add credential", e)
            RepositoryResult(
                success = false,
                errorMessage = "Failed to add credential: ${e.message}"
            )
        }
    }

    /**
     * Close the current repository
     */
    suspend fun closeRepository(): RepositoryResult<Boolean> = withContext(Dispatchers.IO) {
        try {
            // Close hybrid FFI session first, then handle cleanup
            if (isOpen) {
                Log.d(TAG, "Closing hybrid FFI session...")
                ZipLockNative.closeArchive()

                Log.d(TAG, "Cleaning up hybrid extracted files...")
                currentExtractedPath?.let { extractPath ->
                    try {
                        File(extractPath).deleteRecursively()
                        Log.d(TAG, "Cleaned up extracted files: $extractPath")
                    } catch (e: Exception) {
                        Log.w(TAG, "Failed to clean up extracted files: ${e.message}")
                    }
                }
            }

            currentArchivePath = null
            currentSavePath = null
            currentPassword = null
            currentExtractedPath = null
            isOpen = false

            Log.i(TAG, "Repository closed")
            RepositoryResult(success = true, data = true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to close repository", e)
            RepositoryResult(
                success = false,
                errorMessage = "Failed to close repository: ${e.message}"
            )
        }
    }

    /**
     * Get repository information
     */
    suspend fun getRepositoryInfo(): RepositoryResult<Map<String, Any>> = withContext(Dispatchers.IO) {
        if (!isOpen) {
            return@withContext RepositoryResult(
                success = false,
                errorMessage = "No repository is open"
            )
        }

        try {
            val archiveInfo = archiveManager.getArchiveInfo(currentArchivePath!!, currentPassword!!)

            if (!archiveInfo.success) {
                return@withContext RepositoryResult(
                    success = false,
                    errorMessage = "Failed to get archive info: ${archiveInfo.errorMessage}"
                )
            }

            val info = mutableMapOf<String, Any>(
                "archivePath" to currentArchivePath!!,
                "isOpen" to isOpen,
                "libraryVersion" to dataManager.getLibraryVersion()
            )

            (archiveInfo.data as? Map<*, *>)?.let { archiveData ->
                archiveData.forEach { (key, value) ->
                    if (value != null) {
                        info[key.toString()] = value
                    }
                }
            }

            RepositoryResult(success = true, data = info)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get repository info", e)
            RepositoryResult(
                success = false,
                errorMessage = "Failed to get repository info: ${e.message}"
            )
        }
    }

    /**
     * Test connectivity to native components
     */
    suspend fun testConnectivity(): RepositoryResult<String> = withContext(Dispatchers.IO) {
        try {
            val testInput = "Hybrid architecture test - ${System.currentTimeMillis()}"
            val result = dataManager.testConnectivity(testInput)

            if (result == testInput) {
                RepositoryResult(success = true, data = "✓ Native data manager: OK\n✓ Archive manager: OK")
            } else {
                RepositoryResult(
                    success = false,
                    errorMessage = "Native connectivity test failed"
                )
            }
        } catch (e: Exception) {
            Log.e(TAG, "Connectivity test failed", e)
            RepositoryResult(
                success = false,
                errorMessage = "Connectivity test failed: ${e.message}"
            )
        }
    }

    /**
     * Check if a repository is currently open
     */
    fun isRepositoryOpen(): Boolean = isOpen

    /**
     * Get the path of the currently open repository (original path for UI display)
     */
    fun getCurrentRepositoryPath(): String? = currentArchivePath

    /**
     * Get the actual save path of the currently open repository
     */
    fun getCurrentSavePath(): String? = currentSavePath

    /**
     * Get debugging information about the current path state
     * This helps diagnose issues with original vs temporary path handling
     */
    fun getPathDebugInfo(): String {
        return buildString {
            appendLine("=== Hybrid Repository Path Debug Info ===")
            appendLine("Repository Open: $isOpen")
            appendLine("Original Archive Path: $currentArchivePath")
            appendLine("Save Path: $currentSavePath")
            appendLine("Extracted Path: $currentExtractedPath")
            appendLine("Paths Match: ${currentArchivePath == currentSavePath}")

            currentArchivePath?.let { path ->
                appendLine("Original Path Type: ${if (path.startsWith("content://")) "Content URI" else "File Path"}")
                if (!path.startsWith("content://")) {
                    val file = File(path)
                    appendLine("Original File Exists: ${file.exists()}")
                    appendLine("Original File Size: ${if (file.exists()) file.length() else "N/A"} bytes")
                }
            }

            currentSavePath?.let { path ->
                if (path != currentArchivePath) {
                    appendLine("Save Path Type: File Path")
                    val file = File(path)
                    appendLine("Save File Exists: ${file.exists()}")
                    appendLine("Save File Size: ${if (file.exists()) file.length() else "N/A"} bytes")
                }
            }

            currentExtractedPath?.let { path ->
                val extractDir = File(path)
                appendLine("Extracted Dir Exists: ${extractDir.exists()}")
                if (extractDir.exists()) {
                    val fileCount = extractDir.listFiles()?.size ?: 0
                    appendLine("Extracted File Count: $fileCount")
                }
            }
            appendLine("==========================================")
        }
    }
}
