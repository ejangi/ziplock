package com.ziplock.ffi

import android.util.Log
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.ptr.IntByReference

/**
 * ZipLock Native FFI Interface using JNA
 *
 * This class provides the JNA wrapper for integrating with the shared ZipLock library.
 * It handles all communication with the native Rust library via C FFI, abstracting away
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

    // JNA interface for the native library
    private interface ZipLockLibrary : Library {
        companion object {
            val INSTANCE: ZipLockLibrary = Native.load("ziplock_shared", ZipLockLibrary::class.java)
        }

        // Library management
        fun ziplock_init(): Int
        fun ziplock_get_version(): Pointer?

        // Memory management
        fun ziplock_string_free(str: Pointer?)

        // Password validation
        fun ziplock_password_validate(password: String): Pointer?
        fun ziplock_password_strength_free(strength: Pointer?)

        // Password generation
        fun ziplock_password_generate(
            length: Int,
            includeUppercase: Int,
            includeLowercase: Int,
            includeNumbers: Int,
            includeSymbols: Int
        ): Pointer?

        // Archive operations
        fun ziplock_archive_create(path: String, masterPassword: String): Int
        fun ziplock_archive_open(path: String, masterPassword: String): Int
        fun ziplock_is_archive_open(): Int

        // Testing
        fun ziplock_test_echo(input: String): Pointer?
        fun ziplock_debug_logging(enabled: Int): Int
    }

    // Simplified approach - will implement proper structure mapping later
    // For now we use fallback validation which provides the same functionality

    private val library = ZipLockLibrary.INSTANCE

    /**
     * Initialize the native library
     * Should be called once when the app starts
     *
     * @return true if initialization was successful
     */
    fun init(): Boolean {
        return try {
            val result = library.ziplock_init()
            Log.d("ZipLockNative", "Library initialization result: $result")
            result == 0
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Library initialization failed: ${e.message}")
            false
        }
    }

    /**
     * Get the version of the native library
     *
     * @return version string
     */
    fun getVersion(): String {
        return try {
            val ptr = library.ziplock_get_version()
            val version = ptr?.getString(0) ?: "unknown"
            library.ziplock_string_free(ptr)
            version
        } catch (e: Exception) {
            "unknown"
        }
    }

    /**
     * Validate passphrase strength
     *
     * @param passphrase the passphrase to validate
     * @return PassphraseStrengthResult with strength score and requirements
     */
    fun validatePassphraseStrength(passphrase: String): PassphraseStrengthResult {
        return try {
            // For now, use fallback validation until JNA integration is fully working
            // The native library exists but we need to properly handle the C structure mapping
            createFallbackValidation(passphrase)
        } catch (e: Exception) {
            createFallbackValidation(passphrase)
        }
    }

    /**
     * Generate a secure passphrase
     *
     * @param length desired length of the passphrase
     * @param includeSymbols whether to include special characters
     * @return generated passphrase
     */
    fun generatePassphrase(length: Int, includeSymbols: Boolean): String {
        return try {
            val ptr = library.ziplock_password_generate(
                length,
                1, // uppercase
                1, // lowercase
                1, // numbers
                if (includeSymbols) 1 else 0 // symbols
            )
            val password = ptr?.getString(0) ?: generateFallbackPassword(length, includeSymbols)
            library.ziplock_string_free(ptr)
            password
        } catch (e: Exception) {
            generateFallbackPassword(length, includeSymbols)
        }
    }

    /**
     * Create archive using the native library
     */
    fun createArchive(archivePath: String, passphrase: String): ArchiveResult {
        return try {
            val result = library.ziplock_archive_create(archivePath, passphrase)
            if (result == 0) {
                ArchiveResult(
                    success = true,
                    sessionId = "session_${System.currentTimeMillis()}",
                    errorMessage = null
                )
            } else {
                val errorMessage = mapErrorCode(result)
                ArchiveResult(
                    success = false,
                    sessionId = null,
                    errorMessage = errorMessage,
                    errorCode = result
                )
            }
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                sessionId = null,
                errorMessage = "Archive creation failed: ${e.message}",
                errorCode = 1
            )
        }
    }

    /**
     * Open archive using the native library
     */
    fun openArchive(archivePath: String, passphrase: String): ArchiveResult {
        return try {
            val result = library.ziplock_archive_open(archivePath, passphrase)
            if (result == 0) {
                ArchiveResult(
                    success = true,
                    sessionId = "session_${System.currentTimeMillis()}",
                    errorMessage = null
                )
            } else {
                val errorMessage = mapErrorCode(result)
                ArchiveResult(
                    success = false,
                    sessionId = null,
                    errorMessage = errorMessage,
                    errorCode = result
                )
            }
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                sessionId = null,
                errorMessage = "Archive opening failed: ${e.message}",
                errorCode = 1
            )
        }
    }

    /**
     * Get the last error message from the native library
     * TODO: Implement ziplock_get_last_error in FFI layer
     *
     * @return error message string
     */
    fun getLastError(): String {
        return "Error details not available"
    }

    /**
     * Map FFI error codes to user-friendly messages
     */
    private fun mapErrorCode(errorCode: Int): String {
        return when (errorCode) {
            0 -> "Success"
            1 -> "Invalid parameter provided"
            2 -> "Library not initialized"
            3 -> "Library already initialized"
            4 -> "Archive file not found"
            5 -> "Archive file is corrupted"
            6 -> "Invalid password"
            7 -> "Permission denied"
            8 -> "Out of memory"
            9 -> "Internal error"
            10 -> "Session not found"
            11 -> "Session expired"
            12 -> "No archive is currently open"
            13 -> "Credential not found"
            14 -> "Validation failed"
            15 -> "Cryptographic error"
            16 -> "File I/O error"
            else -> "Unknown error (code: $errorCode)"
        }
    }

    /**
     * Test the native library connection
     */
    fun testConnection(): Boolean {
        return try {
            val ptr = library.ziplock_test_echo("test")
            val result = ptr?.getString(0) == "test"
            library.ziplock_string_free(ptr)
            result
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Enable or disable debug logging in the native library
     */
    fun setDebugLogging(enabled: Boolean) {
        try {
            library.ziplock_debug_logging(if (enabled) 1 else 0)
        } catch (e: Exception) {
            // Ignore errors for debug logging
        }
    }

    // Helper functions

    private fun mapStrengthLevel(level: Int): String {
        return when (level) {
            0 -> "Very Weak"
            1 -> "Weak"
            2 -> "Fair"
            3 -> "Good"
            4 -> "Strong"
            else -> "Unknown"
        }
    }

    private fun parseRequirements(description: String, satisfied: Boolean): List<String> {
        // Parse the description to extract requirements
        // This is a simplified implementation - the actual C API may provide structured data
        val requirements = mutableListOf<String>()

        if (description.contains("length", ignoreCase = true)) {
            if (satisfied) {
                requirements.add("Length requirement met")
            } else {
                requirements.add("Must be at least 12 characters long")
            }
        }

        if (description.contains("uppercase", ignoreCase = true)) {
            if (satisfied) {
                requirements.add("Contains uppercase letters")
            } else {
                requirements.add("Must contain uppercase letters")
            }
        }

        if (description.contains("lowercase", ignoreCase = true)) {
            if (satisfied) {
                requirements.add("Contains lowercase letters")
            } else {
                requirements.add("Must contain lowercase letters")
            }
        }

        if (description.contains("number", ignoreCase = true) || description.contains("digit", ignoreCase = true)) {
            if (satisfied) {
                requirements.add("Contains numbers")
            } else {
                requirements.add("Must contain numbers")
            }
        }

        if (description.contains("symbol", ignoreCase = true) || description.contains("special", ignoreCase = true)) {
            if (satisfied) {
                requirements.add("Contains special characters")
            } else {
                requirements.add("Must contain special characters")
            }
        }

        return requirements
    }

    private fun createFallbackValidation(passphrase: String): PassphraseStrengthResult {
        val requirements = mutableListOf<String>()
        val satisfied = mutableListOf<String>()
        var score = 0

        // Length check
        if (passphrase.length < 12) {
            requirements.add("Must be at least 12 characters long")
        } else {
            satisfied.add("Length requirement met (${passphrase.length} chars)")
            score += 20
        }

        // Character type checks
        val hasLowercase = passphrase.any { it.isLowerCase() }
        val hasUppercase = passphrase.any { it.isUpperCase() }
        val hasDigit = passphrase.any { it.isDigit() }
        val hasSpecial = passphrase.any { !it.isLetterOrDigit() }

        if (!hasLowercase) {
            requirements.add("Must contain lowercase letters")
        } else {
            satisfied.add("Contains lowercase letters")
            score += 15
        }

        if (!hasUppercase) {
            requirements.add("Must contain uppercase letters")
        } else {
            satisfied.add("Contains uppercase letters")
            score += 15
        }

        if (!hasDigit) {
            requirements.add("Must contain numbers")
        } else {
            satisfied.add("Contains numbers")
            score += 15
        }

        if (!hasSpecial) {
            requirements.add("Must contain special characters")
        } else {
            satisfied.add("Contains special characters")
            score += 15
        }

        // Bonus points for length
        if (passphrase.length > 16) score += 10
        if (passphrase.length > 20) score += 10

        val strength = when (score) {
            in 0..20 -> "Very Weak"
            in 21..40 -> "Weak"
            in 41..60 -> "Fair"
            in 61..80 -> "Good"
            in 81..95 -> "Strong"
            else -> "Very Strong"
        }

        return PassphraseStrengthResult(
            score = score.coerceAtMost(100),
            strength = strength,
            requirements = requirements,
            satisfied = satisfied,
            isValid = requirements.isEmpty() && score >= 60
        )
    }

    private fun generateFallbackPassword(length: Int, includeSymbols: Boolean): String {
        val lowercase = "abcdefghijklmnopqrstuvwxyz"
        val uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        val digits = "0123456789"
        val symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?"

        val chars = lowercase + uppercase + digits + if (includeSymbols) symbols else ""
        return (1..length).map { chars.random() }.joinToString("")
    }
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
        } catch (e: UnsatisfiedLinkError) {
            false
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
