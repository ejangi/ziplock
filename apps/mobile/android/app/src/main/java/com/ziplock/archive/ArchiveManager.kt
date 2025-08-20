package com.ziplock.archive

import android.content.Context
import android.net.Uri
import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.commons.compress.archivers.sevenz.SevenZArchiveEntry
import org.apache.commons.compress.archivers.sevenz.SevenZFile
import org.apache.commons.compress.archivers.sevenz.SevenZOutputFile
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException
import java.nio.file.Files
import java.nio.file.StandardCopyOption

/**
 * Native Android archive manager that handles 7z operations without relying on Rust FFI.
 * This replaces the problematic sevenz_rust2 library with pure Java/Kotlin implementation.
 */
class ArchiveManager(private val context: Context) {

    companion object {
        private const val TAG = "ArchiveManager"
    }

    data class ArchiveResult(
        val success: Boolean,
        val errorMessage: String? = null,
        val data: Any? = null
    )

    data class ArchiveEntry(
        val name: String,
        val size: Long,
        val isDirectory: Boolean,
        val content: ByteArray? = null
    )

    /**
     * Create a new encrypted 7z archive
     */
    suspend fun createArchive(
        archivePath: String,
        password: String,
        sourceDir: File
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            val archiveFile = File(archivePath)

            // Ensure parent directory exists
            archiveFile.parentFile?.mkdirs()

            SevenZOutputFile(archiveFile, password.toCharArray()).use { sevenZOutput ->
                addDirectoryToArchive(sevenZOutput, sourceDir, "")
            }

            ArchiveResult(success = true)
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Failed to create archive: ${e.message}"
            )
        }
    }

    /**
     * Open and extract an encrypted 7z archive
     */
    suspend fun openArchive(
        archivePath: String,
        password: String,
        extractToDir: File
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            val archiveFile = File(archivePath)

            if (!archiveFile.exists()) {
                return@withContext ArchiveResult(
                    success = false,
                    errorMessage = "Archive file not found: $archivePath"
                )
            }

            // Ensure extraction directory exists
            extractToDir.mkdirs()

            SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                var entry: SevenZArchiveEntry?

                while (sevenZFile.nextEntry.also { entry = it } != null) {
                    val currentEntry = entry!!
                    val entryFile = File(extractToDir, currentEntry.name)

                    // Ensure parent directory exists
                    entryFile.parentFile?.mkdirs()

                    if (currentEntry.isDirectory) {
                        entryFile.mkdirs()
                    } else {
                        FileOutputStream(entryFile).use { output ->
                            val content = ByteArray(currentEntry.size.toInt())
                            sevenZFile.read(content)
                            output.write(content)
                        }
                    }
                }
            }

            ArchiveResult(success = true, data = extractToDir)
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Failed to open archive: ${e.message}"
            )
        }
    }

    /**
     * List contents of an encrypted 7z archive without extracting
     */
    suspend fun listArchiveContents(
        archivePath: String,
        password: String
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            val archiveFile = File(archivePath)
            val entries = mutableListOf<ArchiveEntry>()

            SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                var entry: SevenZArchiveEntry?

                while (sevenZFile.nextEntry.also { entry = it } != null) {
                    val currentEntry = entry!!
                    entries.add(
                        ArchiveEntry(
                            name = currentEntry.name,
                            size = currentEntry.size,
                            isDirectory = currentEntry.isDirectory
                        )
                    )
                }
            }

            ArchiveResult(success = true, data = entries)
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Failed to list archive contents: ${e.message}"
            )
        }
    }

    /**
     * Extract a specific file from the archive
     */
    suspend fun extractFile(
        archivePath: String,
        password: String,
        fileName: String
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            val archiveFile = File(archivePath)

            SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                var entry: SevenZArchiveEntry?

                while (sevenZFile.nextEntry.also { entry = it } != null) {
                    val currentEntry = entry!!

                    if (currentEntry.name == fileName && !currentEntry.isDirectory) {
                        val content = ByteArray(currentEntry.size.toInt())
                        sevenZFile.read(content)

                        return@withContext ArchiveResult(
                            success = true,
                            data = ArchiveEntry(
                                name = currentEntry.name,
                                size = currentEntry.size,
                                isDirectory = false,
                                content = content
                            )
                        )
                    }
                }
            }

            ArchiveResult(
                success = false,
                errorMessage = "File not found in archive: $fileName"
            )
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Failed to extract file: ${e.message}"
            )
        }
    }

    /**
     * Add a file to an existing archive
     */
    suspend fun addFileToArchive(
        archivePath: String,
        password: String,
        fileToAdd: File,
        entryName: String
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            // Create a temporary archive
            val tempArchive = File.createTempFile("ziplock_temp", ".7z")

            // First, extract existing content
            val tempExtractDir = Files.createTempDirectory("ziplock_extract").toFile()

            val extractResult = openArchive(archivePath, password, tempExtractDir)
            if (!extractResult.success) {
                return@withContext extractResult
            }

            // Add the new file
            val targetFile = File(tempExtractDir, entryName)
            targetFile.parentFile?.mkdirs()
            Files.copy(fileToAdd.toPath(), targetFile.toPath(), StandardCopyOption.REPLACE_EXISTING)

            // Create new archive with all content
            val createResult = createArchive(tempArchive.absolutePath, password, tempExtractDir)
            if (!createResult.success) {
                return@withContext createResult
            }

            // Replace original archive
            Files.move(tempArchive.toPath(), File(archivePath).toPath(), StandardCopyOption.REPLACE_EXISTING)

            // Cleanup
            tempExtractDir.deleteRecursively()

            ArchiveResult(success = true)
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Failed to add file to archive: ${e.message}"
            )
        }
    }

    /**
     * Validate if a file is a valid 7z archive
     */
    suspend fun validateArchive(
        archivePath: String,
        password: String? = null
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            val archiveFile = File(archivePath)
            Log.d(TAG, "Archive file path: ${archiveFile.absolutePath}")
            Log.d(TAG, "Archive file exists: ${archiveFile.exists()}")

            if (!archiveFile.exists()) {
                return@withContext ArchiveResult(
                    success = false,
                    errorMessage = "Archive file does not exist"
                )
            }

            Log.d(TAG, "Archive file size: ${archiveFile.length()} bytes")
            if (archiveFile.length() < 32) {
                return@withContext ArchiveResult(
                    success = false,
                    errorMessage = "File too small to be a valid 7z archive"
                )
            }

            // Try to open the archive
            Log.d(TAG, "Attempting to open archive with Apache Commons Compress...")
            if (password != null) {
                Log.d(TAG, "Opening password-protected archive")
                SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                    Log.d(TAG, "SevenZFile opened successfully with password")
                    // Try to read first entry to validate password and format
                    val firstEntry = sevenZFile.nextEntry
                    Log.d(TAG, "First entry read: ${firstEntry?.name ?: "null"}")
                }
            } else {
                Log.d(TAG, "Opening archive without password")
                SevenZFile(archiveFile).use { sevenZFile ->
                    Log.d(TAG, "SevenZFile opened successfully without password")
                    // Try to read first entry to validate format
                    val firstEntry = sevenZFile.nextEntry
                    Log.d(TAG, "First entry read: ${firstEntry?.name ?: "null"}")
                }
            }

            Log.d(TAG, "Archive validation completed successfully")
            ArchiveResult(success = true)
        } catch (e: Exception) {
            Log.e(TAG, "Archive validation failed with exception", e)
            Log.e(TAG, "Exception type: ${e.javaClass.simpleName}")
            Log.e(TAG, "Exception message: ${e.message}")
            Log.e(TAG, "Exception cause: ${e.cause?.message}")
            ArchiveResult(
                success = false,
                errorMessage = "Archive validation failed: ${e.javaClass.simpleName}: ${e.message}"
            )
        }
    }

    /**
     * Get archive information (size, entry count, etc.)
     */
    suspend fun getArchiveInfo(
        archivePath: String,
        password: String
    ): ArchiveResult = withContext(Dispatchers.IO) {
        try {
            val archiveFile = File(archivePath)
            var entryCount = 0
            var totalUncompressedSize = 0L

            SevenZFile(archiveFile, password.toCharArray()).use { sevenZFile ->
                var entry: SevenZArchiveEntry?

                while (sevenZFile.nextEntry.also { entry = it } != null) {
                    val currentEntry = entry!!
                    entryCount++
                    totalUncompressedSize += currentEntry.size
                }
            }

            val info = mapOf(
                "entryCount" to entryCount,
                "compressedSize" to archiveFile.length(),
                "uncompressedSize" to totalUncompressedSize,
                "compressionRatio" to if (totalUncompressedSize > 0) {
                    (archiveFile.length().toDouble() / totalUncompressedSize.toDouble())
                } else 0.0
            )

            ArchiveResult(success = true, data = info)
        } catch (e: Exception) {
            ArchiveResult(
                success = false,
                errorMessage = "Failed to get archive info: ${e.message}"
            )
        }
    }

    private fun addDirectoryToArchive(
        sevenZOutput: SevenZOutputFile,
        directory: File,
        basePath: String
    ) {
        directory.listFiles()?.forEach { file ->
            val entryName = if (basePath.isEmpty()) file.name else "$basePath/${file.name}"

            if (file.isDirectory) {
                // Add directory entry
                val entry = sevenZOutput.createArchiveEntry(file, entryName)
                sevenZOutput.putArchiveEntry(entry)
                sevenZOutput.closeArchiveEntry()

                // Recursively add directory contents
                addDirectoryToArchive(sevenZOutput, file, entryName)
            } else {
                // Add file entry
                val entry = sevenZOutput.createArchiveEntry(file, entryName)
                sevenZOutput.putArchiveEntry(entry)

                FileInputStream(file).use { input ->
                    val buffer = ByteArray(8192)
                    var bytesRead: Int
                    while (input.read(buffer).also { bytesRead = it } != -1) {
                        sevenZOutput.write(buffer, 0, bytesRead)
                    }
                }

                sevenZOutput.closeArchiveEntry()
            }
        }
    }
}
