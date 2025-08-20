package com.ziplock

import com.ziplock.viewmodel.RepositoryViewModel
import com.ziplock.viewmodel.RepositoryState
import com.ziplock.viewmodel.PassphraseStrength
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.*
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*

/**
 * Unit tests for RepositoryViewModel
 *
 * Tests the business logic and state management functionality
 * without requiring the actual FFI library integration.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class RepositoryViewModelTest {

    private lateinit var viewModel: RepositoryViewModel
    private val testDispatcher = StandardTestDispatcher()

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        viewModel = RepositoryViewModel()
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `initial state should be correct`() {
        // Verify initial UI state
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertNull(uiState.errorMessage)
        assertNull(uiState.successMessage)

        // Verify initial repository state
        val repositoryState = viewModel.repositoryState.value
        assertTrue(repositoryState is RepositoryState.None)
    }

    @Test
    fun `openRepository with valid inputs should succeed`() = runTest {
        val testPath = "/test/archive.7z"
        val testPassphrase = "validpassword"

        // Start opening repository
        viewModel.openRepository(testPath, testPassphrase)

        // Verify loading state is set
        assertTrue(viewModel.uiState.value.isLoading)
        assertNull(viewModel.uiState.value.errorMessage)

        // Advance time to complete the operation
        advanceTimeBy(2000)

        // Verify final state
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertEquals("Archive opened successfully", uiState.successMessage)
        assertNull(uiState.errorMessage)

        // Verify repository state
        val repositoryState = viewModel.repositoryState.value
        assertTrue(repositoryState is RepositoryState.Opened)
        if (repositoryState is RepositoryState.Opened) {
            assertEquals(testPath, repositoryState.archivePath)
            assertTrue(repositoryState.sessionId.isNotEmpty())
        }
    }

    @Test
    fun `openRepository with empty path should fail`() = runTest {
        val emptyPath = ""
        val testPassphrase = "validpassword"

        viewModel.openRepository(emptyPath, testPassphrase)

        // Advance time to complete the operation
        advanceTimeBy(2000)

        // Verify error state
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertEquals("Archive file path is required", uiState.errorMessage)
        assertNull(uiState.successMessage)

        // Repository should remain in None state
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)
    }

    @Test
    fun `openRepository with empty passphrase should fail`() = runTest {
        val testPath = "/test/archive.7z"
        val emptyPassphrase = ""

        viewModel.openRepository(testPath, emptyPassphrase)

        // Advance time to complete the operation
        advanceTimeBy(2000)

        // Verify error state
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertEquals("Passphrase is required", uiState.errorMessage)
        assertNull(uiState.successMessage)

        // Repository should remain in None state
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)
    }

    @Test
    fun `createRepository with valid inputs should succeed`() = runTest {
        val testPath = "/test/newarchive.7z"
        val testPassphrase = "strongpassword123"

        viewModel.createRepository(testPath, testPassphrase)

        // Verify loading state
        assertTrue(viewModel.uiState.value.isLoading)

        // Advance time to complete the operation
        advanceTimeBy(2500)

        // Verify final state
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertEquals("New archive created successfully", uiState.successMessage)
        assertNull(uiState.errorMessage)

        // Verify repository state
        val repositoryState = viewModel.repositoryState.value
        assertTrue(repositoryState is RepositoryState.Created)
        if (repositoryState is RepositoryState.Created) {
            assertEquals(testPath, repositoryState.archivePath)
            assertTrue(repositoryState.sessionId.isNotEmpty())
        }
    }

    @Test
    fun `createRepository with weak passphrase should fail`() = runTest {
        val testPath = "/test/newarchive.7z"
        val weakPassphrase = "weak" // Less than 8 characters

        viewModel.createRepository(testPath, weakPassphrase)

        // Advance time to complete the operation
        advanceTimeBy(2500)

        // Verify error state
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertEquals("Passphrase must be at least 8 characters long", uiState.errorMessage)
        assertNull(uiState.successMessage)

        // Repository should remain in None state
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)
    }

    @Test
    fun `closeRepository should reset state`() = runTest {
        // First open a repository
        viewModel.openRepository("/test/archive.7z", "password")
        advanceTimeBy(2000)

        // Verify repository is opened
        assertTrue(viewModel.repositoryState.value is RepositoryState.Opened)

        // Close the repository
        viewModel.closeRepository()

        // Verify state is reset
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)
        val uiState = viewModel.uiState.value
        assertFalse(uiState.isLoading)
        assertNull(uiState.errorMessage)
        assertNull(uiState.successMessage)
    }

    @Test
    fun `clearError should remove error message`() {
        // First trigger an error
        runTest {
            viewModel.openRepository("", "password")
            advanceTimeBy(2000)
        }

        // Verify error exists
        assertNotNull(viewModel.uiState.value.errorMessage)

        // Clear error
        viewModel.clearError()

        // Verify error is cleared
        assertNull(viewModel.uiState.value.errorMessage)
    }

    @Test
    fun `clearSuccess should remove success message`() {
        runTest {
            viewModel.openRepository("/test/archive.7z", "password")
            advanceTimeBy(2000)
        }

        // Verify success message exists
        assertNotNull(viewModel.uiState.value.successMessage)

        // Clear success
        viewModel.clearSuccess()

        // Verify success message is cleared
        assertNull(viewModel.uiState.value.successMessage)
    }

    @Test
    fun `validatePassphrase should return correct strength for weak password`() {
        val weakPassword = "123"
        val validation = viewModel.validatePassphrase(weakPassword)

        assertEquals(PassphraseStrength.VeryWeak, validation.strength)
        assertFalse(validation.isValid)
        assertTrue(validation.requirements.contains("At least 8 characters"))
        assertTrue(validation.requirements.contains("At least one uppercase letter"))
        assertTrue(validation.requirements.contains("At least one lowercase letter"))
    }

    @Test
    fun `validatePassphrase should return correct strength for good password`() {
        val goodPassword = "Password123"
        val validation = viewModel.validatePassphrase(goodPassword)

        assertEquals(PassphraseStrength.Strong, validation.strength)
        assertTrue(validation.isValid)
        assertTrue(validation.satisfied.contains("Minimum length (8 characters)"))
        assertTrue(validation.satisfied.contains("Contains uppercase letter"))
        assertTrue(validation.satisfied.contains("Contains lowercase letter"))
        assertTrue(validation.satisfied.contains("Contains number"))
        assertTrue(validation.requirements.contains("At least one special character"))
    }

    @Test
    fun `validatePassphrase should return very strong for excellent password`() {
        val excellentPassword = "MySecureP@ssw0rd2024!"
        val validation = viewModel.validatePassphrase(excellentPassword)

        assertEquals(PassphraseStrength.VeryStrong, validation.strength)
        assertTrue(validation.isValid)
        assertTrue(validation.requirements.isEmpty())
        assertEquals(5, validation.satisfied.size)
        assertTrue(validation.satisfied.contains("Minimum length (8 characters)"))
        assertTrue(validation.satisfied.contains("Contains uppercase letter"))
        assertTrue(validation.satisfied.contains("Contains lowercase letter"))
        assertTrue(validation.satisfied.contains("Contains number"))
        assertTrue(validation.satisfied.contains("Contains special character"))
    }

    @Test
    fun `isCloudStorageFile should detect Google Drive paths`() {
        val googleDrivePath = "/Android/data/com.google.android.apps.docs/files/passwords.7z"
        assertTrue(viewModel.isCloudStorageFile(googleDrivePath))
    }

    @Test
    fun `isCloudStorageFile should detect Dropbox paths`() {
        val dropboxPath = "/Android/data/com.dropbox.android/files/passwords.7z"
        assertTrue(viewModel.isCloudStorageFile(dropboxPath))
    }

    @Test
    fun `isCloudStorageFile should detect SAF content URIs`() {
        val safUri = "content://com.android.providers.media.documents/document/1234"
        assertTrue(viewModel.isCloudStorageFile(safUri))
    }

    @Test
    fun `isCloudStorageFile should not detect local paths`() {
        val localPath = "/storage/emulated/0/Download/passwords.7z"
        assertFalse(viewModel.isCloudStorageFile(localPath))
    }

    @Test
    fun `isCloudStorageFile should be case insensitive`() {
        val mixedCasePath = "/android/data/com.GOOGLE.android.apps.docs/files/passwords.7z"
        assertTrue(viewModel.isCloudStorageFile(mixedCasePath))
    }

    @Test
    fun `multiple operations should maintain correct state`() = runTest {
        // Open repository
        viewModel.openRepository("/test/archive1.7z", "password1")
        advanceTimeBy(2000)
        assertTrue(viewModel.repositoryState.value is RepositoryState.Opened)

        // Close repository
        viewModel.closeRepository()
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)

        // Create new repository
        viewModel.createRepository("/test/archive2.7z", "password123")
        advanceTimeBy(2500)
        assertTrue(viewModel.repositoryState.value is RepositoryState.Created)

        // Close again
        viewModel.closeRepository()
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)
    }

    @Test
    fun `error state should not affect repository state on failure`() = runTest {
        // Attempt to open with invalid input
        viewModel.openRepository("", "")
        advanceTimeBy(2000)

        // Repository should remain in None state despite error
        assertTrue(viewModel.repositoryState.value is RepositoryState.None)
        assertNotNull(viewModel.uiState.value.errorMessage)
    }

    @Test
    fun `loading state should be properly managed during operations`() = runTest {
        // Initial state should not be loading
        assertFalse(viewModel.uiState.value.isLoading)

        // Start operation
        viewModel.openRepository("/test/archive.7z", "password")

        // Should be loading immediately
        assertTrue(viewModel.uiState.value.isLoading)

        // Complete operation
        advanceTimeBy(2000)

        // Should no longer be loading
        assertFalse(viewModel.uiState.value.isLoading)
    }
}
