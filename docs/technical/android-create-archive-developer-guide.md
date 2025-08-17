# Android Create Archive Wizard - Developer Guide

This guide provides developers with practical information for working with, extending, or maintaining the Create Archive wizard in the ZipLock Android application.

## Quick Start

### Basic Integration

To integrate the Create Archive wizard into your activity or fragment:

```kotlin
@Composable
fun MyScreen() {
    var showCreateWizard by remember { mutableStateOf(false) }
    
    if (showCreateWizard) {
        CreateArchiveWizard(
            onArchiveCreated = { archivePath ->
                // Handle successful archive creation
                println("Archive created at: $archivePath")
                showCreateWizard = false
            },
            onCancel = {
                // Handle wizard cancellation
                showCreateWizard = false
            }
        )
    } else {
        // Your main UI
        Button(onClick = { showCreateWizard = true }) {
            Text("Create New Archive")
        }
    }
}
```

### With Custom ViewModel

For advanced use cases where you need to customize the wizard behavior:

```kotlin
@Composable
fun CustomCreateArchiveScreen() {
    val viewModel: CreateArchiveViewModel = viewModel()
    
    CreateArchiveWizard(
        onArchiveCreated = { archivePath ->
            // Custom handling
        },
        onCancel = {
            viewModel.reset() // Clean up state
        },
        viewModel = viewModel // Use custom instance
    )
}
```

## Architecture Deep Dive

### State Management Flow

```
User Input → ViewModel → StateFlow → UI Updates
     ↓           ↓           ↓          ↓
  Validation → Business → Reactive → Recomposition
             → Logic   → Updates
```

### Key Components

#### 1. CreateArchiveViewModel

**Responsibilities:**
- State management for all wizard steps
- FFI integration for archive creation
- Real-time passphrase validation
- Error handling and user feedback

**Key Methods:**
```kotlin
// Navigation
fun proceedToNext()
fun goBack()
fun updateStep(step: CreateArchiveStep)

// State Updates
fun updatePassphrase(passphrase: String)
fun setDestination(path: String, name: String)
fun updateArchiveName(name: String)

// Validation
fun canProceed(): Boolean
fun validateArchiveName(name: String): String?

// Lifecycle
fun reset()
fun clearError()
```

#### 2. CreateArchiveWizard Composable

**Responsibilities:**
- UI rendering for all wizard steps
- User interaction handling
- Integration with ViewModel
- File picker coordination

**Step Components:**
- `WelcomeStep` - Introduction and overview
- `SelectDestinationStep` - Folder selection with SAF
- `ArchiveNameStep` - Name input with validation
- `CreatePassphraseStep` - Password creation with strength feedback
- `ConfirmPassphraseStep` - Password confirmation
- `CreatingStep` - Progress indicator
- `SuccessStep` - Completion and next actions

## Customization Guide

### Custom Validation Rules

To add custom passphrase validation:

```kotlin
class CustomCreateArchiveViewModel : CreateArchiveViewModel() {
    
    override fun createFallbackValidation(passphrase: String): PassphraseStrengthResult {
        val baseResult = super.createFallbackValidation(passphrase)
        
        // Add custom rules
        val customRequirements = baseResult.requirements.toMutableList()
        val customSatisfied = baseResult.satisfied.toMutableList()
        
        // Example: Require at least 3 numbers
        val digitCount = passphrase.count { it.isDigit() }
        if (digitCount < 3) {
            customRequirements.add("Must contain at least 3 numbers")
        } else {
            customSatisfied.add("Contains sufficient numbers ($digitCount)")
        }
        
        return baseResult.copy(
            requirements = customRequirements,
            satisfied = customSatisfied,
            isValid = customRequirements.isEmpty()
        )
    }
}
```

### Custom Step Content

To modify or add wizard steps:

