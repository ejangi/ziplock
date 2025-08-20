//! FFI (Foreign Function Interface) layer for ZipLock shared library
//!
//! This module provides C-compatible bindings for the ZipLock shared library,
//! enabling integration with mobile platforms (iOS, Android) and other languages
//! that can interface with C libraries.

#![allow(static_mut_refs)] // FFI requires static mut for C compatibility

use crate::api::{ApiSession, ZipLockApi};
use crate::archive::ArchiveConfig;
use crate::logging::{init_logging, is_debug_enabled, set_debug_enabled, LoggingConfig};

#[cfg(target_os = "android")]
use crate::archive::{is_android_saf_available, is_content_uri};
use crate::models::CredentialRecord;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

extern crate libc;

/// Error codes for FFI operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipLockError {
    Success = 0,
    InvalidParameter = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    ArchiveNotFound = 4,
    ArchiveCorrupted = 5,
    InvalidPassword = 6,
    PermissionDenied = 7,
    OutOfMemory = 8,
    InternalError = 9,
    SessionNotFound = 10,
    SessionExpired = 11,
    ArchiveNotOpen = 12,
    CredentialNotFound = 13,
    ValidationFailed = 14,
    CryptoError = 15,
    IoError = 16,
}

/// C-compatible credential record structure
#[repr(C)]
pub struct CCredentialRecord {
    pub id: *mut c_char,
    pub title: *mut c_char,
    pub credential_type: *mut c_char,
    pub notes: *mut c_char,
    pub created_at: i64,
    pub updated_at: i64,
    pub field_count: usize,
    pub fields: *mut CCredentialField,
    pub tag_count: usize,
    pub tags: *mut *mut c_char,
}

/// C-compatible credential field structure
#[repr(C)]
pub struct CCredentialField {
    pub name: *mut c_char,
    pub value: *mut c_char,
    pub field_type: *mut c_char,
    pub label: *mut c_char,
    pub sensitive: c_int,
    pub required: c_int,
}

/// C-compatible structure for credential templates
#[repr(C)]
pub struct CCredentialTemplate {
    pub name: *mut c_char,
    pub description: *mut c_char,
    pub field_count: c_int,
    pub fields: *mut CFieldTemplate,
    pub tag_count: c_int,
    pub tags: *mut *mut c_char,
}

/// C-compatible structure for field templates
#[repr(C)]
pub struct CFieldTemplate {
    pub name: *mut c_char,
    pub field_type: *mut c_char,
    pub label: *mut c_char,
    pub required: c_int,
    pub sensitive: c_int,
    pub default_value: *mut c_char,
    pub validation_min_length: c_int,
    pub validation_max_length: c_int,
    pub validation_pattern: *mut c_char,
    pub validation_message: *mut c_char,
}

/// C-compatible validation result structure
#[repr(C)]
pub struct CValidationResult {
    pub is_valid: c_int,
    pub can_auto_repair: c_int,
    pub issue_count: usize,
    pub issues: *mut *mut c_char,
}

/// Global state for the FFI layer
struct FFIState {
    api: Option<Arc<ZipLockApi>>,
    runtime: Option<Runtime>,
    sessions: HashMap<String, ApiSession>,
    last_error: Option<String>,
}

static mut FFI_STATE: Option<Mutex<FFIState>> = None;

/// Initialize the ZipLock library
#[no_mangle]
pub extern "C" fn ziplock_init() -> c_int {
    unsafe {
        if FFI_STATE.is_some() {
            return ZipLockError::AlreadyInitialized as c_int;
        }

        // Initialize logging system first
        if let Err(e) = init_logging() {
            eprintln!("Failed to initialize logging: {}", e);
            // Continue anyway, logging is not critical for core functionality
        }

        let runtime = match Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        let config = ArchiveConfig::default();
        let api = match ZipLockApi::new(config) {
            Ok(api) => Arc::new(api),
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        let state = FFIState {
            api: Some(api),
            runtime: Some(runtime),
            sessions: HashMap::new(),
            last_error: None,
        };

        FFI_STATE = Some(Mutex::new(state));

        crate::log_info!("ZipLock FFI library initialized successfully");
        ZipLockError::Success as c_int
    }
}

/// Shutdown the ZipLock library
#[no_mangle]
pub extern "C" fn ziplock_shutdown() -> c_int {
    unsafe {
        if let Some(state_mutex) = FFI_STATE.take() {
            let mut state = match state_mutex.lock() {
                Ok(state) => state,
                Err(_) => return ZipLockError::InternalError as c_int,
            };

            state.api = None;
            state.runtime = None;
            state.sessions.clear();
            state.last_error = None;
        }
    }
    ZipLockError::Success as c_int
}

/// Create a new session
#[no_mangle]
pub extern "C" fn ziplock_session_create() -> *mut c_char {
    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => return ptr::null_mut(),
        };

        let mut state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => return ptr::null_mut(),
        };

        let api = match state.api.as_ref() {
            Some(api) => api,
            None => return ptr::null_mut(),
        };

        let _runtime = match state.runtime.as_ref() {
            Some(rt) => rt,
            None => return ptr::null_mut(),
        };

        let session = match api.create_session() {
            Ok(session) => session,
            Err(_) => return ptr::null_mut(),
        };

        let session_id = session.session_id.clone();
        state.sessions.insert(session_id.clone(), session);

        match CString::new(session_id) {
            Ok(cstring) => cstring.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Create a new archive
#[no_mangle]
pub extern "C" fn ziplock_archive_create(
    path: *const c_char,
    master_password: *const c_char,
) -> c_int {
    crate::log_debug!("FFI: ziplock_archive_create called");
    if path.is_null() || master_password.is_null() {
        unsafe {
            set_last_error("Invalid parameter: path or password is null");
        }
        return ZipLockError::InvalidParameter as c_int;
    }

    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => {
                set_last_error("Library not initialized");
                return ZipLockError::NotInitialized as c_int;
            }
        };

        let mut state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => {
                set_last_error("Failed to acquire state lock");
                return ZipLockError::InternalError as c_int;
            }
        };

        let api = match state.api.as_ref() {
            Some(api) => api.clone(),
            None => {
                state.last_error = Some("API not initialized".to_string());
                return ZipLockError::NotInitialized as c_int;
            }
        };

        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => {
                state.last_error = Some("Invalid UTF-8 in path parameter".to_string());
                return ZipLockError::InvalidParameter as c_int;
            }
        };

        let password_str = match CStr::from_ptr(master_password).to_str() {
            Ok(s) => s,
            Err(_) => {
                state.last_error = Some("Invalid UTF-8 in password parameter".to_string());
                return ZipLockError::InvalidParameter as c_int;
            }
        };

        let path_buf = PathBuf::from(path_str);

        // Get a reference to the runtime without borrowing state
        if state.runtime.is_none() {
            state.last_error = Some("Runtime not available".to_string());
            return ZipLockError::InternalError as c_int;
        }

        // Create a simple blocking call with basic error handling
        let result = {
            let runtime = state.runtime.as_ref().unwrap();
            runtime.block_on(api.create_archive(path_buf, password_str.to_string()))
        };

        match result {
            Ok(_) => {
                state.last_error = None;
                ZipLockError::Success as c_int
            }
            Err(e) => {
                state.last_error = Some(format!("Failed to create archive: {}", e));
                ZipLockError::InternalError as c_int
            }
        }
    }
}

