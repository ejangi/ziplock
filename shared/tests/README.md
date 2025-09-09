# ZipLock Shared Library Integration Tests

This directory contains integration tests for the ZipLock unified architecture, focusing on testing the complete workflow from credential creation to encrypted archive persistence.

## Overview

These integration tests validate the core functionality of the ZipLock shared library by testing:

- **Archive Persistence**: Creating encrypted 7z archives on disk with credentials
- **Memory Operations**: In-memory archive creation and validation
- **Data Integrity**: Ensuring all credential data is preserved across save/load cycles
- **Error Handling**: Testing password validation, corruption detection, and failure modes
- **Cross-Platform Compatibility**: Using the unified architecture's platform abstraction

## Test Files

### `archive_persistence_test.rs`

Tests that create encrypted archives on disk in the `tests/results/` directory:

- **`test_create_archive_with_credentials`**: Creates archive with multiple credential types
- **`test_load_archive_and_validate_credentials`**: Loads archive and validates all data integrity
- **`test_archive_persistence_across_sessions`**: Tests multiple save/load/modify cycles
- **`test_wrong_password_fails`**: Validates password authentication
- **`test_archive_integrity_validation`**: Tests repository integrity checking
- **`test_edge_cases`**: Tests special characters, unicode, and edge cases

**Features Tested**:
- Multiple credential types (login, credit card, TOTP, secure notes)
- Various field types (text, password, email, URL, phone, dates, etc.)
- Tags and metadata persistence
- TOTP code generation validation
- Archive file creation and cleanup
- Password-based encryption/decryption

### `memory_archive_test.rs`

Tests that operate entirely in memory without touching the filesystem:

- **`test_create_memory_archive`**: Creates archive in memory and validates structure
- **`test_memory_archive_round_trip`**: Full serialize/deserialize cycle validation
- **`test_memory_archive_with_invalid_password`**: Password validation in memory
- **`test_memory_archive_serialization_integrity`**: Complex data serialization testing
- **`test_memory_provider_failure_modes`**: Error handling and failure scenarios

**Features Tested**:
- In-memory archive creation using custom FileOperationProvider
- File map serialization/deserialization
- Complex credential data with unicode and special characters
- YAML serialization integrity
- Custom field types and metadata
- Archive extraction and validation without filesystem I/O

## Test Data

### Credential Types Tested

1. **Login Credentials**
   - Username/password combinations
   - URLs and notes
   - Tags for organization

2. **Credit Card Information**
   - Card numbers, expiry dates, CVV
   - Cardholder names and metadata
   - Sensitive field handling

3. **TOTP-Enabled Accounts**
   - Secret keys for time-based codes
   - TOTP code generation validation
   - Recovery codes and backup information

4. **Banking Information**
   - Account and routing numbers
   - Sensitive financial data handling
   - Connection strings and credentials

5. **API Keys and Tokens**
   - Service API keys
   - Endpoints and usage information
   - Development and AI service credentials

6. **Secure Notes**
   - SSH private/public key pairs
   - Multi-line text areas
   - Server and infrastructure information

### Field Types Tested

- `Text` - Basic text fields
- `Password` - Sensitive password fields
- `Email` - Email address validation
- `Url` - URL formatting and validation
- `Username` - User identification fields
- `Phone` - Phone number formatting
- `CreditCardNumber` - Credit card validation
- `ExpiryDate` - Date formatting
- `Cvv` - Security code handling
- `TotpSecret` - TOTP secret key storage
- `TextArea` - Multi-line text content
- `Number` - Numeric data
- `Date` - Date field handling
- `Custom` - Custom field types

## Running the Tests

### Individual Test Files

```bash
# Run archive persistence tests
cargo test --package ziplock-shared --test archive_persistence_test

# Run memory archive tests  
cargo test --package ziplock-shared --test memory_archive_test
```

### All Integration Tests

```bash
# Run both test files
cargo test --package ziplock-shared --test archive_persistence_test --test memory_archive_test

# Run with output
cargo test --package ziplock-shared --test archive_persistence_test --test memory_archive_test -- --nocapture
```

### Specific Test Cases

```bash
# Run a specific test
cargo test --package ziplock-shared --test archive_persistence_test test_create_archive_with_credentials

# Run tests matching a pattern
cargo test --package ziplock-shared archive_persistence
```