```kotlin
@Composable
fun CustomCreateArchiveWizard(
    onArchiveCreated: (String) -> Unit,
    onCancel: () -> Unit,
    viewModel: CreateArchiveViewModel = viewModel()
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    
    // Add custom step handling
    when (uiState.currentStep) {
        CreateArchiveStep.ArchiveName -> CustomArchiveNameStep(
            archiveName = uiState.archiveName,
            onArchiveNameChange = viewModel::updateArchiveName,
            onNext = viewModel::proceedToNext,
            onBack = viewModel::goBack,
            canProceed = viewModel.canProceed()
        )
        // ... other steps
    }
}

@Composable
private fun CustomArchiveNameStep(
    archiveName: String,
    onArchiveNameChange: (String) -> Unit,
    onNext: () -> Unit,
    onBack: () -> Unit,
    canProceed: Boolean
) {
    Column {
        // Custom UI for archive name step
        Text("Custom Archive Name Step")
        
        // Add dropdown for predefined names
        val predefinedNames = listOf("Personal", "Work", "Family", "Custom")
        var selectedName by remember { mutableStateOf("Custom") }
        
        DropdownMenu(/* ... */) {
            predefinedNames.forEach { name ->
                DropdownMenuItem(
                    text = { Text(name) },
                    onClick = {
                        if (name != "Custom") {
                            onArchiveNameChange(name)
                        }
                        selectedName = name
                    }
                )
            }
        }
        
        // Show text input only for custom names
        if (selectedName == "Custom") {
            ZipLockTextInput(
                value = archiveName,
                onValueChange = onArchiveNameChange,
                placeholder = "Enter custom archive name"
            )
        }
        
        WizardNavigationButtons(
            onBack = onBack,
            onNext = onNext,
            canProceed = canProceed
        )
    }
}
```

### Custom Error Handling

To add custom error handling:

```kotlin
class CustomCreateArchiveViewModel : CreateArchiveViewModel() {
    
    private fun mapCustomErrors(error: Throwable): String {
        return when {
            error.message?.contains("custom_error_code") == true ->
                "This is a custom error message for users"
            error is SecurityException ->
                "Security error: Please check app permissions"
            else -> super.mapErrorToUserMessage(error)
        }
    }
    
    override suspend fun createArchive(
        destinationPath: String,
        archiveName: String,
        passphrase: String
    ) {
        try {
            super.createArchive(destinationPath, archiveName, passphrase)
        } catch (e: Exception) {
            val userMessage = mapCustomErrors(e)
            setError(userMessage)
        }
    }
}
```

## Testing Guide

### Unit Testing ViewModel

```kotlin
@Test
fun `custom validation should work correctly`() = runTest {
    val viewModel = CustomCreateArchiveViewModel()
    
    // Test custom passphrase requirements
    viewModel.updatePassphrase("abc123") // Should fail custom digit rule
    advanceUntilIdle()
    
    val strength = viewModel.passphraseStrength.value
    assertNotNull(strength)
    assertTrue(strength!!.requirements.any { it.contains("3 numbers") })
    assertFalse(strength.isValid)
    
    // Test with sufficient digits
    viewModel.updatePassphrase("Abc123456!")
    advanceUntilIdle()
    
    val newStrength = viewModel.passphraseStrength.value
    assertTrue(newStrength!!.satisfied.any { it.contains("sufficient numbers") })
}
```

### UI Testing with Compose

```kotlin
@Test
fun `create archive wizard should complete full flow`() {
    composeTestRule.setContent {
        CreateArchiveWizard(
            onArchiveCreated = { /* mock */ },
            onCancel = { /* mock */ }
        )
    }
    
    // Test welcome step
    composeTestRule.onNodeWithText("Create New Archive").assertIsDisplayed()
    composeTestRule.onNodeWithText("Get Started").performClick()
    
    // Test destination selection
    composeTestRule.onNodeWithText("Select Destination").assertIsDisplayed()
    // Mock file picker interaction
    
    // Continue through remaining steps...
}
```

## FFI Integration Details

### Archive Creation Flow

```kotlin
// 1. Validate inputs
if (!ZipLockNativeHelper.validateLibrary()) {
    throw Exception("FFI library not available")
}

// 2. Construct archive path
val fullPath = constructArchivePath(destinationPath, archiveName)

// 3. Create archive via FFI
val result = ZipLockNative.createArchive(fullPath, passphrase)

// 4. Handle result
if (result.isSuccess()) {
    // Success handling
} else {
    val error = ZipLockNativeHelper.getDetailedError(result)
    throw Exception(error)
}
```

### Error Code Mapping

```kotlin
fun mapFFIErrorToUserMessage(errorCode: Int): String {
    return when (errorCode) {
        1 -> "Invalid archive format"
        2 -> "Incorrect passphrase"
        3 -> "File not found"
        4 -> "Permission denied - check folder access"
        5 -> "Archive file is corrupted"
        6 -> "Network error - check cloud storage connection"
        7 -> "Cloud storage conflict detected"
        8 -> "Invalid session - please restart"
        9 -> "Archive is locked by another process"
        10 -> "Insufficient storage space"
        else -> "Unknown error (code: $errorCode)"
    }
}
```

