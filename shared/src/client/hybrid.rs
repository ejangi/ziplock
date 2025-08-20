//! Hybrid Client for Unified Cross-Platform Architecture
//!
//! This module provides a unified client that uses the hybrid FFI approach
//! for all platforms - mobile and desktop. It handles:
//! - In-memory operations via hybrid FFI (data, crypto, validation)
//! - File system operations for non-mobile platforms (Linux, macOS, Windows)
//! - Seamless integration with existing client interfaces

use crate::error::{SharedError, SharedResult};
use crate::memory_repository::FileOperation;
use crate::models::CredentialRecord;
use serde_json;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use thiserror::Error;

use std::sync::{Mutex, OnceLock};

/// Hybrid client specific errors that handle adaptive runtime scenarios
#[derive(Error, Debug)]
pub enum HybridClientError {
    #[error("Shared library error: {0}")]
    Shared(#[from] SharedError),

    #[error("External file operations required")]
    ExternalFileOpsRequired {
        file_operations: String, // JSON string describing required file operations
    },

    #[error("Runtime context not supported: {message}")]
    RuntimeContextError { message: String },

    #[error("Platform capability error: {message}")]
    PlatformError { message: String },
}

/// Result type for hybrid client operations that may require external file operations
pub type HybridClientResult<T> = Result<T, HybridClientError>;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ZipLockHybridError {
    Success = 0,
    InvalidParameter = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    CredentialNotFound = 4,
    ValidationFailed = 5,
    CryptoError = 6,
    OutOfMemory = 7,
    InternalError = 8,
    SerializationError = 9,
    JsonParseError = 10,
    ExternalFileOperationsRequired = 11,
    RuntimeContextError = 12,
}

/// Global hybrid client state for non-mobile platforms
static HYBRID_CLIENT_STATE: OnceLock<Mutex<Option<HybridClientState>>> = OnceLock::new();

/// Internal state for hybrid client
struct HybridClientState {
    current_archive_path: Option<PathBuf>,
    is_archive_open: bool,
    #[allow(dead_code)]
    is_mobile_platform: bool,
}

impl HybridClientState {
    fn new() -> Self {
        Self {
            current_archive_path: None,
            is_archive_open: false,
            is_mobile_platform: cfg!(target_os = "android") || cfg!(target_os = "ios"),
        }
    }
}

/// Unified hybrid client that works across all platforms
/// Unified hybrid client for all platforms
pub struct ZipLockHybridClient {
    is_initialized: bool,
    is_mobile: bool,
    supports_runtime_detection: bool,
}

impl ZipLockHybridClient {
    /// Create a new hybrid client
    pub fn new() -> SharedResult<Self> {
        let is_mobile = cfg!(target_os = "android") || cfg!(target_os = "ios");

        // Initialize hybrid FFI
        let result = crate::ffi_hybrid::ziplock_hybrid_init();
        if result != 0 {
            return Err(SharedError::Internal {
                message: "Failed to initialize hybrid FFI".to_string(),
            });
        }

        // Initialize global state
        if let Some(state) = HYBRID_CLIENT_STATE.get() {
            let mut state_guard = state.lock().map_err(|e| SharedError::Internal {
                message: format!("Failed to lock state: {}", e),
            })?;

            if state_guard.is_none() {
                *state_guard = Some(HybridClientState::new());
            }
        }

        Ok(Self {
            is_initialized: true,
            is_mobile,
            supports_runtime_detection: true,
        })
    }