/// Open an existing archive
#[no_mangle]
pub extern "C" fn ziplock_archive_open(
    path: *const c_char,
    master_password: *const c_char,
) -> c_int {
    crate::log_debug!("FFI: ziplock_archive_open called");

    // Android emulator debugging (affects both x86_64 and ARM emulators)
    #[cfg(target_os = "android")]
    {
        eprintln!("ANDROID DEBUG: Running on Android platform");
        eprintln!("ANDROID DEBUG: Archive path: {:?}", path);
        eprintln!("ANDROID DEBUG: sevenz_rust2 may have emulator compatibility issues");

        if crate::platform::android::is_android_emulator() {
            eprintln!("EMULATOR WARNING: Detected emulator environment");
            eprintln!("EMULATOR WARNING: sevenz_rust2 library may crash in emulated environments");
        }
    }

    // Set up panic hook to prevent crashes
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("FFI PANIC in ziplock_archive_open: {}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("Panic occurred at {}:{}", location.file(), location.line());
        }
        if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Panic payload: {}", payload);
        } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Panic payload: {}", payload);
        }
    }));

    // Use eprintln for debugging
    eprintln!("FFI STEP 1: Entering ziplock_archive_open function");

    // Wrap the entire function in a panic-catching block
    let result = std::panic::catch_unwind(|| {
        eprintln!("FFI STEP 2: Inside panic catch block");

        if path.is_null() || master_password.is_null() {
            eprintln!("FFI Error: Null pointer parameters");
            return ZipLockError::InvalidParameter as c_int;
        }
        eprintln!("FFI STEP 3: Parameter validation passed");

        eprintln!("FFI Info: Starting archive open operation");

        unsafe {
            eprintln!("FFI STEP 4: Entering unsafe block");

            let state_mutex = match FFI_STATE.as_ref() {
                Some(state) => {
                    eprintln!("FFI Info: FFI state found");
                    state
                }
                None => {
                    eprintln!("FFI Error: FFI not initialized");
                    return ZipLockError::NotInitialized as c_int;
                }
            };
            eprintln!("FFI STEP 5: Got FFI state mutex");

            let state = match state_mutex.lock() {
                Ok(state) => {
                    eprintln!("FFI Info: Successfully locked FFI state");
                    state
                }
                Err(e) => {
                    eprintln!("FFI Error: Failed to lock FFI state: {}", e);
                    return ZipLockError::InternalError as c_int;
                }
            };
            eprintln!("FFI STEP 6: Locked FFI state successfully");

            let api = match state.api.as_ref() {
                Some(api) => {
                    eprintln!("FFI Info: API instance found");
                    api
                }
                None => {
                    eprintln!("FFI Error: API not initialized");
                    return ZipLockError::NotInitialized as c_int;
                }
            };
            eprintln!("FFI STEP 7: Got API instance");

            let runtime = match state.runtime.as_ref() {
                Some(rt) => {
                    eprintln!("FFI Info: Runtime found");
                    rt
                }
                None => {
                    eprintln!("FFI Error: Runtime not initialized");
                    return ZipLockError::InternalError as c_int;
                }
            };
            eprintln!("FFI STEP 8: Got runtime instance");

            let path_str = match CStr::from_ptr(path).to_str() {
                Ok(s) => {
                    eprintln!("FFI Info: Path parsed: {}", s);
                    s
                }
                Err(e) => {
                    eprintln!("FFI Error: Invalid path string: {}", e);
                    return ZipLockError::InvalidParameter as c_int;
                }
            };
            eprintln!("FFI STEP 9: Parsed path string");

            let password_str = match CStr::from_ptr(master_password).to_str() {
                Ok(s) => {
                    eprintln!("FFI Info: Password parsed (length: {})", s.len());
                    s
                }
                Err(e) => {
                    eprintln!("FFI Error: Invalid password string: {}", e);
                    return ZipLockError::InvalidParameter as c_int;
                }
            };
            eprintln!("FFI STEP 10: Parsed password string");

            let path_buf = PathBuf::from(path_str);
            eprintln!("FFI STEP 11: Created PathBuf: {:?}", path_buf);

            // Check if this is an Android content URI
            #[cfg(target_os = "android")]
            {
                eprintln!("FFI STEP 12: Checking if content URI on Android");
                if is_content_uri(path_str) {
                    eprintln!("FFI Info: Detected content URI");
                    // For content URIs, we need Android SAF to be initialized
                    if !is_android_saf_available() {
                        let error_msg = format!(
                            "Android SAF not available for content URI: {}. Please ensure SAF callbacks are initialized.",
                            path_str
                        );
                        eprintln!("FFI Error: {}", error_msg);
                        set_last_error(&error_msg);
                        return ZipLockError::InternalError as c_int;
                    }
                    eprintln!("FFI Info: Using Android SAF for content URI: {}", path_str);
                } else {
                    eprintln!("FFI Info: Using regular file path: {}", path_str);
                }
            }
            eprintln!("FFI STEP 13: Content URI check completed");

            // Pre-validation for file path archives
            #[cfg(target_os = "android")]
            if !is_content_uri(path_str) {
                if let Ok(metadata) = std::fs::metadata(&path_buf) {
                    let file_size = metadata.len();
                    eprintln!("FFI Info: Archive file size: {} bytes", file_size);

                    if file_size < 32 {
                        let error_msg = format!(
                            "Archive file is too small ({} bytes) to be valid 7z",
                            file_size
                        );
                        eprintln!("FFI Error: {}", error_msg);
                        set_last_error(&error_msg);
                        return ZipLockError::ArchiveCorrupted as c_int;
                    }

                    if file_size < 512 {
                        eprintln!(
                            "FFI Warning: Archive file is small ({} bytes), may be newly created",
                            file_size
                        );
                    }
                } else {
                    eprintln!("FFI Warning: Could not get file metadata for pre-validation");
                }
            }

            eprintln!("FFI Info: About to call API open_archive - Critical section starts here");

            // Android emulator safety check (all architectures)
            #[cfg(target_os = "android")]
            {
                if crate::platform::android::is_android_emulator() {
                    eprintln!("EMULATOR CRITICAL: Attempting archive open on Android emulator");
                    eprintln!("EMULATOR CRITICAL: sevenz_rust2 library has known crashes in emulated environments");
                    eprintln!(
                        "EMULATOR CRITICAL: Archive size: {} bytes, Path: {}",
                        std::fs::metadata(&path_buf).map(|m| m.len()).unwrap_or(0),
                        path_str
                    );
                    eprintln!("EMULATOR CRITICAL: Consider testing on real device if crash occurs");
                }

                // Additional file validation for Android
                if let Ok(metadata) = std::fs::metadata(&path_buf) {
                    eprintln!(
                        "ANDROID DEBUG: File size verification: {} bytes",
                        metadata.len()
                    );
                    if metadata.len() < 1000 {
                        eprintln!("ANDROID WARNING: Very small archive file - may trigger library edge cases");
                    }
                }
            }

            // Create the future first with extra debugging
            let archive_open_future = {
                eprintln!(
                    "FFI STEP 14: Creating archive open future - About to call api.open_archive()"
                );

                eprintln!("FFI STEP 14.1: Creating archive open future");
                api.open_archive(path_buf.clone(), password_str.to_string())
            };
            eprintln!("FFI STEP 15: Archive open future created successfully");

            eprintln!("FFI Info: Archive open future created, setting up timeout");
            let timeout_duration = std::time::Duration::from_secs(300); // 5 minute timeout

            eprintln!("FFI STEP 16: About to block on archive opening with timeout");

            // Final safety warning for Android emulators
            #[cfg(target_os = "android")]
            {
                if crate::platform::android::is_android_emulator() {
                    eprintln!("EMULATOR CRITICAL: Entering sevenz_rust2 library call zone");
                    eprintln!(
                        "EMULATOR CRITICAL: If SIGABRT occurs here, test on real Android device"
                    );
                    eprintln!(
                        "EMULATOR CRITICAL: Known issue affects both x86_64 and ARM emulators"
                    );
                }

                // Add timeout warning for small files
                if std::fs::metadata(&path_buf).map(|m| m.len()).unwrap_or(0) < 2000 {
                    eprintln!("ANDROID WARNING: Small archive file - using reduced timeout");
                }
            }

            match runtime.block_on(tokio::time::timeout(timeout_duration, archive_open_future)) {
                Ok(Ok(_)) => {
                    eprintln!("FFI Success: Archive opened successfully: {}", path_str);
                    #[cfg(target_os = "android")]
                    {
                        if crate::platform::android::is_android_emulator() {
                            eprintln!("EMULATOR SUCCESS: Archive opened successfully on Android emulator!");
                            eprintln!("EMULATOR SUCCESS: sevenz_rust2 library worked correctly in emulated environment");
                        } else {
                            eprintln!(
                                "ANDROID SUCCESS: Archive opened successfully on real device"
                            );
                        }
                    }
                    ZipLockError::Success as c_int
                }
                Ok(Err(e)) => {
                    let error_msg = format!("Failed to open archive {}: {}", path_str, e);
                    eprintln!("FFI Error: {}", error_msg);

                    // Enhanced error classification for better diagnostics
                    let error_str = e.to_string();
                    eprintln!("FFI Error Details: {}", error_str);

                    if error_str.contains("signature validation failed")
                        || error_str.contains("Invalid 7z signature")
                    {
                        eprintln!("FFI Error: Archive signature validation failed - file may not be a valid 7z archive");
                        set_last_error("Archive file does not have a valid 7z signature. The file may be corrupted or not a 7z archive.");
                        return ZipLockError::ArchiveCorrupted as c_int;
                    }

                    if error_str.contains("7z library operation panicked")
                        || error_str.contains("panicked")
                    {
                        eprintln!("FFI Error: 7z library panicked during extraction - likely corrupted archive");
                        set_last_error("Archive extraction failed due to internal library error. The archive may be corrupted or use an incompatible format.");
                        return ZipLockError::ArchiveCorrupted as c_int;
                    }

                    if error_str.contains("too small") || error_str.contains("empty") {
                        eprintln!("FFI Error: Archive is too small or empty");
                        set_last_error(
                            "Archive file is too small or empty to be a valid 7z archive.",
                        );
                        return ZipLockError::ArchiveCorrupted as c_int;
                    }

                    set_last_error(&error_msg);
                    match e {
                        crate::api::ApiError::Archive(archive_err) => match archive_err {
                            crate::archive::ArchiveError::NotFound { .. } => {
                                ZipLockError::ArchiveNotFound as c_int
                            }
                            crate::archive::ArchiveError::Corrupted { .. } => {
                                ZipLockError::ArchiveCorrupted as c_int
                            }
                            crate::archive::ArchiveError::OpenFailed { .. } => {
                                ZipLockError::InternalError as c_int
                            }
                            crate::archive::ArchiveError::ExtractFailed { .. } => {
                                ZipLockError::ArchiveCorrupted as c_int
                            }
                            crate::archive::ArchiveError::CryptoError { .. } => {
                                ZipLockError::InvalidPassword as c_int
                            }
                            _ => ZipLockError::InternalError as c_int,
                        },
                        crate::api::ApiError::InvalidCredentials => {
                            ZipLockError::InvalidPassword as c_int
                        }
                        crate::api::ApiError::AuthenticationRequired => {
                            ZipLockError::InvalidPassword as c_int
                        }
                        _ => ZipLockError::InternalError as c_int,
                    }
                }
                Err(_) => {
                    let error_msg = format!(
                        "Archive opening timed out after {} seconds for {}",
                        timeout_duration.as_secs(),
                        path_str
                    );
                    eprintln!("FFI Error: {}", error_msg);
                    set_last_error(&error_msg);
                    ZipLockError::InternalError as c_int
                }
            }
        }
    });

    eprintln!("FFI STEP 17: Panic catch block completed, handling result");

    // Handle any panics that occurred
    match result {
        Ok(return_code) => {
            eprintln!(
                "FFI Info: Archive open completed with code: {}",
                return_code
            );
            return_code
        }
        Err(panic_err) => {
            let error_msg = if let Some(s) = panic_err.downcast_ref::<String>() {
                format!("Archive opening panicked: {}", s)
            } else if let Some(s) = panic_err.downcast_ref::<&str>() {
                format!("Archive opening panicked: {}", s)
            } else {
                "Archive opening panicked with unknown error".to_string()
            };
            eprintln!("FFI PANIC CAUGHT: {}", error_msg);

            #[cfg(target_os = "android")]
            {
                if crate::platform::android::is_android_emulator() {
                    eprintln!("EMULATOR PANIC: sevenz_rust2 library crashed in Android emulator");
                    eprintln!("EMULATOR PANIC: This affects both x86_64 and ARM emulators");
                    eprintln!(
                        "EMULATOR PANIC: SOLUTION: Test archive operations on real Android device"
                    );
                    eprintln!("EMULATOR PANIC: WORKAROUND: Use emulator only for UI testing, skip archive ops");
                } else {
                    eprintln!("ANDROID PANIC: Unexpected crash on real device - check archive file integrity");
                }
            }

            unsafe {
                set_last_error(&error_msg);
            }
            ZipLockError::InternalError as c_int
        }
    }
}

