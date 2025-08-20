package com.ziplock.ffi

import android.util.Log
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import kotlinx.serialization.decodeFromString
import java.util.*

/**
 * Kotlin wrapper for the ZipLock memory repository FFI functions.
 * This provides a high-level interface to the centralized file structure
 * management implemented in the shared Rust library.
 */
class ZipLockMemoryRepository {

    companion object {
        private const val TAG = "ZipLockMemoryRepository"

        init {
            try {
                System.loadLibrary("ziplock_shared")
                Log.d(TAG, "Native library loaded successfully")
            } catch (e: UnsatisfiedLinkError) {
                Log.e(TAG, "Failed to load native library", e)
                throw RuntimeException("Failed to load ZipLock native library", e)
            }
        }

        // Native function declarations
        @JvmStatic external fun ziplock_hybrid_init(): Int
        @JvmStatic external fun ziplock_hybrid_get_version(): String?
        @JvmStatic external fun ziplock_hybrid_get_last_error(): String?
        @JvmStatic external fun ziplock_hybrid_cleanup(): Int

        // Memory repository specific functions
        @JvmStatic external fun ziplock_hybrid_repository_load_content(filesJson: String): Int
        @JvmStatic external fun ziplock_hybrid_repository_get_file_operations(): String?
        @JvmStatic external fun ziplock_hybrid_repository_add_credential(credentialJson: String): String?
        @JvmStatic external fun ziplock_hybrid_repository_get_credential(credentialId: String): String?
        @JvmStatic external fun ziplock_hybrid_repository_update_credential(credentialJson: String): Int
        @JvmStatic external fun ziplock_hybrid_repository_delete_credential(credentialId: String): Int
        @JvmStatic external fun ziplock_hybrid_repository_list_credentials(): String?
        @JvmStatic external fun ziplock_hybrid_repository_search_credentials(query: String): String?
        @JvmStatic external fun ziplock_hybrid_repository_get_metadata(): String?
        @JvmStatic external fun ziplock_hybrid_repository_get_structure(): String?

        // String memory management
        @JvmStatic external fun ziplock_hybrid_string_free(ptr: String)
    }

    @Serializable
    data class FileOperation(
        val operation: String, // "create", "update", "delete"
        val path: String,
        val content: ByteArray? = null,
        val isDirectory: Boolean,
        val metadata: Map<String, String> = emptyMap()
    ) {
        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as FileOperation

            if (operation != other.operation) return false
            if (path != other.path) return false
            if (content != null) {
                if (other.content == null) return false
                if (!content.contentEquals(other.content)) return false
            } else if (other.content != null) return false
            if (isDirectory != other.isDirectory) return false
            if (metadata != other.metadata) return false

            return true
        }