## Performance Optimization

### State Management Best Practices

```kotlin
// ✅ Good: Use StateFlow for reactive updates
private val _uiState = MutableStateFlow(CreateArchiveUiState())
val uiState: StateFlow<CreateArchiveUiState> = _uiState.asStateFlow()

// ✅ Good: Batch state updates
fun updateMultipleFields(name: String, passphrase: String) {
    _uiState.value = _uiState.value.copy(
        archiveName = name,
        passphrase = passphrase,
        errorMessage = null
    )
}

// ❌ Avoid: Multiple individual updates
fun avoidThis(name: String, passphrase: String) {
    updateArchiveName(name)      // Triggers recomposition
    updatePassphrase(passphrase) // Triggers another recomposition
}
```

### Memory Management

```kotlin
class CreateArchiveViewModel : ViewModel() {
    
    override fun onCleared() {
        super.onCleared()
        
        // Clear sensitive data
        _uiState.value = _uiState.value.copy(
            passphrase = "",
            confirmPassphrase = ""
        )
        
        // Cancel any ongoing operations
        viewModelScope.cancel()
    }
}
```

### Efficient Validation

```kotlin
// Debounce passphrase validation to avoid excessive FFI calls
private var validationJob: Job? = null

fun updatePassphrase(passphrase: String) {
    _uiState.value = _uiState.value.copy(passphrase = passphrase)
    
    validationJob?.cancel()
    validationJob = viewModelScope.launch {
        delay(300) // Debounce for 300ms
        validatePassphraseStrength(passphrase)
    }
}
```

## Troubleshooting

### Common Issues

**1. FFI Library Not Loading**
```kotlin
// Check if library is available
if (!ZipLockNativeHelper.validateLibrary()) {
    Log.e("CreateArchive", "FFI library not available")
    // Show fallback UI or error message
}
```

**2. File Permission Issues**
```kotlin
// Handle storage access gracefully
try {
    val result = ZipLockNative.createArchive(path, passphrase)
    // ...
} catch (e: SecurityException) {
    setError("Permission denied. Please select a folder you have write access to.")
}
```

**3. Memory Leaks**
```kotlin
// Always clean up in onCleared()
override fun onCleared() {
    super.onCleared()
    viewModelScope.cancel()
    // Clear any references
}
```

### Debug Logging

```kotlin
// Enable debug logging for troubleshooting
class CreateArchiveViewModel : ViewModel() {
    
    private val isDebugMode = BuildConfig.DEBUG
    
    private fun debugLog(message: String) {
        if (isDebugMode) {
            Log.d("CreateArchiveWizard", message)
        }
    }
    
    fun updatePassphrase(passphrase: String) {
        debugLog("Updating passphrase, length: ${passphrase.length}")
        // ... rest of method
    }
}
```

## Migration and Compatibility

### Upgrading from Previous Versions

If migrating from an older implementation:

```kotlin
// Old approach (deprecated)
// Direct FFI calls in Composable

// New approach (recommended)
@Composable
fun ModernCreateArchive() {
    val viewModel: CreateArchiveViewModel = viewModel()
    
    CreateArchiveWizard(
        viewModel = viewModel,
        onArchiveCreated = { /* handle */ },
        onCancel = { /* handle */ }
    )
}
```

### Backward Compatibility

The wizard maintains compatibility with:
- Android API 24+ (Android 7.0)
- Existing FFI interfaces
- Legacy archive formats
- Previous ZipLock configurations

## Advanced Features

### Custom Theming

```kotlin
@Composable
fun ThemedCreateArchiveWizard() {
    MaterialTheme(
        colorScheme = CustomColorScheme,
        typography = CustomTypography
    ) {
        CreateArchiveWizard(
            onArchiveCreated = { /* handle */ },
            onCancel = { /* handle */ }
        )
    }
}
```

### Accessibility Support

```kotlin
@Composable
fun AccessibleCreateArchiveWizard() {
    CreateArchiveWizard(
        onArchiveCreated = { /* handle */ },
        onCancel = { /* handle */ },
        modifier = Modifier.semantics {
            contentDescription = "Create new password archive wizard"
            role = Role.Dialog
        }
    )
}
```

This developer guide provides comprehensive information for working with the Create Archive wizard. For additional questions or advanced use cases, refer to the technical documentation or source code comments.