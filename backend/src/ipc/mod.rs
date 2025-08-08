//! IPC (Inter-Process Communication) module for ZipLock backend
//!
//! This module provides Unix domain socket communication between the frontend
//! clients and the backend daemon. It handles secure request/response patterns,
//! connection management, and proper error handling.
//!
//! The IPC protocol uses JSON for message serialization and supports:
//! - Database operations (unlock, lock, create)
//! - Credential CRUD operations
//! - Search and filtering
//! - Status queries

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{BackendResult, IpcError};
use crate::storage::ArchiveManager;
use ziplock_shared::models::CredentialRecord;
use ziplock_shared::validate_master_passphrase_strict;

/// IPC server that handles Unix socket communication
pub struct IpcServer {
    listener: UnixListener,
    archive_manager: Arc<ArchiveManager>,
    active_sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    max_connections: usize,
    config: crate::config::Config,
}

/// Represents an active client session
#[derive(Debug)]
struct ClientSession {
    session_id: String,
    authenticated: bool,
    last_activity: std::time::SystemTime,
    client_info: Option<String>,
}

/// Request message from frontend to backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    /// Unique request ID for correlation
    pub request_id: String,
    /// Session ID (if authenticated)
    pub session_id: Option<String>,
    /// The actual request
    pub request: Request,
}

/// Response message from backend to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Request ID this response corresponds to
    pub request_id: String,
    /// Success or error result
    pub result: RequestResult,
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

    // Repository validation
    RepositoryValidated {
        path: PathBuf,
        size: u64,
        last_modified: std::time::SystemTime,
        is_valid_format: bool,
        display_name: String,
    },
}

impl IpcServer {
    /// Create a new IPC server
    pub async fn new(
        socket_path: PathBuf,
        archive_manager: Arc<ArchiveManager>,
        config: crate::config::Config,
    ) -> BackendResult<Self> {
        // Remove existing socket file if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)
                .with_context(|| format!("Failed to remove existing socket: {:?}", socket_path))
                .map_err(|_e| IpcError::SocketBind {
                    path: socket_path.to_string_lossy().to_string(),
                })?;
        }

