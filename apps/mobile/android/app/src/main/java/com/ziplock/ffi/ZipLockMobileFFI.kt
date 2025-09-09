package com.ziplock.ffi

import android.util.Log
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File

/**
 * ZipLock Mobile FFI Interface - Unified Architecture
 *
 * This class provides the JNA wrapper for the new mobile FFI interface.
 * It follows the unified architecture pattern where:
 * - Mobile FFI handles only memory operations (credentials, validation, business logic)
 * - Android app handles all file I/O (archive reading/writing, SAF integration)
 * - Data exchange happens via JSON file maps
 *
 * The shared library handles:
 * - In-memory credential management
 * - Data validation and cryptography
 * - Business logic enforcement
 * - JSON serialization for file map exchange
 *
 * Android app handles:
 * - Archive file I/O using Storage Access Framework (SAF)
 * - 7z extraction/creation using Apache Commons Compress
 * - File system permissions and security
 * - Converting between archive files and JSON file maps
 */
object ZipLockMobileFFI {

    private const val TAG = "ZipLockMobileFFI"

    // JNA interface for the mobile FFI library
    private interface ZipLockMobileLibrary : Library {
        companion object {
            val INSTANCE: ZipLockMobileLibrary = Native.load("ziplock_shared", ZipLockMobileLibrary::class.java)
        }

        // Repository lifecycle functions
        fun ziplock_mobile_repository_create(): Long
        fun ziplock_mobile_repository_destroy(handle: Long)
        fun ziplock_mobile_repository_initialize(handle: Long): Int
        fun ziplock_mobile_repository_is_initialized(handle: Long): Int

        // File map operations
        fun ziplock_mobile_repository_load_from_files(handle: Long, filesJson: String): Int
        fun ziplock_mobile_repository_serialize_to_files(handle: Long): Pointer?

        // Credential operations
        fun ziplock_mobile_add_credential(handle: Long, credentialJson: String): Int
        fun ziplock_mobile_get_credential(handle: Long, credentialId: String): Pointer?
        fun ziplock_mobile_update_credential(handle: Long, credentialJson: String): Int
        fun ziplock_mobile_delete_credential(handle: Long, credentialId: String): Int
        fun ziplock_mobile_list_credentials(handle: Long): Pointer?

        // Repository state
        fun ziplock_mobile_is_modified(handle: Long): Int
        fun ziplock_mobile_mark_saved(handle: Long): Int
        fun ziplock_mobile_get_stats(handle: Long): Pointer?
        fun ziplock_mobile_clear_credentials(handle: Long): Int

        // Memory management
        fun ziplock_mobile_free_string(strPtr: Pointer)

        // Temporary archive operations
        fun ziplock_mobile_create_temp_archive(filesJson: String, password: String, tempPathOut: Array<Pointer?>): Int
        fun ziplock_mobile_extract_temp_archive(archivePath: String, password: String, filesJsonOut: Array<Pointer?>): Int
    }

    // Error codes matching the Rust FFI interface
    object ErrorCodes {
        const val SUCCESS = 0
        const val INVALID_PARAMETER = 1
        const val NOT_INITIALIZED = 2
        const val ALREADY_INITIALIZED = 3
        const val SERIALIZATION_ERROR = 4
        const val VALIDATION_ERROR = 5
        const val OUT_OF_MEMORY = 6
        const val FILE_ERROR = 7
        const val CREDENTIAL_NOT_FOUND = 8
        const val INVALID_PASSWORD = 9
        const val CORRUPTED_ARCHIVE = 10
        const val PERMISSION_DENIED = 11
        const val FILE_NOT_FOUND = 12
        const val INTERNAL_ERROR = 99
    }

    @Serializable
    data class RepositoryStats(
        val credentialCount: Int = 0,
        val isModified: Boolean = false,
        val isInitialized: Boolean = false
    )

