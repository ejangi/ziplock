package com.ziplock.ffi

import android.content.Context
import android.util.Log
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * ZipLock Native FFI Interface using JNA for Hybrid Architecture
 *
 * This class provides the JNA wrapper for integrating with the hybrid ZipLock library.
 * It handles only data operations and crypto - archive operations are handled by ArchiveManager.
 *
 * The shared library handles:
 * - Credential management and validation
 * - Password generation and strength checking
 * - Data encryption/decryption
 * - Field validation (email, URL, etc.)
 *
 * Archive operations are deliberately excluded to prevent Android emulator crashes.
 */
object ZipLockNative {

    // Session state management
    private var currentSessionId: String? = null
    private var isArchiveCurrentlyOpen: Boolean = false

    // Application context for repository operations
    private var applicationContext: Context? = null

    // Global repository manager for credential persistence
    private var repositoryManager: com.ziplock.repository.HybridRepositoryManager? = null

    // In-memory credential storage for hybrid mode
    private var hybridCredentials: List<Credential> = emptyList()

    // JNA interface for the hybrid native library
    private interface ZipLockLibrary : Library {
        companion object {
            val INSTANCE: ZipLockLibrary = Native.load("ziplock_shared", ZipLockLibrary::class.java)
        }

        // Hybrid FFI functions (data/crypto operations only - no archive operations)
        fun ziplock_hybrid_init(): Int
        fun ziplock_hybrid_get_version(): Pointer?
        fun ziplock_hybrid_get_last_error(): Pointer?
        fun ziplock_hybrid_cleanup(): Int

        // Memory management
        fun ziplock_hybrid_string_free(ptr: Pointer?)
        fun ziplock_hybrid_credential_free(credentialId: Long)

        // Credential operations
        fun ziplock_hybrid_credential_new(title: String, credentialType: String): Long
        fun ziplock_hybrid_credential_add_field(
            credentialId: Long,
            name: String,
            fieldType: Int,
            value: String,
            label: String?,
            sensitive: Int
        ): Int
        fun ziplock_hybrid_credential_get_field(credentialId: Long, name: String): Pointer?
        fun ziplock_hybrid_credential_to_json(credentialId: Long): Pointer?
        fun ziplock_hybrid_credential_from_json(json: String): Long
        fun ziplock_hybrid_credential_validate(credentialId: Long): Int

        // Password and validation operations
        fun ziplock_hybrid_password_generate(
            length: Int,
            uppercase: Int,
            lowercase: Int,
            numbers: Int,
            symbols: Int
        ): Pointer?
        fun ziplock_hybrid_password_strength(password: String): Int
        fun ziplock_hybrid_email_validate(email: String): Int
        fun ziplock_hybrid_url_validate(url: String): Int

        // Data encryption operations
        fun ziplock_hybrid_encrypt_data(data: String, password: String): Pointer?
        fun ziplock_hybrid_decrypt_data(encryptedData: String, password: String): Pointer?
        fun ziplock_hybrid_generate_salt(): Pointer?

        // Utility operations
        fun ziplock_hybrid_test_echo(input: String): Pointer?
    }

    private val library = ZipLockLibrary.INSTANCE

    /**
     * Initialize the hybrid library
     * Should be called once when the app starts
     *
     * @return true if initialization was successful
     */
    fun init(): Boolean {
        return try {
            val result = library.ziplock_hybrid_init()
            Log.d("ZipLockNative", "Hybrid library initialization result: $result")
            result == 0
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, running in fallback mode: ${e.message}")
            // Return true to allow app to continue in fallback mode
            true
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Hybrid library initialization failed: ${e.message}")
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
            val ptr = library.ziplock_hybrid_get_version()
            val version = ptr?.getString(0) ?: "unknown"
            library.ziplock_hybrid_string_free(ptr)
            version
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, returning fallback version")
            "1.0.0-fallback"
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
            val strength = library.ziplock_hybrid_password_strength(passphrase)
            createPassphraseResult(passphrase, strength)
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, using fallback passphrase validation")
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
            val ptr = library.ziplock_hybrid_password_generate(
                length,
                1, // uppercase
                1, // lowercase
                1, // numbers
                if (includeSymbols) 1 else 0 // symbols
            )
            val password = ptr?.getString(0) ?: generateFallbackPassword(length, includeSymbols)
            library.ziplock_hybrid_string_free(ptr)
            password
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, using fallback password generation")
            generateFallbackPassword(length, includeSymbols)
        } catch (e: Exception) {
            generateFallbackPassword(length, includeSymbols)
        }
    }