/// Close the currently open archive
#[no_mangle]
pub extern "C" fn ziplock_archive_close() -> c_int {
    crate::log_debug!("FFI: ziplock_archive_close called");
    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        let api = match state.api.as_ref() {
            Some(api) => api,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let runtime = match state.runtime.as_ref() {
            Some(rt) => rt,
            None => return ZipLockError::InternalError as c_int,
        };

        match runtime.block_on(api.close_archive()) {
            Ok(_) => ZipLockError::Success as c_int,
            Err(_) => ZipLockError::InternalError as c_int,
        }
    }
}

/// Save the currently open archive
#[no_mangle]
pub extern "C" fn ziplock_archive_save() -> c_int {
    crate::log_debug!("FFI: ziplock_archive_save called");
    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        let api = match state.api.as_ref() {
            Some(api) => api,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let runtime = match state.runtime.as_ref() {
            Some(rt) => rt,
            None => return ZipLockError::InternalError as c_int,
        };

        match runtime.block_on(api.save_archive()) {
            Ok(_) => ZipLockError::Success as c_int,
            Err(_) => ZipLockError::InternalError as c_int,
        }
    }
}

/// List all credentials
#[no_mangle]
pub extern "C" fn ziplock_credential_list(
    credentials: *mut *mut CCredentialRecord,
    count: *mut usize,
) -> c_int {
    if credentials.is_null() || count.is_null() {
        return ZipLockError::InvalidParameter as c_int;
    }

    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        let api = match state.api.as_ref() {
            Some(api) => api,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let runtime = match state.runtime.as_ref() {
            Some(rt) => rt,
            None => return ZipLockError::InternalError as c_int,
        };

        let credential_list = match runtime.block_on(api.list_credentials()) {
            Ok(list) => list,
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        *count = credential_list.len();

        if credential_list.is_empty() {
            *credentials = ptr::null_mut();
            return ZipLockError::Success as c_int;
        }

        // Allocate array for C credential records
        let c_credentials =
            libc::malloc(credential_list.len() * std::mem::size_of::<CCredentialRecord>())
                as *mut CCredentialRecord;

        if c_credentials.is_null() {
            return ZipLockError::OutOfMemory as c_int;
        }

        // Convert each credential record
        for (i, credential) in credential_list.iter().enumerate() {
            let c_cred = c_credentials.add(i);
            if convert_credential_to_c(credential, c_cred).is_err() {
                // Clean up on error
                ziplock_credential_list_free(c_credentials, i);
                return ZipLockError::InternalError as c_int;
            }
        }

        *credentials = c_credentials;
        ZipLockError::Success as c_int
    }
}

/// Free credential list memory
#[no_mangle]
pub extern "C" fn ziplock_credential_list_free(credentials: *mut CCredentialRecord, count: usize) {
    if credentials.is_null() {
        return;
    }

    unsafe {
        for i in 0..count {
            let c_cred = credentials.add(i);
            free_c_credential(&mut *c_cred);
        }
        libc::free(credentials as *mut c_void);
    }
}

/// Create a new credential with simple string parameters
#[no_mangle]
pub extern "C" fn ziplock_credential_create_simple(
    title: *const c_char,
    credential_type: *const c_char,
    username: *const c_char,
    password: *const c_char,
    url: *const c_char,
    notes: *const c_char,
) -> c_int {
    crate::log_debug!("FFI: ziplock_credential_create_simple called");

    if title.is_null() || credential_type.is_null() {
        crate::log_error!("FFI: Invalid parameters - title or credential_type is null");
        unsafe {
            set_last_error("Invalid parameters: title and credential_type cannot be null");
        }
        return ZipLockError::InvalidParameter as c_int;
    }

    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => return ZipLockError::NotInitialized as c_int,
        };

        let state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => return ZipLockError::InternalError as c_int,
        };

        // Get the current API instance
        let api = match state.api.as_ref() {
            Some(api) => api,
            None => {
                set_last_error("API not initialized");
                return ZipLockError::NotInitialized as c_int;
            }
        };

        // Convert C strings to Rust strings
        let title_str = match CStr::from_ptr(title).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_last_error("Invalid UTF-8 in title");
                return ZipLockError::InvalidParameter as c_int;
            }
        };

        let credential_type_str = match CStr::from_ptr(credential_type).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_last_error("Invalid UTF-8 in credential_type");
                return ZipLockError::InvalidParameter as c_int;
            }
        };

        // Create a basic credential record
        let mut credential = CredentialRecord::new(title_str, credential_type_str);

        // Set optional fields if provided
        if !username.is_null() {
            if let Ok(username_str) = CStr::from_ptr(username).to_str() {
                credential.set_field(
                    "username",
                    crate::models::CredentialField::username(username_str),
                );
            }
        }

        if !password.is_null() {
            if let Ok(password_str) = CStr::from_ptr(password).to_str() {
                credential.set_field(
                    "password",
                    crate::models::CredentialField::password(password_str),
                );
            }
        }

        if !url.is_null() {
            if let Ok(url_str) = CStr::from_ptr(url).to_str() {
                credential.set_field("url", crate::models::CredentialField::url(url_str));
            }
        }

        if !notes.is_null() {
            if let Ok(notes_str) = CStr::from_ptr(notes).to_str() {
                credential.notes = Some(notes_str.to_string());
            }
        }

        // Create the credential using the API
        let runtime = match state.runtime.as_ref() {
            Some(rt) => rt,
            None => {
                set_last_error("Runtime not initialized");
                return ZipLockError::NotInitialized as c_int;
            }
        };

        match runtime.block_on(api.create_credential(credential)) {
            Ok(_) => ZipLockError::Success as c_int,
            Err(e) => {
                set_last_error(&format!("Failed to create credential: {}", e));
                ZipLockError::InternalError as c_int
            }
        }
    }
}

