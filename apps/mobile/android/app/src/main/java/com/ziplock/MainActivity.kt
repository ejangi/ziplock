package com.ziplock

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import com.ziplock.ffi.ZipLockNative
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.compose.ui.text.font.FontWeight
import com.ziplock.ui.screens.CreateArchiveWizard
import com.ziplock.ui.screens.RepositorySelectionScreen
import com.ziplock.ui.screens.CredentialsListScreen
import com.ziplock.ui.theme.*
import com.ziplock.viewmodel.RepositoryViewModel
import com.ziplock.viewmodel.RepositoryViewModelFactory
import com.ziplock.viewmodel.CredentialsViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.compose.runtime.collectAsState
import com.ziplock.viewmodel.RepositoryState
import com.ziplock.viewmodel.CreateArchiveViewModel

class MainActivity : ComponentActivity() {

    private val repositoryViewModel: RepositoryViewModel by lazy {
        ViewModelProvider(this, RepositoryViewModelFactory(this))[RepositoryViewModel::class.java]
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Check for incoming file URI from intent
        val fileUri = intent.getStringExtra("file_uri")
        val openedFromFile = intent.getBooleanExtra("opened_from_file", false)

        // Initialize the native library
        try {
            println("MainActivity: Initializing ZipLock native library...")
            val initResult = ZipLockNative.init()
            println("MainActivity: Init result: $initResult")

            if (initResult) {
                Log.d("MainActivity", "ZipLock native library initialized successfully")
                println("MainActivity: ZipLock native library initialized successfully")



                // Get library version
                val version = ZipLockNative.getVersion()
                println("MainActivity: Library version: $version")
                Log.d("MainActivity", "ZipLock library version: $version")
            } else {
                Log.e("MainActivity", "Failed to initialize ZipLock native library")
                println("MainActivity: ERROR - Failed to initialize ZipLock native library")
            }
        } catch (e: Exception) {
            Log.e("MainActivity", "Error initializing ZipLock native library: ${e.message}")
            println("MainActivity: EXCEPTION - Error initializing ZipLock native library: ${e.message}")
            e.printStackTrace()
        }

        setContent {
            ZipLockTheme {
                MainApp(
                    repositoryViewModel = repositoryViewModel,
                    initialFileUri = if (openedFromFile) fileUri else null
                )
            }
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
        repositoryViewModel.hasValidLastArchive() -> {
            Screen.AutoOpenLastArchive
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
                    onRepositorySelected = { filePath, passphrase ->
                        println("Selected file: $filePath")
                        println("Passphrase length: ${passphrase.length}")
                        repositoryViewModel.openRepository(filePath, passphrase)
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
                    archivePath = repositoryScreen.archivePath,
                    onClose = {
                        currentScreen = Screen.RepositorySelection(repositoryScreen.archivePath)
                    },
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(paddingValues)
                )
            }
        }
    }
}

sealed class Screen {
    object AutoOpenLastArchive : Screen()
    data class RepositorySelection(val initialFilePath: String? = null) : Screen()
    object CreateArchive : Screen()
    data class RepositoryOpened(val archivePath: String) : Screen()
}

@Composable
fun AutoOpenArchiveScreen(
    repositoryViewModel: RepositoryViewModel,
    onArchiveOpened: (String) -> Unit,
    onSelectDifferent: () -> Unit,
    modifier: Modifier = Modifier
) {
    val lastArchivePath = repositoryViewModel.getLastOpenedArchivePath()
    var passphrase by remember { mutableStateOf("") }

    val uiState by repositoryViewModel.uiState.collectAsState()

    LaunchedEffect(uiState.successMessage) {
        if (uiState.successMessage != null) {
            lastArchivePath?.let { onArchiveOpened(it) }
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
                lastArchivePath?.let { path ->
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
    archivePath: String,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    // Create credentials view model
    val credentialsViewModel: CredentialsViewModel = viewModel()
    val credentialsUiState by credentialsViewModel.uiState.collectAsState()
    val searchQuery by credentialsViewModel.searchQuery.collectAsState()

    CredentialsListScreen(
        credentials = credentialsUiState.credentials,
        searchQuery = searchQuery,
        onSearchQueryChange = { query ->
            credentialsViewModel.updateSearchQuery(query)
        },
        onCredentialClick = { credential ->
            credentialsViewModel.selectCredential(credential)
        },
        onCloseArchive = {
            // Close the archive through the credentials view model
            if (credentialsViewModel.closeArchive()) {
                onClose()
            }
        },
        onAddCredential = {
            // TODO: Navigate to add credential screen
            println("Add credential button clicked")
        },
        onLoadMockData = {
            // Development feature: Load mock data for testing
            credentialsViewModel.loadMockCredentials()
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
