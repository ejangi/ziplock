package com.ziplock.ffi

/**
 * ZipLock Native FFI Interface
 *
 * This class provides the JNI wrapper for integrating with the shared ZipLock library.
 * It handles all communication with the native Rust library via FFI, abstracting away
 * the complexity of cryptographic operations and archive management.
 *
 * The shared library handles:
 * - 7z archive creation and opening
 * - AES encryption/decryption
 * - Password validation and key derivation
 * - Cloud storage file handling
 * - File locking and conflict prevention
 *
 * All cryptographic operations are performed in the native library, ensuring
 * consistent security across all platforms.
 */
object ZipLockNative {

    // Load the native library
    init {
        try {
            System.loadLibrary("ziplock_shared")
        } catch (e: UnsatisfiedLinkError) {
            throw RuntimeException("Failed to load ZipLock native library", e)
        }
    }

    /**
     * Initialize the native library
     * Should be called once when the app starts
     *
     * @return true if initialization was successful
     */
    external fun initialize(): Boolean

    /**
     * Get the version of the native library
     *
     * @return version string
     */
    external fun getVersion(): String

    /**
     * Open an existing archive
     *
     * @param archivePath path to the .7z archive file
     * @param passphrase user-provided passphrase for decryption
     * @return ArchiveResult containing session ID or error information
     */
    external fun openArchive(archivePath: String, passphrase: String): ArchiveResult

    /**
     * Create a new archive
     *
     * @param archivePath path where the new .7z archive should be created
     * @param passphrase user-provided passphrase for encryption
     * @return ArchiveResult containing session ID or error information
     */
    external fun createArchive(archivePath: String, passphrase: String): ArchiveResult

    /**
     * Close an archive session
     *
     * @param sessionId the session ID returned from openArchive or createArchive
     * @return true if the archive was closed successfully
     */
    external fun closeArchive(sessionId: String): Boolean

    /**
     * Check if an archive is valid and accessible
     *
     * @param archivePath path to the archive file
     * @return true if the archive is valid
     */
    external fun isValidArchive(archivePath: String): Boolean

    /**
     * Verify a passphrase without fully opening the archive
     *
     * @param archivePath path to the archive file
     * @param passphrase passphrase to verify
     * @return true if the passphrase is correct
     */
    external fun verifyPassphrase(archivePath: String, passphrase: String): Boolean

    /**
     * Get the number of credentials in an archive
     *
     * @param sessionId active session ID
     * @return number of credentials, or -1 on error
     */
    external fun getCredentialCount(sessionId: String): Int

    /**
     * List all credential IDs in the archive
     *
     * @param sessionId active session ID
     * @return array of credential IDs
     */
    external fun listCredentials(sessionId: String): Array<String>

    /**
     * Get a credential by ID
     *
     * @param sessionId active session ID
     * @param credentialId ID of the credential to retrieve
     * @return Credential object or null if not found
     */
    external fun getCredential(sessionId: String, credentialId: String): Credential?

    /**
     * Add a new credential to the archive
     *
     * @param sessionId active session ID
     * @param credential the credential to add
     * @return the ID of the newly created credential, or null on error
     */
    external fun addCredential(sessionId: String, credential: Credential): String?

    /**
     * Update an existing credential
     *
     * @param sessionId active session ID
     * @param credentialId ID of the credential to update
     * @param credential updated credential data
     * @return true if the update was successful
     */
    external fun updateCredential(sessionId: String, credentialId: String, credential: Credential): Boolean

    /**
     * Delete a credential
     *
     * @param sessionId active session ID
     * @param credentialId ID of the credential to delete
     * @return true if the deletion was successful
     */
    external fun deleteCredential(sessionId: String, credentialId: String): Boolean

    /**
     * Save changes to the archive
     *
     * @param sessionId active session ID
     * @return true if the save was successful
     */
    external fun saveArchive(sessionId: String): Boolean

    /**
     * Search credentials by query
     *
     * @param sessionId active session ID
     * @param query search query string
     * @return array of matching credential IDs
     */
    external fun searchCredentials(sessionId: String, query: String): Array<String>

    /**
     * Check if the archive file is stored in cloud storage
     *
     * @param archivePath path to check
     * @return true if the file is in cloud storage
     */
    external fun isCloudStorageFile(archivePath: String): Boolean

    /**
     * Get cloud storage handling information
     *
     * @param archivePath path to the archive
     * @return CloudStorageInfo with details about cloud handling
     */
    external fun getCloudStorageInfo(archivePath: String): CloudStorageInfo

    /**
     * Validate passphrase strength
     *
     * @param passphrase the passphrase to validate
     * @return PassphraseStrengthResult with strength score and requirements
     */
    external fun validatePassphraseStrength(passphrase: String): PassphraseStrengthResult