/// Free a string allocated by the native library
#[no_mangle]
pub extern "C" fn ziplock_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Get library version
#[no_mangle]
pub extern "C" fn ziplock_get_version() -> *mut c_char {
    match CString::new(crate::VERSION) {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Get the last error message
#[no_mangle]
pub extern "C" fn ziplock_get_last_error() -> *mut c_char {
    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => {
                // Return "Not initialized" error if FFI state doesn't exist
                return match CString::new("Library not initialized") {
                    Ok(cstring) => cstring.into_raw(),
                    Err(_) => ptr::null_mut(),
                };
            }
        };

        let state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => {
                return match CString::new("Failed to access error state") {
                    Ok(cstring) => cstring.into_raw(),
                    Err(_) => ptr::null_mut(),
                };
            }
        };

        match &state.last_error {
            Some(error) => match CString::new(error.as_str()) {
                Ok(cstring) => cstring.into_raw(),
                Err(_) => ptr::null_mut(),
            },
            None => match CString::new("No error") {
                Ok(cstring) => cstring.into_raw(),
                Err(_) => ptr::null_mut(),
            },
        }
    }
}

/// Set the last error message (internal helper function)
unsafe fn set_last_error(error_message: &str) {
    if let Some(state_mutex) = FFI_STATE.as_ref() {
        if let Ok(mut state) = state_mutex.lock() {
            state.last_error = Some(error_message.to_string());
        }
    }
}

