//! IPC Client for Backend Communication
//!
//! This module provides the client-side implementation for communicating
//! with the ZipLock backend daemon via Unix domain sockets.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};
use ziplock_shared::models::CredentialRecord;

/// IPC request wrapper that matches backend expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub request_id: String,
    pub session_id: Option<String>,
    pub request: Request,
}

/// Available request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    // Authentication and session management
    Ping {
        client_info: Option<String>,
    },
    CreateSession,
    GetStatus,

    // Archive management
    CreateArchive {
        archive_path: PathBuf,
        master_password: String,
    },
    UnlockDatabase {
        archive_path: PathBuf,
        master_password: String,
    },
    LockDatabase,

    // Credential operations
    ListCredentials {
        include_sensitive: bool,
    },
    GetCredential {
        credential_id: String,
    },
    CreateCredential {
        credential: CredentialRecord,
    },
    UpdateCredential {
        credential_id: String,
        credential: CredentialRecord,
    },
    DeleteCredential {
        credential_id: String,
    },
    SearchCredentials {
        query: String,
        include_fields: bool,
        include_tags: bool,
        include_notes: bool,
    },

    // Utility operations
    SaveArchive,
    GetArchiveInfo,
}

/// Request result (success or error)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestResult {
    Success(ResponseData),
    Error {
        error_type: String,
        message: String,
        details: Option<String>,
    },
}

/// Response message from backend to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Request ID this response corresponds to
    pub request_id: String,
    /// Success or error result
    pub result: RequestResult,
}

/// Response data for successful requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    // Simple responses
    Pong {
        server_version: String,
        uptime_seconds: u64,
    },
    SessionCreated {
        session_id: String,
    },
    DatabaseUnlocked {
        credential_count: usize,
    },
    DatabaseLocked,
    ArchiveCreated,
    ArchiveSaved,

    // Status information
    Status {
        is_locked: bool,
        archive_path: Option<PathBuf>,
        credential_count: Option<usize>,
        last_activity: Option<u64>,
    },

    // Credential data
    CredentialList {
        credentials: Vec<CredentialRecord>,
    },
    Credential {
        credential: CredentialRecord,
    },
    CredentialCreated {
        credential_id: String,
    },
    CredentialUpdated,
    CredentialDeleted,
    SearchResults {
        credentials: Vec<CredentialRecord>,
        total_matches: usize,
    },

    // Archive information
    ArchiveInfo {
        path: PathBuf,
        credential_count: usize,
        created_at: std::time::SystemTime,
        last_modified: std::time::SystemTime,
    },
}

/// Simplified credential summary for listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSummary {
    pub id: String,
    pub title: String,
    pub credential_type: String,
    pub tags: Vec<String>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

/// Full credential data with all fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialData {
    pub id: String,
    pub title: String,
    pub credential_type: String,
    pub fields: Vec<(String, ziplock_shared::models::CredentialField)>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

/// Backend status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub version: String,
    pub uptime: u64,
    pub is_locked: bool,
    pub archive_path: Option<PathBuf>,
    pub credential_count: Option<usize>,
}

