# IPC Client Examples

This guide provides practical examples for communicating with the ZipLock backend service through Inter-Process Communication (IPC). These examples demonstrate how to build client applications that interact with ZipLock's encrypted password storage.

## Overview

ZipLock uses a client-server architecture where the backend service handles all cryptographic operations and data management, while frontend clients provide the user interface. Communication occurs through Unix domain sockets using a JSON-based protocol.

## Communication Protocol

### Message Format

All IPC messages use JSON format with the following structure:

**Request Format:**
```json
{
  "request_id": "unique-request-id",
  "session_id": "optional-session-id",
  "request": {
    "type": "RequestType",
    "parameters": {}
  }
}
```

**Response Format:**
```json
{
  "request_id": "matching-request-id",
  "result": {
    "Success": {
      "data": {}
    }
  }
}
```

### Available Requests

- `Ping` - Test connectivity and get server info
- `CreateSession` - Establish a new client session
- `CreateArchive` - Create a new encrypted archive
- `UnlockDatabase` - Open and decrypt an existing archive
- `CreateCredential` - Add a new credential to the database
- `ListCredentials` - Retrieve all credentials (with optional filtering)
- `GetCredential` - Retrieve a specific credential by ID
- `SearchCredentials` - Search credentials by query
- `SaveArchive` - Save changes to the encrypted archive
- `LockDatabase` - Close and lock the database

## Complete Rust Client Example

Here's a comprehensive Rust client implementation that demonstrates all major operations:

```rust
//! Simple example client for ZipLock backend IPC communication
//!
//! This example demonstrates how to communicate with the ZipLock backend daemon
//! using Unix domain sockets. It shows the complete flow of operations from
//! creating a session to managing credentials.
//!
//! Usage:
//! ```bash
//! # Start the backend daemon first
//! cargo run --bin ziplock-backend
//!
//! # Then run this example
//! cargo run --example simple_client
//! ```

use anyhow::{Context, Result};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

use ziplock_shared::models::{CredentialField, CredentialRecord};