    /**
     * Generate a secure passphrase
     *
     * @param length desired length of the passphrase
     * @param includeSymbols whether to include special characters
     * @return generated passphrase
     */
    external fun generatePassphrase(length: Int, includeSymbols: Boolean): String

    /**
     * Export archive to different format
     *
     * @param sessionId active session ID
     * @param exportPath path where to export
     * @param format export format (e.g., "csv", "json")
     * @return true if export was successful
     */
    external fun exportArchive(sessionId: String, exportPath: String, format: String): Boolean

    /**
     * Import credentials from file
     *
     * @param sessionId active session ID
     * @param importPath path to the file to import
     * @param format import format (e.g., "csv", "json")
     * @return number of imported credentials, or -1 on error
     */
    external fun importCredentials(sessionId: String, importPath: String, format: String): Int

    /**
     * Get the last error message from the native library
     *
     * @return error message string
     */
    external fun getLastError(): String

    /**
     * Clear the last error message
     */
    external fun clearLastError()

    /**
     * Enable or disable debug logging in the native library
     *
     * @param enabled whether to enable debug logging
     */
    external fun setDebugLogging(enabled: Boolean)

    /**
     * Get library build information
     *
     * @return BuildInfo with version, build date, and feature flags
     */
    external fun getBuildInfo(): BuildInfo

    /**
     * Cleanup native resources
     * Should be called when the app is shutting down
     */
    external fun cleanup()
}

/**
 * Result from archive operations
 */
data class ArchiveResult(
    val success: Boolean,
    val sessionId: String?,
    val errorMessage: String?,
    val errorCode: Int = 0
) {
    fun isSuccess(): Boolean = success
}

/**
 * Credential data structure
 */
data class Credential(
    val id: String = "",
    val title: String,
    val username: String = "",
    val password: String = "",
    val url: String = "",
    val notes: String = "",
    val credentialType: String = "login",
    val tags: List<String> = emptyList(),
    val customFields: Map<String, String> = emptyMap(),
    val createdAt: Long = System.currentTimeMillis(),
    val updatedAt: Long = System.currentTimeMillis(),
    val favorite: Boolean = false
)

/**
 * Cloud storage information
 */
data class CloudStorageInfo(
    val isCloudFile: Boolean,
    val provider: String = "",
    val localCopyPath: String = "",
    val needsSync: Boolean = false,
    val conflictDetected: Boolean = false,
    val lastSyncTime: Long = 0
)

/**
 * Passphrase strength validation result
 */
data class PassphraseStrengthResult(
    val score: Int,
    val strength: String,
    val requirements: List<String>,
    val satisfied: List<String>,
    val isValid: Boolean
)

/**
 * Build information from the native library
 */
data class BuildInfo(
    val version: String,
    val buildDate: String,
    val gitCommit: String,
    val features: List<String>,
    val target: String
)

/**
 * Exception thrown by native library operations
 */
class ZipLockNativeException(
    message: String,
    val errorCode: Int = 0,
    cause: Throwable? = null
) : Exception(message, cause)

/**
 * Helper functions for working with the native library
 */
object ZipLockNativeHelper {

    /**
     * Safe wrapper for archive operations that handles exceptions
     */
    inline fun <T> safeNativeCall(operation: () -> T): Result<T> {
        return try {
            Result.success(operation())
        } catch (e: Exception) {
            Result.failure(ZipLockNativeException(
                message = e.message ?: "Unknown native library error",
                cause = e
            ))
        }
    }

    /**
     * Convert native error codes to user-friendly messages
     */
    fun mapErrorCode(errorCode: Int): String {
        return when (errorCode) {
            1 -> "Invalid archive format"
            2 -> "Incorrect passphrase"
            3 -> "File not found"
            4 -> "Permission denied"
            5 -> "Archive is corrupted"
            6 -> "Network error"
            7 -> "Cloud storage conflict"
            8 -> "Invalid session"
            9 -> "Archive is locked"
            10 -> "Insufficient storage space"
            else -> "Unknown error (code: $errorCode)"
        }
    }

    /**
     * Validate that the native library is properly loaded
     */
    fun validateLibrary(): Boolean {
        return try {
            ZipLockNative.getVersion().isNotEmpty()
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Get a descriptive error message combining native error and mapping
     */
    fun getDetailedError(result: ArchiveResult): String {
        val baseMessage = result.errorMessage ?: ""
        val mappedMessage = if (result.errorCode != 0) {
            mapErrorCode(result.errorCode)
        } else null

        return when {
            baseMessage.isNotEmpty() && mappedMessage != null -> "$baseMessage ($mappedMessage)"
            baseMessage.isNotEmpty() -> baseMessage
            mappedMessage != null -> mappedMessage
            else -> "Unknown error occurred"
        }
    }
}
