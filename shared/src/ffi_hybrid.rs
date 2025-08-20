//! Hybrid FFI layer for ZipLock - Unified Cross-Platform Operations
//!
//! This module provides a unified C-compatible interface that handles:
//! - Data validation, cryptography, and business logic operations (all platforms)
//! - Filesystem operations for non-mobile platforms (Linux, macOS, Windows)
//! - In-memory repository management for mobile platforms (Android, iOS)

#![allow(static_mut_refs)]

use crate::api::ZipLockApi;
use crate::archive::{ArchiveConfig, ArchiveManager};
use crate::memory_repository::MemoryRepository;
use crate::models::{CredentialRecord, FieldType};
use crate::validation::validate_credential;
use crate::yaml::YamlUtils;

use base64::Engine;
use serde_json;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::path::PathBuf;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Runtime metrics for telemetry and performance tracking
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub strategy_selections: HashMap<String, u64>,
    pub platform_detections: HashMap<String, u64>,
    pub operation_timings: HashMap<String, Duration>,
    pub total_operations: u64,
    pub fallback_count: u64,
    pub error_count: u64,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self {
            strategy_selections: HashMap::new(),
            platform_detections: HashMap::new(),
            operation_timings: HashMap::new(),
            total_operations: 0,
            fallback_count: 0,
            error_count: 0,
        }
    }

    pub fn record_strategy_selection(&mut self, strategy: &str) {
        *self
            .strategy_selections
            .entry(strategy.to_string())
            .or_insert(0) += 1;
        self.total_operations += 1;

        if strategy == "external_file_ops" {
            self.fallback_count += 1;
        }
    }

    pub fn record_platform_detection(&mut self, platform: &str) {
        *self
            .platform_detections
            .entry(platform.to_string())
            .or_insert(0) += 1;
    }

    pub fn record_operation_timing(&mut self, operation: &str, duration: Duration) {
        self.operation_timings
            .insert(operation.to_string(), duration);
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    pub fn get_fallback_rate(&self) -> f64 {
        if self.total_operations > 0 {
            (self.fallback_count as f64) / (self.total_operations as f64)
        } else {
            0.0
        }
    }

    pub fn get_error_rate(&self) -> f64 {
        if self.total_operations > 0 {
            (self.error_count as f64) / (self.total_operations as f64)
        } else {
            0.0
        }
    }
}

/// Global runtime metrics
static RUNTIME_METRICS: OnceLock<Mutex<RuntimeMetrics>> = OnceLock::new();

/// Error codes for hybrid FFI operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipLockHybridError {
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

/// Runtime strategy for adaptive execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeStrategy {
    CreateOwned,     // Create our own runtime (standalone usage)
    UseExisting,     // Use existing runtime (called from async context)
    ExternalFileOps, // Delegate file ops to caller (mobile-style)
}

/// Global state for the hybrid FFI
static HYBRID_FFI_STATE: OnceLock<Mutex<Option<HybridFfiState>>> = OnceLock::new();

/// Hybrid FFI state structure
struct HybridFfiState {
    api: Option<ZipLockApi>,
    repository: Option<MemoryRepository>,
    credentials: HashMap<u32, CredentialRecord>,
    next_credential_id: u32,
    last_error: Option<String>,
    current_archive_path: Option<PathBuf>,
    current_archive_password: Option<String>,
    archive_manager: Option<Arc<ArchiveManager>>,
    runtime: Option<tokio::runtime::Runtime>,
    runtime_strategy: RuntimeStrategy,
}

impl HybridFfiState {
    fn new() -> Self {
        let runtime_strategy = Self::detect_runtime_context();

        let runtime = match runtime_strategy {
            RuntimeStrategy::CreateOwned => tokio::runtime::Runtime::new().ok(),
            _ => None,
        };

        Self {
            api: None,
            repository: None,
            credentials: HashMap::new(),
            next_credential_id: 1,
            last_error: None,
            current_archive_path: None,
            current_archive_password: None,
            archive_manager: None,
            runtime,
            runtime_strategy,
        }
    }

    /// Detect the current runtime context and determine the best strategy
    fn detect_runtime_context() -> RuntimeStrategy {
        let start_time = Instant::now();

        // Check if we're likely on a mobile platform first
        #[cfg(target_os = "android")]
        {
            Self::record_platform_detection("android");
            Self::record_strategy_selection("external_file_ops");
            Self::record_detection_timing(start_time.elapsed());
            return RuntimeStrategy::ExternalFileOps;
        }

        #[cfg(target_os = "ios")]
        {
            Self::record_platform_detection("ios");
            Self::record_strategy_selection("external_file_ops");
            Self::record_detection_timing(start_time.elapsed());
            return RuntimeStrategy::ExternalFileOps;
        }

        // For desktop platforms, check if we're in an async context
        match tokio::runtime::Handle::try_current() {
            Ok(_) => {
                // We're in an existing async context - use external file ops to prevent nested runtime
                crate::log_debug!(
                    "FFI: Existing async runtime detected, using external file operations"
                );
                Self::record_platform_detection("desktop_async");
                Self::record_strategy_selection("external_file_ops");
                Self::record_detection_timing(start_time.elapsed());
                RuntimeStrategy::ExternalFileOps
            }
            Err(_) => {
                // No existing runtime, safe to create our own
                crate::log_debug!("FFI: No existing runtime detected, creating owned runtime");
                Self::record_platform_detection("desktop_sync");
                Self::record_strategy_selection("create_owned");
                Self::record_detection_timing(start_time.elapsed());
                RuntimeStrategy::CreateOwned
            }
        }
    }

