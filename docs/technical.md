# ZipLock Technical Documentation

This document serves as the central index for all technical documentation in the ZipLock project. Technical documentation is organized into focused topics, each maintained as individual markdown files in the `docs/technical/` directory.

## ⚠️ IMPORTANT: Technical Documentation Location

**ALL TECHNICAL DOCUMENTATION MUST BE PLACED IN THE `docs/technical/` DIRECTORY.**

This includes but is not limited to:
- Implementation guides and technical specifications
- Architecture documentation
- API documentation
- Mobile integration guides
- Platform-specific implementation details
- Security implementation details
- Performance optimization guides

Do not place technical documentation in the root `docs/` directory or other locations. The `docs/technical/` directory is the designated location for all technical content to maintain organization and discoverability.

## Architecture and Design

- [Architecture Overview](architecture.md) - Complete system architecture and component relationships
- [Design Guidelines](design.md) - UI/UX design principles and visual standards
- [Repository Detection Implementation](technical/repository-detection-implementation.md) - Technical implementation details for repository detection
- [Mobile Integration Guide](technical/mobile-integration.md) - Complete mobile platform integration documentation
- [Mobile Shared Implementation](technical/mobile-shared-implementation.md) - Shared library integration for mobile platforms

## Security and Cryptography

- Security architecture and threat model (see [Architecture Overview](architecture.md#security-architecture))
- Encryption implementation details
- Key management and derivation processes
- Authentication and session management

## Storage and Data Management

- 7z archive format usage and optimization
- Data structure specifications
- Backup and recovery mechanisms
- File locking and concurrent access prevention

## Inter-Process Communication

- Frontend-backend communication protocols
- API specifications and data formats
- Error handling and message passing
- Session management across components

## Platform-Specific Implementation

### Desktop Platforms
- Linux implementation details (Rust + iced/GTK4)
- Windows implementation details (Rust + Tauri)
- macOS implementation planning (Swift + SwiftUI)

### Mobile Platforms
- iOS implementation planning (Swift + SwiftUI)
- Android implementation planning (Kotlin + Jetpack Compose)

## Development and Testing

- [Build Guide](build.md) - Comprehensive build troubleshooting, containerized builds, and glibc compatibility
- Build system configuration
- Testing strategies and coverage requirements
- Continuous integration setup
- Code quality standards and linting rules

## Performance and Optimization

- Compression algorithm selection and tuning
- Memory management best practices
- UI responsiveness optimization
- Resource usage monitoring

## Contributing Technical Documentation

When adding new technical documentation:

1. **Create individual files** in the `docs/technical/` directory
2. **Use descriptive filenames** that clearly indicate the content (e.g., `encryption-implementation.md`)
3. **Follow naming convention** of lowercase words separated by hyphens
4. **Update this index** by adding appropriate links in the relevant sections
5. **Cross-reference related documents** to maintain documentation cohesion
6. **Include code examples** and diagrams where appropriate
7. **Maintain consistent formatting** following the project's documentation standards

## Protected Files

The following documentation files should **not be edited** as they serve specific project management purposes:

- `docs/01-initial-prompt.txt` - Original project requirements and prompt
- `docs/TODO.md` - Project task tracking and development roadmap

These files provide historical context and project planning information that should remain unchanged to preserve the development history and planning artifacts.

## Documentation Standards

All technical documentation should follow these guidelines:

- **Clear headings** with proper markdown hierarchy
- **Code examples** formatted with appropriate syntax highlighting
- **Diagrams and flowcharts** for complex processes
- **Cross-references** to related documentation
- **Version compatibility notes** where applicable
- **Security considerations** for implementation details
- **Platform-specific notes** when features differ across platforms

For questions about technical documentation or suggestions for new topics, please open an issue or start a discussion in the project repository.