        // Create parent directory if needed
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create socket directory: {:?}", parent))
                .map_err(|_e| IpcError::SocketBind {
                    path: socket_path.to_string_lossy().to_string(),
                })?;
        }

        // Bind to the Unix socket
        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!("Failed to bind to socket: {:?}", socket_path))
            .map_err(|_e| IpcError::SocketBind {
                path: socket_path.to_string_lossy().to_string(),
            })?;

        // Set socket permissions (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&socket_path)
                .context("Failed to get socket metadata")?
                .permissions();
            perms.set_mode(0o600); // Owner read/write only
            std::fs::set_permissions(&socket_path, perms)
                .context("Failed to set socket permissions")
                .map_err(|e| IpcError::SocketPermissions {
                    reason: e.to_string(),
                })?;
        }

        info!("IPC server bound to socket: {:?}", socket_path);

        Ok(Self {
            listener,
            archive_manager,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            max_connections: 100, // TODO: Make this configurable
            config,
        })
    }

    /// Run the IPC server
    pub async fn run(self) -> Result<()> {
        info!("Starting IPC server");

        // Start session cleanup task
        let sessions_cleanup = Arc::clone(&self.active_sessions);
        tokio::spawn(async move {
            Self::session_cleanup_task(sessions_cleanup).await;
        });

        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    info!("IPC server accepted new connection");
                    let sessions = Arc::clone(&self.active_sessions);
                    let archive_manager = Arc::clone(&self.archive_manager);

                    // Check connection limit
                    let current_connections = sessions.read().await.len();
                    if current_connections >= self.max_connections {
                        warn!("Connection limit reached, rejecting new connection");
                        // Close the connection
                        drop(stream);
                        continue;
                    }

                    info!("Spawning handler for new connection");

                    // Handle client connection in a separate task
                    let config = self.config.clone();
                    tokio::spawn(async move {
                        info!("Starting client handler task");
                        if let Err(e) =
                            Self::handle_client(stream, sessions, archive_manager, config).await
                        {
                            error!("Client handler error: {}", e);
                        }
                        info!("Client handler task completed");
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    info!("Continuing to accept other connections after error");
                    // Continue accepting other connections
                }
            }
        }
    }

    /// Handle a client connection
    async fn handle_client(
        stream: UnixStream,
        sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
        archive_manager: Arc<ArchiveManager>,
        config: crate::config::Config,
    ) -> Result<()> {
        debug!("New client connected");
        info!("New client connected to backend");

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();

            // Read request from client
            match buf_reader.read_line(&mut line).await {
                Ok(0) => {
                    // Client disconnected
                    debug!("Client disconnected");
                    info!("Client disconnected from backend");
                    break;
                }
                Ok(bytes_read) => {
                    info!("Read {} bytes from client: '{}'", bytes_read, line.trim());
                    let line = line.trim();
                    if line.is_empty() {
                        info!("Received empty line, continuing...");
                        continue;
                    }

                    // Process the request
                    let response =
                        match Self::process_request(line, &sessions, &archive_manager, &config)
                            .await
                        {
                            Ok(response) => response,
                            Err(e) => {
                                error!("Request processing error: {}", e);

                                // Create error response
                                IpcResponse {
                                    request_id: "unknown".to_string(),
                                    result: RequestResult::Error {
                                        error_type: "ProcessingError".to_string(),
                                        message: "Failed to process request".to_string(),
                                        details: Some(e.to_string()),
                                    },
                                }
                            }
                        };

                    // Send response
                    let response_json =
                        serde_json::to_string(&response).context("Failed to serialize response")?;

                    writer
                        .write_all(response_json.as_bytes())
                        .await
                        .context("Failed to write response")?;
                    writer
                        .write_all(b"\n")
                        .await
                        .context("Failed to write newline")?;
                    writer.flush().await.context("Failed to flush response")?;
                }
                Err(e) => {
                    error!("Failed to read from client: {}", e);
                    break;
                }
            }
        }

        debug!("Client handler finished");
        Ok(())
    }

    /// Process a request from a client
    async fn process_request(
        request_line: &str,
        sessions: &Arc<RwLock<HashMap<String, ClientSession>>>,
        archive_manager: &Arc<ArchiveManager>,
        config: &crate::config::Config,
    ) -> Result<IpcResponse> {
        debug!("Received raw request: {}", request_line);
        info!("Backend received IPC request: {}", request_line);

        // Parse request with detailed error information
        let request: IpcRequest = match serde_json::from_str::<IpcRequest>(request_line) {
            Ok(req) => {
                info!("Successfully parsed IPC request: {:?}", req.request);
                req
            }
            Err(e) => {
                error!("JSON parsing failed!");
                error!("Error: {}", e);
                error!("Raw request line: '{}'", request_line);
                error!("Request line length: {} bytes", request_line.len());
                error!("Request line as bytes: {:?}", request_line.as_bytes());

                // Try to see if it's a partial JSON or malformed
                if request_line.starts_with('{') {
                    error!("Looks like JSON but parsing failed - possibly incomplete");
                } else {
                    error!("Does not start with '{{' - not valid JSON");
                }

                return Err(anyhow::anyhow!(
                    "Failed to parse request JSON. Error: {}. Request: '{}'",
                    e,
                    request_line
                ));
            }
        };

        debug!("Parsed request: {:?}", request);

        debug!("Processing request: {:?}", request.request);

        // Validate session for authenticated requests
        let session_validation = if let Some(session_id) = &request.session_id {
            let sessions_guard = sessions.read().await;
            if let Some(session) = sessions_guard.get(session_id) {
                // Check if session has expired (1 hour timeout)
                let now = std::time::SystemTime::now();
                let timeout = std::time::Duration::from_secs(3600); // 1 hour

                if now
                    .duration_since(session.last_activity)
                    .unwrap_or_default()
                    > timeout
                {
                    Some(Err("SessionExpired"))
                } else if session.authenticated {
                    Some(Ok(true))
                } else {
                    Some(Ok(false))
                }
            } else {
                Some(Err("SessionNotFound"))
            }
        } else {
            None
        };

        // Check for session validation errors before processing authenticated requests
        let is_authenticated = match session_validation {
            Some(Ok(authenticated)) => authenticated,
            Some(Err(error_type)) => {
                // Return session error immediately for authenticated operations
                if !matches!(
                    request.request,
                    Request::Ping { .. } | Request::CreateSession
                ) {
                    return Ok(IpcResponse {
                        request_id: request.request_id,
                        result: RequestResult::Error {
                            error_type: error_type.to_string(),
                            message: match error_type {
                                "SessionExpired" => {
                                    "Session expired - please re-authenticate".to_string()
                                }
                                "SessionNotFound" => {
                                    "Session not found - please create a new session".to_string()
                                }
                                _ => "Session validation failed".to_string(),
                            },
                            details: None,
                        },
                    });
                }
                false
            }
            None => false,
        };

        // Process the request
        let result = match request.request {
            Request::Ping { client_info } => Self::handle_ping(client_info).await,

            Request::CreateSession => Self::handle_create_session(sessions).await,

            Request::UnlockDatabase {
                archive_path,
                master_password,
            } => {
                if let Some(session_id) = &request.session_id {
                    Self::handle_unlock_database(
                        session_id,
                        archive_path,
                        master_password,
                        sessions,
                        archive_manager,
                    )
                    .await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "AuthenticationRequired".to_string(),
                        message: "Session required for database operations".to_string(),
                        details: None,
                    })
                }
            }

            Request::LockDatabase => {
                if is_authenticated {
                    Self::handle_lock_database(
                        request.session_id.as_ref().unwrap(),
                        sessions,
                        archive_manager,
                    )
                    .await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::GetStatus => Self::handle_get_status(archive_manager).await,

            Request::CreateArchive {
                archive_path,
                master_password,
            } => {
                Self::handle_create_archive(archive_path, master_password, archive_manager, config)
                    .await
            }

            Request::ValidateRepository { archive_path } => {
                Self::handle_validate_repository(archive_path, archive_manager).await
            }

            // Credential operations (require authentication)
            Request::ListCredentials { include_sensitive } => {
                if is_authenticated {
                    Self::handle_list_credentials(include_sensitive, archive_manager).await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::GetCredential { credential_id } => {
                if is_authenticated {
                    Self::handle_get_credential(&credential_id, archive_manager).await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::CreateCredential { credential } => {
                if is_authenticated {
                    Self::handle_create_credential(credential, archive_manager).await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::UpdateCredential {
                credential_id,
                credential,
            } => {
                if is_authenticated {
                    Self::handle_update_credential(&credential_id, credential, archive_manager)
                        .await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::DeleteCredential { credential_id } => {
                if is_authenticated {
                    Self::handle_delete_credential(&credential_id, archive_manager).await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::SearchCredentials {
                query,
                include_fields,
                include_tags,
                include_notes,
            } => {
                if is_authenticated {
                    Self::handle_search_credentials(
                        &query,
                        include_fields,
                        include_tags,
                        include_notes,
                        archive_manager,
                    )
                    .await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::SaveArchive => {
                if is_authenticated {
                    Self::handle_save_archive(archive_manager).await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }

            Request::GetArchiveInfo => {
                if is_authenticated {
                    Self::handle_get_archive_info(archive_manager).await
                } else {
                    Ok(RequestResult::Error {
                        error_type: "NotAuthenticated".to_string(),
                        message: "Database not unlocked".to_string(),
                        details: None,
                    })
                }
            }
        };

        let response_result = match result {
            Ok(success_result) => success_result,
            Err(e) => {
                error!("Request processing failed: {}", e);
                RequestResult::Error {
                    error_type: "PROCESSING_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                }
            }
        };

        // Update session activity for successful authenticated requests
        if let Some(session_id) = &request.session_id {
            if matches!(response_result, RequestResult::Success(_)) {
                let mut sessions_guard = sessions.write().await;
                if let Some(session) = sessions_guard.get_mut(session_id) {
                    session.last_activity = std::time::SystemTime::now();
                }
            }
        }

        Ok(IpcResponse {
            request_id: request.request_id,
            result: response_result,
        })
    }

    /// Handle ping request
    async fn handle_ping(client_info: Option<String>) -> Result<RequestResult> {
        debug!("Ping from client: {:?}", client_info);

        Ok(RequestResult::Success(ResponseData::Pong {
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0, // TODO: track actual uptime
        }))
    }

    /// Handle create session request
    async fn handle_create_session(
        sessions: &Arc<RwLock<HashMap<String, ClientSession>>>,
    ) -> Result<RequestResult> {
        let session_id = Uuid::new_v4().to_string();
        let session = ClientSession {
            session_id: session_id.clone(),
            authenticated: false,
            last_activity: std::time::SystemTime::now(),
            client_info: None,
        };

        sessions.write().await.insert(session_id.clone(), session);
        debug!("Created session: {}", session_id);

        Ok(RequestResult::Success(ResponseData::SessionCreated {
            session_id,
        }))
    }

    /// Handle unlock database request
    async fn handle_unlock_database(
        session_id: &str,
        archive_path: PathBuf,
        master_password: String,
        sessions: &Arc<RwLock<HashMap<String, ClientSession>>>,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager
            .open_archive(&archive_path, &master_password)
            .await
        {
            Ok(()) => {
                // Mark session as authenticated
                if let Some(session) = sessions.write().await.get_mut(session_id) {
                    session.authenticated = true;
                    session.last_activity = std::time::SystemTime::now();
                }

                let credentials = archive_manager
                    .list_credentials()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to list credentials: {}", e))?;

                info!("Database unlocked with {} credentials", credentials.len());

                Ok(RequestResult::Success(ResponseData::DatabaseUnlocked {
                    credential_count: credentials.len(),
                }))
            }
            Err(e) => {
                warn!("Failed to unlock database: {}", e);
                Ok(RequestResult::Error {
                    error_type: "UnlockFailed".to_string(),
                    message: "Failed to unlock database".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle lock database request
    async fn handle_lock_database(
        session_id: &str,
        sessions: &Arc<RwLock<HashMap<String, ClientSession>>>,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager.close_archive().await {
            Ok(()) => {
                // Mark session as not authenticated
                if let Some(session) = sessions.write().await.get_mut(session_id) {
                    session.authenticated = false;
                }

                info!("Database locked");
                Ok(RequestResult::Success(ResponseData::DatabaseLocked))
            }
            Err(e) => {
                error!("Failed to lock database: {}", e);
                Ok(RequestResult::Error {
                    error_type: "LockFailed".to_string(),
                    message: "Failed to lock database".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle get status request
    async fn handle_get_status(archive_manager: &Arc<ArchiveManager>) -> Result<RequestResult> {
        let is_locked = !archive_manager.is_open().await;

        let (archive_path, credential_count) = if !is_locked {
            // TODO: Get actual archive path and credential count
            (Some(PathBuf::from("placeholder.7z")), Some(0))
        } else {
            (None, None)
        };

        Ok(RequestResult::Success(ResponseData::Status {
            is_locked,
            archive_path,
            credential_count,
            last_activity: Some(0), // TODO: track actual last activity
        }))
    }

    /// Handle create archive request
    async fn handle_create_archive(
        archive_path: PathBuf,
        master_password: String,
        archive_manager: &Arc<ArchiveManager>,
        config: &crate::config::Config,
    ) -> Result<RequestResult> {
        // Validate master passphrase using shared validation
        if let Err(e) = validate_master_passphrase_strict(
            &master_password,
            &config.security.passphrase_requirements,
        ) {
            return Ok(RequestResult::Error {
                error_type: "ValidationFailed".to_string(),
                message: "Master passphrase does not meet security requirements".to_string(),
                details: Some(e.to_string()),
            });
        }

        match archive_manager
            .create_archive(&archive_path, &master_password)
            .await
        {
            Ok(()) => {
                info!("Created new archive: {:?}", archive_path);
                Ok(RequestResult::Success(ResponseData::ArchiveCreated))
            }
            Err(e) => {
                error!("Failed to create archive: {}", e);
                Ok(RequestResult::Error {
                    error_type: "CreateFailed".to_string(),
                    message: "Failed to create archive".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle validate repository request
    async fn handle_validate_repository(
        archive_path: PathBuf,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        // Use the API handlers to validate the repository
        let api_handlers = crate::api::ApiHandlers::new(
            archive_manager.clone(),
            crate::config::Config::default(), // We just need a config instance for API handlers
        );

        match api_handlers.validate_repository(archive_path).await {
            Ok(repo_info) => {
                info!("Repository validation successful: {:?}", repo_info.path);
                Ok(RequestResult::Success(ResponseData::RepositoryValidated {
                    path: repo_info.path,
                    size: repo_info.size,
                    last_modified: repo_info.last_modified,
                    is_valid_format: repo_info.is_valid_format,
                    display_name: repo_info.display_name,
                }))
            }
            Err(e) => {
                warn!("Repository validation failed: {}", e);
                Ok(RequestResult::Error {
                    error_type: "ValidationFailed".to_string(),
                    message: "Repository validation failed".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle list credentials request
    async fn handle_list_credentials(
        include_sensitive: bool,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager.list_credentials().await {
            Ok(mut credentials) => {
                if !include_sensitive {
                    // Sanitize sensitive data
                    credentials = credentials
                        .into_iter()
                        .map(|cred| cred.sanitized())
                        .collect();
                }

                Ok(RequestResult::Success(ResponseData::CredentialList {
                    credentials,
                }))
            }
            Err(e) => {
                error!("Failed to list credentials: {}", e);
                Ok(RequestResult::Error {
                    error_type: "ListFailed".to_string(),
                    message: "Failed to list credentials".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle get credential request
    async fn handle_get_credential(
        credential_id: &str,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager.get_credential(credential_id).await {
            Ok(credential) => Ok(RequestResult::Success(ResponseData::Credential {
                credential,
            })),
            Err(e) => {
                error!("Failed to get credential {}: {}", credential_id, e);
                Ok(RequestResult::Error {
                    error_type: "NotFound".to_string(),
                    message: format!("Credential {} not found", credential_id),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle create credential request
    async fn handle_create_credential(
        credential: CredentialRecord,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager.add_credential(credential).await {
            Ok(credential_id) => {
                info!("Created credential: {}", credential_id);
                Ok(RequestResult::Success(ResponseData::CredentialCreated {
                    credential_id,
                }))
            }
            Err(e) => {
                error!("Failed to create credential: {}", e);
                Ok(RequestResult::Error {
                    error_type: "CreateFailed".to_string(),
                    message: "Failed to create credential".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle update credential request
    async fn handle_update_credential(
        credential_id: &str,
        credential: CredentialRecord,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager
            .update_credential(credential_id, credential)
            .await
        {
            Ok(()) => {
                info!("Updated credential: {}", credential_id);
                Ok(RequestResult::Success(ResponseData::CredentialUpdated))
            }
            Err(e) => {
                error!("Failed to update credential {}: {}", credential_id, e);
                Ok(RequestResult::Error {
                    error_type: "UpdateFailed".to_string(),
                    message: format!("Failed to update credential {}", credential_id),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle delete credential request
    async fn handle_delete_credential(
        credential_id: &str,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager.delete_credential(credential_id).await {
            Ok(()) => {
                info!("Deleted credential: {}", credential_id);
                Ok(RequestResult::Success(ResponseData::CredentialDeleted))
            }
            Err(e) => {
                error!("Failed to delete credential {}: {}", credential_id, e);
                Ok(RequestResult::Error {
                    error_type: "DeleteFailed".to_string(),
                    message: format!("Failed to delete credential {}", credential_id),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle search credentials request
    async fn handle_search_credentials(
        query: &str,
        _include_fields: bool,
        _include_tags: bool,
        _include_notes: bool,
        archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        match archive_manager.search_credentials(query).await {
            Ok(credentials) => {
                let total_matches = credentials.len();
                Ok(RequestResult::Success(ResponseData::SearchResults {
                    credentials,
                    total_matches,
                }))
            }
            Err(e) => {
                error!("Failed to search credentials: {}", e);
                Ok(RequestResult::Error {
                    error_type: "SearchFailed".to_string(),
                    message: "Failed to search credentials".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle save archive request
    async fn handle_save_archive(archive_manager: &Arc<ArchiveManager>) -> Result<RequestResult> {
        match archive_manager.save_archive().await {
            Ok(()) => {
                info!("Archive saved");
                Ok(RequestResult::Success(ResponseData::ArchiveSaved))
            }
            Err(e) => {
                error!("Failed to save archive: {}", e);
                Ok(RequestResult::Error {
                    error_type: "SaveFailed".to_string(),
                    message: "Failed to save archive".to_string(),
                    details: Some(e.to_string()),
                })
            }
        }
    }

    /// Handle get archive info request
    async fn handle_get_archive_info(
        _archive_manager: &Arc<ArchiveManager>,
    ) -> Result<RequestResult> {
        // TODO: Implement actual archive info retrieval
        Ok(RequestResult::Success(ResponseData::ArchiveInfo {
            path: PathBuf::from("placeholder.7z"),
            credential_count: 0,
            created_at: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
        }))
    }

    /// Background task to clean up expired sessions
    async fn session_cleanup_task(sessions: Arc<RwLock<HashMap<String, ClientSession>>>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes

        loop {
            interval.tick().await;

            let now = std::time::SystemTime::now();
            let timeout = std::time::Duration::from_secs(3600); // 1 hour

            let mut sessions_guard = sessions.write().await;
            let expired_sessions: Vec<String> = sessions_guard
                .iter()
                .filter_map(|(id, session)| {
                    if now
                        .duration_since(session.last_activity)
                        .unwrap_or_default()
                        > timeout
                    {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for session_id in expired_sessions {
                sessions_guard.remove(&session_id);
                debug!("Cleaned up expired session: {}", session_id);
            }
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Cleanup socket file
        // Note: We can't get the socket path here, but the OS will clean it up
        debug!("IpcServer dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ipc_request_serialization() {
        let request = IpcRequest {
            request_id: "test-123".to_string(),
            session_id: Some("session-456".to_string()),
            request: Request::Ping {
                client_info: Some("test-client".to_string()),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: IpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.request_id, "test-123");
        assert_eq!(deserialized.session_id, Some("session-456".to_string()));
    }

    #[test]
    fn test_ipc_response_serialization() {
        let response = IpcResponse {
            request_id: "test-123".to_string(),
            result: RequestResult::Success(ResponseData::Pong {
                server_version: "1.0.0".to_string(),
                uptime_seconds: 12345,
            }),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: IpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.request_id, "test-123");

        if let RequestResult::Success(ResponseData::Pong {
            server_version,
            uptime_seconds,
        }) = deserialized.result
        {
            assert_eq!(server_version, "1.0.0");
            assert_eq!(uptime_seconds, 12345);
        } else {
            panic!("Wrong response type after deserialization");
        }
    }

    #[test]
    fn test_frontend_json_parsing() {
        // This is the exact JSON format that the frontend sends
        let frontend_json = r#"{
  "request_id": "test-create-archive",
  "session_id": null,
  "request": {
    "CreateArchive": {
      "archive_path": "/home/user/test.7z",
      "master_password": "test_password"
    }
  }
}"#;

        println!("Testing backend parsing of frontend JSON:");
        println!("{}", frontend_json);

        // Test if backend can parse this JSON
        let parsed_result = serde_json::from_str::<IpcRequest>(frontend_json);
        match parsed_result {
            Ok(parsed) => {
                println!("✅ Successfully parsed frontend JSON");
                assert_eq!(parsed.request_id, "test-create-archive");
                assert_eq!(parsed.session_id, None);

                match parsed.request {
                    Request::CreateArchive {
                        archive_path,
                        master_password,
                    } => {
                        assert_eq!(archive_path.to_string_lossy(), "/home/user/test.7z");
                        assert_eq!(master_password, "test_password");
                        println!("✅ Request details parsed correctly");
                    }
                    _ => panic!("Wrong request type parsed"),
                }
            }
            Err(e) => {
                println!("❌ Failed to parse frontend JSON: {}", e);
                panic!("Backend cannot parse frontend JSON: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_ipc_password_validation_flow() {
        use crate::config::{
            CompressionConfig, Config, IpcConfig, LoggingConfig, SecurityConfig, StorageConfig,
        };
        use crate::storage::ArchiveManager;
        use tempfile::tempdir;
        use ziplock_shared::ValidationPresets;

        // Create test config with production password requirements
        let temp_dir = tempdir().unwrap();
        let storage_config = StorageConfig {
            default_archive_dir: temp_dir.path().to_path_buf(),
            max_archive_size_mb: 10,
            backup_count: 3,
            auto_backup: true,
            backup_dir: None,
            file_lock_timeout: 5,
            temp_dir: None,
            verify_integrity: true,
            min_password_length: Some(12),
            compression: CompressionConfig {
                level: 6,
                solid: false,
                multi_threaded: true,
                dictionary_size_mb: 32,
                block_size_mb: 0,
            },
        };

        let config = Config {
            ipc: IpcConfig {
                socket_path: temp_dir.path().join("test.sock"),
                socket_permissions: 0o600,
                max_connections: 10,
                connection_timeout: 10,
                request_timeout: 30,
                log_requests: false,
            },
            storage: storage_config,
            security: SecurityConfig {
                passphrase_requirements: ValidationPresets::production(),
                ..Default::default()
            },
            logging: LoggingConfig::default(),
            limits: crate::config::LimitsConfig::default(),
        };

        let archive_manager = Arc::new(ArchiveManager::new(config.storage.clone()).unwrap());
        let archive_path = temp_dir.path().join("test.7z");

        // Test exact frontend scenario: "ZipLock123!" (11 characters)
        let result = IpcServer::handle_create_archive(
            archive_path.clone(),
            "ZipLock123!".to_string(),
            &archive_manager,
            &config,
        )
        .await;

        // Should fail with password validation error
        assert!(result.is_ok()); // handle_create_archive returns Ok(RequestResult::Error {...})
        if let Ok(RequestResult::Error {
            error_type,
            message,
            details,
        }) = result
        {
            assert_eq!(error_type, "ValidationFailed");
            assert_eq!(
                message,
                "Master passphrase does not meet security requirements"
            );
            assert!(details
                .as_ref()
                .unwrap()
                .contains("Must be at least 12 characters long"));
            println!("✅ IPC correctly rejected 11-character password via shared validation");
        } else {
            panic!("Expected ValidationFailed error");
        }

        // Test with valid strong password
        let archive_path2 = temp_dir.path().join("test2.7z");
        let result = IpcServer::handle_create_archive(
            archive_path2,
            "MySecurePassphrase123!".to_string(),
            &archive_manager,
            &config,
        )
        .await;

        assert!(result.is_ok());
        if let Ok(RequestResult::Success(ResponseData::ArchiveCreated)) = result {
            println!("✅ IPC correctly accepted strong password via shared validation");
        } else {
            panic!("Expected ArchiveCreated success");
        }
    }

    #[test]
    fn test_session_creation() {
        let session = ClientSession {
            session_id: "test-session".to_string(),
            authenticated: false,
            last_activity: std::time::SystemTime::now(),
            client_info: Some("test-client".to_string()),
        };

        assert_eq!(session.session_id, "test-session");
        assert!(!session.authenticated);
        assert_eq!(session.client_info, Some("test-client".to_string()));
    }
}