    /// Record platform detection for telemetry
    fn record_platform_detection(platform: &str) {
        if let Some(metrics) = RUNTIME_METRICS.get() {
            if let Ok(mut metrics_guard) = metrics.lock() {
                metrics_guard.record_platform_detection(platform);
            }
        }
    }

    /// Record strategy selection for telemetry
    fn record_strategy_selection(strategy: &str) {
        if let Some(metrics) = RUNTIME_METRICS.get() {
            if let Ok(mut metrics_guard) = metrics.lock() {
                metrics_guard.record_strategy_selection(strategy);
            }
        }
    }

    /// Record detection timing for telemetry
    fn record_detection_timing(duration: Duration) {
        if let Some(metrics) = RUNTIME_METRICS.get() {
            if let Ok(mut metrics_guard) = metrics.lock() {
                metrics_guard.record_operation_timing("runtime_detection", duration);
            }
        }
    }

    /// Execute an async operation using the appropriate strategy
    fn execute_async<F, T>(&self, future: F) -> Result<T, String>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
        T: Send + 'static,
    {
        match &self.runtime_strategy {
            RuntimeStrategy::CreateOwned => {
                if let Some(runtime) = &self.runtime {
                    runtime.block_on(future).map_err(|e| e.to_string())
                } else {
                    Err("No runtime available for owned execution".to_string())
                }
            }
            RuntimeStrategy::UseExisting => {
                // This shouldn't happen anymore since we map UseExisting to ExternalFileOps
                Err("ExternalFileOperationsRequired".to_string())
            }
            RuntimeStrategy::ExternalFileOps => {
                // Signal that external file operations are required
                Err("ExternalFileOperationsRequired".to_string())
            }
        }
    }

    /// Check if file operations should be handled externally
    fn requires_external_file_ops(&self) -> bool {
        matches!(self.runtime_strategy, RuntimeStrategy::ExternalFileOps)
    }

    fn set_error(&mut self, error: String) {
        self.last_error = Some(error);
    }

    fn get_next_id(&mut self) -> u32 {
        let id = self.next_credential_id;
        self.next_credential_id += 1;
        id
    }
}

/// Set the last error message
/// Set the last error message for FFI functions
fn set_last_error(message: &str) {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                state.set_error(message.to_string());
            }
        }
    }
}

/// Helper function to convert C string to Rust string
unsafe fn c_str_to_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        return None;
    }
    CStr::from_ptr(c_str).to_str().ok().map(|s| s.to_string())
}

/// Helper function to convert Rust string to C string
fn string_to_c_str(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Initialize the hybrid FFI library
#[no_mangle]
pub extern "C" fn ziplock_hybrid_init() -> c_int {
    if HYBRID_FFI_STATE.get().is_some() {
        crate::log_warn!("Hybrid FFI already initialized");
        return ZipLockHybridError::AlreadyInitialized as c_int;
    }

    // Initialize runtime metrics
    let _ = RUNTIME_METRICS.set(Mutex::new(RuntimeMetrics::new()));

    // Create archive config with defaults
    let config = ArchiveConfig::default();

    // Create API instance
    let api = match ZipLockApi::new(config) {
        Ok(api) => api,
        Err(e) => {
            crate::log_error!("Failed to create ZipLock API: {}", e);
            return ZipLockHybridError::InternalError as c_int;
        }
    };

    // Create memory repository
    let repository = MemoryRepository::new();

    // Initialize global state
    let mut state = HybridFfiState::new();
    state.api = Some(api);
    state.repository = Some(repository);

    // Store the state
    if let Err(_) = HYBRID_FFI_STATE.set(Mutex::new(Some(state))) {
        crate::log_error!("Failed to set hybrid FFI state");
        return ZipLockHybridError::InternalError as c_int;
    }

    crate::log_info!("Hybrid FFI initialized successfully");
    ZipLockHybridError::Success as c_int
}

/// Cleanup and shutdown the hybrid FFI library
#[no_mangle]
pub extern "C" fn ziplock_hybrid_cleanup() -> c_int {
    crate::log_debug!("Hybrid FFI: Cleaning up");

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                // Don't drop runtime in async context - just clear the reference
                if matches!(state.runtime_strategy, RuntimeStrategy::ExternalFileOps) {
                    state.runtime = None;
                }
                // Clear other state
                state.credentials.clear();
                state.last_error = None;
            }
        }
    }

    crate::log_info!("Hybrid FFI cleanup completed");
    ZipLockHybridError::Success as c_int
}

/// Get the last error message
#[no_mangle]
pub extern "C" fn ziplock_hybrid_get_last_error() -> *mut c_char {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                if let Some(error) = &state.last_error {
                    return string_to_c_str(error.clone());
                }
            }
        }
    }
    ptr::null_mut()
}