/// IPC client for communicating with the backend
#[derive(Debug)]
pub struct IpcClient {
    socket_path: PathBuf,
    stream: Option<UnixStream>,
    session_id: Option<String>,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            stream: None,
            session_id: None,
        }
    }

    /// Set the session ID for this client
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Connect to the backend daemon
    pub async fn connect(&mut self) -> Result<()> {
        debug!("Connecting to backend at {:?}", self.socket_path);

        let stream = UnixStream::connect(&self.socket_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to backend socket: {:?}",
                    self.socket_path
                )
            })?;

        self.stream = Some(stream);
        info!("Connected to backend daemon");
        Ok(())
    }

    /// Disconnect from the backend
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
            debug!("Disconnected from backend");
        }
        self.session_id = None;
        Ok(())
    }

    /// Check if connected to backend
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Send a request and receive response
    pub async fn send_request(&mut self, request: Request) -> Result<IpcResponse> {
        self.send_request_with_session(request, self.session_id.clone())
            .await
    }

    /// Send a request with specific session ID (or None for no session)
    async fn send_request_with_session(
        &mut self,
        request: Request,
        session_id: Option<String>,
    ) -> Result<IpcResponse> {
        if self.stream.is_none() {
            self.connect().await?;
        }

        let stream = self.stream.as_mut().unwrap();

        // Wrap request in proper IPC format
        let ipc_request = IpcRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            session_id,
            request,
        };

        // Serialize request
        let request_json =
            serde_json::to_string(&ipc_request).context("Failed to serialize request")?;

        debug!("Sending request: {}", request_json);
        info!("Sending IPC request to backend: {}", request_json);

        // Send request with newline delimiter
        stream
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send request")?;
        stream
            .write_all(b"\n")
            .await
            .context("Failed to send newline delimiter")?;
        stream.flush().await.context("Failed to flush request")?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .context("Failed to read response")?;

        debug!("Received response: {}", response_line.trim());
        info!(
            "Received IPC response from backend: {}",
            response_line.trim()
        );

        // Parse response
        let response: IpcResponse =
            serde_json::from_str(response_line.trim()).context("Failed to parse response")?;

        match &response.result {
            RequestResult::Error {
                error_type,
                message,
                ..
            } => {
                warn!("Backend returned error: {} - {}", error_type, message);
            }
            RequestResult::Success(_) => {
                debug!("Request processed successfully");
            }
        }

        Ok(response)
    }

    /// Create a new archive
    pub async fn create_archive(
        &mut self,
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<()> {
        // Create session if we don't have one
        if self.session_id.is_none() {
            self.create_session().await?;
        }

        let request = Request::CreateArchive {
            archive_path,
            master_password,
        };

        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::ArchiveCreated) => {
                info!("Archive created successfully");
                Ok(())
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!(
                "Unexpected response from backend. Please try again."
            )),
        }
    }

    /// Open an existing archive
    pub async fn open_archive(
        &mut self,
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<()> {
        // Create session if we don't have one
        if self.session_id.is_none() {
            self.create_session().await?;
        }

        let request = Request::UnlockDatabase {
            archive_path,
            master_password,
        };

        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::DatabaseUnlocked { .. }) => {
                info!("Archive opened successfully");
                Ok(())
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!(
                "Unexpected response from backend. Please try again."
            )),
        }
    }

    /// Create a session with the backend
    pub async fn create_session(&mut self) -> Result<()> {
        let request = Request::CreateSession;
        let response = self.send_request_with_session(request, None).await?;

        match response.result {
            RequestResult::Success(ResponseData::SessionCreated { session_id }) => {
                info!("Session created with ID: {}", session_id);
                self.session_id = Some(session_id);
                Ok(())
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "Failed to create session: {}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!(
                "Unexpected response when creating session. Please try again."
            )),
        }
    }

    /// Close the current archive
    pub async fn close_archive(&mut self) -> Result<()> {
        let request = Request::LockDatabase;
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::DatabaseLocked) => {
                info!("Archive closed successfully");
                Ok(())
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!(
                "Unexpected response from backend. Please try again."
            )),
        }
    }

    /// List all credentials in the archive
    pub async fn list_credentials(&mut self) -> Result<Vec<CredentialSummary>> {
        let request = Request::ListCredentials {
            include_sensitive: false,
        };
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::CredentialList { credentials }) => {
                debug!("Retrieved {} credentials", credentials.len());
                // Convert to CredentialSummary format expected by frontend
                let summaries: Vec<CredentialSummary> = credentials
                    .into_iter()
                    .map(|cred| CredentialSummary {
                        id: cred.id,
                        title: cred.title,
                        credential_type: cred.credential_type,
                        tags: cred.tags,
                        created_at: cred.created_at,
                        updated_at: cred.updated_at,
                    })
                    .collect();
                Ok(summaries)
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!("Unexpected response format")),
        }
    }

    /// Get a specific credential by ID
    pub async fn get_credential(&mut self, id: String) -> Result<CredentialData> {
        let request = Request::GetCredential {
            credential_id: id.clone(),
        };
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::Credential { credential }) => {
                debug!("Retrieved credential: {}", credential.title);
                // Convert CredentialRecord to CredentialData
                let credential_data = CredentialData {
                    id: credential.id,
                    title: credential.title,
                    credential_type: credential.credential_type,
                    fields: credential.fields.into_iter().collect(),
                    tags: credential.tags,
                    notes: credential.notes,
                    created_at: credential.created_at,
                    updated_at: credential.updated_at,
                };
                Ok(credential_data)
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!("Unexpected response format")),
        }
    }

    /// Create a new credential
    pub async fn create_credential(&mut self, credential: CredentialData) -> Result<()> {
        // Convert CredentialData to CredentialRecord
        let credential_record = ziplock_shared::models::CredentialRecord {
            id: credential.id,
            title: credential.title,
            credential_type: credential.credential_type,
            fields: credential.fields.into_iter().collect(),
            tags: credential.tags,
            notes: credential.notes,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        };

        let request = Request::CreateCredential {
            credential: credential_record,
        };
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::CredentialCreated { .. }) => {
                info!("Credential created successfully");
                Ok(())
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!(
                "Unexpected response from backend. Please try again."
            )),
        }
    }

    /// Search credentials
    pub async fn search_credentials(&mut self, query: String) -> Result<Vec<CredentialSummary>> {
        let request = Request::SearchCredentials {
            query,
            include_fields: true,
            include_tags: true,
            include_notes: true,
        };
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::SearchResults { credentials, .. }) => {
                debug!("Search returned {} results", credentials.len());
                // Convert to CredentialSummary format
                let summaries: Vec<CredentialSummary> = credentials
                    .into_iter()
                    .map(|cred| CredentialSummary {
                        id: cred.id,
                        title: cred.title,
                        credential_type: cred.credential_type,
                        tags: cred.tags,
                        created_at: cred.created_at,
                        updated_at: cred.updated_at,
                    })
                    .collect();
                Ok(summaries)
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!("Unexpected response format")),
        }
    }

    /// Get backend status
    pub async fn get_status(&mut self) -> Result<BackendStatus> {
        let request = Request::GetStatus;
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::Status {
                is_locked,
                archive_path,
                credential_count,
                last_activity,
            }) => {
                let status = BackendStatus {
                    version: "0.1.0".to_string(), // TODO: Get from backend
                    uptime: last_activity.unwrap_or(0),
                    is_locked,
                    archive_path,
                    credential_count,
                };
                debug!(
                    "Backend status: version {}, uptime {}s",
                    status.version, status.uptime
                );
                Ok(status)
            }
            RequestResult::Error { message, .. } => Err(anyhow::anyhow!(
                "{}",
                Self::convert_backend_error_to_user_message(&message)
            )),
            _ => Err(anyhow::anyhow!("Unexpected response format")),
        }
    }

    /// Ping the backend to check connectivity
    pub async fn ping(&mut self) -> Result<()> {
        let request = Request::Ping { client_info: None };
        let response = self.send_request(request).await?;

        match response.result {
            RequestResult::Success(ResponseData::Pong { .. }) => {
                debug!("Backend ping successful");
                Ok(())
            }
            RequestResult::Error { message, .. } => {
                Err(anyhow::anyhow!("Backend connection failed: {}", Self::convert_backend_error_to_user_message(&message)))
            }
            _ => Err(anyhow::anyhow!("Unable to communicate with backend. Please check that the ZipLock daemon is running.")),
        }
    }

    /// Get the current session ID
    pub fn get_session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    /// Convert backend error messages to user-friendly messages
    fn convert_backend_error_to_user_message(backend_message: &str) -> String {
        // Common error patterns and their user-friendly alternatives
        if backend_message.contains("Failed to bind to socket") {
            "Unable to start the backend service. Please check if another instance is running."
                .to_string()
        } else if backend_message.contains("Connection lost")
            || backend_message.contains("ConnectionLost")
        {
            "Lost connection to the backend service. Please restart the application.".to_string()
        } else if backend_message.contains("Session expired")
            || backend_message.contains("SessionExpired")
        {
            "Your session has expired. Please unlock the database again.".to_string()
        } else if backend_message.contains("Authentication timeout")
            || backend_message.contains("AuthTimeout")
        {
            "Authentication session timed out. Please unlock the database again.".to_string()
        } else if backend_message.contains("Authentication failed")
            || backend_message.contains("Invalid passphrase")
        {
            "Incorrect passphrase. Please check your password and try again.".to_string()
        } else if backend_message.contains("Archive not found")
            || backend_message.contains("File not found")
        {
            "The password archive file could not be found. Please check the file path.".to_string()
        } else if backend_message.contains("Permission denied") {
            "Permission denied. Please check file permissions or run with appropriate privileges."
                .to_string()
        } else if backend_message.contains("Invalid archive format")
            || backend_message.contains("Corrupt")
        {
            "The archive file appears to be corrupted or in an invalid format.".to_string()
        } else if backend_message.contains("Network") || backend_message.contains("IPC error") {
            "Communication error with the backend service. Please restart the application."
                .to_string()
        } else if backend_message.contains("Timeout") {
            "Operation timed out. The backend service may be overloaded.".to_string()
        } else if backend_message.contains("Database locked") {
            "The password database is currently locked by another process.".to_string()
        } else if backend_message.contains("Validation") {
            "The provided data failed validation. Please check your input and try again."
                .to_string()
        } else {
            // For unknown errors, provide a generic message but include some context
            format!("An error occurred: {}", backend_message)
        }
    }

    /// Check if an error indicates session timeout/expiry
    pub fn is_session_timeout_error(error_message: &str) -> bool {
        error_message.contains("Session expired")
            || error_message.contains("SessionExpired")
            || error_message.contains("Authentication timeout")
            || error_message.contains("AuthTimeout")
    }

    /// Check if backend is available and responsive
    pub async fn check_backend_availability(&mut self) -> bool {
        match self.ping().await {
            Ok(()) => true,
            Err(e) => {
                warn!("Backend not available: {}", e);
                // Try to reconnect
                let _ = self.disconnect().await;
                false
            }
        }
    }

    /// Get the default socket path for the backend
    pub fn default_socket_path() -> PathBuf {
        dirs::runtime_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("ziplock")
            .join("backend.sock")
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        if let Some(_stream) = self.stream.take() {
            debug!("IPC client dropped, connection closed");
        }
    }
}

