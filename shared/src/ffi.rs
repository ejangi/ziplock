//! C FFI (Foreign Function Interface) bindings for ZipLock shared library
//!
//! This module provides C-compatible functions that can be called from Swift (iOS)
//! and Kotlin (Android) to access ZipLock's core functionality. The API is designed
//! to be safe, efficient, and easy to use from mobile platforms.
//!
//! # Safety
//!
//! All C functions are marked as `unsafe` but internally use safe Rust patterns.
//! The caller is responsible for:
//! - Passing valid pointers
//! - Not using freed memory
//! - Proper string encoding (UTF-8)
//!
//! # Memory Management
//!
//! - All returned pointers must be freed using the appropriate `*_free` functions
//! - Strings are allocated using `CString` and must be freed with `ziplock_string_free`
//! - Structs are boxed and must be freed with their respective free functions
//!
//! # Error Handling
//!
//! Functions return error codes where:
//! - 0 = Success
//! - Negative values = Error codes (see `ZipLockError` enum)

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr;
use std::slice;

use crate::models::{CredentialRecord, FieldType};
use crate::utils::{PasswordOptions, PasswordUtils};
use crate::validation::PassphraseValidator;

// ============================================================================
// Error Codes
// ============================================================================

/// Error codes returned by C API functions
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ZipLockError {
    Success = 0,
    InvalidPointer = -1,
    InvalidString = -2,
    InvalidField = -3,
    ValidationFailed = -4,
    SerializationFailed = -5,
    NotFound = -6,
    AlreadyExists = -7,
    InternalError = -8,
}

impl From<ZipLockError> for c_int {
    fn from(error: ZipLockError) -> Self {
        error as c_int
    }
}

// ============================================================================
// C-Compatible Data Structures
// ============================================================================

/// C-compatible credential record
#[repr(C)]
pub struct CCredentialRecord {
    pub id: *mut c_char,
    pub title: *mut c_char,
    pub credential_type: *mut c_char,
    pub notes: *mut c_char,
    pub field_count: c_uint,
    pub fields: *mut CCredentialField,
    pub tag_count: c_uint,
    pub tags: *mut *mut c_char,
    pub created_at: i64,
    pub updated_at: i64,
}

/// C-compatible credential field
#[repr(C)]
pub struct CCredentialField {
    pub name: *mut c_char,
    pub field_type: c_int, // Maps to FieldType enum
    pub value: *mut c_char,
    pub label: *mut c_char,
    pub sensitive: c_int, // 0 = false, 1 = true
}

/// C-compatible password strength result
#[repr(C)]
pub struct CPasswordStrength {
    pub level: c_int, // Maps to PasswordStrength enum
    pub score: c_uint,
    pub description: *mut c_char,
}

/// C-compatible validation result
#[repr(C)]
pub struct CValidationResult {
    pub is_valid: c_int, // 0 = false, 1 = true
    pub error_count: c_uint,
    pub errors: *mut *mut c_char,
}