/// Free a C string returned by other functions
/// Free a string allocated by the library
#[no_mangle]
pub extern "C" fn ziplock_hybrid_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Get runtime metrics as JSON string
#[no_mangle]
pub extern "C" fn ziplock_hybrid_get_metrics() -> *mut c_char {
    if let Some(metrics) = RUNTIME_METRICS.get() {
        if let Ok(metrics_guard) = metrics.lock() {
            let metrics_json = serde_json::json!({
                "strategy_selections": metrics_guard.strategy_selections,
                "total_operations": metrics_guard.total_operations,
                "fallback_count": metrics_guard.fallback_count,
                "fallback_rate": metrics_guard.get_fallback_rate(),
                "error_count": metrics_guard.error_count,
                "error_rate": metrics_guard.get_error_rate(),
                "platform_detections": metrics_guard.platform_detections,
                "operation_timings": metrics_guard.operation_timings.iter()
                    .map(|(k, v)| (k.clone(), v.as_millis()))
                    .collect::<HashMap<String, u128>>()
            });

            if let Ok(json_string) = serde_json::to_string(&metrics_json) {
                if let Ok(c_string) = CString::new(json_string) {
                    return c_string.into_raw();
                }
            }
        }
    }

    ptr::null_mut()
}

/// Reset runtime metrics
#[no_mangle]
pub extern "C" fn ziplock_hybrid_reset_metrics() -> c_int {
    if let Some(metrics) = RUNTIME_METRICS.get() {
        if let Ok(mut metrics_guard) = metrics.lock() {
            *metrics_guard = RuntimeMetrics::new();
            crate::log_info!("Runtime metrics reset successfully");
            return ZipLockHybridError::Success as c_int;
        }
    }

    ZipLockHybridError::InternalError as c_int
}

/// Log current metrics to debug output
#[no_mangle]
pub extern "C" fn ziplock_hybrid_log_metrics() -> c_int {
    if let Some(metrics) = RUNTIME_METRICS.get() {
        if let Ok(metrics_guard) = metrics.lock() {
            crate::log_info!("=== ZipLock Adaptive Runtime Metrics ===");
            crate::log_info!("Total operations: {}", metrics_guard.total_operations);
            crate::log_info!(
                "Fallback rate: {:.2}%",
                metrics_guard.get_fallback_rate() * 100.0
            );
            crate::log_info!("Error rate: {:.2}%", metrics_guard.get_error_rate() * 100.0);

            crate::log_info!("Strategy selections:");
            for (strategy, count) in &metrics_guard.strategy_selections {
                crate::log_info!("  {}: {}", strategy, count);
            }

            crate::log_info!("Platform detections:");
            for (platform, count) in &metrics_guard.platform_detections {
                crate::log_info!("  {}: {}", platform, count);
            }

            crate::log_info!("Recent operation timings:");
            for (operation, duration) in &metrics_guard.operation_timings {
                crate::log_info!("  {}: {}ms", operation, duration.as_millis());
            }
            crate::log_info!("==========================================");

            return ZipLockHybridError::Success as c_int;
        }
    }

    ZipLockHybridError::InternalError as c_int
}

/// Create a new credential and return its ID
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_create(
    title: *const c_char,
    credential_type: *const c_char,
) -> u64 {
    let title_str = unsafe { c_str_to_string(title) };
    let type_str = unsafe { c_str_to_string(credential_type) };

    if title_str.is_none() || type_str.is_none() {
        set_last_error("Invalid title or credential type");
        return 0;
    }

    let title = title_str.unwrap();
    let credential_type = type_str.unwrap();

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                let id = state.get_next_id();
                let credential = CredentialRecord::new(title, credential_type);

                // Validate the credential
                if let Err(e) = validate_credential(&credential) {
                    crate::log_error!("FFI: Credential validation failed: {}", e);
                    state.set_error(format!("Credential validation failed: {}", e));
                    return 0;
                }

                crate::log_info!(
                    "FFI: Creating new credential '{}' with ID {}",
                    credential.title,
                    id
                );
                state.credentials.insert(id, credential);
                crate::log_debug!(
                    "FFI: Total credentials in state: {}",
                    state.credentials.len()
                );
                return id as u64;
            }
        }
    }

    set_last_error("Failed to access FFI state");
    0
}

/// Add a field to a credential
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_add_field(
    credential_id: u64,
    field_name: *const c_char,
    field_value: *const c_char,
    field_type: c_int,
    sensitive: c_int,
) -> c_int {
    let credential_id = credential_id as u32;
    let name_str = unsafe { c_str_to_string(field_name) };
    let value_str = unsafe { c_str_to_string(field_value) };

    if name_str.is_none() || value_str.is_none() {
        set_last_error("Invalid field name or value");
        return ZipLockHybridError::InvalidParameter as c_int;
    }

    let name = name_str.unwrap();
    let value = value_str.unwrap();
    let is_sensitive = sensitive != 0;

    let field_type_enum = match field_type {
        0 => FieldType::Text,
        1 => FieldType::Password,
        2 => FieldType::Email,
        3 => FieldType::Url,
        4 => FieldType::Username,
        5 => FieldType::Phone,
        6 => FieldType::CreditCardNumber,
        7 => FieldType::ExpiryDate,
        8 => FieldType::Cvv,
        9 => FieldType::TotpSecret,
        10 => FieldType::TextArea,
        11 => FieldType::Number,
        12 => FieldType::Date,
        13 => FieldType::Custom("custom".to_string()),
        _ => {
            set_last_error("Invalid field type");
            return ZipLockHybridError::InvalidParameter as c_int;
        }
    };

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                if let Some(credential) = state.credentials.get_mut(&credential_id) {
                    credential.set_field(
                        name,
                        crate::models::CredentialField::new(field_type_enum, value, is_sensitive),
                    );
                    return ZipLockHybridError::Success as c_int;
                }
            }
        }
    }

    set_last_error("Credential not found");
    ZipLockHybridError::CredentialNotFound as c_int
}

