package com.ziplock.ui.screens

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.documentfile.provider.DocumentFile
import com.ziplock.R
import com.ziplock.ui.theme.*
import com.ziplock.utils.FileUtils


/**
 * Repository Selection Screen
 *
 * This screen replicates the Linux UI functionality for selecting an archive file
 * and providing a passphrase to unlock it. It provides:
 *
 * - File picker for selecting .7z archive files
 * - Passphrase input field with visibility toggle
 * - Real-time validation feedback
 * - Loading states and error handling
 * - Consistent styling with the Linux implementation
 */
@Composable
fun RepositorySelectionScreen(
    hybridRepositoryViewModel: com.ziplock.viewmodel.HybridRepositoryViewModel? = null,
    repositoryViewModel: com.ziplock.viewmodel.RepositoryViewModel? = null,
    onArchiveOpened: (String) -> Unit,
    onCreateNew: () -> Unit,
    modifier: Modifier = Modifier,
    initialFilePath: String? = null
) {
    // Use hybrid view model by default, fallback to legacy if provided
    val hybridVM = hybridRepositoryViewModel
    val legacyVM = repositoryViewModel
    require(hybridVM != null || legacyVM != null) { "Either hybridRepositoryViewModel or repositoryViewModel must be provided" }

    // Handle different view model types
    val isLoading = if (hybridVM != null) {
        val hybridState by hybridVM.uiState.collectAsState()
        hybridState.isLoading
    } else {
        val legacyState by legacyVM!!.uiState.collectAsState()
        legacyState.isLoading
    }

    val errorMessage = if (hybridVM != null) {
        val hybridState by hybridVM.uiState.collectAsState()
        hybridState.errorMessage
    } else {
        val legacyState by legacyVM!!.uiState.collectAsState()
        legacyState.errorMessage
    }

    var selectedFilePath by remember { mutableStateOf<String?>(initialFilePath) }

    // For hybrid VM, watch repository state for success
    if (hybridVM != null) {
        val repositoryState by hybridVM.repositoryState.collectAsState()
        LaunchedEffect(repositoryState) {
            val currentState = repositoryState
            if (currentState is com.ziplock.viewmodel.HybridRepositoryViewModel.HybridRepositoryState.Open) {
                onArchiveOpened(currentState.path)
            }
        }
    }

    // For legacy VM, watch UI state for success
    if (legacyVM != null) {
        val legacyState by legacyVM.uiState.collectAsState()
        LaunchedEffect(legacyState.successMessage) {
            if (legacyState.successMessage != null) {
                selectedFilePath?.let { onArchiveOpened(it) }
            }
        }
    }
    var selectedFileName by remember { mutableStateOf<String?>(
        initialFilePath?.let { path ->
            extractUserFriendlyFileName(path)
        }
    ) }
    var passphrase by remember { mutableStateOf("") }
    var localErrorMessage by remember { mutableStateOf<String?>(null) }

    // Use repository view model state for loading and errors
    LaunchedEffect(errorMessage) {
        if (errorMessage != null) {
            localErrorMessage = errorMessage
        }
    }
    var passphraseError by remember { mutableStateOf<String?>(null) }

    val context = LocalContext.current

    // File picker launcher for selecting .7z files
    val filePickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.OpenDocument()
    ) { uri ->
        uri?.let {
            // Get the actual file path and name
            val documentFile = DocumentFile.fromSingleUri(context, it)
            selectedFileName = documentFile?.name
            selectedFilePath = it.toString()

            // Clear any previous errors when a new file is selected
            localErrorMessage = null
        }
    }

    // Validation logic
    val isValidForm = selectedFilePath != null &&
                     passphrase.isNotBlank() &&
                     passphraseError == null

    // Passphrase validation
    LaunchedEffect(passphrase) {
        passphraseError = when {
            passphrase.isEmpty() -> null
            passphrase.length < 3 -> "Passphrase too short"
            else -> null
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(ZipLockColors.LightBackground)
            .padding(ZipLockSpacing.MainContentPadding)
            .verticalScroll(rememberScrollState()),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Spacer(modifier = Modifier.height(ZipLockSpacing.Large))

        // Logo and Title
        Image(
            painter = painterResource(id = R.drawable.ziplock_icon_512),
            contentDescription = "ZipLock Logo",
            modifier = Modifier
                .size(ZipLockDimensions.LogoSize)
                .padding(bottom = ZipLockSpacing.Large)
        )

        Text(
            text = "Open Archive",
            style = ZipLockTypography.ExtraLarge,
            color = ZipLockColors.LogoPurple,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(bottom = ZipLockSpacing.Small)
        )

        Text(
            text = "Select your password archive and enter your passphrase",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(bottom = ZipLockSpacing.ExtraLarge)
        )

        // Main content card
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
                // Error alert if present
                (errorMessage ?: localErrorMessage)?.let { error ->
                    ZipLockAlert(
                        level = AlertLevel.Error,
                        message = error,
                        onDismiss = {
                            localErrorMessage = null
                        },
                        modifier = Modifier.padding(bottom = ZipLockSpacing.Standard)
                    )
                }

                // File selection section
                Text(
                    text = "Archive File",
                    style = ZipLockTypography.Medium,
                    color = ZipLockColors.DarkText,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = ZipLockSpacing.Small)
                )

                ZipLockFilePicker(
                    selectedFileName = selectedFileName,
                    onFileSelect = {
                        // Launch file picker with .7z filter
                        filePickerLauncher.launch(arrayOf("application/x-7z-compressed"))
                    },
                    placeholder = "Select archive file (.7z)",
                    modifier = Modifier.padding(bottom = ZipLockSpacing.Standard)
                )

                // Passphrase section
                Text(
                    text = "Passphrase",
                    style = ZipLockTypography.Medium,
                    color = ZipLockColors.DarkText,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = ZipLockSpacing.Small)
                )

                ZipLockTextInput(
                    value = passphrase,
                    onValueChange = { passphrase = it },
                    placeholder = "Enter your passphrase",
                    isPassword = true,
                    style = when {
                        passphraseError != null -> ZipLockTextInputStyle.Invalid
                        passphrase.isNotBlank() && passphraseError == null -> ZipLockTextInputStyle.Valid
                        else -> ZipLockTextInputStyle.Standard
                    },
                    imeAction = ImeAction.Done,
                    keyboardActions = KeyboardActions(
                        onDone = {
                            if (passphrase.isNotBlank() && !isLoading && selectedFilePath != null) {
                                val path = selectedFilePath!!
                                localErrorMessage = null

                                try {
                                    // Call appropriate view model method
                                    if (hybridVM != null) {
                                        // For hybrid system, pass original path (it will handle content URI conversion internally)
                                        println("RepositorySelectionScreen: Opening with hybrid system: '$path'")
                                        hybridVM.openRepository(path, passphrase)
                                    } else {
                                        // For legacy system, convert content URI to usable file path
                                        val usableFilePath = if (path.startsWith("content://")) {
                                            val uri = android.net.Uri.parse(path)
                                            val fileName = selectedFileName ?: "archive.7z"
                                            FileUtils.getUsableFilePath(context, uri, fileName)
                                        } else {
                                            path
                                        }
                                        println("RepositorySelectionScreen: Converting path '$path' to '$usableFilePath' for legacy system")
                                        legacyVM?.let { vm ->
                                            vm.openRepository(usableFilePath, passphrase)
                                        }
                                    }
                                } catch (e: Exception) {
                                    val errorMsg = when {
                                        e.message?.contains("authentication", ignoreCase = true) == true ->
                                            "Incorrect passphrase. Please check your password and try again."
                                        e.message?.contains("not found", ignoreCase = true) == true ->
                                            "The archive file could not be found. Please check the file path."
                                        e.message?.contains("permission", ignoreCase = true) == true ->
                                            "Permission denied. Please check file permissions."
                                        else -> "Failed to open archive. Please try again."
                                    }
                                    localErrorMessage = errorMsg
                                    passphraseError = errorMsg
                                }
                            }
                        }
                    ),
                    leadingIcon = ZipLockIcons.Lock,
                    modifier = Modifier.padding(bottom = ZipLockSpacing.Small)
                )

                // Passphrase error message
                passphraseError?.let { error ->
                    Text(
                        text = error,
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.ErrorRed,
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(bottom = ZipLockSpacing.Standard)
                    )
                }

                Spacer(modifier = Modifier.height(ZipLockSpacing.Large))

                // Action buttons
                if (isLoading) {
                    ZipLockLoadingIndicator(
                        message = "Opening archive...",
                        modifier = Modifier.padding(ZipLockSpacing.Large)
                    )
                } else {
                    // Open Archive button
                    ZipLockButton(
                        text = "Open Archive",
                        onClick = {
                            selectedFilePath?.let { path ->
                                localErrorMessage = null

                                try {
                                    // Call appropriate view model method
                                    if (hybridVM != null) {
                                        // For hybrid system, pass original path (it will handle content URI conversion internally)
                                        println("RepositorySelectionScreen: Opening with hybrid system: '$path'")
                                        hybridVM.openRepository(path, passphrase)
                                    } else {
                                        // For legacy system, convert content URI to usable file path
                                        val usableFilePath = if (path.startsWith("content://")) {
                                            val uri = android.net.Uri.parse(path)
                                            val fileName = selectedFileName ?: "archive.7z"
                                            FileUtils.getUsableFilePath(context, uri, fileName)
                                        } else {
                                            path
                                        }
                                        println("RepositorySelectionScreen: Converting path '$path' to '$usableFilePath' for legacy system")
                                        legacyVM?.let { vm ->
                                            vm.openRepository(usableFilePath, passphrase)
                                        }
                                    }
                                } catch (e: Exception) {
                                    val errorMsg = when {
                                        e.message?.contains("authentication", ignoreCase = true) == true ->
                                            "Incorrect passphrase. Please check your password and try again."
                                        e.message?.contains("not found", ignoreCase = true) == true ->
                                            "The archive file could not be found. Please check the file path."
                                        e.message?.contains("permission", ignoreCase = true) == true ->
                                            "Permission denied. Please check file permissions."
                                        else -> "Failed to open archive. Please try again."
                                    }
                                    localErrorMessage = errorMsg
                                }
                            }
                        },
                        enabled = isValidForm,
                        icon = ZipLockIcons.FolderOpen,
                        style = ZipLockButtonStyle.Primary,
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(bottom = ZipLockSpacing.Medium)
                    )


                }
            }
        }

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        // Create New Archive button
        ZipLockButton(
            text = "Create New Archive",
            onClick = onCreateNew,
            icon = ZipLockIcons.Plus,
            style = ZipLockButtonStyle.Secondary,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        // Help text
        Text(
            text = "Need help? ZipLock uses 7z archives to securely store your passwords with AES encryption.",
            style = ZipLockTypography.Small,
            color = ZipLockColors.LightGrayText,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(horizontal = ZipLockSpacing.Standard)
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Large))
    }
}