/// Check if an archive is currently open
#[no_mangle]
pub extern "C" fn ziplock_is_archive_open() -> c_int {
    unsafe {
        let state_mutex = match FFI_STATE.as_ref() {
            Some(state) => state,
            None => return 0,
        };

        let state = match state_mutex.lock() {
            Ok(state) => state,
            Err(_) => return 0,
        };

        let api = match state.api.as_ref() {
            Some(api) => api,
            None => return 0,
        };

        let runtime = match state.runtime.as_ref() {
            Some(rt) => rt,
            None => return 0,
        };

        if runtime.block_on(api.is_archive_open()) {
            1
        } else {
            0
        }
    }
}

/// Helper function to convert CredentialRecord to C structure
fn convert_credential_to_c(
    credential: &CredentialRecord,
    c_cred: *mut CCredentialRecord,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Convert basic fields
        (*c_cred).id = CString::new(credential.id.clone())?.into_raw();
        (*c_cred).title = CString::new(credential.title.clone())?.into_raw();
        (*c_cred).credential_type = CString::new(credential.credential_type.clone())?.into_raw();

        (*c_cred).notes = if let Some(ref notes) = credential.notes {
            CString::new(notes.clone())?.into_raw()
        } else {
            ptr::null_mut()
        };

        (*c_cred).created_at = credential
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        (*c_cred).updated_at = credential
            .updated_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Convert fields
        (*c_cred).field_count = credential.fields.len();
        if !credential.fields.is_empty() {
            let fields =
                libc::malloc(credential.fields.len() * std::mem::size_of::<CCredentialField>())
                    as *mut CCredentialField;

            if fields.is_null() {
                return Err("Failed to allocate memory for fields".into());
            }

            for (i, (name, field)) in credential.fields.iter().enumerate() {
                let c_field = fields.add(i);
                (*c_field).name = CString::new(name.clone())?.into_raw();
                (*c_field).value = CString::new(field.value.clone())?.into_raw();
                (*c_field).field_type = CString::new(field.field_type.to_string())?.into_raw();
                (*c_field).label = if let Some(ref label) = field.label {
                    CString::new(label.clone())?.into_raw()
                } else {
                    ptr::null_mut()
                };
                (*c_field).sensitive = if field.sensitive { 1 } else { 0 };
                (*c_field).required = 0; // CredentialField doesn't have a required property, default to false
            }
            (*c_cred).fields = fields;
        } else {
            (*c_cred).fields = ptr::null_mut();
        }

        // Convert tags
        (*c_cred).tag_count = credential.tags.len();
        if !credential.tags.is_empty() {
            let tags = libc::malloc(credential.tags.len() * std::mem::size_of::<*mut c_char>())
                as *mut *mut c_char;

            if tags.is_null() {
                return Err("Failed to allocate memory for tags".into());
            }

            for (i, tag) in credential.tags.iter().enumerate() {
                let tag_ptr = tags.add(i);
                *tag_ptr = CString::new(tag.clone())?.into_raw();
            }
            (*c_cred).tags = tags;
        } else {
            (*c_cred).tags = ptr::null_mut();
        }

        Ok(())
    }
}

/// Helper function to free C credential structure
unsafe fn free_c_credential(c_cred: &mut CCredentialRecord) {
    if !c_cred.id.is_null() {
        let _ = CString::from_raw(c_cred.id);
    }
    if !c_cred.title.is_null() {
        let _ = CString::from_raw(c_cred.title);
    }
    if !c_cred.credential_type.is_null() {
        let _ = CString::from_raw(c_cred.credential_type);
    }
    if !c_cred.notes.is_null() {
        let _ = CString::from_raw(c_cred.notes);
    }

    // Free fields
    if !c_cred.fields.is_null() {
        for i in 0..c_cred.field_count {
            let field = c_cred.fields.add(i);
            if !(*field).name.is_null() {
                let _ = CString::from_raw((*field).name);
            }
            if !(*field).value.is_null() {
                let _ = CString::from_raw((*field).value);
            }
            if !(*field).field_type.is_null() {
                let _ = CString::from_raw((*field).field_type);
            }
            if !(*field).label.is_null() {
                let _ = CString::from_raw((*field).label);
            }
        }
        libc::free(c_cred.fields as *mut c_void);
    }

    // Free tags
    if !c_cred.tags.is_null() {
        for i in 0..c_cred.tag_count {
            let tag_ptr = c_cred.tags.add(i);
            if !(*tag_ptr).is_null() {
                let _ = CString::from_raw(*tag_ptr);
            }
        }
        libc::free(c_cred.tags as *mut c_void);
    }
}

/// Get all available credential templates
#[no_mangle]
pub extern "C" fn ziplock_templates_get_all(
    templates: *mut *mut CCredentialTemplate,
    count: *mut c_int,
) -> c_int {
    if templates.is_null() || count.is_null() {
        unsafe {
            set_last_error("Invalid parameters");
        }
        return ZipLockError::InvalidParameter as c_int;
    }

    let template_list = vec![
        crate::models::CommonTemplates::login(),
        crate::models::CommonTemplates::credit_card(),
        crate::models::CommonTemplates::secure_note(),
        crate::models::CommonTemplates::identity(),
        crate::models::CommonTemplates::password(),
        crate::models::CommonTemplates::document(),
        crate::models::CommonTemplates::ssh_key(),
        crate::models::CommonTemplates::bank_account(),
        crate::models::CommonTemplates::api_credentials(),
        crate::models::CommonTemplates::crypto_wallet(),
        crate::models::CommonTemplates::database(),
        crate::models::CommonTemplates::software_license(),
    ];

    let template_count = template_list.len();

    unsafe {
        let c_templates = libc::malloc(template_count * std::mem::size_of::<CCredentialTemplate>())
            as *mut CCredentialTemplate;

        if c_templates.is_null() {
            set_last_error("Out of memory");
            return ZipLockError::OutOfMemory as c_int;
        }

        for (i, template) in template_list.iter().enumerate() {
            let c_template = c_templates.add(i);
            if let Err(_) = convert_template_to_c(template, c_template) {
                // Clean up on error
                for j in 0..i {
                    free_c_template(&mut *c_templates.add(j));
                }
                libc::free(c_templates as *mut c_void);
                return ZipLockError::InternalError as c_int;
            }
        }

        *templates = c_templates;
        *count = template_count as c_int;
        ZipLockError::Success as c_int
    }
}

