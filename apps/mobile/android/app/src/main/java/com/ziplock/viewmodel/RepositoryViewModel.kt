package com.ziplock.viewmodel

import android.content.Context
import android.content.SharedPreferences
import androidx.lifecycle.ViewModel
import com.ziplock.config.AndroidConfigManager
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay
import kotlinx.coroutines.withTimeoutOrNull
import java.io.File
import com.ziplock.ffi.ZipLockNative

/**
 * Repository View Model
 *
 * Manages the state and business logic for repository operations including:
 * - Opening existing archives with passphrase validation
 * - Creating new archives
 * - FFI library integration for archive operations
 * - Error handling and user feedback
 * - Persistent storage of last opened archive path
 *
 * This view model serves as the bridge between the UI and the shared FFI library,
 * handling all repository-related operations without exposing crypto implementation details.
 */
class RepositoryViewModel(private val context: Context) : ViewModel() {

    // Configuration manager for persistent settings
    private val configManager: AndroidConfigManager = AndroidConfigManager(context)

    // UI State
    private val _uiState = MutableStateFlow(RepositoryUiState())
    val uiState: StateFlow<RepositoryUiState> = _uiState.asStateFlow()

    // Repository State
    private val _repositoryState = MutableStateFlow<RepositoryState>(RepositoryState.None)
    val repositoryState: StateFlow<RepositoryState> = _repositoryState.asStateFlow()

    // Expose config manager's last archive path
    val lastArchivePath: StateFlow<String?> = configManager.lastArchivePath

    /**
     * Get the last opened archive path if it still exists
     *
     * @return The path to the last opened archive file, or null if none exists or file is inaccessible
     */
    fun getLastOpenedArchivePath(): String? {
        return configManager.getLastOpenedArchivePath()
    }

    /**
     * Check if there's a valid last opened archive that can be auto-opened
     */
    fun hasValidLastArchive(): Boolean {
        return configManager.hasValidLastArchive()
    }

    /**
     * Open an existing archive
     *
     * @param filePath Path to the .7z archive file
     * @param passphrase User-provided passphrase for decryption
     */
    fun openRepository(filePath: String, passphrase: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null
            )