    /**
     * Initialize session for working with extracted contents (hybrid approach)
     * This replaces archive operations - the actual archive is handled by ArchiveManager
     */
    fun initializeExtractedContentsSession(): ArchiveResult {
        return try {
            // In hybrid mode, we don't open archives directly
            // We just initialize a session for credential management
            currentSessionId = "hybrid_session_${System.currentTimeMillis()}"
            isArchiveCurrentlyOpen = true

            ArchiveResult(
                success = true,
                sessionId = currentSessionId,
                errorMessage = null
            )
        } catch (e: Exception) {
            currentSessionId = null
            isArchiveCurrentlyOpen = false

            ArchiveResult(
                success = false,
                sessionId = null,
                errorMessage = "Session initialization failed: ${e.message}",
                errorCode = 1
            )
        }
    }

    /**
     * Open extracted contents using hybrid approach
     * The archive has already been extracted by ArchiveManager
     */
    fun openExtractedContents(extractedPath: String, passphrase: String): ArchiveResult {
        return try {
            // In hybrid mode, we initialize session for credential management
            // The actual archive extraction is already done by ArchiveManager
            Log.d("ZipLockNative", "Initializing hybrid session for extracted contents at: $extractedPath")

            currentSessionId = "hybrid_session_${System.currentTimeMillis()}"
            isArchiveCurrentlyOpen = true

            // Load existing credentials from the extracted directory
            loadCredentialsFromExtractedDirectory(extractedPath)

            ArchiveResult(
                success = true,
                sessionId = currentSessionId,
                errorMessage = null
            )
        } catch (e: Exception) {
            currentSessionId = null
            isArchiveCurrentlyOpen = false

            ArchiveResult(
                success = false,
                sessionId = null,
                errorMessage = "Extracted contents session failed: ${e.message}",
                errorCode = 1
            )
        }
    }

