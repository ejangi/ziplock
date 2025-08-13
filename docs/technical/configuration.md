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
# ZipLock Backend Configuration Example
# Copy to ~/.config/ziplock/config.yml (Linux) or %APPDATA%/ZipLock/config.yml (Windows)

# IPC Communication Settings
ipc:
  # Unix socket path for backend communication
  socket_path: "/tmp/ziplock/backend.sock"
  # Socket file permissions (octal)
  socket_permissions: 0o600
  # Maximum concurrent connections
  max_connections: 10
  # Connection timeout in seconds
  connection_timeout: 30
  # Request timeout in seconds
  request_timeout: 60
  # Log all IPC requests (for debugging)
  log_requests: false

# Storage and Archive Settings
storage:
  # Default directory for storing archives
  default_archive_dir: "~/Documents/ZipLock"
  # Maximum archive file size in MB (0 = unlimited)
  max_archive_size_mb: 500
  # Number of backup copies to maintain
  backup_count: 5
  # Enable automatic backup creation
  auto_backup: true
  # Custom backup directory (optional)
  backup_dir: null
  # File lock timeout in seconds
  file_lock_timeout: 30
  # Custom temporary directory (optional)
  temp_dir: null
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
    # Enable multi-threaded compression
    multi_threaded: true
    # Dictionary size in MB for compression
    dictionary_size_mb: 64
    # Block size in MB (0 = auto)
    block_size_mb: 0

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
  # Argon2 key derivation settings
  key_derivation_iterations: 3
  key_derivation_memory_kb: 65536  # 64 MB
  key_derivation_parallelism: 4

  # Auto-lock timeout in seconds (0 = disabled)
  auto_lock_timeout: 900  # 15 minutes

  # Maximum authentication attempts before lockout
  max_auth_attempts: 5
  # Lockout duration in seconds after max attempts
  auth_lockout_duration: 300  # 5 minutes

  # Enable memory protection features
  memory_protection: true
  # Clipboard clear timeout in seconds
  clipboard_clear_timeout: 30

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
  # Maximum log file size in MB before rotation
  max_log_size_mb: 10
  # Number of rotated log files to keep
  log_rotation_count: 5
  # Use JSON format for structured logging
  json_format: false
  # Enable audit logging for security events
  audit_logging: true

# Performance and Resource Limits
limits:
  # Maximum memory usage in MB
  max_memory_mb: 512
  # Maximum cached credentials in memory
  max_cached_credentials: 1000
  # Cache TTL in seconds
  cache_ttl: 300  # 5 minutes
  # Maximum concurrent operations
  max_concurrent_operations: 10
  # Operation timeout in seconds
  operation_timeout: 120  # 2 minutes
  # Enable performance metrics collection
  enable_metrics: false
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
    multi_threaded: true
  validation:
    enable_comprehensive_validation: true
    deep_validation: true
    auto_repair: true
    fail_on_critical_issues: true
    log_validation_details: false

security:
  auto_lock_timeout: 900
  memory_protection: true
  passphrase_requirements:
    min_length: 16
    require_lowercase: true
    require_uppercase: true
    require_numeric: true
    require_special: true

logging:
  level: "warn"
  file_logging: true
  audit_logging: true
```

### Development Profile

```yaml
storage:
  backup_count: 3
  compression:
    level: 1  # Faster builds
    multi_threaded: true
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
  json_format: false  # Easier to read during development
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

### IPC Settings

Controls how the frontend communicates with the backend service:

- `socket_path`: Location of the Unix domain socket for communication
- `socket_permissions`: File permissions for the socket (security)
- `max_connections`: Prevents resource exhaustion
- `connection_timeout`: Prevents hanging connections
- `request_timeout`: Maximum time for a single request
- `log_requests`: Enable for debugging communication issues

### Storage Settings

Controls how archives are stored and managed:

- `default_archive_dir`: Where new archives are created by default
- `max_archive_size_mb`: Prevents extremely large archive files
- `backup_count`: Number of automatic backups to maintain
- `auto_backup`: Whether to create backups automatically
- `file_lock_timeout`: Prevents concurrent access issues
- `verify_integrity`: Basic integrity checking on archive open

### Compression Settings

Fine-tune 7z compression behavior:

- `level`: Higher values provide better compression but slower performance
- `solid`: Improves compression ratio but slower random access
- `multi_threaded`: Use multiple CPU cores for compression
- `dictionary_size_mb`: Larger dictionaries improve compression for similar data
- `block_size_mb`: Controls memory usage during compression

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

- `key_derivation_*`: Argon2 parameters for master key derivation
- `auto_lock_timeout`: Automatically lock after inactivity
- `max_auth_attempts`: Brute force protection
- `memory_protection`: Use secure memory allocation when available
- `clipboard_clear_timeout`: Automatically clear clipboard after copying passwords
- `passphrase_requirements`: Enforce strong master passphrase policies

### Logging Settings

Control logging behavior and output:

- `level`: Minimum log level to record
- `file_logging`: Whether to write logs to files
- `log_file`: Custom log file location
- `max_log_size_mb`: Log rotation size threshold
- `json_format`: Use structured JSON logging
- `audit_logging`: Enable security event logging

### Performance Limits

Prevent resource exhaustion:

- `max_memory_mb`: Total memory usage limit
- `max_cached_credentials`: Credential cache size limit
- `cache_ttl`: How long to keep cached data
- `max_concurrent_operations`: Prevent resource overload
- `operation_timeout`: Maximum time for any single operation
- `enable_metrics`: Collect performance metrics

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
- `ZIPLOCK_SOCKET_PATH`: Override the IPC socket path
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
- Check that socket paths are in writable directories
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
- [IPC Client Examples](ipc-client-examples.md) - Examples of using configured backend services
- [Mobile Integration](mobile-integration.md) - Configuration considerations for mobile platforms