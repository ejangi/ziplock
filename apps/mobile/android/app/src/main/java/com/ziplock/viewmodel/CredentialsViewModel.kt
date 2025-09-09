package com.ziplock.viewmodel

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.ziplock.repository.MobileRepositoryManager
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * ViewModel for managing credentials list state and operations - Unified Architecture
 *
 * This ViewModel has been updated to use the unified architecture pattern:
 * - Uses MobileRepositoryManager for all repository operations
 * - Follows proper separation between UI, business logic, and data layers
 * - Handles credential CRUD operations through the mobile FFI
 * - Provides reactive state management for the UI
 * - Manages search and filtering functionality
 *
 * The ViewModel acts as a bridge between the UI layer and the repository layer,
 * handling all credential-related operations and state management.
 */
class CredentialsViewModel(private val context: Context) : ViewModel() {

    companion object {
        private const val TAG = "CredentialsViewModel"
    }

    // Dependencies
    private val repositoryManager: MobileRepositoryManager = MobileRepositoryManager.getInstance(context)

    // UI State
    private val _uiState = MutableStateFlow(CredentialsUiState())
    val uiState: StateFlow<CredentialsUiState> = _uiState.asStateFlow()

    // Search functionality
    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    // Repository state
    private val _repositoryOpen = MutableStateFlow(false)
    val repositoryOpen: StateFlow<Boolean> = _repositoryOpen.asStateFlow()

    init {
        Log.d(TAG, "Initializing CredentialsViewModel")

        // Check initial repository state
        viewModelScope.launch {
            refreshRepositoryState()
            if (_repositoryOpen.value) {
                loadCredentials()
            }
        }
    }