    /**
     * Load credentials from the extracted directory into memory
     * This populates the hybridCredentials list with persisted data
     */
    private fun loadCredentialsFromExtractedDirectory(extractedPath: String) {
        try {
            Log.d("ZipLockNative", "=== DEBUGGING CREDENTIAL LOADING ===")
            Log.d("ZipLockNative", "Loading persisted credentials from extracted directory: $extractedPath")

            // List all files in the extracted directory to see what's actually there
            val extractedDir = java.io.File(extractedPath)
            if (extractedDir.exists() && extractedDir.isDirectory()) {
                val allFiles = extractedDir.listFiles()
                Log.d("ZipLockNative", "Files in extracted directory:")
                allFiles?.forEach { file ->
                    Log.d("ZipLockNative", "  - ${file.name} (${file.length()} bytes)")
                }
            } else {
                Log.e("ZipLockNative", "Extracted directory does not exist or is not a directory: $extractedPath")
                hybridCredentials = emptyList()
                return
            }

            // Check for credentials.json first
            val credentialsFile = java.io.File(extractedPath, "credentials.json")
            Log.d("ZipLockNative", "Looking for credentials.json: exists=${credentialsFile.exists()}")

            // Also check for YML files in case FFI library uses different format
            val credentialsYmlFile = java.io.File(extractedPath, "credentials.yml")
            val metadataYmlFile = java.io.File(extractedPath, "metadata.yml")
            Log.d("ZipLockNative", "Looking for credentials.yml: exists=${credentialsYmlFile.exists()}")
            Log.d("ZipLockNative", "Looking for metadata.yml: exists=${metadataYmlFile.exists()}")

            if (!credentialsFile.exists()) {
                Log.w("ZipLockNative", "No credentials.json file found in extracted directory")
                Log.w("ZipLockNative", "This means either:")
                Log.w("ZipLockNative", "  1. No credentials were saved previously")
                Log.w("ZipLockNative", "  2. Credentials are in a different format (YML?)")
                Log.w("ZipLockNative", "  3. File persistence failed during save")
                hybridCredentials = emptyList()
                return
            }

            val credentialsJson = credentialsFile.readText()
            Log.d("ZipLockNative", "Found credentials.json file, parsing JSON: ${credentialsJson.length} characters")
            Log.d("ZipLockNative", "Credentials JSON content preview: ${credentialsJson.take(200)}...")

            // Parse the JSON using kotlinx.serialization
            val json = kotlinx.serialization.json.Json {
                ignoreUnknownKeys = true
            }

            // Parse as HybridRepositoryManager.SerializedCredential format
            val serializedCredentials = json.decodeFromString<List<JsonObject>>(credentialsJson)

            val loadedCredentials = mutableListOf<Credential>()

            for (jsonObj in serializedCredentials) {
                try {
                    val id = jsonObj["id"]?.jsonPrimitive?.content ?: continue
                    val title = jsonObj["title"]?.jsonPrimitive?.content ?: continue
                    val type = jsonObj["type"]?.jsonPrimitive?.content ?: "login"
                    val fieldsObj = jsonObj["fields"]?.jsonObject ?: continue
                    val createdAt = jsonObj["createdAt"]?.jsonPrimitive?.content?.toLongOrNull() ?: System.currentTimeMillis()
                    val updatedAt = jsonObj["updatedAt"]?.jsonPrimitive?.content?.toLongOrNull() ?: System.currentTimeMillis()

                    // Extract fields
                    val username = fieldsObj["username"]?.jsonPrimitive?.content ?: ""
                    val password = fieldsObj["password"]?.jsonPrimitive?.content ?: ""
                    val url = fieldsObj["url"]?.jsonPrimitive?.content ?: ""
                    val notes = fieldsObj["notes"]?.jsonPrimitive?.content ?: ""

                    // Extract tags (if present)
                    val tagsArray = jsonObj["tags"]?.jsonArray
                    val tags = tagsArray?.mapNotNull { it.jsonPrimitive?.content } ?: emptyList()

                    val credential = Credential(
                        id = id,
                        title = title,
                        credentialType = type,
                        username = username,
                        password = password,
                        url = url,
                        notes = notes,
                        tags = tags,
                        createdAt = createdAt,
                        updatedAt = updatedAt
                    )

                    loadedCredentials.add(credential)
                    Log.d("ZipLockNative", "Loaded credential: ${credential.title} (${credential.id})")

                } catch (e: Exception) {
                    Log.w("ZipLockNative", "Failed to parse credential from JSON: ${e.message}")
                }
            }

            hybridCredentials = loadedCredentials
            Log.d("ZipLockNative", "Successfully loaded ${hybridCredentials.size} credentials from extracted directory")
            Log.d("ZipLockNative", "=== CREDENTIAL LOADING COMPLETE ===")

        } catch (e: Exception) {
            Log.e("ZipLockNative", "CRITICAL ERROR: Failed to load credentials from extracted directory")
            Log.e("ZipLockNative", "Error message: ${e.message}")
            Log.e("ZipLockNative", "Error type: ${e.javaClass.simpleName}")
            Log.e("ZipLockNative", "Stack trace:", e)
            hybridCredentials = emptyList()
        }
    }

