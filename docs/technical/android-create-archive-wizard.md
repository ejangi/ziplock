# Android Create Archive Wizard Implementation

This document describes the implementation of the Create Archive wizard for the ZipLock Android application, which guides users through creating new encrypted password archives.

## Overview

The Create Archive wizard is a multi-step guided interface that helps users create new ZipLock archives with proper validation, security, and cloud storage support. It follows the design principles established in the Linux implementation while adapting to Android's UI patterns and storage model.

## Architecture

### Components

1. **CreateArchiveWizard** (`ui/screens/CreateArchiveWizard.kt`)
   - Main Composable that orchestrates the wizard flow
   - Handles UI state management and navigation between steps
   - Integrates with ViewModel for business logic

2. **CreateArchiveViewModel** (`viewmodel/CreateArchiveViewModel.kt`)
   - Manages wizard state and business logic
   - Handles FFI integration for archive creation
   - Provides real-time passphrase validation

3. **CreateArchiveStep** (enum)
   - Defines the wizard steps: Welcome, SelectDestination, ArchiveName, CreatePassphrase, ConfirmPassphrase, Creating, Success

## Wizard Flow

### Step 1: Welcome
- Introduction to the archive creation process
- Explains the purpose and security of ZipLock archives
- Single "Get Started" button to begin

### Step 2: Select Destination
- File picker for choosing destination folder
- Supports both local storage and cloud services
- Uses Android's Storage Access Framework (SAF)
- Displays selected folder name with helpful tips

### Step 3: Archive Name
- Text input for archive filename (without .7z extension)
- Real-time validation for non-empty names
- Shows preview of final filename with extension
- Default name: "ZipLock"

### Step 4: Create Passphrase
- Password input with visibility toggle
- Real-time strength validation using FFI
- Visual feedback with progress bar and requirements list
- Color-coded strength indicators (red/yellow/green/purple)
- Important security warning about passphrase recovery

### Step 5: Confirm Passphrase
- Second password input to confirm passphrase
- Real-time matching validation
- Visual feedback for match/mismatch status
- Final "Create Archive" button

### Step 6: Creating
- Progress indicator during archive creation
- Uses FFI library for actual archive creation
- Progress updates and status messages
- Cannot be cancelled once started

### Step 7: Success
- Confirmation of successful creation
- Archive location display
- Options to open archive or create another

## Passphrase Validation

### FFI Integration
The wizard uses the shared ZipLock library for passphrase validation through `ZipLockNative.validatePassphraseStrength()`:

```kotlin
val strength = ZipLockNative.validatePassphraseStrength(passphrase)
```

### Fallback Validation
When FFI is unavailable, a fallback validation system provides:
- Length requirements (minimum 12 characters)
- Character type requirements (uppercase, lowercase, numbers, special characters)
- Scoring system (0-100)
- Strength levels: Very Weak, Weak, Fair, Good, Strong, Very Strong

### Validation Display
- **Requirements List**: Shows unmet requirements with red ✗ and satisfied requirements with green ✓
- **Strength Indicator**: Progress bar with color coding
- **Score Display**: Numeric score out of 100
- **Real-time Updates**: Validation occurs as user types

