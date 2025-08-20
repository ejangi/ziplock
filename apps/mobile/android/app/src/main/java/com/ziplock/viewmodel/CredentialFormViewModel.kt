package com.ziplock.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ziplock.ffi.ZipLockNative
import com.ziplock.ffi.ZipLockDataManager
import com.ziplock.repository.HybridRepositoryManager

import com.ziplock.ffi.ZipLockNativeHelper
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay
import android.util.Log

/**
 * ViewModel for managing credential form state and operations
 */
class CredentialFormViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(CredentialFormUiState())
    val uiState: StateFlow<CredentialFormUiState> = _uiState.asStateFlow()

    /**
     * Save a new credential
     */
    fun saveCredential(
        template: ZipLockNativeHelper.CredentialTemplate,
        title: String,
        fields: Map<String, String>,
        tags: List<String>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit
    ) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isSaving = true,
                errorMessage = null
            )

            try {
                // Add a small delay to show saving state
                delay(300)

                // Save credential using hybrid architecture
                println("CredentialFormViewModel: Attempting to save credential: $title")
                val saveResult = saveCredentialToArchive(template, title, fields, tags)

                if (saveResult.success) {
                    _uiState.value = _uiState.value.copy(
                        isSaving = false,
                        errorMessage = null
                    )
                    println("CredentialFormViewModel: Successfully saved credential: $title")
                    onSuccess()
                } else {
                    _uiState.value = _uiState.value.copy(
                        isSaving = false,
                        errorMessage = saveResult.errorMessage ?: "Failed to save credential"
                    )
                    println("CredentialFormViewModel: Failed to save credential: ${saveResult.errorMessage}")
                    onError(saveResult.errorMessage ?: "Failed to save credential")
                }
            } catch (e: Exception) {
                println("CredentialFormViewModel: Exception saving credential: ${e.message}")
                e.printStackTrace()
                val errorMsg = "Error saving credential: ${e.message}"
                _uiState.value = _uiState.value.copy(
                    isSaving = false,
                    errorMessage = errorMsg
                )
                onError(errorMsg)
            }
        }
    }

    /**
     * Update an existing credential
     */
    fun updateCredential(
        credentialId: String,
        template: ZipLockNativeHelper.CredentialTemplate,
        title: String,
        fields: Map<String, String>,
        tags: List<String>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit
    ) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isSaving = true,
                errorMessage = null
            )

            try {
                // Add a small delay to show saving state
                delay(300)

                // Update credential using hybrid architecture
                val updateResult = saveCredentialToArchive(template, title, fields, tags, credentialId)

                if (updateResult.success) {
                    _uiState.value = _uiState.value.copy(
                        isSaving = false,
                        errorMessage = null
                    )
                    println("CredentialFormViewModel: Successfully updated credential: $title")
                    onSuccess()
                } else {
                    _uiState.value = _uiState.value.copy(
                        isSaving = false,
                        errorMessage = updateResult.errorMessage ?: "Failed to update credential"
                    )
                    println("CredentialFormViewModel: Failed to update credential: ${updateResult.errorMessage}")
                    onError(updateResult.errorMessage ?: "Failed to update credential")
                }
            } catch (e: Exception) {
                println("CredentialFormViewModel: Exception updating credential: ${e.message}")
                e.printStackTrace()
                val errorMsg = "Error updating credential: ${e.message}"
                _uiState.value = _uiState.value.copy(
                    isSaving = false,
                    errorMessage = errorMsg
                )
                onError(errorMsg)
            }
        }
    }

    /**
     * Validate form data before saving
     */
    fun validateForm(
        title: String,
        template: ZipLockNativeHelper.CredentialTemplate,
        fields: Map<String, String>
    ): FormValidationResult {
        val errors = mutableListOf<String>()

        // Validate title
        if (title.isBlank()) {
            errors.add("Title is required")
        }

        // Validate required fields
        for (field in template.fields) {
            if (field.required) {
                val value = fields[field.name]
                if (value.isNullOrBlank()) {
                    errors.add("${field.label} is required")
                }
            }
        }

        return FormValidationResult(
            isValid = errors.isEmpty(),
            errors = errors
        )
    }

    /**
     * Create credential object from form data
     */
    private fun createCredentialFromForm(
        template: ZipLockNativeHelper.CredentialTemplate,
        title: String,
        fields: Map<String, String>,
        tags: List<String>,
        existingId: String? = null
    ): ZipLockNative.Credential {
        // Extract common fields from the form data
        val username = fields["username"] ?: ""
        val url = fields["url"] ?: fields["website"] ?: ""
        val notes = fields["notes"] ?: fields["note"] ?: ""

        // For secure notes, the content field should be stored in notes
        val finalNotes = if (template.name == "secure_note") {
            fields["content"] ?: notes
        } else {
            notes
        }

        // Store all fields for later processing
        val allFields = fields.toMutableMap()
        allFields["title"] = title
        if (username.isNotEmpty()) allFields["username"] = username
        if (url.isNotEmpty()) allFields["url"] = url
        if (finalNotes.isNotEmpty()) allFields["notes"] = finalNotes

        return ZipLockNative.Credential(
            id = existingId ?: generateCredentialId(),
            title = title,
            credentialType = template.name,
            username = username,
            url = url,
            notes = finalNotes,
            tags = tags
        )
    }

    /**
     * Generate a unique credential ID
     */
    private fun generateCredentialId(): String {
        return "cred_${System.currentTimeMillis()}_${(1000..9999).random()}"
    }

    /**
     * Save credential using ZipLockNative saveCredential (fallback approach)
     */
    private suspend fun saveCredentialToArchive(
        template: ZipLockNativeHelper.CredentialTemplate,
        title: String,
        fields: Map<String, String>,
        tags: List<String>,
        existingId: String? = null
    ): OperationResult {
        delay(500) // Brief delay for UI feedback

        try {
            println("CredentialFormViewModel: Creating credential: $title")
            println("CredentialFormViewModel: Archive open status: ${ZipLockNative.isArchiveOpen()}")

            // Create credential object from form data
            val credential = createCredentialFromForm(
                template = template,
                title = title,
                fields = fields,
                tags = tags,
                existingId = existingId
            )

            // Try to save using ZipLockNative
            val result = ZipLockNative.saveCredential(credential)

            println("CredentialFormViewModel: Save result: $result")

            return if (result) {
                println("CredentialFormViewModel: Successfully saved credential: $title")
                OperationResult(success = true)
            } else {
                println("CredentialFormViewModel: Failed to save credential via ZipLockNative")
                OperationResult(
                    success = false,
                    errorMessage = "Failed to save credential"
                )
            }
        } catch (e: Exception) {
            println("CredentialFormViewModel: Exception saving credential: ${e.message}")
            Log.e("CredentialFormViewModel", "Exception saving credential", e)
            return OperationResult(
                success = false,
                errorMessage = "Error saving credential: ${e.message}"
            )
        }
    }

    /**
     * Simulate credential update operation
     * TODO: Replace with actual FFI call
     */
    private suspend fun updateCredentialInArchive(credential: ZipLockNative.Credential): OperationResult {
        delay(1000) // Simulate network/disk operation

        // Check if archive is open
        if (!ZipLockNative.isArchiveOpen()) {
            return OperationResult(
                success = false,
                errorMessage = "Archive is not open. Please open an archive first."
            )
        }

        // Simulate occasional failures for testing
        if (Math.random() < 0.1) { // 10% failure rate
            return OperationResult(
                success = false,
                errorMessage = "Network error: Could not update credential. Please try again."
            )
        }

        println("CredentialFormViewModel: Simulated updating credential: ${credential.title}")
        return OperationResult(success = true)
    }

    /**
     * Clear any error messages
     */
    fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Reset form state
     */
    fun resetForm() {
        _uiState.value = CredentialFormUiState()
    }

    /**
     * Check if archive is open
     */
    fun isArchiveOpen(): Boolean {
        return try {
            ZipLockNative.isArchiveOpen()
        } catch (e: Exception) {
            println("CredentialFormViewModel: Error checking archive status: ${e.message}")
            false
        }
    }
}

/**
 * UI state for credential form
 */
data class CredentialFormUiState(
    val isSaving: Boolean = false,
    val errorMessage: String? = null
) {
    val hasError: Boolean
        get() = errorMessage != null
}

/**
 * Form validation result
 */
data class FormValidationResult(
    val isValid: Boolean,
    val errors: List<String> = emptyList()
)

/**
 * Operation result for save/update operations
 */
data class OperationResult(
    val success: Boolean,
    val errorMessage: String? = null
)