/// Get credential information as YAML
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_get_yaml(credential_id: u64) -> *mut c_char {
    let credential_id = credential_id as u32;

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                if let Some(credential) = state.credentials.get(&credential_id) {
                    match YamlUtils::serialize_credential(credential) {
                        Ok(yaml) => return string_to_c_str(yaml),
                        Err(e) => {
                            set_last_error(&format!("YAML serialization failed: {}", e));
                            return ptr::null_mut();
                        }
                    }
                }
            }
        }
    }

    set_last_error("Credential not found");
    ptr::null_mut()
}

/// Update credential from YAML
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_update_yaml(
    credential_id: u64,
    yaml: *const c_char,
) -> c_int {
    let credential_id = credential_id as u32;
    let yaml_str = unsafe { c_str_to_string(yaml) };

    if yaml_str.is_none() {
        set_last_error("Invalid YAML string");
        return ZipLockHybridError::InvalidParameter as c_int;
    }

    let yaml_string = yaml_str.unwrap();

    // Parse YAML to credential
    let updated_credential: CredentialRecord = match YamlUtils::deserialize_credential(&yaml_string)
    {
        Ok(cred) => cred,
        Err(e) => {
            set_last_error(&format!("YAML parsing failed: {}", e));
            return ZipLockHybridError::SerializationError as c_int;
        }
    };

    // Validate the credential
    if let Err(e) = validate_credential(&updated_credential) {
        set_last_error(&format!("Credential validation failed: {}", e));
        return ZipLockHybridError::ValidationFailed as c_int;
    }

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                if state.credentials.contains_key(&credential_id) {
                    state.credentials.insert(credential_id, updated_credential);
                    return ZipLockHybridError::Success as c_int;
                }
            }
        }
    }

    set_last_error("Credential not found");
    ZipLockHybridError::CredentialNotFound as c_int
}

/// Create credential from YAML and return its ID
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_from_yaml(yaml: *const c_char) -> u64 {
    let yaml_str = unsafe { c_str_to_string(yaml) };

    if yaml_str.is_none() {
        set_last_error("Invalid YAML string");
        return 0;
    }

    let yaml_string = yaml_str.unwrap();

    // Parse YAML to credential
    let credential: CredentialRecord = match YamlUtils::deserialize_credential(&yaml_string) {
        Ok(cred) => cred,
        Err(e) => {
            set_last_error(&format!("YAML parsing failed: {}", e));
            return 0;
        }
    };

    // Validate the credential
    if let Err(e) = validate_credential(&credential) {
        set_last_error(&format!("Credential validation failed: {}", e));
        return 0;
    }

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                let id = state.get_next_id();
                state.credentials.insert(id, credential);
                return id as u64;
            }
        }
    }

    set_last_error("Failed to access FFI state");
    0
}

/// Get all credentials as YAML array
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_list_yaml() -> *mut c_char {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                crate::log_debug!(
                    "FFI: Listing credentials, found {} credentials in state",
                    state.credentials.len()
                );
                let credentials: Vec<&CredentialRecord> = state.credentials.values().collect();
                match serde_yaml::to_string(&credentials) {
                    Ok(yaml) => {
                        crate::log_debug!(
                            "FFI: Successfully serialized {} credentials to YAML",
                            credentials.len()
                        );
                        return string_to_c_str(yaml);
                    }
                    Err(e) => {
                        crate::log_error!("FFI: YAML serialization failed: {}", e);
                        set_last_error(&format!("YAML serialization failed: {}", e));
                        return ptr::null_mut();
                    }
                }
            }
        }
    }

    crate::log_error!("FFI: Failed to access FFI state for credential listing");
    set_last_error("Failed to access FFI state");
    ptr::null_mut()
}

/// Simple field validation functions
#[no_mangle]
pub extern "C" fn ziplock_hybrid_validate_email(email: *const c_char) -> c_int {
    let email_str = unsafe { c_str_to_string(email) };
    if let Some(email) = email_str {
        // Basic email validation
        if email.contains('@') && email.contains('.') && email.len() > 5 {
            return 1; // Valid
        }
    }
    0 // Invalid
}

#[no_mangle]
pub extern "C" fn ziplock_hybrid_validate_url(url: *const c_char) -> c_int {
    let url_str = unsafe { c_str_to_string(url) };
    if let Some(url) = url_str {
        // Basic URL validation
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://") {
            return 1; // Valid
        }
    }
    0 // Invalid
}

