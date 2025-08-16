/**
 * ZipLock C API Header
 * 
 * This header provides C-compatible functions for integrating ZipLock's
 * core functionality into mobile applications (iOS and Android).
 * 
 * Memory Management:
 * - All returned pointers must be freed using the appropriate *_free functions
 * - Strings are UTF-8 encoded and null-terminated
 * - Pass NULL for optional parameters
 * 
 * Error Handling:
 * - Functions return 0 on success, negative values on error
 * - Use ziplock_get_last_error() to get detailed error information
 * 
 * Thread Safety:
 * - All functions are thread-safe unless otherwise noted
 * - Credential objects should not be accessed concurrently
 */

#ifndef ZIPLOCK_H
#define ZIPLOCK_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

// ============================================================================
// Constants and Enums
// ============================================================================

// Error codes
typedef enum {
    ZIPLOCK_SUCCESS = 0,
    ZIPLOCK_ERROR_INVALID_POINTER = -1,
    ZIPLOCK_ERROR_INVALID_STRING = -2,
    ZIPLOCK_ERROR_INVALID_FIELD = -3,
    ZIPLOCK_ERROR_VALIDATION_FAILED = -4,
    ZIPLOCK_ERROR_SERIALIZATION_FAILED = -5,
    ZIPLOCK_ERROR_NOT_FOUND = -6,
    ZIPLOCK_ERROR_ALREADY_EXISTS = -7,
    ZIPLOCK_ERROR_INTERNAL = -8
} ziplock_error_t;

// Field types
typedef enum {
    ZIPLOCK_FIELD_TEXT = 0,
    ZIPLOCK_FIELD_PASSWORD = 1,
    ZIPLOCK_FIELD_EMAIL = 2,
    ZIPLOCK_FIELD_URL = 3,
    ZIPLOCK_FIELD_USERNAME = 4,
    ZIPLOCK_FIELD_PHONE = 5,
    ZIPLOCK_FIELD_CREDIT_CARD_NUMBER = 6,
    ZIPLOCK_FIELD_EXPIRY_DATE = 7,
    ZIPLOCK_FIELD_CVV = 8,
    ZIPLOCK_FIELD_TOTP_SECRET = 9,
    ZIPLOCK_FIELD_TEXT_AREA = 10,
    ZIPLOCK_FIELD_NUMBER = 11,
    ZIPLOCK_FIELD_DATE = 12,
    ZIPLOCK_FIELD_CUSTOM = 13
} ziplock_field_type_t;

// Password strength levels
typedef enum {
    ZIPLOCK_STRENGTH_VERY_WEAK = 0,
    ZIPLOCK_STRENGTH_WEAK = 1,
    ZIPLOCK_STRENGTH_FAIR = 2,
    ZIPLOCK_STRENGTH_GOOD = 3,
    ZIPLOCK_STRENGTH_STRONG = 4
} ziplock_password_strength_level_t;

// ============================================================================
// Data Structures
// ============================================================================

// Opaque handle types (actual structures defined in Rust)
typedef struct ziplock_credential ziplock_credential_t;
typedef struct ziplock_field ziplock_field_t;
typedef struct ziplock_search_result ziplock_search_result_t;
typedef struct ziplock_validation_result ziplock_validation_result_t;

// C-compatible structures for data exchange
typedef struct {
    char* id;
    char* title;
    char* credential_type;
    char* notes;
    uint32_t field_count;
    ziplock_field_t* fields;
    uint32_t tag_count;
    char** tags;
    int64_t created_at;    // Unix timestamp
    int64_t updated_at;    // Unix timestamp
} ziplock_credential_data_t;

typedef struct {
    char* name;
    ziplock_field_type_t field_type;
    char* value;
    char* label;
    int32_t sensitive;     // 0 = false, 1 = true
} ziplock_field_data_t;

typedef struct {
    ziplock_password_strength_level_t level;
    uint32_t score;        // 0-100
    char* description;
} ziplock_password_strength_t;

typedef struct {
    int32_t is_valid;      // 0 = false, 1 = true
    uint32_t error_count;
    char** errors;
} ziplock_validation_result_data_t;

typedef struct {
    uint32_t credential_count;
    ziplock_credential_data_t* credentials;
} ziplock_search_result_data_t;

// ============================================================================
// Library Management
// ============================================================================

/**
 * Initialize the ZipLock library
 * Must be called before using any other functions
 * 
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_init(void);

/**
 * Get the library version string
 * 
 * @return Version string (must be freed with ziplock_string_free)
 */
char* ziplock_get_version(void);

/**
 * Get the last error message
 * 
 * @return Error message string (must be freed with ziplock_string_free)
 */
char* ziplock_get_last_error(void);

// ============================================================================
// Memory Management
// ============================================================================

/**
 * Free a string allocated by the library
 * 
 * @param str String to free (can be NULL)
 */
void ziplock_string_free(char* str);

/**
 * Free a credential object
 * 
 * @param credential Credential to free (can be NULL)
 */
void ziplock_credential_free(ziplock_credential_t* credential);

/**
 * Free credential data structure
 * 
 * @param data Credential data to free (can be NULL)
 */
void ziplock_credential_data_free(ziplock_credential_data_t* data);

/**
 * Free password strength result
 * 
 * @param strength Password strength result to free (can be NULL)
 */
void ziplock_password_strength_free(ziplock_password_strength_t* strength);

/**
 * Free validation result
 * 
 * @param result Validation result to free (can be NULL)
 */
void ziplock_validation_result_free(ziplock_validation_result_data_t* result);

/**
 * Free search result
 * 
 * @param result Search result to free (can be NULL)
 */
void ziplock_search_result_free(ziplock_search_result_data_t* result);

