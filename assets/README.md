# ZipLock Assets

This directory contains static assets used throughout the ZipLock application, including icons, fonts, and other visual resources.

## Icons

The icons used in ZipLock are sourced from [Iconoir](https://iconoir.com/), a beautiful collection of free SVG icons created by Luca Burgio and contributors.

### Icon Attribution

- **Source**: [Iconoir](https://iconoir.com/)
- **License**: MIT License
- **Creator**: Luca Burgio and contributors
- **Repository**: https://github.com/lucaburgio/iconoir

### Icons Used

| Icon | File | Usage |
|------|------|-------|
| Alert | `icons/alert.svg` | Info alerts and general notifications |
| Check | `icons/check.svg` | Success states and confirmations |
| Error | `icons/error.svg` | Error states and failures |
| Warning | `icons/warning.svg` | Warning states and cautions |
| Eye (Solid) | `icons/eye-solid.svg` | Password visibility toggle |
| ZipLock Logo | `icons/ziplock-logo.svg` | Main application logo |

### Application Icons

The ZipLock application icons (`ziplock-icon-*.png`) are custom-created for the project and follow the design guidelines outlined in `docs/design.md`.

## Fonts

Font assets will be placed in the `fonts/` directory when custom fonts are added to the project.

## License

- **Iconoir icons**: MIT License - see [Iconoir License](https://github.com/lucaburgio/iconoir/blob/main/LICENSE)
- **ZipLock custom assets**: Apache 2.0 License (same as the main project)

## Usage

Icons are embedded directly into the frontend applications using Rust's `include_bytes!` macro for optimal performance and to ensure they're always available regardless of the installation method.

### Adding New Icons

When adding new icons to the project:

1. Download the SVG from [Iconoir](https://iconoir.com/)
2. Place it in the `icons/` directory
3. Update the frontend theme files to include the new icon
4. Copy the icon to all frontend resource directories:
   - `frontend/linux/resources/icons/`
   - `frontend/windows/resources/icons/`
   - `frontend/mobile/android/app/src/main/res/drawable/`
   - `frontend/mobile/ios/ZipLock/Resources/`
5. Update this README to document the new icon

## Acknowledgments

Special thanks to:
- [Iconoir](https://iconoir.com/) and Luca Burgio for providing beautiful, free SVG icons
- The open-source community for creating and maintaining high-quality design resources