#[no_mangle]
pub extern "C" fn ziplock_hybrid_validate_phone(phone: *const c_char) -> c_int {
    let phone_str = unsafe { c_str_to_string(phone) };
    if let Some(phone) = phone_str {
        // Basic phone validation - contains digits and common separators
        let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() >= 10 && cleaned.len() <= 15 {
            return 1; // Valid
        }
    }
    0 // Invalid
}

/// Password generation (simplified without crypto module)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_generate_password(
    length: c_int,
    include_uppercase: c_int,
    include_lowercase: c_int,
    include_numbers: c_int,
    include_symbols: c_int,
) -> *mut c_char {
    use rand::Rng;

    if length <= 0 || length > 1000 {
        set_last_error("Invalid password length");
        return ptr::null_mut();
    }

    let mut charset = String::new();

    if include_lowercase != 0 {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if include_uppercase != 0 {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if include_numbers != 0 {
        charset.push_str("0123456789");
    }
    if include_symbols != 0 {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    if charset.is_empty() {
        set_last_error("No character sets selected");
        return ptr::null_mut();
    }

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect();

    string_to_c_str(password)
}

/// Calculate password strength (simplified)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_calculate_password_strength(password: *const c_char) -> c_double {
    let password_str = unsafe { c_str_to_string(password) };
    if let Some(pwd) = password_str {
        let mut score = 0.0;

        // Length bonus
        score += (pwd.len() as f64 * 4.0).min(50.0);

        // Character variety bonus
        if pwd.chars().any(|c| c.is_ascii_lowercase()) {
            score += 10.0;
        }
        if pwd.chars().any(|c| c.is_ascii_uppercase()) {
            score += 10.0;
        }
        if pwd.chars().any(|c| c.is_ascii_digit()) {
            score += 10.0;
        }
        if pwd.chars().any(|c| !c.is_alphanumeric()) {
            score += 20.0;
        }

        // Cap at 100
        score.min(100.0)
    } else {
        0.0
    }
}

/// Calculate password entropy (simplified)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_calculate_entropy(password: *const c_char) -> c_double {
    let password_str = unsafe { c_str_to_string(password) };
    if let Some(pwd) = password_str {
        let mut charset_size = 0;

        if pwd.chars().any(|c| c.is_ascii_lowercase()) {
            charset_size += 26;
        }
        if pwd.chars().any(|c| c.is_ascii_uppercase()) {
            charset_size += 26;
        }
        if pwd.chars().any(|c| c.is_ascii_digit()) {
            charset_size += 10;
        }
        if pwd.chars().any(|c| !c.is_alphanumeric()) {
            charset_size += 32;
        }

        if charset_size > 0 {
            (pwd.len() as f64) * (charset_size as f64).log2()
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Simple encryption/decryption (base64 encoding for demo purposes)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_encrypt_string(
    plaintext: *const c_char,
    _key: *const c_char,
) -> *mut c_char {
    let text = unsafe { c_str_to_string(plaintext) };
    if let Some(text) = text {
        let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
        return string_to_c_str(encoded);
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn ziplock_hybrid_decrypt_string(
    ciphertext: *const c_char,
    _key: *const c_char,
) -> *mut c_char {
    let cipher = unsafe { c_str_to_string(ciphertext) };
    if let Some(cipher) = cipher {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&cipher) {
            if let Ok(text) = String::from_utf8(decoded) {
                return string_to_c_str(text);
            }
        }
    }
    ptr::null_mut()
}

/// Generate a salt (random base64 string)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_generate_salt(length: c_int) -> *mut c_char {
    use rand::Rng;

    if length <= 0 || length > 1000 {
        set_last_error("Invalid salt length");
        return ptr::null_mut();
    }

    let mut rng = rand::thread_rng();
    let salt: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(&salt);
    string_to_c_str(encoded)
}

/// Delete a credential
#[no_mangle]
pub extern "C" fn ziplock_hybrid_credential_delete(credential_id: u64) -> c_int {
    let credential_id = credential_id as u32;

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                if state.credentials.remove(&credential_id).is_some() {
                    return ZipLockHybridError::Success as c_int;
                }
            }
        }
    }

    set_last_error("Credential not found");
    ZipLockHybridError::CredentialNotFound as c_int
}

/// Get repository structure information
#[no_mangle]
pub extern "C" fn ziplock_hybrid_repository_get_structure() -> *mut c_char {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                if let Some(repository) = state.repository.as_ref() {
                    let structure = repository.get_structure();
                    match serde_yaml::to_string(structure) {
                        Ok(yaml) => return string_to_c_str(yaml),
                        Err(e) => {
                            set_last_error(&format!("YAML serialization failed: {}", e));
                            return ptr::null_mut();
                        }
                    }
                }
            }
        }
    }

    set_last_error("Repository not available");
    ptr::null_mut()
}

// Desktop-specific functions (Linux, macOS, Windows) that handle filesystem operations

