package com.ziplock.viewmodel

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import android.util.Log
import com.ziplock.config.AndroidConfigManager
import com.ziplock.repository.MobileRepositoryManager

/**
 * Repository View Model - Unified Architecture
 *
 * Updated to use the unified architecture pattern:
 * - Uses MobileRepositoryManager for all repository operations
 * - Follows the separation of concerns between mobile FFI and Android file I/O
 * - Provides a clean interface for the UI layer
 * - Handles archive file operations via Storage Access Framework (SAF)
 * - Delegates memory operations to the shared library via mobile FFI
 *
 * This view model serves as the bridge between the UI and the unified architecture,
 * handling all repository-related operations through the proper abstractions.
 */
class RepositoryViewModel(private val context: Context) : ViewModel() {

    companion object {
        private const val TAG = "RepositoryViewModel"
    }

    // Dependencies
    private val configManager: AndroidConfigManager = AndroidConfigManager(context)
    private val repositoryManager: MobileRepositoryManager = MobileRepositoryManager.getInstance(context)

    // UI State
    private val _uiState = MutableStateFlow(RepositoryUiState())
    val uiState: StateFlow<RepositoryUiState> = _uiState.asStateFlow()

    // Repository State
    private val _repositoryState = MutableStateFlow<MobileRepositoryManager.RepositoryState?>(null)
    val repositoryState: StateFlow<MobileRepositoryManager.RepositoryState?> = _repositoryState.asStateFlow()

    // Expose config manager's last archive path
    val lastArchivePath: StateFlow<String?> = configManager.lastArchivePath

    init {
        // Initialize repository manager
        viewModelScope.launch {
            try {
                val initialized = repositoryManager.initialize()
                if (!initialized) {
                    Log.w(TAG, "Repository manager initialization failed")
                    _uiState.value = _uiState.value.copy(
                        errorMessage = "Failed to initialize repository manager"
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error initializing repository manager", e)
                _uiState.value = _uiState.value.copy(
                    errorMessage = "Initialization error: ${e.message}"
                )
            }
        }
    }

    /**
     * Get the last opened archive path if it still exists
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
     * Open an existing archive using the unified architecture
     *
     * @param archiveUri URI to the .7z archive file (typically from SAF)
     * @param passphrase User-provided passphrase for decryption
     */
    fun openRepository(archiveUri: Uri, passphrase: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null
            )

            try {
                Log.d(TAG, "Opening repository: $archiveUri")

                // Validate inputs
                if (passphrase.isBlank()) {
                    throw IllegalArgumentException("Passphrase is required")
                }

                // Use repository manager to open the archive
                val result = repositoryManager.openRepository(archiveUri, passphrase)

                when (result) {
                    is MobileRepositoryManager.RepositoryResult.Success -> {
                        val repositoryState = result.data

                        // Update UI state
                        _repositoryState.value = repositoryState

                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            errorMessage = null
                        )

                        // Update config manager with successful open
                        configManager.setLastArchivePath(archiveUri.toString())

                        Log.d(TAG, "Repository opened successfully: ${repositoryState.credentialCount} credentials")
                    }

                    is MobileRepositoryManager.RepositoryResult.Error<*> -> {
                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            errorMessage = "Failed to open archive: ${result.message}"
                        )

                        _repositoryState.value = MobileRepositoryManager.RepositoryState(
                            isOpen = false,
                            isModified = false,
                            credentialCount = 0
                        )

                        Log.e(TAG, "Failed to open repository: ${result.message}", result.exception)
                    }
                }

            } catch (e: Exception) {
                val errorMessage = "Error opening repository: ${e.message}"
                Log.e(TAG, errorMessage, e)

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = errorMessage
                )

