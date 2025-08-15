//! FFI (Foreign Function Interface) layer for ZipLock shared library
//!
//! This module provides C-compatible bindings for the ZipLock shared library,
//! enabling integration with mobile platforms (iOS, Android) and other languages
//! that can interface with C libraries.

#![allow(static_mut_refs)] // FFI requires static mut for C compatibility

use crate::api::{ApiSession, ZipLockApi};
use crate::archive::ArchiveConfig;
use crate::models::CredentialRecord;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

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
}

static mut FFI_STATE: Option<Mutex<FFIState>> = None;

/// Get access to the global FFI state (for internal use by client module)
pub unsafe fn get_ffi_state() -> Option<&'static Mutex<FFIState>> {
    FFI_STATE.as_ref()
}

/// Initialize the ZipLock library
#[no_mangle]
pub extern "C" fn ziplock_init() -> c_int {
    unsafe {
        if FFI_STATE.is_some() {
            return ZipLockError::AlreadyInitialized as c_int;
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
        };

        FFI_STATE = Some(Mutex::new(state));
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
    if path.is_null() || master_password.is_null() {
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

        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return ZipLockError::InvalidParameter as c_int,
        };

        let password_str = match CStr::from_ptr(master_password).to_str() {
            Ok(s) => s,
            Err(_) => return ZipLockError::InvalidParameter as c_int,
        };

        let path_buf = PathBuf::from(path_str);

        match runtime.block_on(api.create_archive(path_buf, password_str.to_string())) {
            Ok(_) => ZipLockError::Success as c_int,
            Err(_) => ZipLockError::InternalError as c_int,
        }
    }
}

/// Open an existing archive
#[no_mangle]
pub extern "C" fn ziplock_archive_open(
    path: *const c_char,
    master_password: *const c_char,
) -> c_int {
    if path.is_null() || master_password.is_null() {
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

        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return ZipLockError::InvalidParameter as c_int,
        };

        let password_str = match CStr::from_ptr(master_password).to_str() {
            Ok(s) => s,
            Err(_) => return ZipLockError::InvalidParameter as c_int,
        };

        let path_buf = PathBuf::from(path_str);

        match runtime.block_on(api.open_archive(path_buf, password_str.to_string())) {
            Ok(_) => ZipLockError::Success as c_int,
            Err(_) => ZipLockError::InternalError as c_int,
        }
    }
}

/// Close the current archive
#[no_mangle]
pub extern "C" fn ziplock_archive_close() -> c_int {
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

/// Save the current archive
#[no_mangle]
pub extern "C" fn ziplock_archive_save() -> c_int {
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

/// Free a C string allocated by the library
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