    @Serializable
    data class CredentialRecord(
        val id: String,
        val title: String,
        @SerialName("credential_type")
        val credentialType: String,
        val fields: Map<String, CredentialField>,
        val tags: List<String> = emptyList(),
        val notes: String? = null,
        @SerialName("created_at")
        val createdAt: Long,
        @SerialName("updated_at")
        val updatedAt: Long,
        @SerialName("accessed_at")
        val accessedAt: Long,
        val favorite: Boolean = false,
        @SerialName("folder_path")
        val folderPath: String? = null
    )

    @Serializable
    data class CredentialField(
        val value: String,
        @SerialName("field_type")
        val fieldType: FieldType,
        val sensitive: Boolean = false,
        val label: String? = null,
        val metadata: Map<String, String> = emptyMap()
    )

    @Serializable
    enum class FieldType {
        @SerialName("Text")
        Text,
        @SerialName("Password")
        Password,
        @SerialName("Email")
        Email,
        @SerialName("Url")
        Url,
        @SerialName("Username")
        Username,
        @SerialName("Phone")
        Phone,
        @SerialName("CreditCardNumber")
        CreditCardNumber,
        @SerialName("ExpiryDate")
        ExpiryDate,
        @SerialName("Cvv")
        Cvv,
        @SerialName("TotpSecret")
        TotpSecret,
        @SerialName("TextArea")
        TextArea,
        @SerialName("Number")
        Number
    }

