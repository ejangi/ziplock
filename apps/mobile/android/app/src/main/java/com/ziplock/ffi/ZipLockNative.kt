package com.ziplock.ffi

import android.util.Log
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference

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

    // Session state management
    private var currentSessionId: String? = null
    private var isArchiveCurrentlyOpen: Boolean = false

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
        fun ziplock_archive_close(): Int
        fun ziplock_is_archive_open(): Int

        // Credential operations
        fun ziplock_credential_list(credentials: PointerByReference, count: IntByReference): Int
        fun ziplock_credential_list_free(credentials: Pointer, count: Int)

        // Testing
        fun ziplock_debug_logging(enabled: Int): Int
    }

    // JNA Structure classes for FFI integration
    @Structure.FieldOrder("id", "title", "credential_type", "notes", "created_at", "updated_at", "field_count", "fields", "tag_count", "tags")
    class CCredentialRecord : Structure() {
        @JvmField var id: Pointer? = null
        @JvmField var title: Pointer? = null
        @JvmField var credential_type: Pointer? = null
        @JvmField var notes: Pointer? = null
        @JvmField var created_at: Long = 0
        @JvmField var updated_at: Long = 0
        @JvmField var field_count: Int = 0
        @JvmField var fields: Pointer? = null
        @JvmField var tag_count: Int = 0
        @JvmField var tags: Pointer? = null
    }

    @Structure.FieldOrder("name", "value", "field_type", "label", "sensitive", "required")
    class CCredentialField : Structure() {
        @JvmField var name: Pointer? = null
        @JvmField var value: Pointer? = null
        @JvmField var field_type: Pointer? = null
        @JvmField var label: Pointer? = null
        @JvmField var sensitive: Int = 0
        @JvmField var required: Int = 0
    }

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
                // Successfully created - update session state
                currentSessionId = "session_${System.currentTimeMillis()}"
                isArchiveCurrentlyOpen = true

                ArchiveResult(
                    success = true,
                    sessionId = currentSessionId,
                    errorMessage = null
                )
            } else {
                // Failed to create - clear session state
                currentSessionId = null
                isArchiveCurrentlyOpen = false

                val errorMessage = mapErrorCode(result)
                ArchiveResult(
                    success = false,
                    sessionId = null,
                    errorMessage = errorMessage,
                    errorCode = result
                )
            }
        } catch (e: Exception) {
            // Exception occurred - clear session state
            currentSessionId = null
            isArchiveCurrentlyOpen = false

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
                // Successfully opened - update session state
                currentSessionId = "session_${System.currentTimeMillis()}"
                isArchiveCurrentlyOpen = true

                ArchiveResult(
                    success = true,
                    sessionId = currentSessionId,
                    errorMessage = null
                )
            } else {
                // Failed to open - clear session state
                currentSessionId = null
                isArchiveCurrentlyOpen = false

                val errorMessage = mapErrorCode(result)
                ArchiveResult(
                    success = false,
                    sessionId = null,
                    errorMessage = errorMessage,
                    errorCode = result
                )
            }
        } catch (e: Exception) {
            // Exception occurred - clear session state
            currentSessionId = null
            isArchiveCurrentlyOpen = false

            ArchiveResult(
                success = false,
                sessionId = null,
                errorMessage = "Archive opening failed: ${e.message}",
                errorCode = 1
            )
        }
    }

    /**
     * List all credentials in the currently open archive
     */
    fun listCredentials(): CredentialListResult {
        return try {
            // Check if archive is open (this will sync state)
            if (!isArchiveOpen()) {
                val healthCheck = performHealthCheck()
                Log.w("ZipLockNative", "Archive not open. Health check:\n$healthCheck")
                return CredentialListResult(
                    success = false,
                    credentials = emptyList(),
                    errorMessage = "No archive is currently open. FFI Status:\n$healthCheck"
                )
            }

            // Call native credential list function
            Log.d("ZipLockNative", "Calling ziplock_credential_list...")
            val credentialsPtr = PointerByReference()
            val count = IntByReference()

            val result = library.ziplock_credential_list(credentialsPtr, count)
            Log.d("ZipLockNative", "ziplock_credential_list returned: $result")

            if (result != 0) {
                val errorMessage = mapErrorCode(result)
                Log.e("ZipLockNative", "Failed to list credentials: $errorMessage (code: $result)")
                return CredentialListResult(
                    success = false,
                    credentials = emptyList(),
                    errorMessage = errorMessage
                )
            }

            val credentialCount = count.value
            Log.d("ZipLockNative", "Found $credentialCount credentials")
            val credentials = mutableListOf<Credential>()

            if (credentialCount > 0 && credentialsPtr.value != null) {
                Log.d("ZipLockNative", "Parsing $credentialCount credentials...")
                // Parse the array of C credential records
                val credentialArray = credentialsPtr.value

                for (i in 0 until credentialCount) {
                    try {
                        Log.d("ZipLockNative", "Parsing credential $i...")
                        // Calculate offset for the i-th structure
                        val structSize = Native.getNativeSize(CCredentialRecord::class.java)
                        val recordPtr = credentialArray.share((i * structSize).toLong())

                        val record = Structure.newInstance(CCredentialRecord::class.java, recordPtr) as CCredentialRecord
                        record.read()

                        // Convert C structure to Kotlin data class
                        val credential = convertCCredentialToKotlin(record)
                        credentials.add(credential)
                        Log.d("ZipLockNative", "Successfully parsed credential: ${credential.title}")
                    } catch (e: Exception) {
                        Log.e("ZipLockNative", "Failed to parse credential $i: ${e.message}", e)
                    }
                }

                Log.d("ZipLockNative", "Freeing native memory for $credentialCount credentials")
                // Free the native memory
                library.ziplock_credential_list_free(credentialsPtr.value, credentialCount)
            } else {
                Log.d("ZipLockNative", "No credentials found or null pointer")
            }

            Log.d("ZipLockNative", "Returning ${credentials.size} parsed credentials")
            CredentialListResult(
                success = true,
                credentials = credentials,
                errorMessage = null
            )
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Exception in listCredentials: ${e.message}", e)
            CredentialListResult(
                success = false,
                credentials = emptyList(),
                errorMessage = "Failed to list credentials: ${e.message}"
            )
        }
    }

    /**
     * Create mock credential list for development/testing
     * This can be used for testing the UI with sample data
     */
    private fun createMockCredentialList(): CredentialListResult {
        val mockCredentials = listOf(
            Credential(
                id = "cred_1",
                title = "Google Account",
                credentialType = "login",
                url = "https://accounts.google.com",
                username = "user@gmail.com"
            ),
            Credential(
                id = "cred_2",
                title = "Bank of America",
                credentialType = "bank_account",
                url = "https://bankofamerica.com"
            ),
            Credential(
                id = "cred_3",
                title = "Visa Credit Card",
                credentialType = "credit_card"
            ),
            Credential(
                id = "cred_4",
                title = "WiFi Password",
                credentialType = "secure_note",
                notes = "Home network credentials"
            ),
            Credential(
                id = "cred_5",
                title = "SSH Server Key",
                credentialType = "ssh_key",
                url = "192.168.1.100"
            )
        )

        return CredentialListResult(
            success = true,
            credentials = mockCredentials,
            errorMessage = null
        )
    }

    /**
     * Close the current archive and clear session state
     */
    fun closeArchive(): Boolean {
        return try {
            Log.d("ZipLockNative", "Closing archive...")

            // Call native library close function
            val result = library.ziplock_archive_close()

            if (result == 0) {
                Log.d("ZipLockNative", "Archive closed successfully")
                // Clear our session state only on success
                currentSessionId = null
                isArchiveCurrentlyOpen = false
                true
            } else {
                Log.e("ZipLockNative", "Failed to close archive: ${mapErrorCode(result)}")
                false
            }
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Exception closing archive: ${e.message}", e)
            // Clear state even on exception to prevent stuck state
            currentSessionId = null
            isArchiveCurrentlyOpen = false
            false
        }
    }

    /**
     * Check if an archive is currently open
     */
    fun isArchiveOpen(): Boolean {
        return try {
            // Check native library state first
            val nativeIsOpen = library.ziplock_is_archive_open() == 1

            // Sync our local state with native state
            if (!nativeIsOpen && isArchiveCurrentlyOpen) {
                Log.w("ZipLockNative", "Native library says archive is closed, syncing local state")
                isArchiveCurrentlyOpen = false
                currentSessionId = null
            } else if (nativeIsOpen && !isArchiveCurrentlyOpen) {
                Log.w("ZipLockNative", "Native library says archive is open, syncing local state")
                isArchiveCurrentlyOpen = true
                if (currentSessionId == null) {
                    currentSessionId = "session_${System.currentTimeMillis()}"
                }
            }

            Log.d("ZipLockNative", "Archive open status - Native: $nativeIsOpen, Local: $isArchiveCurrentlyOpen")
            nativeIsOpen
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Error checking archive status: ${e.message}", e)
            false
        }
    }

    /**
     * Get current session ID
     */
    fun getCurrentSessionId(): String? {
        return currentSessionId
    }

    /**
     * Convert C credential record to Kotlin data class
     */
    private fun convertCCredentialToKotlin(record: CCredentialRecord): Credential {
        Log.d("ZipLockNative", "Converting C credential to Kotlin...")

        val id = try {
            record.id?.getString(0) ?: ""
        } catch (e: Exception) {
            Log.w("ZipLockNative", "Failed to read id: ${e.message}")
            ""
        }

        val title = try {
            record.title?.getString(0) ?: ""
        } catch (e: Exception) {
            Log.w("ZipLockNative", "Failed to read title: ${e.message}")
            "Unknown"
        }

        val credentialType = try {
            record.credential_type?.getString(0) ?: ""
        } catch (e: Exception) {
            Log.w("ZipLockNative", "Failed to read credential_type: ${e.message}")
            "unknown"
        }

        val notes = try {
            record.notes?.getString(0) ?: ""
        } catch (e: Exception) {
            Log.w("ZipLockNative", "Failed to read notes: ${e.message}")
            ""
        }

        Log.d("ZipLockNative", "Basic fields - id: $id, title: $title, type: $credentialType")

        // Parse tags if present
        val tags = mutableListOf<String>()
        if (record.tag_count > 0 && record.tags != null) {
            try {
                Log.d("ZipLockNative", "Parsing ${record.tag_count} tags...")
                val tagsArray = record.tags!!.getPointerArray(0, record.tag_count)
                for (tagPtr in tagsArray) {
                    if (tagPtr != null) {
                        val tag = tagPtr.getString(0)
                        if (tag.isNotBlank()) {
                            tags.add(tag)
                            Log.d("ZipLockNative", "Added tag: $tag")
                        }
                    }
                }
            } catch (e: Exception) {
                Log.e("ZipLockNative", "Failed to parse tags: ${e.message}", e)
            }
        }

        // Parse fields to extract common fields like username, password, url
        var username = ""
        var password = ""
        var url = ""

        if (record.field_count > 0 && record.fields != null) {
            try {
                Log.d("ZipLockNative", "Parsing ${record.field_count} fields...")
                for (i in 0 until record.field_count) {
                    val fieldStructSize = Native.getNativeSize(CCredentialField::class.java)
                    val fieldPtr = record.fields!!.share((i * fieldStructSize).toLong())

                    val field = Structure.newInstance(CCredentialField::class.java, fieldPtr) as CCredentialField
                    field.read()

                    val fieldName = field.name?.getString(0) ?: ""
                    val fieldValue = field.value?.getString(0) ?: ""

                    Log.d("ZipLockNative", "Field $i: $fieldName = $fieldValue")

                    when (fieldName.lowercase()) {
                        "username", "user", "login" -> username = fieldValue
                        "password", "pass" -> password = fieldValue
                        "url", "website", "site" -> url = fieldValue
                    }
                }
            } catch (e: Exception) {
                Log.e("ZipLockNative", "Failed to parse fields: ${e.message}", e)
            }
        }

        return Credential(
            id = id,
            title = title,
            credentialType = credentialType,
            username = username,
            password = password,
            url = url,
            notes = notes,
            tags = tags,
            createdAt = record.created_at,
            updatedAt = record.updated_at
        )
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
            Log.d("ZipLockNative", "Testing library connection with version check...")
            // Use getVersion as a test since ziplock_test_echo doesn't exist in the library
            val version = getVersion()
            val result = version.isNotBlank() && version != "unknown"
            Log.d("ZipLockNative", "Version test result: $result (version: '$version')")
            result
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Version test failed with exception: ${e.message}", e)
            false
        }
    }

    /**
     * Enable or disable debug logging in the native library
     */
    fun setDebugLogging(enabled: Boolean) {
        try {
            Log.d("ZipLockNative", "Setting debug logging to: $enabled")
            val result = library.ziplock_debug_logging(if (enabled) 1 else 0)
            Log.d("ZipLockNative", "Debug logging result: $result")
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Failed to set debug logging: ${e.message}", e)
        }
    }

    /**
     * Comprehensive FFI health check
     */
    fun performHealthCheck(): String {
        val results = mutableListOf<String>()

        try {
            // Test library loading
            results.add("Library loading: SUCCESS")

            // Test version function
            try {
                val version = getVersion()
                results.add("Version check: SUCCESS ($version)")
            } catch (e: Exception) {
                results.add("Version check: FAILED (${e.message})")
            }

            // Test version function (as connection test)
            try {
                val testResult = testConnection()
                results.add("Connection test: ${if (testResult) "SUCCESS" else "FAILED"}")
            } catch (e: Exception) {
                results.add("Connection test: FAILED (${e.message})")
            }

            // Test archive status check
            try {
                val isOpen = library.ziplock_is_archive_open()
                results.add("Archive status check: SUCCESS (open=$isOpen)")
            } catch (e: Exception) {
                results.add("Archive status check: FAILED (${e.message})")
            }

        } catch (e: Exception) {
            results.add("Library loading: FAILED (${e.message})")
        }

        return results.joinToString("\n")
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
    val updatedAt: Long = System.currentTimeMillis()
)

/**
 * Result of credential listing operation
 */
data class CredentialListResult(
    val success: Boolean,
    val credentials: List<Credential>,
    val errorMessage: String? = null
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