    /**
     * Load all credentials from the repository
     */
    fun loadCredentials() {
        viewModelScope.launch {
            try {
                Log.d(TAG, "Loading credentials")

                _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)

                val result = repositoryManager.listCredentials()

                when (result) {
                    is MobileRepositoryManager.RepositoryResult.Success -> {
                        val credentials = result.data
                        Log.d(TAG, "Loaded ${credentials.size} credentials")

                        // Convert CredentialRecord to Map<String, Any> for UI compatibility
                        val credentialMaps = credentials.map { record ->
                            mapOf(
                                "id" to record.id,
                                "title" to record.title,
                                "credentialType" to record.credentialType,
                                "fields" to record.fields.mapValues { (_, field) ->
                                    mapOf(
                                        "value" to field.value,
                                        "fieldType" to field.fieldType,
                                        "label" to field.label,
                                        "sensitive" to field.sensitive
                                    )
                                },
                                "createdAt" to record.createdAt,
                                "updatedAt" to record.updatedAt,
                                "tags" to record.tags
                            )
                        }

                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            credentials = credentialMaps,
                            errorMessage = null,
                            isEmpty = credentialMaps.isEmpty()
                        )

                        // Apply current search filter if active
                        applySearchFilter(_searchQuery.value)
                    }

                    is MobileRepositoryManager.RepositoryResult.Error -> {
                        val errorMessage = "Failed to load credentials: ${result.message}"
                        Log.e(TAG, errorMessage, result.exception)

                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            errorMessage = errorMessage,
                            credentials = emptyList(),
                            filteredCredentials = emptyList(),
                            isEmpty = true
                        )
                    }
                }

            } catch (e: Exception) {
                val errorMessage = "Error loading credentials: ${e.message}"
                Log.e(TAG, errorMessage, e)

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = errorMessage,
                    credentials = emptyList(),
                    filteredCredentials = emptyList(),
                    isEmpty = true
                )
            }
        }
    }

    /**
     * Refresh credentials list
     */
    fun refreshCredentials() {
        Log.d(TAG, "Refreshing credentials")
        loadCredentials()
    }

    /**
     * Refresh credentials list (alias for refreshCredentials)
     */
    fun refresh() {
        refreshCredentials()
    }

    /**
     * Delete a credential by ID
     */
    fun deleteCredential(credentialId: String) {
        viewModelScope.launch {
            try {
                Log.d(TAG, "Deleting credential: $credentialId")

                _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)

                val result = repositoryManager.deleteCredential(credentialId)

                when (result) {
                    is MobileRepositoryManager.RepositoryResult.Success -> {
                        Log.d(TAG, "Credential deleted successfully")

                        // Remove from current list immediately for better UX
                        val updatedCredentials = _uiState.value.credentials.filter {
                            (it["id"] as String) != credentialId
                        }

                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            credentials = updatedCredentials,
                            errorMessage = null,
                            isEmpty = updatedCredentials.isEmpty()
                        )

                        // Apply search filter to updated list
                        applySearchFilter(_searchQuery.value)
                    }

                    is MobileRepositoryManager.RepositoryResult.Error -> {
                        val errorMessage = "Failed to delete credential: ${result.message}"
                        Log.e(TAG, errorMessage, result.exception)

                        _uiState.value = _uiState.value.copy(
                            isLoading = false,
                            errorMessage = errorMessage
                        )
                    }
                }

            } catch (e: Exception) {
                val errorMessage = "Error deleting credential: ${e.message}"
                Log.e(TAG, errorMessage, e)

                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    errorMessage = errorMessage
                )
            }
        }
    }

    /**
     * Update search query and filter credentials
     */
    fun updateSearchQuery(query: String) {
        _searchQuery.value = query
        applySearchFilter(query)
    }

    /**
     * Clear search query
     */
    fun clearSearch() {
        _searchQuery.value = ""
        applySearchFilter("")
    }

    /**
     * Apply search filter to credentials
     */
    private fun applySearchFilter(query: String) {
        val credentials = _uiState.value.credentials

        val filtered = if (query.isBlank()) {
            credentials
        } else {
            credentials.filter { credential ->
                val title = (credential["title"] as? String)?.lowercase() ?: ""
                val credentialType = (credential["credentialType"] as? String)?.lowercase() ?: ""
                val tags = (credential["tags"] as? List<*>)?.joinToString(" ") { it.toString().lowercase() } ?: ""

                // Search in fields
                val fieldsText = (credential["fields"] as? Map<*, *>)?.values?.joinToString(" ") { fieldValue ->
                    if (fieldValue is Map<*, *>) {
                        val value = fieldValue["value"] as? String ?: ""
                        val sensitive = fieldValue["sensitive"] as? Boolean ?: false
                        // Don't search in sensitive fields for security
                        if (!sensitive) value.lowercase() else ""
                    } else {
                        ""
                    }
                } ?: ""

                val searchText = "$title $credentialType $tags $fieldsText"
                searchText.contains(query.lowercase())
            }
        }

        _uiState.value = _uiState.value.copy(
            filteredCredentials = filtered,
            hasSearchResults = filtered.isNotEmpty() || query.isBlank()
        )
    }

    /**
     * Get credential by ID
     */
    suspend fun getCredential(credentialId: String): Map<String, Any>? {
        return try {
            Log.d(TAG, "Getting credential: $credentialId")

            val result = repositoryManager.getCredential(credentialId)

            when (result) {
                is MobileRepositoryManager.RepositoryResult.Success<*> -> {
                    Log.d(TAG, "Retrieved credential successfully")
                    result.data as? Map<String, Any>
                }

                is MobileRepositoryManager.RepositoryResult.Error -> {
                    Log.e(TAG, "Failed to get credential: ${result.message}", result.exception)
                    null
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting credential: ${e.message}", e)
            null
        }
    }

    /**
     * Get credentials by type
     */
    fun getCredentialsByType(type: String): List<Map<String, Any>> {
        return _uiState.value.credentials.filter { credential ->
            (credential["credentialType"] as? String) == type
        }
    }

    /**
     * Get credentials by tag
     */
    fun getCredentialsByTag(tag: String): List<Map<String, Any>> {
        return _uiState.value.credentials.filter { credential ->
            val tags = credential["tags"] as? List<*> ?: emptyList<Any>()
            tags.contains(tag)
        }
    }

    /**
     * Get unique credential types
     */
    fun getCredentialTypes(): List<String> {
        return _uiState.value.credentials
            .mapNotNull { it["credentialType"] as? String }
            .distinct()
            .sorted()
    }

    /**
     * Get unique tags
     */
    fun getTags(): List<String> {
        return _uiState.value.credentials
            .flatMap { credential ->
                (credential["tags"] as? List<*>)?.mapNotNull { it as? String } ?: emptyList()
            }
            .distinct()
            .sorted()
    }

    /**
     * Check if repository is currently open
     */
    fun checkRepositoryStatus() {
        viewModelScope.launch {
            refreshRepositoryState()
        }
    }

    /**
     * Refresh repository state
     */
    private suspend fun refreshRepositoryState() {
        try {
            val stateResult = repositoryManager.getRepositoryState()

            if (stateResult is MobileRepositoryManager.RepositoryResult.Success<*>) {
                val repositoryState = stateResult.data
                if (repositoryState is MobileRepositoryManager.RepositoryState) {
                    _repositoryOpen.value = repositoryState.isOpen

                    if (!repositoryState.isOpen) {
                        // Clear credentials if repository is closed
                        _uiState.value = _uiState.value.copy(
                            credentials = emptyList(),
                            filteredCredentials = emptyList(),
                            isEmpty = true
                        )
                    }
                } else {
                    _repositoryOpen.value = false
                }
            } else if (stateResult is MobileRepositoryManager.RepositoryResult.Error<*>) {
                Log.w(TAG, "Failed to get repository state: ${stateResult.message}")
                _repositoryOpen.value = false
            } else {
                Log.w(TAG, "Unknown repository state result type")
                _repositoryOpen.value = false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error checking repository status", e)
            _repositoryOpen.value = false
        }
    }

    /**
     * Clear all credentials from the repository
     */
    fun clearCredentials() {
        viewModelScope.launch {
            try {
                Log.d(TAG, "Clearing all credentials")

                _uiState.value = _uiState.value.copy(
                    credentials = emptyList(),
                    isLoading = false,
                    errorMessage = null,
                    isEmpty = true,
                    hasSearchResults = true
                )

                _searchQuery.value = ""

                Log.d(TAG, "All credentials cleared")
            } catch (e: Exception) {
                Log.e(TAG, "Error clearing credentials: ${e.message}", e)
                _uiState.value = _uiState.value.copy(
                    errorMessage = "Failed to clear credentials: ${e.message}"
                )
            }
        }
    }

    /**
     * Clear all error messages
     */
    private fun clearError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }

    /**
     * Clear credentials state
     */
    fun clearCredentialsState() {
        _uiState.value = _uiState.value.copy(
            credentials = emptyList(),
            isLoading = false,
            errorMessage = null,
            isEmpty = true
        )
    }

    /**
     * Sort credentials by title
     */
    fun sortCredentialsByTitle(ascending: Boolean = true) {
        val credentials = _uiState.value.credentials.sortedBy {
            (it["title"] as? String)?.lowercase() ?: ""
        }.let { if (ascending) it else it.reversed() }

        _uiState.value = _uiState.value.copy(credentials = credentials)
        applySearchFilter(_searchQuery.value) // Reapply search filter
    }

    /**
     * Sort credentials by type
     */
    fun sortCredentialsByType(ascending: Boolean = true) {
        val credentials = _uiState.value.credentials.sortedBy {
            (it["credentialType"] as? String)?.lowercase() ?: ""
        }.let { if (ascending) it else it.reversed() }

        _uiState.value = _uiState.value.copy(credentials = credentials)
        applySearchFilter(_searchQuery.value) // Reapply search filter
    }

    /**
     * Sort credentials by last modified
     */
    fun sortCredentialsByDate(ascending: Boolean = true) {
        val credentials = _uiState.value.credentials.sortedBy {
            (it["updatedAt"] as? Long) ?: 0L
        }.let { if (ascending) it else it.reversed() }

        _uiState.value = _uiState.value.copy(credentials = credentials)
        applySearchFilter(_searchQuery.value) // Reapply search filter
    }

    override fun onCleared() {
        super.onCleared()
        Log.d(TAG, "CredentialsViewModel cleared")
    }
}

/**
 * UI State for credentials screen
 */
data class CredentialsUiState(
    val isLoading: Boolean = false,
    val credentials: List<Map<String, Any>> = emptyList(),
    val filteredCredentials: List<Map<String, Any>> = emptyList(),
    val errorMessage: String? = null,
    val isEmpty: Boolean = true,
    val hasSearchResults: Boolean = true
)

/**
 * Factory for creating CredentialsViewModel instances
 */
class CredentialsViewModelFactory(
    private val context: Context
) : ViewModelProvider.Factory {
    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(CredentialsViewModel::class.java)) {
            return CredentialsViewModel(context) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class")
    }
}
