package com.ziplock.archive

import android.content.Context
import android.net.Uri
import android.util.Log
import org.apache.commons.compress.archivers.sevenz.SevenZArchiveEntry
import org.apache.commons.compress.archivers.sevenz.SevenZFile
import org.apache.commons.compress.archivers.sevenz.SevenZOutputFile
import org.apache.commons.compress.archivers.sevenz.SevenZMethod
import org.apache.commons.compress.archivers.sevenz.SevenZMethodConfiguration
import org.apache.commons.compress.utils.SeekableInMemoryByteChannel
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.nio.charset.StandardCharsets
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.spec.IvParameterSpec
import javax.crypto.spec.SecretKeySpec
import javax.crypto.spec.PBEKeySpec
import javax.crypto.SecretKeyFactory
import java.security.MessageDigest

/**
 * Native Archive Manager for 7z operations on Android
 *
 * This class handles all archive file I/O operations using Apache Commons Compress.
 * It provides the file operations layer for the unified architecture, where:
 * - Android handles all archive operations (this class)
 * - Mobile FFI handles only memory operations
 *
 * Key responsibilities:
 * - Extract 7z archives to file maps
 * - Create 7z archives from file maps
 * - Handle password-based encryption (AES-256)
 * - Integrate with Storage Access Framework (SAF)
 * - Provide secure temporary file handling
 */
class NativeArchiveManager(private val context: Context) {

    companion object {
        private const val TAG = "NativeArchiveManager"

        // Encryption constants
        private const val AES_KEY_LENGTH = 256
        private const val AES_IV_LENGTH = 16
        private const val PBKDF2_ITERATIONS = 100000
        private const val SALT_LENGTH = 32

        // 7z compression settings
        private val DEFAULT_COMPRESSION_METHOD = SevenZMethod.LZMA2
        private const val DEFAULT_COMPRESSION_LEVEL = 5

        // File size limits (for safety)
        private const val MAX_ARCHIVE_SIZE = 500 * 1024 * 1024 // 500MB
        private const val MAX_EXTRACTED_FILE_SIZE = 50L * 1024 * 1024 // 50MB per file
        private const val MAX_FILES_IN_ARCHIVE = 10000
    }

    /**
     * Result of archive extraction operation
     */
    data class ExtractionResult(
        val success: Boolean,
        val fileMap: Map<String, ByteArray>? = null,
        val error: String? = null,
        val extractedFileCount: Int = 0,
        val totalSizeBytes: Long = 0
    )

    /**
     * Result of archive creation operation
     */
    data class CreationResult(
        val success: Boolean,
        val archiveData: ByteArray? = null,
        val error: String? = null,
        val compressedSizeBytes: Long = 0,
        val compressionRatio: Float = 0f,
        val isEncrypted: Boolean = false
    )

    /**
     * Configuration for archive operations
     */
    data class ArchiveConfig(
        val compressionMethod: SevenZMethod = DEFAULT_COMPRESSION_METHOD,
        val compressionLevel: Int = DEFAULT_COMPRESSION_LEVEL,
        val enableEncryption: Boolean = true,
        val validateExtractedFiles: Boolean = true,
        val maxFileSize: Long = MAX_EXTRACTED_FILE_SIZE,
        val maxFiles: Int = MAX_FILES_IN_ARCHIVE
    )

    /**
     * Extract a 7z archive from URI to file map
     *
     * @param archiveUri URI of the 7z archive file
     * @param password Password for encrypted archive
     * @param config Configuration for extraction
     * @return ExtractionResult with file map or error
     */
    fun extractArchive(
        archiveUri: Uri,
        password: String,
        config: ArchiveConfig = ArchiveConfig()
    ): ExtractionResult {
        Log.d(TAG, "Starting extraction from URI: $archiveUri")

        return try {
            val archiveData = readArchiveFromUri(archiveUri)
                ?: return ExtractionResult(false, error = "Failed to read archive from URI")

            extractArchiveFromBytes(archiveData, password, config)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to extract archive", e)
            ExtractionResult(false, error = "Extraction failed: ${e.message}")
        }
    }