/**
 * Preview for the Repository Selection Screen
 */
/**
 * Extract a user-friendly filename from a file path or content URI
 */
private fun extractUserFriendlyFileName(path: String): String {
    return when {
        path.startsWith("content://") -> {
            // Handle Android content URIs
            try {
                // First try to extract from the document ID part
                val documentId = path.substringAfterLast("/")

                // Decode URL encoding
                val decoded = java.net.URLDecoder.decode(documentId, "UTF-8")

                // Extract filename from various content URI formats
                when {
                    // Format: "primary:Documents/filename.7z" or "1234:filename.7z"
                    decoded.contains(":") && decoded.contains("/") -> {
                        decoded.substringAfterLast("/")
                    }
                    // Format: "primary:filename.7z"
                    decoded.contains(":") -> {
                        decoded.substringAfterLast(":")
                    }
                    // Format: just "filename.7z"
                    decoded.contains(".") -> {
                        decoded
                    }
                    // Fallback
                    else -> "Selected Archive"
                }
            } catch (e: Exception) {
                // If decoding fails, try simple extraction
                path.substringAfterLast("/").takeIf {
                    it.isNotEmpty() && it.contains(".")
                } ?: "Selected Archive"
            }
        }
        else -> {
            // Handle regular file paths
            path.substringAfterLast("/")
        }
    }
}

@Preview(showBackground = true)
@Preview
@Composable
fun RepositorySelectionScreenPreview() {
    val context = LocalContext.current
    RepositorySelectionScreen(
        repositoryViewModel = com.ziplock.viewmodel.RepositoryViewModel(context),
        onArchiveOpened = { _ -> },
        onCreateNew = { }
    )
}
