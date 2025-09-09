package com.ziplock

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.repeatOnLifecycle
import com.ziplock.utils.DebugUtils
import com.ziplock.utils.PlatformUtils
import com.ziplock.utils.XZTestUtils
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import com.ziplock.viewmodel.RepositoryViewModel
import com.ziplock.viewmodel.RepositoryViewModelFactory
import com.ziplock.repository.MobileRepositoryManager
import kotlinx.coroutines.launch
import androidx.lifecycle.ViewModelProvider
import com.ziplock.ffi.ZipLockNativeHelper

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.compose.ui.text.font.FontWeight
import kotlinx.coroutines.delay
import com.ziplock.ui.screens.CreateArchiveWizard
import com.ziplock.ui.screens.RepositorySelectionScreen
import com.ziplock.ui.screens.CredentialsListScreen
import com.ziplock.ui.screens.CredentialTemplateSelectionScreen
import com.ziplock.ui.screens.CredentialFormScreen

import com.ziplock.ui.theme.*
import com.ziplock.ffi.ZipLockNative
import com.ziplock.viewmodel.CredentialFormViewModel
import com.ziplock.viewmodel.CredentialFormViewModelFactory
import com.ziplock.viewmodel.CredentialsViewModel
import com.ziplock.viewmodel.CredentialsViewModelFactory
import androidx.compose.runtime.collectAsState
import com.ziplock.viewmodel.CreateArchiveViewModel
import androidx.compose.ui.platform.LocalContext

class MainActivity : ComponentActivity() {

    companion object {
        private const val TAG = "MainActivity"
    }