/// Helper functions for common IPC operations
pub mod helpers {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    /// Create an IPC client with default settings and connect
    pub async fn create_client() -> Result<IpcClient> {
        let socket_path = IpcClient::default_socket_path();
        let mut client = IpcClient::new(socket_path);
        client.connect().await?;
        Ok(client)
    }

    /// Try to create a client with timeout
    pub async fn create_client_with_timeout(timeout_secs: u64) -> Result<IpcClient> {
        timeout(Duration::from_secs(timeout_secs), create_client())
            .await
            .context("Timeout connecting to backend")?
    }

    /// Check if backend is running
    pub async fn is_backend_running() -> bool {
        match create_client_with_timeout(2).await {
            Ok(mut client) => client.check_backend_availability().await,
            Err(_) => false,
        }
    }

    /// Start backend if not running (placeholder for future implementation)
    pub async fn ensure_backend_running() -> Result<()> {
        if !is_backend_running().await {
            // Here we would implement backend auto-start
            // For now, just return an error
            return Err(anyhow::anyhow!(
                "Backend is not running. Please start the ZipLock backend daemon first."
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_request_serialization() {
        let request = Request::CreateArchive {
            archive_path: PathBuf::from("/test/archive.7z"),
            master_password: "test_password".to_string(),
        };

        let ipc_request = IpcRequest {
            request_id: "test-123".to_string(),
            session_id: None,
            request,
        };

        let serialized = serde_json::to_string(&ipc_request).unwrap();
        let deserialized: IpcRequest = serde_json::from_str(&serialized).unwrap();

        match deserialized.request {
            Request::CreateArchive {
                archive_path,
                master_password,
            } => {
                assert_eq!(archive_path, PathBuf::from("/test/archive.7z"));
                assert_eq!(master_password, "test_password");
                assert_eq!(deserialized.request_id, "test-123");
            }
            _ => panic!("Wrong request type after deserialization"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = IpcResponse {
            request_id: "test-456".to_string(),
            result: RequestResult::Success(ResponseData::ArchiveCreated),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: IpcResponse = serde_json::from_str(&serialized).unwrap();

        match deserialized.result {
            RequestResult::Success(ResponseData::ArchiveCreated) => {
                assert_eq!(deserialized.request_id, "test-456");
            }
            _ => panic!("Wrong response type after deserialization"),
        }
    }

    #[test]
    fn test_default_socket_path() {
        let path = IpcClient::default_socket_path();
        assert!(path.to_string_lossy().contains("ziplock"));
        assert!(path.to_string_lossy().ends_with("backend.sock"));
    }

    #[test]
    fn test_create_archive_json_output() {
        let request = Request::CreateArchive {
            archive_path: PathBuf::from("/home/user/test.7z"),
            master_password: "test_password".to_string(),
        };

        let ipc_request = IpcRequest {
            request_id: "test-create-archive".to_string(),
            session_id: None,
            request,
        };

        let json = serde_json::to_string_pretty(&ipc_request).unwrap();
        println!("Frontend will send JSON:");
        println!("{}", json);

        // Also test deserialization to make sure it round-trips
        let parsed: IpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.request_id, "test-create-archive");
        match parsed.request {
            Request::CreateArchive {
                archive_path,
                master_password,
            } => {
                assert_eq!(archive_path, PathBuf::from("/home/user/test.7z"));
                assert_eq!(master_password, "test_password");
            }
            _ => panic!("Wrong request type"),
        }
    }
}