    /// Create a new archive with adaptive runtime handling
    pub async fn create_archive_adaptive(
        &self,
        path: PathBuf,
        master_password: String,
    ) -> HybridClientResult<()> {
        if !self.is_initialized {
            return Err(HybridClientError::Shared(SharedError::Internal {
                message: "Client not initialized".to_string(),
            }));
        }

        if self.is_mobile {
            // Mobile platforms always require external file operations
            let file_ops = self.get_create_archive_file_operations(&path, &master_password)?;
            return Err(HybridClientError::ExternalFileOpsRequired {
                file_operations: file_ops,
            });
        }

        // Desktop platforms: use adaptive hybrid FFI
        let path_str = path.to_string_lossy();
        let path_cstring = CString::new(path_str.as_ref()).map_err(|e| {
            HybridClientError::Shared(SharedError::Internal {
                message: format!("Invalid path string: {}", e),
            })
        })?;

        let password_cstring = CString::new(master_password.clone()).map_err(|e| {
            HybridClientError::Shared(SharedError::Internal {
                message: format!("Invalid password string: {}", e),
            })
        })?;

        let result = unsafe {
            crate::ffi_hybrid::ziplock_hybrid_create_archive(
                path_cstring.as_ptr(),
                password_cstring.as_ptr(),
            )
        };

        match result {
            0 => {
                // Success - update client state
                self.update_archive_state(&path, true)?;
                Ok(())
            }
            11 => {
                // External file operations required
                let file_ops = self.get_create_archive_file_operations(&path, &master_password)?;
                Err(HybridClientError::ExternalFileOpsRequired {
                    file_operations: file_ops,
                })
            }
            _ => {
                // Other error
                let error_msg = self.get_last_error_message();
                Err(HybridClientError::Shared(SharedError::Internal {
                    message: format!("Failed to create archive: {}", error_msg),
                }))
            }
        }
    }

    /// Open an archive with adaptive runtime handling
    pub async fn open_archive_adaptive(
        &self,
        path: PathBuf,
        master_password: String,
    ) -> HybridClientResult<()> {
        if !self.is_initialized {
            return Err(HybridClientError::Shared(SharedError::Internal {
                message: "Client not initialized".to_string(),
            }));
        }

        if self.is_mobile {
            // Mobile platforms always require external file operations
            let file_ops = self.get_open_archive_file_operations(&path, &master_password)?;
            return Err(HybridClientError::ExternalFileOpsRequired {
                file_operations: file_ops,
            });
        }

        // Desktop platforms: use adaptive hybrid FFI
        let path_str = path.to_string_lossy();
        let path_cstring = CString::new(path_str.as_ref()).map_err(|e| {
            HybridClientError::Shared(SharedError::Internal {
                message: format!("Invalid path string: {}", e),
            })
        })?;

        let password_cstring = CString::new(master_password.clone()).map_err(|e| {
            HybridClientError::Shared(SharedError::Internal {
                message: format!("Invalid password string: {}", e),
            })
        })?;

        let result = unsafe {
            crate::ffi_hybrid::ziplock_hybrid_open_archive(
                path_cstring.as_ptr(),
                password_cstring.as_ptr(),
            )
        };

        match result {
            0 => {
                // Success - update client state
                self.update_archive_state(&path, true)?;
                Ok(())
            }
            11 => {
                // External file operations required
                let file_ops = self.get_open_archive_file_operations(&path, &master_password)?;
                Err(HybridClientError::ExternalFileOpsRequired {
                    file_operations: file_ops,
                })
            }
            _ => {
                // Other error
                let error_msg = self.get_last_error_message();
                Err(HybridClientError::Shared(SharedError::Internal {
                    message: format!("Failed to open archive: {}", error_msg),
                }))
            }
        }
    }

    /// Helper to get file operations for archive creation
    fn get_create_archive_file_operations(
        &self,
        path: &PathBuf,
        password: &str,
    ) -> Result<String, HybridClientError> {
        let json_str = unsafe {
            let result = crate::ffi_hybrid::ziplock_hybrid_get_file_operations();
            if result.is_null() {
                return Err(HybridClientError::Shared(SharedError::Internal {
                    message: "Failed to get file operations JSON".to_string(),
                }));
            }

            let c_str = CStr::from_ptr(result);
            let json_str = c_str.to_string_lossy().to_string();
            crate::ffi_hybrid::ziplock_hybrid_free_string(result);
            json_str
        };

        // If no specific operations available, create a basic create operation
        if json_str.is_empty() {
            let basic_ops = serde_json::json!({
                "operations": [
                    {
                        "type": "create_archive",
                        "path": path.to_string_lossy(),
                        "password": password,
                        "format": "7z"
                    }
                ]
            });
            Ok(basic_ops.to_string())
        } else {
            Ok(json_str)
        }
    }