    // Use unified architecture
    private val repositoryViewModel: RepositoryViewModel by viewModels {
        RepositoryViewModelFactory(this)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Check for incoming file URI from intent
        val fileUri = intent.getStringExtra("file_uri")
        val openedFromFile = intent.getBooleanExtra("opened_from_file", false)

        // Log platform information for debugging
        PlatformUtils.logPlatformInfo()

        // Show compatibility warning if needed
        PlatformUtils.getArchiveCompatibilityMessage()?.let { message ->
            Log.i(TAG, "Platform Compatibility: $message")
            println("MainActivity: $message")
        }

        // Test XZ library availability first
        try {
            println("MainActivity: Testing XZ library availability...")
            val testResult = XZTestUtils.runComprehensiveTest()
            val report = testResult.getFormattedReport()
            println("MainActivity: XZ Test Report:")
            println(report)
            Log.i(TAG, "XZ Test Report: $report")

            val classLoaderInfo = XZTestUtils.getClassLoaderInfo()
            println("MainActivity: ClassLoader Info:")
            println(classLoaderInfo)
            Log.d(TAG, classLoaderInfo)

            if (!testResult.overallSuccess) {
                Log.e(TAG, "❌ XZ library test failed - archive operations may not work")
                println("MainActivity: ❌ XZ library test failed")
            } else {
                Log.i(TAG, "✅ XZ library test passed")
                println("MainActivity: ✅ XZ library test passed")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error during XZ test", e)
            println("MainActivity: Error during XZ test: ${e.message}")
        }

        // Initialize unified architecture
        try {
            println("MainActivity: Initializing unified architecture...")

            // Initialize repository manager (handled in ViewModel)
            Log.i(TAG, "✅ Unified architecture initialization started")
            println("MainActivity: ✅ Unified architecture mode enabled")

            // Initialize native FFI library
            println("MainActivity: Initializing native library...")
            val initResult = ZipLockNative.init()
            println("MainActivity: Native init result: $initResult")
            if (initResult == 0) {
                Log.d(TAG, "Legacy library initialized successfully")
                println("MainActivity: ✅ Legacy fallback available")

                // Additional warning for x86 emulator users
                if (PlatformUtils.isX86Emulator()) {
                    Log.w(TAG, "⚠️ Running on x86 emulator - archive operations may crash")
                    println("MainActivity: ⚠️ WARNING: x86 emulator detected - consider using ARM emulator")
                }

                // Set application context for credential persistence
                ZipLockNative.setContext(this)

                Log.d("MainActivity", "Android SAF is available via unified architecture")
                println("MainActivity: Android SAF integrated via unified architecture")

                // Get library version
                val version = ZipLockNative.getVersion()
                println("MainActivity: Library version: $version")
                Log.d("MainActivity", "ZipLock library version: $version")

                // Initialize debug settings using DebugUtils
                try {
                    DebugUtils.initializeDebugSettings(this)
                    Log.d("MainActivity", "Debug settings initialized successfully")
                    println("MainActivity: Debug settings initialized")

                    // Run debug tests in debug builds
                    try {
                        val testResult = DebugUtils.runDebugTests(this)
                        Log.d("MainActivity", "Debug tests completed: ${testResult.allTestsPassed}")
                        testResult.testResults.forEach { result ->
                            println("MainActivity: $result")
                        }
                    } catch (e: Exception) {
                        Log.w("MainActivity", "Debug tests failed: ${e.message}")
                    }
                } catch (e: Exception) {
                    Log.w("MainActivity", "Exception during debug initialization: ${e.message}")
                    println("MainActivity: WARNING - Exception during debug initialization: ${e.message}")
                }
            } else {
                Log.e("MainActivity", "Failed to initialize ZipLock native library")
                println("MainActivity: ERROR - Failed to initialize ZipLock native library")
            }
        } catch (e: Exception) {
            Log.e("MainActivity", "Error initializing ZipLock native library: ${e.message}")
            println("MainActivity: EXCEPTION - Error initializing ZipLock native library: ${e.message}")
            e.printStackTrace()
        }

        // Set up lifecycle-aware archive management
        setupLifecycleAwareArchiveManagement()

        setContent {
            ZipLockTheme {
                MainApp(
                    repositoryViewModel = repositoryViewModel,
                    initialFileUri = if (openedFromFile) fileUri else null
                )
            }
        }
    }

    /**
     * Set up lifecycle-aware archive management to handle app pause/resume states
     */
    private fun setupLifecycleAwareArchiveManagement() {
        lifecycleScope.launch {
            lifecycle.repeatOnLifecycle(Lifecycle.State.STARTED) {
                // This block runs when the app is in the foreground
                Log.d(TAG, "App is in foreground")
            }
        }
    }

    override fun onPause() {
        super.onPause()
        Log.d(TAG, "App paused - archives remain open for quick resume")
        // Note: We don't close archives on pause to allow quick resume
        // Archives will be closed in onDestroy or ViewModel.onCleared()
    }

    override fun onResume() {
        super.onResume()
        Log.d(TAG, "App resumed")
        // Any necessary resume logic can be added here
    }

    override fun onStop() {
        super.onStop()
        Log.d(TAG, "App stopped - preparing for potential termination")
        // App is no longer visible, but we keep archives open
        // as Android may just be switching to another app temporarily
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "App being destroyed - final cleanup")

        try {
            // The RepositoryViewModel.onCleared() will handle repository closing
            // This is just for additional Android-specific cleanup

            // Cleanup Android SAF resources
            ZipLockNative.cleanup()
            Log.d(TAG, "Android SAF cleanup completed")

        } catch (e: Exception) {
            Log.w(TAG, "Exception during final cleanup: ${e.message}")
        }
    }
}



