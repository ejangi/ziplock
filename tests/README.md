# ZipLock Integration Tests

This directory contains comprehensive integration tests for the ZipLock password manager, focusing on credential persistence, data integrity, and end-to-end functionality.

## Overview

The integration tests verify that:
- Credentials are properly saved to the encrypted repository
- Changes persist across archive close/reopen cycles
- Data integrity is maintained during operations
- The frontend and backend communicate correctly
- Edge cases and error conditions are handled properly

## Test Structure

### Integration Tests (`integration/`)

#### `credential_persistence_test.rs`
Comprehensive tests for credential data persistence:
- **Credential Creation**: Verifies new credentials are saved and retrievable
- **Credential Updates**: Ensures updates are persisted to disk
- **Multiple Operations**: Tests complex workflows with multiple credentials
- **Edge Cases**: Special characters, large data, empty fields
- **Archive Integrity**: Multiple save/load cycles
- **Data Consistency**: Rapid consecutive operations

#### `simple_persistence_test.rs`
Focused tests for basic persistence functionality:
- **Basic Save/Retrieve**: Simple credential creation and retrieval
- **Update Persistence**: Credential modification persistence
- **Bug Verification**: Demonstrates the issue when auto-save is disabled

### Running Tests

#### Quick Test Run
```bash
# Run all integration tests
cargo test --test credential_persistence_test
cargo test --test simple_persistence_test

# Run with output
cargo test --test credential_persistence_test -- --nocapture
```

#### Using the Test Runner Scripts

**For Fresh Environment (starts its own backend):**
```bash
# Run complete test suite
./scripts/run-integration-tests.sh

# Run only integration tests
./scripts/run-integration-tests.sh --integration-only

# Run with verbose output
./scripts/run-integration-tests.sh --verbose

# Use existing backend if running
./scripts/run-integration-tests.sh --use-existing-backend

# Kill existing backend and start fresh
./scripts/run-integration-tests.sh --kill-existing-backend
```

**For Existing Backend (recommended for development):**
```bash
# Run with already running backend
./scripts/run-tests-with-existing-backend.sh

# Run only storage tests
./scripts/run-tests-with-existing-backend.sh --storage-only

# Quick test run
./scripts/run-tests-with-existing-backend.sh --quick
```

**Test Backend Connectivity:**
```bash
# Quick connectivity check
./scripts/test-backend-connection.sh

# Verbose connectivity test
./scripts/test-backend-connection.sh --verbose

# Wait for backend to become ready
./scripts/test-backend-connection.sh --wait 30
```

#### CI/CD Integration
```bash
# For automated builds
./scripts/run-integration-tests.sh --no-build

# For environments with existing backend
./scripts/run-tests-with-existing-backend.sh --integration-only
```

## Test Configuration

### Environment Variables
- `RUST_LOG=debug` - Enable debug logging during tests
- `ZIPLOCK_TEST_TIMEOUT=30` - Set test timeout in seconds

### Test Data
- Tests use temporary directories that are automatically cleaned up
- Each test creates its own isolated archive file
- No shared state between tests

## Test Scripts Overview

### 1. `run-integration-tests.sh` - Full Test Suite
- Builds the project from scratch
- Starts its own backend service
- Runs complete test suite (unit + integration + E2E)
- Handles backend lifecycle management
- Best for: CI/CD, fresh environments, complete validation

### 2. `run-tests-with-existing-backend.sh` - Development Testing
- Uses already running backend service
- Focuses on integration and storage tests
- Fast execution (no build/startup time)
- Best for: Development workflow, quick validation

### 3. `test-backend-connection.sh` - Connectivity Verification
- Tests if backend is running and responsive
- Validates socket connectivity
- Provides troubleshooting information
- Best for: Debugging backend issues, pre-test validation

## Key Test Scenarios

### 1. Credential Persistence Flow
```
Create Archive → Add Credential → Save → Close → Reopen → Verify Data
```

### 2. Update Persistence Flow
```
Open Archive → Update Credential → Save → Close → Reopen → Verify Changes
```

### 3. Multiple Operations Flow
```
Create → Update → Delete → Mixed Operations → Verify Final State
```

### 4. Error Handling
```
Invalid Operations → Verify Graceful Failures → Data Integrity Maintained
```

## Expected Test Results

### ✅ Passing Tests Indicate:
- Credentials are properly saved to the encrypted 7z archive
- The backend automatically saves changes after operations
- Data persists across application restarts
- Archive integrity is maintained
- Frontend-backend communication works correctly

### ❌ Failing Tests May Indicate:
- Missing auto-save functionality
- Data corruption issues
- Encryption/decryption problems
- File locking issues
- IPC communication failures
- Backend not running or not responsive
- Socket permission issues
- Archive file corruption

## Debugging Test Failures

### 1. Check Test Logs
```bash
# Test output is saved to test-results/
cat test-results/integration-tests.log
cat test-results/backend.log
```

### 2. Check Backend Status
```bash
# Test if backend is responsive
./scripts/test-backend-connection.sh --verbose

# Check backend process
pgrep -af ziplock-backend
```