// IPC Protocol Definitions
#[derive(serde::Serialize, serde::Deserialize)]
struct IpcRequest {
    request_id: String,
    session_id: Option<String>,
    request: Request,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct IpcResponse {
    request_id: String,
    result: RequestResult,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum Request {
    Ping { client_info: Option<String> },
    CreateSession,
    CreateArchive { archive_path: PathBuf, master_password: String },
    UnlockDatabase { archive_path: PathBuf, master_password: String },
    CreateCredential { credential: CredentialRecord },
    ListCredentials { include_sensitive: bool },
    GetCredential { credential_id: String },
    SearchCredentials { query: String, include_fields: bool, include_tags: bool, include_notes: bool },
    SaveArchive,
    LockDatabase,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum RequestResult {
    Success(ResponseData),
    Error { error_type: String, message: String, details: Option<String> },
}

#[derive(serde::Serialize, serde::Deserialize)]
enum ResponseData {
    Pong { server_version: String, uptime_seconds: u64 },
    SessionCreated { session_id: String },
    ArchiveCreated,
    DatabaseUnlocked { credential_count: usize },
    CredentialCreated { credential_id: String },
    CredentialList { credentials: Vec<CredentialRecord> },
    Credential { credential: CredentialRecord },
    SearchResults { credentials: Vec<CredentialRecord>, total_matches: usize },
    ArchiveSaved,
    DatabaseLocked,
}

struct ZipLockClient {
    stream: UnixStream,
    reader: BufReader<tokio::io::ReadHalf<UnixStream>>,
    writer: tokio::io::WriteHalf<UnixStream>,
    session_id: Option<String>,
}

impl ZipLockClient {
    /// Connect to the ZipLock backend service
    async fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)
            .await
            .context("Failed to connect to ZipLock backend")?;

        let (read_half, write_half) = tokio::io::split(stream);
        let reader = BufReader::new(read_half);
        
        // Clone stream for the struct
        let stream = UnixStream::connect(socket_path).await?;
        
        Ok(Self {
            stream,
            reader,
            writer: write_half,
            session_id: None,
        })
    }

    /// Send a request and wait for response
    async fn send_request(&mut self, request: Request) -> Result<ResponseData> {
        let request_id = Uuid::new_v4().to_string();
        
        let ipc_request = IpcRequest {
            request_id: request_id.clone(),
            session_id: self.session_id.clone(),
            request,
        };

        // Send request
        let json = serde_json::to_string(&ipc_request)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;

        // Read response
        let mut response_line = String::new();
        self.reader.read_line(&mut response_line).await?;
        
        let response: IpcResponse = serde_json::from_str(&response_line)
            .context("Failed to parse response JSON")?;

        // Verify request ID matches
        if response.request_id != request_id {
            anyhow::bail!("Response request ID mismatch");
        }

        match response.result {
            RequestResult::Success(data) => Ok(data),
            RequestResult::Error { error_type, message, details } => {
                let error_msg = format!("{}: {}", error_type, message);
                let full_error = if let Some(details) = details {
                    format!("{} (Details: {})", error_msg, details)
                } else {
                    error_msg
                };
                anyhow::bail!(full_error);
            }
        }
    }

    /// Test connectivity with the backend
    async fn ping(&mut self) -> Result<(String, u64)> {
        let response = self.send_request(Request::Ping {
            client_info: Some("ZipLock Example Client v1.0".to_string()),
        }).await?;

        match response {
            ResponseData::Pong { server_version, uptime_seconds } => {
                println!("✓ Connected to ZipLock backend v{} (uptime: {}s)", 
                         server_version, uptime_seconds);
                Ok((server_version, uptime_seconds))
            }
            _ => anyhow::bail!("Unexpected response to ping"),
        }
    }

    /// Create a new client session
    async fn create_session(&mut self) -> Result<String> {
        let response = self.send_request(Request::CreateSession).await?;

        match response {
            ResponseData::SessionCreated { session_id } => {
                println!("✓ Created session: {}", session_id);
                self.session_id = Some(session_id.clone());
                Ok(session_id)
            }
            _ => anyhow::bail!("Unexpected response to create session"),
        }
    }

    /// Create a new encrypted archive
    async fn create_archive(&mut self, path: PathBuf, password: String) -> Result<()> {
        let response = self.send_request(Request::CreateArchive {
            archive_path: path.clone(),
            master_password: password,
        }).await?;

        match response {
            ResponseData::ArchiveCreated => {
                println!("✓ Created archive: {}", path.display());
                Ok(())
            }
            _ => anyhow::bail!("Unexpected response to create archive"),
        }
    }

    /// Unlock an existing archive
    async fn unlock_database(&mut self, path: PathBuf, password: String) -> Result<usize> {
        let response = self.send_request(Request::UnlockDatabase {
            archive_path: path.clone(),
            master_password: password,
        }).await?;

        match response {
            ResponseData::DatabaseUnlocked { credential_count } => {
                println!("✓ Unlocked database: {} ({} credentials)", 
                         path.display(), credential_count);
                Ok(credential_count)
            }
            _ => anyhow::bail!("Unexpected response to unlock database"),
        }
    }

    /// Create a sample credential for testing
    async fn create_sample_credential(&mut self) -> Result<String> {
        let mut fields = HashMap::new();
        fields.insert("username".to_string(), CredentialField::new_text("user@example.com"));
        fields.insert("password".to_string(), CredentialField::new_password("SecurePassword123!"));
        fields.insert("website".to_string(), CredentialField::new_url("https://example.com"));

        let credential = CredentialRecord::new(
            "Example Service".to_string(),
            fields,
            vec!["example".to_string(), "test".to_string()],
            Some("Sample credential for testing".to_string()),
        );

        let response = self.send_request(Request::CreateCredential { credential }).await?;

        match response {
            ResponseData::CredentialCreated { credential_id } => {
                println!("✓ Created credential: {}", credential_id);
                Ok(credential_id)
            }
            _ => anyhow::bail!("Unexpected response to create credential"),
        }
    }

    /// List all credentials
    async fn list_credentials(&mut self, include_sensitive: bool) -> Result<Vec<CredentialRecord>> {
        let response = self.send_request(Request::ListCredentials { include_sensitive }).await?;

        match response {
            ResponseData::CredentialList { credentials } => {
                println!("✓ Retrieved {} credentials", credentials.len());
                for (i, cred) in credentials.iter().enumerate() {
                    println!("  {}: {} (ID: {})", i + 1, cred.name, cred.id);
                    if !cred.tags.is_empty() {
                        println!("     Tags: {}", cred.tags.join(", "));
                    }
                }
                Ok(credentials)
            }
            _ => anyhow::bail!("Unexpected response to list credentials"),
        }
    }

    /// Get a specific credential by ID
    async fn get_credential(&mut self, credential_id: String) -> Result<CredentialRecord> {
        let response = self.send_request(Request::GetCredential { credential_id: credential_id.clone() }).await?;

        match response {
            ResponseData::Credential { credential } => {
                println!("✓ Retrieved credential: {}", credential.name);
                println!("  ID: {}", credential.id);
                println!("  Fields:");
                for (name, field) in &credential.fields {
                    let value = if field.is_sensitive {
                        "[SENSITIVE]".to_string()
                    } else {
                        field.value.clone()
                    };
                    println!("    {}: {}", name, value);
                }
                if let Some(notes) = &credential.notes {
                    println!("  Notes: {}", notes);
                }
                Ok(credential)
            }
            _ => anyhow::bail!("Unexpected response to get credential"),
        }
    }

    /// Search credentials by query
    async fn search_credentials(&mut self, query: String) -> Result<Vec<CredentialRecord>> {
        let response = self.send_request(Request::SearchCredentials {
            query: query.clone(),
            include_fields: true,
            include_tags: true,
            include_notes: true,
        }).await?;

        match response {
            ResponseData::SearchResults { credentials, total_matches } => {
                println!("✓ Search for '{}' found {} matches", query, total_matches);
                for (i, cred) in credentials.iter().enumerate() {
                    println!("  {}: {}", i + 1, cred.name);
                }
                Ok(credentials)
            }
            _ => anyhow::bail!("Unexpected response to search credentials"),
        }
    }

    /// Save changes to the archive
    async fn save_archive(&mut self) -> Result<()> {
        let response = self.send_request(Request::SaveArchive).await?;

        match response {
            ResponseData::ArchiveSaved => {
                println!("✓ Archive saved successfully");
                Ok(())
            }
            _ => anyhow::bail!("Unexpected response to save archive"),
        }
    }

    /// Lock the database
    async fn lock_database(&mut self) -> Result<()> {
        let response = self.send_request(Request::LockDatabase).await?;

        match response {
            ResponseData::DatabaseLocked => {
                println!("✓ Database locked");
                Ok(())
            }
            _ => anyhow::bail!("Unexpected response to lock database"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 ZipLock IPC Client Example");
    println!("================================");

    // Connect to backend
    let mut client = ZipLockClient::connect("/tmp/ziplock/backend.sock").await?;

    // Run example workflow
    run_example_workflow(&mut client).await?;

    println!("\n✅ Example completed successfully!");
    Ok(())
}

async fn run_example_workflow(client: &mut ZipLockClient) -> Result<()> {
    // 1. Test connectivity
    println!("\n1. Testing connectivity...");
    client.ping().await?;

    // 2. Create session
    println!("\n2. Creating session...");
    client.create_session().await?;

    // 3. Create a test archive
    println!("\n3. Creating test archive...");
    let archive_path = PathBuf::from("/tmp/test_archive.7z");
    client.create_archive(archive_path.clone(), "TestPassword123!".to_string()).await?;

    // 4. Unlock the database
    println!("\n4. Unlocking database...");
    client.unlock_database(archive_path, "TestPassword123!".to_string()).await?;

    // 5. Create a sample credential
    println!("\n5. Creating sample credential...");
    let credential_id = client.create_sample_credential().await?;

    // 6. List all credentials
    println!("\n6. Listing all credentials...");
    client.list_credentials(false).await?;

    // 7. Get the specific credential
    println!("\n7. Getting specific credential...");
    client.get_credential(credential_id).await?;

    // 8. Search credentials
    println!("\n8. Searching credentials...");
    client.search_credentials("example".to_string()).await?;

    // 9. Save the archive
    println!("\n9. Saving archive...");
    client.save_archive().await?;

    // 10. Lock the database
    println!("\n10. Locking database...");
    client.lock_database().await?;

    Ok(())
}
```

## Error Handling

The IPC protocol includes comprehensive error handling. Errors are returned in the following format:

```json
{
  "request_id": "request-id",
  "result": {
    "Error": {
      "error_type": "ValidationError",
      "message": "Master password is too weak",
      "details": "Password must be at least 12 characters"
    }
  }
}
```

Common error types:
- `ValidationError`: Input validation failures
- `AuthenticationError`: Authentication or authorization failures
- `StorageError`: File system or archive-related errors
- `CryptographyError`: Encryption/decryption failures
- `SessionError`: Session management issues
- `InternalError`: Unexpected internal errors

## Building and Running

To use this example:

1. **Build the project**:
   ```bash
   cargo build --release
   ```

2. **Start the backend service**:
   ```bash
   ./target/release/ziplock-backend
   ```

3. **Run the example client**:
   ```bash
   cargo run --example simple_client
   ```

## Integration Patterns

### Connection Management

For production applications, implement connection pooling and retry logic:

```rust
use tokio::time::{timeout, Duration};

impl ZipLockClient {
    async fn connect_with_retry(socket_path: &str, max_retries: u32) -> Result<Self> {
        for attempt in 1..=max_retries {
            match timeout(Duration::from_secs(5), Self::connect(socket_path)).await {
                Ok(Ok(client)) => return Ok(client),
                Ok(Err(e)) => {
                    if attempt == max_retries {
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(_) => {
                    if attempt == max_retries {
                        anyhow::bail!("Connection timeout after {} attempts", max_retries);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        unreachable!()
    }
}
```

### Session Management

Sessions should be managed carefully to prevent resource leaks:

```rust
struct SessionManager {
    client: ZipLockClient,
    session_id: Option<String>,
}

impl SessionManager {
    async fn ensure_session(&mut self) -> Result<&str> {
        if self.session_id.is_none() {
            let session_id = self.client.create_session().await?;
            self.session_id = Some(session_id);
        }
        Ok(self.session_id.as_ref().unwrap())
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        // Clean up session when dropped
        if self.session_id.is_some() {
            // Note: In a real implementation, you'd want to properly close the session
            log::info!("Session manager dropped, session should be cleaned up");
        }
    }
}
```

### Async Best Practices

When integrating with async frameworks like Tokio:

```rust
use tokio::sync::{mpsc, oneshot};

// Channel-based request handling
struct RequestHandler {
    sender: mpsc::Sender<(Request, oneshot::Sender<Result<ResponseData>>)>,
}

impl RequestHandler {
    fn new(client: ZipLockClient) -> Self {
        let (sender, mut receiver) = mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut client = client;
            while let Some((request, response_sender)) = receiver.recv().await {
                let result = client.send_request(request).await;
                let _ = response_sender.send(result);
            }
        });

        Self { sender }
    }

    async fn send_request(&self, request: Request) -> Result<ResponseData> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.sender.send((request, response_sender)).await?;
        response_receiver.await?
    }
}
```

## Security Considerations

### Socket Permissions

Ensure the Unix socket has appropriate permissions:

```rust
use std::os::unix::fs::PermissionsExt;

// Set socket permissions to 0o600 (owner read/write only)
if let Err(e) = std::fs::set_permissions(&socket_path, 
    std::fs::Permissions::from_mode(0o600)) {
    log::warn!("Failed to set socket permissions: {}", e);
}
```

### Memory Management

When handling sensitive data:

```rust
use zeroize::Zeroize;

struct SecureString {
    data: String,
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}
```

## Testing

Create unit tests for your IPC client:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_client_connection() {
        // Start a test backend service
        // This would typically use a test harness
        let client = ZipLockClient::connect("/tmp/test_socket").await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let mut client = test_client().await;
        let (version, uptime) = client.ping().await.unwrap();
        assert!(!version.is_empty());
        assert!(uptime >= 0);
    }
}
```

## Related Documentation

- [Architecture Overview](../architecture.md) - System architecture and component relationships
- [Mobile Integration](mobile-integration.md) - Mobile platform integration examples
- [Configuration Guide](configuration.md) - Backend configuration options
- [Build Guide](build.md) - Building and packaging the client applications

## Troubleshooting

### Common Issues

**Connection refused**:
- Ensure the backend service is running
- Check that the socket path is correct
- Verify socket permissions

**JSON parsing errors**:
- Ensure proper message framing (newline-delimited)
- Check for proper JSON encoding
- Verify all required fields are present

**Session errors**:
- Create a session before making authenticated requests
- Handle session expiration gracefully
- Implement session renewal logic

### Debug Logging

Enable debug logging in your client:

```rust
env_logger::init();
log::debug!("Sending request: {:?}", request);
```

For detailed protocol debugging, enable request logging in the backend configuration:

```yaml
ipc:
  log_requests: true
```