    /**
     * Extract a 7z archive from byte array to file map
     *
     * @param archiveData Byte array containing 7z archive
     * @param password Password for encrypted archive
     * @param config Configuration for extraction
     * @return ExtractionResult with file map or error
     */
    fun extractArchiveFromBytes(
        archiveData: ByteArray,
        password: String,
        config: ArchiveConfig = ArchiveConfig()
    ): ExtractionResult {
        if (archiveData.size > MAX_ARCHIVE_SIZE) {
            return ExtractionResult(false, error = "Archive too large: ${archiveData.size} bytes")
        }

        Log.d(TAG, "Extracting archive from ${archiveData.size} bytes")
        Log.d(TAG, "EXTRACTION DEBUG: Password provided: ${password.isNotEmpty()}")
        Log.d(TAG, "EXTRACTION DEBUG: Password length: ${password.length}")

        return try {
            // Create temporary file for SevenZFile (it requires file access)
            val tempFile = createTempFile("ziplock_extract", ".7z")
            tempFile.writeBytes(archiveData)

            val fileMap = mutableMapOf<String, ByteArray>()
            var extractedFiles = 0
            var totalSize = 0L

            Log.d(TAG, "Extracting archive from temp file: ${tempFile.absolutePath}")

            // Test if archive is encrypted by trying without password first
            val isEncrypted = try {
                if (password.isNotEmpty()) {
                    // Try opening without password to test encryption
                    SevenZFile(tempFile).use { testFile ->
                        val testEntry = testFile.nextEntry
                        if (testEntry != null) {
                            // Try to read content - this should fail for encrypted archives
                            val buffer = ByteArray(minOf(100, testEntry.size.toInt()))
                            testFile.read(buffer)
                            Log.w(TAG, "⚠️ WARNING: Archive opened successfully without password - may not be encrypted!")
                            false // Not encrypted
                        } else {
                            Log.d(TAG, "ℹ️ Archive has no entries (empty)")
                            false
                        }
                    }
                } else {
                    false // No password provided, assume unencrypted
                }
            } catch (e: Exception) {
                Log.d(TAG, "✅ Archive requires password (failed to open without password): ${e.message}")
                true // Encrypted
            }

            Log.d(TAG, "EXTRACTION DEBUG: Archive appears encrypted: $isEncrypted")

            SevenZFile(tempFile, password.toCharArray()).use { sevenZFile ->
                Log.d(TAG, "✅ Archive opened successfully with provided password")

            var entry: SevenZArchiveEntry? = sevenZFile.nextEntry
            while (entry != null) {
                val currentEntry = entry

                // Safety checks
                if (extractedFiles >= config.maxFiles) {
                    Log.w(TAG, "Too many files in archive, stopping at ${config.maxFiles}")
                    break
                }

                if (currentEntry.size > config.maxFileSize) {
                    Log.w(TAG, "Skipping large file: ${currentEntry.name} (${currentEntry.size} bytes)")
                    entry = sevenZFile.nextEntry
                    continue
                }

                if (currentEntry.isDirectory) {
                    Log.d(TAG, "Skipping directory: ${currentEntry.name}")
                    entry = sevenZFile.nextEntry
                    continue
                }

                // Extract file content
                val content = ByteArray(currentEntry.size.toInt())
                val bytesRead = sevenZFile.read(content)

                if (bytesRead != currentEntry.size.toInt()) {
                    Log.w(TAG, "Incomplete read for ${currentEntry.name}: $bytesRead/${currentEntry.size}")
                    entry = sevenZFile.nextEntry
                    continue
                }

                // Validate file content if enabled
                if (config.validateExtractedFiles) {
                    if (!isValidRepositoryFile(currentEntry.name, content)) {
                        Log.w(TAG, "Skipping invalid file: ${currentEntry.name}")
                        entry = sevenZFile.nextEntry
                        continue
                    }
                } else {
                    // Even with validation disabled, reject obviously invalid files
                    val filename = currentEntry.name.lowercase()
                    if (filename.endsWith(".json") || filename.endsWith(".md") ||
                        filename.endsWith(".txt") || filename.endsWith(".readme")) {
                        Log.w(TAG, "Rejecting non-repository file: ${currentEntry.name}")
                        entry = sevenZFile.nextEntry
                        continue
                    }
                }

                fileMap[currentEntry.name] = content
                extractedFiles++
                totalSize += content.size

                Log.d(TAG, "Extracted: ${currentEntry.name} (${content.size} bytes)")

                // Get next entry
                entry = sevenZFile.nextEntry
            }
        }

            // Clean up temp file
            tempFile.delete()

            Log.d(TAG, "Extraction completed: $extractedFiles files, $totalSize bytes")

            ExtractionResult(
                success = true,
                fileMap = fileMap,
                extractedFileCount = extractedFiles,
                totalSizeBytes = totalSize
            )

        } catch (e: Exception) {
            Log.e(TAG, "Archive extraction failed", e)
            ExtractionResult(false, error = "Extraction failed: ${e.message}")
        }
    }