// ============================================================================
// Credential Management
// ============================================================================

/**
 * Create a new credential
 * 
 * @param title Credential title (required)
 * @param credential_type Credential type (e.g., "login", "credit_card")
 * @return New credential object or NULL on error
 */
ziplock_credential_t* ziplock_credential_new(const char* title, const char* credential_type);

/**
 * Create a credential from a template
 * 
 * @param template_name Template name (e.g., "login", "credit_card", "secure_note")
 * @param title Credential title
 * @return New credential object or NULL on error
 */
ziplock_credential_t* ziplock_credential_from_template(const char* template_name, const char* title);

/**
 * Get credential data as a C structure
 * 
 * @param credential Credential object
 * @return Credential data (must be freed with ziplock_credential_data_free)
 */
ziplock_credential_data_t* ziplock_credential_get_data(const ziplock_credential_t* credential);

/**
 * Update credential from data structure
 * 
 * @param credential Credential object to update
 * @param data New credential data
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_credential_set_data(ziplock_credential_t* credential, const ziplock_credential_data_t* data);

/**
 * Add a field to a credential
 * 
 * @param credential Credential object
 * @param name Field name
 * @param field_type Field type
 * @param value Field value
 * @param label Field label (optional, can be NULL)
 * @param sensitive Whether field is sensitive (0 = false, 1 = true)
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_credential_add_field(
    ziplock_credential_t* credential,
    const char* name,
    ziplock_field_type_t field_type,
    const char* value,
    const char* label,
    int32_t sensitive
);

/**
 * Get a field value from a credential
 * 
 * @param credential Credential object
 * @param field_name Field name
 * @return Field value (must be freed with ziplock_string_free) or NULL if not found
 */
char* ziplock_credential_get_field(const ziplock_credential_t* credential, const char* field_name);

/**
 * Remove a field from a credential
 * 
 * @param credential Credential object
 * @param field_name Field name
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_credential_remove_field(ziplock_credential_t* credential, const char* field_name);

/**
 * Add a tag to a credential
 * 
 * @param credential Credential object
 * @param tag Tag to add
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_credential_add_tag(ziplock_credential_t* credential, const char* tag);

/**
 * Remove a tag from a credential
 * 
 * @param credential Credential object
 * @param tag Tag to remove
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_credential_remove_tag(ziplock_credential_t* credential, const char* tag);

/**
 * Check if a credential has a specific tag
 * 
 * @param credential Credential object
 * @param tag Tag to check
 * @return 1 if has tag, 0 if not, negative error code on failure
 */
int32_t ziplock_credential_has_tag(const ziplock_credential_t* credential, const char* tag);

// ============================================================================
// Validation
// ============================================================================

/**
 * Validate a credential
 * 
 * @param credential Credential to validate
 * @return Validation result (must be freed with ziplock_validation_result_free)
 */
ziplock_validation_result_data_t* ziplock_credential_validate(const ziplock_credential_t* credential);

/**
 * Validate a password and get strength information
 * 
 * @param password Password to validate
 * @return Password strength result (must be freed with ziplock_password_strength_free)
 */
ziplock_password_strength_t* ziplock_password_validate(const char* password);

/**
 * Validate an email address
 * 
 * @param email Email address to validate
 * @return 1 if valid, 0 if invalid
 */
int32_t ziplock_email_validate(const char* email);

/**
 * Validate a URL
 * 
 * @param url URL to validate
 * @return 1 if valid, 0 if invalid
 */
int32_t ziplock_url_validate(const char* url);

// ============================================================================
// Password Generation
// ============================================================================

/**
 * Generate a secure password
 * 
 * @param length Password length (1-256)
 * @param include_uppercase Include uppercase letters (0 = false, 1 = true)
 * @param include_lowercase Include lowercase letters (0 = false, 1 = true)
 * @param include_numbers Include numbers (0 = false, 1 = true)
 * @param include_symbols Include symbols (0 = false, 1 = true)
 * @return Generated password (must be freed with ziplock_string_free) or NULL on error
 */
char* ziplock_password_generate(
    uint32_t length,
    int32_t include_uppercase,
    int32_t include_lowercase,
    int32_t include_numbers,
    int32_t include_symbols
);

// ============================================================================
// Search and Utilities
// ============================================================================

/**
 * Search credentials by query string
 * 
 * @param credentials Array of credentials to search
 * @param credential_count Number of credentials in array
 * @param query Search query
 * @return Search result (must be freed with ziplock_search_result_free)
 */
ziplock_search_result_data_t* ziplock_credentials_search(
    const ziplock_credential_data_t* credentials,
    uint32_t credential_count,
    const char* query
);

/**
 * Format a credit card number for display (mask middle digits)
 * 
 * @param card_number Credit card number
 * @return Formatted card number (must be freed with ziplock_string_free)
 */
char* ziplock_credit_card_format(const char* card_number);

/**
 * Generate a TOTP code from a secret
 * 
 * @param secret Base32-encoded TOTP secret
 * @param time_step Time step in seconds (typically 30)
 * @return 6-digit TOTP code (must be freed with ziplock_string_free) or NULL on error
 */
char* ziplock_totp_generate(const char* secret, uint32_t time_step);

// ============================================================================
// Testing and Debugging
// ============================================================================

/**
 * Test function to verify FFI is working correctly
 * 
 * @param input Test input string
 * @return Echo response (must be freed with ziplock_string_free)
 */
char* ziplock_test_echo(const char* input);

/**
 * Enable or disable debug logging
 * 
 * @param enabled 1 to enable, 0 to disable
 * @return 0 on success, negative error code on failure
 */
int32_t ziplock_debug_logging(int32_t enabled);

#ifdef __cplusplus
}
#endif

#endif // ZIPLOCK_H