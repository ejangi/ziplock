package com.ziplock.viewmodel

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ziplock.ffi.ZipLockNative
import com.ziplock.ffi.ZipLockNativeHelper
import com.ziplock.ffi.PassphraseStrengthResult
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
                    val strength = ZipLockNative.validatePassphraseStrength(passphrase)
                    _passphraseStrength.value = strength
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

                // Create the archive using the FFI library with the real file path
                val result = ZipLockNative.createArchive(archiveInfo.workingPath, passphrase)
                Log.d("CreateArchiveViewModel", "Archive creation result: success=${result.success}, error=${result.errorMessage}")

                updateProgress(0.5f)

                if (result.success) {
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
                } else {
                    // Map specific error codes to better messages
                    val userMessage = when (result.errorMessage) {
                        "Internal error" -> "Failed to create archive. The selected location may not be writable or the filename may be invalid."
                        "Permission denied" -> "Permission denied. Please check that you have write access to the selected folder."
                        "Invalid parameter provided" -> "Invalid archive name or destination. Please check your inputs and try again."
                        else -> result.errorMessage ?: "Failed to create archive"
                    }
                    throw Exception(userMessage)
                }

            } catch (e: Exception) {
                // Clean up working file if creation failed
                archiveInfo?.let { info ->
                    try {
                        java.io.File(info.workingPath).delete()
                    } catch (cleanupException: Exception) {
                        // Ignore cleanup errors
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
            ZipLockNativeHelper.validateLibrary()
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