    /**
     * Create a 7z archive from file map
     *
     * @param fileMap Map of file paths to byte arrays
     * @param password Password for archive encryption
     * @param config Configuration for creation
     * @return CreationResult with archive data or error
     */
    fun createArchive(
        fileMap: Map<String, ByteArray>,
        password: String,
        config: ArchiveConfig = ArchiveConfig()
    ): CreationResult {
        if (fileMap.size > config.maxFiles) {
            return CreationResult(false, error = "Too many files: ${fileMap.size}")
        }

        val originalSize = fileMap.values.sumOf { it.size.toLong() }
        Log.d(TAG, "Creating archive from ${fileMap.size} files (${originalSize} bytes)")
        Log.d(TAG, "Encryption enabled: ${config.enableEncryption}, password provided: ${password.isNotEmpty()}")

        // CRITICAL DEBUG: Log encryption details for debugging
        Log.d(TAG, "ENCRYPTION DEBUG: config.enableEncryption = ${config.enableEncryption}")
        Log.d(TAG, "ENCRYPTION DEBUG: password.isNotEmpty() = ${password.isNotEmpty()}")
        Log.d(TAG, "ENCRYPTION DEBUG: password.length = ${password.length}")
        val willBeEncrypted = config.enableEncryption && password.isNotEmpty()
        Log.d(TAG, "ENCRYPTION DEBUG: Archive will be encrypted: $willBeEncrypted")
        if (!willBeEncrypted) {
            Log.w(TAG, "🚨 WARNING: Archive will be UNENCRYPTED!")
        }

        return try {
            // Create temporary file for SevenZOutputFile
            val tempFile = createTempFile("ziplock_create", ".7z")

            // Create SevenZOutputFile with or without password
            val sevenZOutput = if (config.enableEncryption && password.isNotEmpty()) {
                Log.d(TAG, "✅ Creating encrypted 7z archive with password")
                Log.d(TAG, "ENCRYPTION DEBUG: Password chars: ${password.toCharArray().size}")
                Log.d(TAG, "ENCRYPTION DEBUG: Temp file: ${tempFile.absolutePath}")
                try {
                    val encryptedOutput = SevenZOutputFile(tempFile, password.toCharArray())
                    Log.d(TAG, "✅ SevenZOutputFile created successfully with password")
                    encryptedOutput
                } catch (e: Exception) {
                    Log.e(TAG, "❌ CRITICAL: Failed to create encrypted SevenZOutputFile", e)
                    throw e
                }
            } else {
                Log.w(TAG, "⚠️ Creating unencrypted 7z archive (enableEncryption=${config.enableEncryption}, hasPassword=${password.isNotEmpty()})")
                Log.w(TAG, "🚨 SECURITY WARNING: Archive will be stored WITHOUT encryption!")
                try {
                    val unencryptedOutput = SevenZOutputFile(tempFile)
                    Log.d(TAG, "⚠️ SevenZOutputFile created successfully WITHOUT password")
                    unencryptedOutput
                } catch (e: Exception) {
                    Log.e(TAG, "❌ Failed to create unencrypted SevenZOutputFile", e)
                    throw e
                }
            }

            sevenZOutput.use { output ->
                Log.d(TAG, "ENCRYPTION DEBUG: Setting up archive content...")

                // Set compression method
                output.setContentMethods(listOf(SevenZMethodConfiguration(config.compressionMethod)))
                Log.d(TAG, "ENCRYPTION DEBUG: Compression method set to: ${config.compressionMethod}")

                // Add files to archive
                var filesAdded = 0
                for ((path, content) in fileMap) {
                    if (content.size > config.maxFileSize) {
                        Log.w(TAG, "Skipping large file: $path (${content.size} bytes)")
                        continue
                    }

                    Log.d(TAG, "ENCRYPTION DEBUG: Adding file to archive: $path")
                    Log.d(TAG, "ENCRYPTION DEBUG: File content preview: ${String(content.take(50).toByteArray()).take(20)}...")

                    val entry = output.createArchiveEntry(File(path), path)
                    entry?.size = content.size.toLong()

                    output.putArchiveEntry(entry)
                    output.write(content)
                    output.closeArchiveEntry()
                    filesAdded++

                    Log.d(TAG, "✅ Added to archive: $path (${content.size} bytes)")
                }

                Log.d(TAG, "ENCRYPTION DEBUG: Total files added to archive: $filesAdded")
            }

            // Read the created archive
            val archiveData = tempFile.readBytes()
            val compressedSize = archiveData.size.toLong()
            val compressionRatio = if (originalSize > 0) {
                (compressedSize.toFloat() / originalSize.toFloat())
            } else 1.0f

            // CRITICAL DEBUG: Test if archive is actually encrypted
            Log.d(TAG, "ENCRYPTION DEBUG: Archive data size: ${archiveData.size} bytes")
            Log.d(TAG, "ENCRYPTION DEBUG: Testing archive encryption...")

            val archiveString = String(archiveData.take(500).toByteArray(), Charsets.ISO_8859_1)
            var foundPlaintextContent = false

            for ((path, content) in fileMap) {
                val contentString = String(content, Charsets.UTF_8)
                val lines = contentString.lines().filter { it.trim().length > 5 }
                for (line in lines.take(3)) {
                    val cleanLine = line.trim()
                    if (cleanLine.isNotEmpty() && archiveString.contains(cleanLine)) {
                        Log.e(TAG, "🚨 CRITICAL SECURITY FAILURE: Found plaintext content in archive!")
                        Log.e(TAG, "  Content found: '$cleanLine' from file: $path")
                        foundPlaintextContent = true
                        break
                    }
                }
                if (foundPlaintextContent) break
            }

            if (!foundPlaintextContent && willBeEncrypted) {
                Log.d(TAG, "✅ Archive content appears encrypted (no plaintext found)")
            } else if (!willBeEncrypted) {
                Log.d(TAG, "ℹ️ Archive is intentionally unencrypted")
            }

            // Clean up temp file
            tempFile.delete()

            Log.d(TAG, "Archive created: ${archiveData.size} bytes (ratio: ${String.format("%.2f", compressionRatio)})")
            Log.d(TAG, "ENCRYPTION DEBUG: Final encryption status - willBeEncrypted: $willBeEncrypted, foundPlaintext: $foundPlaintextContent")

            // CRITICAL: Update isEncrypted based on actual analysis
            val actuallyEncrypted = willBeEncrypted && !foundPlaintextContent

            if (willBeEncrypted && foundPlaintextContent) {
                Log.e(TAG, "🚨 ENCRYPTION FAILURE: Archive was supposed to be encrypted but contains plaintext!")
                return CreationResult(false, error = "Encryption failed - archive contains plaintext content")
            }

            Log.d(TAG, "ENCRYPTION DEBUG: Final result - actuallyEncrypted: $actuallyEncrypted")

            CreationResult(
                success = true,
                archiveData = archiveData,
                compressedSizeBytes = compressedSize,
                compressionRatio = compressionRatio,
                isEncrypted = actuallyEncrypted
            )

        } catch (e: Exception) {
            Log.e(TAG, "Archive creation failed", e)
            CreationResult(false, error = "Creation failed: ${e.message}")
        }
    }

