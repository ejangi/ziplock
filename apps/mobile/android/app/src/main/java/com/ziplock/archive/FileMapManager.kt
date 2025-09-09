package com.ziplock.archive

import android.util.Log
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.nio.charset.StandardCharsets

/**
 * File Map Manager for JSON file map exchange between Android and Mobile FFI
 *
 * This class handles the conversion between:
 * - Archive files (7z format) ↔ File maps (Map<String, ByteArray>)
 * - File maps ↔ JSON strings for FFI exchange
 *
 * The file map format represents the internal structure of a ZipLock repository:
 * - "metadata.yml": Repository metadata
 * - "credentials/{uuid}/record.yml": Individual credential records
 * - "index.yml": Optional credential index
 *
 * This follows the unified architecture pattern where mobile FFI handles only
 * memory operations and Android handles all file I/O operations.
 */
object FileMapManager {

    private const val TAG = "FileMapManager"

    private val json = Json {
        ignoreUnknownKeys = true
        prettyPrint = false
        encodeDefaults = true
    }

    @Serializable
    data class RepositoryMetadata(
        val version: String = "1.0",
        val format: String = "memory-v1",
        val createdAt: Long,
        val lastModified: Long,
        val credentialCount: Int = 0,
        val structureVersion: String = "1.0",
        val generator: String = "ziplock-android"
    )

    /**
     * Convert file map to JSON string for FFI exchange
     *
     * @param fileMap Map of file paths to byte arrays
     * @return JSON string with base64-encoded file contents, or null on error
     */
    fun fileMapToJson(fileMap: Map<String, ByteArray>): String? {
        return try {
            // Convert byte arrays to base64 strings
            val base64Map = fileMap.mapValues { (path, bytes) ->
                try {
                    android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to encode file to base64: $path", e)
                    return null
                }
            }

            val jsonString = json.encodeToString(base64Map)
            Log.d(TAG, "Converted file map with ${fileMap.size} files to JSON (${jsonString.length} chars)")
            jsonString
        } catch (e: Exception) {
            Log.e(TAG, "Failed to convert file map to JSON", e)
            null
        }
    }

    /**
     * Convert JSON string from FFI to file map
     *
     * @param filesJson JSON string with base64-encoded file contents
     * @return Map of file paths to byte arrays, or null on error
     */
    fun jsonToFileMap(filesJson: String): Map<String, ByteArray>? {
        return try {
            // Parse JSON to base64 map
            val base64Map = json.decodeFromString<Map<String, String>>(filesJson)

            // Convert base64 strings back to byte arrays
            val fileMap = mutableMapOf<String, ByteArray>()

            for ((path, base64) in base64Map) {
                val bytes = try {
                    android.util.Base64.decode(base64, android.util.Base64.NO_WRAP)
                } catch (e: IllegalArgumentException) {
                    // If base64 decode fails, treat as UTF-8 text
                    Log.w(TAG, "Base64 decode failed for $path, treating as UTF-8 text")
                    base64.toByteArray(StandardCharsets.UTF_8)
                }
                fileMap[path] = bytes
            }

            Log.d(TAG, "Converted JSON to file map with ${fileMap.size} files")
            fileMap
        } catch (e: Exception) {
            Log.e(TAG, "Failed to convert JSON to file map", e)
            null
        }
    }

    /**
     * Create an empty repository file map with default metadata
     *
     * @return File map with initial metadata.yml
     */
    fun createEmptyRepository(): Map<String, ByteArray> {
        val metadata = RepositoryMetadata(
            createdAt = System.currentTimeMillis(),
            lastModified = System.currentTimeMillis(),
            credentialCount = 0
        )

        val metadataYaml = """
            version: "${metadata.version}"
            format: "${metadata.format}"
            created_at: ${metadata.createdAt}
            last_modified: ${metadata.lastModified}
            credential_count: ${metadata.credentialCount}
            structure_version: "${metadata.structureVersion}"
            generator: "${metadata.generator}"
        """.trimIndent()

        return mapOf(
            "metadata.yml" to metadataYaml.toByteArray(StandardCharsets.UTF_8)
        )
    }

    /**
     * Normalize metadata by adding missing required fields with defaults
     *
     * @param fileMap Original file map
     * @return File map with normalized metadata.yml
     */
    fun normalizeFileMap(fileMap: Map<String, ByteArray>): Map<String, ByteArray> {
        val normalizedMap = fileMap.toMutableMap()

        if (fileMap.containsKey("metadata.yml")) {
            val metadataContent = String(fileMap["metadata.yml"]!!, StandardCharsets.UTF_8)
            var normalizedContent = metadataContent

            // Add missing format field
            if (!metadataContent.contains("format:")) {
                normalizedContent += "\nformat: memory-v1"
                Log.d(TAG, "Added missing format field to metadata.yml")
            }

            // Add missing version field
            if (!metadataContent.contains("version:")) {
                normalizedContent = "version: 1.0\n$normalizedContent"
                Log.d(TAG, "Added missing version field to metadata.yml")
            }

            normalizedMap["metadata.yml"] = normalizedContent.toByteArray(StandardCharsets.UTF_8)
        }

        return normalizedMap
    }

