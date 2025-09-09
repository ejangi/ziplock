package com.ziplock.viewmodel

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider

/**
 * ViewModelFactory for CredentialFormViewModel that provides Context dependency
 *
 * This factory is needed because CredentialFormViewModel now requires a Context
 * to access MobileRepositoryManager in the unified architecture.
 */
class CredentialFormViewModelFactory(
    private val context: Context
) : ViewModelProvider.Factory {

    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(CredentialFormViewModel::class.java)) {
            return CredentialFormViewModel(context) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class: ${modelClass.name}")
    }
}