    /**
     * List all credentials using hybrid approach
     * Note: In hybrid mode, credentials are managed in-memory and need to be
     * loaded/saved through the HybridRepositoryManager
     */
    fun listCredentials(): CredentialListResult {
        return try {
            // Check if session is active
            if (!isArchiveOpen()) {
                return CredentialListResult(
                    success = false,
                    credentials = emptyList(),
                    errorMessage = "No hybrid session is currently active"
                )
            }

            Log.d("ZipLockNative", "Listing credentials in hybrid mode...")
            Log.d("ZipLockNative", "Current session ID: $currentSessionId")
            Log.d("ZipLockNative", "Archive currently open: $isArchiveCurrentlyOpen")

            // In hybrid mode, return credentials from in-memory storage
            Log.d("ZipLockNative", "Hybrid mode: returning ${hybridCredentials.size} credentials from memory")

            // Debug: Show details of each credential
            hybridCredentials.forEachIndexed { index, credential ->
                Log.d("ZipLockNative", "  Credential $index: ${credential.title} (ID: ${credential.id})")
            }

            return CredentialListResult(
                success = true,
                credentials = hybridCredentials,
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
     * Create a new credential using hybrid approach
     * Returns a credential ID that can be used for further operations
     */
    fun createCredential(title: String, credentialType: String): HybridCredentialResult {
        return try {
            Log.d("ZipLockNative", "Creating credential: $title")

            // Check if session is active
            if (!isArchiveOpen()) {
                return HybridCredentialResult(
                    success = false,
                    credentialId = 0,
                    errorMessage = "No hybrid session is currently active"
                )
            }

            // Call hybrid FFI function to create credential
            val credentialId = library.ziplock_hybrid_credential_new(title, credentialType)

            if (credentialId != 0L) {
                Log.d("ZipLockNative", "Successfully created credential: $title (ID: $credentialId)")
                HybridCredentialResult(
                    success = true,
                    credentialId = credentialId,
                    errorMessage = null
                )
            } else {
                val errorMsg = getHybridLastError()
                Log.e("ZipLockNative", "Failed to create credential: $errorMsg")
                HybridCredentialResult(
                    success = false,
                    credentialId = 0,
                    errorMessage = errorMsg
                )
            }
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Exception in createCredential: ${e.message}", e)
            HybridCredentialResult(
                success = false,
                credentialId = 0,
                errorMessage = "Failed to create credential: ${e.message}"
            )
        }
    }

    /**
     * Add a field to an existing credential
     */
    fun addCredentialField(
        credentialId: Long,
        name: String,
        value: String,
        fieldType: FieldType = FieldType.TEXT,
        sensitive: Boolean = false
    ): HybridOperationResult {
        return try {
            if (!isArchiveOpen()) {
                return HybridOperationResult(
                    success = false,
                    errorMessage = "No hybrid session is currently active"
                )
            }

            val result = library.ziplock_hybrid_credential_add_field(
                credentialId,
                name,
                fieldType.value,
                value,
                null, // label
                if (sensitive) 1 else 0
            )

            if (result == 0) {
                HybridOperationResult(success = true)
            } else {
                val errorMsg = getHybridLastError()
                HybridOperationResult(success = false, errorMessage = errorMsg)
            }
        } catch (e: Exception) {
            HybridOperationResult(
                success = false,
                errorMessage = "Failed to add field: ${e.message}"
            )
        }
    }

    /**
     * Get a field value from a credential
     */
    fun getCredentialField(credentialId: Long, fieldName: String): String? {
        return try {
            val ptr = library.ziplock_hybrid_credential_get_field(credentialId, fieldName)
            val value = ptr?.getString(0)
            library.ziplock_hybrid_string_free(ptr)
            value
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Validate email address
     */
    fun validateEmail(email: String): Boolean {
        return try {
            library.ziplock_hybrid_email_validate(email) == 1
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, using fallback email validation")
            // Simple fallback email validation
            email.contains("@") && email.contains(".") && email.length > 5
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Validate URL
     */
    fun validateUrl(url: String): Boolean {
        return try {
            library.ziplock_hybrid_url_validate(url) == 1
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, using fallback URL validation")
            // Simple fallback URL validation
            url.startsWith("http://") || url.startsWith("https://") || url.startsWith("ftp://")
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Close the current hybrid session and clear session state
     */
    fun closeArchive(): Boolean {
        return try {
            Log.d("ZipLockNative", "Closing hybrid session...")

            // In hybrid mode, we just need to cleanup the session
            val result = library.ziplock_hybrid_cleanup()

            if (result == 0) {
                Log.d("ZipLockNative", "Hybrid session closed successfully")
                currentSessionId = null
                isArchiveCurrentlyOpen = false
                hybridCredentials = emptyList() // Clear credentials from memory
                true
            } else {
                Log.e("ZipLockNative", "Failed to close hybrid session")
                false
            }
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, clearing session state in fallback mode")
            currentSessionId = null
            isArchiveCurrentlyOpen = false
            hybridCredentials = emptyList() // Clear credentials from memory
            true
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Exception closing hybrid session: ${e.message}", e)
            // Clear state even on exception to prevent stuck state
            currentSessionId = null
            isArchiveCurrentlyOpen = false
            false
        }
    }

    /**
     * Check if a hybrid session is currently active
     */
    fun isArchiveOpen(): Boolean {
        return try {
            isArchiveCurrentlyOpen
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Error checking session status: ${e.message}", e)
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
     * Get last error from hybrid FFI
     */
    private fun getHybridLastError(): String {
        return try {
            val ptr = library.ziplock_hybrid_get_last_error()
            val error = ptr?.getString(0) ?: "Unknown error"
            library.ziplock_hybrid_string_free(ptr)
            error
        } catch (e: UnsatisfiedLinkError) {
            "Native library not available"
        } catch (e: Exception) {
            "Failed to get error: ${e.message}"
        }
    }

    // Helper functions for passphrase validation
    private fun createPassphraseResult(passphrase: String, strength: Int): PassphraseStrengthResult {
        val score = (strength * 20).coerceAtMost(100) // Convert 0-5 scale to 0-100
        val strengthLabel = when (strength) {
            0 -> "Very Weak"
            1 -> "Weak"
            2 -> "Fair"
            3 -> "Good"
            4 -> "Strong"
            5 -> "Very Strong"
            else -> "Unknown"
        }

        val requirements = mutableListOf<String>()
        val satisfied = mutableListOf<String>()

        // Basic validation
        if (passphrase.length < 12) {
            requirements.add("Must be at least 12 characters long")
        } else {
            satisfied.add("Length requirement met")
        }

        return PassphraseStrengthResult(
            score = score,
            strength = strengthLabel,
            requirements = requirements,
            satisfied = satisfied,
            isValid = requirements.isEmpty() && score >= 60
        )
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

    // Data classes for hybrid FFI results
    data class HybridCredentialResult(
        val success: Boolean,
        val credentialId: Long = 0,
        val errorMessage: String? = null
    )

    data class HybridOperationResult(
        val success: Boolean,
        val errorMessage: String? = null
    )

    data class ArchiveResult(
        val success: Boolean,
        val sessionId: String? = null,
        val errorMessage: String? = null,
        val errorCode: Int = 0
    )

    data class CredentialListResult(
        val success: Boolean,
        val credentials: List<Credential>,
        val errorMessage: String? = null
    )

    data class PassphraseStrengthResult(
        val score: Int,
        val strength: String,
        val requirements: List<String>,
        val satisfied: List<String>,
        val isValid: Boolean
    )

    data class Credential(
        val id: String = "",
        val title: String = "",
        val credentialType: String = "",
        val username: String = "",
        val password: String = "",
        val url: String = "",
        val notes: String = "",
        val tags: List<String> = emptyList(),
        val createdAt: Long = 0,
        val updatedAt: Long = 0
    )

    enum class FieldType(val value: Int) {
        TEXT(0),
        PASSWORD(1),
        EMAIL(2),
        URL(3),
        USERNAME(4),
        PHONE(5),
        CREDIT_CARD_NUMBER(6),
        EXPIRY_DATE(7),
        CVV(8),
        TOTP_SECRET(9),
        TEXT_AREA(10),
        NUMBER(11),
        DATE(12),
        CUSTOM(13)
    }

    /**
     * Test the native library connection
     */
    fun testConnection(): Boolean {
        return try {
            Log.d("ZipLockNative", "Testing library connection with version check...")
            val version = getVersion()
            val result = version.isNotBlank() && version != "unknown"
            Log.d("ZipLockNative", "Version test result: $result (version: '$version')")
            result
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, connection test failed gracefully")
            true // Return true to allow fallback mode
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Version test failed with exception: ${e.message}", e)
            false
        }
    }

    /**
     * Get the last error message from the native library
     */
    fun getLastError(): String? {
        return try {
            getHybridLastError()
        } catch (e: UnsatisfiedLinkError) {
            Log.w("ZipLockNative", "Native library not available, no error to retrieve")
            null
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Failed to get last error: ${e.message}")
            null
        }
    }

    /**
     * Check if Android SAF is available
     */
    fun isAndroidSafAvailable(): Boolean {
        return try {
            // Android SAF not available in hybrid mode
            false
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Failed to check Android SAF availability: ${e.message}")
            false
        }
    }

    // Callback interfaces for Android SAF operations (for compatibility)
    interface AndroidSafOpenCallback {
        fun callback(contentUri: String): Int
    }

    interface AndroidSafReadCallback {
        fun callback(fd: Int, buffer: Pointer, size: Int): Int
    }

    interface AndroidSafWriteCallback {
        fun callback(fd: Int, data: Pointer, size: Int): Int
    }

    interface AndroidSafCloseCallback {
        fun callback(fd: Int): Int
    }

    interface AndroidSafGetSizeCallback {
        fun callback(fd: Int): Long
    }

    interface AndroidSafCreateTempFileCallback {
        fun callback(name: String, pathOut: com.sun.jna.ptr.PointerByReference): Int
    }

    // Debug and testing functions
    fun testLogging(message: String): Boolean {
        return try {
            Log.d("ZipLockNative", "Test logging: $message")
            true
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Test logging failed: ${e.message}")
            false
        }
    }

    fun enableDebugLogging(): Boolean {
        return try {
            Log.d("ZipLockNative", "Debug logging enabled")
            true
        } catch (e: Exception) {
            false
        }
    }

    fun disableDebugLogging(): Boolean {
        return try {
            Log.d("ZipLockNative", "Debug logging disabled")
            true
        } catch (e: Exception) {
            false
        }
    }

    fun isDebugLoggingEnabled(): Boolean {
        return true // Always enabled in debug builds
    }

    fun configureLogging(level: String): Boolean {
        return try {
            Log.d("ZipLockNative", "Logging configured to level: $level")
            true
        } catch (e: Exception) {
            false
        }
    }

    fun testAndroidSaf(contentUri: String): Boolean {
        return try {
            Log.d("ZipLockNative", "Testing Android SAF with URI: $contentUri")
            false // Not supported in hybrid mode
        } catch (e: Exception) {
            false
        }
    }

    fun isAndroidEmulator(): Boolean {
        return try {
            val brand = android.os.Build.BRAND
            val device = android.os.Build.DEVICE
            val model = android.os.Build.MODEL
            val product = android.os.Build.PRODUCT

            brand.startsWith("generic") ||
            device.startsWith("generic") ||
            model.contains("Emulator") ||
            model.contains("Android SDK") ||
            product.contains("sdk") ||
            product.contains("emulator")
        } catch (e: Exception) {
            false
        }
    }

    fun hasArchiveCompatibilityIssues(): Boolean {
        return isAndroidEmulator() // Emulators may have compatibility issues
    }

    fun getPlatformCompatibilityWarning(): String? {
        return if (hasArchiveCompatibilityIssues()) {
            "Running on Android emulator - some archive operations may have reduced performance"
        } else {
            null
        }
    }

    fun getAndroidPlatformDescription(): String {
        return try {
            val version = android.os.Build.VERSION.RELEASE
            val sdk = android.os.Build.VERSION.SDK_INT
            val brand = android.os.Build.BRAND
            val model = android.os.Build.MODEL
            "Android $version (API $sdk) on $brand $model"
        } catch (e: Exception) {
            "Android (unknown version)"
        }
    }

    fun createArchive(
        archivePath: String,
        passphrase: String,
        compressionLevel: Int = 5
    ): ArchiveResult {
        return try {
            Log.d("ZipLockNative", "Creating archive in hybrid mode: $archivePath")
            // In hybrid mode, archive creation is handled by ArchiveManager
            ArchiveResult(
                success = false,
                errorMessage = "Archive creation should be handled by ArchiveManager in hybrid mode"
            )
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Archive creation failed: ${e.message}"
            )
        }
    }

    fun openArchive(archivePath: String, passphrase: String): ArchiveResult {
        return try {
            Log.d("ZipLockNative", "Opening archive in hybrid mode: $archivePath")
            // In hybrid mode, archive operations are handled by ArchiveManager
            ArchiveResult(
                success = false,
                errorMessage = "Archive opening should be handled by ArchiveManager in hybrid mode"
            )
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Archive opening failed: ${e.message}"
            )
        }
    }

    fun saveCredential(credential: Credential): Boolean {
        return try {
            Log.d("ZipLockNative", "Saving credential: ${credential.title}")

            if (isArchiveOpen()) {
                Log.d("ZipLockNative", "Archive is open, saving credential with persistence")

                // Add credential to internal list for immediate availability
                val credentialsList = hybridCredentials.toMutableList()

                // Remove existing credential with same ID if updating
                credentialsList.removeAll { it.id == credential.id }

                // Add the new/updated credential
                credentialsList.add(credential)
                hybridCredentials = credentialsList

                Log.d("ZipLockNative", "Updated in-memory credentials list. Total count: ${hybridCredentials.size}")

                // Trigger persistence to disk through HybridRepositoryManager
                try {
                    val repoManager = repositoryManager
                    if (repoManager != null && repoManager.isRepositoryOpen()) {
                        // Convert to SerializedCredential format and save asynchronously
                        kotlinx.coroutines.GlobalScope.launch {
                            try {
                                // Convert ZipLockNative.Credential to SerializedCredential format
                                val serializedCredentials = hybridCredentials.map { cred ->
                                    // Convert credential fields to map format
                                    val fields = mutableMapOf<String, String>()
                                    val sensitiveFields = mutableSetOf<String>()

                                    // Add standard fields
                                    if (cred.url.isNotEmpty()) fields["url"] = cred.url
                                    if (cred.username.isNotEmpty()) {
                                        fields["username"] = cred.username
                                        sensitiveFields.add("username")
                                    }
                                    if (cred.password.isNotEmpty()) {
                                        fields["password"] = cred.password
                                        sensitiveFields.add("password")
                                    }
                                    if (cred.notes.isNotEmpty()) fields["notes"] = cred.notes

                                    // Note: Custom fields not implemented in current Credential data class
                                    // Future enhancement: add customFields: Map<String, String> to Credential

                                    com.ziplock.repository.HybridRepositoryManager.SerializedCredential(
                                        id = cred.id,
                                        title = cred.title,
                                        type = cred.credentialType,
                                        fields = fields,
                                        sensitiveFields = sensitiveFields,
                                        tags = cred.tags.toSet(),
                                        createdAt = System.currentTimeMillis(),
                                        updatedAt = System.currentTimeMillis()
                                    )
                                }

                                // Save directly to repository using internal method
                                val saveResult = repoManager.saveSerializedCredentials(serializedCredentials)
                                if (saveResult.success) {
                                    Log.d("ZipLockNative", "Credentials successfully persisted to disk")
                                } else {
                                    Log.w("ZipLockNative", "Failed to persist credentials: ${saveResult.errorMessage}")
                                }
                            } catch (e: Exception) {
                                Log.e("ZipLockNative", "Exception during credential persistence: ${e.message}")
                            }
                        }
                    } else {
                        Log.w("ZipLockNative", "Repository manager not available or no repository open, credential saved in memory only")
                    }
                } catch (e: Exception) {
                    Log.w("ZipLockNative", "Failed to trigger persistence, credential saved in memory only: ${e.message}")
                }

                Log.d("ZipLockNative", "Credential saved successfully. Total credentials: ${hybridCredentials.size}")
                true
            } else {
                Log.w("ZipLockNative", "No archive is open")
                false
            }
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Failed to save credential: ${e.message}")
            false
        }
    }

    /**
     * Set application context for repository operations
     */
    fun setApplicationContext(context: Context) {
        applicationContext = context.applicationContext
        Log.d("ZipLockNative", "Application context set for credential persistence")
    }

    /**
     * Set the active repository manager instance for credential persistence
     */
    fun setRepositoryManager(manager: com.ziplock.repository.HybridRepositoryManager?) {
        repositoryManager = manager
        if (manager != null) {
            Log.d("ZipLockNative", "Repository manager set for credential persistence")
        } else {
            Log.d("ZipLockNative", "Repository manager cleared")
        }
    }

    /**
     * Get the current repository manager instance
     */
    fun getRepositoryManager(): com.ziplock.repository.HybridRepositoryManager? = repositoryManager

    fun testContentUriAccess(contentUri: String): Boolean {
        return try {
            Log.d("ZipLockNative", "Testing content URI access: $contentUri")
            false // Not supported in hybrid mode
        } catch (e: Exception) {
            false
        }
    }

    fun validateLibrary(): Boolean {
        return try {
            val version = getVersion()
            version.isNotBlank() && version != "unknown"
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Initialize Android SAF with callbacks
     */
    fun initializeAndroidSaf(context: Context): Boolean {
        return try {
            // Android SAF initialization not available in hybrid mode
            false
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Failed to initialize Android SAF: ${e.message}")
            false
        }
    }

    /**
     * Cleanup Android SAF resources
     */
    fun cleanupAndroidSaf(): Boolean {
        return try {
            // Android SAF cleanup not available in hybrid mode
            true
        } catch (e: Exception) {
            Log.e("ZipLockNative", "Failed to cleanup Android SAF: ${e.message}")
            false
        }
    }
}

/**
 * Helper functions for working with the native library
 */
object ZipLockNativeHelper {

    /**
     * Get all available credential templates
     */
    fun getAllTemplates(): List<CredentialTemplate> {
        return getBuiltinTemplates()
    }

    /**
     * Get a specific credential template by name
     */
    fun getTemplateByName(name: String): CredentialTemplate? {
        return getBuiltinTemplates().find { it.name == name }
    }

    /**
     * Get a template for a specific credential type
     */
    fun getTemplateForType(credentialType: String): CredentialTemplate {
        return getTemplateByName(credentialType) ?: getTemplateByName("login")!!
    }

    /**
     * Get built-in templates
     */
    private fun getBuiltinTemplates(): List<CredentialTemplate> {
        return listOf(
            CredentialTemplate(
                name = "login",
                description = "Website or application login",
                fields = listOf(
                    FieldTemplate("username", "Username", "Username", false, false, null, null),
                    FieldTemplate("password", "Password", "Password", false, true, null, null),
                    FieldTemplate("website", "Url", "Website", false, false, null, null)
                ),
                defaultTags = listOf("login")
            ),
            CredentialTemplate(
                name = "credit_card",
                description = "Credit card information",
                fields = listOf(
                    FieldTemplate("cardholder", "Text", "Cardholder Name", false, false, null, null),
                    FieldTemplate("number", "CreditCardNumber", "Card Number", false, true, null, null),
                    FieldTemplate("expiry", "ExpiryDate", "Expiry Date", false, false, null, null),
                    FieldTemplate("cvv", "Cvv", "CVV", false, true, null, null)
                ),
                defaultTags = listOf("credit_card")
            ),
            CredentialTemplate(
                name = "secure_note",
                description = "Secure note or document",
                fields = listOf(
                    FieldTemplate("content", "TextArea", "Content", false, false, null, null)
                ),
                defaultTags = listOf("note")
            )
        )
    }

    // Data classes for templates
    data class CredentialTemplate(
        val name: String,
        val description: String,
        val fields: List<FieldTemplate>,
        val defaultTags: List<String>
    )

    data class FieldTemplate(
        val name: String,
        val fieldType: String,
        val label: String,
        val required: Boolean,
        val sensitive: Boolean,
        val defaultValue: String?,
        val validation: FieldValidation?
    )

    data class FieldValidation(
        val minLength: Int?,
        val maxLength: Int?,
        val pattern: String?,
        val message: String?
    )
}
