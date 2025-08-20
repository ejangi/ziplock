package com.ziplock.viewmodel

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.ziplock.config.AndroidConfigManager
import com.ziplock.ffi.ZipLockDataManager
import com.ziplock.ffi.ZipLockNative
import com.ziplock.repository.HybridRepositoryManager
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.runBlocking
import java.io.File

/**
 * Hybrid Repository View Model
 *
 * This is an enhanced version of RepositoryViewModel that uses the hybrid architecture:
 * - Kotlin-based archive operations (no more sevenz_rust2 crashes)
 * - Rust-based data validation, crypto, and business logic
 * - Fallback support to old FFI approach if needed
 *
 * Key improvements:
 * - Eliminates Android emulator SIGABRT crashes
 * - Faster and more reliable testing in emulators
 * - Better Android platform integration
 * - Maintains all security and validation benefits from Rust core
 */
class HybridRepositoryViewModel(private val context: Context) : ViewModel() {

    companion object {
        private const val TAG = "HybridRepositoryViewModel"
        private const val OPERATION_TIMEOUT_MS = 30000L // 30 seconds
    }

    // Configuration manager for persistent settings
    private val configManager: AndroidConfigManager = AndroidConfigManager(context)

    // Hybrid repository manager (new approach)
    private val hybridManager = HybridRepositoryManager(context)

    // Legacy native interface (fallback)
    private val legacyNative = ZipLockNative

    // UI State
    private val _uiState = MutableStateFlow(HybridRepositoryUiState())
    val uiState: StateFlow<HybridRepositoryUiState> = _uiState.asStateFlow()

    // Repository State
    private val _repositoryState = MutableStateFlow<HybridRepositoryState>(HybridRepositoryState.None)
    val repositoryState: StateFlow<HybridRepositoryState> = _repositoryState.asStateFlow()

    // Expose config manager's last archive path
    val lastArchivePath: StateFlow<String?> = configManager.lastArchivePath

    // Architecture mode
    private val _architectureMode = MutableStateFlow(ArchitectureMode.HYBRID)
    val architectureMode: StateFlow<ArchitectureMode> = _architectureMode.asStateFlow()

    init {
        initializeHybridManager()
    }