### 3. Run Individual Tests
```bash
# Run a specific test with output
cargo test test_credential_creation_persistence -- --nocapture

# Run only storage tests
cargo test --package ziplock-backend storage:: -- --test-threads=1
```

### 4. Enable Debug Logging
```bash
# Set environment variable for detailed logs
RUST_LOG=debug cargo test --test credential_persistence_test

# Run with existing backend and debug logging
RUST_LOG=debug ./scripts/run-tests-with-existing-backend.sh --verbose
```

### 5. Manual Verification
```bash
# Check if archive files are created
ls -la /tmp/test-archives/

# Check socket files
ls -la /tmp/ziplock*.sock

# Test socket connectivity manually
echo '{"request_id":"test","session_id":null,"request":{"Ping":{"client_info":"manual-test"}}}' | nc -U /tmp/ziplock.sock
```

## Common Issues and Solutions

### Issue: Tests Fail with "Archive not found"
**Solution**: 
- Ensure the backend service is running: `pgrep ziplock-backend`
- Check file permissions in test directories
- Verify sufficient disk space in /tmp

### Issue: "Session timeout" or "Backend not responsive" errors
**Solution**: 
- Test connectivity: `./scripts/test-backend-connection.sh`
- Check backend startup and IPC socket creation
- Verify socket permissions: `ls -la /tmp/ziplock.sock`
- Restart backend if needed

### Issue: Credential fields are empty after reopen
**Solution**: 
- This indicates the auto-save fix is not working
- Verify the backend saves after operations
- Check backend logs for save errors
- Run storage-specific tests: `./scripts/run-tests-with-existing-backend.sh --storage-only`

### Issue: Permission denied errors
**Solution**: 
- Check file system permissions and available disk space
- Ensure user can write to /tmp directory
- Verify socket file permissions

### Issue: "Backend failed to start within 30 seconds"
**Solution**: 
- Backend might already be running: use `--use-existing-backend`
- Or use the existing backend script: `./scripts/run-tests-with-existing-backend.sh`
- Check if port/socket is already in use

## Test Data Validation

The tests verify the following data integrity aspects:

### Field Data
- ✅ Field values match exactly after save/load
- ✅ Field types are preserved
- ✅ Sensitive field flags are maintained
- ✅ Field metadata is preserved

### Credential Metadata
- ✅ Titles and types are preserved
- ✅ Tags are maintained
- ✅ Notes persist correctly
- ✅ Timestamps are updated appropriately

### Archive Structure
- ✅ Proper 7z encryption
- ✅ Correct directory structure (`credentials/{id}/record.yml`)
- ✅ Metadata files are created
- ✅ File integrity across operations

## Performance Benchmarks

The integration tests also serve as performance benchmarks:

- **Single Credential Operations**: < 100ms
- **Multiple Credential Batch**: < 500ms for 10 credentials
- **Archive Save/Load**: < 1s for archives with 100 credentials
- **Search Operations**: < 50ms for 1000 credentials

## Contributing to Tests

### Adding New Tests
1. Create test functions in appropriate files
2. Use the `CredentialPersistenceTest` fixture for setup
3. Follow the naming convention: `test_[functionality]_[scenario]`
4. Include both positive and negative test cases

### Test Naming Convention
- `test_credential_[operation]_[scenario]` - For credential operations
- `test_archive_[operation]_[scenario]` - For archive-level operations
- `test_[component]_[functionality]_[condition]` - For specific component tests

### Test Documentation
- Include docstring comments explaining test purpose
- Document expected outcomes
- Note any special setup requirements
- Reference related issues or PRs

## Security Considerations

### Test Data
- Use only test data in integration tests
- Never use real passwords or sensitive information
- Test credentials are clearly marked as test data

### Cleanup
- All test artifacts are automatically cleaned up
- Temporary files are securely deleted
- No sensitive data remains after test completion

### Isolation
- Each test runs in isolation
- No shared state between tests
- Independent temporary directories per test

## Future Test Enhancements

### Planned Additions
- [ ] Concurrent access simulation tests
- [ ] Network interruption simulation
- [ ] Large dataset performance tests
- [ ] Memory usage profiling
- [ ] Security vulnerability tests
- [ ] Cross-platform compatibility tests

### Test Automation
- [x] Multiple test runner scripts for different scenarios
- [x] Backend connectivity verification
- [x] Comprehensive error handling and reporting
- [ ] Automatic test runs on PR creation
- [ ] Performance regression detection
- [ ] Test coverage reporting
- [ ] Automated test result notifications

## Development Workflow

### Daily Development Testing
```bash
# 1. Start backend once
cargo run --release --bin ziplock-backend &

# 2. Run quick tests during development
./scripts/run-tests-with-existing-backend.sh --quick

# 3. Run full integration tests before commits
./scripts/run-tests-with-existing-backend.sh
```

### Pre-commit Testing
```bash
# Full validation before committing
./scripts/run-integration-tests.sh --integration-only
```

### CI/CD Pipeline Testing
```bash
# In CI environment
./scripts/run-integration-tests.sh --no-build --kill-existing-backend
```

---

For questions about the integration tests, please refer to the [main documentation](../docs/) or open an issue in the project repository.