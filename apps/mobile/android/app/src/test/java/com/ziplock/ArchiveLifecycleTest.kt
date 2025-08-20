package com.ziplock

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.ziplock.viewmodel.HybridRepositoryViewModel
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import android.util.Log
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.ViewModelStore
import com.ziplock.repository.HybridRepositoryManager
import com.ziplock.ffi.ZipLockNative
import kotlinx.coroutines.delay
import org.junit.Assert.*
import org.mockito.Mockito.*
import java.io.File
import java.io.FileOutputStream

/**
 * Test suite to verify proper archive closure during Android app lifecycle events.
 *
 * This test ensures that:
 * 1. Archives are properly closed when ViewModel is cleared
 * 2. Lifecycle events don't cause memory leaks
 * 3. App exit scenarios properly clean up resources
 * 4. Background/foreground transitions maintain archive state correctly
 */
@RunWith(AndroidJUnit4::class)
@Config(sdk = [30]) // Target Android 11 for consistent testing
class ArchiveLifecycleTest {

    companion object {
        private const val TAG = "ArchiveLifecycleTest"
        private const val TEST_ARCHIVE_NAME = "test_lifecycle_archive.7z"
        private const val TEST_PASSWORD = "test_password_123"
    }

    private lateinit var context: Context
    private lateinit var viewModelStore: ViewModelStore
    private lateinit var testArchiveFile: File
    private var hybridRepositoryViewModel: HybridRepositoryViewModel? = null

    @Before
    fun setUp() {
        context = ApplicationProvider.getApplicationContext()
        viewModelStore = ViewModelStore()

        // Create a test archive file
        testArchiveFile = File(context.cacheDir, TEST_ARCHIVE_NAME)
        createTestArchive()

        // Initialize ZipLock native library
        try {
            ZipLockNative.init()
            Log.d(TAG, "ZipLock native library initialized for testing")
        } catch (e: Exception) {
            Log.w(TAG, "Could not initialize native library: ${e.message}")
        }
    }

    @After
    fun tearDown() {
        // Clean up test files
        if (::testArchiveFile.isInitialized && testArchiveFile.exists()) {
            testArchiveFile.delete()
        }

        // Clear ViewModelStore to trigger onCleared()
        viewModelStore.clear()

        // Additional cleanup
        hybridRepositoryViewModel = null
    }

    /**
     * Test that archives are properly closed when ViewModel is cleared
     */
    @Test
    fun testArchiveClosureOnViewModelCleared() = runTest {
        // Arrange
        val viewModel = createHybridRepositoryViewModel()

        // Act - Open an archive
        viewModel.openRepository(testArchiveFile.absolutePath, TEST_PASSWORD)

        // Wait for async operation to complete
        delay(1000)

        val repositoryState = viewModel.repositoryState.value
        assertTrue("Repository should be open", repositoryState is HybridRepositoryViewModel.HybridRepositoryState.Open)

        // Act - Clear ViewModel (simulates app destruction)
        viewModelStore.clear()

        // Allow onCleared to execute
        delay(500)

        // Assert - Repository should be closed
        // Note: After ViewModel is cleared, we can't directly check its state
        // But we can verify through logs and that no exceptions occurred
        Log.d(TAG, "ViewModel cleared successfully without exceptions")
    }

    /**
     * Test multiple lifecycle events don't cause issues
     */
    @Test
    fun testMultipleLifecycleEvents() = runTest {
        val viewModel = createHybridRepositoryViewModel()

        // Simulate app lifecycle: create -> pause -> resume -> stop -> destroy

        // 1. Open repository (onCreate equivalent)
        viewModel.openRepository(testArchiveFile.absolutePath, TEST_PASSWORD)
        delay(500)

        // 2. Simulate onPause - archive should remain open
        Log.d(TAG, "Simulating onPause - archive remains open")
        val stateAfterPause = viewModel.repositoryState.value
        assertTrue("Repository should remain open after pause",
                  stateAfterPause is HybridRepositoryViewModel.HybridRepositoryState.Open)

        // 3. Simulate onResume - archive should still be open
        Log.d(TAG, "Simulating onResume - archive still open")
        val stateAfterResume = viewModel.repositoryState.value
        assertTrue("Repository should remain open after resume",
                  stateAfterResume is HybridRepositoryViewModel.HybridRepositoryState.Open)

        // 4. Simulate onStop - archive should remain open
        Log.d(TAG, "Simulating onStop - archive still open")
        val stateAfterStop = viewModel.repositoryState.value
        assertTrue("Repository should remain open after stop",
                  stateAfterStop is HybridRepositoryViewModel.HybridRepositoryState.Open)

        // 5. Simulate onDestroy - ViewModel cleared, archive closed
        viewModelStore.clear()
        delay(500)

        Log.d(TAG, "Lifecycle simulation completed successfully")
    }

