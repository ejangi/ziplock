# ZipLock Password Manager

<div align="center">
  <img src="assets/icons/ziplock-logo.svg" alt="ZipLock Logo" width="128" height="128">

  **A secure, portable password manager using encrypted 7z archives**

  [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE.md)
  [![Linux Build](https://img.shields.io/github/actions/workflow/status/ejangi/ziplock/linux-build.yml?branch=main&label=linux%20build)](https://github.com/ejangi/ziplock/actions/workflows/linux-build.yml)
  [![Android Build](https://img.shields.io/github/actions/workflow/status/ejangi/ziplock/android.yml?branch=main&label=android%20build)](https://github.com/ejangi/ziplock/actions/workflows/android.yml)
  [![Security Audit](https://img.shields.io/badge/security-audited-green.svg)](docs/architecture.md#security-architecture)
</div>

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [User Experience](#user-experience)
- [Platform Support](#platform-support)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Build & Development](#build--development)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)
- [Support](#support)

## 🔐 Overview

ZipLock is a modern, secure password manager that stores your credentials in a single encrypted 7z archive file. Built with a focus on security, portability, and user control, ZipLock ensures your sensitive data remains encrypted and under your complete control.

Unlike cloud-based password managers, ZipLock gives you full ownership of your data. Your encrypted password database is a single file that you can store anywhere - on your local drive, in your preferred cloud storage service, or on a USB drive. This approach provides maximum flexibility while maintaining the highest security standards.

### Why ZipLock?

- **🔒 Your Data, Your Control**: No cloud dependencies, no vendor lock-in
- **📦 True Portability**: Single encrypted file you can store anywhere
- **🛡️ Zero Knowledge**: Your master key never leaves your device
- **🔓 Open Source**: Fully auditable code under Apache 2.0 license
- **🌐 Cross-Platform**: Native apps for all major platforms

## ✨ Key Features

### Security First
- **AES-256 Encryption**: Military-grade encryption with robust key derivation using Argon2
- **Secure Memory Management**: Master key stored only in memory, never persisted to disk
- **Auto-Lock Protection**: Configurable timeout to automatically lock your vault
- **File Locking**: Prevents concurrent access and data corruption during sync operations

### Powerful Organization
- **Full-Text Search**: Search across all credential fields instantly
- **Smart Tagging**: Organize credentials with custom tags for easy filtering

### Modern User Experience
- **Clean, Flat Design**: Modern interface with dark/light theme support
- **TOTP Generation**: Built-in two-factor authentication code generation
- **Password Generator**: Create strong, customizable passwords
- **Browser Integration**: Seamless auto-fill through browser extensions
- **Import/Export**: Easy migration from other password managers

### Advanced Features
- **Version History**: Track and restore previous versions of credentials
- **Configurable Compression**: Advanced 7z compression with solid compression and multi-threading
- **Backup Management**: Automatic backup rotation with configurable retention
- **Cross-Device Sync**: Use any file sync service (Dropbox, OneDrive, iCloud, etc.)

## 🎯 User Experience

ZipLock is designed to be intuitive and efficient for both new users and power users:

### First-Time Setup
1. **Create Your Vault**: Choose a strong master passphrase with real-time strength validation
2. **Select Storage Location**: Pick where to store your encrypted database file
3. **Import Existing Data**: Easily migrate from other password managers (optional)

### Daily Usage
- **Quick Access**: Fast unlock with master passphrase
- **Instant Search**: Find credentials as you type
- **One-Click Actions**: Copy passwords, usernames, and TOTP codes with a single click
- **Auto-Fill**: Browser extensions provide seamless login automation

### Advanced Management
- **Custom Templates**: Create credential types that match your specific needs
- **Bulk Operations**: Tag, organize, and manage multiple credentials at once
- **Security Monitoring**: Password strength analysis and duplicate detection
- **Audit Trail**: Track when credentials were last accessed or modified

## 📱 Platform Support

ZipLock follows a unified architecture with native applications calling a shared core library directly:

| Platform | Status | Technology | Features |
|----------|--------|------------|----------|
| **Linux** | ✅ Stable | Rust + iced/GTK4 | Full desktop experience, Wayland support |
| **Windows** | 📋 Planned | Rust + Tauri | Native Windows integration |
| **iOS** | 📋 Planned | Swift + SwiftUI | Direct FFI integration with shared core |
| **Android** | 🚧 In Development | Kotlin + Jetpack Compose | Direct FFI integration with shared core |
| **macOS** | 📋 Planned | Swift + SwiftUI | Native macOS experience |

### Architecture Benefits
- **Direct Integration**: Lightweight apps with direct access to core functionality
- **Secure Core**: All cryptographic operations handled by memory-safe Rust library
- **Consistent Experience**: Single implementation ensures identical behavior across platforms
- **Platform Native**: Each client uses platform-specific UI technologies with shared business logic

## 🏗️ Architecture

ZipLock follows a unified architecture with direct FFI integration for maximum portability and simplicity:

```
┌─────────────────┐    Direct   ┌─────────────────┐    File I/O   ┌─────────────────┐
│  Application    │    FFI      │   Shared Core   │ ◄───────────► │ Encrypted 7z    │
│                 │ ◄─────────► │    Library      │               │ Archive         │
│ • Linux (Rust)  │             │     (Rust)      │               │                 │
│ • Windows(Rust) │             │                 │               │                 │
│ • iOS (Swift)   │             │ • Archive Ops   │               │                 │
│ • Android(Kotlin│             │ • Cryptography  │               │                 │
│ • macOS (Swift) │             │ • Validation    │               │                 │
│                 │             │ • C FFI API     │               │                 │
└─────────────────┘             │ • Data Models   │               └─────────────────┘
                                │ • Session Mgmt  │
                                └─────────────────┘
```

### Key Architectural Principles
- **Unified Implementation**: Single core library serves all platforms through FFI
- **Direct Integration**: No IPC overhead, direct function calls for optimal performance
- **Memory Safety**: Rust's guarantees protect against common security vulnerabilities
- **Platform Native UI**: Each platform uses native UI technologies with shared business logic

## 🚀 Getting Started

### Quick Installation

**Linux (Ubuntu/Debian)**:
```bash
wget -O- https://github.com/ejangi/ziplock/releases/latest/download/ziplock_amd64.deb
sudo dpkg -i ziplock_amd64.deb
```

**Arch Linux**:
```bash
# From AUR (recommended)
yay -S ziplock
# or
paru -S ziplock

# Manual installation from release
wget -O- https://github.com/ejangi/ziplock/releases/latest/download/ziplock-0.1.7.tar.gz
# Extract and follow PKGBUILD instructions
```

**Windows**: Not yet available - Windows implementation is currently in development

**iOS**: Available on the App Store (coming soon)

**Android**: Available on Google Play Store (coming soon)

### Building from Source

#### Prerequisites
- **Rust**: 1.70+ with Cargo
- **System Dependencies**: Platform-specific GUI toolkit dependencies

#### Build Steps
```bash
# Clone the repository
git clone https://github.com/ejangi/ziplock.git
cd ziplock

# Build the shared library
cargo build --release --manifest-path shared/Cargo.toml

# Build the app (Linux example)
cargo build --release --bin ziplock --manifest-path apps/linux/Cargo.toml

# Run ZipLock
./target/release/ziplock
```

For detailed build instructions, see the [Build Guide](docs/technical/build.md).

### Configuration

ZipLock can be customized through configuration files:

**Linux**: `~/.config/ziplock/config.yml`
**Windows**: `%APPDATA%/ZipLock/config.yml`

> **⚠️ Configuration Format Change**: Starting with version 0.2.0, ZipLock uses YAML format for all configuration files instead of TOML. If you have existing `.toml` config files, run the migration script: `./scripts/migrations/migrate-config-from-toml-to-yaml.sh`

For complete configuration documentation and examples, see the [Configuration Guide](docs/technical/configuration.md).

## 📖 Documentation

### User Documentation
- [User Guide](docs/TODO.md#user-guide) - Complete guide to using ZipLock (planned)
- [Security Architecture](docs/architecture.md#security-architecture) - Understanding ZipLock's security approach
- [FAQ](docs/TODO.md#faq) - Frequently asked questions (planned)

### Technical Documentation
- [Architecture Overview](docs/architecture.md) - Detailed system architecture
- [Design Guidelines](docs/design.md) - UI/UX design principles
- [Mobile Integration](docs/technical/mobile-integration.md) - iOS and Android integration examples
- [Configuration Guide](docs/technical/configuration.md) - Complete configuration reference with examples
- [Repository Detection Implementation](docs/technical/repository-detection-implementation.md) - Technical implementation details
- [Validation Implementation](docs/technical/validation-implementation.md) - Comprehensive validation system details

### Developer Documentation
- [Development Guide](docs/TODO.md#development-guide) - Setting up the development environment (planned)
- [Contributing Guidelines](docs/TODO.md#contributing-guidelines) - How to contribute to ZipLock (planned)
- [Build Guide](docs/technical/build.md) - Comprehensive build process, packaging, troubleshooting, and CI/CD setup

## 🤝 Contributing

We welcome contributions to ZipLock! Whether you're fixing bugs, adding features, improving documentation, or helping with translations, your help is appreciated.

### Ways to Contribute
- **Code**: Submit pull requests for bug fixes and new features
- **Documentation**: Help improve user and developer documentation
- **Testing**: Report bugs and help test new features
- **Design**: Contribute to UI/UX improvements
- **Translation**: Help translate ZipLock to new languages

### Getting Started
1. Read our [Contributing Guidelines](docs/TODO.md#contributing-guidelines) (planned)
2. Check out [good first issues](https://github.com/ejangi/ziplock/labels/good%20first%20issue)
3. Join our [discussions](https://github.com/ejangi/ziplock/discussions) to connect with the community

### Development Setup
```bash
# Fork and clone the repository
git clone https://github.com/ejangi/ziplock.git
cd ziplock

# Install dependencies and build
cargo build

# Run tests
cargo test

# Start development servers
./scripts/dev/run-linux.sh
```

## 📄 License

ZipLock is licensed under the [Apache License 2.0](LICENSE.md). This means you can:
- ✅ Use it commercially
- ✅ Modify and distribute it
- ✅ Include it in proprietary software
- ✅ Use it privately

The Apache 2.0 license provides strong protection for both users and contributors while ensuring the software remains free and open source.

## 🙏 Acknowledgments

ZipLock is built on the shoulders of giants:

- **[7-Zip](https://www.7-zip.org/)** - For the excellent archive format and compression algorithms
- **[sevenz-rust2](https://github.com/hasenbanck/sevenz-rust2)** - Pure Rust implementation of 7z format
- **[Iconoir](https://iconoir.com/)** - Beautiful free SVG icons used throughout the UI
- **Rust Community** - For excellent cryptography and systems programming crates
- **Contributors** - Everyone who has contributed code, documentation, and feedback
- **[Zed Agenctic Editor](https://zed.dev/agentic)** - This entire app was vibe-coded with Zed's self-hosted Claude Sonnet 4 agent.

## 📞 Support

### Getting Help
- **Documentation**: Check our comprehensive [documentation](docs/)
- **Community Discussions**: Join [GitHub Discussions](https://github.com/ejangi/ziplock/discussions)
- **Issue Tracker**: Report bugs on [GitHub Issues](https://github.com/ejangi/ziplock/issues)

### Security
If you discover a security vulnerability, please follow our [Security Policy](SECURITY.md) for responsible disclosure.

### Professional Support
For enterprise deployments and professional support, contact James Angus at [james@ejangi.com](mailto:james@ejangi.com).

---

<div align="center">
    <p>Made with ❤️ by James Angus <james@ejangi.com> using <a href="https://zed.dev/agentic" target="_blank">Zed</a></p>
  <p>🔐 Your security is our priority 🔐</p>
</div>
