# ZipLock Android App

> **📍 Documentation has moved!**
> 
> All Android development documentation has been consolidated into the main technical documentation for better organization and discoverability. The original files have been removed as their content is now fully integrated into the comprehensive guide.
>
> **⚠️ IMPORTANT**: All technical documentation for ZipLock MUST be placed in the `docs/technical/` directory. This ensures proper organization, discoverability, and maintenance of project documentation.

## Quick Links

- **📚 [Complete Android Development Guide](../../../docs/technical/android.md)** - Comprehensive documentation covering:
  - 🚀 5-minute quick start
  - 🛠️ Development setup (Android Studio, emulator, etc.)
  - 📱 Android app implementation details
  - 🔧 Native library compilation
  - 🔗 FFI integration patterns
  - 🐛 Troubleshooting guide
  - 🔒 Security considerations
  - 📋 Development roadmap

- **🎯 [Quick Start Section](../../../docs/technical/android.md#quick-start-5-minutes)** - Get running in 5 minutes
- **⚙️ [Setup Guide](../../../docs/technical/android.md#development-setup)** - Detailed development environment setup
- **🏗️ [Project Structure](../../../docs/technical/android.md#android-app-implementation)** - Understanding the codebase

## What's Here

This directory contains the ZipLock Android application source code:

```
android/
├── app/                          # Main Android application module
│   ├── src/main/java/com/ziplock/
│   │   ├── SplashActivity.kt     # Splash screen with ZipLock branding
│   │   └── MainActivity.kt       # Main app activity (placeholder)
│   └── src/main/res/             # Resources (layouts, colors, strings)
├── build.gradle                  # Project configuration
└── README.md                     # This file
```

## Current Status

✅ **Ready to run**: Professional splash screen with Material 3 design  
🔄 **In development**: FFI integration with ZipLock core library  
📋 **Planned**: Full password management functionality  

## Development Workflow

1. **First time setup**: Follow the [Android Development Guide](../../../docs/technical/android.md)
2. **Daily development**: Use Android Studio with the configured emulator
3. **Building**: Native libraries are built separately via Docker scripts
4. **Testing**: Run on emulator or physical device

## Support

For help with Android development:
- Check the [troubleshooting section](../../../docs/technical/android.md#troubleshooting)
- Review the [development setup guide](../../../docs/technical/android.md#development-setup)
- Look at project issues and discussions

---

**Note**: This README serves as a redirect to the consolidated documentation. The comprehensive Android development guide contains all setup instructions, troubleshooting tips, and implementation details that were previously scattered across multiple files.

**Documentation Policy**: Following ZipLock's documentation standards, all technical documentation is centralized in `docs/technical/` to maintain organization and ensure easy discovery by developers.