/// Create an archive on disk (desktop platforms only)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_create_archive(
    archive_path: *const c_char,
    password: *const c_char,
) -> c_int {
    let path_str = unsafe { c_str_to_string(archive_path) };
    let password_str = unsafe { c_str_to_string(password) };

    if path_str.is_none() || password_str.is_none() {
        set_last_error("Invalid archive path or password");
        return ZipLockHybridError::InvalidParameter as c_int;
    }

    let archive_path = PathBuf::from(path_str.unwrap());
    let password = password_str.unwrap();

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                // Check if external file operations are required
                if state.requires_external_file_ops() {
                    crate::log_info!(
                        "FFI: External file operations required for creating archive in current context"
                    );
                    set_last_error("External file operations required - use platform-specific archive handling");
                    return ZipLockHybridError::ExternalFileOperationsRequired as c_int;
                }

                if let Some(_api) = &state.api {
                    // Use direct synchronous archive creation to avoid spawn_blocking deadlock
                    let create_result = create_archive_sync(&archive_path, &password);

                    match create_result {
                        Ok(_) => {
                            // Store archive state for future operations
                            let archive_manager = match ArchiveManager::new(ArchiveConfig::default())
                            {
                                Ok(manager) => manager,
                                Err(e) => {
                                    set_last_error(&format!(
                                        "Failed to create archive manager: {}",
                                        e
                                    ));
                                    return ZipLockHybridError::InternalError as c_int;
                                }
                            };

                            state.current_archive_path = Some(archive_path);
                            state.current_archive_password = Some(password);
                            state.archive_manager = Some(Arc::new(archive_manager));

                            return ZipLockHybridError::Success as c_int;
                        }
                        Err(e) => {
                            set_last_error(&format!("Failed to create archive: {}", e));
                            return ZipLockHybridError::InternalError as c_int;
                        }
                    }
                }
            }
        }
    }

    set_last_error("FFI not initialized");
    ZipLockHybridError::NotInitialized as c_int
}

/// Synchronous archive creation to avoid async/blocking deadlocks in FFI
fn create_archive_sync(
    archive_path: &PathBuf,
    password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::time::SystemTime;
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize)]
    struct SimpleArchiveMetadata {
        version: String,
        created_at: SystemTime,
        last_modified: SystemTime,
        credential_count: usize,
    }

    if archive_path.exists() {
        return Err("Archive already exists".into());
    }

    // Create parent directory if needed
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create temporary directory for archive operations
    let temp_dir = TempDir::new()?;

    // Create metadata file
    let metadata = SimpleArchiveMetadata {
        version: "1.0".to_string(),
        created_at: SystemTime::now(),
        last_modified: SystemTime::now(),
        credential_count: 0,
    };

    // Save metadata to temp directory
    let metadata_path = temp_dir.path().join("metadata.yml");
    let metadata_yaml = serde_yaml::to_string(&metadata)?;
    fs::write(&metadata_path, metadata_yaml)?;

    // Create credentials directory
    let credentials_dir = temp_dir.path().join("credentials");
    fs::create_dir_all(&credentials_dir)?;

    // Create a README file
    let readme_content = format!(
        "# ZipLock Archive\n\n\
        This is a ZipLock encrypted archive created on {}.\n\
        \n\
        Version: {}\n\
        Format: 7z with AES-256 encryption\n",
        metadata
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        metadata.version
    );

    let readme_path = temp_dir.path().join("README.md");
    fs::write(&readme_path, readme_content)?;

    // Create a placeholder file in the credentials directory
    let placeholder_content = "# ZipLock Credentials Directory\n\n\
        This directory will contain your encrypted credential files.\n";

    let placeholder_path = credentials_dir.join(".ziplock_placeholder");
    fs::write(&placeholder_path, placeholder_content)?;

    // Create the encrypted archive directly without spawn_blocking
    sevenz_rust2::compress_to_path_encrypted(temp_dir.path(), archive_path, password.into())?;

    Ok(())
}