    /**
     * Initialize the hybrid repository manager
     */
    private fun initializeHybridManager() {
        viewModelScope.launch {
            try {
                Log.i(TAG, "Initializing hybrid repository manager...")

                val result = hybridManager.initialize()
                if (result.success) {
                    Log.i(TAG, "Hybrid repository manager initialized successfully")
                    _uiState.value = _uiState.value.copy(
                        isInitialized = true,
                        initializationMessage = "✓ Hybrid architecture ready"
                    )
                } else {
                    Log.w(TAG, "Hybrid initialization failed, falling back to legacy: ${result.errorMessage}")
                    _architectureMode.value = ArchitectureMode.LEGACY_FALLBACK
                    _uiState.value = _uiState.value.copy(
                        isInitialized = true,
                        initializationMessage = "⚠ Using legacy mode: ${result.errorMessage}"
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to initialize hybrid manager", e)
                _architectureMode.value = ArchitectureMode.LEGACY_FALLBACK
                _uiState.value = _uiState.value.copy(
                    isInitialized = true,
                    initializationMessage = "⚠ Fallback mode: ${e.message}"
                )
            }
        }
    }

    /**
     * Get the last opened archive path if it still exists
     */
    fun getLastArchivePathIfExists(): String? {
        return configManager.getLastOpenedArchivePath()?.let { path ->
            path
        }
    }

    /**
     * Open repository using hybrid approach with fallback
     */
    fun openRepository(archivePath: String, masterPassword: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null,
                operationStatus = "Opening repository..."
            )

            try {
                val result = when (_architectureMode.value) {
                    ArchitectureMode.HYBRID -> openRepositoryHybrid(archivePath, masterPassword)
                    ArchitectureMode.LEGACY_FALLBACK -> openRepositoryLegacy(archivePath, masterPassword)
                }

                if (result.success) {
                    configManager.setLastArchivePath(archivePath)
                    _repositoryState.value = HybridRepositoryState.Open(
                        path = archivePath,
                        metadata = result.metadata,
                        mode = _architectureMode.value
                    )
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        operationStatus = "Repository opened successfully"
                    )
                    Log.i(TAG, "Repository opened successfully: $archivePath")
                } else {
                    // Log detailed error information for debugging
                    Log.e(TAG, "Failed to open repository: ${result.errorMessage}")
                    Log.e(TAG, "Architecture mode: ${_architectureMode.value}")
                    Log.e(TAG, "Archive path: $archivePath")

                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        errorMessage = "Archive opening failed: ${result.errorMessage}\n\nNote: Legacy FFI disabled to prevent emulator crashes.",
                        operationStatus = "Failed to open repository"
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception during repository open", e)
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = "Unexpected error: ${e.message}",
                    operationStatus = "Operation failed"
                )
            }
        }
    }

    /**
     * Open repository using hybrid architecture
     */
    private suspend fun openRepositoryHybrid(
        archivePath: String,
        masterPassword: String
    ): RepositoryOpenResult {
        Log.i(TAG, "Opening repository with hybrid architecture: $archivePath")

        return withTimeoutOrNull(OPERATION_TIMEOUT_MS) {
            val result = hybridManager.openRepository(archivePath, masterPassword)
            if (result.success) {
                // Register the repository manager with ZipLockNative for credential persistence
                ZipLockNative.setRepositoryManager(hybridManager)
                Log.i(TAG, "Repository manager registered with ZipLockNative for credential persistence")

                RepositoryOpenResult(
                    success = true,
                    metadata = result.data?.let { RepositoryMetadata.fromHybrid(it) }
                )
            } else {
                // DO NOT fallback to legacy FFI - it causes SIGABRT crashes in emulator
                Log.w(TAG, "Hybrid approach failed, but NOT falling back to legacy to prevent crash")
                Log.e(TAG, "Hybrid repository opening failed: ${result.errorMessage}")
                RepositoryOpenResult(
                    success = false,
                    errorMessage = "Hybrid archive opening failed: ${result.errorMessage ?: "Unknown error"}"
                )
            }
        } ?: RepositoryOpenResult(
            success = false,
            errorMessage = "Operation timed out (${OPERATION_TIMEOUT_MS}ms)"
        )
    }

    /**
     * Open repository using legacy FFI approach
     */
    private suspend fun openRepositoryLegacy(
        archivePath: String,
        masterPassword: String
    ): RepositoryOpenResult {
        Log.i(TAG, "Opening repository with legacy FFI: $archivePath")

        return withTimeoutOrNull(OPERATION_TIMEOUT_MS) {
            try {
                // Pre-validation
                val file = File(archivePath)
                if (!file.exists()) {
                    return@withTimeoutOrNull RepositoryOpenResult(
                        success = false,
                        errorMessage = "Archive file not found: $archivePath"
                    )
                }

                if (file.length() < 32) {
                    return@withTimeoutOrNull RepositoryOpenResult(
                        success = false,
                        errorMessage = "File too small to be valid archive"
                    )
                }

                // Attempt legacy open
                val legacyResult = legacyNative.openArchive(archivePath, masterPassword)
                if (legacyResult.success) {
                    RepositoryOpenResult(
                        success = true,
                        metadata = RepositoryMetadata.createLegacy()
                    )
                } else {
                    val errorMsg = try {
                        legacyNative.getLastError()
                    } catch (e: Exception) {
                        "Legacy FFI operation failed: ${e.message}"
                    }
                    RepositoryOpenResult(
                        success = false,
                        errorMessage = errorMsg ?: "Unknown legacy FFI error"
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Legacy FFI exception", e)
                RepositoryOpenResult(
                    success = false,
                    errorMessage = "Legacy operation crashed: ${e.message}"
                )
            }
        } ?: RepositoryOpenResult(
            success = false,
            errorMessage = "Legacy operation timed out"
        )
    }

    /**
     * Create new repository using hybrid approach
     */
    fun createRepository(archivePath: String, masterPassword: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null,
                operationStatus = "Creating repository..."
            )

            try {
                val result = when (_architectureMode.value) {
                    ArchitectureMode.HYBRID -> createRepositoryHybrid(archivePath, masterPassword)
                    ArchitectureMode.LEGACY_FALLBACK -> createRepositoryLegacy(archivePath, masterPassword)
                }

                if (result.success) {
                    configManager.setLastArchivePath(archivePath)
                    _repositoryState.value = HybridRepositoryState.Open(
                        path = archivePath,
                        metadata = result.metadata,
                        mode = _architectureMode.value
                    )
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        operationStatus = "Repository created successfully"
                    )
                    Log.i(TAG, "Repository created successfully: $archivePath")
                } else {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        errorMessage = result.errorMessage,
                        operationStatus = "Failed to create repository"
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Exception during repository creation", e)
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = "Unexpected error: ${e.message}",
                    operationStatus = "Operation failed"
                )
            }
        }
    }

    /**
     * Create repository using hybrid architecture
     */
    private suspend fun createRepositoryHybrid(
        archivePath: String,
        masterPassword: String
    ): RepositoryOpenResult {
        Log.i(TAG, "Creating repository with hybrid architecture: $archivePath")

        return withTimeoutOrNull(OPERATION_TIMEOUT_MS) {
            val result = hybridManager.createRepository(archivePath, masterPassword)
            if (result.success) {
                RepositoryOpenResult(
                    success = true,
                    metadata = RepositoryMetadata.createNew()
                )
            } else {
                RepositoryOpenResult(
                    success = false,
                    errorMessage = result.errorMessage
                )
            }
        } ?: RepositoryOpenResult(
            success = false,
            errorMessage = "Create operation timed out"
        )
    }

    /**
     * Create repository using legacy FFI approach
     */
    private suspend fun createRepositoryLegacy(
        archivePath: String,
        masterPassword: String
    ): RepositoryOpenResult {
        Log.i(TAG, "Creating repository with legacy FFI: $archivePath")

        return withTimeoutOrNull(OPERATION_TIMEOUT_MS) {
            try {
                val legacyResult = legacyNative.createArchive(archivePath, masterPassword)
                if (legacyResult.success) {
                    RepositoryOpenResult(
                        success = true,
                        metadata = RepositoryMetadata.createLegacy()
                    )
                } else {
                    RepositoryOpenResult(
                        success = false,
                        errorMessage = legacyNative.getLastError() ?: "Unknown create error"
                    )
                }
            } catch (e: Exception) {
                RepositoryOpenResult(
                    success = false,
                    errorMessage = "Legacy create failed: ${e.message}"
                )
            }
        } ?: RepositoryOpenResult(
            success = false,
            errorMessage = "Legacy create timed out"
        )
    }

    /**
     * Close the current repository
     */
    fun closeRepository() {
        viewModelScope.launch {
            try {
                when (_architectureMode.value) {
                    ArchitectureMode.HYBRID -> {
                        hybridManager.closeRepository()
                        // Unregister repository manager from ZipLockNative
                        ZipLockNative.setRepositoryManager(null)
                        Log.i(TAG, "Repository manager unregistered from ZipLockNative")
                    }
                    ArchitectureMode.LEGACY_FALLBACK -> {
                        legacyNative.closeArchive()
                    }
                }

                _repositoryState.value = HybridRepositoryState.None
                _uiState.value = _uiState.value.copy(
                    operationStatus = "Repository closed"
                )
                Log.i(TAG, "Repository closed")
            } catch (e: Exception) {
                Log.e(TAG, "Error closing repository", e)
                _uiState.value = _uiState.value.copy(
                    errorMessage = "Failed to close repository: ${e.message}"
                )
            }
        }
    }

    /**
     * Test connectivity to both hybrid and legacy systems
     */
    fun testConnectivity() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                operationStatus = "Testing connectivity..."
            )

            try {
                val hybridResult = hybridManager.testConnectivity()
                val legacyResult = try {
                    val success = legacyNative.testConnection()
                    if (success) "Legacy test passed" else "Legacy test failed"
                } catch (e: Exception) {
                    "Legacy test failed: ${e.message}"
                }

                val status = buildString {
                    appendLine("🔧 Connectivity Test Results:")
                    appendLine()
                    appendLine("Hybrid Architecture:")
                    if (hybridResult.success) {
                        appendLine("✅ ${hybridResult.data}")
                    } else {
                        appendLine("❌ ${hybridResult.errorMessage}")
                    }
                    appendLine()
                    appendLine("Legacy FFI:")
                    if (legacyResult == "Legacy test") {
                        appendLine("✅ Legacy FFI: OK")
                    } else {
                        appendLine("❌ Legacy FFI: $legacyResult")
                    }
                    appendLine()
                    appendLine("Current Mode: ${_architectureMode.value}")
                }

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    operationStatus = status.trim()
                )
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = "Connectivity test failed: ${e.message}"
                )
            }
        }
    }

    /**
     * Switch between hybrid and legacy modes (for testing)
     */
    fun switchArchitectureMode(mode: ArchitectureMode) {
        _architectureMode.value = mode
        Log.i(TAG, "Switched to architecture mode: $mode")
        _uiState.value = _uiState.value.copy(
            operationStatus = "Switched to ${mode.displayName}"
        )
    }

    /**
     * Get detailed repository information
     */
    fun getRepositoryInfo() {
        val state = _repositoryState.value
        if (state !is HybridRepositoryState.Open) {
            _uiState.value = _uiState.value.copy(
                operationStatus = "No repository is open"
            )
            return
        }

        viewModelScope.launch {
            try {
                val info = when (_architectureMode.value) {
                    ArchitectureMode.HYBRID -> {
                        val result = hybridManager.getRepositoryInfo()
                        if (result.success) {
                            result.data?.entries?.joinToString("\n") { "${it.key}: ${it.value}" }
                        } else {
                            "Failed to get hybrid info: ${result.errorMessage}"
                        }
                    }
                    ArchitectureMode.LEGACY_FALLBACK -> {
                        "Legacy mode - limited info available\nPath: ${state.path}"
                    }
                }

                _uiState.value = _uiState.value.copy(
                    operationStatus = "Repository Info:\n$info"
                )
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    errorMessage = "Failed to get repository info: ${e.message}"
                )
            }
        }
    }

    /**
     * Clear error message
     */
    fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Clear operation status
     */
    fun clearStatus() {
        _uiState.value = _uiState.value.copy(operationStatus = null)
    }

    // Data classes for state management
    data class HybridRepositoryUiState(
        val isLoading: Boolean = false,
        val isInitialized: Boolean = false,
        val errorMessage: String? = null,
        val operationStatus: String? = null,
        val initializationMessage: String? = null
    )

    sealed class HybridRepositoryState {
        object None : HybridRepositoryState()
        data class Open(
            val path: String,
            val metadata: RepositoryMetadata?,
            val mode: ArchitectureMode
        ) : HybridRepositoryState()
    }

    data class RepositoryOpenResult(
        val success: Boolean,
        val errorMessage: String? = null,
        val metadata: RepositoryMetadata? = null
    )

    data class RepositoryMetadata(
        val version: String,
        val createdAt: Long,
        val lastModified: Long,
        val credentialCount: Int,
        val format: String
    ) {
        companion object {
            fun fromHybrid(hybridMetadata: HybridRepositoryManager.RepositoryMetadata): RepositoryMetadata {
                return RepositoryMetadata(
                    version = hybridMetadata.version,
                    createdAt = hybridMetadata.createdAt,
                    lastModified = hybridMetadata.lastModified,
                    credentialCount = hybridMetadata.credentialCount,
                    format = hybridMetadata.format
                )
            }

            fun createNew(): RepositoryMetadata {
                val now = System.currentTimeMillis()
                return RepositoryMetadata(
                    version = "1.0",
                    createdAt = now,
                    lastModified = now,
                    credentialCount = 0,
                    format = "hybrid-v1"
                )
            }

            fun createLegacy(): RepositoryMetadata {
                val now = System.currentTimeMillis()
                return RepositoryMetadata(
                    version = "1.0",
                    createdAt = now,
                    lastModified = now,
                    credentialCount = 0,
                    format = "legacy-ffi"
                )
            }
        }
    }

    enum class ArchitectureMode(val displayName: String) {
        HYBRID("Hybrid (Kotlin + Rust)"),
        LEGACY_FALLBACK("Legacy FFI Only")
    }

    /**
     * Clean up resources when ViewModel is destroyed
     * This ensures archives are properly closed when the app exits
     */
    override fun onCleared() {
        super.onCleared()
        try {
            Log.d(TAG, "ViewModel being cleared, closing repository if open...")

            // Check if repository is currently open
            val currentState = _repositoryState.value
            if (currentState is HybridRepositoryState.Open) {
                // Close repository synchronously in onCleared to ensure cleanup
                when (_architectureMode.value) {
                    ArchitectureMode.HYBRID -> {
                        runBlocking {
                            hybridManager.closeRepository()
                        }
                        // Unregister repository manager from ZipLockNative
                        ZipLockNative.setRepositoryManager(null)
                    }
                    ArchitectureMode.LEGACY_FALLBACK -> {
                        legacyNative.closeArchive()
                    }
                }
                Log.i(TAG, "Repository closed during ViewModel cleanup")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error during ViewModel cleanup", e)
        }
    }

    // Factory for dependency injection
    class Factory(private val context: Context) : ViewModelProvider.Factory {
        @Suppress("UNCHECKED_CAST")
        override fun <T : ViewModel> create(modelClass: Class<T>): T {
            if (modelClass.isAssignableFrom(HybridRepositoryViewModel::class.java)) {
                return HybridRepositoryViewModel(context) as T
            }
            throw IllegalArgumentException("Unknown ViewModel class")
        }
    }
}