                _repositoryState.value = MobileRepositoryManager.RepositoryState(
                    isOpen = false,
                    isModified = false,
                    credentialCount = 0
                )
            }
        }
    }

    /**
     * Open repository with file path (legacy compatibility)
     */
    fun openRepository(filePath: String, passphrase: String) {
        try {
            val uri = if (filePath.startsWith("content://")) {
                Uri.parse(filePath)
            } else {
                Uri.fromFile(java.io.File(filePath))
            }
            openRepository(uri, passphrase)
        } catch (e: Exception) {
            Log.e(TAG, "Error converting file path to URI: $filePath", e)
            _uiState.value = _uiState.value.copy(
                errorMessage = "Invalid file path: ${e.message}"
            )
        }
    }

    /**
     * Create a new archive
     *
     * @param archiveUri URI where the new archive should be saved
     * @param passphrase Passphrase to encrypt the new archive
     */
    fun createRepository(archiveUri: Uri, passphrase: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null
            )

            try {
                Log.d(TAG, "Creating new repository: $archiveUri")

                // Validate inputs
                if (passphrase.isBlank()) {
                    throw IllegalArgumentException("Passphrase is required")
                }

                // Use repository manager to create the archive
                val result = repositoryManager.createRepository(archiveUri, passphrase)

                when (result) {
                    is MobileRepositoryManager.RepositoryResult.Success<*> -> {
                        val repositoryState = result.data as MobileRepositoryManager.RepositoryState

                        _repositoryState.value = repositoryState

                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            errorMessage = null
                        )

                        // Update config manager with successful open
                        configManager.setLastArchivePath(archiveUri.toString())

                        Log.d(TAG, "Repository created successfully")
                    }

                    is MobileRepositoryManager.RepositoryResult.Error<*> -> {
                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            errorMessage = "Failed to create archive: ${result.message}"
                        )

                        _repositoryState.value = MobileRepositoryManager.RepositoryState(
                            isOpen = false,
                            isModified = false,
                            credentialCount = 0
                        )

                        Log.e(TAG, "Failed to create repository: ${result.message}", result.exception)
                    }
                }

            } catch (e: Exception) {
                val errorMessage = "Error creating repository: ${e.message}"
                Log.e(TAG, errorMessage, e)

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = errorMessage
                )

                _repositoryState.value = MobileRepositoryManager.RepositoryState(
                    isOpen = false,
                    isModified = false,
                    credentialCount = 0
                )
            }
        }
    }

    /**
     * Save the current repository
     */
    fun saveRepository() {
        viewModelScope.launch {
            try {
                Log.d(TAG, "Saving repository")

                val result = repositoryManager.saveRepository()

                when (result) {
                    is MobileRepositoryManager.RepositoryResult.Success<*> -> {
                        // Update repository state to reflect saved status
                        val currentState = _repositoryState.value
                        if (currentState != null && currentState.isOpen) {
                            _repositoryState.value = currentState.copy(
                                isModified = false
                            )
                        }

                        Log.d(TAG, "Repository saved successfully")
                    }

                    is MobileRepositoryManager.RepositoryResult.Error -> {
                        _uiState.value = _uiState.value.copy(
                            errorMessage = "Failed to save repository: ${result.message}"
                        )

                        Log.e(TAG, "Failed to save repository: ${result.message}", result.exception)
                    }
                }

            } catch (e: Exception) {
                val errorMessage = "Error saving repository: ${e.message}"
                Log.e(TAG, errorMessage, e)

                _uiState.value = _uiState.value.copy(
                    errorMessage = errorMessage
                )
            }
        }
    }

    /**
     * Close the current repository
     */
    fun closeRepository() {
        viewModelScope.launch {
            try {
                Log.d(TAG, "Closing repository")

                repositoryManager.closeRepository()

                _repositoryState.value = MobileRepositoryManager.RepositoryState(
                    isOpen = false,
                    isModified = false,
                    credentialCount = 0
                )

                // Clear any error messages
                _uiState.value = _uiState.value.copy(
                    errorMessage = null
                )

                Log.d(TAG, "Repository closed")

            } catch (e: Exception) {
                Log.e(TAG, "Error closing repository", e)
            }
        }
    }

    /**
     * Check if repository is currently open
     */
    fun isRepositoryOpen(): Boolean {
        return _repositoryState.value?.isOpen ?: false
    }

    /**
     * Check if repository has unsaved changes
     */
    fun isRepositoryModified(): Boolean {
        val state = _repositoryState.value
        return state?.isOpen == true && state.isModified
    }

    /**
     * Get current repository statistics
     */
    suspend fun getRepositoryStats(): MobileRepositoryManager.RepositoryState? {
        return try {
            val result = repositoryManager.getRepositoryState()

            if (result is MobileRepositoryManager.RepositoryResult.Success<*>) {
                val data = result.data
                if (data is MobileRepositoryManager.RepositoryState) {
                    data
                } else {
                    null
                }
            } else if (result is MobileRepositoryManager.RepositoryResult.Error<*>) {
                Log.e(TAG, "Failed to get repository stats: ${result.message}")
                null
            } else {
                Log.w(TAG, "Unknown repository result type")
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting repository stats", e)
            null
        }
    }

    /**
     * Clear any error messages
     */
    fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Refresh repository state
     */
    fun refreshRepositoryState() {
        viewModelScope.launch {
            try {
                val stats = getRepositoryStats()
                if (stats != null && stats.isOpen) {
                    val currentState = _repositoryState.value
                    if (currentState != null && currentState.isOpen) {
                        _repositoryState.value = currentState.copy(
                            credentialCount = stats.credentialCount,
                            isModified = stats.isModified
                        )
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error refreshing repository state", e)
            }
        }
    }

    override fun onCleared() {
        super.onCleared()
        // Clean up any resources
        try {
            repositoryManager.closeRepository()
        } catch (e: Exception) {
            Log.e(TAG, "Error during cleanup", e)
        }
    }
}

/**
 * UI State for repository operations
 */
data class RepositoryUiState(
    val isLoading: Boolean = false,
    val errorMessage: String? = null
)

/**
 * Factory for creating RepositoryViewModel instances
 */
class RepositoryViewModelFactory(
    private val context: Context
) : ViewModelProvider.Factory {
    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(RepositoryViewModel::class.java)) {
            return RepositoryViewModel(context) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class")
    }
}