        override fun hashCode(): Int {
            var result = operation.hashCode()
            result = 31 * result + path.hashCode()
            result = 31 * result + (content?.contentHashCode() ?: 0)
            result = 31 * result + isDirectory.hashCode()
            result = 31 * result + metadata.hashCode()
            return result
        }
    }

    @Serializable
    data class RepositoryMetadata(
        val version: String,
        val format: String,
        val createdAt: Long,
        val lastModified: Long,
        val credentialCount: Int,
        val structureVersion: String,
        val generator: String
    )

    @Serializable
    data class RepositoryFileInfo(
        val path: String,
        val size: Long,
        val isDirectory: Boolean,
        val modified: Long,
        val permissions: String? = null,
        val contentHash: String? = null
    )

    @Serializable
    data class RepositoryStructure(
        val version: String,
        val files: List<RepositoryFileInfo>,
        val metadata: Map<String, String>,
        val createdAt: Long,
        val modifiedAt: Long
    )

    @Serializable
    data class SerializedField(
        val value: String,
        val fieldType: String,
        val sensitive: Boolean,
        val label: String? = null,
        val placeholder: String? = null,
        val validation: String? = null
    )

    @Serializable
    data class SerializedCredential(
        val id: String,
        val title: String,
        val credentialType: String,
        val fields: Map<String, SerializedField>,
        val tags: List<String> = emptyList(),
        val notes: String? = null,
        val createdAt: Long,
        val updatedAt: Long
    )

    data class RepositoryResult<T>(
        val success: Boolean,
        val data: T? = null,
        val errorMessage: String? = null
    )

    private val json = Json {
        prettyPrint = false
        ignoreUnknownKeys = true
        encodeDefaults = true
    }

    private var isInitialized = false

    /**
     * Initialize the memory repository
     */
    fun initialize(): RepositoryResult<Boolean> {
        if (isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository already initialized"
            )
        }

        val result = ziplock_hybrid_init()
        if (result != 0) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error during initialization"
            Log.e(TAG, "Failed to initialize repository: $error")
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        isInitialized = true
        Log.i(TAG, "Memory repository initialized successfully")
        return RepositoryResult(success = true, data = true)
    }

    /**
     * Get library version
     */
    fun getVersion(): String {
        return ziplock_hybrid_get_version() ?: "unknown"
    }

    /**
     * Load repository content from extracted archive files
     */
    fun loadContent(files: Map<String, ByteArray>): RepositoryResult<Boolean> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        try {
            // Convert byte arrays to base64 for transport to native code
            val filesBase64 = files.mapValues { (_, bytes) ->
                Base64.getEncoder().encodeToString(bytes)
            }

            val filesJson = json.encodeToString(filesBase64)
            val result = ziplock_hybrid_repository_load_content(filesJson)

            if (result != 0) {
                val error = ziplock_hybrid_get_last_error() ?: "Unknown error loading content"
                return RepositoryResult(
                    success = false,
                    errorMessage = error
                )
            }

            return RepositoryResult(success = true, data = true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to load repository content", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to serialize files for loading: ${e.message}"
            )
        }
    }

    /**
     * Get file operations needed to persist the repository
     */
    fun getFileOperations(): RepositoryResult<List<FileOperation>> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val operationsJson = ziplock_hybrid_repository_get_file_operations()
        if (operationsJson == null) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error getting file operations"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        try {
            val operations: List<FileOperation> = json.decodeFromString(operationsJson)
            return RepositoryResult(success = true, data = operations)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse file operations", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to parse file operations: ${e.message}"
            )
        }
    }

    /**
     * Add a credential to the repository
     */
    fun addCredential(credential: SerializedCredential): RepositoryResult<String> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        try {
            // Convert to YAML format for the native library
            val credentialYaml = convertCredentialToYaml(credential)
            val credentialId = ziplock_hybrid_repository_add_credential(credentialYaml)

            if (credentialId == null) {
                val error = ziplock_hybrid_get_last_error() ?: "Unknown error adding credential"
                return RepositoryResult(
                    success = false,
                    errorMessage = error
                )
            }

            return RepositoryResult(success = true, data = credentialId)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to add credential", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to serialize credential: ${e.message}"
            )
        }
    }

    /**
     * Get a credential from the repository
     */
    fun getCredential(credentialId: String): RepositoryResult<SerializedCredential> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val credentialJson = ziplock_hybrid_repository_get_credential(credentialId)
        if (credentialJson == null) {
            val error = ziplock_hybrid_get_last_error() ?: "Credential not found"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        try {
            val credential: SerializedCredential = convertCredentialFromYaml(credentialJson)
            return RepositoryResult(success = true, data = credential)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse credential", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to parse credential: ${e.message}"
            )
        }
    }

    /**
     * Update a credential in the repository
     */
    fun updateCredential(credential: SerializedCredential): RepositoryResult<Boolean> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        try {
            val credentialYaml = convertCredentialToYaml(credential)
            val result = ziplock_hybrid_repository_update_credential(credentialYaml)

            if (result != 0) {
                val error = ziplock_hybrid_get_last_error() ?: "Unknown error updating credential"
                return RepositoryResult(
                    success = false,
                    errorMessage = error
                )
            }

            return RepositoryResult(success = true, data = true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to update credential", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to serialize credential: ${e.message}"
            )
        }
    }

    /**
     * Delete a credential from the repository
     */
    fun deleteCredential(credentialId: String): RepositoryResult<Boolean> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val result = ziplock_hybrid_repository_delete_credential(credentialId)
        if (result != 0) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error deleting credential"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        return RepositoryResult(success = true, data = true)
    }

    /**
     * List all credentials in the repository
     */
    fun listCredentials(): RepositoryResult<List<SerializedCredential>> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val credentialsJson = ziplock_hybrid_repository_list_credentials()
        if (credentialsJson == null) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error listing credentials"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        try {
            val credentials: List<SerializedCredential> = convertCredentialsListFromYaml(credentialsJson)
            return RepositoryResult(success = true, data = credentials)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse credentials list", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to parse credentials: ${e.message}"
            )
        }
    }

    /**
     * Search credentials in the repository
     */
    fun searchCredentials(query: String): RepositoryResult<List<SerializedCredential>> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val credentialsJson = ziplock_hybrid_repository_search_credentials(query)
        if (credentialsJson == null) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error searching credentials"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        try {
            val credentials: List<SerializedCredential> = convertCredentialsListFromYaml(credentialsJson)
            return RepositoryResult(success = true, data = credentials)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse search results", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to parse search results: ${e.message}"
            )
        }
    }

    /**
     * Get repository metadata
     */
    fun getMetadata(): RepositoryResult<RepositoryMetadata> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val metadataJson = ziplock_hybrid_repository_get_metadata()
        if (metadataJson == null) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error getting metadata"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        try {
            val metadata: RepositoryMetadata = convertMetadataFromYaml(metadataJson)
            return RepositoryResult(success = true, data = metadata)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse metadata", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to parse metadata: ${e.message}"
            )
        }
    }

    /**
     * Get repository structure
     */
    fun getStructure(): RepositoryResult<RepositoryStructure> {
        if (!isInitialized) {
            return RepositoryResult(
                success = false,
                errorMessage = "Repository not initialized"
            )
        }

        val structureJson = ziplock_hybrid_repository_get_structure()
        if (structureJson == null) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error getting structure"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        try {
            val structure: RepositoryStructure = convertStructureFromYaml(structureJson)
            return RepositoryResult(success = true, data = structure)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse structure", e)
            return RepositoryResult(
                success = false,
                errorMessage = "Failed to parse structure: ${e.message}"
            )
        }
    }

    /**
     * Get the last error message
     */
    fun getLastError(): String? {
        return ziplock_hybrid_get_last_error()
    }

    /**
     * Clean up the repository
     */
    fun cleanup(): RepositoryResult<Boolean> {
        if (!isInitialized) {
            return RepositoryResult(success = true, data = true)
        }

        val result = ziplock_hybrid_cleanup()
        isInitialized = false

        if (result != 0) {
            val error = ziplock_hybrid_get_last_error() ?: "Unknown error during cleanup"
            return RepositoryResult(
                success = false,
                errorMessage = error
            )
        }

        return RepositoryResult(success = true, data = true)
    }

    /**
     * Check if repository is initialized
     */
    fun isInitialized(): Boolean {
        return isInitialized
    }

    /**
     * Helper functions for YAML conversion
     * Since the native library expects YAML format for credentials but JSON for other data structures
     */
    private fun convertCredentialToYaml(credential: SerializedCredential): String {
        // Convert SerializedCredential to a format that matches Rust CredentialRecord
        val rustCredential = mapOf(
            "id" to credential.id,
            "title" to credential.title,
            "credential_type" to credential.credentialType,
            "fields" to credential.fields.mapValues { (_, field) ->
                mapOf(
                    "field_type" to field.fieldType,
                    "value" to field.value,
                    "sensitive" to field.sensitive,
                    "label" to field.label,
                    "metadata" to emptyMap<String, String>()
                )
            },
            "tags" to credential.tags,
            "notes" to credential.notes,
            "created_at" to mapOf(
                "secs_since_epoch" to credential.createdAt,
                "nanos_since_epoch" to 0
            ),
            "updated_at" to mapOf(
                "secs_since_epoch" to credential.updatedAt,
                "nanos_since_epoch" to 0
            )
        )

        // For now, we'll use JSON serialization since we don't have a YAML library
        // The native code will handle the actual YAML parsing
        return json.encodeToString(rustCredential)
    }

    private fun convertCredentialFromYaml(yamlString: String): SerializedCredential {
        // Parse the YAML response from native code
        // For now, assuming it's in JSON format since we don't have a YAML parser
        val rustCredential: Map<String, Any> = json.decodeFromString(yamlString)

        val fieldsMap = rustCredential["fields"] as? Map<String, Map<String, Any>> ?: emptyMap()
        val fields = fieldsMap.mapValues { (_, fieldData) ->
            SerializedField(
                value = fieldData["value"] as? String ?: "",
                fieldType = fieldData["field_type"] as? String ?: "Text",
                sensitive = fieldData["sensitive"] as? Boolean ?: false,
                label = fieldData["label"] as? String,
                placeholder = null,
                validation = null
            )
        }

        val createdAtMap = rustCredential["created_at"] as? Map<String, Any>
        val updatedAtMap = rustCredential["updated_at"] as? Map<String, Any>

        return SerializedCredential(
            id = rustCredential["id"] as? String ?: "",
            title = rustCredential["title"] as? String ?: "",
            credentialType = rustCredential["credential_type"] as? String ?: "",
            fields = fields,
            tags = (rustCredential["tags"] as? List<*>)?.filterIsInstance<String>() ?: emptyList(),
            notes = rustCredential["notes"] as? String,
            createdAt = (createdAtMap?.get("secs_since_epoch") as? Number)?.toLong() ?: 0L,
            updatedAt = (updatedAtMap?.get("secs_since_epoch") as? Number)?.toLong() ?: 0L
        )
    }

    private fun convertCredentialsListFromYaml(yamlString: String): List<SerializedCredential> {
        // Parse list of credentials from YAML
        val credentialsList: List<*> = json.decodeFromString(yamlString)
        return credentialsList.filterIsInstance<Map<String, Any>>().map { credentialMap ->
            convertCredentialFromYaml(json.encodeToString(credentialMap))
        }
    }

    private fun convertMetadataFromYaml(yamlString: String): RepositoryMetadata {
        // Parse metadata from YAML
        val metadataMap: Map<String, Any> = json.decodeFromString(yamlString)
        return RepositoryMetadata(
            version = metadataMap["version"] as? String ?: "1.0",
            format = metadataMap["format"] as? String ?: "memory-v1",
            createdAt = (metadataMap["created_at"] as? Number)?.toLong() ?: 0L,
            lastModified = (metadataMap["last_modified"] as? Number)?.toLong() ?: 0L,
            credentialCount = (metadataMap["credential_count"] as? Number)?.toInt() ?: 0,
            structureVersion = metadataMap["structure_version"] as? String ?: "1.0",
            generator = metadataMap["generator"] as? String ?: "ziplock-shared"
        )
    }

    private fun convertStructureFromYaml(yamlString: String): RepositoryStructure {
        // Parse repository structure from YAML
        val structureMap: Map<String, Any> = json.decodeFromString(yamlString)

        val filesList = structureMap["files"] as? List<*> ?: emptyList<Any>()
        val files = filesList.filterIsInstance<Map<String, Any>>().map { fileMap ->

            RepositoryFileInfo(
                path = fileMap["path"] as? String ?: "",
                size = (fileMap["size"] as? Number)?.toLong() ?: 0L,
                isDirectory = fileMap["is_directory"] as? Boolean ?: false,
                modified = (fileMap["modified"] as? Number)?.toLong() ?: 0L,
                permissions = fileMap["permissions"] as? String,
                contentHash = fileMap["content_hash"] as? String
            )
        }

        val metadata = structureMap["metadata"] as? Map<*, *> ?: emptyMap<String, String>()
        val metadataStrings = metadata.entries.associate { (k, v) ->
            k.toString() to v.toString()
        }

        return RepositoryStructure(
            version = structureMap["version"] as? String ?: "1.0",
            files = files,
            metadata = metadataStrings,
            createdAt = (structureMap["created_at"] as? Number)?.toLong() ?: 0L,
            modifiedAt = (structureMap["modified_at"] as? Number)?.toLong() ?: 0L
        )
    }
}