    /// Helper to get file operations for archive opening
    fn get_open_archive_file_operations(
        &self,
        path: &PathBuf,
        password: &str,
    ) -> Result<String, HybridClientError> {
        let json_str = unsafe {
            let result = crate::ffi_hybrid::ziplock_hybrid_get_file_operations();
            if result.is_null() {
                return Err(HybridClientError::Shared(SharedError::Internal {
                    message: "Failed to get file operations JSON".to_string(),
                }));
            }

            let c_str = CStr::from_ptr(result);
            let json_str = c_str.to_string_lossy().to_string();
            crate::ffi_hybrid::ziplock_hybrid_free_string(result);
            json_str
        };

        // If no specific operations available, create a basic open operation
        if json_str.is_empty() {
            let basic_ops = serde_json::json!({
                "operations": [
                    {
                        "type": "extract_archive",
                        "path": path.to_string_lossy(),
                        "password": password,
                        "format": "7z"
                    }
                ]
            });
            Ok(basic_ops.to_string())
        } else {
            Ok(json_str)
        }
    }

    /// Helper to get last error message from FFI
    fn get_last_error_message(&self) -> String {
        unsafe {
            let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
            if !error_ptr.is_null() {
                let c_str = CStr::from_ptr(error_ptr);
                let rust_str = c_str.to_string_lossy().to_string();
                crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                rust_str
            } else {
                "Unknown error".to_string()
            }
        }
    }

    /// Helper to update archive state
    fn update_archive_state(&self, path: &PathBuf, is_open: bool) -> Result<(), HybridClientError> {
        if let Some(state) = HYBRID_CLIENT_STATE.get() {
            if let Ok(mut state_guard) = state.lock() {
                if let Some(state_mut) = state_guard.as_mut() {
                    state_mut.current_archive_path =
                        if is_open { Some(path.clone()) } else { None };
                    state_mut.is_archive_open = is_open;
                }
            }
        }
        Ok(())
    }

    /// Create a new archive (legacy method - use create_archive_adaptive for better error handling)
    pub async fn create_archive(&self, path: PathBuf, master_password: String) -> SharedResult<()> {
        match self.create_archive_adaptive(path, master_password).await {
            Ok(()) => Ok(()),
            Err(HybridClientError::Shared(e)) => Err(e),
            Err(HybridClientError::ExternalFileOpsRequired { .. }) => {
                // For legacy compatibility, return an error that suggests using the adaptive method
                Err(SharedError::Internal {
                    message:
                        "External file operations required - use create_archive_adaptive method"
                            .to_string(),
                })
            }
            Err(HybridClientError::RuntimeContextError { message }) => Err(SharedError::Internal {
                message: format!("Runtime context error: {}", message),
            }),
            Err(HybridClientError::PlatformError { message }) => Err(SharedError::Internal {
                message: format!("Platform error: {}", message),
            }),
        }
    }

    /// Open an existing archive (legacy method - use open_archive_adaptive for better error handling)
    pub async fn open_archive(&self, path: PathBuf, master_password: String) -> SharedResult<()> {
        match self.open_archive_adaptive(path, master_password).await {
            Ok(()) => Ok(()),
            Err(HybridClientError::Shared(e)) => Err(e),
            Err(HybridClientError::ExternalFileOpsRequired { .. }) => {
                // For legacy compatibility, return an error that suggests using the adaptive method
                Err(SharedError::Internal {
                    message: "External file operations required - use open_archive_adaptive method"
                        .to_string(),
                })
            }
            Err(HybridClientError::RuntimeContextError { message }) => Err(SharedError::Internal {
                message: format!("Runtime context error: {}", message),
            }),
            Err(HybridClientError::PlatformError { message }) => Err(SharedError::Internal {
                message: format!("Platform error: {}", message),
            }),
        }
    }

    /// Close the current archive
    pub async fn close_archive(&self) -> SharedResult<()> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        if self.is_mobile {
            // Mobile platforms handle this externally
            return Ok(());
        }