/// Get a specific credential template by name
#[no_mangle]
pub extern "C" fn ziplock_template_get_by_name(
    name: *const c_char,
    template: *mut CCredentialTemplate,
) -> c_int {
    if name.is_null() || template.is_null() {
        unsafe {
            set_last_error("Invalid parameters");
        }
        return ZipLockError::InvalidParameter as c_int;
    }

    let template_name = unsafe {
        match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error("Invalid template name string");
                return ZipLockError::InvalidParameter as c_int;
            }
        }
    };

    let rust_template = match template_name {
        "login" => crate::models::CommonTemplates::login(),
        "credit_card" => crate::models::CommonTemplates::credit_card(),
        "secure_note" => crate::models::CommonTemplates::secure_note(),
        "identity" => crate::models::CommonTemplates::identity(),
        "password" => crate::models::CommonTemplates::password(),
        "document" => crate::models::CommonTemplates::document(),
        "ssh_key" => crate::models::CommonTemplates::ssh_key(),
        "bank_account" => crate::models::CommonTemplates::bank_account(),
        "api_credentials" => crate::models::CommonTemplates::api_credentials(),
        "crypto_wallet" => crate::models::CommonTemplates::crypto_wallet(),
        "database" => crate::models::CommonTemplates::database(),
        "software_license" => crate::models::CommonTemplates::software_license(),
        _ => {
            unsafe {
                set_last_error("Unknown template name");
            }
            return ZipLockError::InvalidParameter as c_int;
        }
    };

    unsafe {
        match convert_template_to_c(&rust_template, template) {
            Ok(_) => ZipLockError::Success as c_int,
            Err(_) => ZipLockError::InternalError as c_int,
        }
    }
}

/// Free credential template list memory
#[no_mangle]
pub extern "C" fn ziplock_templates_free(templates: *mut CCredentialTemplate, count: c_int) {
    if templates.is_null() || count <= 0 {
        return;
    }

    unsafe {
        for i in 0..count {
            free_c_template(&mut *templates.add(i as usize));
        }
        libc::free(templates as *mut c_void);
    }
}

/// Free a single credential template
#[no_mangle]
pub extern "C" fn ziplock_template_free(template: *mut CCredentialTemplate) {
    if template.is_null() {
        return;
    }

    unsafe {
        // Free the template's memory
        let _ = Box::from_raw(template);
    }
}

// Android SAF function re-exports
#[cfg(target_os = "android")]
pub use crate::archive::{
    ziplock_android_saf_cleanup, ziplock_android_saf_init, ziplock_android_saf_is_available,
    ziplock_android_saf_test,
};