### Strength Levels and Colors
| Level | Score | Color | Usage |
|-------|-------|-------|-------|
| Very Weak | 0-20 | Red (#ef476f) | Prevent submission |
| Weak | 21-40 | Red (#ef476f) | Prevent submission |
| Fair | 41-60 | Yellow (#fcbf49) | Warning |
| Good | 61-80 | Green (#06d6a0) | Allow submission |
| Strong | 81-95 | Green (#06d6a0) | Positive feedback |
| Very Strong | 96-100 | Purple (#8338ec) | Excellent |

## Cloud Storage Support

### Storage Access Framework Integration
The wizard uses Android's SAF for file selection:
- `ActivityResultContracts.OpenDocumentTree()` for directory selection
- Support for cloud storage providers (Google Drive, Dropbox, OneDrive)
- Content URIs are properly handled by the FFI library

### Cloud Detection
The shared library automatically detects cloud storage locations and applies appropriate handling:
- Copy-to-local strategy for safe operations
- Conflict detection and prevention
- Automatic sync back on completion

## UI Components and Styling

### Theme Integration
Uses ZipLock design system components:
- `ZipLockButton` with consistent styling
- `ZipLockTextInput` with validation states
- `ZipLockAlert` for errors and warnings
- `ZipLockFilePicker` for folder selection
- `ZipLockLoadingIndicator` for progress states

### Navigation Pattern
- Header with cancel button and progress indicator
- Step-based content area with consistent styling
- Back/Next navigation buttons (enabled/disabled based on validation)
- Error handling with dismissible alerts

### Responsive Design
- Scrollable content for different screen sizes
- Proper spacing and padding using ZipLockSpacing
- Icon integration with ZipLockIcons
- Touch-friendly button sizes

## State Management

### ViewModel Architecture
```kotlin
data class CreateArchiveUiState(
    val currentStep: CreateArchiveStep,
    val destinationPath: String?,
    val destinationName: String?,
    val archiveName: String,
    val passphrase: String,
    val confirmPassphrase: String,
    val showPassphrase: Boolean,
    val showConfirmPassphrase: Boolean,
    val errorMessage: String?,
    val isLoading: Boolean,
    val creationProgress: Float,
    val createdArchivePath: String?
)
```

### State Flow Management
- `StateFlow` for UI state updates
- Separate flow for passphrase strength results
- Reactive UI updates using `collectAsStateWithLifecycle()`

## FFI Integration

### Archive Creation
```kotlin
val result = ZipLockNative.createArchive(fullArchivePath, passphrase)
if (result.isSuccess()) {
    // Success handling
} else {
    val errorMessage = ZipLockNativeHelper.getDetailedError(result)
    // Error handling
}
```

### Error Handling
- User-friendly error messages using `ZipLockNativeHelper.mapErrorCode()`
- Graceful fallback when FFI is unavailable
- Proper error state management and recovery

## Testing

### Unit Tests
Comprehensive test coverage for `CreateArchiveViewModel`:
- State transitions and validation
- Navigation flow testing
- Error handling scenarios
- Passphrase validation (with and without FFI)

### Test Files
- `CreateArchiveViewModelTest.kt` - Core functionality tests
- Mocked FFI interactions for reliable testing

## Integration with Main App

### MainActivity Integration
```kotlin
when (currentScreen) {
    Screen.CreateArchive -> {
        CreateArchiveWizard(
            onArchiveCreated = { archivePath ->
                currentScreen = Screen.RepositoryOpened(archivePath)
            },
            onCancel = {
                currentScreen = Screen.RepositorySelection
            }
        )
    }
}
```

### Navigation Flow
1. User clicks "Create New Archive" in RepositorySelectionScreen
2. Navigate to CreateArchiveWizard
3. Complete wizard steps
4. On success: Navigate to opened archive
5. On cancel: Return to repository selection

## Security Considerations

### Passphrase Handling
- Passphrases stored in ViewModel memory only
- No persistent storage of sensitive data
- Cleared on navigation away from wizard
- Real-time validation without network calls

### File Handling
- Uses Android's secure storage APIs
- Proper permission handling for file access
- Cloud storage safety through FFI library
- No temporary files containing sensitive data

## Performance Optimizations

### Efficient Validation
- Debounced passphrase validation
- Lazy composition of UI elements
- Efficient state updates using StateFlow
- Minimal recomposition through proper state management

### Memory Management
- ViewModel lifecycle awareness
- Proper cleanup of resources
- Efficient file operations through FFI

## Future Enhancements

### Planned Features
1. **Advanced Passphrase Options**
   - Passphrase generation with customizable criteria
   - Import from password managers
   - Biometric integration for convenience

2. **Enhanced Cloud Support**
   - Provider-specific optimizations
   - Offline mode handling
   - Real-time sync status monitoring

3. **Improved UX**
   - Animated transitions between steps
   - Better progress visualization
   - Contextual help and tips

4. **Accessibility**
   - Screen reader support
   - High contrast mode
   - Keyboard navigation

## Troubleshooting

### Common Issues

**FFI Library Not Available**
- Fallback validation automatically activates
- User sees basic validation instead of advanced scoring
- Archive creation may fail with clear error message

**File Permission Issues**
- Storage Access Framework handles permissions
- Clear error messages for permission denied scenarios
- Guidance for user to select appropriate folders

**Cloud Storage Problems**
- FFI library handles cloud detection and safety
- Conflict resolution through copy-to-local strategy
- User feedback for sync status and issues

### Debug Information
- Comprehensive logging through ViewModel
- Error state preservation for troubleshooting
- Clear error messages mapped from FFI error codes

## Compatibility

### Android Versions
- Minimum SDK: 24 (Android 7.0)
- Target SDK: Latest stable
- Storage Access Framework support for all versions

### Device Types
- Phone and tablet layouts
- Different screen sizes and orientations
- Various storage configurations

This implementation provides a comprehensive, secure, and user-friendly way to create ZipLock archives on Android, matching the functionality and quality of the Linux implementation while adapting to Android's unique platform characteristics.