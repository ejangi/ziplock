package com.ziplock.viewmodel

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ziplock.ffi.ZipLockNative
import com.ziplock.ffi.ZipLockNativeHelper
import com.ziplock.utils.PassphraseStrengthResult

import com.ziplock.utils.FileUtils
import com.ziplock.utils.WritableArchiveInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * ViewModel for the Create Archive Wizard
 *
 * Manages the state and business logic for creating new ZipLock archives,
 * including passphrase validation, FFI integration, and progress tracking.
 */
class CreateArchiveViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(CreateArchiveUiState())
    val uiState: StateFlow<CreateArchiveUiState> = _uiState.asStateFlow()

    private val _passphraseStrength = MutableStateFlow<PassphraseStrengthResult?>(null)
    val passphraseStrength: StateFlow<PassphraseStrengthResult?> = _passphraseStrength.asStateFlow()

    init {
        // Initialize gracefully without crashing
        // FFI availability will be checked when needed
    }

    /**
     * Update the current wizard step
     */
    fun updateStep(step: CreateArchiveStep) {
        Log.d("CreateArchiveViewModel", "updateStep called: $step")
        _uiState.value = _uiState.value.copy(
            currentStep = step,
            errorMessage = null
        )
    }

    /**
     * Set the destination folder path
     */
    fun setDestination(path: String, name: String) {
        Log.d("CreateArchiveViewModel", "setDestination called: path=$path, name=$name")
        _uiState.value = _uiState.value.copy(
            destinationPath = path,
            destinationName = name,
            errorMessage = null
        )
    }

    /**
     * Update the archive name with validation and sanitization
     */
    fun updateArchiveName(name: String) {
        val sanitizedName = sanitizeArchiveName(name)
        _uiState.value = _uiState.value.copy(
            archiveName = sanitizedName,
            errorMessage = null
        )
    }

    /**
     * Update the passphrase and validate it
     */
    fun updatePassphrase(passphrase: String) {
        _uiState.value = _uiState.value.copy(
            passphrase = passphrase,
            errorMessage = null
        )

        // Validate passphrase strength asynchronously
        if (passphrase.isNotEmpty()) {
            viewModelScope.launch {
                validatePassphraseStrength(passphrase)
            }
        } else {
            _passphraseStrength.value = null
        }
    }

    /**
     * Update the confirmation passphrase
     */
    fun updateConfirmPassphrase(confirmPassphrase: String) {
        _uiState.value = _uiState.value.copy(
            confirmPassphrase = confirmPassphrase,
            errorMessage = null
        )
    }

    /**
     * Toggle passphrase visibility
     */
    fun togglePassphraseVisibility() {
        _uiState.value = _uiState.value.copy(
            showPassphrase = !_uiState.value.showPassphrase
        )
    }

    /**
     * Toggle confirm passphrase visibility
     */
    fun toggleConfirmPassphraseVisibility() {
        _uiState.value = _uiState.value.copy(
            showConfirmPassphrase = !_uiState.value.showConfirmPassphrase
        )
    }

    /**
     * Validate passphrase strength using FFI
     */
    private suspend fun validatePassphraseStrength(passphrase: String) {
        withContext(Dispatchers.IO) {
            try {
                // Check if FFI library is available before using it
                if (isFFIAvailable()) {
                    val result = PassphraseStrengthResult.analyze(passphrase)
                    _passphraseStrength.value = result
                } else {
                    // Use fallback validation when FFI is not available
                    val fallbackStrength = createFallbackValidation(passphrase)
                    _passphraseStrength.value = fallbackStrength
                }
            } catch (e: Exception) {
                // Fallback validation if FFI call fails
                val fallbackStrength = createFallbackValidation(passphrase)
                _passphraseStrength.value = fallbackStrength
            }
        }
    }

    /**
     * Create fallback passphrase validation when FFI is not available
     */
    private fun createFallbackValidation(passphrase: String): PassphraseStrengthResult {
        // Use the PassphraseStrengthResult.analyze method instead of custom logic
        return PassphraseStrengthResult.analyze(passphrase)
    }

    /**
     * Check if the user can proceed to the next step
     */
    fun canProceed(): Boolean {
        val currentState = _uiState.value
        val result = when (currentState.currentStep) {
            CreateArchiveStep.SelectDestination -> currentState.destinationPath != null
            CreateArchiveStep.ArchiveName -> currentState.archiveName.isNotBlank()
            CreateArchiveStep.CreatePassphrase -> _passphraseStrength.value?.isValid == true
            CreateArchiveStep.ConfirmPassphrase -> {
                currentState.confirmPassphrase.isNotEmpty() &&
                currentState.passphrase == currentState.confirmPassphrase
            }
            CreateArchiveStep.Creating -> false
            CreateArchiveStep.Success -> true
        }
        Log.d("CreateArchiveViewModel", "canProceed for ${currentState.currentStep}: $result")
        Log.d("CreateArchiveViewModel", "destinationPath: ${currentState.destinationPath}")
        Log.d("CreateArchiveViewModel", "archiveName: '${currentState.archiveName}'")
        Log.d("CreateArchiveViewModel", "passphrase strength valid: ${_passphraseStrength.value?.isValid}")
        return result
    }

    /**
     * Proceed to the next step with validation
     */
    fun proceedToNext() {
        val currentState = _uiState.value
        Log.d("CreateArchiveViewModel", "proceedToNext called, current step: ${currentState.currentStep}")

        when (currentState.currentStep) {
            CreateArchiveStep.SelectDestination -> {
                Log.d("CreateArchiveViewModel", "SelectDestination step, destinationPath: ${currentState.destinationPath}")
                if (currentState.destinationPath != null) {
                    Log.d("CreateArchiveViewModel", "Proceeding to ArchiveName step")
                    updateStep(CreateArchiveStep.ArchiveName)
                } else {
                    Log.d("CreateArchiveViewModel", "No destination selected, showing error")
                    setError("Please select a destination folder where your archive will be saved.")
                }
            }
            CreateArchiveStep.ArchiveName -> {
                Log.d("CreateArchiveViewModel", "ArchiveName step, archiveName: '${currentState.archiveName}'")
                if (currentState.archiveName.isNotBlank()) {
                    Log.d("CreateArchiveViewModel", "Proceeding to CreatePassphrase step")
                    updateStep(CreateArchiveStep.CreatePassphrase)
                } else {
                    Log.d("CreateArchiveViewModel", "Archive name is blank, showing error")
                    setError("Please enter a name for your archive. This will be the filename of your .7z file.")
                }
            }
            CreateArchiveStep.CreatePassphrase -> {
                Log.d("CreateArchiveViewModel", "CreatePassphrase step, passphrase strength valid: ${_passphraseStrength.value?.isValid}")
                if (_passphraseStrength.value?.isValid == true) {
                    Log.d("CreateArchiveViewModel", "Proceeding to ConfirmPassphrase step")
                    updateStep(CreateArchiveStep.ConfirmPassphrase)
                } else {
                    Log.d("CreateArchiveViewModel", "Passphrase not strong enough, showing error")
                    setError("Please create a stronger passphrase that meets all the security requirements.")
                }
            }
            CreateArchiveStep.ConfirmPassphrase -> {
                Log.d("CreateArchiveViewModel", "ConfirmPassphrase step, passphrases match: ${currentState.passphrase == currentState.confirmPassphrase}")
                if (currentState.passphrase == currentState.confirmPassphrase) {
                    Log.d("CreateArchiveViewModel", "Starting archive creation")
                    setError("Context is required for archive creation. Please use startArchiveCreation(context) instead.")
                } else {
                    Log.d("CreateArchiveViewModel", "Passphrases don't match, showing error")
                    setError("The passphrases do not match. Please make sure both entries are identical.")
                }
            }
            else -> { /* No action for Creating and Success */ }
        }
    }

    /**
     * Go back to the previous step
     */
    fun goBack() {
        val currentState = _uiState.value

        val previousStep = when (currentState.currentStep) {
            CreateArchiveStep.SelectDestination -> return // No previous step
            CreateArchiveStep.ArchiveName -> CreateArchiveStep.SelectDestination
            CreateArchiveStep.CreatePassphrase -> CreateArchiveStep.ArchiveName
            CreateArchiveStep.ConfirmPassphrase -> CreateArchiveStep.CreatePassphrase
            else -> return // No previous step or can't go back
        }

        updateStep(previousStep)
    }

    /**
     * Start the archive creation process with Android context for content URI handling
     */
    fun startArchiveCreation(context: Context) {
        val currentState = _uiState.value
        Log.d("CreateArchiveViewModel", "startArchiveCreation called")

        _uiState.value = currentState.copy(
            currentStep = CreateArchiveStep.Creating,
            isLoading = true,
            creationProgress = 0f,
            errorMessage = null
        )

        viewModelScope.launch {
            // CRITICAL DEBUG: Log passphrase at the start of archive creation
            Log.d("CreateArchiveViewModel", "ENCRYPTION DEBUG: Starting archive creation")
            Log.d("CreateArchiveViewModel", "ENCRYPTION DEBUG: UI passphrase length: ${currentState.passphrase.length}")
            Log.d("CreateArchiveViewModel", "ENCRYPTION DEBUG: UI passphrase empty: ${currentState.passphrase.isEmpty()}")
            if (currentState.passphrase.isEmpty()) {
                Log.e("CreateArchiveViewModel", "🚨 CRITICAL: UI passphrase is empty! Archive will be unencrypted!")
            }

            createArchive(
                context = context,
                destinationPath = currentState.destinationPath!!,
                archiveName = currentState.archiveName,
                passphrase = currentState.passphrase
            )
        }
    }

    /**
     * Create the archive using FFI with proper content URI handling
     */
    private suspend fun createArchive(
        context: Context,
        destinationPath: String,
        archiveName: String,
        passphrase: String
    ) {
        withContext(Dispatchers.IO) {
            var archiveInfo: WritableArchiveInfo? = null
            try {
                // Convert destination URI to a writable file path
                val destinationUri = Uri.parse(destinationPath)
                archiveInfo = FileUtils.getWritableArchivePath(context, destinationUri, archiveName)

                // Update progress
                updateProgress(0.1f)
                Log.d("CreateArchiveViewModel", "Creating archive at working path: ${archiveInfo.workingPath}")
                if (archiveInfo.needsCopyBack) {
                    Log.d("CreateArchiveViewModel", "Will copy back to: ${archiveInfo.finalDestinationUri}")
                }

                // CRITICAL DEBUG: Log passphrase details for encryption verification
                Log.d("CreateArchiveViewModel", "ENCRYPTION DEBUG: Passphrase provided: ${passphrase.isNotEmpty()}")
                Log.d("CreateArchiveViewModel", "ENCRYPTION DEBUG: Passphrase length: ${passphrase.length}")
                if (passphrase.isEmpty()) {
                    Log.w("CreateArchiveViewModel", "⚠️ WARNING: Empty passphrase - archive will be UNENCRYPTED!")
                } else {
                    Log.d("CreateArchiveViewModel", "✓ Non-empty passphrase - archive should be encrypted")
                }

                // Create the archive using the repository manager
                val workingFileUri = Uri.fromFile(java.io.File(archiveInfo.workingPath))
                val repositoryManager = com.ziplock.repository.MobileRepositoryManager.getInstance(context)
                Log.d("CreateArchiveViewModel", "Calling createRepository with URI: $workingFileUri")
                Log.d("CreateArchiveViewModel", "ENCRYPTION DEBUG: Calling createRepository with passphrase length: ${passphrase.length}")
                val result = repositoryManager.createRepository(workingFileUri, passphrase)

                updateProgress(0.5f)
                Log.d("CreateArchiveViewModel", "Repository creation result: ${result::class.simpleName}")

                when (result) {
                    is com.ziplock.repository.MobileRepositoryManager.RepositoryResult.Success<*> -> {
                        Log.d("CreateArchiveViewModel", "Archive creation successful")
                        repositoryManager.closeRepository() // Close the new empty repository
                    // If we need to copy back to the original location, do it now
                    if (archiveInfo.needsCopyBack && archiveInfo.finalDestinationUri != null) {
                        updateProgress(0.7f)
                        Log.d("CreateArchiveViewModel", "Copying archive back to final destination")

                        val copySuccess = FileUtils.copyBackToDestination(
                            context,
                            archiveInfo.workingPath,
                            destinationUri
                        )

                        if (!copySuccess) {
                            throw Exception("Failed to copy archive to destination folder")
                        }
                    }

                    updateProgress(1.0f)

                    val finalPath = if (archiveInfo.needsCopyBack) {
                        archiveInfo.finalDestinationUri?.toString() ?: destinationPath
                    } else {
                        archiveInfo.workingPath
                    }

                    _uiState.value = _uiState.value.copy(
                        currentStep = CreateArchiveStep.Success,
                        isLoading = false,
                        createdArchivePath = finalPath,
                        errorMessage = null
                    )
                    }

                    is com.ziplock.repository.MobileRepositoryManager.RepositoryResult.Error<*> -> {
                        Log.e("CreateArchiveViewModel", "Repository creation failed: ${result.message}")
                        // Map specific error codes to better messages
                        val userMessage = when {
                            result.message.contains("Internal error", ignoreCase = true) -> "Failed to create archive. The selected location may not be writable or the filename may be invalid."
                            result.message.contains("permission", ignoreCase = true) -> "Permission denied. Please check that you have write access to the selected folder."
                            result.message.contains("Invalid parameter", ignoreCase = true) -> "Invalid archive name or destination. Please check your inputs and try again."
                            else -> result.message
                        }
                        throw Exception(userMessage)
                    }
                }

            } catch (e: Exception) {
                Log.e("CreateArchiveViewModel", "Archive creation failed with exception", e)
                // Clean up working file if creation failed
                archiveInfo?.let { info ->
                    try {
                        java.io.File(info.workingPath).delete()
                        Log.d("CreateArchiveViewModel", "Cleaned up working file: ${info.workingPath}")
                    } catch (deleteException: Exception) {
                        Log.w("CreateArchiveViewModel", "Failed to clean up working file", deleteException)
                    }
                }

                val userMessage = when {
                    e.message?.contains("permission", ignoreCase = true) == true ->
                        "Permission denied. Please check that you have write access to the selected folder."
                    e.message?.contains("space", ignoreCase = true) == true ->
                        "Insufficient storage space. Please free up space and try again."
                    e.message?.contains("network", ignoreCase = true) == true ->
                        "Network error. Please check your internet connection for cloud storage."
                    e.message?.contains("copy", ignoreCase = true) == true ->
                        "Archive was created but could not be moved to the selected folder. Please try a different location."
                    else -> "Failed to create archive: ${e.message ?: "Unknown error occurred"}"
                }
                setError(userMessage)
            }
        }
    }

    /**
     * Update creation progress
     */
    private fun updateProgress(progress: Float) {
        _uiState.value = _uiState.value.copy(creationProgress = progress)
    }

    /**
     * Set an error message and stop loading
     */
    private fun setError(message: String) {
        _uiState.value = _uiState.value.copy(
            errorMessage = message,
            isLoading = false,
            currentStep = if (_uiState.value.currentStep == CreateArchiveStep.Creating) {
                CreateArchiveStep.ConfirmPassphrase
            } else {
                _uiState.value.currentStep
            }
        )
    }

    /**
     * Clear the current error message
     */
    fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Reset the wizard to the beginning
     */
    fun reset() {
        _uiState.value = CreateArchiveUiState()
        _passphraseStrength.value = null
    }

    /**
     * Get the created archive path (for success step)
     */
    fun getCreatedArchivePath(): String? {
        return _uiState.value.createdArchivePath
    }

    /**
     * Clean up resources when ViewModel is destroyed
     */
    override fun onCleared() {
        super.onCleared()
        // Clean up any temporary files that might have been created
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _uiState.value.createdArchivePath?.let { path ->
                    // Only clean up if it's a temporary working file
                    if (path.contains("/cache/new_archives/")) {
                        java.io.File(path).delete()
                    }
                }
            } catch (e: Exception) {
                // Ignore cleanup errors
            }
        }
    }

    /**
     * Safely check if FFI library is available without throwing exceptions
     */
    private fun isFFIAvailable(): Boolean {
        return try {
            ZipLockNative.init() == 0
        } catch (e: UnsatisfiedLinkError) {
            false
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Sanitize archive name to prevent invalid characters
     */
    private fun sanitizeArchiveName(name: String): String {
        // Remove invalid filename characters and limit length
        return name
            .replace(Regex("[<>:\"/\\\\|?*]"), "") // Remove invalid file characters
            .replace(Regex("\\s+"), " ") // Normalize whitespace
            .trim()
            .take(100) // Limit length to prevent filesystem issues
    }

    /**
     * Validate archive name for common issues
     */
    fun validateArchiveName(name: String): String? {
        return when {
            name.isBlank() -> "Archive name cannot be empty"
            name.length > 100 -> "Archive name is too long (maximum 100 characters)"
            name.contains(Regex("[<>:\"/\\\\|?*]")) -> "Archive name contains invalid characters"
            name.startsWith(".") -> "Archive name cannot start with a dot"
            name.endsWith(".") -> "Archive name cannot end with a dot"
            name.equals("CON", ignoreCase = true) ||
            name.equals("PRN", ignoreCase = true) ||
            name.equals("AUX", ignoreCase = true) ||
            name.equals("NUL", ignoreCase = true) -> "Archive name conflicts with system reserved names"
            else -> null
        }
    }
}

/**
 * UI state for the Create Archive wizard
 */
data class CreateArchiveUiState(
    val currentStep: CreateArchiveStep = CreateArchiveStep.SelectDestination,
    val destinationPath: String? = null,
    val destinationName: String? = null,
    val archiveName: String = "ZipLock",
    val passphrase: String = "",
    val confirmPassphrase: String = "",
    val showPassphrase: Boolean = false,
    val showConfirmPassphrase: Boolean = false,
    val errorMessage: String? = null,
    val isLoading: Boolean = false,
    val creationProgress: Float = 0f,
    val createdArchivePath: String? = null
)

/**
 * Steps in the Create Archive wizard
 */
enum class CreateArchiveStep {
    SelectDestination,
    ArchiveName,
    CreatePassphrase,
    ConfirmPassphrase,
    Creating,
    Success
}
