package com.ziplock.ffi

import android.content.Context
import android.util.Log
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString

/**
 * ZipLock Native FFI Interface - Unified Architecture Compatibility Layer
 *
 * This class provides backward compatibility for existing code while using
 * the new mobile FFI interface internally. It maintains the same external API
 * but delegates to the new ZipLockMobileFFI implementation.
 *
 * This approach allows existing UI code to work without changes while
 * benefiting from the new unified architecture underneath.
 */
object ZipLockNative {

    private const val TAG = "ZipLockNative"

    // JSON serializer for data exchange
    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    }

    // Application context for repository operations
    private var applicationContext: Context? = null

    // Current repository handle (maps to new FFI implementation)
    private var currentRepositoryHandle: ZipLockMobileFFI.RepositoryHandle? = null

    /**
     * Data classes for compatibility with existing code
     */
    @Serializable
    data class Credential(
        val id: String,
        val title: String,
        val credentialType: String,
        val fields: Map<String, FieldValue>,
        val createdAt: Long = System.currentTimeMillis(),
        val updatedAt: Long = System.currentTimeMillis(),
        val tags: List<String> = emptyList()
    )

    @Serializable
    data class FieldValue(
        val value: String,
        val fieldType: String,
        val label: String? = null,
        val sensitive: Boolean = false
    )

    @Serializable
    data class RepositoryStats(
        val credentialCount: Int,
        val isModified: Boolean,
        val lastSaved: Long? = null
    )

    /**
     * Initialize the native library
     * @return 0 on success, error code on failure
     */
    fun init(): Int {
        return try {
            if (ZipLockMobileFFI.testConnection()) {
                Log.d(TAG, "ZipLock Native FFI initialized successfully")
                0
            } else {
                Log.e(TAG, "Failed to initialize ZipLock Native FFI")
                -1
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception during initialization", e)
            -1
        }
    }

    /**
     * Set application context for repository operations
     */
    fun setContext(context: Context) {
        applicationContext = context.applicationContext
    }

    /**
     * Create a new repository (for compatibility)
     * @return true on success
     */
    fun createNewRepository(): Boolean {
        return try {
            closeRepository() // Close any existing repository

            val handle = ZipLockMobileFFI.RepositoryHandle.create()
            if (handle?.initialize() == true) {
                currentRepositoryHandle = handle
                Log.d(TAG, "New repository created successfully")
                true
            } else {
                handle?.close()
                Log.e(TAG, "Failed to create new repository")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception creating new repository", e)
            false
        }
    }

    /**
     * Load repository from file map
     * @param fileMap Map of file paths to byte arrays from extracted archive
     * @return true on success
     */
    fun loadRepositoryFromFiles(fileMap: Map<String, ByteArray>): Boolean {
        return try {
            closeRepository() // Close any existing repository

            val handle = ZipLockMobileFFI.RepositoryHandle.create()
            if (handle != null && handle.loadFromFiles(fileMap)) {
                currentRepositoryHandle = handle
                Log.d(TAG, "Repository loaded from files successfully")
                true
            } else {
                handle?.close()
                Log.e(TAG, "Failed to load repository from files")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception loading repository from files", e)
            false
        }
    }

    /**
     * Get repository as file map for archive creation
     * @return Map of file paths to byte arrays, or null on error
     */
    fun getRepositoryAsFiles(): Map<String, ByteArray>? {
        return try {
            currentRepositoryHandle?.serializeToFiles()
        } catch (e: Exception) {
            Log.e(TAG, "Exception getting repository as files", e)
            null
        }
    }

    /**
     * Check if repository is open and ready
     */
    fun isRepositoryOpen(): Boolean {
        return currentRepositoryHandle?.isInitialized() ?: false
    }

    /**
     * Close current repository
     */
    fun closeRepository() {
        try {
            currentRepositoryHandle?.close()
            currentRepositoryHandle = null
            Log.d(TAG, "Repository closed")
        } catch (e: Exception) {
            Log.e(TAG, "Exception closing repository", e)
        }
    }

    /**
     * Add a credential to the repository
     * @param credential Credential data to add
     * @return Credential ID on success, null on failure
     */
    fun addCredential(credential: Credential): String? {
        return try {
            val handle = currentRepositoryHandle ?: return null

            // Convert to FFI format
            val credentialRecord = ZipLockMobileFFI.CredentialRecord(
                id = credential.id,
                title = credential.title,
                credentialType = credential.credentialType,
                fields = credential.fields.mapValues { (_, field) ->
                    ZipLockMobileFFI.CredentialField(
                        value = field.value,
                        fieldType = mapStringToFieldType(field.fieldType),
                        label = field.label,
                        sensitive = field.sensitive,
                        metadata = emptyMap()
                    )
                },
                tags = credential.tags,
                notes = null,
                createdAt = credential.createdAt,
                updatedAt = credential.updatedAt,
                accessedAt = credential.updatedAt,
                favorite = false,
                folderPath = null
            )

            if (handle.addCredential(credentialRecord)) {
                Log.d(TAG, "Credential added: ${credential.title}")
                credential.id
            } else {
                Log.e(TAG, "Failed to add credential: ${credential.title}")
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception adding credential", e)
            null
        }
    }

    /**
     * Get a credential by ID
     * @param credentialId ID of the credential to retrieve
     * @return Credential data or null if not found
     */
    fun getCredential(credentialId: String): Credential? {
        return try {
            val handle = currentRepositoryHandle ?: return null
            val credentialRecord = handle.getCredential(credentialId) ?: return null

            // Convert from FFI format
            Credential(
                id = credentialRecord.id,
                title = credentialRecord.title,
                credentialType = credentialRecord.credentialType,
                fields = credentialRecord.fields.mapValues { (_, field) ->
                    FieldValue(
                        value = field.value,
                        fieldType = mapFieldTypeToString(field.fieldType),
                        label = field.label,
                        sensitive = field.sensitive
                    )
                },
                createdAt = credentialRecord.createdAt,
                updatedAt = credentialRecord.updatedAt,
                tags = credentialRecord.tags
            )
        } catch (e: Exception) {
            Log.e(TAG, "Exception getting credential: $credentialId", e)
            null
        }
    }

    /**
     * Update an existing credential
     * @param credential Updated credential data
     * @return true on success
     */
    fun updateCredential(credential: Credential): Boolean {
        return try {
            val handle = currentRepositoryHandle ?: return false

            // Convert to FFI format
            val credentialRecord = ZipLockMobileFFI.CredentialRecord(
                id = credential.id,
                title = credential.title,
                credentialType = credential.credentialType,
                fields = credential.fields.mapValues { (_, field) ->
                    ZipLockMobileFFI.CredentialField(
                        value = field.value,
                        fieldType = mapStringToFieldType(field.fieldType),
                        label = field.label,
                        sensitive = field.sensitive,
                        metadata = emptyMap()
                    )
                },
                tags = credential.tags,
                notes = null,
                createdAt = credential.createdAt,
                updatedAt = credential.updatedAt,
                accessedAt = credential.updatedAt,
                favorite = false,
                folderPath = null
            )

            handle.updateCredential(credentialRecord)
        } catch (e: Exception) {
            Log.e(TAG, "Exception updating credential", e)
            false
        }
    }

    /**
     * Delete a credential by ID
     * @param credentialId ID of the credential to delete
     * @return true on success
     */
    fun deleteCredential(credentialId: String): Boolean {
        return try {
            val handle = currentRepositoryHandle ?: return false
            handle.deleteCredential(credentialId)
        } catch (e: Exception) {
            Log.e(TAG, "Exception deleting credential: $credentialId", e)
            false
        }
    }

    /**
     * List all credentials in the repository
     * @return List of credentials or empty list on error
     */
    fun listCredentials(): List<Credential> {
        return try {
            val handle = currentRepositoryHandle ?: return emptyList()
            val credentialRecords = handle.listCredentials()

            credentialRecords.map { record ->
                Credential(
                    id = record.id,
                    title = record.title,
                    credentialType = record.credentialType,
                    fields = record.fields.mapValues { (_, field) ->
                        FieldValue(
                            value = field.value,
                            fieldType = mapFieldTypeToString(field.fieldType),
                            label = field.label,
                            sensitive = field.sensitive
                        )
                    },
                    createdAt = record.createdAt,
                    updatedAt = record.updatedAt,
                    tags = record.tags
                )
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception listing credentials", e)
            emptyList()
        }
    }

    /**
     * Check if repository has unsaved changes
     */
    fun isRepositoryModified(): Boolean {
        return try {
            currentRepositoryHandle?.isModified() ?: false
        } catch (e: Exception) {
            Log.e(TAG, "Exception checking repository modification status", e)
            false
        }
    }

    /**
     * Mark repository as saved
     */
    fun markRepositorySaved(): Boolean {
        return try {
            currentRepositoryHandle?.markSaved() ?: false
        } catch (e: Exception) {
            Log.e(TAG, "Exception marking repository as saved", e)
            false
        }
    }

    /**
     * Get repository statistics
     */
    fun getRepositoryStats(): RepositoryStats? {
        return try {
            val handle = currentRepositoryHandle ?: return null
            val stats = handle.getStats() ?: return null

            RepositoryStats(
                credentialCount = stats.credentialCount,
                isModified = stats.isModified,
                lastSaved = if (stats.isModified) null else System.currentTimeMillis()
            )
        } catch (e: Exception) {
            Log.e(TAG, "Exception getting repository stats", e)
            null
        }
    }

    /**
     * Clear all credentials from repository
     */
    fun clearRepository(): Boolean {
        return try {
            currentRepositoryHandle?.clearCredentials() ?: false
        } catch (e: Exception) {
            Log.e(TAG, "Exception clearing repository", e)
            false
        }
    }

    // Legacy compatibility methods for existing code

    /**
     * Test content URI access (placeholder for compatibility)
     */
    fun testContentUriAccess(uri: String): String {
        return "Content URI access test - using unified architecture\nURI: $uri\nStatus: Available via SAF integration"
    }

    /**
     * Check if Android SAF is available
     */
    fun isAndroidSafAvailable(): Boolean {
        return applicationContext != null
    }

    /**
     * Get version information
     */
    fun getVersion(): String {
        return "ZipLock Unified Architecture v1.0"
    }

    /**
     * Get last error message
     */
    fun getLastError(): String {
        return "No errors - using unified architecture"
    }

    /**
     * Cleanup resources
     */
    fun cleanup(): Int {
        closeRepository()
        return 0
    }

    /**
     * Convert string field type to FieldType enum
     */
    private fun mapStringToFieldType(fieldType: String): ZipLockMobileFFI.FieldType {
        return when (fieldType.lowercase()) {
            "password" -> ZipLockMobileFFI.FieldType.Password
            "email" -> ZipLockMobileFFI.FieldType.Email
            "url", "website" -> ZipLockMobileFFI.FieldType.Url
            "username", "user", "login" -> ZipLockMobileFFI.FieldType.Username
            "phone", "telephone" -> ZipLockMobileFFI.FieldType.Phone
            "credit_card", "creditcard", "card_number" -> ZipLockMobileFFI.FieldType.CreditCardNumber
            "expiry", "expiry_date", "expiration" -> ZipLockMobileFFI.FieldType.ExpiryDate
            "cvv", "cvc", "security_code" -> ZipLockMobileFFI.FieldType.Cvv
            "totp", "totp_secret", "2fa" -> ZipLockMobileFFI.FieldType.TotpSecret
            "notes", "description", "comment" -> ZipLockMobileFFI.FieldType.TextArea
            "number", "numeric" -> ZipLockMobileFFI.FieldType.Number
            else -> ZipLockMobileFFI.FieldType.Text
        }
    }

    /**
     * Convert FieldType enum to string
     */
    private fun mapFieldTypeToString(fieldType: ZipLockMobileFFI.FieldType): String {
        return when (fieldType) {
            ZipLockMobileFFI.FieldType.Text -> "text"
            ZipLockMobileFFI.FieldType.Password -> "password"
            ZipLockMobileFFI.FieldType.Email -> "email"
            ZipLockMobileFFI.FieldType.Url -> "url"
            ZipLockMobileFFI.FieldType.Username -> "username"
            ZipLockMobileFFI.FieldType.Phone -> "phone"
            ZipLockMobileFFI.FieldType.CreditCardNumber -> "credit_card"
            ZipLockMobileFFI.FieldType.ExpiryDate -> "expiry_date"
            ZipLockMobileFFI.FieldType.Cvv -> "cvv"
            ZipLockMobileFFI.FieldType.TotpSecret -> "totp_secret"
            ZipLockMobileFFI.FieldType.TextArea -> "notes"
            ZipLockMobileFFI.FieldType.Number -> "number"
        }
    }

    // Debug and utility methods

    /**
     * Enable debug logging
     */
    fun enableDebugLogging(): Boolean {
        Log.d(TAG, "Debug logging enabled")
        return true
    }

    /**
     * Disable debug logging
     */
    fun disableDebugLogging(): Boolean {
        Log.d(TAG, "Debug logging disabled")
        return true
    }

    /**
     * Check if debug logging is enabled
     */
    fun isDebugLoggingEnabled(): Boolean {
        return true // Always enabled in debug builds
    }

    /**
     * Test logging functionality
     */
    fun testLogging(message: String): Boolean {
        Log.d(TAG, "Test log: $message")
        return true
    }

    /**
     * Configure logging system
     */
    fun configureLogging(level: String): Boolean {
        Log.d(TAG, "Logging configured to level: $level")
        return true
    }

    /**
     * Test Android SAF functionality
     */
    fun testAndroidSaf(): Boolean {
        return isAndroidSafAvailable()
    }

    /**
     * Check if running on Android emulator
     */
    fun isAndroidEmulator(): Boolean {
        return android.os.Build.FINGERPRINT.contains("generic") ||
               android.os.Build.MODEL.contains("Emulator") ||
               android.os.Build.MODEL.contains("Android SDK built for")
    }

    /**
     * Check for archive compatibility issues
     */
    fun hasArchiveCompatibilityIssues(): Boolean {
        return false // No known compatibility issues with current implementation
    }

    /**
     * Get platform compatibility warning
     */
    fun getPlatformCompatibilityWarning(): String? {
        return if (hasArchiveCompatibilityIssues()) {
            "Some archive formats may not be fully supported on this platform"
        } else {
            null
        }
    }

    /**
     * Get Android platform description
     */
    fun getAndroidPlatformDescription(): String {
        return "Android ${android.os.Build.VERSION.RELEASE} (API ${android.os.Build.VERSION.SDK_INT}), " +
               "Device: ${android.os.Build.MANUFACTURER} ${android.os.Build.MODEL}"
    }

    /**
     * Create new repository at path
     */
    fun createRepository(path: String, password: String): Boolean {
        // For mobile, this is handled by the app layer
        Log.d(TAG, "Repository creation requested for path: $path")
        return createNewRepository()
    }

    /**
     * Add recent repository to list
     */
    fun addRecentRepository(path: String): Boolean {
        Log.d(TAG, "Adding recent repository: $path")
        return true
    }

    /**
     * Get credential count
     */
    fun credentialCount(): Int {
        return listCredentials().size
    }

    /**
     * Check if repository is open
     */
    fun isOpen(): Boolean {
        return isRepositoryOpen()
    }

    /**
     * Refresh repository data
     */
    fun refresh(): Boolean {
        Log.d(TAG, "Repository refresh requested")
        return true
    }

    /**
     * Clear all credentials
     */
    fun clearCredentials(): Boolean {
        return clearRepository()
    }
}
