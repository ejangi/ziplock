# Configuration Guide

This guide covers all configuration options available in ZipLock, including comprehensive examples and best practices for different deployment scenarios.

## Configuration File Location

ZipLock looks for configuration files in the following locations:

- **Linux**: `~/.config/ziplock/config.yml`
- **Windows**: `%APPDATA%/ZipLock/config.yml`
- **macOS**: `~/Library/Application Support/ZipLock/config.yml`

## Configuration Format

Starting with version 0.2.0, ZipLock uses YAML format for all configuration files. If you have existing `.toml` config files, run the migration script:

```bash
./scripts/migrations/migrate-config-from-toml-to-yaml.sh
```

## Complete Configuration Example

Here's a comprehensive example configuration file showing all available options:

```yaml
# ZipLock Configuration Example
# Copy to ~/.config/ziplock/config.yml (Linux) or %APPDATA%/ZipLock/config.yml (Windows)

# Storage and Archive Settings
storage:
  # Default directory for storing archives
  default_archive_dir: "~/Documents/ZipLock"
  # Number of backup copies to maintain
  backup_count: 5
  # Enable automatic backup creation
  auto_backup: true
  # Custom backup directory (optional)
  backup_dir: null
  # File lock timeout in seconds
  file_lock_timeout: 30
  # Enable basic archive integrity checking
  verify_integrity: true
  # Minimum master password length
  min_password_length: 12

  # 7z Compression Settings
  compression:
    # Compression level (0-9, higher = better compression but slower)
    level: 6
    # Use solid compression (better compression but slower random access)
    solid: false

  # Repository Validation Settings
  validation:
    # Enable comprehensive repository validation when opening archives
    # This performs detailed checks beyond basic integrity verification
    enable_comprehensive_validation: true

    # Enable deep validation that checks all credential files individually
    # This is more thorough but takes longer for large repositories
    deep_validation: true

    # Check for legacy format repositories and automatically migrate them
    # Helps maintain compatibility with older ZipLock versions
    check_legacy_formats: true

    # Validate YAML schema compliance for all repository files
    # Ensures all files conform to the expected format specification
    validate_schemas: true

    # Automatically attempt to repair validation issues when found
    # If disabled, validation issues will only be reported but not fixed
    auto_repair: true

    # Fail repository opening if critical validation issues are found
    # When false, the repository will still open with warnings
    fail_on_critical_issues: true

    # Log detailed validation reports for monitoring and debugging
    # Useful for troubleshooting repository issues
    log_validation_details: false

# Security Settings
security:
  # Auto-lock timeout in seconds (0 = disabled)
  auto_lock_timeout: 900  # 15 minutes

  # Master passphrase requirements
  passphrase_requirements:
    min_length: 12
    require_lowercase: true
    require_uppercase: true
    require_numeric: true
    require_special: true
    max_length: 0  # 0 = no limit
    min_unique_chars: 8

# Logging Settings
logging:
  # Log level: trace, debug, info, warn, error
  level: "info"
  # Enable file logging
  file_logging: true
  # Log file path (defaults to system log directory)
  log_file: null
```

## Configuration Profiles

Different deployment scenarios may require different configuration approaches. Here are some recommended profiles:

### Production Profile (Recommended Default)

```yaml
storage:
  backup_count: 10
  auto_backup: true
  compression:
    level: 6
    solid: false
  validation:
    enable_comprehensive_validation: true
    deep_validation: true
    auto_repair: true
    fail_on_critical_issues: true
    log_validation_details: false

security:
  auto_lock_timeout: 900
  passphrase_requirements:
    min_length: 16
    require_lowercase: true
    require_uppercase: true
    require_numeric: true
    require_special: true

logging:
  level: "warn"
  file_logging: true
```

### Development Profile

```yaml
storage:
  backup_count: 3
  compression:
    level: 1  # Faster builds
  validation:
    enable_comprehensive_validation: true
    deep_validation: false  # Faster validation
    auto_repair: true
    fail_on_critical_issues: false
    log_validation_details: true  # Detailed debugging

security:
  auto_lock_timeout: 0  # No auto-lock during development
  passphrase_requirements:
    min_length: 8  # Relaxed for testing

logging:
  level: "debug"
  file_logging: true
```

### Legacy Compatibility Profile

```yaml
storage:
  backup_count: 5
  validation:
    enable_comprehensive_validation: false
    deep_validation: false
    check_legacy_formats: true
    validate_schemas: false
    auto_repair: true
    fail_on_critical_issues: false

security:
  passphrase_requirements:
    min_length: 8
    require_lowercase: false
    require_uppercase: false
    require_numeric: false
    require_special: false

logging:
  level: "info"
```

## Configuration Sections

### Storage Settings

Controls how archives are stored and managed:

