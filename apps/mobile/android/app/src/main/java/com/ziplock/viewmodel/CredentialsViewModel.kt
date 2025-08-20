package com.ziplock.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope

import com.ziplock.ffi.ZipLockNative
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay

/**
 * ViewModel for managing credentials list state and operations
 */
class CredentialsViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(CredentialsUiState())
    val uiState: StateFlow<CredentialsUiState> = _uiState.asStateFlow()

    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    private val _archiveOpen = MutableStateFlow(false)
    val archiveOpen: StateFlow<Boolean> = _archiveOpen.asStateFlow()

    init {
        println("CredentialsViewModel: Initializing...")
        // Check initial state
        val initiallyOpen = ZipLockNative.isArchiveOpen()
        println("CredentialsViewModel: Initial archive open status: $initiallyOpen")
        _archiveOpen.value = initiallyOpen
        // Note: loadCredentials() is now called externally when archive is confirmed open
        // This prevents race conditions where the UI loads before the archive is fully opened
    }

    /**
     * Test FFI connection
     */
    private fun testFFIConnection(): Boolean {
        return try {
            println("CredentialsViewModel: Testing FFI connection...")
            val testResult = ZipLockNative.testConnection()
            println("CredentialsViewModel: FFI test result: $testResult")

            val version = ZipLockNative.getVersion()
            println("CredentialsViewModel: FFI library version: $version")

            testResult
        } catch (e: Exception) {
            println("CredentialsViewModel: FFI test failed: ${e.message}")
            e.printStackTrace()
            false
        }
    }

    /**
     * Load credentials from the archive
     */
    fun loadCredentials() {
        println("CredentialsViewModel: loadCredentials() called")
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null
            )

            try {
                // Test FFI connection first
                if (!testFFIConnection()) {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        errorMessage = "FFI library connection failed. Please restart the app."
                    )
                    return@launch
                }

                // Add a small delay to show loading state for better UX
                delay(500)

                // Check if archive is open first
                val isOpen = ZipLockNative.isArchiveOpen()
                println("CredentialsViewModel: Archive is open: $isOpen")
                _archiveOpen.value = isOpen

                if (!isOpen) {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        errorMessage = "Archive is not open. Please open an archive first."
                    )
                    return@launch
                }

                println("CredentialsViewModel: Calling listCredentials...")
                val result = ZipLockNative.listCredentials()
                println("CredentialsViewModel: listCredentials result - success: ${result.success}, credentials: ${result.credentials.size}, error: ${result.errorMessage}")

                if (result.success) {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        credentials = result.credentials,
                        errorMessage = null
                    )
                    println("CredentialsViewModel: Successfully loaded ${result.credentials.size} credentials")
                } else {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        errorMessage = result.errorMessage ?: "Failed to load credentials"
                    )
                    println("CredentialsViewModel: Failed to load credentials: ${result.errorMessage}")
                }
            } catch (e: Exception) {
                println("CredentialsViewModel: Exception loading credentials: ${e.message}")
                e.printStackTrace()
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = "Error loading credentials: ${e.message}"
                )
            }
        }
    }

    /**
     * Load mock credentials for testing UI
     * This can be used during development to test the credentials list UI
     */
    fun loadMockCredentials() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                errorMessage = null
            )

            try {
                delay(1000) // Simulate network/disk delay

                val mockCredentials = listOf(
                    ZipLockNative.Credential(
                        id = "cred_1",
                        title = "Google Account",
                        credentialType = "login",
                        url = "https://accounts.google.com",
                        username = "user@gmail.com",
                        tags = listOf("work", "email")
                    ),
                    ZipLockNative.Credential(
                        id = "cred_2",
                        title = "Bank of America",
                        credentialType = "bank_account",
                        url = "https://bankofamerica.com",
                        tags = listOf("finance", "bank")
                    ),
                    ZipLockNative.Credential(
                        id = "cred_3",
                        title = "Visa Credit Card",
                        credentialType = "credit_card",
                        tags = listOf("finance", "payment")
                    ),
                    ZipLockNative.Credential(
                        id = "cred_4",
                        title = "WiFi Password",
                        credentialType = "secure_note",
                        notes = "Home network credentials",
                        tags = listOf("home", "network")
                    ),
                    ZipLockNative.Credential(
                        id = "cred_5",
                        title = "SSH Server Key",
                        credentialType = "ssh_key",
                        url = "192.168.1.100",
                        username = "admin",
                        tags = listOf("server", "development")
                    )
                )

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    credentials = mockCredentials,
                    errorMessage = null
                )
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = "Error loading mock credentials: ${e.message}"
                )
            }
        }
    }

    /**
     * Update search query
     */
    fun updateSearchQuery(query: String) {
        _searchQuery.value = query
    }

    /**
     * Clear search query
     */
    fun clearSearch() {
        _searchQuery.value = ""
    }

    /**
     * Refresh credentials list
     */
    fun refresh() {
        loadCredentials()
    }

    /**
     * Handle add credential action
     */
    fun addCredential() {
        // TODO: Navigate to add credential screen
        println("Add credential requested")
    }

    /**
     * Close the current archive
     */
    fun closeArchive(): Boolean {
        return try {
            println("CredentialsViewModel: Closing archive...")

            // Check if an archive is actually open before trying to close
            val isOpen = ZipLockNative.isArchiveOpen()
            if (!isOpen) {
                println("CredentialsViewModel: No archive is open, clearing UI state")
                // Clear credentials UI state even if no archive was open
                _uiState.value = _uiState.value.copy(
                    credentials = emptyList(),
                    errorMessage = null
                )
                _archiveOpen.value = false
                return true // Return success since the desired state (no open archive) is achieved
            }

            val result = ZipLockNative.closeArchive()
            println("CredentialsViewModel: Close archive result: $result")

            if (result) {
                // Clear credentials when archive is closed
                _uiState.value = _uiState.value.copy(
                    credentials = emptyList(),
                    errorMessage = null
                )
                _archiveOpen.value = false
                println("CredentialsViewModel: Archive closed successfully, cleared credentials")
            } else {
                println("CredentialsViewModel: Failed to close archive")
            }
            result
        } catch (e: Exception) {
            println("CredentialsViewModel: Exception closing archive: ${e.message}")
            e.printStackTrace()
            _uiState.value = _uiState.value.copy(
                errorMessage = "Error closing archive: ${e.message}"
            )
            false
        }
    }

    /**
     * Check if archive is currently open
     */
    fun isArchiveOpen(): Boolean {
        val isOpen = ZipLockNative.isArchiveOpen()
        println("CredentialsViewModel: isArchiveOpen() = $isOpen")
        return isOpen
    }

    /**
     * Handle credential selection
     */
    fun addCredential(credential: ZipLockNative.Credential) {
        // For now, just log the selection
        // TODO: Navigate to credential detail view
        println("Selected credential: ${credential.title} (${credential.id})")
    }

    /**
     * Clear any error messages
     */
    fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Clear credentials state for cleanup
     */
    fun clearCredentials() {
        _uiState.value = _uiState.value.copy(
            credentials = emptyList(),
            errorMessage = null,
            isLoading = false
        )
        _archiveOpen.value = false
        println("CredentialsViewModel: Cleared credentials state")
    }

    /**
     * Get filtered credentials based on search query
     */
    fun getFilteredCredentials(): List<ZipLockNative.Credential> {
        val query = _searchQuery.value
        val credentials = _uiState.value.credentials

        return if (query.isBlank()) {
            credentials
        } else {
            credentials.filter { credential: ZipLockNative.Credential ->
                credential.title.contains(query, ignoreCase = true) ||
                credential.credentialType.contains(query, ignoreCase = true) ||
                credential.username.contains(query, ignoreCase = true) ||
                credential.url.contains(query, ignoreCase = true) ||
                credential.notes.contains(query, ignoreCase = true) ||
                credential.tags.any { tag: String -> tag.contains(query, ignoreCase = true) }
            }
        }
    }

    /**
     * Get credentials grouped by type for potential future use
     */
    fun getCredentialsGroupedByType(): Map<String, List<ZipLockNative.Credential>> {
        return _uiState.value.credentials.groupBy { it.credentialType }
    }

    /**
     * Get unique credential types for potential filtering
     */
    fun getCredentialTypes(): List<String> {
        return _uiState.value.credentials
            .map { it.credentialType }
            .distinct()
            .sorted()
    }

    /**
     * Get all unique tags for potential filtering
     */
    fun getAllTags(): List<String> {
        return _uiState.value.credentials
            .flatMap { credential: ZipLockNative.Credential -> credential.tags }
            .distinct()
            .sorted()
    }

    /**
     * Get credentials statistics
     */
    fun getCredentialsStats(): CredentialsStats {
        val credentials = _uiState.value.credentials
        val typeGroups = credentials.groupBy { credential: ZipLockNative.Credential -> credential.credentialType }

        return CredentialsStats(
            totalCount = credentials.size,
            typeCount = typeGroups.size,
            mostCommonType = typeGroups.maxByOrNull { (_, credentialList): Map.Entry<String, List<ZipLockNative.Credential>> -> credentialList.size }?.key ?: "",
            tagCount = getAllTags().size
        )
    }
}

/**
 * UI state for credentials list
 */
data class CredentialsUiState(
    val isLoading: Boolean = false,
    val credentials: List<ZipLockNative.Credential> = emptyList(),
    val errorMessage: String? = null
) {
    val isEmpty: Boolean
        get() = credentials.isEmpty() && !isLoading

    val hasError: Boolean
        get() = errorMessage != null
}

/**
 * Statistics about credentials
 */
data class CredentialsStats(
    val totalCount: Int,
    val typeCount: Int,
    val mostCommonType: String,
    val tagCount: Int
)
