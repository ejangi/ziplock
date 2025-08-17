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
import com.ziplock.ui.screens.CreateArchiveWizard
import com.ziplock.ui.screens.RepositorySelectionScreen
import com.ziplock.ui.theme.*
import com.ziplock.viewmodel.RepositoryViewModel
import com.ziplock.viewmodel.RepositoryState
import com.ziplock.viewmodel.CreateArchiveViewModel

class MainActivity : ComponentActivity() {

    private val repositoryViewModel: RepositoryViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Initialize the native library
        try {
            val initResult = ZipLockNative.init()
            if (initResult) {
                Log.d("MainActivity", "ZipLock native library initialized successfully")
            } else {
                Log.e("MainActivity", "Failed to initialize ZipLock native library")
            }
        } catch (e: Exception) {
            Log.e("MainActivity", "Error initializing ZipLock native library: ${e.message}")
        }

        setContent {
            ZipLockTheme {
                MainApp(repositoryViewModel = repositoryViewModel)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainApp(repositoryViewModel: RepositoryViewModel) {
    var currentScreen by remember { mutableStateOf<Screen>(Screen.RepositorySelection) }

    Scaffold(
        containerColor = ZipLockColors.LightBackground
    ) { paddingValues ->
        when (currentScreen) {
            Screen.RepositorySelection -> {
                RepositorySelectionScreen(
                    onRepositorySelected = { filePath, passphrase ->
                        // TODO: Open the repository and navigate to main screen
                        println("Selected file: $filePath")
                        println("Passphrase length: ${passphrase.length}")
                        currentScreen = Screen.RepositoryOpened(filePath)
                    },
                    onCreateNew = {
                        currentScreen = Screen.CreateArchive
                    },
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
                        currentScreen = Screen.RepositorySelection
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
                        currentScreen = Screen.RepositorySelection
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
    object RepositorySelection : Screen()
    object CreateArchive : Screen()
    data class RepositoryOpened(val archivePath: String) : Screen()
}

@Composable
fun RepositoryOpenedScreen(
    archivePath: String,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    // Placeholder for the main password manager interface
    // This will be implemented in future iterations

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(ZipLockSpacing.MainContentPadding),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = "Repository Opened",
            style = ZipLockTypography.ExtraLarge,
            color = ZipLockColors.LogoPurple
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "Archive: ${archivePath.substringAfterLast('/')}",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.DarkText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ZipLockColors.White),
            elevation = CardDefaults.cardElevation(defaultElevation = ZipLockDimensions.CardElevation)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(ZipLockSpacing.ExtraLarge),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "🔓",
                    style = ZipLockTypography.ExtraLarge.copy(fontSize = 48.sp)
                )

                Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

                Text(
                    text = "Archive Unlocked Successfully",
                    style = ZipLockTypography.Header,
                    color = ZipLockColors.DarkText
                )

                Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

                Text(
                    text = "The main password manager interface will be implemented here. You can now access your encrypted credentials.",
                    style = ZipLockTypography.Normal,
                    color = ZipLockColors.LightGrayText,
                    textAlign = TextAlign.Center
                )

                Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

                ZipLockButton(
                    text = "Close Archive",
                    onClick = onClose,
                    style = ZipLockButtonStyle.Secondary,
                    icon = ZipLockIcons.Lock,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }
    }
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
        // Create a mock view model for preview
        val mockViewModel = RepositoryViewModel()
        MainApp(repositoryViewModel = mockViewModel)
    }
}
