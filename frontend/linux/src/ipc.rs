//! IPC Client for Backend Communication
//!
//! This module provides the client-side implementation for communicating
//! with the ZipLock backend daemon via Unix domain sockets.
//!
//! This is a simplified mock implementation for frontend development.
//! In a real application, this would involve actual IPC communication.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::debug;
use uuid::Uuid;
use ziplock_shared::models::{CredentialField, CredentialRecord, CredentialTemplate};
use ziplock_shared::utils;

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
    UnlockDatabase {
        archive_path: PathBuf,
        master_password: String,
    },
    LockDatabase,
    GetStatus,

    // Archive management
    CreateArchive {
        archive_path: PathBuf,
        master_password: String,
    },
    ValidateRepository {
        archive_path: PathBuf,
    },
    ValidateArchiveComprehensive {
        archive_path: PathBuf,
        master_password: String,
    },
    RepairArchive {
        archive_path: PathBuf,
        master_password: String,
    },

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

/// IPC response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub request_id: String,
    pub result: RequestResult,
}

/// Available response result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestResult {
    Success(ResponseData),
    Error {
        error_type: String,
        message: String,
        details: Option<String>,
    },
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
        created_at: SystemTime,
        last_modified: SystemTime,
    },

    // Repository validation
    RepositoryValidated {
        path: PathBuf,
        size: u64,
        last_modified: SystemTime,
        is_valid_format: bool,
        display_name: String,
    },

    // Comprehensive validation report
    ValidationReport {
        // Note: In real implementation, this would include the full validation report
        // For now, we'll use basic fields that the frontend can display
        is_valid: bool,
        issues_count: usize,
        can_auto_repair: bool,
        credential_count: usize,
        custom_type_count: usize,
        total_size_bytes: u64,
    },

    // Archive repair completed
    ArchiveRepaired {
        is_valid: bool,
        remaining_issues: usize,
        credential_count: usize,
        custom_type_count: usize,
        total_size_bytes: u64,
    },
}

/// Main IPC client struct
pub struct IpcClient {
    socket_path: PathBuf,
    reader: Option<BufReader<tokio::io::ReadHalf<UnixStream>>>,
    writer: Option<tokio::io::WriteHalf<UnixStream>>,
    session_id: Option<String>,
}

