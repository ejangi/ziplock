# ZipLock Android Application

## Overview

The ZipLock Android application provides a native mobile interface for the ZipLock password manager. It replicates the functionality of the Linux desktop application while providing a mobile-optimized user experience using Jetpack Compose and Material Design 3.

## Architecture

### Design System

The Android app implements a comprehensive design system that mirrors the Linux application's `theme.rs`:

- **ZipLockTheme.kt**: Core theme system with colors, typography, spacing, and dimensions
- **ZipLockIcons.kt**: Icon definitions using Material Icons with fallbacks
- **ZipLockComponents.kt**: Reusable UI components with consistent styling

### Key Components

#### Theme System (`ui/theme/`)
- **Colors**: Matches Linux theme colors exactly (LogoPurple #8338EC, validation colors, etc.)
- **Typography**: Consistent text styling with proper hierarchy
- **Spacing**: Standardized spacing values for consistent layouts
- **Dimensions**: Standard component sizes and elevation values

#### UI Components
- **ZipLockButton**: Styled buttons with multiple variants (Primary, Secondary, Destructive, etc.)
- **ZipLockTextInput**: Text input fields with validation states and password visibility toggle
- **ZipLockAlert**: Alert components for error/success/warning messages
- **ZipLockFilePicker**: File selection component with cloud storage awareness
- **ZipLockPasswordToggle**: Password visibility toggle with eye icon

#### Screen Architecture
- **RepositorySelectionScreen**: Main entry point for selecting and opening archives
- **RepositoryViewModel**: Business logic and state management
- **FFI Integration**: Native library wrapper for cryptographic operations

## Features

### Current Implementation

#### Repository Selection
- File picker integration for selecting .7z archive files
- Passphrase input with real-time validation
- Password visibility toggle with professional eye icon
- Cloud storage detection and handling
- Loading states and error feedback

#### Design Compliance
- Flat design philosophy matching design.md specifications
- Consistent color palette with validation states
- Professional password input with visibility controls
- User-friendly error messages
- Responsive layout for different screen sizes

#### Security Integration
- FFI wrapper ready for shared library integration
- Cloud storage file detection (matches cloud-storage-implementation.md)
- Secure passphrase handling (no crypto implementation in UI)
- Proper error handling and user feedback

### FFI Integration Structure

#### Native Library Interface (`ffi/ZipLockNative.kt`)
```kotlin
// Archive operations
external fun openArchive(archivePath: String, passphrase: String): ArchiveResult
external fun createArchive(archivePath: String, passphrase: String): ArchiveResult
external fun closeArchive(sessionId: String): Boolean

// Credential management
external fun getCredential(sessionId: String, credentialId: String): Credential?
external fun addCredential(sessionId: String, credential: Credential): String?
external fun updateCredential(sessionId: String, credentialId: String, credential: Credential): Boolean

// Cloud storage support
external fun isCloudStorageFile(archivePath: String): Boolean
external fun getCloudStorageInfo(archivePath: String): CloudStorageInfo
```

#### State Management
- **RepositoryViewModel**: Manages repository state and business logic
- **RepositoryUiState**: UI state including loading, errors, and success messages
- **RepositoryState**: Repository status (None, Opened, Created)

## File Structure

```
app/src/main/java/com/ziplock/
├── MainActivity.kt                    # Main activity with navigation
├── SplashActivity.kt                 # Splash screen
├── ui/
│   ├── theme/
│   │   ├── ZipLockTheme.kt          # Core theme system
│   │   ├── ZipLockIcons.kt          # Icon definitions
│   │   └── ZipLockComponents.kt     # Reusable components
│   └── screens/
│       └── RepositorySelectionScreen.kt # Archive selection UI
├── viewmodel/
│   └── RepositoryViewModel.kt        # Business logic and state
└── ffi/
    └── ZipLockNative.kt             # FFI wrapper for shared library
```

## Design Principles

### Visual Design
- **Flat Design**: Minimalist aesthetics without gradients or drop shadows
- **Color Consistency**: Uses exact color values from Linux theme
- **Typography**: Clear hierarchy with proper font weights and sizes
- **Spacing**: Consistent spacing using standardized values

### User Experience
- **Progressive Disclosure**: Show relevant information at the right time
- **Error Prevention**: Real-time validation and clear feedback
- **Accessibility**: High contrast colors and proper content descriptions
- **Performance**: Optimized layouts and efficient state management

### Security
- **No Crypto in UI**: All cryptographic operations handled by shared library
- **Secure Defaults**: Safe input handling and proper error messages
- **Cloud Awareness**: Automatic detection of cloud storage files
- **Session Management**: Proper session tracking and cleanup

## Component Usage Examples

### Basic Text Input
```kotlin
ZipLockTextInput(
    value = passphrase,
    onValueChange = { passphrase = it },
    placeholder = "Enter your passphrase",
    isPassword = true,
    style = ZipLockTextInputStyle.Standard,
    leadingIcon = ZipLockIcons.Lock
)
```

### File Picker
```kotlin
ZipLockFilePicker(
    selectedFileName = selectedFileName,
    onFileSelect = { /* Launch file picker */ },
    placeholder = "Select archive file (.7z)"
)
```

### Alert Messages
```kotlin
ZipLockAlert(
    level = AlertLevel.Error,
    message = "Incorrect passphrase. Please try again.",
    onDismiss = { /* Clear error */ }
)
```

### Styled Buttons
```kotlin
ZipLockButton(
    text = "Open Archive",
    onClick = { /* Open archive */ },
    style = ZipLockButtonStyle.Primary,
    icon = ZipLockIcons.FolderOpen,
    enabled = isValidForm
)
```

## Development Workflow

### Setup
1. Follow the main Android setup guide in `docs/technical/android.md`
2. Ensure native libraries are built and placed in `jniLibs/`
3. Verify all dependencies are correctly configured

### Building
```bash
./gradlew assembleDebug    # Debug build
./gradlew assembleRelease  # Release build
```

### Testing
```bash
./gradlew test            # Unit tests
./gradlew connectedCheck  # Instrumented tests
```

## Integration Status

### ✅ Completed
- [x] Theme system matching Linux implementation
- [x] Repository selection screen
- [x] File picker integration
- [x] Passphrase input with validation
- [x] Error handling and user feedback
- [x] Cloud storage detection structure
- [x] FFI wrapper architecture

### 🚧 In Progress
- [ ] Native library compilation for Android
- [ ] FFI JNI bridge implementation
- [ ] Main password manager interface
- [ ] Credential list and detail views

### 📋 Planned
- [ ] Credential creation and editing
- [ ] Search and filtering
- [ ] Settings and preferences
- [ ] Export/import functionality
- [ ] Biometric authentication
- [ ] Auto-fill service integration

## Dependencies

### Core Dependencies
- **Jetpack Compose**: Modern UI toolkit
- **Material 3**: Material Design components
- **ViewModel**: Architecture component for state management
- **Navigation Compose**: Type-safe navigation
- **DocumentFile**: File picker and document handling

### Native Integration
- **JNI**: Java Native Interface for FFI calls
- **NDK**: Android Native Development Kit
- **Shared Library**: Cross-compiled Rust library

## Performance Considerations

### UI Performance
- Efficient recomposition using `remember` and `LaunchedEffect`
- Proper state hoisting to minimize recomposition scope
- Lazy loading for large lists (future implementation)

### Memory Management
- Proper disposal of native resources
- Efficient bitmap handling for icons
- Proper lifecycle-aware components

### Security Performance
- Minimal sensitive data retention in memory
- Secure deletion of temporary files
- Efficient cloud storage handling

## Testing Strategy

### Unit Tests
- ViewModel business logic
- Utility functions
- Error message mapping

### Integration Tests
- FFI wrapper functionality
- File picker integration
- Navigation flow

### UI Tests
- Screen composition
- User interaction flows
- Accessibility compliance

## Future Enhancements

### Mobile-Specific Features
- **Biometric Authentication**: Fingerprint/face unlock
- **Auto-fill Service**: System-wide password auto-fill
- **Share Extensions**: Secure sharing of credentials
- **Widget Support**: Quick access widget

### Advanced UI Features
- **Dark Theme**: Full dark mode support
- **Dynamic Colors**: Material You color theming
- **Adaptive Layouts**: Tablet and foldable support
- **Animations**: Smooth transitions and micro-interactions

### Cloud Integration
- **Real-time Sync Monitoring**: Detect active sync operations
- **Conflict Resolution UI**: User interface for handling conflicts
- **Provider-Specific Optimizations**: Enhanced cloud service integration

## Contributing

### Code Style
- Follow Kotlin coding conventions
- Use meaningful variable and function names
- Add KDoc comments for public APIs
- Maintain consistent formatting

### Component Development
- Follow the established theme system
- Use provided spacing and color values
- Implement proper accessibility support
- Include preview functions for Compose components

### Testing Requirements
- Add unit tests for new ViewModels
- Include UI tests for new screens
- Test accessibility features
- Verify integration with FFI layer

## Troubleshooting

### Common Issues
- **Native Library Loading**: Ensure libraries are in correct `jniLibs/` directories
- **File Picker Permissions**: Check storage permissions in manifest
- **Theme Issues**: Verify color definitions match Linux implementation
- **Compose Previews**: May not show custom fonts, test on device

### Debugging Tips
- Use Android Studio's Compose Inspector
- Enable debug logging in ViewModels
- Monitor memory usage during FFI calls
- Test on various screen sizes and orientations

## Resources

### Documentation
- [Android Development Guide](../../../docs/technical/android.md)
- [Design Guidelines](../../../docs/design.md)
- [Cloud Storage Implementation](../../../docs/technical/cloud-storage-implementation.md)

### External Resources
- [Jetpack Compose Documentation](https://developer.android.com/jetpack/compose)
- [Material Design 3](https://m3.material.io/)
- [Android Architecture Guidelines](https://developer.android.com/topic/architecture)