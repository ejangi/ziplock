package com.ziplock.ui.screens

import android.content.Context
import android.util.Log
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.documentfile.provider.DocumentFile
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.ziplock.R
import com.ziplock.ffi.ZipLockNative
import com.ziplock.ui.theme.*
import com.ziplock.viewmodel.CreateArchiveViewModel
import com.ziplock.viewmodel.CreateArchiveStep

/**
 * Create Archive Wizard
 *
 * A multi-step wizard for creating new ZipLock archives that guides users through:
 * 1. Welcome and introduction
 * 2. Destination folder selection (supports cloud storage)
 * 3. Archive name input
 * 4. Passphrase creation with real-time validation
 * 5. Passphrase confirmation
 * 6. Archive creation progress
 *
 * This matches the Linux wizard functionality and integrates with the FFI shared library.
 */



@Composable
fun CreateArchiveWizard(
    onArchiveCreated: (String) -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
    viewModel: CreateArchiveViewModel = viewModel()
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val passphraseStrength: ZipLockNative.PassphraseStrengthResult? by viewModel.passphraseStrength.collectAsStateWithLifecycle()
    val context = LocalContext.current

    // File picker for destination directory
    val directoryPickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.OpenDocumentTree()
    ) { uri ->
        uri?.let {
            val documentFile = DocumentFile.fromTreeUri(context, it)
            viewModel.setDestination(
                path = it.toString(),
                name = documentFile?.name ?: "Selected Folder"
            )
        }
    }

    // Handle successful archive creation
    LaunchedEffect(uiState.currentStep, uiState.createdArchivePath) {
        if (uiState.currentStep == CreateArchiveStep.Success && uiState.createdArchivePath != null) {
            // Archive was created successfully, but only call onArchiveCreated when user explicitly chooses to open it
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
        // Header with logo and progress
        WizardHeader(
            currentStep = uiState.currentStep,
            onCancel = {
                viewModel.reset()
                onCancel()
            }
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Large))

        // Main content card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = ZipLockColors.White),
            elevation = CardDefaults.cardElevation(defaultElevation = ZipLockDimensions.CardElevation)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(ZipLockSpacing.ExtraLarge)
            ) {


                // Step content
                when (uiState.currentStep) {
                    CreateArchiveStep.SelectDestination -> SelectDestinationStep(
                        destinationName = uiState.destinationName,
                        onSelectDestination = { directoryPickerLauncher.launch(null) },
                        onNext = {
                            Log.d("CreateArchiveWizard", "SelectDestination onNext clicked")
                            viewModel.proceedToNext()
                        },
                        onBack = {
                            Log.d("CreateArchiveWizard", "SelectDestination onBack clicked")
                            viewModel.goBack()
                        },
                        canProceed = viewModel.canProceed()
                    )

                    CreateArchiveStep.ArchiveName -> ArchiveNameStep(
                        archiveName = uiState.archiveName,
                        onArchiveNameChange = { viewModel.updateArchiveName(it) },
                        onNext = {
                            Log.d("CreateArchiveWizard", "ArchiveName onNext clicked")
                            viewModel.proceedToNext()
                        },
                        onBack = {
                            Log.d("CreateArchiveWizard", "ArchiveName onBack clicked")
                            viewModel.goBack()
                        },
                        canProceed = viewModel.canProceed()
                    )

                    CreateArchiveStep.CreatePassphrase -> CreatePassphraseStep(
                        passphrase = uiState.passphrase,
                        showPassphrase = uiState.showPassphrase,
                        passphraseStrength = passphraseStrength,
                        onPassphraseChange = { viewModel.updatePassphrase(it) },
                        onToggleVisibility = { viewModel.togglePassphraseVisibility() },
                        onNext = {
                            Log.d("CreateArchiveWizard", "CreatePassphrase onNext clicked")
                            viewModel.proceedToNext()
                        },
                        onBack = {
                            Log.d("CreateArchiveWizard", "CreatePassphrase onBack clicked")
                            viewModel.goBack()
                        },
                        canProceed = viewModel.canProceed()
                    )

                    CreateArchiveStep.ConfirmPassphrase -> ConfirmPassphraseStep(
                        passphrase = uiState.passphrase,
                        confirmPassphrase = uiState.confirmPassphrase,
                        showConfirmPassphrase = uiState.showConfirmPassphrase,
                        onConfirmPassphraseChange = { viewModel.updateConfirmPassphrase(it) },
                        onToggleVisibility = { viewModel.toggleConfirmPassphraseVisibility() },
                        onNext = {
                            Log.d("CreateArchiveWizard", "ConfirmPassphrase onNext clicked (Create Archive)")
                            // Use the new context-aware archive creation method
                            if (uiState.passphrase == uiState.confirmPassphrase) {
                                viewModel.startArchiveCreation(context)
                            } else {
                                viewModel.proceedToNext() // This will show the error message
                            }
                        },
                        onBack = {
                            Log.d("CreateArchiveWizard", "ConfirmPassphrase onBack clicked")
                            viewModel.goBack()
                        },
                        canProceed = viewModel.canProceed()
                    )

                    CreateArchiveStep.Creating -> CreatingStep(
                        progress = uiState.creationProgress,
                        archiveName = uiState.archiveName
                    )

                    CreateArchiveStep.Success -> SuccessStep(
                        archivePath = uiState.createdArchivePath ?: "",
                        onOpenArchive = {
                            uiState.createdArchivePath?.let { onArchiveCreated(it) }
                        },
                        onCreateAnother = {
                            viewModel.reset()
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun WizardHeader(
    currentStep: CreateArchiveStep,
    onCancel: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Cancel button
        ZipLockButton(
            text = "Cancel",
            onClick = onCancel,
            style = ZipLockButtonStyle.Secondary,
            icon = ZipLockIcons.Close,
            modifier = Modifier.widthIn(min = 100.dp)
        )

        // Logo
        Image(
            painter = painterResource(id = R.drawable.ziplock_icon_512),
            contentDescription = "ZipLock Logo",
            modifier = Modifier.size(48.dp)
        )

        // Progress indicator (steps 1-5 show progress)
        if (currentStep != CreateArchiveStep.Success) {
            val (currentStepNumber, totalSteps) = getStepProgress(currentStep)

            Column(
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "$currentStepNumber / $totalSteps",
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.LightGrayText
                )

                // Mini progress bar
                LinearProgressIndicator(
                    progress = currentStepNumber.toFloat() / totalSteps,
                    modifier = Modifier.width(60.dp).height(2.dp),
                    color = ZipLockColors.LogoPurple,
                    trackColor = ZipLockColors.VeryLightGray
                )
            }
        } else {
            Spacer(modifier = Modifier.width(100.dp))
        }
    }
}

@Composable
private fun SelectDestinationStep(
    destinationName: String?,
    onSelectDestination: () -> Unit,
    onNext: () -> Unit,
    onBack: () -> Unit,
    canProceed: Boolean
) {
    Column {
        Text(
            text = "Create New Archive",
            style = ZipLockTypography.ExtraLarge,
            color = ZipLockColors.LogoPurple,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "Welcome to the ZipLock archive creation wizard. This will guide you through creating a new encrypted password archive.",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Text(
            text = "Select Destination",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "Choose where to save your new archive. You can select local storage or cloud storage folders.",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Text(
            text = "Destination Folder",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.DarkText,
            modifier = Modifier.padding(bottom = ZipLockSpacing.Small)
        )

        ZipLockFilePicker(
            selectedFileName = destinationName,
            onFileSelect = onSelectDestination,
            placeholder = "Select destination folder..."
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "💡 Tip: ZipLock supports both local storage and cloud services like Google Drive, Dropbox, and OneDrive.",
            style = ZipLockTypography.Small,
            color = ZipLockColors.LightGrayText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        WizardNavigationButtons(
            onBack = onBack,
            onNext = onNext,
            canProceed = canProceed,
            nextText = "Continue"
        )
    }
}

@Composable
private fun ArchiveNameStep(
    archiveName: String,
    onArchiveNameChange: (String) -> Unit,
    onNext: () -> Unit,
    onBack: () -> Unit,
    canProceed: Boolean
) {
    Column {
        Text(
            text = "Archive Name",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "Enter a name for your password archive. This will be the filename of your .7z archive.",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Text(
            text = "Archive Name",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.DarkText,
            modifier = Modifier.padding(bottom = ZipLockSpacing.Small)
        )

        ZipLockTextInput(
            value = archiveName,
            onValueChange = onArchiveNameChange,
            placeholder = "Enter archive name",
            style = if (archiveName.isNotBlank()) ZipLockTextInputStyle.Valid else ZipLockTextInputStyle.Standard,
            imeAction = ImeAction.Next,
            leadingIcon = ZipLockIcons.Archive
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        // Archive name validation feedback
        if (archiveName.isNotBlank()) {
            val validationError = validateArchiveNameClient(archiveName)
            if (validationError != null) {
                Text(
                    text = "⚠️ $validationError",
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.ErrorRed
                )
            } else {
                Text(
                    text = "The archive will be saved as: $archiveName.7z",
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.LightGrayText
                )
            }
        } else {
            Text(
                text = "The archive will be saved as: $archiveName.7z",
                style = ZipLockTypography.Small,
                color = ZipLockColors.LightGrayText
            )
        }

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        WizardNavigationButtons(
            onBack = onBack,
            onNext = onNext,
            canProceed = canProceed,
            nextText = "Continue"
        )
    }
}

@Composable
private fun CreatePassphraseStep(
    passphrase: String,
    showPassphrase: Boolean,
    passphraseStrength: ZipLockNative.PassphraseStrengthResult?,
    onPassphraseChange: (String) -> Unit,
    onToggleVisibility: () -> Unit,
    onNext: () -> Unit,
    onBack: () -> Unit,
    canProceed: Boolean
) {
    Column {
        Text(
            text = "Create Passphrase",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "Create a strong master passphrase to protect your archive. This cannot be recovered if forgotten!",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Text(
            text = "Master Passphrase",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.DarkText,
            modifier = Modifier.padding(bottom = ZipLockSpacing.Small)
        )

        ZipLockTextInput(
            value = passphrase,
            onValueChange = onPassphraseChange,
            placeholder = "Enter your master passphrase",
            isPassword = !showPassphrase,
            style = when {
                passphrase.isEmpty() -> ZipLockTextInputStyle.Standard
                passphraseStrength?.isValid == true -> ZipLockTextInputStyle.Valid
                else -> ZipLockTextInputStyle.Invalid
            },
            imeAction = ImeAction.Next,
            leadingIcon = ZipLockIcons.Lock
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        // Passphrase strength indicator and requirements
        PassphraseValidationDisplay(passphraseStrength = passphraseStrength)

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        ZipLockAlert(
            level = AlertLevel.Warning,
            title = "⚠️ Important",
            message = "There is no way to recover your archive if you forget your master passphrase. Write it down and keep it safe!",
            dismissible = false
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        WizardNavigationButtons(
            onBack = onBack,
            onNext = onNext,
            canProceed = canProceed,
            nextText = "Continue"
        )
    }
}

@Composable
private fun ConfirmPassphraseStep(
    passphrase: String,
    confirmPassphrase: String,
    showConfirmPassphrase: Boolean,
    onConfirmPassphraseChange: (String) -> Unit,
    onToggleVisibility: () -> Unit,
    onNext: () -> Unit,
    onBack: () -> Unit,
    canProceed: Boolean
) {
    val passphraseMatch = confirmPassphrase.isNotEmpty() && passphrase == confirmPassphrase

    Column {
        Text(
            text = "Confirm Passphrase",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "Please enter your passphrase again to confirm it matches.",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Text(
            text = "Confirm Passphrase",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.DarkText,
            modifier = Modifier.padding(bottom = ZipLockSpacing.Small)
        )

        ZipLockTextInput(
            value = confirmPassphrase,
            onValueChange = onConfirmPassphraseChange,
            placeholder = "Re-enter your passphrase",
            isPassword = !showConfirmPassphrase,
            style = when {
                confirmPassphrase.isEmpty() -> ZipLockTextInputStyle.Standard
                passphraseMatch -> ZipLockTextInputStyle.Valid
                else -> ZipLockTextInputStyle.Invalid
            },
            imeAction = ImeAction.Done,
            keyboardActions = KeyboardActions(
                onDone = {
                    if (canProceed) {
                        onNext()
                    }
                }
            ),
            leadingIcon = ZipLockIcons.Lock
        )

        if (confirmPassphrase.isNotEmpty()) {
            Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

            Row(
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = if (passphraseMatch) "✓" else "✗",
                    style = ZipLockTypography.Medium,
                    color = if (passphraseMatch) ZipLockColors.SuccessGreen else ZipLockColors.ErrorRed
                )

                Spacer(modifier = Modifier.width(ZipLockSpacing.Small))

                Text(
                    text = if (passphraseMatch) "Passphrases match" else "Passphrases do not match",
                    style = ZipLockTypography.Small,
                    color = if (passphraseMatch) ZipLockColors.SuccessGreen else ZipLockColors.ErrorRed
                )
            }
        }

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        WizardNavigationButtons(
            onBack = onBack,
            onNext = onNext,
            canProceed = canProceed,
            nextText = "Create Archive"
        )
    }
}

@Composable
private fun CreatingStep(
    progress: Float,
    archiveName: String
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = "Creating Archive",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "Please wait while your encrypted archive is being created...",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        ZipLockLoadingIndicator(
            message = "Creating $archiveName.7z"
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Large))

        LinearProgressIndicator(
            progress = progress,
            modifier = Modifier.fillMaxWidth(),
            color = ZipLockColors.LogoPurple,
            trackColor = ZipLockColors.VeryLightGray
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "${(progress * 100).toInt()}% complete",
            style = ZipLockTypography.Small,
            color = ZipLockColors.LightGrayText
        )
    }
}

@Composable
private fun SuccessStep(
    archivePath: String,
    onOpenArchive: () -> Unit,
    onCreateAnother: () -> Unit
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = "Archive Created!",
            style = ZipLockTypography.ExtraLarge,
            color = ZipLockColors.SuccessGreen,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "Your encrypted archive has been successfully created and is ready to use.",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        Text(
            text = "✅",
            style = ZipLockTypography.ExtraLarge.copy(fontSize = 48.sp)
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        ZipLockAlert(
            level = AlertLevel.Success,
            title = "Archive Location",
            message = "Your archive has been saved to: ${archivePath.substringAfterLast('/')}",
            dismissible = false,
            modifier = Modifier.padding(bottom = ZipLockSpacing.Standard)
        )

        ZipLockButton(
            text = "Open Archive",
            onClick = onOpenArchive,
            style = ZipLockButtonStyle.Primary,
            icon = ZipLockIcons.FolderOpen,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        ZipLockButton(
            text = "Create Another Archive",
            onClick = onCreateAnother,
            style = ZipLockButtonStyle.Secondary,
            icon = ZipLockIcons.Plus,
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
private fun WizardNavigationButtons(
    onBack: () -> Unit,
    onNext: () -> Unit,
    canProceed: Boolean,
    nextText: String = "Next"
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(ZipLockSpacing.Standard)
    ) {
        ZipLockButton(
            text = "Back",
            onClick = onBack,
            style = ZipLockButtonStyle.Secondary,
            icon = ZipLockIcons.ArrowLeft,
            modifier = Modifier.weight(1f)
        )

        ZipLockButton(
            text = nextText,
            onClick = {
                Log.d("CreateArchiveWizard", "WizardNavigationButtons: '$nextText' clicked, enabled: $canProceed")
                onNext()
            },
            enabled = canProceed,
            style = if (canProceed) ZipLockButtonStyle.Primary else ZipLockButtonStyle.Disabled,
            icon = ZipLockIcons.ArrowRight,
            modifier = Modifier.weight(1f)
        )
    }
}

@Composable
private fun PassphraseValidationDisplay(
    passphraseStrength: ZipLockNative.PassphraseStrengthResult?
) {
    Column {
        // Strength indicator
        passphraseStrength?.let { strength ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Strength: ${strength.strength}",
                    style = ZipLockTypography.Medium,
                    color = when (strength.strength) {
                        "Very Weak", "Weak" -> ZipLockColors.ErrorRed
                        "Fair" -> ZipLockColors.WarningYellow
                        "Good", "Strong" -> ZipLockColors.SuccessGreen
                        "Very Strong" -> ZipLockColors.LogoPurple
                        else -> ZipLockColors.LightGrayText
                    }
                )

                Text(
                    text = "${strength.score}/100",
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.LightGrayText
                )
            }

            Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

            // Progress bar for strength
            LinearProgressIndicator(
                progress = strength.score / 100f,
                modifier = Modifier.fillMaxWidth(),
                color = when (strength.strength) {
                    "Very Weak", "Weak" -> ZipLockColors.ErrorRed
                    "Fair" -> ZipLockColors.WarningYellow
                    "Good", "Strong" -> ZipLockColors.SuccessGreen
                    "Very Strong" -> ZipLockColors.LogoPurple
                    else -> ZipLockColors.LightGrayText
                },
                trackColor = ZipLockColors.VeryLightGray
            )

            Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))
        }

        // Requirements display
        Text(
            text = "Passphrase Requirements:",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.DarkText
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        passphraseStrength?.let { strength ->
            // Show violations
            strength.requirements.forEach { requirement: String ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.padding(vertical = 2.dp)
                ) {
                    Text(
                        text = "✗",
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.ErrorRed
                    )

                    Spacer(modifier = Modifier.width(ZipLockSpacing.Small))

                    Text(
                        text = requirement,
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.ErrorRed
                    )
                }
            }

            // Show satisfied requirements
            strength.satisfied.forEach { satisfaction: String ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.padding(vertical = 2.dp)
                ) {
                    Text(
                        text = "✓",
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.SuccessGreen
                    )

                    Spacer(modifier = Modifier.width(ZipLockSpacing.Small))

                    Text(
                        text = satisfaction,
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.SuccessGreen
                    )
                }
            }
        } ?: run {
            // Show default requirements when no passphrase entered
            listOf(
                "At least 12 characters long",
                "Contains uppercase letters",
                "Contains lowercase letters",
                "Contains numbers",
                "Contains special characters"
            ).forEach { requirement ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.padding(vertical = 2.dp)
                ) {
                    Text(
                        text = "•",
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.LightGrayText
                    )

                    Spacer(modifier = Modifier.width(ZipLockSpacing.Small))

                    Text(
                        text = requirement,
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.LightGrayText
                    )
                }
            }
        }
    }
}

/**
 * Client-side archive name validation
 */
private fun validateArchiveNameClient(name: String): String? {
    return when {
        name.isBlank() -> "Archive name cannot be empty"
        name.length > 100 -> "Archive name is too long (maximum 100 characters)"
        name.contains(Regex("[<>:\"/\\\\|?*]")) -> "Contains invalid characters: < > : \" / \\ | ? *"
        name.startsWith(".") -> "Cannot start with a dot"
        name.endsWith(".") -> "Cannot end with a dot"
        name.matches(Regex("(?i)(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])")) -> "Reserved system name"
        else -> null
    }
}

/**
 * Get step progress information
 */
private fun getStepProgress(currentStep: CreateArchiveStep): Pair<Int, Int> {
    return when (currentStep) {
        CreateArchiveStep.SelectDestination -> 1 to 4
        CreateArchiveStep.ArchiveName -> 2 to 4
        CreateArchiveStep.CreatePassphrase -> 3 to 4
        CreateArchiveStep.ConfirmPassphrase -> 4 to 4
        CreateArchiveStep.Creating -> 4 to 4
        CreateArchiveStep.Success -> 4 to 4
    }
}




@Preview(showBackground = true)
@Composable
fun CreateArchiveWizardPreview() {
    CreateArchiveWizard(
        onArchiveCreated = { },
        onCancel = { }
    )
}