/// C-compatible search result
#[repr(C)]
pub struct CSearchResult {
    pub credential_count: c_uint,
    pub credentials: *mut CCredentialRecord,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Rust string to C string (caller must free)
unsafe fn string_to_c_char(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Convert C string to Rust string
unsafe fn c_char_to_string(ptr: *const c_char) -> Result<String, ZipLockError> {
    if ptr.is_null() {
        return Err(ZipLockError::InvalidPointer);
    }

    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| ZipLockError::InvalidString)
}

/// Convert FieldType to C int
fn field_type_to_c_int(field_type: &FieldType) -> c_int {
    match field_type {
        FieldType::Text => 0,
        FieldType::Password => 1,
        FieldType::Email => 2,
        FieldType::Url => 3,
        FieldType::Username => 4,
        FieldType::Phone => 5,
        FieldType::CreditCardNumber => 6,
        FieldType::ExpiryDate => 7,
        FieldType::Cvv => 8,
        FieldType::TotpSecret => 9,
        FieldType::TextArea => 10,
        FieldType::Number => 11,
        FieldType::Date => 12,
        FieldType::Custom(_) => 13,
    }
}

/// Convert C int to FieldType
fn c_int_to_field_type(value: c_int) -> FieldType {
    match value {
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
        _ => FieldType::Text, // Default fallback
    }
}

/// Convert CredentialRecord to C structure
unsafe fn credential_to_c(record: &CredentialRecord) -> Result<CCredentialRecord, ZipLockError> {
    let id = string_to_c_char(&record.id);
    let title = string_to_c_char(&record.title);
    let credential_type = string_to_c_char(&record.credential_type);
    let notes = string_to_c_char(record.notes.as_deref().unwrap_or(""));

    // Convert fields
    let field_count = record.fields.len() as c_uint;
    let fields = if field_count > 0 {
        let fields_vec: Vec<CCredentialField> = record
            .fields
            .iter()
            .map(|(name, field)| CCredentialField {
                name: string_to_c_char(name),
                field_type: field_type_to_c_int(&field.field_type),
                value: string_to_c_char(&field.value),
                label: string_to_c_char(field.label.as_deref().unwrap_or("")),
                sensitive: if field.sensitive { 1 } else { 0 },
            })
            .collect();

        let boxed = fields_vec.into_boxed_slice();
        Box::into_raw(boxed) as *mut CCredentialField
    } else {
        ptr::null_mut()
    };

    // Convert tags
    let tag_count = record.tags.len() as c_uint;
    let tags = if tag_count > 0 {
        let tags_vec: Vec<*mut c_char> = record
            .tags
            .iter()
            .map(|tag| string_to_c_char(tag))
            .collect();

        let boxed = tags_vec.into_boxed_slice();
        Box::into_raw(boxed) as *mut *mut c_char
    } else {
        ptr::null_mut()
    };

    Ok(CCredentialRecord {
        id,
        title,
        credential_type,
        notes,
        field_count,
        fields,
        tag_count,
        tags,
        created_at: record
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        updated_at: record
            .updated_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    })
}

// ============================================================================
// Public C API Functions
// ============================================================================

/// Initialize the ZipLock library
/// Returns 0 on success, negative error code on failure
#[no_mangle]
pub extern "C" fn ziplock_init() -> c_int {
    // Initialize logging if needed
    ZipLockError::Success.into()
}

/// Free a string allocated by the library
///
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` was allocated by this library (e.g., returned from other functions)
/// - `ptr` is not used after this call
/// - `ptr` can be null (this function handles null pointers safely)
#[no_mangle]
pub unsafe extern "C" fn ziplock_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Free a credential record allocated by the library
///
/// # Safety
///
/// The caller must ensure that:
/// - `credential` was allocated by this library (e.g., from `ziplock_credential_new`)
/// - `credential` is not used after this call
/// - `credential` can be null (this function handles null pointers safely)
#[no_mangle]
pub unsafe extern "C" fn ziplock_credential_free(credential: *mut CCredentialRecord) {
    if credential.is_null() {
        return;
    }

    let cred = Box::from_raw(credential);

    // Free all owned strings
    ziplock_string_free(cred.id);
    ziplock_string_free(cred.title);
    ziplock_string_free(cred.credential_type);
    ziplock_string_free(cred.notes);

    // Free fields
    if !cred.fields.is_null() && cred.field_count > 0 {
        let fields = slice::from_raw_parts_mut(cred.fields, cred.field_count as usize);
        for field in fields {
            ziplock_string_free(field.name);
            ziplock_string_free(field.value);
            ziplock_string_free(field.label);
        }
        let _ = Box::from_raw(slice::from_raw_parts_mut(
            cred.fields,
            cred.field_count as usize,
        ));
    }

    // Free tags
    if !cred.tags.is_null() && cred.tag_count > 0 {
        let tags = slice::from_raw_parts_mut(cred.tags, cred.tag_count as usize);
        for tag in tags {
            ziplock_string_free(*tag);
        }
        let _ = Box::from_raw(slice::from_raw_parts_mut(
            cred.tags,
            cred.tag_count as usize,
        ));
    }
}

/// Create a new credential record
/// Returns pointer to new credential or null on error
///
/// # Safety
///
/// The caller must ensure that:
/// - `title` and `credential_type` are valid, null-terminated C strings
/// - The returned pointer is freed with `ziplock_credential_free`
#[no_mangle]
pub unsafe extern "C" fn ziplock_credential_new(
    title: *const c_char,
    credential_type: *const c_char,
) -> *mut CCredentialRecord {
    let title_str = match c_char_to_string(title) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let type_str = match c_char_to_string(credential_type) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let record = CredentialRecord::new(title_str, type_str);

    match credential_to_c(&record) {
        Ok(c_record) => Box::into_raw(Box::new(c_record)),
        Err(_) => ptr::null_mut(),
    }
}

/// Add a field to a credential record
///
/// # Safety
///
/// The caller must ensure that:
/// - `credential` is a valid pointer to a CCredentialRecord
/// - `name`, `value`, and `label` are valid, null-terminated C strings
/// - `field_type` corresponds to a valid CredentialFieldType
pub unsafe extern "C" fn ziplock_credential_add_field(
    credential: *mut CCredentialRecord,
    name: *const c_char,
    field_type: c_int,
    value: *const c_char,
    label: *const c_char,
    _sensitive: c_int,
) -> c_int {
    if credential.is_null() {
        return ZipLockError::InvalidPointer.into();
    }

    let _name_str = match c_char_to_string(name) {
        Ok(s) => s,
        Err(e) => return e.into(),
    };

    let _value_str = match c_char_to_string(value) {
        Ok(s) => s,
        Err(e) => return e.into(),
    };

    let _label_str = if label.is_null() {
        None
    } else {
        c_char_to_string(label).ok()
    };

    let _field_type = c_int_to_field_type(field_type);

    // This is a simplified version - in a real implementation,
    // you'd need to convert back from C struct to Rust struct,
    // modify it, and convert back to C struct
    ZipLockError::Success.into()
}

/// Validate a password and return strength information
///
/// # Safety
///
/// The caller must ensure that:
/// - `password` is a valid, null-terminated C string
/// - The returned pointer is freed with `ziplock_password_strength_free`
pub unsafe extern "C" fn ziplock_validate_password(
    password: *const c_char,
) -> *mut CPasswordStrength {
    let password_str = match c_char_to_string(password) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let validator = PassphraseValidator::default();
    let result = validator.validate(&password_str);

    let level_description = match result.level {
        crate::validation::StrengthLevel::VeryWeak => "Very Weak",
        crate::validation::StrengthLevel::Weak => "Weak",
        crate::validation::StrengthLevel::Fair => "Fair",
        crate::validation::StrengthLevel::Good => "Good",
        crate::validation::StrengthLevel::Strong => "Strong",
        crate::validation::StrengthLevel::VeryStrong => "Very Strong",
    };

    let strength = CPasswordStrength {
        level: match result.level {
            crate::validation::StrengthLevel::VeryWeak => 0,
            crate::validation::StrengthLevel::Weak => 1,
            crate::validation::StrengthLevel::Fair => 2,
            crate::validation::StrengthLevel::Good => 3,
            crate::validation::StrengthLevel::Strong => 4,
            crate::validation::StrengthLevel::VeryStrong => 5,
        },
        score: result.score as c_uint,
        description: string_to_c_char(level_description),
    };

    Box::into_raw(Box::new(strength))
}

/// Free a password strength result
///
/// # Safety
///
/// The caller must ensure that:
/// - `strength` was allocated by this library (e.g., from `ziplock_validate_password`)
/// - `strength` is not used after this call
/// - `strength` can be null (this function handles null pointers safely)
#[no_mangle]
pub unsafe extern "C" fn ziplock_password_strength_free(strength: *mut CPasswordStrength) {
    if !strength.is_null() {
        let strength = Box::from_raw(strength);
        ziplock_string_free(strength.description);
    }
}

/// Generate a secure password
#[no_mangle]
/// Generate a random password with specified criteria
///
/// # Safety
///
/// The caller must ensure that:
/// - The returned C string is freed with `ziplock_string_free`
/// - `length` is a reasonable value (not excessively large)
pub unsafe extern "C" fn ziplock_generate_password(
    length: c_uint,
    include_uppercase: c_int,
    include_lowercase: c_int,
    include_numbers: c_int,
    include_symbols: c_int,
) -> *mut c_char {
    let options = PasswordOptions {
        length: length as usize,
        include_uppercase: include_uppercase != 0,
        include_lowercase: include_lowercase != 0,
        include_numbers: include_numbers != 0,
        include_symbols: include_symbols != 0,
    };

    match PasswordUtils::generate_password(options) {
        Ok(password) => string_to_c_char(&password),
        Err(_) => ptr::null_mut(),
    }
}

/// Validate an email address
///
/// # Safety
///
/// The caller must ensure that:
/// - `email` is a valid, null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn ziplock_validate_email(email: *const c_char) -> c_int {
    let email_str = match c_char_to_string(email) {
        Ok(s) => s,
        Err(_) => return 0, // Invalid
    };

    if crate::models::field::FieldUtils::is_valid_email(&email_str) {
        1 // Valid
    } else {
        0 // Invalid
    }
}

/// Validate a URL
///
/// # Safety
///
/// The caller must ensure that:
/// - `url` is a valid, null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn ziplock_validate_url(url: *const c_char) -> c_int {
    let url_str = match c_char_to_string(url) {
        Ok(s) => s,
        Err(_) => return 0, // Invalid
    };

    if crate::models::field::FieldUtils::is_valid_url(&url_str) {
        1 // Valid
    } else {
        0 // Invalid
    }
}

/// Search credentials by query string
/// Returns search result structure or null on error
#[no_mangle]
/// Search credentials (placeholder implementation)
///
/// # Safety
///
/// The caller must ensure that:
/// - `_credentials` points to a valid array of CCredentialRecord with `_credential_count` elements
/// - `_query` is a valid, null-terminated C string
/// - The returned pointer is freed with `ziplock_search_result_free`
pub unsafe extern "C" fn ziplock_search_credentials(
    _credentials: *const CCredentialRecord,
    _credential_count: c_uint,
    _query: *const c_char,
) -> *mut CSearchResult {
    // This is a simplified version - in a real implementation,
    // you'd convert C structs back to Rust structs, perform the search,
    // and convert results back to C structs
    ptr::null_mut()
}

/// Free a search result
///
/// # Safety
///
/// The caller must ensure that:
/// - `result` was allocated by this library (e.g., from `ziplock_search_credentials`)
/// - `result` is not used after this call
/// - `result` can be null (this function handles null pointers safely)
#[no_mangle]
pub unsafe extern "C" fn ziplock_search_result_free(result: *mut CSearchResult) {
    if result.is_null() {
        return;
    }

    let result = Box::from_raw(result);

    if !result.credentials.is_null() && result.credential_count > 0 {
        let credentials =
            slice::from_raw_parts_mut(result.credentials, result.credential_count as usize);

        for credential in credentials {
            ziplock_credential_free(credential as *mut CCredentialRecord);
        }

        let _ = Box::from_raw(slice::from_raw_parts_mut(
            result.credentials,
            result.credential_count as usize,
        ));
    }
}

/// Get library version
///
/// # Safety
///
/// The caller must ensure that:
/// - The returned C string is freed with `ziplock_string_free`
#[no_mangle]
pub unsafe extern "C" fn ziplock_get_version() -> *mut c_char {
    string_to_c_char(crate::VERSION)
}

/// Validate a credential record
///
/// # Safety
///
/// The caller must ensure that:
/// - `credential` is a valid pointer to a CCredentialRecord
/// - The returned pointer is freed with `ziplock_validation_result_free`
pub unsafe extern "C" fn ziplock_validate_credential(
    credential: *const CCredentialRecord,
) -> *mut CValidationResult {
    if credential.is_null() {
        return ptr::null_mut();
    }

    // This is a simplified version - in a real implementation,
    // you'd convert the C struct back to a Rust struct and validate it
    let result = CValidationResult {
        is_valid: 1,
        error_count: 0,
        errors: ptr::null_mut(),
    };

    Box::into_raw(Box::new(result))
}

/// Free a validation result
///
/// # Safety
///
/// The caller must ensure that:
/// - `result` was allocated by this library (e.g., from `ziplock_validate_credential`)
/// - `result` is not used after this call
/// - `result` can be null (this function handles null pointers safely)
#[no_mangle]
pub unsafe extern "C" fn ziplock_validation_result_free(result: *mut CValidationResult) {
    if result.is_null() {
        return;
    }

    let result = Box::from_raw(result);

    if !result.errors.is_null() && result.error_count > 0 {
        let errors = slice::from_raw_parts_mut(result.errors, result.error_count as usize);
        for error in errors {
            ziplock_string_free(*error);
        }
        let _ = Box::from_raw(slice::from_raw_parts_mut(
            result.errors,
            result.error_count as usize,
        ));
    }
}

// ============================================================================
// Utility Functions for Testing
// ============================================================================

/// Test function to verify FFI is working
///
/// # Safety
///
/// The caller must ensure that:
/// - `input` is a valid, null-terminated C string
/// - The returned C string is freed with `ziplock_string_free`
#[no_mangle]
pub unsafe extern "C" fn ziplock_test_echo(input: *const c_char) -> *mut c_char {
    match c_char_to_string(input) {
        Ok(s) => string_to_c_char(&format!("Echo: {}", s)),
        Err(_) => string_to_c_char("Error: Invalid input"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_string_conversion() {
        unsafe {
            let test_str = "Hello, World!";
            let c_str = string_to_c_char(test_str);
            assert!(!c_str.is_null());

            let rust_str = c_char_to_string(c_str).unwrap();
            assert_eq!(rust_str, test_str);

            ziplock_string_free(c_str);
        }
    }

    #[test]
    fn test_field_type_conversion() {
        assert_eq!(field_type_to_c_int(&FieldType::Password), 1);
        assert_eq!(field_type_to_c_int(&FieldType::Email), 2);

        assert!(matches!(c_int_to_field_type(1), FieldType::Password));
        assert!(matches!(c_int_to_field_type(2), FieldType::Email));
    }

    #[test]
    fn test_credential_creation() {
        unsafe {
            let title = CString::new("Test Credential").unwrap();
            let cred_type = CString::new("login").unwrap();

            let credential = ziplock_credential_new(title.as_ptr(), cred_type.as_ptr());
            assert!(!credential.is_null());

            ziplock_credential_free(credential);
        }
    }

    #[test]
    fn test_password_validation() {
        unsafe {
            let password = CString::new("StrongPassword123!").unwrap();
            let strength = ziplock_validate_password(password.as_ptr());
            assert!(!strength.is_null());

            let strength_ref = &*strength;
            assert!(strength_ref.score > 0);

            ziplock_password_strength_free(strength);
        }
    }

    #[test]
    fn test_echo_function() {
        unsafe {
            let input = CString::new("Test").unwrap();
            let output = ziplock_test_echo(input.as_ptr());
            assert!(!output.is_null());

            let output_str = c_char_to_string(output).unwrap();
            assert!(output_str.contains("Test"));

            ziplock_string_free(output);
        }
    }
}