    /**
     * Validate file map structure
     *
     * @param fileMap File map to validate
     * @return ValidationResult with success status and any issues found
     */
    fun validateFileMap(fileMap: Map<String, ByteArray>): ValidationResult {
        val issues = mutableListOf<String>()

        // Check for required metadata file
        if (!fileMap.containsKey("metadata.yml")) {
            issues.add("Missing required metadata.yml file")
        } else {
            // Validate metadata structure - be lenient about missing fields
            try {
                val metadataContent = String(fileMap["metadata.yml"]!!, StandardCharsets.UTF_8)
                if (!metadataContent.contains("version:")) {
                    Log.w(TAG, "metadata.yml missing version field - will use default")
                }
                if (!metadataContent.contains("format:")) {
                    Log.w(TAG, "metadata.yml missing format field - will use default")
                }
            } catch (e: Exception) {
                issues.add("Invalid metadata.yml content: ${e.message}")
            }
        }

        // Check credential file structure
        val credentialFiles = fileMap.keys.filter { it.startsWith("credentials/") && it.endsWith("/record.yml") }
        val credentialCount = credentialFiles.size

        // Validate each credential file has a valid UUID directory structure
        for (credPath in credentialFiles) {
            val pathParts = credPath.split("/")
            if (pathParts.size != 3 || pathParts[0] != "credentials" || pathParts[2] != "record.yml") {
                issues.add("Invalid credential file path: $credPath")
                continue
            }

            val uuid = pathParts[1]
            if (!isValidUuid(uuid)) {
                issues.add("Invalid UUID in credential path: $credPath")
            }

            // Check if file content is valid YAML
            try {
                val content = String(fileMap[credPath]!!, StandardCharsets.UTF_8)
                if (!content.contains("id:") || !content.contains("title:")) {
                    issues.add("Credential file missing required fields: $credPath")
                }
            } catch (e: Exception) {
                issues.add("Invalid credential file content: $credPath - ${e.message}")
            }
        }

        // Log extra files but don't treat them as validation errors
        for (path in fileMap.keys) {
            if (path != "metadata.yml" &&
                path != "index.yml" &&
                !path.endsWith(".ziplock_placeholder") &&
                !path.startsWith("credentials/") &&
                !path.startsWith("attachments/")) {
                Log.d(TAG, "Extra file found (ignoring): $path")
            }
        }

        Log.d(TAG, "File map validation: ${credentialFiles.size} credentials, ${issues.size} issues")

        return ValidationResult(
            isValid = issues.isEmpty(),
            credentialCount = credentialCount,
            issues = issues
        )
    }

    /**
     * Extract repository statistics from file map
     *
     * @param fileMap File map to analyze
     * @return RepositoryInfo with statistics
     */
    fun getRepositoryInfo(fileMap: Map<String, ByteArray>): RepositoryInfo {
        var credentialCount = 0
        var hasMetadata = false
        var hasIndex = false
        val fileTypes = mutableMapOf<String, Int>()

        for (path in fileMap.keys) {
            when {
                path == "metadata.yml" -> hasMetadata = true
                path == "index.yml" -> hasIndex = true
                path.startsWith("credentials/") && path.endsWith("/record.yml") -> {
                    credentialCount++
                    fileTypes["credentials"] = fileTypes.getOrDefault("credentials", 0) + 1
                }
                path.startsWith("attachments/") -> {
                    fileTypes["attachments"] = fileTypes.getOrDefault("attachments", 0) + 1
                }
                else -> {
                    fileTypes["other"] = fileTypes.getOrDefault("other", 0) + 1
                }
            }
        }

        return RepositoryInfo(
            totalFiles = fileMap.size,
            credentialCount = credentialCount,
            hasMetadata = hasMetadata,
            hasIndex = hasIndex,
            fileTypes = fileTypes,
            totalSize = fileMap.values.sumOf { it.size }
        )
    }

    /**
     * Merge two file maps, with the second taking precedence for conflicts
     *
     * @param base Base file map
     * @param overlay File map to overlay on top
     * @return Merged file map
     */
    fun mergeFileMaps(base: Map<String, ByteArray>, overlay: Map<String, ByteArray>): Map<String, ByteArray> {
        val merged = base.toMutableMap()
        merged.putAll(overlay)

        Log.d(TAG, "Merged file maps: ${base.size} + ${overlay.size} = ${merged.size} files")
        return merged
    }

    /**
     * Create a backup of a file map with timestamp
     *
     * @param fileMap Original file map
     * @return File map with backup metadata
     */
    fun createBackup(fileMap: Map<String, ByteArray>): Map<String, ByteArray> {
        val backup = fileMap.toMutableMap()
        val timestamp = System.currentTimeMillis()

        val backupInfo = """
            backup_timestamp: $timestamp
            backup_date: "${java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss").format(java.util.Date(timestamp))}"
            original_file_count: ${fileMap.size}
            backup_generator: "ziplock-android"
        """.trimIndent()

        backup["backup_info.yml"] = backupInfo.toByteArray(StandardCharsets.UTF_8)

        Log.d(TAG, "Created backup with ${backup.size} files (timestamp: $timestamp)")
        return backup
    }

    /**
     * Simple UUID validation
     */
    private fun isValidUuid(uuid: String): Boolean {
        val uuidRegex = Regex("^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        return uuidRegex.matches(uuid)
    }

    /**
     * Result of file map validation
     */
    data class ValidationResult(
        val isValid: Boolean,
        val credentialCount: Int,
        val issues: List<String>
    )

    /**
     * Information about a repository file map
     */
    data class RepositoryInfo(
        val totalFiles: Int,
        val credentialCount: Int,
        val hasMetadata: Boolean,
        val hasIndex: Boolean,
        val fileTypes: Map<String, Int>,
        val totalSize: Int
    )
}
