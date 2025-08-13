# ZipLock Examples

This directory contains example files and configurations for ZipLock.

## Configuration Examples

### `config.example.yml`
A comprehensive example configuration file showing all available settings for the ZipLock backend. This includes:

- Storage and archive settings
- Compression configuration
- **Repository validation settings** - Controls comprehensive validation and auto-repair
- Security configuration
- Logging options
- Performance limits

To use this configuration:
1. Copy to your config directory: `~/.config/ziplock/config.yml` (Linux)
2. Modify settings as needed for your environment
3. See the inline comments for detailed explanations of each option

### Validation Configuration Profiles

The example config includes several validation profiles you can use:

**Production Profile (Default):**
- Comprehensive validation enabled
- Auto-repair enabled
- Fails on critical issues
- Detailed logging disabled for performance

**Development Profile:**
- Faster validation (deep validation disabled)
- Detailed logging enabled
- Allows opening repositories with issues

**Legacy Compatibility Profile:**
- Minimal validation
- Maximum compatibility with older repositories

## Usage

To test the validation system with the example configuration, use the development demo script:

```bash
# From project root
./scripts/dev/demo-validation.sh
```

This script will use the example configuration to demonstrate the comprehensive validation and auto-repair capabilities.

## Configuration Documentation

For detailed information about configuration options, see:
- [Technical Documentation](../docs/technical/validation-implementation.md)
- [Architecture Overview](../docs/architecture.md)