/// Open an archive and load credentials (desktop platforms only)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_open_archive(
    archive_path: *const c_char,
    password: *const c_char,
) -> c_int {
    let path_str = unsafe { c_str_to_string(archive_path) };
    let password_str = unsafe { c_str_to_string(password) };

    if path_str.is_none() || password_str.is_none() {
        set_last_error("Invalid archive path or password");
        return ZipLockHybridError::InvalidParameter as c_int;
    }

    let archive_path = PathBuf::from(path_str.unwrap());
    let password = password_str.unwrap();

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                // Check if external file operations are required
                if state.requires_external_file_ops() {
                    crate::log_info!(
                        "FFI: External file operations required for opening archive in current context"
                    );
                    set_last_error("External file operations required - use platform-specific archive handling");
                    return ZipLockHybridError::ExternalFileOperationsRequired as c_int;
                }

                // We can use integrated file operations
                let archive_path_clone = archive_path.clone();
                let password_clone = password.clone();

                let open_future = async move {
                    let archive_manager = ArchiveManager::new(ArchiveConfig::default())
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                    archive_manager
                        .open_archive(archive_path_clone, password_clone)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                    let credentials = archive_manager
                        .list_credentials()
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                    Result::<
                        (ArchiveManager, Vec<CredentialRecord>),
                        Box<dyn std::error::Error + Send + Sync>,
                    >::Ok((archive_manager, credentials))
                };

                let result = state.execute_async(open_future);

                match result {
                    Ok((archive_manager, credentials)) => {
                        // Clear existing credentials and load from archive
                        crate::log_info!(
                            "FFI: Loading {} credentials from opened archive",
                            credentials.len()
                        );
                        state.credentials.clear();
                        let mut next_id = 1u32;

                        for credential in credentials {
                            crate::log_debug!(
                                "FFI: Loading credential '{}' with ID {}",
                                credential.title,
                                next_id
                            );
                            state.credentials.insert(next_id, credential);
                            next_id += 1;
                        }

                        state.next_credential_id = next_id;

                        // Store archive state for future save operations
                        state.current_archive_path = Some(archive_path);
                        state.current_archive_password = Some(password);
                        state.archive_manager = Some(Arc::new(archive_manager));

                        crate::log_info!(
                            "FFI: Successfully loaded {} credentials into state",
                            state.credentials.len()
                        );
                        return ZipLockHybridError::Success as c_int;
                    }
                    Err(e) => {
                        if e.contains("ExternalFileOperationsRequired") {
                            crate::log_info!(
                                "FFI: Runtime context changed, external file operations now required"
                            );
                            set_last_error("External file operations required - use platform-specific archive handling");
                            return ZipLockHybridError::ExternalFileOperationsRequired as c_int;
                        }

                        set_last_error(&format!(
                            "Failed to open archive or load credentials: {}",
                            e
                        ));
                        return ZipLockHybridError::InternalError as c_int;
                    }
                }
            }
        }
    }

    set_last_error("FFI not initialized");
    ZipLockHybridError::NotInitialized as c_int
}

/// Save all credentials to the open archive (desktop platforms only)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_save_archive() -> c_int {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                // Check if external file operations are required
                if state.requires_external_file_ops() {
                    crate::log_info!(
                        "FFI: External file operations required for saving archive in current context"
                    );
                    set_last_error("External file operations required - use platform-specific archive handling");
                    return ZipLockHybridError::ExternalFileOperationsRequired as c_int;
                }

                // Check if we have an archive opened
                let archive_path = match &state.current_archive_path {
                    Some(path) => path.clone(),
                    None => {
                        set_last_error("No archive is currently open");
                        return ZipLockHybridError::NotInitialized as c_int;
                    }
                };

                let archive_password = match &state.current_archive_password {
                    Some(password) => password.clone(),
                    None => {
                        set_last_error("No archive password available");
                        return ZipLockHybridError::NotInitialized as c_int;
                    }
                };

                // Use existing archive manager if available
                if let Some(_archive_manager) = &state.archive_manager {
                    // Collect credentials to avoid borrowing state
                    let credentials: Vec<CredentialRecord> =
                        state.credentials.values().cloned().collect();

                    // Clone necessary values
                    let archive_path_clone = archive_path.clone();
                    let archive_password_clone = archive_password.clone();

                    let save_future = async move {
                        // Create a new archive manager for this save operation
                        let temp_manager = ArchiveManager::new(ArchiveConfig::default())
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                        temp_manager
                            .open_archive(archive_path_clone, archive_password_clone)
                            .await
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                        // Add all current credentials
                        for credential in credentials {
                            match temp_manager.get_credential(credential.id.clone()).await {
                                Ok(_) => {
                                    // Credential exists, update it
                                    temp_manager
                                        .update_credential(
                                            credential.id.clone(),
                                            credential.clone(),
                                        )
                                        .await
                                        .map_err(|e| {
                                            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                        })?;
                                }
                                Err(_) => {
                                    // Credential doesn't exist, create it
                                    temp_manager
                                        .add_credential(credential.clone())
                                        .await
                                        .map_err(|e| {
                                            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                        })?;
                                }
                            }
                        }

                        // Save the archive
                        temp_manager
                            .save_archive()
                            .await
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    };

                    let result = state.execute_async(save_future);

                    match result {
                        Ok(_) => return ZipLockHybridError::Success as c_int,
                        Err(e) => {
                            if e.contains("ExternalFileOperationsRequired") {
                                crate::log_info!(
                                    "FFI: Runtime context changed, external file operations now required"
                                );
                                set_last_error("External file operations required - use platform-specific archive handling");
                                return ZipLockHybridError::ExternalFileOperationsRequired as c_int;
                            }

                            set_last_error(&format!("Failed to save archive: {}", e));
                            return ZipLockHybridError::InternalError as c_int;
                        }
                    }
                } else {
                    set_last_error("No archive manager available");
                    return ZipLockHybridError::NotInitialized as c_int;
                }
            }
        }
    }

    set_last_error("FFI not initialized");
    ZipLockHybridError::NotInitialized as c_int
}

/// Get library version
#[no_mangle]
pub extern "C" fn ziplock_hybrid_get_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    string_to_c_str(version.to_string())
}