/// Convert a Rust CredentialTemplate to C structure
unsafe fn convert_template_to_c(
    template: &crate::models::CredentialTemplate,
    c_template: *mut CCredentialTemplate,
) -> Result<(), ()> {
    let name = match CString::new(template.name.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => return Err(()),
    };

    let description = match CString::new(template.description.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => {
            let _ = CString::from_raw(name);
            return Err(());
        }
    };

    // Convert fields
    let field_count = template.fields.len();
    let c_fields = if field_count > 0 {
        let fields_ptr = libc::malloc(field_count * std::mem::size_of::<CFieldTemplate>())
            as *mut CFieldTemplate;
        if fields_ptr.is_null() {
            let _ = CString::from_raw(name);
            let _ = CString::from_raw(description);
            return Err(());
        }

        for (i, field) in template.fields.iter().enumerate() {
            let c_field = fields_ptr.add(i);
            if convert_field_template_to_c(field, c_field).is_err() {
                // Clean up fields created so far
                for j in 0..i {
                    free_c_field_template(&mut *fields_ptr.add(j));
                }
                libc::free(fields_ptr as *mut c_void);
                let _ = CString::from_raw(name);
                let _ = CString::from_raw(description);
                return Err(());
            }
        }
        fields_ptr
    } else {
        ptr::null_mut()
    };

    // Convert tags
    let tag_count = template.default_tags.len();
    let c_tags = if tag_count > 0 {
        let tags_ptr =
            libc::malloc(tag_count * std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
        if tags_ptr.is_null() {
            if !c_fields.is_null() {
                for i in 0..field_count {
                    free_c_field_template(&mut *c_fields.add(i));
                }
                libc::free(c_fields as *mut c_void);
            }
            let _ = CString::from_raw(name);
            let _ = CString::from_raw(description);
            return Err(());
        }

        for (i, tag) in template.default_tags.iter().enumerate() {
            match CString::new(tag.clone()) {
                Ok(c_tag) => *tags_ptr.add(i) = c_tag.into_raw(),
                Err(_) => {
                    // Clean up tags created so far
                    for j in 0..i {
                        let _ = CString::from_raw(*tags_ptr.add(j));
                    }
                    libc::free(tags_ptr as *mut c_void);
                    if !c_fields.is_null() {
                        for k in 0..field_count {
                            free_c_field_template(&mut *c_fields.add(k));
                        }
                        libc::free(c_fields as *mut c_void);
                    }
                    let _ = CString::from_raw(name);
                    let _ = CString::from_raw(description);
                    return Err(());
                }
            }
        }

        tags_ptr
    } else {
        ptr::null_mut()
    };

    (*c_template).name = name;
    (*c_template).description = description;
    (*c_template).field_count = field_count as c_int;
    (*c_template).fields = c_fields;
    (*c_template).tag_count = tag_count as c_int;
    (*c_template).tags = c_tags;

    Ok(())
}

/// Convert a Rust FieldTemplate to C structure
unsafe fn convert_field_template_to_c(
    field: &crate::models::FieldTemplate,
    c_field: *mut CFieldTemplate,
) -> Result<(), ()> {
    let name = match CString::new(field.name.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => return Err(()),
    };

    let field_type = match CString::new(field.field_type.to_string()) {
        Ok(s) => s.into_raw(),
        Err(_) => {
            let _ = CString::from_raw(name);
            return Err(());
        }
    };

    let label = match CString::new(field.label.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => {
            let _ = CString::from_raw(name);
            let _ = CString::from_raw(field_type);
            return Err(());
        }
    };

    let default_value = if let Some(ref value) = field.default_value {
        match CString::new(value.clone()) {
            Ok(s) => s.into_raw(),
            Err(_) => {
                let _ = CString::from_raw(name);
                let _ = CString::from_raw(field_type);
                let _ = CString::from_raw(label);
                return Err(());
            }
        }
    } else {
        ptr::null_mut()
    };

    let (validation_pattern, validation_message, validation_min_length, validation_max_length) =
        if let Some(ref validation) = field.validation {
            let pattern = if let Some(ref pattern) = validation.pattern {
                match CString::new(pattern.clone()) {
                    Ok(s) => s.into_raw(),
                    Err(_) => {
                        let _ = CString::from_raw(name);
                        let _ = CString::from_raw(field_type);
                        let _ = CString::from_raw(label);
                        if !default_value.is_null() {
                            let _ = CString::from_raw(default_value);
                        }
                        return Err(());
                    }
                }
            } else {
                ptr::null_mut()
            };

            let message = if let Some(ref message) = validation.message {
                match CString::new(message.clone()) {
                    Ok(s) => s.into_raw(),
                    Err(_) => {
                        let _ = CString::from_raw(name);
                        let _ = CString::from_raw(field_type);
                        let _ = CString::from_raw(label);
                        if !default_value.is_null() {
                            let _ = CString::from_raw(default_value);
                        }
                        if !pattern.is_null() {
                            let _ = CString::from_raw(pattern);
                        }
                        return Err(());
                    }
                }
            } else {
                ptr::null_mut()
            };

            (
                pattern,
                message,
                validation.min_length.map(|v| v as c_int).unwrap_or(-1),
                validation.max_length.map(|v| v as c_int).unwrap_or(-1),
            )
        } else {
            (ptr::null_mut(), ptr::null_mut(), -1, -1)
        };

    (*c_field).name = name;
    (*c_field).field_type = field_type;
    (*c_field).label = label;
    (*c_field).required = if field.required { 1 } else { 0 };
    (*c_field).sensitive = if field.sensitive { 1 } else { 0 };
    (*c_field).default_value = default_value;
    (*c_field).validation_min_length = validation_min_length;
    (*c_field).validation_max_length = validation_max_length;
    (*c_field).validation_pattern = validation_pattern;
    (*c_field).validation_message = validation_message;

    Ok(())
}

/// Free a C credential template structure
unsafe fn free_c_template(c_template: &mut CCredentialTemplate) {
    if !c_template.name.is_null() {
        let _ = CString::from_raw(c_template.name);
    }
    if !c_template.description.is_null() {
        let _ = CString::from_raw(c_template.description);
    }

    // Free fields
    if !c_template.fields.is_null() {
        for i in 0..c_template.field_count {
            free_c_field_template(&mut *c_template.fields.add(i as usize));
        }
        libc::free(c_template.fields as *mut c_void);
    }

    // Free tags
    if !c_template.tags.is_null() {
        for i in 0..c_template.tag_count {
            let tag_ptr = c_template.tags.add(i as usize);
            if !(*tag_ptr).is_null() {
                let _ = CString::from_raw(*tag_ptr);
            }
        }
        libc::free(c_template.tags as *mut c_void);
    }
}

/// Free a C field template structure
unsafe fn free_c_field_template(c_field: &mut CFieldTemplate) {
    if !c_field.name.is_null() {
        let _ = CString::from_raw(c_field.name);
    }
    if !c_field.field_type.is_null() {
        let _ = CString::from_raw(c_field.field_type);
    }
    if !c_field.label.is_null() {
        let _ = CString::from_raw(c_field.label);
    }
    if !c_field.default_value.is_null() {
        let _ = CString::from_raw(c_field.default_value);
    }
    if !c_field.validation_pattern.is_null() {
        let _ = CString::from_raw(c_field.validation_pattern);
    }
    if !c_field.validation_message.is_null() {
        let _ = CString::from_raw(c_field.validation_message);
    }
}

// ============================================================================
// Logging Control Functions
// ============================================================================

/// Enable or disable debug logging
#[no_mangle]
pub extern "C" fn ziplock_logging_set_debug_enabled(enabled: c_int) -> c_int {
    let enabled_bool = enabled != 0;
    println!(
        "FFI: ziplock_logging_set_debug_enabled called with enabled={}",
        enabled_bool
    );

    match set_debug_enabled(enabled_bool) {
        Ok(()) => {
            if enabled_bool {
                crate::log_debug!("Debug logging has been enabled via FFI");
                crate::log_info!("Debug logging is now active");
            } else {
                println!("FFI: Debug logging has been disabled");
            }
            println!("FFI: Debug logging state change successful");
            ZipLockError::Success as c_int
        }
        Err(e) => {
            let error_msg = format!("Failed to set debug logging: {}", e);
            println!("FFI: Error setting debug logging: {}", error_msg);
            unsafe {
                set_last_error(&error_msg);
            }
            ZipLockError::InternalError as c_int
        }
    }
}

/// Check if debug logging is currently enabled
#[no_mangle]
pub extern "C" fn ziplock_logging_is_debug_enabled() -> c_int {
    if is_debug_enabled() {
        1
    } else {
        0
    }
}

/// Set logging configuration with detailed options
#[no_mangle]
pub extern "C" fn ziplock_logging_configure(
    debug_enabled: c_int,
    include_timestamps: c_int,
    include_thread_info: c_int,
    max_line_length: c_int,
) -> c_int {
    let config = LoggingConfig {
        debug_enabled: debug_enabled != 0,
        level: if debug_enabled != 0 {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        },
        include_timestamps: include_timestamps != 0,
        include_thread_info: include_thread_info != 0,
        include_spans: debug_enabled != 0,
        target_prefix: Some("ZipLock".to_string()),
        max_line_length: if max_line_length > 0 {
            Some(max_line_length as usize)
        } else {
            Some(1024)
        },
    };

    crate::log_info!(
        "Configuring logging: debug={}, timestamps={}, thread_info={}, max_length={}",
        config.debug_enabled,
        config.include_timestamps,
        config.include_thread_info,
        config.max_line_length.unwrap_or(0)
    );

    match crate::logging::configure_logging(config) {
        Ok(()) => {
            crate::log_debug!("Logging configuration updated successfully");
            ZipLockError::Success as c_int
        }
        Err(e) => {
            unsafe {
                set_last_error(&format!("Failed to configure logging: {}", e));
            }
            ZipLockError::InternalError as c_int
        }
    }
}

/// Log a test message at different levels (for debugging the logging system)
#[no_mangle]
pub extern "C" fn ziplock_logging_test(message: *const c_char) -> c_int {
    println!("FFI: ziplock_logging_test called");

    if message.is_null() {
        println!("FFI: Test message pointer is null");
        unsafe {
            set_last_error("Test message pointer is null");
        }
        return ZipLockError::InvalidParameter as c_int;
    }

    let c_str = unsafe { CStr::from_ptr(message) };
    let message_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            println!("FFI: Invalid UTF-8 in test message");
            unsafe {
                set_last_error("Invalid UTF-8 in test message");
            }
            return ZipLockError::InvalidParameter as c_int;
        }
    };

    println!("FFI: Testing logging with message: {}", message_str);

    crate::log_error!("TEST ERROR: {}", message_str);
    crate::log_warn!("TEST WARN: {}", message_str);
    crate::log_info!("TEST INFO: {}", message_str);
    crate::log_debug!("TEST DEBUG: {}", message_str);

    println!("FFI: All log levels tested");
    crate::log_info!("Logging test completed with message: {}", message_str);

    ZipLockError::Success as c_int
}

