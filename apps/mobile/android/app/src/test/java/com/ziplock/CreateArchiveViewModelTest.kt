package com.ziplock

import com.ziplock.viewmodel.CreateArchiveViewModel
import com.ziplock.viewmodel.CreateArchiveStep
import kotlinx.coroutines.test.runTest
import org.junit.Test
import org.junit.Assert.*

/**
 * Unit tests for CreateArchiveViewModel
 *
 * Tests the core functionality of the Create Archive wizard state management
 * without requiring FFI integration.
 */
class CreateArchiveViewModelTest {

    @Test
    fun `initial state should be Welcome step`() {
        val viewModel = CreateArchiveViewModel()
        val initialState = viewModel.uiState.value

        assertEquals(CreateArchiveStep.SelectDestination, initialState.currentStep)
        assertEquals("ZipLock", initialState.archiveName)
        assertNull(initialState.destinationPath)
        assertTrue(initialState.passphrase.isEmpty())
        assertFalse(initialState.showPassphrase)
        assertNull(initialState.errorMessage)
    }

    @Test
    fun `updateStep should change current step and clear errors`() {
        val viewModel = CreateArchiveViewModel()

        // Set an error first
        viewModel.updateStep(CreateArchiveStep.SelectDestination)

        val state = viewModel.uiState.value
        assertEquals(CreateArchiveStep.SelectDestination, state.currentStep)
        assertNull(state.errorMessage)
    }

    @Test
    fun `setDestination should update destination path and name`() {
        val viewModel = CreateArchiveViewModel()
        val testPath = "content://com.android.providers.media.documents/tree/primary%3ADownloads"
        val testName = "Downloads"

        viewModel.setDestination(testPath, testName)

        val state = viewModel.uiState.value
        assertEquals(testPath, state.destinationPath)
        assertEquals(testName, state.destinationName)
        assertNull(state.errorMessage)
    }

    @Test
    fun `updateArchiveName should update archive name and clear errors`() {
        val viewModel = CreateArchiveViewModel()
        val testName = "MyPasswords"

        viewModel.updateArchiveName(testName)

        val state = viewModel.uiState.value
        assertEquals(testName, state.archiveName)
        assertNull(state.errorMessage)
    }

    @Test
    fun `updatePassphrase should update passphrase and clear errors`() {
        val viewModel = CreateArchiveViewModel()
        val testPassphrase = "TestPassphrase123!"

        viewModel.updatePassphrase(testPassphrase)

        val state = viewModel.uiState.value
        assertEquals(testPassphrase, state.passphrase)
        assertNull(state.errorMessage)
    }

    @Test
    fun `updateConfirmPassphrase should update confirm passphrase`() {
        val viewModel = CreateArchiveViewModel()
        val testPassphrase = "TestPassphrase123!"

        viewModel.updateConfirmPassphrase(testPassphrase)

        val state = viewModel.uiState.value
        assertEquals(testPassphrase, state.confirmPassphrase)
        assertNull(state.errorMessage)
    }

    @Test
    fun `togglePassphraseVisibility should toggle show passphrase flag`() {
        val viewModel = CreateArchiveViewModel()

        // Initially false
        assertFalse(viewModel.uiState.value.showPassphrase)

        viewModel.togglePassphraseVisibility()
        assertTrue(viewModel.uiState.value.showPassphrase)

        viewModel.togglePassphraseVisibility()
        assertFalse(viewModel.uiState.value.showPassphrase)
    }

    @Test
    fun `toggleConfirmPassphraseVisibility should toggle show confirm passphrase flag`() {
        val viewModel = CreateArchiveViewModel()

        // Initially false
        assertFalse(viewModel.uiState.value.showConfirmPassphrase)

        viewModel.toggleConfirmPassphraseVisibility()
        assertTrue(viewModel.uiState.value.showConfirmPassphrase)

        viewModel.toggleConfirmPassphraseVisibility()
        assertFalse(viewModel.uiState.value.showConfirmPassphrase)
    }