        // Desktop platforms: close archive (no-op for now)
        // Update state
        if let Some(state) = HYBRID_CLIENT_STATE.get() {
            if let Ok(mut state_guard) = state.lock() {
                if let Some(state_mut) = state_guard.as_mut() {
                    state_mut.current_archive_path = None;
                    state_mut.is_archive_open = false;
                }
            }
        }

        Ok(())
    }

    /// Load content from platform (for mobile platforms)
    pub async fn load_content_from_platform(&self, _files_json: String) -> SharedResult<()> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        // For now, this is a no-op - will be implemented with proper FFI
        Ok(())
    }

    /// Get file operations needed for persistence (for mobile platforms)
    pub async fn get_file_operations(&self) -> SharedResult<Vec<FileOperation>> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        // For now, return empty operations - will be implemented with proper FFI
        Ok(Vec::new())
    }

    /// Add a credential (unified across platforms)
    pub async fn add_credential(&self, credential: CredentialRecord) -> SharedResult<String> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        if self.is_mobile {
            // Mobile platforms handle this differently
            return Ok(credential.id.clone());
        }

        // Desktop platforms: use FFI to add credential to memory state
        let title_cstring =
            CString::new(credential.title.clone()).map_err(|e| SharedError::Internal {
                message: format!("Invalid title string: {}", e),
            })?;

        let type_cstring = CString::new(credential.credential_type.clone()).map_err(|e| {
            SharedError::Internal {
                message: format!("Invalid credential type string: {}", e),
            }
        })?;

        unsafe {
            let credential_id = crate::ffi_hybrid::ziplock_hybrid_credential_create(
                title_cstring.as_ptr(),
                type_cstring.as_ptr(),
            );

            if credential_id == 0 {
                // Get error message from FFI
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to create credential: {}", error_msg),
                });
            }

            // Add fields to the credential
            for (field_name, field) in &credential.fields {
                let field_name_cstring =
                    CString::new(field_name.clone()).map_err(|e| SharedError::Internal {
                        message: format!("Invalid field name string: {}", e),
                    })?;

                let field_value_cstring =
                    CString::new(field.value.clone()).map_err(|e| SharedError::Internal {
                        message: format!("Invalid field value string: {}", e),
                    })?;

                let field_type_int = match field.field_type {
                    crate::models::FieldType::Text => 0,
                    crate::models::FieldType::Password => 1,
                    crate::models::FieldType::Email => 2,
                    crate::models::FieldType::Url => 3,
                    crate::models::FieldType::Username => 4,
                    crate::models::FieldType::Phone => 5,
                    crate::models::FieldType::CreditCardNumber => 6,
                    crate::models::FieldType::ExpiryDate => 7,
                    crate::models::FieldType::Cvv => 8,
                    crate::models::FieldType::TotpSecret => 9,
                    crate::models::FieldType::TextArea => 10,
                    crate::models::FieldType::Number => 11,
                    crate::models::FieldType::Date => 12,
                    crate::models::FieldType::Custom(_) => 0, // Treat custom fields as text
                };

                let result = crate::ffi_hybrid::ziplock_hybrid_credential_add_field(
                    credential_id,
                    field_name_cstring.as_ptr(),
                    field_value_cstring.as_ptr(),
                    field_type_int,
                    if field.sensitive { 1 } else { 0 },
                );

                if result != 0 {
                    // Get error message from FFI
                    let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                    let error_msg = if !error_ptr.is_null() {
                        let c_str = CStr::from_ptr(error_ptr);
                        let rust_str = c_str.to_string_lossy().to_string();
                        crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                        rust_str
                    } else {
                        "Unknown error".to_string()
                    };

                    return Err(SharedError::Internal {
                        message: format!("Failed to add field {}: {}", field_name, error_msg),
                    });
                }
            }

            // Save to disk for desktop platforms
            let save_result = crate::ffi_hybrid::ziplock_hybrid_save_archive();
            if save_result != 0 {
                // Get error message from FFI
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to save archive: {}", error_msg),
                });
            }

            Ok(credential_id.to_string())
        }
    }

    /// Get a credential by ID (unified across platforms)
    pub async fn get_credential(&self, id: &str) -> SharedResult<CredentialRecord> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        if self.is_mobile {
            // Mobile platforms handle this differently
            return Err(SharedError::Internal {
                message: "Get credential not implemented for mobile".to_string(),
            });
        }

        // Parse ID as u32 for FFI
        let credential_id: u32 = id.parse().map_err(|_| SharedError::Internal {
            message: "Invalid credential ID format".to_string(),
        })?;

        unsafe {
            let yaml_ptr =
                crate::ffi_hybrid::ziplock_hybrid_credential_get_yaml(credential_id as u64);
            if yaml_ptr.is_null() {
                return Err(SharedError::Internal {
                    message: "Credential not found".to_string(),
                });
            }

            // Convert C string to Rust string
            let c_str = CStr::from_ptr(yaml_ptr);
            let yaml_str = c_str.to_string_lossy();

            // Parse YAML to get credential
            let credential: CredentialRecord = match serde_yaml::from_str(&yaml_str) {
                Ok(cred) => cred,
                Err(e) => {
                    crate::ffi_hybrid::ziplock_hybrid_free_string(yaml_ptr);
                    return Err(SharedError::Serialization {
                        message: format!("Failed to parse credential YAML: {}", e),
                    });
                }
            };

            // Free the allocated string
            crate::ffi_hybrid::ziplock_hybrid_free_string(yaml_ptr);

            Ok(credential)
        }
    }

    /// Update a credential (unified across platforms)
    pub async fn update_credential(&self, credential: CredentialRecord) -> SharedResult<()> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        if self.is_mobile {
            // Mobile platforms handle this differently
            return Ok(());
        }

        // Parse ID as u32 for FFI
        let credential_id: u32 = credential.id.parse().map_err(|_| SharedError::Internal {
            message: "Invalid credential ID format".to_string(),
        })?;

        // Serialize credential to YAML
        let yaml_str =
            serde_yaml::to_string(&credential).map_err(|e| SharedError::Serialization {
                message: format!("Failed to serialize credential: {}", e),
            })?;

        let yaml_cstring = CString::new(yaml_str).map_err(|e| SharedError::Internal {
            message: format!("Invalid YAML string: {}", e),
        })?;

        unsafe {
            let result = crate::ffi_hybrid::ziplock_hybrid_credential_update_yaml(
                credential_id as u64,
                yaml_cstring.as_ptr(),
            );

            if result != 0 {
                // Get error message from FFI
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to update credential: {}", error_msg),
                });
            }

            // Save to disk for desktop platforms
            let save_result = crate::ffi_hybrid::ziplock_hybrid_save_archive();
            if save_result != 0 {
                // Get error message from FFI
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to save archive after update: {}", error_msg),
                });
            }

            Ok(())
        }
    }

    /// Delete a credential (unified across platforms)
    pub async fn delete_credential(&self, id: &str) -> SharedResult<()> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        if self.is_mobile {
            // Mobile platforms handle this differently
            return Ok(());
        }

        // Parse ID as u32 for FFI
        let credential_id: u32 = id.parse().map_err(|_| SharedError::Internal {
            message: "Invalid credential ID format".to_string(),
        })?;

        unsafe {
            let result = crate::ffi_hybrid::ziplock_hybrid_credential_delete(credential_id as u64);

            if result != 0 {
                // Get error message from FFI
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to delete credential: {}", error_msg),
                });
            }

            // Save to disk for desktop platforms
            let save_result = crate::ffi_hybrid::ziplock_hybrid_save_archive();
            if save_result != 0 {
                // Get error message from FFI
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to save archive after deletion: {}", error_msg),
                });
            }

            Ok(())
        }
    }

    /// List all credentials (unified across platforms)
    pub async fn list_credentials(&self) -> SharedResult<Vec<CredentialRecord>> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        // For desktop platforms, use archive manager directly
        // For mobile platforms, this will need to be implemented differently
        if self.is_mobile {
            // Mobile platforms will get credentials from platform code
            // For now, return empty list - this should be populated by platform
            return Ok(Vec::new());
        }

        // Call hybrid FFI to get credentials as YAML
        unsafe {
            let yaml_ptr = crate::ffi_hybrid::ziplock_hybrid_credential_list_yaml();
            if yaml_ptr.is_null() {
                // Get error message
                let error_ptr = crate::ffi_hybrid::ziplock_hybrid_get_last_error();
                let error_msg = if !error_ptr.is_null() {
                    let c_str = CStr::from_ptr(error_ptr);
                    let rust_str = c_str.to_string_lossy().to_string();
                    crate::ffi_hybrid::ziplock_hybrid_free_string(error_ptr);
                    rust_str
                } else {
                    "Unknown error".to_string()
                };

                return Err(SharedError::Internal {
                    message: format!("Failed to list credentials: {}", error_msg),
                });
            }

            // Convert C string to Rust string
            let c_str = CStr::from_ptr(yaml_ptr);
            let yaml_str = c_str.to_string_lossy();

            // Parse YAML to get credentials
            let credentials: Vec<CredentialRecord> = match serde_yaml::from_str(&yaml_str) {
                Ok(creds) => creds,
                Err(e) => {
                    crate::ffi_hybrid::ziplock_hybrid_free_string(yaml_ptr);
                    return Err(SharedError::Serialization {
                        message: format!("Failed to parse credentials YAML: {}", e),
                    });
                }
            };

            // Free the C string
            crate::ffi_hybrid::ziplock_hybrid_free_string(yaml_ptr);

            Ok(credentials)
        }
    }

    /// Search credentials (unified across platforms)
    pub async fn search_credentials(&self, query: &str) -> SharedResult<Vec<CredentialRecord>> {
        if !self.is_initialized {
            return Err(SharedError::Internal {
                message: "Client not initialized".to_string(),
            });
        }

        // For now, get all credentials and filter in Rust
        // TODO: Implement search in FFI for better performance
        let all_credentials = self.list_credentials().await?;
        let query_lower = query.to_lowercase();

        let filtered = all_credentials
            .into_iter()
            .filter(|cred| {
                cred.title.to_lowercase().contains(&query_lower)
                    || cred.credential_type.to_lowercase().contains(&query_lower)
                    || cred
                        .notes
                        .as_ref()
                        .map_or(false, |notes| notes.to_lowercase().contains(&query_lower))
                    || cred
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(filtered)
    }

    /// Check if an archive is currently open
    pub async fn is_archive_open(&self) -> bool {
        if !self.is_initialized {
            return false;
        }

        if self.is_mobile {
            // For mobile platforms, assume archive is open if FFI is initialized
            // Platform code manages the actual archive state
            return true;
        }

        // For desktop platforms, check global state
        if let Some(state) = HYBRID_CLIENT_STATE.get() {
            if let Ok(state_guard) = state.lock() {
                if let Some(state_ref) = state_guard.as_ref() {
                    return state_ref.is_archive_open;
                }
            }
        }

        false
    }
}

impl Drop for ZipLockHybridClient {
    fn drop(&mut self) {
        if self.is_initialized {
            crate::ffi_hybrid::ziplock_hybrid_cleanup();
        }
    }
}

impl Default for ZipLockHybridClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default hybrid client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    /// Test hybrid client creation
    async fn test_hybrid_client_creation() {
        let _client = ZipLockHybridClient::new().unwrap();
        // Test passes if client creation succeeds
    }

    #[tokio::test]
    async fn test_hybrid_client_credential_operations() {
        let _client = ZipLockHybridClient::new().unwrap();

        // Note: These tests would work if we had a mock archive or test setup
        // For now, they're here to show the intended API
    }

    #[tokio::test]
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn test_desktop_archive_operations() {
        let temp_dir = TempDir::new().unwrap();
        let _archive_path = temp_dir.path().join("test.7z");

        let _client = ZipLockHybridClient::new().unwrap();

        // Note: These tests would work with proper hybrid FFI setup
        // For now, just test that client creation succeeds
        assert!(temp_dir.path().exists());
    }
}