            try {
                // Validate inputs
                if (filePath.isBlank()) {
                    throw IllegalArgumentException("Archive file path is required")
                }

                if (passphrase.isBlank()) {
                    throw IllegalArgumentException("Passphrase is required")
                }

                // For content URIs, run diagnostics first
                if (filePath.startsWith("content://")) {
                    println("RepositoryViewModel: Detected content URI, running diagnostics...")
                    val diagnostics = ZipLockNative.testContentUriAccess(filePath)
                    println("RepositoryViewModel: Content URI diagnostics:\n$diagnostics")

                    // Check SAF availability
                    val safAvailable = ZipLockNative.isAndroidSafAvailable()
                    if (!safAvailable) {
                        throw Exception("Android Storage Access Framework is not available. Please restart the app and try again.")
                    }

                    println("RepositoryViewModel: SAF is available, proceeding with archive opening...")
                }

                // Call FFI library to open the archive with timeout
                delay(500) // Small delay for better UX

                // Convert content URI to usable file path for native library
                val usableFilePath = if (filePath.startsWith("content://")) {
                    val uri = android.net.Uri.parse(filePath)
                    val fileName = uri.lastPathSegment ?: "archive.7z"
                    com.ziplock.utils.FileUtils.getUsableFilePath(context, uri, fileName)
                } else {
                    filePath
                }

                println("RepositoryViewModel: Converting path '$filePath' to '$usableFilePath'")

                // Verify file exists and is accessible before calling native library
                val file = File(usableFilePath)
                if (!file.exists()) {
                    throw Exception("Archive file does not exist: $usableFilePath")
                }
                if (!file.canRead()) {
                    throw Exception("Cannot read archive file (permission denied): $usableFilePath")
                }
                if (file.length() == 0L) {
                    throw Exception("Archive file is empty: $usableFilePath")
                }

                println("RepositoryViewModel: File verification passed - size: ${file.length()} bytes")

                // Additional file inspection
                try {
                    val fileBytes = file.readBytes()
                    println("RepositoryViewModel: File header (first 16 bytes): ${fileBytes.take(16).joinToString(" ") { "%02x".format(it) }}")

                    // Check if it's a valid 7z file (should start with "7z¼¯'")
                    val expectedHeader = byteArrayOf(0x37, 0x7A, 0xBC.toByte(), 0xAF.toByte(), 0x27, 0x1C)
                    val actualHeader = fileBytes.take(6).toByteArray()
                    val isValid7z = actualHeader.contentEquals(expectedHeader)
                    println("RepositoryViewModel: Is valid 7z header: $isValid7z")

                    if (!isValid7z) {
                        // Try copying to cache and see if that helps
                        val cacheFile = File(context.cacheDir, "temp_archive_${System.currentTimeMillis()}.7z")
                        file.copyTo(cacheFile, overwrite = true)
                        println("RepositoryViewModel: Copied to cache: ${cacheFile.absolutePath} (${cacheFile.length()} bytes)")

                        // Use the cached file instead
                        val newUsableFilePath = cacheFile.absolutePath
                        println("RepositoryViewModel: Using cached file path: $newUsableFilePath")
                        println("RepositoryViewModel: Opening archive at path: $newUsableFilePath")

                        // Set a reasonable timeout for archive opening (5 minutes for large files)
                        val timeoutMs = 300_000L // 5 minutes
                        val startTime = System.currentTimeMillis()

                        val result = kotlinx.coroutines.withTimeoutOrNull(timeoutMs) {
                            try {
                                println("RepositoryViewModel: Calling ZipLockNative.openArchive with cached file...")
                                ZipLockNative.openArchive(newUsableFilePath, passphrase)
                            } catch (e: Exception) {
                                println("RepositoryViewModel: Native library call failed with cached file: ${e.message}")
                                e.printStackTrace()
                                throw Exception("Native library error with cached file: ${e.message}", e)
                            }
                        }

                        val elapsed = System.currentTimeMillis() - startTime
                        println("RepositoryViewModel: Archive opening (cached) took ${elapsed}ms")

                        if (result == null) {
                            throw Exception("Archive opening timed out after ${timeoutMs / 1000} seconds. This may be due to a large file or network issues.")
                        }

                        println("RepositoryViewModel: Open archive result (cached) - success: ${result.success}, sessionId: ${result.sessionId}, error: ${result.errorMessage}")

                        if (result.success) {
                            _uiState.value = _uiState.value.copy(
                                isLoading = false,
                                successMessage = "Archive opened successfully",
                                errorMessage = null
                            )
                        } else {
                            _uiState.value = _uiState.value.copy(
                                isLoading = false,
                                errorMessage = result.errorMessage ?: "Failed to open archive with cached file"
                            )
                        }
                        return@launch
                    }
                } catch (e: Exception) {
                    println("RepositoryViewModel: File inspection failed: ${e.message}")
                }

                println("RepositoryViewModel: Opening archive at path: $usableFilePath")

                // Set a reasonable timeout for archive opening (5 minutes for large files)
                val timeoutMs = 300_000L // 5 minutes
                val startTime = System.currentTimeMillis()

                val result = kotlinx.coroutines.withTimeoutOrNull(timeoutMs) {
                    try {
                        // Additional validation before calling native library
                        val file = File(usableFilePath)
                        println("RepositoryViewModel: Pre-call validation:")
                        println("  - File exists: ${file.exists()}")
                        println("  - File readable: ${file.canRead()}")
                        println("  - File size: ${file.length()} bytes")
                        println("  - File absolute path: ${file.absolutePath}")
                        println("  - Passphrase length: ${passphrase.length}")

                        // Validate file path for native library compatibility
                        val sanitizedPath = file.absolutePath
                        println("  - Sanitized path: $sanitizedPath")

                        // Check for problematic characters that might cause native library issues
                        if (sanitizedPath.contains('\u0000') || sanitizedPath.contains('\n') || sanitizedPath.contains('\r')) {
                            throw Exception("File path contains invalid characters that may cause native library issues")
                        }

                        // Ensure path is not too long (typical filesystem limit)
                        if (sanitizedPath.length > 4096) {
                            throw Exception("File path is too long for native library")
                        }

                        // Check if archive is already open
                        val isOpen = ZipLockNative.isArchiveOpen()
                        println("  - Archive already open: $isOpen")

                        if (isOpen) {
                            println("RepositoryViewModel: Closing existing archive before opening new one...")
                            ZipLockNative.closeArchive()
                        }

                        // Detailed file analysis before opening
                        println("RepositoryViewModel: Performing detailed file analysis...")
                        try {
                            val fileBytes = file.readBytes()
                            println("  - Successfully read ${fileBytes.size} bytes from file")

                            // Check 7z signature (should be "7z\xBC\xAF\x27\x1C")
                            if (fileBytes.size >= 6) {
                                val signature = fileBytes.take(6).map { String.format("%02X", it.toInt() and 0xFF) }.joinToString(" ")
                                println("  - File signature: $signature")

                                val expected = listOf(0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C)
                                val matches = fileBytes.take(6).zip(expected).all { (actual, expected) ->
                                    (actual.toInt() and 0xFF) == expected
                                }
                                println("  - 7z signature valid: $matches")
                            }

                            // Check for null bytes in path (common cause of native crashes)
                            val pathBytes = sanitizedPath.toByteArray(Charsets.UTF_8)
                            val hasNullBytes = pathBytes.contains(0.toByte())
                            println("  - Path has null bytes: $hasNullBytes")
                            println("  - Path byte length: ${pathBytes.size}")

                        } catch (e: Exception) {
                            println("  - File analysis failed: ${e.message}")
                        }

                        println("RepositoryViewModel: Calling ZipLockNative.openArchive with sanitized path...")
                        ZipLockNative.openArchive(sanitizedPath, passphrase)
                    } catch (e: Exception) {
                        println("RepositoryViewModel: Native library call failed: ${e.message}")
                        e.printStackTrace()
                        null
                    }
                }

                val elapsed = System.currentTimeMillis() - startTime
                println("RepositoryViewModel: Archive opening took ${elapsed}ms")

                if (result == null) {
                    throw Exception("Archive opening timed out after ${timeoutMs / 1000} seconds. This may be due to a large file or network issues.")
                }

                println("RepositoryViewModel: Open archive result - success: ${result.success}, sessionId: ${result.sessionId}, error: ${result.errorMessage}")

                if (!result.success) {
                    // Get detailed error information from native library
                    val detailedError = ZipLockNative.getLastError()
                    var errorMessage = detailedError ?: result.errorMessage ?: "Unknown error"

                    // Provide more specific error messages for common content URI issues
                    if (filePath.startsWith("content://")) {
                        when {
                            errorMessage.contains("Android SAF not available") -> {
                                errorMessage = "Storage Access Framework error. Please restart the app and try again."
                            }
                            errorMessage.contains("Failed to open content URI") -> {
                                errorMessage = "Cannot access the selected file. Please ensure you have permission and the file exists."
                            }
                            errorMessage.contains("Failed to create temporary file") -> {
                                errorMessage = "Insufficient storage space or permission denied. Please free up space and try again."
                            }
                            errorMessage.contains("Invalid master password") || errorMessage.contains("CryptoError") -> {
                                errorMessage = "Incorrect password. Please check your password and try again."
                            }
                            errorMessage.contains("timed out") -> {
                                errorMessage = "The file is taking too long to open. This may be due to file size or network issues."
                            }
                        }
                    }

                    println("RepositoryViewModel: Detailed error: $errorMessage")
                    throw Exception(errorMessage)
                }

                if (result.success) {
                    val sessionId = result.sessionId ?: generateSessionId()
                    _repositoryState.value = RepositoryState.Opened(
                        archivePath = filePath,
                        sessionId = sessionId
                    )
                    println("RepositoryViewModel: Archive opened successfully with session: $sessionId")

                    // Verify the archive is actually open
                    val isOpen = ZipLockNative.isArchiveOpen()
                    println("RepositoryViewModel: Archive open verification: $isOpen")
                }

                // Save the successfully opened archive path
                configManager.setLastArchivePath(filePath)

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    successMessage = "Archive opened successfully"
                )

            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = mapErrorMessage(e)
                )
            }
        }
    }

    /**
     * Create a new archive
     *
     * @param filePath Path where the new .7z archive should be created
     * @param passphrase User-provided passphrase for encryption
     */
    fun createRepository(filePath: String, passphrase: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null
            )

            try {
                // Validate inputs
                if (filePath.isBlank()) {
                    throw IllegalArgumentException("Archive file path is required")
                }

                if (passphrase.length < 8) {
                    throw IllegalArgumentException("Passphrase must be at least 8 characters long")
                }

                // TODO: Integrate with shared FFI library
                // This is where we'll call the shared library to create a new archive

                // Simulate FFI call for now
                delay(2000) // Simulate processing time

                // Example of what the FFI integration would look like:
                /*
                val result = ZipLockNative.createArchive(filePath, passphrase)
                if (result.isSuccess()) {
                    _repositoryState.value = RepositoryState.Created(
                        archivePath = filePath,
                        sessionId = result.sessionId
                    )
                } else {
                    throw Exception(result.errorMessage)
                }
                */

                // For now, simulate successful creation
                _repositoryState.value = RepositoryState.Created(
                    archivePath = filePath,
                    sessionId = generateSessionId()
                )

                // Save the successfully created archive path
                configManager.setLastArchivePath(filePath)

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    successMessage = "New archive created successfully"
                )

            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = mapErrorMessage(e)
                )
            }
        }
    }

    /**
     * Close the currently open repository
     */
    fun closeRepository() {
        viewModelScope.launch {
            try {
                // TODO: Integrate with shared FFI library to properly close the archive
                /*
                when (val state = _repositoryState.value) {
                    is RepositoryState.Opened -> {
                        ZipLockNative.closeArchive(state.sessionId)
                    }
                    is RepositoryState.Created -> {
                        ZipLockNative.closeArchive(state.sessionId)
                    }
                    else -> { /* No action needed */ }
                }
                */

                _repositoryState.value = RepositoryState.None
                _uiState.value = RepositoryUiState() // Reset to initial state

            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    errorMessage = "Failed to close repository: ${e.message}"
                )
            }
        }
    }

    /**
     * Clear the saved last archive path
     */
    fun clearLastArchivePath() {
        configManager.clearLastArchivePath()
    }

    /**
     * Clear error messages
     */
    fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Clear success messages
     */
    fun clearSuccess() {
        _uiState.value = _uiState.value.copy(successMessage = null)
    }

    /**
     * Validate passphrase strength
     *
     * @param passphrase The passphrase to validate
     * @return PassphraseValidation result with strength and requirements
     */
    fun validatePassphrase(passphrase: String): PassphraseValidation {
        val requirements = mutableListOf<String>()
        val satisfied = mutableListOf<String>()

        // Length requirement
        if (passphrase.length < 8) {
            requirements.add("At least 8 characters")
        } else {
            satisfied.add("Minimum length (8 characters)")
        }

        // Uppercase requirement
        if (!passphrase.any { it.isUpperCase() }) {
            requirements.add("At least one uppercase letter")
        } else {
            satisfied.add("Contains uppercase letter")
        }

        // Lowercase requirement
        if (!passphrase.any { it.isLowerCase() }) {
            requirements.add("At least one lowercase letter")
        } else {
            satisfied.add("Contains lowercase letter")
        }

        // Number requirement
        if (!passphrase.any { it.isDigit() }) {
            requirements.add("At least one number")
        } else {
            satisfied.add("Contains number")
        }

        // Special character requirement
        if (!passphrase.any { !it.isLetterOrDigit() }) {
            requirements.add("At least one special character")
        } else {
            satisfied.add("Contains special character")
        }

        // Calculate strength score
        val score = when {
            requirements.size > 3 -> PassphraseStrength.VeryWeak
            requirements.size > 2 -> PassphraseStrength.Weak
            requirements.size > 1 -> PassphraseStrength.Fair
            requirements.size == 1 -> PassphraseStrength.Good
            requirements.isEmpty() && passphrase.length < 12 -> PassphraseStrength.Strong
            else -> PassphraseStrength.VeryStrong
        }

        return PassphraseValidation(
            strength = score,
            requirements = requirements,
            satisfied = satisfied,
            isValid = requirements.isEmpty() && passphrase.length >= 8
        )
    }

    /**
     * Check if a file path represents a cloud storage location
     * Implements the cloud storage detection from the cloud-storage-implementation.md
     */
    fun isCloudStorageFile(filePath: String): Boolean {
        val cloudPatterns = listOf(
            // Android cloud storage patterns
            "/Android/data/com.google.android.apps.docs/",
            "/Android/data/com.dropbox.android/",
            "/Android/data/com.microsoft.skydrive/",
            "/Android/data/com.box.android/",
            "/Android/data/com.nextcloud.client/",

            // Storage Access Framework patterns
            "content://com.android.providers.media.documents/",
            "content://com.android.externalstorage.documents/",

            // Generic cloud indicators
            "/cloud/", "/sync/", "/googledrive/", "/dropbox/", "/onedrive/"
        )

        return cloudPatterns.any { pattern ->
            filePath.contains(pattern, ignoreCase = true)
        }
    }

    /**
     * Map technical errors to user-friendly messages
     */
    private fun mapErrorMessage(error: Exception): String {
        return when {
            error.message?.contains("authentication", ignoreCase = true) == true ||
            error.message?.contains("passphrase", ignoreCase = true) == true ||
            error.message?.contains("password", ignoreCase = true) == true ->
                "Incorrect passphrase. Please check your password and try again."

            error.message?.contains("not found", ignoreCase = true) == true ||
            error.message?.contains("no such file", ignoreCase = true) == true ->
                "The archive file could not be found. Please check the file path."

            error.message?.contains("permission", ignoreCase = true) == true ||
            error.message?.contains("access denied", ignoreCase = true) == true ->
                "Permission denied. Please check file permissions or try a different location."

            error.message?.contains("corrupted", ignoreCase = true) == true ||
            error.message?.contains("invalid", ignoreCase = true) == true ->
                "The archive file appears to be corrupted or invalid."

            error.message?.contains("network", ignoreCase = true) == true ||
            error.message?.contains("connection", ignoreCase = true) == true ->
                "Network error. Please check your connection and try again."

            error is IllegalArgumentException ->
                error.message ?: "Invalid input provided."

            else -> "Failed to open archive. Please try again."
        }
    }



    /**
     * Generate a unique session ID for tracking archive operations
     */
    private fun generateSessionId(): String {
        return "session_${System.currentTimeMillis()}_${(1000..9999).random()}"
    }
}

/**
 * ViewModelFactory for RepositoryViewModel that requires context
 */
class RepositoryViewModelFactory(private val context: Context) : ViewModelProvider.Factory {
    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(RepositoryViewModel::class.java)) {
            return RepositoryViewModel(context) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class")
    }
}

/**
 * UI State for the repository operations
 */
data class RepositoryUiState(
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
    val successMessage: String? = null
)

/**
 * Repository state tracking
 */
sealed class RepositoryState {
    object None : RepositoryState()

    data class Opened(
        val archivePath: String,
        val sessionId: String
    ) : RepositoryState()

    data class Created(
        val archivePath: String,
        val sessionId: String
    ) : RepositoryState()
}

/**
 * Passphrase validation result
 */
data class PassphraseValidation(
    val strength: PassphraseStrength,
    val requirements: List<String>,
    val satisfied: List<String>,
    val isValid: Boolean
)

/**
 * Passphrase strength levels matching the design.md specification
 */
enum class PassphraseStrength(val score: Int, val label: String) {
    VeryWeak(10, "Very Weak"),
    Weak(30, "Weak"),
    Fair(50, "Fair"),
    Good(70, "Good"),
    Strong(85, "Strong"),
    VeryStrong(95, "Very Strong")
}