impl IpcClient {
    /// Get the default socket path for the backend
    pub fn default_socket_path() -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            // Use the shared utility to ensure consistency with backend
            utils::SocketUtils::default_socket_path()
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Fallback for other OS or for testing
            PathBuf::from("./ziplock.sock")
        }
    }

    /// Create a new IPC client
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            socket_path: Self::default_socket_path(),
            reader: None,
            writer: None,
            session_id: None,
        })
    }

    /// Set the session ID for subsequent requests
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Connect to the backend daemon
    pub async fn connect(&mut self) -> Result<(), String> {
        debug!("Connecting to backend at {:?}", self.socket_path); // Removed session_id for simplicity as it's optional
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("Failed to connect to backend: {}", e))?;
        let (read_half, write_half) = tokio::io::split(stream);
        self.reader = Some(BufReader::new(read_half));
        self.writer = Some(write_half);
        Ok(())
    }

    /// Send a request and receive response
    async fn send_request(&mut self, request: Request) -> Result<ResponseData, String> {
        let request_id = Uuid::new_v4().to_string();
        let ipc_request = IpcRequest {
            request_id: request_id.clone(),
            session_id: self.session_id.clone(),
            request,
        };

        if self.reader.is_none() || self.writer.is_none() {
            self.connect().await?;
        }

        let writer = self.writer.as_mut().ok_or("Writer not initialized")?;
        let reader = self.reader.as_mut().ok_or("Reader not initialized")?;

        let request_json = serde_json::to_string(&ipc_request)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        writer
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to send newline: {}", e))?;

        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let ipc_response: IpcResponse = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to deserialize response: {}", e))?;

        if ipc_response.request_id != request_id {
            return Err("Mismatched response ID".to_string());
        }

        match ipc_response.result {
            RequestResult::Error { message, .. } => Err(message),
            RequestResult::Success(response_data) => Ok(response_data),
        }
    }

    /// Get the current session ID
    pub fn get_session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    /// Create a new archive
    pub async fn create_archive(
        &mut self,
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<(), String> {
        let request = Request::CreateArchive {
            archive_path,
            master_password,
        };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::ArchiveCreated => Ok(()),
            _ => Err("Unexpected response for CreateArchive".to_string()),
        }
    }

    /// Unlock an existing database/archive
    pub async fn open_archive(
        &mut self,
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<(), String> {
        let request = Request::UnlockDatabase {
            archive_path,
            master_password,
        };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::DatabaseUnlocked { .. } => Ok(()),
            _ => Err("Unexpected response for UnlockDatabase".to_string()),
        }
    }

    /// Get all credential types/templates (not supported by backend)
    pub async fn get_credential_types(
        &mut self,
        _session_id: Option<String>,
    ) -> Result<Vec<CredentialTemplate>, String> {
        // This functionality is not currently supported by the backend
        Err("Credential types not supported".to_string())
    }

    /// Create a new credential
    pub async fn create_credential(
        &mut self,
        session_id: Option<String>,
        title: String,
        credential_type: String,
        fields: HashMap<String, CredentialField>,
        tags: Vec<String>,
        notes: Option<String>,
    ) -> Result<String, String> {
        self.session_id = session_id;

        // Create CredentialRecord from individual fields
        let mut credential = CredentialRecord::new(title, credential_type);
        credential.fields = fields;
        credential.tags = tags;
        credential.notes = notes;

        let request = Request::CreateCredential { credential };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::CredentialCreated { credential_id } => Ok(credential_id),
            _ => Err("Unexpected response for CreateCredential".to_string()),
        }
    }

    /// Get a specific credential by ID
    pub async fn get_credential(
        &mut self,
        session_id: Option<String>,
        credential_id: String,
    ) -> Result<CredentialRecord, String> {
        self.session_id = session_id;
        let request = Request::GetCredential { credential_id };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::Credential { credential } => Ok(credential),
            _ => Err("Unexpected response for GetCredential".to_string()),
        }
    }

    /// Update an existing credential
    pub async fn update_credential(
        &mut self,
        session_id: Option<String>,
        credential: CredentialRecord,
    ) -> Result<(), String> {
        self.session_id = session_id;
        // The credential already has its ID set

        let request = Request::UpdateCredential {
            credential_id: credential.id.clone(),
            credential,
        };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::CredentialUpdated => Ok(()),
            _ => Err("Unexpected response for UpdateCredential".to_string()),
        }
    }

    /// Ping the backend to check connectivity
    pub async fn ping(&mut self) -> Result<(), String> {
        let request = Request::Ping { client_info: None };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::Pong { .. } => Ok(()),
            _ => Err("Unexpected response for Ping".to_string()),
        }
    }

    /// Create a session with the backend
    pub async fn create_session(&mut self) -> Result<(), String> {
        let request = Request::CreateSession;
        let response = self.send_request(request).await?;
        match response {
            ResponseData::SessionCreated { session_id } => {
                self.session_id = Some(session_id);
                Ok(())
            }
            _ => Err("Unexpected response for CreateSession".to_string()),
        }
    }

    /// List all credentials in the archive
    pub async fn list_credentials(&mut self) -> Result<Vec<CredentialRecord>, String> {
        let request = Request::ListCredentials {
            include_sensitive: false,
        };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::CredentialList { credentials } => Ok(credentials),
            _ => Err("Unexpected response for ListCredentials".to_string()),
        }
    }

    /// Delete a credential by ID
    pub async fn delete_credential(
        &mut self,
        session_id: Option<String>,
        credential_id: String,
    ) -> Result<(), String> {
        self.session_id = session_id;
        let request = Request::DeleteCredential { credential_id };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::CredentialDeleted => Ok(()),
            _ => Err("Unexpected response for DeleteCredential".to_string()),
        }
    }

    /// Close the current archive
    pub async fn close_archive(&mut self) -> Result<(), String> {
        let request = Request::LockDatabase;
        let response = self.send_request(request).await?;
        match response {
            ResponseData::DatabaseLocked => Ok(()),
            _ => Err("Unexpected response for LockDatabase".to_string()),
        }
    }

    /// Validate repository format without requiring master password
    pub async fn validate_repository(&mut self, archive_path: PathBuf) -> Result<bool, String> {
        let request = Request::ValidateRepository { archive_path };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::RepositoryValidated {
                is_valid_format, ..
            } => Ok(is_valid_format),
            _ => Err("Unexpected response for ValidateRepository".to_string()),
        }
    }

    /// Perform comprehensive validation of an archive file
    pub async fn validate_archive_comprehensive(
        &mut self,
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<(bool, usize, bool), String> {
        let request = Request::ValidateArchiveComprehensive {
            archive_path,
            master_password,
        };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::ValidationReport {
                is_valid,
                issues_count,
                can_auto_repair,
                ..
            } => Ok((is_valid, issues_count, can_auto_repair)),
            _ => Err("Unexpected response for ValidateArchiveComprehensive".to_string()),
        }
    }

    /// Repair an archive file by fixing validation issues
    pub async fn repair_archive(
        &mut self,
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<(bool, usize), String> {
        let request = Request::RepairArchive {
            archive_path,
            master_password,
        };
        let response = self.send_request(request).await?;
        match response {
            ResponseData::ArchiveRepaired {
                is_valid,
                remaining_issues,
                ..
            } => Ok((is_valid, remaining_issues)),
            _ => Err("Unexpected response for RepairArchive".to_string()),
        }
    }

    /// Placeholder for checking session timeout error
    pub fn is_session_timeout_error(error_message: &str) -> bool {
        error_message.contains("Session expired") || error_message.contains("SessionExpired")
    }
}