/// Get file operations needed for external execution (for async contexts)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_get_file_operations() -> *mut c_char {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                let _credentials: Vec<CredentialRecord> =
                    state.credentials.values().cloned().collect();

                // Create file operations JSON
                let mut operations = Vec::new();

                // Add create directory operation for credentials
                operations.push(serde_json::json!({
                    "type": "create_directory",
                    "path": "credentials"
                }));

                // Add operations for each credential
                for (id, credential) in &state.credentials {
                    let credential_dir = format!("credentials/{}", id);
                    operations.push(serde_json::json!({
                        "type": "create_directory",
                        "path": credential_dir
                    }));

                    // Serialize credential to YAML
                    match serde_yaml::to_string(&credential) {
                        Ok(yaml_content) => {
                            let file_path = format!("credentials/{}/record.yml", id);
                            operations.push(serde_json::json!({
                                "type": "write_file",
                                "path": file_path,
                                "content": yaml_content
                            }));
                        }
                        Err(e) => {
                            set_last_error(&format!("Failed to serialize credential: {}", e));
                            return ptr::null_mut();
                        }
                    }
                }

                // Return operations as JSON string
                match serde_json::to_string(&operations) {
                    Ok(json) => return string_to_c_str(json),
                    Err(e) => {
                        set_last_error(&format!("Failed to serialize file operations: {}", e));
                        return ptr::null_mut();
                    }
                }
            }
        }
    }

    set_last_error("FFI not initialized");
    ptr::null_mut()
}

/// Load credentials from extracted file contents (for external file operations)
#[no_mangle]
pub extern "C" fn ziplock_hybrid_load_from_extracted_files(files_json: *const c_char) -> c_int {
    let files_str = unsafe { c_str_to_string(files_json) };

    if files_str.is_none() {
        set_last_error("Invalid files JSON");
        return ZipLockHybridError::InvalidParameter as c_int;
    }

    let files_json = files_str.unwrap();

    // Parse the files JSON
    let files_map: std::collections::HashMap<String, String> =
        match serde_json::from_str(&files_json) {
            Ok(map) => map,
            Err(e) => {
                set_last_error(&format!("Failed to parse files JSON: {}", e));
                return ZipLockHybridError::JsonParseError as c_int;
            }
        };

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                state.credentials.clear();
                let mut next_id = 1u32;

                // Look for credential files in the format credentials/{id}/record.yml
                for (file_path, content) in files_map {
                    if file_path.starts_with("credentials/") && file_path.ends_with("/record.yml") {
                        // Extract credential ID from path
                        let path_parts: Vec<&str> = file_path.split('/').collect();
                        if path_parts.len() >= 3 {
                            if let Ok(credential_id) = path_parts[1].parse::<u32>() {
                                // Parse credential from YAML content
                                match serde_yaml::from_str::<CredentialRecord>(&content) {
                                    Ok(credential) => {
                                        crate::log_debug!(
                                            "FFI: Loading credential '{}' with ID {}",
                                            credential.title,
                                            credential_id
                                        );
                                        state.credentials.insert(credential_id, credential);
                                        if credential_id >= next_id {
                                            next_id = credential_id + 1;
                                        }
                                    }
                                    Err(e) => {
                                        crate::log_warn!(
                                            "FFI: Failed to parse credential from {}: {}",
                                            file_path,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                state.next_credential_id = next_id;

                crate::log_info!(
                    "FFI: Successfully loaded {} credentials from extracted files",
                    state.credentials.len()
                );

                return ZipLockHybridError::Success as c_int;
            }
        }
    }

    set_last_error("FFI not initialized");
    ZipLockHybridError::NotInitialized as c_int
}

/// Set archive information for external file operations mode
#[no_mangle]
pub extern "C" fn ziplock_hybrid_set_archive_info(
    archive_path: *const c_char,
    password: *const c_char,
) -> c_int {
    let path_str = unsafe { c_str_to_string(archive_path) };
    let password_str = unsafe { c_str_to_string(password) };

    if path_str.is_none() || password_str.is_none() {
        set_last_error("Invalid archive path or password");
        return ZipLockHybridError::InvalidParameter as c_int;
    }

    let archive_path = PathBuf::from(path_str.unwrap());
    let password = password_str.unwrap();

    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(mut state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_mut() {
                state.current_archive_path = Some(archive_path);
                state.current_archive_password = Some(password);

                crate::log_info!("FFI: Archive information set for external file operations");
                return ZipLockHybridError::Success as c_int;
            }
        }
    }

    set_last_error("FFI not initialized");
    ZipLockHybridError::NotInitialized as c_int
}

/// Check what runtime strategy is currently being used
#[no_mangle]
pub extern "C" fn ziplock_hybrid_get_runtime_strategy() -> c_int {
    if let Some(state_mutex) = HYBRID_FFI_STATE.get() {
        if let Ok(state_guard) = state_mutex.lock() {
            if let Some(state) = state_guard.as_ref() {
                return match state.runtime_strategy {
                    RuntimeStrategy::CreateOwned => 0,
                    RuntimeStrategy::UseExisting => 1, // This shouldn't happen anymore
                    RuntimeStrategy::ExternalFileOps => 2,
                };
            }
        }
    }
    -1 // Error/not initialized
}

/// Test echo function for debugging
#[no_mangle]
pub extern "C" fn ziplock_hybrid_test_echo(input: *const c_char) -> *mut c_char {
    let input_str = unsafe { c_str_to_string(input) };
    if let Some(text) = input_str {
        return string_to_c_str(format!("Echo: {}", text));
    }
    ptr::null_mut()
}