## Test Results Location

- **Disk Archives**: Created in `tests/results/` directory with unique timestamped filenames
- **Auto-Cleanup**: Archive files are automatically cleaned up via `Drop` implementation
- **No Persistence**: Test files don't persist between test runs
- **Thread Safety**: Each test uses unique filenames to avoid conflicts in parallel execution

## Architecture Validation

These tests validate key aspects of the unified architecture:

### Memory Repository
- Pure in-memory credential operations
- No file I/O during business logic operations
- Data validation and integrity checking
- CRUD operations on credentials

### File Operation Provider
- Clean separation between memory and file operations
- Platform-specific file handling abstraction
- 7z archive creation/extraction via `sevenz-rust2`
- AES-256 encryption/decryption

### Repository Manager
- Coordination between memory and file operations
- Save/load cycle management
- Error handling across components
- Session management and state tracking

### Data Models
- Comprehensive credential data structures
- Field type system with validation
- Metadata and tag management
- Serialization/deserialization integrity

## Security Testing

### Encryption Validation
- Password-based archive encryption
- AES-256 encryption verification
- Invalid password rejection
- Archive corruption detection

### Data Protection
- Sensitive field identification
- Secure memory handling
- No plaintext credential leakage
- Proper cleanup of sensitive data

### Error Handling
- Graceful failure modes
- No sensitive data in error messages
- Corruption detection and reporting
- Authentication failure handling

## Performance Characteristics

These tests serve as performance benchmarks:

- **Credential Operations**: < 1ms for individual CRUD operations
- **Archive Creation**: < 100ms for archives with 5 credentials
- **Archive Loading**: < 50ms for small to medium archives
- **Memory Usage**: Efficient in-memory operations with minimal overhead
- **TOTP Generation**: < 1ms for code generation

## Development Guidelines

### Adding New Tests

1. Follow the naming convention: `test_[component]_[functionality]`
2. Use the existing test fixtures (`ArchivePersistenceTest`, `MemoryArchiveTest`)
3. Include both positive and negative test scenarios
4. Test edge cases and error conditions
5. Validate data integrity in all scenarios

### Test Data Guidelines

- Use realistic but fictional test data
- Never use real passwords or sensitive information
- Test with unicode characters and special symbols
- Include empty, minimal, and maximal data scenarios
- Test various credential types and field combinations

### Error Testing

- Test invalid passwords and corrupted archives
- Validate error messages don't leak sensitive data
- Test failure modes and recovery scenarios
- Ensure graceful degradation

## Continuous Integration

These tests are designed to run in CI/CD environments:

- **No External Dependencies**: Tests run entirely self-contained
- **Deterministic**: Results are consistent across runs
- **Fast Execution**: Complete test suite runs in < 5 seconds
- **Parallel Safe**: Tests use unique identifiers to avoid conflicts
- **Auto-Cleanup**: No artifacts left after test completion

## Troubleshooting

### Common Issues

1. **"Failed to create tests/results directory"**
   - Ensure write permissions on the project directory
   - Check available disk space

2. **"Archive file should exist on disk"**
   - Verify the `DesktopFileProvider` is working correctly
   - Check that archive creation isn't failing silently

3. **"InvalidPassword" errors**
   - Verify password consistency between create and open operations
   - Check that mock archive format is handled correctly

4. **Compilation errors**
   - Ensure all dependencies are available
   - Check that the shared library API matches test expectations

### Debug Mode

Run tests with debug output:

```bash
RUST_LOG=debug cargo test --package ziplock-shared --test archive_persistence_test -- --nocapture
```

### Test Isolation

Each test runs in isolation with:
- Unique archive filenames (timestamp + thread ID)
- Independent repository instances
- Automatic cleanup via `Drop` traits
- No shared state between tests

## Future Enhancements

Planned test improvements:

- [ ] Performance regression testing
- [ ] Large dataset handling (1000+ credentials)
- [ ] Concurrent access simulation
- [ ] Memory leak detection
- [ ] Cross-platform file system testing
- [ ] Backup/restore validation
- [ ] Plugin system integration tests

---

These integration tests provide comprehensive validation of the ZipLock unified architecture, ensuring data integrity, security, and cross-platform compatibility across all supported operations.