- `default_archive_dir`: Where new archives are created by default
- `backup_count`: Number of automatic backups to maintain
- `auto_backup`: Whether to create backups automatically
- `backup_dir`: Custom backup directory (optional, defaults to subdirectory of archive dir)
- `file_lock_timeout`: Prevents concurrent access issues
- `verify_integrity`: Basic integrity checking on archive open
- `min_password_length`: Minimum master password length requirement

### Compression Settings

Fine-tune 7z compression behavior:

- `level`: Higher values provide better compression but slower performance (0-9)
- `solid`: Improves compression ratio but slower random access

### Validation Settings

Controls repository validation and auto-repair:

- `enable_comprehensive_validation`: Enable advanced validation beyond basic integrity
- `deep_validation`: Check individual credential files (slower but thorough)
- `check_legacy_formats`: Maintain compatibility with older ZipLock versions
- `validate_schemas`: Ensure all files conform to expected format
- `auto_repair`: Automatically fix validation issues when possible
- `fail_on_critical_issues`: Whether to refuse opening repositories with critical issues
- `log_validation_details`: Enable detailed validation logging for troubleshooting

### Security Settings

Configure security policies and encryption:

- `auto_lock_timeout`: Automatically lock after inactivity (seconds, 0 = disabled)
- `passphrase_requirements`: Enforce strong master passphrase policies
  - `min_length`: Minimum passphrase length
  - `require_lowercase`: Require lowercase letters
  - `require_uppercase`: Require uppercase letters
  - `require_numeric`: Require numbers
  - `require_special`: Require special characters
  - `max_length`: Maximum passphrase length (0 = no limit)
  - `min_unique_chars`: Minimum number of unique characters

### Logging Settings

Control logging behavior and output:

- `level`: Minimum log level to record (trace, debug, info, warn, error)
- `file_logging`: Whether to write logs to files
- `log_file`: Custom log file location (optional)

## Frontend Configuration

The frontend maintains its own configuration file for UI and application settings:

```yaml
# Frontend Configuration Example
repository:
  # Current repository path
  path: null
  # Default directory for creating new repositories
  default_directory: "~/Documents/ZipLock"
  # Recently accessed repositories
  recent_repositories: []
  # Maximum number of recent repositories to remember
  max_recent: 10
  # Additional directories to search for repositories
  search_directories: []

ui:
  # Window dimensions
  window_width: 1200
  window_height: 800
  # Remember and restore window state
  remember_window_state: true
  # Show setup wizard on startup if no repository is configured
  show_wizard_on_startup: true

app:
  # Auto-lock timeout in minutes (0 = disabled)
  auto_lock_timeout: 15
  # Clear clipboard after copying password (seconds)
  clipboard_timeout: 30
  # Enable auto-backup
  enable_backup: true

version: "1.0"
```

## Testing Configuration

To test the validation system with example configuration:

```bash
# From project root
./scripts/dev/demo-validation.sh
```

This script demonstrates the comprehensive validation and auto-repair capabilities using a demo configuration (see `scripts/dev/demo-config.yml`). The demo configuration is optimized for testing with verbose logging and relaxed security settings.

## Migration from TOML

If you're upgrading from a version that used TOML configuration:

1. **Backup your current config**: Copy your existing `.toml` file to a safe location
2. **Run the migration script**: `./scripts/migrations/migrate-config-from-toml-to-yaml.sh`
3. **Verify the migration**: Check that all your settings were converted correctly
4. **Remove old files**: Delete the `.toml` files after confirming the migration worked

## Environment Variables

Some configuration options can be overridden with environment variables:

- `ZIPLOCK_CONFIG_PATH`: Override the default configuration file location
- `ZIPLOCK_LOG_LEVEL`: Override the logging level (trace, debug, info, warn, error)
- `ZIPLOCK_AUTO_LOCK_TIMEOUT`: Override the auto-lock timeout

Example:
```bash
ZIPLOCK_LOG_LEVEL=debug ZIPLOCK_AUTO_LOCK_TIMEOUT=0 ziplock
```

## Troubleshooting Configuration

### Common Issues

**Configuration file not found**:
- Ensure the file is in the correct location for your platform
- Check file permissions (should be readable by the user running ZipLock)
- Verify YAML syntax using a YAML validator

**Invalid YAML syntax**:
- Use proper indentation (spaces, not tabs)
- Ensure proper quoting of string values
- Check for trailing spaces or special characters

**Permission denied errors**:
- Ensure the config directory is writable
- Verify file permissions match security requirements

### Validation

To validate your configuration file:

```bash
# Check YAML syntax
python3 -c "import yaml; yaml.safe_load(open('~/.config/ziplock/config.yml'))"

# Test configuration with ZipLock
ziplock --validate-config
```

## Related Documentation

- [Validation Implementation](validation-implementation.md) - Technical details about repository validation
- [Build Guide](build.md) - Build-time configuration options
- [Architecture Overview](../architecture.md) - How configuration fits into the overall system
- [Mobile Integration](mobile-integration.md) - Configuration considerations for mobile platforms