    /**
     * Save archive data to URI using Storage Access Framework
     *
     * @param archiveData Byte array containing archive
     * @param destinationUri URI where to save the archive
     * @return true if successful, false otherwise
     */
    fun saveArchiveToUri(archiveData: ByteArray, destinationUri: Uri): Boolean {
        Log.d(TAG, "Saving ${archiveData.size} bytes to URI: $destinationUri")

        return try {
            context.contentResolver.openOutputStream(destinationUri)?.use { outputStream ->
                outputStream.write(archiveData)
                outputStream.flush()
                true
            } ?: false
        } catch (e: Exception) {
            Log.e(TAG, "Failed to save archive to URI", e)
            false
        }
    }

    /**
     * Read archive data from URI using Storage Access Framework
     *
     * @param archiveUri URI of the archive file
     * @return Byte array containing archive data, or null on error
     */
    private fun readArchiveFromUri(archiveUri: Uri): ByteArray? {
        Log.d(TAG, "Reading archive from URI: $archiveUri")

        return try {
            context.contentResolver.openInputStream(archiveUri)?.use { inputStream ->
                val buffer = ByteArrayOutputStream()
                val data = ByteArray(8192)
                var bytesRead: Int

                while (inputStream.read(data).also { bytesRead = it } != -1) {
                    buffer.write(data, 0, bytesRead)

                    // Safety check to prevent memory exhaustion
                    if (buffer.size() > MAX_ARCHIVE_SIZE) {
                        Log.e(TAG, "Archive too large during read: ${buffer.size()} bytes")
                        return null
                    }
                }

                val result = buffer.toByteArray()
                Log.d(TAG, "Read ${result.size} bytes from archive")
                result
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to read archive from URI", e)
            null
        }
    }

    /**
     * Validate if a file is a valid repository file
     *
     * @param filename Name of the file
     * @param content File content as byte array
     * @return true if the file appears to be valid
     */
    private fun isValidRepositoryFile(filename: String, content: ByteArray): Boolean {
        // Basic file name validation
        if (filename.contains("..") || filename.startsWith("/") || filename.contains("\\")) {
            Log.w(TAG, "Invalid file name: $filename")
            return false
        }

        // Check file size - allow empty placeholder files
        if (content.size > MAX_EXTRACTED_FILE_SIZE) {
            Log.w(TAG, "File too large: $filename (${content.size} bytes)")
            return false
        }

        // Allow empty content only for placeholder files
        if (content.isEmpty() && !filename.endsWith(".ziplock_placeholder")) {
            Log.w(TAG, "Invalid file size: $filename (${content.size} bytes)")
            return false
        }

        // Validate specific file types
        when {
            filename.endsWith(".yml") || filename.endsWith(".yaml") -> {
                return isValidYamlContent(content)
            }
            filename == "metadata.yml" -> {
                return isValidMetadataFile(content)
            }
            filename.startsWith("credentials/") && filename.endsWith("/record.yml") -> {
                return isValidCredentialFile(content)
            }
            filename == "index.yml" -> {
                return true // Index file is optional, any YAML content is acceptable
            }
            filename.endsWith(".ziplock_placeholder") -> {
                return true // Placeholder files for empty directories
            }
            else -> {
                Log.w(TAG, "Unknown file type: $filename")
                return false
            }
        }
    }

    /**
     * Validate YAML content
     */
    private fun isValidYamlContent(content: ByteArray): Boolean {
        return try {
            val text = String(content, StandardCharsets.UTF_8)
            // Basic YAML validation - check for YAML-like structure
            text.isNotBlank() &&
            (text.contains(":") || text.contains("---")) &&
            !text.contains("\u0000") // No null bytes
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Validate metadata file
     */

    private fun isValidMetadataFile(content: ByteArray): Boolean {
        return try {
            val text = String(content, StandardCharsets.UTF_8)
            text.contains("version:") &&
            text.contains("format:") &&
            text.contains("created_at:")
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Validate credential file
     */

    private fun isValidCredentialFile(content: ByteArray): Boolean {
        return try {
            val text = String(content, StandardCharsets.UTF_8)
            text.contains("id:") &&
            text.contains("title:") &&
            text.contains("credential_type:")
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Create a secure temporary file
     */
    private fun createTempFile(prefix: String, suffix: String): File {
        val tempDir = File(context.cacheDir, "ziplock_temp")
        if (!tempDir.exists()) {
            tempDir.mkdirs()
        }

        return File.createTempFile(prefix, suffix, tempDir)
    }

    /**
     * Test archive operations with a simple file map
     *
     * @return true if archive operations are working correctly
     */

    fun testArchiveOperations(): Boolean {
        return try {
            val testFileMap = mapOf(
                "credentials/.ziplock_placeholder" to "".toByteArray(),
                "metadata.yml" to "version: 1.0\nformat: memory-v1\ncreated_at: 1700000000\nlast_modified: 1700000000\ncredential_count: 0".toByteArray()
            )

            val password = "test123"

            // Test creation
            val createResult = createArchive(testFileMap, password)
            if (!createResult.success || createResult.archiveData == null) {
                Log.e(TAG, "Test creation failed: ${createResult.error}")
                return false
            }

            // Test extraction
            val extractResult = extractArchiveFromBytes(createResult.archiveData, password)
            if (!extractResult.success || extractResult.fileMap == null) {
                Log.e(TAG, "Test extraction failed: ${extractResult.error}")
                return false
            }

            // Debug: Log extracted files
            Log.d(TAG, "Test extraction completed. Files found: ${extractResult.fileMap.keys.joinToString(", ")}")

            // Verify content - check for placeholder file
            val extractedPlaceholder = extractResult.fileMap["credentials/.ziplock_placeholder"]
            if (extractedPlaceholder == null) {
                Log.e(TAG, "Test content verification failed - placeholder file missing. Available files: ${extractResult.fileMap.keys}")
                return false
            }
            Log.d(TAG, "Placeholder file found: ${extractedPlaceholder.size} bytes")

            // Verify metadata file
            val extractedMetadata = extractResult.fileMap["metadata.yml"]
            if (extractedMetadata == null) {
                Log.e(TAG, "Test content verification failed - metadata.yml missing. Available files: ${extractResult.fileMap.keys}")
                return false
            }

            val metadataContent = String(extractedMetadata)
            if (!metadataContent.contains("format: memory-v1")) {
                Log.e(TAG, "Test content verification failed - metadata invalid. Content: $metadataContent")
                return false
            }
            Log.d(TAG, "Metadata file validated: ${extractedMetadata.size} bytes")

            Log.d(TAG, "Archive operations test passed")
            true

        } catch (e: Exception) {
            Log.e(TAG, "Archive operations test failed", e)
            false
        }
    }

    /**
     * Clean up temporary files
     */
    fun cleanupTempFiles() {
        try {
            val tempDir = File(context.cacheDir, "ziplock_temp")
            if (tempDir.exists()) {
                tempDir.listFiles()?.forEach { file: File ->
                    if (file.isFile) {
                        file.delete()
                        Log.d(TAG, "Cleaned up temp file: ${file.name}")
                    }
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to clean up temp files", e)
        }
    }
}