/// Check if running in Android emulator environment
///
/// This function exposes the centralized emulator detection to Android apps
/// through the FFI interface.
///
/// # Returns
///
/// 1 if running in an Android emulator, 0 otherwise
#[no_mangle]
pub extern "C" fn ziplock_is_android_emulator() -> c_int {
    if crate::platform::android::is_android_emulator() {
        1
    } else {
        0
    }
}

/// Get Android platform description
///
/// Returns a string describing the current Android platform environment.
/// The caller is responsible for freeing the returned string.
///
/// # Returns
///
/// Pointer to a C string describing the platform, or null on error
#[no_mangle]
pub extern "C" fn ziplock_get_android_platform_description() -> *mut c_char {
    let description = crate::platform::android::get_android_environment_description();

    match CString::new(description) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Check if current platform has known archive operation issues
///
/// # Returns
///
/// 1 if the platform has known issues, 0 otherwise
#[no_mangle]
pub extern "C" fn ziplock_has_archive_compatibility_issues() -> c_int {
    if crate::platform::has_archive_compatibility_issues() {
        1
    } else {
        0
    }
}

/// Get platform compatibility warning message
///
/// Returns a warning message if the current platform has compatibility issues,
/// or null if no issues are known. The caller is responsible for freeing the returned string.
///
/// # Returns
///
/// Pointer to a C string with warning message, or null if no warning
#[no_mangle]
pub extern "C" fn ziplock_get_platform_compatibility_warning() -> *mut c_char {
    match crate::platform::get_platform_compatibility_warning() {
        Some(warning) => match CString::new(warning) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod template_tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_get_all_templates() {
        unsafe {
            let mut templates: *mut CCredentialTemplate = ptr::null_mut();
            let mut count: c_int = 0;

            let result = ziplock_templates_get_all(&mut templates, &mut count);
            assert_eq!(result, ZipLockError::Success as c_int);
            assert!(!templates.is_null());
            assert_eq!(count, 12); // We have 12 built-in templates

            // Verify we can read the first template
            let first_template = &*templates;
            assert!(!first_template.name.is_null());

            let name = CStr::from_ptr(first_template.name).to_str().unwrap();
            assert!(!name.is_empty());

            // Clean up
            ziplock_templates_free(templates, count);
        }
    }

    #[test]
    fn test_get_template_by_name() {
        unsafe {
            let mut template = CCredentialTemplate {
                name: ptr::null_mut(),
                description: ptr::null_mut(),
                field_count: 0,
                fields: ptr::null_mut(),
                tag_count: 0,
                tags: ptr::null_mut(),
            };

            let template_name = CString::new("login").unwrap();
            let result = ziplock_template_get_by_name(template_name.as_ptr(), &mut template);
            assert_eq!(result, ZipLockError::Success as c_int);

            assert!(!template.name.is_null());
            let name = CStr::from_ptr(template.name).to_str().unwrap();
            assert_eq!(name, "login");

            assert!(!template.description.is_null());
            let description = CStr::from_ptr(template.description).to_str().unwrap();
            assert_eq!(description, "Website or application login");

            assert!(template.field_count > 0);
            assert!(!template.fields.is_null());

            // Clean up
            ziplock_template_free(&mut template);
        }
    }

    #[test]
    fn test_get_template_by_invalid_name() {
        unsafe {
            let mut template = CCredentialTemplate {
                name: ptr::null_mut(),
                description: ptr::null_mut(),
                field_count: 0,
                fields: ptr::null_mut(),
                tag_count: 0,
                tags: ptr::null_mut(),
            };

            let template_name = CString::new("invalid_template").unwrap();
            let result = ziplock_template_get_by_name(template_name.as_ptr(), &mut template);
            assert_eq!(result, ZipLockError::InvalidParameter as c_int);
        }
    }

    #[test]
    fn test_template_fields_structure() {
        unsafe {
            let mut template = CCredentialTemplate {
                name: ptr::null_mut(),
                description: ptr::null_mut(),
                field_count: 0,
                fields: ptr::null_mut(),
                tag_count: 0,
                tags: ptr::null_mut(),
            };

            let template_name = CString::new("credit_card").unwrap();
            let result = ziplock_template_get_by_name(template_name.as_ptr(), &mut template);
            assert_eq!(result, ZipLockError::Success as c_int);

            assert!(template.field_count >= 4); // Credit card should have at least 4 fields
            assert!(!template.fields.is_null());

            // Check first field
            let first_field = &*template.fields;
            assert!(!first_field.name.is_null());
            assert!(!first_field.field_type.is_null());
            assert!(!first_field.label.is_null());

            let field_name = CStr::from_ptr(first_field.name).to_str().unwrap();
            assert!(!field_name.is_empty());

            // Clean up
            ziplock_template_free(&mut template);
        }
    }

    #[test]
    fn test_all_template_names() {
        let template_names = vec![
            "login",
            "credit_card",
            "secure_note",
            "identity",
            "password",
            "document",
            "ssh_key",
            "bank_account",
            "api_credentials",
            "crypto_wallet",
            "database",
            "software_license",
        ];

        for template_name in template_names {
            unsafe {
                let mut template = CCredentialTemplate {
                    name: ptr::null_mut(),
                    description: ptr::null_mut(),
                    field_count: 0,
                    fields: ptr::null_mut(),
                    tag_count: 0,
                    tags: ptr::null_mut(),
                };

                let name_cstring = CString::new(template_name).unwrap();
                let result = ziplock_template_get_by_name(name_cstring.as_ptr(), &mut template);
                assert_eq!(
                    result,
                    ZipLockError::Success as c_int,
                    "Failed to get template: {}",
                    template_name
                );

                assert!(!template.name.is_null());
                let name = CStr::from_ptr(template.name).to_str().unwrap();
                assert_eq!(name, template_name);

                // Clean up
                ziplock_template_free(&mut template);
            }
        }
    }
}