@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainApp(
    repositoryViewModel: RepositoryViewModel,
    initialFileUri: String? = null
) {
    // Check for incoming file URI or last opened archive and determine initial screen
    val initialScreen = when {
        initialFileUri != null -> {
            // If opened from file, go directly to repository selection with the file pre-filled
            Screen.RepositorySelection(initialFileUri)
        }
        else -> {
            Screen.RepositorySelection()
        }
    }

    var currentScreen by remember { mutableStateOf<Screen>(initialScreen) }

    Scaffold(
        containerColor = ZipLockColors.LightBackground
    ) { paddingValues ->
        when (currentScreen) {
            Screen.AutoOpenLastArchive -> {
                // Auto-open screen for last used archive
                AutoOpenArchiveScreen(
                    repositoryViewModel = repositoryViewModel,
                    onArchiveOpened = { filePath ->
                        currentScreen = Screen.RepositoryOpened(filePath)
                    },
                    onSelectDifferent = {
                        currentScreen = Screen.RepositorySelection()
                    },
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }

            is Screen.RepositorySelection -> {
                val repositorySelectionScreen = currentScreen as Screen.RepositorySelection
                RepositorySelectionScreen(
                    repositoryViewModel = repositoryViewModel,
                    onArchiveOpened = { filePath ->
                        currentScreen = Screen.RepositoryOpened(filePath)
                    },
                    onCreateNew = {
                        currentScreen = Screen.CreateArchive
                    },
                    initialFilePath = repositorySelectionScreen.initialFilePath,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }

            Screen.CreateArchive -> {
                CreateArchiveWizard(
                    onArchiveCreated = { archivePath ->
                        // Archive created successfully, open it
                        currentScreen = Screen.RepositoryOpened(archivePath)
                    },
                    onCancel = {
                        currentScreen = Screen.RepositorySelection()
                    },
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }

            is Screen.RepositoryOpened -> {
                val repositoryScreen = currentScreen as Screen.RepositoryOpened
                RepositoryOpenedScreen(
                    onClose = {
                        println("MainActivity: onClose callback triggered, navigating to RepositorySelection")
                        currentScreen = Screen.RepositorySelection()
                        println("MainActivity: Navigation to RepositorySelection completed")
                    },
                    onAddCredential = {
                        currentScreen = Screen.CredentialTemplateSelection
                    },
                    onEditCredential = { credential ->
                        currentScreen = Screen.CredentialEdit(credential)
                    },
                    repositoryViewModel = repositoryViewModel,
                    shouldRefresh = repositoryScreen.shouldRefresh,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }

            Screen.CredentialTemplateSelection -> {
                CredentialTemplateSelectionScreen(
                    onTemplateSelected = { template ->
                        currentScreen = Screen.CredentialForm(convertFromZipLockNativeHelperTemplate(template))
                    },
                    onCancel = {
                        currentScreen = Screen.RepositoryOpened("", shouldRefresh = false)
                    },
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }

            is Screen.CredentialForm -> {
                val credentialFormScreen = currentScreen as Screen.CredentialForm
                val context = LocalContext.current
                val credentialFormViewModel: CredentialFormViewModel = viewModel(
                    factory = CredentialFormViewModelFactory(context)
                )
                val formUiState by credentialFormViewModel.uiState.collectAsState()

                CredentialFormScreen(
                    template = convertToZipLockNativeHelperTemplate(credentialFormScreen.template),
                    onSave = { title, fields, tags ->
                        credentialFormViewModel.saveCredential(
                            template = credentialFormScreen.template.name,
                            title = title,
                            fields = fields,
                            tags = tags,
                            onSuccess = {
                                // Navigate back to credentials list and trigger refresh
                                currentScreen = Screen.RepositoryOpened("", shouldRefresh = true)
                            },
                            onError = { error ->
                                // Error is handled by the ViewModel's UI state
                                println("Save credential error: $error")
                            }
                        )
                    },
                    onCancel = {
                        currentScreen = Screen.RepositoryOpened("", shouldRefresh = false)
                    },
                    isSaving = formUiState.isSaving,
                    errorMessage = formUiState.errorMessage,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }

            is Screen.CredentialEdit -> {
                val credentialEditScreen = currentScreen as Screen.CredentialEdit
                val context = LocalContext.current
                val credentialFormViewModel: CredentialFormViewModel = viewModel(
                    factory = CredentialFormViewModelFactory(context)
                )
                val formUiState by credentialFormViewModel.uiState.collectAsState()

                // Debug the credential being edited
                val editCredential = credentialEditScreen.credential
                println("MainActivity: Editing credential - ID: '${editCredential.id}', Title: '${editCredential.title}'")
                println("MainActivity: Edit credential fields: ${editCredential.fields.keys}")
                editCredential.fields.forEach { (key, field) ->
                    println("MainActivity: Edit field '$key' = '${field.value}' (${field.fieldType})")
                }

                // Use a basic template for editing (will be enhanced later)
                val template = createBasicTemplate(credentialEditScreen.credential.credentialType)

                CredentialFormScreen(
                    template = convertToZipLockNativeHelperTemplate(template),
                    existingCredential = credentialEditScreen.credential,
                    onSave = { title, fields, tags ->
                        credentialFormViewModel.updateCredential(
                            credentialId = credentialEditScreen.credential.id,
                            template = template.name,
                            title = title,
                            fields = fields,
                            tags = tags,
                            onSuccess = {
                                // Navigate back to credentials list and trigger refresh
                                currentScreen = Screen.RepositoryOpened("", shouldRefresh = true)
                            },
                            onError = { error ->
                                // Error is handled by the ViewModel's UI state
                                println("Update credential error: $error")
                            }
                        )
                    },
                    onCancel = {
                        currentScreen = Screen.RepositoryOpened("", shouldRefresh = false)
                    },
                    isSaving = formUiState.isSaving,
                    errorMessage = formUiState.errorMessage,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }
        }
    }
}

// Basic credential template for compatibility
data class BasicCredentialTemplate(
    val id: String,
    val name: String,
    val credentialType: String,
    val fields: List<TemplateField>
)

data class TemplateField(
    val id: String,
    val name: String,
    val fieldType: String,
    val required: Boolean = false,
    val sensitive: Boolean = false
)

// Helper function to create basic templates
fun createBasicTemplate(credentialType: String): BasicCredentialTemplate {
    return when (credentialType.lowercase()) {
        "login", "website" -> BasicCredentialTemplate(
            id = "basic_login",
            name = "Login",
            credentialType = "login",
            fields = listOf(
                TemplateField("username", "Username", "text", required = false),
                TemplateField("password", "Password", "password", required = false, sensitive = true),
                TemplateField("url", "Website URL", "url"),
                TemplateField("notes", "Notes", "textarea")
            )
        )
        "note", "secure_note" -> BasicCredentialTemplate(
            id = "basic_note",
            name = "Secure Note",
            credentialType = "note",
            fields = listOf(
                TemplateField("title", "Title", "text", required = true),
                TemplateField("content", "Content", "textarea", required = true, sensitive = true)
            )
        )
        else -> BasicCredentialTemplate(
            id = "basic_generic",
            name = "Generic",
            credentialType = credentialType,
            fields = listOf(
                TemplateField("title", "Title", "text", required = true),
                TemplateField("value", "Value", "text", sensitive = true),
                TemplateField("notes", "Notes", "textarea")
            )
        )
    }
}

// Helper function to convert ZipLockNativeHelper.CredentialTemplate to BasicCredentialTemplate
fun convertFromZipLockNativeHelperTemplate(template: ZipLockNativeHelper.CredentialTemplate): BasicCredentialTemplate {
    return BasicCredentialTemplate(
        id = template.id,
        name = template.name,
        credentialType = template.credentialType,
        fields = template.fields.map { field ->
            TemplateField(
                id = field.id,
                name = field.name,
                fieldType = field.fieldType,
                required = field.required,
                sensitive = field.sensitive
            )
        }
    )
}

// Helper function to convert BasicCredentialTemplate to ZipLockNativeHelper.CredentialTemplate
fun convertToZipLockNativeHelperTemplate(basicTemplate: BasicCredentialTemplate): ZipLockNativeHelper.CredentialTemplate {
    return ZipLockNativeHelper.CredentialTemplate(
        id = basicTemplate.id,
        name = basicTemplate.name,
        credentialType = basicTemplate.credentialType,
        description = "Converted from BasicCredentialTemplate",
        fields = basicTemplate.fields.map { field ->
            ZipLockNativeHelper.TemplateField(
                id = field.id,
                name = field.name,
                fieldType = field.fieldType,
                required = field.required,
                sensitive = field.sensitive,
                placeholder = "Enter ${field.name.lowercase()}"
            )
        },
        category = "general"
    )
}

sealed class Screen {
    object AutoOpenLastArchive : Screen()
    data class RepositorySelection(val initialFilePath: String? = null) : Screen()
    object CreateArchive : Screen()
    data class RepositoryOpened(val archivePath: String, val shouldRefresh: Boolean = false) : Screen()
    object CredentialTemplateSelection : Screen()
    data class CredentialForm(val template: BasicCredentialTemplate) : Screen()
    data class CredentialEdit(val credential: ZipLockNative.Credential) : Screen()
}

@Composable
fun AutoOpenArchiveScreen(
    repositoryViewModel: RepositoryViewModel,
    onArchiveOpened: (String) -> Unit,
    onSelectDifferent: () -> Unit,
    modifier: Modifier = Modifier
) {
    val lastArchivePath by repositoryViewModel.lastArchivePath.collectAsState()
    var passphrase by remember { mutableStateOf("") }

    val uiState by repositoryViewModel.uiState.collectAsState()
    val repositoryState by repositoryViewModel.repositoryState.collectAsState()

    LaunchedEffect(repositoryState) {
        when (val currentState = repositoryState) {
            is MobileRepositoryManager.RepositoryState -> {
                if (currentState.isOpen) {
                    onArchiveOpened(currentState.archiveName ?: "Unknown Archive")
                }
            }
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        // Header
        Text(
            text = "Welcome Back",
            style = MaterialTheme.typography.headlineMedium,
            color = ZipLockColors.DarkText,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Enter your passphrase to open:",
            style = MaterialTheme.typography.bodyMedium,
            color = ZipLockColors.LightGrayText,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(8.dp))

        // Show archive file name
        lastArchivePath?.let { path ->
            val fileName = path.substringAfterLast("/")
            Text(
                text = fileName,
                style = MaterialTheme.typography.bodyLarge,
                color = ZipLockColors.LogoPurple,
                textAlign = TextAlign.Center,
                fontWeight = FontWeight.Medium
            )
        }

        Spacer(modifier = Modifier.height(32.dp))

        // Passphrase input
        ZipLockTextInput(
            value = passphrase,
            onValueChange = { passphrase = it },
            placeholder = "Enter your passphrase",
            isPassword = true,
            imeAction = ImeAction.Done,
            keyboardActions = KeyboardActions(
                onDone = {
                    if (passphrase.isNotBlank() && !uiState.isLoading) {
                        lastArchivePath?.let { path: String ->
                            repositoryViewModel.openRepository(path, passphrase)
                        }
                    }
                }
            ),
            modifier = Modifier.fillMaxWidth()
        )

        // Error message
        uiState.errorMessage?.let { error ->
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = error,
                color = ZipLockColors.ErrorRed,
                style = MaterialTheme.typography.bodySmall,
                textAlign = TextAlign.Center
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Open button
        ZipLockButton(
            text = if (uiState.isLoading) "Opening..." else "Open Archive",
            onClick = {
                lastArchivePath?.let { path: String ->
                    repositoryViewModel.openRepository(path, passphrase)
                }
            },
            style = ZipLockButtonStyle.Primary,
            enabled = passphrase.isNotBlank() && !uiState.isLoading,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(16.dp))

        // Select different archive button
        ZipLockButton(
            text = "Choose Different Archive",
            onClick = onSelectDifferent,
            style = ZipLockButtonStyle.Secondary,
            enabled = !uiState.isLoading,
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
fun RepositoryOpenedScreen(
    onClose: () -> Unit,
    onAddCredential: () -> Unit,
    onEditCredential: (ZipLockNative.Credential) -> Unit,
    repositoryViewModel: RepositoryViewModel,
    shouldRefresh: Boolean = false,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val credentialsViewModel: CredentialsViewModel = viewModel(
        factory = CredentialsViewModelFactory(context)
    )
    val credentialsUiState by credentialsViewModel.uiState.collectAsState()
    val searchQuery by credentialsViewModel.searchQuery.collectAsState()
    // Watch for repository state changes and load credentials when archive is fully opened
    val repositoryState by repositoryViewModel.repositoryState.collectAsState()
    LaunchedEffect(repositoryState) {
        when (val currentState = repositoryState) {
            is MobileRepositoryManager.RepositoryState -> {
                if (currentState.isOpen) {
                    // Repository is confirmed open, now we can safely load credentials
                    println("MainActivity: Repository confirmed open: ${currentState.archiveName}")
                    println("MainActivity: Waiting briefly for background initialization to complete...")
                    // Small delay to ensure all background initialization has completed
                    delay(500)
                    println("MainActivity: Loading credentials now that archive is fully ready...")
                    credentialsViewModel.loadCredentials()
                } else {
                    println("MainActivity: Repository state is: $currentState")
                }
            }
            else -> {
                println("MainActivity: Unknown repository state: $repositoryState")
            }
        }
    }

    // Refresh credentials when shouldRefresh is true
    LaunchedEffect(shouldRefresh) {
        if (shouldRefresh) {
            credentialsViewModel.refresh()
        }
    }

    // Always render credentials UI - let the parent handle navigation
    CredentialsListScreen(
        credentials = credentialsUiState.credentials,
        searchQuery = searchQuery,
        onSearchQueryChange = { query ->
            credentialsViewModel.updateSearchQuery(query)
        },
        onCredentialClick = { credential ->
            val credentialTitle = (credential["title"] as? String) ?: "Unknown"
            println("MainActivity: Credential clicked: $credentialTitle")

            // Convert Map<String, Any> to ZipLockNative.Credential for compatibility
            val zipLockCredential = ZipLockNative.Credential(
                id = (credential["id"] as? String) ?: "",
                title = (credential["title"] as? String) ?: "",
                credentialType = (credential["credentialType"] as? String) ?: "login",
                fields = (credential["fields"] as? Map<String, Any>)?.mapValues { (_, value) ->
                    when (value) {
                        is Map<*, *> -> ZipLockNative.FieldValue(
                            value = (value["value"] as? String) ?: "",
                            fieldType = (value["fieldType"] as? String) ?: "text",
                            label = value["label"] as? String,
                            sensitive = (value["sensitive"] as? Boolean) ?: false
                        )
                        else -> ZipLockNative.FieldValue(
                            value = value.toString(),
                            fieldType = "text"
                        )
                    }
                } ?: emptyMap(),
                createdAt = (credential["createdAt"] as? Long) ?: System.currentTimeMillis(),
                updatedAt = (credential["updatedAt"] as? Long) ?: System.currentTimeMillis(),
                tags = (credential["tags"] as? List<*>)?.mapNotNull { it as? String } ?: emptyList()
            )
            onEditCredential(zipLockCredential)
        },
        onCloseArchive = {
            // Close the archive and navigate back
            println("MainActivity: Close archive button clicked")

            // Close both the credentials view model and the repository view model
            credentialsViewModel.clearCredentials()
            repositoryViewModel.closeRepository()
            println("MainActivity: Archive closed")

            // Clear the credentials state to prevent stale data
            credentialsViewModel.clearCredentialsState()

            println("MainActivity: Navigating back to repository selection")
            onClose()
            println("MainActivity: Navigation completed")
        },
        onAddCredential = onAddCredential,
        onRefresh = {
            // Refresh credentials from the archive
            credentialsViewModel.loadCredentials()
        },
        isLoading = credentialsUiState.isLoading,
        errorMessage = credentialsUiState.errorMessage,
        modifier = modifier
    )
}

@Composable
fun ZipLockTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = lightColorScheme(
            primary = ZipLockColors.LogoPurple,
            onPrimary = ZipLockColors.White,
            secondary = ZipLockColors.LogoPurpleLight,
            onSecondary = ZipLockColors.White,
            tertiary = ZipLockColors.SuccessGreen,
            onTertiary = ZipLockColors.White,
            error = ZipLockColors.ErrorRed,
            onError = ZipLockColors.White,
            background = ZipLockColors.LightBackground,
            onBackground = ZipLockColors.DarkText,
            surface = ZipLockColors.White,
            onSurface = ZipLockColors.DarkText,
            surfaceVariant = ZipLockColors.VeryLightGray,
            onSurfaceVariant = ZipLockColors.LightGrayText
        ),
        typography = Typography(
            displayLarge = ZipLockTypography.ExtraLarge,
            displayMedium = ZipLockTypography.Large,
            displaySmall = ZipLockTypography.Header,
            headlineLarge = ZipLockTypography.Large,
            headlineMedium = ZipLockTypography.Header,
            headlineSmall = ZipLockTypography.Medium,
            titleLarge = ZipLockTypography.Header,
            titleMedium = ZipLockTypography.Medium,
            titleSmall = ZipLockTypography.Normal,
            bodyLarge = ZipLockTypography.Normal,
            bodyMedium = ZipLockTypography.Normal,
            bodySmall = ZipLockTypography.Small,
            labelLarge = ZipLockTypography.Medium,
            labelMedium = ZipLockTypography.Normal,
            labelSmall = ZipLockTypography.Small
        ),
        content = content
    )
}

@Preview(showBackground = true)
@Composable
fun MainAppPreview() {
    ZipLockTheme {
        // Create a mock view model for preview with mock context
        val mockContext = androidx.compose.ui.platform.LocalContext.current
        val mockViewModel = RepositoryViewModel(mockContext)
        MainApp(repositoryViewModel = mockViewModel)
    }
}