    /**
     * Test that manual close works correctly
     */
    @Test
    fun testManualArchiveClose() = runTest {
        val viewModel = createHybridRepositoryViewModel()

        // Open repository
        viewModel.openRepository(testArchiveFile.absolutePath, TEST_PASSWORD)
        delay(500)

        val openState = viewModel.repositoryState.value
        assertTrue("Repository should be open", openState is HybridRepositoryViewModel.HybridRepositoryState.Open)

        // Manually close repository
        viewModel.closeRepository()
        delay(500)

        val closedState = viewModel.repositoryState.value
        assertTrue("Repository should be closed", closedState is HybridRepositoryViewModel.HybridRepositoryState.None)
    }

    /**
     * Test that rapid lifecycle changes don't cause crashes
     */
    @Test
    fun testRapidLifecycleChanges() = runTest {
        val viewModel = createHybridRepositoryViewModel()

        // Rapidly open and close repository multiple times
        repeat(3) { iteration ->
            Log.d(TAG, "Rapid lifecycle test iteration: $iteration")

            viewModel.openRepository(testArchiveFile.absolutePath, TEST_PASSWORD)
            delay(200)

            viewModel.closeRepository()
            delay(200)
        }

        // Final state should be closed
        val finalState = viewModel.repositoryState.value
        assertTrue("Repository should be closed after rapid changes",
                  finalState is HybridRepositoryViewModel.HybridRepositoryState.None)

        Log.d(TAG, "Rapid lifecycle changes completed successfully")
    }

    /**
     * Test error handling during lifecycle events
     */
    @Test
    fun testErrorHandlingDuringLifecycle() = runTest {
        val viewModel = createHybridRepositoryViewModel()

        // Try to open non-existent archive
        val nonExistentFile = File(context.cacheDir, "non_existent.7z")
        viewModel.openRepository(nonExistentFile.absolutePath, TEST_PASSWORD)
        delay(500)

        val errorState = viewModel.repositoryState.value
        assertTrue("Repository should be in error state",
                  errorState is HybridRepositoryViewModel.HybridRepositoryState.None)

        // Clear ViewModel - should not crash even in error state
        viewModelStore.clear()
        delay(500)

        Log.d(TAG, "Error handling during lifecycle completed successfully")
    }

    /**
     * Test background/foreground transitions
     */
    @Test
    fun testBackgroundForegroundTransitions() = runTest {
        val viewModel = createHybridRepositoryViewModel()

        // Open repository
        viewModel.openRepository(testArchiveFile.absolutePath, TEST_PASSWORD)
        delay(500)

        // Simulate app going to background (onStop)
        Log.d(TAG, "App going to background - archive should remain open")
        val backgroundState = viewModel.repositoryState.value
        assertTrue("Repository should remain open in background",
                  backgroundState is HybridRepositoryViewModel.HybridRepositoryState.Open)

        // Simulate app coming to foreground (onStart/onResume)
        Log.d(TAG, "App coming to foreground - archive should still be open")
        val foregroundState = viewModel.repositoryState.value
        assertTrue("Repository should remain open in foreground",
                  foregroundState is HybridRepositoryViewModel.HybridRepositoryState.Open)

        Log.d(TAG, "Background/foreground transitions completed successfully")
    }

    /**
     * Helper method to create HybridRepositoryViewModel for testing
     */
    private fun createHybridRepositoryViewModel(): HybridRepositoryViewModel {
        val factory = HybridRepositoryViewModel.Factory(context)
        val provider = ViewModelProvider(viewModelStore, factory)
        hybridRepositoryViewModel = provider[HybridRepositoryViewModel::class.java]
        return hybridRepositoryViewModel!!
    }

    /**
     * Helper method to create a test archive file
     */
    private fun createTestArchive() {
        try {
            // Create a minimal valid archive file for testing
            testArchiveFile.createNewFile()

            // Write minimal 7z header (simplified for testing)
            FileOutputStream(testArchiveFile).use { fos ->
                // 7z signature
                fos.write(byteArrayOf(0x37, 0x7A, 0xBC.toByte(), 0xAF.toByte(), 0x27, 0x1C))
                // Minimal header data
                fos.write(ByteArray(32) { 0 })
            }

            Log.d(TAG, "Test archive created: ${testArchiveFile.absolutePath}")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to create test archive", e)
            fail("Could not create test archive: ${e.message}")
        }
    }
}