    private val library = ZipLockMobileLibrary.INSTANCE
    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        explicitNulls = false
    }

    /**
     * Repository handle wrapper for type safety and automatic cleanup
     */
    class RepositoryHandle private constructor(private val handle: Long) : AutoCloseable {

        companion object {
            /**
             * Create a new repository handle
             * @return RepositoryHandle or null if creation failed
             */
            fun create(): RepositoryHandle? {
                val handle = try {
                    library.ziplock_mobile_repository_create()
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to create repository handle", e)
                    return null
                }

                return if (handle != 0L) {
                    RepositoryHandle(handle)
                } else {
                    Log.e(TAG, "Repository creation returned null handle")
                    null
                }
            }
        }

        /**
         * Get the internal handle value for debugging
         * @return the handle value
         */
        fun getHandle(): Long = handle

        /**
         * Initialize an empty repository
         * @return true if initialization was successful
         */
        fun initialize(): Boolean {
            return try {
                val result = library.ziplock_mobile_repository_initialize(handle)
                if (result != ErrorCodes.SUCCESS) {
                    Log.e(TAG, "Repository initialization failed with error code: $result")
                    false
                } else {
                    Log.d(TAG, "Repository initialized successfully")
                    true
                }
            } catch (e: Exception) {
                Log.e(TAG, "Repository initialization threw exception", e)
                false
            }
        }

        /**
         * Check if repository is initialized
         * @return true if repository is ready for use
         */
        fun isInitialized(): Boolean {
            return try {
                library.ziplock_mobile_repository_is_initialized(handle) == 1
            } catch (e: Exception) {
                Log.e(TAG, "Failed to check repository initialization status", e)
                false
            }
        }

        /**
         * Load repository data from file map JSON
         * The file map should be a JSON object mapping file paths to base64-encoded content
         *
         * @param fileMap Map of file paths to byte arrays from extracted archive
         * @return true if loading was successful
         */
        fun loadFromFiles(fileMap: Map<String, ByteArray>): Boolean {
            return try {
                // Debug: Check repository state before loading
                val isInitBefore = library.ziplock_mobile_repository_is_initialized(handle)
                Log.d(TAG, "DEBUG: Repository initialized before load: $isInitBefore")
                Log.d(TAG, "DEBUG: Repository handle: $handle")
                Log.d(TAG, "DEBUG: File map keys: ${fileMap.keys}")

                // Validate Base64 encoding for each file
                fileMap.forEach { (key, value) ->
                    try {
                        val encoded = android.util.Base64.encodeToString(value, android.util.Base64.NO_WRAP)
                        android.util.Base64.decode(encoded, android.util.Base64.NO_WRAP)
                        Log.d(TAG, "DEBUG: Base64 valid for $key (${value.size} bytes)")

                        // Log content preview for metadata
                        if (key == "metadata.yml") {
                            val preview = String(value).take(200)
                            Log.d(TAG, "DEBUG: Metadata content preview: $preview")
                        }
                    } catch (e: Exception) {
                        Log.e(TAG, "DEBUG: Base64 invalid for $key: ${e.message}")
                    }
                }

                // Convert byte arrays to base64 strings for JSON serialization
                val base64Map = fileMap.mapValues { (_, bytes) ->
                    android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
                }

                val filesJson = json.encodeToString(base64Map)
                Log.d(TAG, "DEBUG: Files JSON length: ${filesJson.length}")
                Log.d(TAG, "DEBUG: JSON first 200 chars: ${filesJson.take(200)}")

                val result = library.ziplock_mobile_repository_load_from_files(handle, filesJson)

                if (result != ErrorCodes.SUCCESS) {
                    val errorMessage = getErrorMessage(result)
                    Log.e(TAG, "Failed to load from files with error code: $result - $errorMessage")
                    Log.e(TAG, "DEBUG: Repository handle: $handle")
                    Log.e(TAG, "DEBUG: Is initialized after failure: ${library.ziplock_mobile_repository_is_initialized(handle)}")
                    Log.e(TAG, "DEBUG: File map size: ${fileMap.size}")
                    Log.e(TAG, "DEBUG: JSON size: ${filesJson.length}")

                    // Log file map structure for debugging
                    fileMap.forEach { (key, value) ->
                        Log.e(TAG, "DEBUG: File in map: $key (${value.size} bytes)")
                        if (key == "metadata.yml") {
                            val content = String(value)
                            Log.e(TAG, "DEBUG: metadata.yml content: ${content.take(300)}")
                        }
                    }

                    // Special handling for common errors
                    if (result == ErrorCodes.ALREADY_INITIALIZED) {
                        Log.e(TAG, "CRITICAL: Repository was already initialized before loadFromFiles() - this is a programming error!")
                        Log.e(TAG, "FIX: Do NOT call initialize() before loadFromFiles() - loadFromFiles initializes internally")
                    } else if (result == ErrorCodes.SERIALIZATION_ERROR) {
                        Log.e(TAG, "Archive contains invalid or corrupted repository data")
                    } else if (result == ErrorCodes.INTERNAL_ERROR) {
                        Log.e(TAG, "Repository format validation failed - archive may not be a valid ZipLock repository")
                        Log.e(TAG, "INTERNAL_ERROR details: This could be caused by:")
                        Log.e(TAG, "  1. Invalid metadata.yml format")
                        Log.e(TAG, "  2. Missing required fields in metadata")
                        Log.e(TAG, "  3. Credential count mismatch")
                        Log.e(TAG, "  4. Corrupted credential files")
                        Log.e(TAG, "  5. UTF-8 encoding issues")
                    }

                    false
                } else {
                    Log.d(TAG, "Repository loaded from files successfully")
                    Log.d(TAG, "DEBUG: Repository initialized after success: ${library.ziplock_mobile_repository_is_initialized(handle)}")
                    true
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception while loading from files", e)
                Log.e(TAG, "DEBUG: Exception occurred with handle: $handle")
                false
            }
        }

        /**
         * Serialize repository to file map for archive creation
         * @return Map of file paths to byte arrays, or null on error
         */
        fun serializeToFiles(): Map<String, ByteArray>? {
            return try {
                val resultPtr = library.ziplock_mobile_repository_serialize_to_files(handle)
                if (resultPtr == null) {
                    Log.e(TAG, "Repository serialization returned null")
                    return null
                }

                val filesJson = resultPtr.getString(0)
                library.ziplock_mobile_free_string(resultPtr)

                // Parse JSON and convert base64 strings back to byte arrays
                val base64Map = json.decodeFromString<Map<String, String>>(filesJson)
                val fileMap = base64Map.mapValues { (_, base64) ->
                    try {
                        android.util.Base64.decode(base64, android.util.Base64.NO_WRAP)
                    } catch (e: IllegalArgumentException) {
                        // If base64 decode fails, treat as UTF-8 text
                        base64.toByteArray(Charsets.UTF_8)
                    }
                }

                Log.d(TAG, "Repository serialized to ${fileMap.size} files")
                fileMap
            } catch (e: Exception) {
                Log.e(TAG, "Exception while serializing to files", e)
                null
            }
        }

        /**
         * Add a new credential to the repository
         * @param credential CredentialRecord to add
         * @return true if credential was added successfully
         */
        fun addCredential(credential: CredentialRecord): Boolean {
            return try {
                val credentialJson = json.encodeToString(credential)
                val result = library.ziplock_mobile_add_credential(handle, credentialJson)

                if (result != ErrorCodes.SUCCESS) {
                    Log.e(TAG, "Failed to add credential with error code: $result")
                    false
                } else {
                    Log.d(TAG, "Credential added successfully: ${credential.title}")
                    true
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception while adding credential", e)
                false
            }
        }

        /**
         * Retrieve a credential by ID
         * @param credentialId ID of the credential to retrieve
         * @return CredentialRecord or null if not found or error occurred
         */
        fun getCredential(credentialId: String): CredentialRecord? {
            return try {
                val resultPtr = library.ziplock_mobile_get_credential(handle, credentialId)
                if (resultPtr == null) {
                    Log.w(TAG, "Credential not found: $credentialId")
                    return null
                }

                val credentialJson = resultPtr.getString(0)
                library.ziplock_mobile_free_string(resultPtr)

                val credential = json.decodeFromString<CredentialRecord>(credentialJson)
                Log.d(TAG, "Retrieved credential: ${credential.title}")
                credential
            } catch (e: Exception) {
                Log.e(TAG, "Exception while getting credential: $credentialId", e)
                null
            }
        }

        /**
         * Update an existing credential
         * @param credential CredentialRecord with updated data
         * @return true if credential was updated successfully
         */
        fun updateCredential(credential: CredentialRecord): Boolean {
            return try {
                Log.d(TAG, "DEBUG: About to update credential - ID: '${credential.id}', Title: '${credential.title}'")
                if (credential.id.isEmpty()) {
                    Log.w(TAG, "DEBUG: Credential has EMPTY ID - FFI will need to handle ID generation or lookup by title")
                }
                Log.d(TAG, "DEBUG: Credential fields: ${credential.fields.keys}")
                credential.fields.forEach { (key, field) ->
                    Log.d(TAG, "DEBUG: Field '$key' = '${field.value}' (${field.fieldType})")
                }

                val credentialJson = json.encodeToString(credential)
                Log.d(TAG, "DEBUG: Credential JSON length: ${credentialJson.length}")
                Log.d(TAG, "DEBUG: Credential JSON preview: ${credentialJson.take(200)}...")

                val result = library.ziplock_mobile_update_credential(handle, credentialJson)

                if (result != ErrorCodes.SUCCESS) {
                    Log.e(TAG, "Failed to update credential with error code: $result")
                    false
                } else {
                    Log.d(TAG, "Credential updated successfully: ${credential.title}")

                    // Immediately try to get the credential back to verify it was saved correctly
                    try {
                        val verifyCredential = getCredential(credential.id)
                        if (verifyCredential != null) {
                            Log.d(TAG, "DEBUG: Verification - Retrieved credential has fields: ${verifyCredential.fields.keys}")
                            verifyCredential.fields.forEach { (key, field) ->
                                Log.d(TAG, "DEBUG: Verification - Field '$key' = '${field.value}'")
                            }
                        } else {
                            Log.w(TAG, "DEBUG: Verification - Could not retrieve credential after update")
                        }
                    } catch (e: Exception) {
                        Log.w(TAG, "DEBUG: Verification failed: ${e.message}")
                    }

                    true
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception while updating credential", e)
                false
            }
        }

        /**
         * Delete a credential by ID
         * @param credentialId ID of the credential to delete
         * @return true if credential was deleted successfully
         */
        fun deleteCredential(credentialId: String): Boolean {
            return try {
                val result = library.ziplock_mobile_delete_credential(handle, credentialId)

                if (result != ErrorCodes.SUCCESS) {
                    Log.e(TAG, "Failed to delete credential with error code: $result")
                    false
                } else {
                    Log.d(TAG, "Credential deleted successfully: $credentialId")
                    true
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception while deleting credential: $credentialId", e)
                false
            }
        }

        /**
         * List all credentials in the repository
         *
         * @return List of CredentialRecord or empty list on error
         */
        fun listCredentials(): List<CredentialRecord> {
            return try {
                val resultPtr = library.ziplock_mobile_list_credentials(handle)
                if (resultPtr == null) {
                    Log.w(TAG, "List credentials returned null")
                    return emptyList()
                }

                val credentialsJson = resultPtr.getString(0)
                library.ziplock_mobile_free_string(resultPtr)

                Log.d(TAG, "DEBUG: Raw JSON from FFI: $credentialsJson")
                Log.d(TAG, "DEBUG: JSON length: ${credentialsJson.length}")
                Log.d(TAG, "DEBUG: JSON first 100 chars: ${credentialsJson.take(100)}")

                // Try to parse as full credentials first
                try {
                    val credentials = json.decodeFromString<List<CredentialRecord>>(credentialsJson)
                    Log.d(TAG, "Successfully parsed as full credentials: ${credentials.size} credentials")
                    credentials.forEachIndexed { index, cred ->
                        Log.d(TAG, "DEBUG: Credential $index - ID: '${cred.id}', Title: '${cred.title}', Fields: ${cred.fields.keys}")
                    }
                    return credentials
                } catch (fullParseException: Exception) {
                    Log.w(TAG, "Failed to parse as full credentials: ${fullParseException.message}")

                    // Try to parse as tuples and convert
                    try {
                        val tuples = json.decodeFromString<List<List<String>>>(credentialsJson)
                        Log.d(TAG, "Successfully parsed as tuples: ${tuples.size} tuples")

                        val credentials = tuples.mapNotNull { tuple ->
                            if (tuple.size >= 2) {
                                val credentialId = tuple[0]
                                val title = tuple[1]

                                Log.w(TAG, "Fallback: Creating credential from tuple - ID: '$credentialId', Title: '$title'")
                                Log.w(TAG, "DEBUG: Full tuple content: $tuple")

                                // Try to get full credential data - even for empty IDs since verification shows it works
                                val fullCredential = try {
                                    Log.d(TAG, "DEBUG: Attempting to get full credential for ID: '$credentialId'")
                                    val cred = getCredential(credentialId)
                                    Log.d(TAG, "DEBUG: Got credential with fields: ${cred?.fields?.keys}")
                                    cred
                                } catch (e: Exception) {
                                    Log.w(TAG, "Failed to get full credential for ID '$credentialId': ${e.message}")
                                    null
                                }

                                // Use full credential if available, otherwise create basic credential
                                val result = fullCredential ?: CredentialRecord(
                                    id = credentialId,
                                    title = title,
                                    credentialType = "login", // Default type
                                    fields = mapOf(
                                        "username" to CredentialField(
                                            value = "",
                                            fieldType = FieldType.Text,
                                            sensitive = false
                                        ),
                                        "password" to CredentialField(
                                            value = "",
                                            fieldType = FieldType.Password,
                                            sensitive = true
                                        )
                                    ),
                                    tags = emptyList(),
                                    notes = null,
                                    createdAt = System.currentTimeMillis(),
                                    updatedAt = System.currentTimeMillis(),
                                    accessedAt = System.currentTimeMillis(),
                                    favorite = false,
                                    folderPath = null
                                )

                                Log.d(TAG, "DEBUG: Final credential - ID: '${result.id}', Fields: ${result.fields.keys}")
                                result
                            } else {
                                null
                            }
                        }

                        Log.d(TAG, "Converted ${credentials.size} tuples to credentials")
                        return credentials
                    } catch (tupleParseException: Exception) {
                        Log.e(TAG, "Failed to parse as tuples: ${tupleParseException.message}")
                        throw fullParseException
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception while listing credentials", e)
                emptyList()
            }
        }

        /**
         * Check if repository has unsaved changes
         * @return true if there are unsaved changes
         */
        fun isModified(): Boolean {
            return try {
                library.ziplock_mobile_is_modified(handle) == 1
            } catch (e: Exception) {
                Log.e(TAG, "Failed to check repository modification status", e)
                false
            }
        }

        /**
         * Mark repository as saved (clears modified flag)
         * Should be called after successfully saving to archive
         * @return true if operation was successful
         */
        fun markSaved(): Boolean {
            return try {
                val result = library.ziplock_mobile_mark_saved(handle)
                result == ErrorCodes.SUCCESS
            } catch (e: Exception) {
                Log.e(TAG, "Failed to mark repository as saved", e)
                false
            }
        }

        /**
         * Get repository statistics
         * @return RepositoryStats or null on error
         */
        fun getStats(): RepositoryStats? {
            return try {
                val resultPtr = library.ziplock_mobile_get_stats(handle)
                if (resultPtr == null) {
                    Log.w(TAG, "Get stats returned null")
                    return null
                }

                val statsJson = resultPtr.getString(0)
                library.ziplock_mobile_free_string(resultPtr)

                val stats = json.decodeFromString<RepositoryStats>(statsJson)
                Log.d(TAG, "Repository stats: ${stats.credentialCount} credentials, modified: ${stats.isModified}")
                stats
            } catch (e: Exception) {
                Log.e(TAG, "Exception while getting repository stats", e)
                null
            }
        }

        /**
         * Clear all credentials from the repository
         * Useful for testing or creating a new repository
         * @return true if operation was successful
         */
        fun clearCredentials(): Boolean {
            return try {
                val result = library.ziplock_mobile_clear_credentials(handle)
                if (result != ErrorCodes.SUCCESS) {
                    Log.e(TAG, "Failed to clear credentials with error code: $result")
                    false
                } else {
                    Log.d(TAG, "All credentials cleared successfully")
                    true
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception while clearing credentials", e)
                false
            }
        }

        override fun close() {
            try {
                library.ziplock_mobile_repository_destroy(handle)
                Log.d(TAG, "Repository handle destroyed")
            } catch (e: Exception) {
                Log.e(TAG, "Exception while destroying repository handle", e)
            }
        }
    }

    /**
     * Test the FFI connection and library loading
     * @return true if the library is loaded and functional
     */
    fun testConnection(): Boolean {
        return try {
            RepositoryHandle.create()?.use { repo ->
                repo.initialize()
            } ?: false
        } catch (e: Exception) {
            Log.e(TAG, "FFI connection test failed", e)
            false
        }
    }

    /**
     * Get error message for error code
     * @param errorCode Error code from FFI operations
     * @return Human-readable error message
     */
    fun getErrorMessage(errorCode: Int): String {
        return when (errorCode) {
            ErrorCodes.SUCCESS -> "Success"
            ErrorCodes.INVALID_PARAMETER -> "Invalid parameter"
            ErrorCodes.NOT_INITIALIZED -> "Repository not initialized"
            ErrorCodes.ALREADY_INITIALIZED -> "Repository already initialized - this usually indicates a programming error where initialize() was called before loadFromFiles()"
            ErrorCodes.SERIALIZATION_ERROR -> "Serialization error - failed to parse repository data"
            ErrorCodes.VALIDATION_ERROR -> "Validation error - repository data does not match expected format"
            ErrorCodes.OUT_OF_MEMORY -> "Out of memory"
            ErrorCodes.FILE_ERROR -> "File operation error"
            ErrorCodes.CREDENTIAL_NOT_FOUND -> "Credential not found"
            ErrorCodes.INVALID_PASSWORD -> "Invalid password"
            ErrorCodes.CORRUPTED_ARCHIVE -> "Archive corrupted or invalid"
            ErrorCodes.PERMISSION_DENIED -> "Permission denied"
            ErrorCodes.FILE_NOT_FOUND -> "File not found"
            ErrorCodes.INTERNAL_ERROR -> "Internal error - repository format validation failed or the archive may not contain a valid ZipLock repository"
            else -> "Unknown error ($errorCode) - please check that the archive contains a valid ZipLock repository"
        }
    }

    /**
     * Create temporary encrypted archive using shared library
     *
     * This function uses the shared library's sevenz-rust2 implementation to create
     * a properly encrypted 7z archive in temporary storage. The Android app can then
     * move this archive to the final location using SAF operations.
     *
     * @param filesJson JSON string containing file map (path -> base64 content)
     * @param password Password for archive encryption
     * @return Path to temporary encrypted archive file, or null on error
     */
    fun createTempArchive(filesJson: String, password: String): String? {
        return try {
            Log.d(TAG, "Creating temporary encrypted archive via shared library")

            if (password.isEmpty()) {
                Log.e(TAG, "Password required for encrypted archive creation")
                return null
            }

            // Use JNA array approach for output parameter
            val tempPathOut = arrayOfNulls<Pointer>(1)

            val result = ZipLockMobileLibrary.INSTANCE.ziplock_mobile_create_temp_archive(
                filesJson, password, tempPathOut
            )

            if (result == ErrorCodes.SUCCESS && tempPathOut[0] != null) {
                val tempPath = tempPathOut[0]!!.getString(0)
                ZipLockMobileLibrary.INSTANCE.ziplock_mobile_free_string(tempPathOut[0]!!)

                Log.d(TAG, "✅ Temporary encrypted archive created at: $tempPath")
                return tempPath
            }

            Log.e(TAG, "❌ Failed to create temporary archive: ${getErrorMessage(result)}")
            return null
        } catch (e: Exception) {
            Log.e(TAG, "Exception creating temporary archive", e)
            null
        }
    }

    /**
     * Extract temporary encrypted archive to file map using shared library
     *
     * This function uses the shared library's sevenz-rust2 implementation to extract
     * a properly encrypted 7z archive from temporary storage, returning the file
     * contents as a JSON map.
     *
     * @param archivePath Path to the encrypted archive file
     * @param password Password for archive decryption
     * @return JSON string containing file map (path -> base64 content), or null on error
     */
    fun extractTempArchive(archivePath: String, password: String): String? {
        return try {
            Log.d(TAG, "Extracting temporary encrypted archive via shared library")
            Log.d(TAG, "Archive path: $archivePath")
            Log.d(TAG, "DEBUG: Password length: ${password.length}")
            Log.d(TAG, "DEBUG: Archive file exists: ${File(archivePath).exists()}")
            Log.d(TAG, "DEBUG: Archive file size: ${File(archivePath).length()} bytes")

            if (password.isEmpty()) {
                Log.e(TAG, "Password required for encrypted archive extraction")
                return null
            }

            // Use JNA array approach for output parameter
            val filesJsonOut = arrayOfNulls<Pointer>(1)

            val result = ZipLockMobileLibrary.INSTANCE.ziplock_mobile_extract_temp_archive(
                archivePath, password, filesJsonOut
            )

            if (result == ErrorCodes.SUCCESS && filesJsonOut[0] != null) {
                val filesJson = filesJsonOut[0]!!.getString(0)
                ZipLockMobileLibrary.INSTANCE.ziplock_mobile_free_string(filesJsonOut[0]!!)

                Log.d(TAG, "✅ Archive extracted successfully (${filesJson.length} chars)")
                Log.d(TAG, "DEBUG: JSON preview: ${filesJson.take(200)}")

                // Validate JSON structure
                try {
                    val fileMap = json.decodeFromString<Map<String, String>>(filesJson)
                    Log.d(TAG, "DEBUG: File map contains ${fileMap.size} files")
                    Log.d(TAG, "DEBUG: File map keys: ${fileMap.keys}")
                } catch (e: Exception) {
                    Log.e(TAG, "DEBUG: JSON validation failed: ${e.message}")
                }

                return filesJson
            }

            Log.e(TAG, "❌ Failed to extract archive: ${getErrorMessage(result)}")
            Log.e(TAG, "DEBUG: FFI result code: $result")
            Log.e(TAG, "DEBUG: filesJsonOut[0] is null: ${filesJsonOut[0] == null}")
            return null
        } catch (e: Exception) {
            Log.e(TAG, "Exception extracting temporary archive", e)
            null
        }
    }

}