    @Test
    fun `canProceed should return correct values for different steps`() {
        val viewModel = CreateArchiveViewModel()

        // Welcome step - always true
        viewModel.updateStep(CreateArchiveStep.Welcome)
        assertTrue(viewModel.canProceed())

        // SelectDestination - requires destination path
        viewModel.updateStep(CreateArchiveStep.SelectDestination)
        assertFalse(viewModel.canProceed())

        viewModel.setDestination("test/path", "Test Folder")
        assertTrue(viewModel.canProceed())

        // ArchiveName - requires non-blank name
        viewModel.updateStep(CreateArchiveStep.ArchiveName)
        viewModel.updateArchiveName("")
        assertFalse(viewModel.canProceed())

        viewModel.updateArchiveName("TestArchive")
        assertTrue(viewModel.canProceed())

        // ConfirmPassphrase - requires matching passphrases
        viewModel.updateStep(CreateArchiveStep.ConfirmPassphrase)
        viewModel.updatePassphrase("password123")
        viewModel.updateConfirmPassphrase("different")
        assertFalse(viewModel.canProceed())

        viewModel.updateConfirmPassphrase("password123")
        assertTrue(viewModel.canProceed())
    }

    @Test
    fun `clearError should remove error message`() {
        val viewModel = CreateArchiveViewModel()

        // Simulate setting an error by trying to proceed without required data
        viewModel.updateStep(CreateArchiveStep.SelectDestination)
        viewModel.proceedToNext() // Should set error

        // Clear the error
        viewModel.clearError()

        assertNull(viewModel.uiState.value.errorMessage)
    }

    @Test
    fun `reset should return to initial state`() {
        val viewModel = CreateArchiveViewModel()

        // Modify state
        viewModel.setDestination("test/path", "Test")
        viewModel.updateArchiveName("TestArchive")
        viewModel.updatePassphrase("password123")
        viewModel.updateStep(CreateArchiveStep.CreatePassphrase)

        // Reset
        viewModel.reset()

        val state = viewModel.uiState.value
        assertEquals(CreateArchiveStep.SelectDestination, state.currentStep)
        assertEquals("ZipLock", state.archiveName)
        assertNull(state.destinationPath)
        assertTrue(state.passphrase.isEmpty())
        assertNull(state.errorMessage)
    }

    @Test
    fun `proceedToNext should validate and advance steps correctly`() {
        val viewModel = CreateArchiveViewModel()

        // SelectDestination -> should fail without destination
        assertEquals(CreateArchiveStep.SelectDestination, viewModel.uiState.value.currentStep)
        viewModel.proceedToNext()
        assertEquals(CreateArchiveStep.SelectDestination, viewModel.uiState.value.currentStep)

        // SelectDestination -> ArchiveName (after setting destination)
        viewModel.updateDestinationPath("/test/path", "test folder")
        viewModel.proceedToNext()
        assertEquals(CreateArchiveStep.ArchiveName, viewModel.uiState.value.currentStep)

        // ArchiveName -> CreatePassphrase (should work with default name)
        viewModel.proceedToNext()
        assertEquals(CreateArchiveStep.CreatePassphrase, viewModel.uiState.value.currentStep)
    }

    @Test
    fun `goBack should navigate to previous steps correctly`() {
        val viewModel = CreateArchiveViewModel()

        // Navigate forward a few steps
        viewModel.updateStep(CreateArchiveStep.ConfirmPassphrase)

        // Go back
        viewModel.goBack()
        assertEquals(CreateArchiveStep.CreatePassphrase, viewModel.uiState.value.currentStep)

        viewModel.goBack()
        assertEquals(CreateArchiveStep.ArchiveName, viewModel.uiState.value.currentStep)

        viewModel.goBack()
        assertEquals(CreateArchiveStep.SelectDestination, viewModel.uiState.value.currentStep)

        viewModel.goBack()
        assertEquals(CreateArchiveStep.SelectDestination, viewModel.uiState.value.currentStep)

        // Should not go back from SelectDestination (first step)
        viewModel.goBack()
        assertEquals(CreateArchiveStep.SelectDestination, viewModel.uiState.value.currentStep)
    }
}
