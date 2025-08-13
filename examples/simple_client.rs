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

// Re-define the IPC types here for the example (normally these would be in a shared crate)
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

/// Simple client for communicating with ZipLock backend
struct ZipLockClient {
    stream: UnixStream,
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
    session_id: Option<String>,
}

impl ZipLockClient {
    /// Connect to the backend daemon
    async fn connect<P: AsRef<std::path::Path>>(socket_path: P) -> Result<Self> {
        println!("Connecting to ZipLock backend at {:?}", socket_path.as_ref());

        let stream = UnixStream::connect(socket_path).await
            .context("Failed to connect to backend socket")?;

        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);

        Ok(Self {
            stream: UnixStream::connect(socket_path.as_ref()).await?, // Dummy for type
            reader,
            writer: write_half,
            session_id: None,
        })
    }

    /// Send a request and receive response
    async fn send_request(&mut self, request: Request) -> Result<RequestResult> {
        let request_id = Uuid::new_v4().to_string();
        let ipc_request = IpcRequest {
            request_id: request_id.clone(),
            session_id: self.session_id.clone(),
            request,
        };

        // Serialize and send request
        let request_json = serde_json::to_string(&ipc_request)
            .context("Failed to serialize request")?;

        self.writer.write_all(request_json.as_bytes()).await
            .context("Failed to write request")?;
        self.writer.write_all(b"\n").await
            .context("Failed to write newline")?;
        self.writer.flush().await
            .context("Failed to flush request")?;

        // Read response
        let mut response_line = String::new();
        self.reader.read_line(&mut response_line).await
            .context("Failed to read response")?;

        let response: IpcResponse = serde_json::from_str(&response_line)
            .context("Failed to deserialize response")?;

        // Verify request ID matches
        if response.request_id != request_id {
            return Err(anyhow::anyhow!("Response ID mismatch"));
        }

        Ok(response.result)
    }

    /// Ping the server
    async fn ping(&mut self) -> Result<()> {
        println!("Pinging server...");

        let result = self.send_request(Request::Ping {
            client_info: Some("ZipLock Example Client".to_string()),
        }).await?;

        match result {
            RequestResult::Success(ResponseData::Pong { server_version, uptime_seconds }) => {
                println!("✓ Server responded! Version: {}, Uptime: {}s", server_version, uptime_seconds);
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Ping failed: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to ping");
            }
        }

        Ok(())
    }

    /// Create a new session
    async fn create_session(&mut self) -> Result<()> {
        println!("Creating session...");

        let result = self.send_request(Request::CreateSession).await?;

        match result {
            RequestResult::Success(ResponseData::SessionCreated { session_id }) => {
                self.session_id = Some(session_id.clone());
                println!("✓ Session created: {}", session_id);
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to create session: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to create session");
            }
        }

        Ok(())
    }

    /// Create a new archive
    async fn create_archive(&mut self, path: PathBuf, password: String) -> Result<()> {
        println!("Creating archive at {:?}...", path);

        let result = self.send_request(Request::CreateArchive {
            archive_path: path.clone(),
            master_password: password,
        }).await?;

        match result {
            RequestResult::Success(ResponseData::ArchiveCreated) => {
                println!("✓ Archive created successfully");
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to create archive: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to create archive");
            }
        }

        Ok(())
    }

    /// Unlock database
    async fn unlock_database(&mut self, path: PathBuf, password: String) -> Result<()> {
        println!("Unlocking database at {:?}...", path);

        let result = self.send_request(Request::UnlockDatabase {
            archive_path: path,
            master_password: password,
        }).await?;

        match result {
            RequestResult::Success(ResponseData::DatabaseUnlocked { credential_count }) => {
                println!("✓ Database unlocked! Found {} credentials", credential_count);
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to unlock database: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to unlock database");
            }
        }

        Ok(())
    }

    /// Create a sample credential
    async fn create_sample_credential(&mut self) -> Result<String> {
        println!("Creating sample credential...");

        let mut credential = CredentialRecord::new(
            "Example Website".to_string(),
            "login".to_string(),
        );

        credential.set_field("username", CredentialField::username("john.doe@example.com"));
        credential.set_field("password", CredentialField::password("super_secure_password_123!"));
        credential.set_field("website", CredentialField::url("https://example.com"));
        credential.add_tag("example");
        credential.add_tag("demo");
        credential.notes = Some("This is a sample credential created by the example client".to_string());

        let result = self.send_request(Request::CreateCredential { credential }).await?;

        match result {
            RequestResult::Success(ResponseData::CredentialCreated { credential_id }) => {
                println!("✓ Credential created with ID: {}", credential_id);
                Ok(credential_id)
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to create credential: {}", message);
                Err(anyhow::anyhow!("Failed to create credential: {}", message))
            }
            _ => {
                println!("✗ Unexpected response to create credential");
                Err(anyhow::anyhow!("Unexpected response"))
            }
        }
    }

    /// List all credentials
    async fn list_credentials(&mut self, include_sensitive: bool) -> Result<()> {
        println!("Listing credentials (include_sensitive: {})...", include_sensitive);

        let result = self.send_request(Request::ListCredentials { include_sensitive }).await?;

        match result {
            RequestResult::Success(ResponseData::CredentialList { credentials }) => {
                println!("✓ Found {} credentials:", credentials.len());
                for (i, cred) in credentials.iter().enumerate() {
                    println!("  {}. {} ({})", i + 1, cred.title, cred.credential_type);
                    println!("     ID: {}", cred.id);
                    println!("     Fields: {}", cred.fields.len());
                    println!("     Tags: {:?}", cred.tags);
                    if let Some(notes) = &cred.notes {
                        let preview = if notes.len() > 50 {
                            format!("{}...", &notes[..50])
                        } else {
                            notes.clone()
                        };
                        println!("     Notes: {}", preview);
                    }
                    println!();
                }
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to list credentials: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to list credentials");
            }
        }

        Ok(())
    }

    /// Get a specific credential
    async fn get_credential(&mut self, credential_id: &str) -> Result<()> {
        println!("Getting credential {}...", credential_id);

        let result = self.send_request(Request::GetCredential {
            credential_id: credential_id.to_string(),
        }).await?;

        match result {
            RequestResult::Success(ResponseData::Credential { credential }) => {
                println!("✓ Credential details:");
                println!("  Title: {}", credential.title);
                println!("  Type: {}", credential.credential_type);
                println!("  ID: {}", credential.id);
                println!("  Fields:");
                for (name, field) in &credential.fields {
                    let value_display = if field.sensitive {
                        "[HIDDEN]".to_string()
                    } else {
                        field.value.clone()
                    };
                    println!("    {}: {} ({})", name, value_display, field.field_type.display_name());
                }
                println!("  Tags: {:?}", credential.tags);
                if let Some(notes) = &credential.notes {
                    println!("  Notes: {}", notes);
                }
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to get credential: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to get credential");
            }
        }

        Ok(())
    }

    /// Search credentials
    async fn search_credentials(&mut self, query: &str) -> Result<()> {
        println!("Searching for '{}'...", query);

        let result = self.send_request(Request::SearchCredentials {
            query: query.to_string(),
            include_fields: true,
            include_tags: true,
            include_notes: true,
        }).await?;

        match result {
            RequestResult::Success(ResponseData::SearchResults { credentials, total_matches }) => {
                println!("✓ Found {} matches:", total_matches);
                for (i, cred) in credentials.iter().enumerate() {
                    println!("  {}. {} ({})", i + 1, cred.title, cred.credential_type);
                    if !cred.tags.is_empty() {
                        println!("     Tags: {:?}", cred.tags);
                    }
                }
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Search failed: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to search");
            }
        }

        Ok(())
    }

    /// Save the archive
    async fn save_archive(&mut self) -> Result<()> {
        println!("Saving archive...");

        let result = self.send_request(Request::SaveArchive).await?;

        match result {
            RequestResult::Success(ResponseData::ArchiveSaved) => {
                println!("✓ Archive saved successfully");
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to save archive: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to save archive");
            }
        }

        Ok(())
    }

    /// Lock the database
    async fn lock_database(&mut self) -> Result<()> {
        println!("Locking database...");

        let result = self.send_request(Request::LockDatabase).await?;

        match result {
            RequestResult::Success(ResponseData::DatabaseLocked) => {
                println!("✓ Database locked successfully");
            }
            RequestResult::Error { message, .. } => {
                println!("✗ Failed to lock database: {}", message);
            }
            _ => {
                println!("✗ Unexpected response to lock database");
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 ZipLock Backend Client Example");
    println!("==================================\n");

    // Determine socket path (matches backend default)
    let socket_path = dirs::runtime_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ziplock")
        .join("backend.sock");

    // Connect to backend
    let mut client = ZipLockClient::connect(&socket_path).await
        .context("Failed to connect to backend. Make sure the daemon is running.")?;

    println!("Connected to backend at {:?}\n", socket_path);

    // Run example workflow
    if let Err(e) = run_example_workflow(&mut client).await {
        println!("Example workflow failed: {}", e);
        return Err(e);
    }

    println!("\n✅ Example completed successfully!");
    Ok(())
}

async fn run_example_workflow(client: &mut ZipLockClient) -> Result<()> {
    // 1. Ping the server
    client.ping().await?;
    println!();

    // 2. Create a session
    client.create_session().await?;
    println!();

    // 3. Create a test archive
    let archive_path = std::env::temp_dir().join("ziplock_example.7z");
    let master_password = "example_password_123!".to_string();

    // Remove existing test archive
    if archive_path.exists() {
        std::fs::remove_file(&archive_path)?;
        println!("Removed existing test archive");
    }

    client.create_archive(archive_path.clone(), master_password.clone()).await?;
    println!();

    // 4. Unlock the database
    client.unlock_database(archive_path.clone(), master_password).await?;
    println!();

    // 5. Create a sample credential
    let credential_id = client.create_sample_credential().await?;
    println!();

    // 6. List all credentials
    client.list_credentials(false).await?; // Without sensitive data
    println!();

    // 7. Get the specific credential we just created
    client.get_credential(&credential_id).await?;
    println!();

    // 8. Search for credentials
    client.search_credentials("example").await?;
    println!();

    // 9. Save the archive
    client.save_archive().await?;
    println!();

    // 10. Lock the database
    client.lock_database().await?;
    println!();

    // Clean up test archive
    if archive_path.exists() {
        std::fs::remove_file(&archive_path)?;
        println!("Cleaned up test archive");
    }

    Ok(())
}
