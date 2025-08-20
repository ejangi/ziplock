#ifndef ZIPLOCK_HYBRID_H
#define ZIPLOCK_HYBRID_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

/**
 * ZipLock Hybrid FFI Interface
 * 
 * This header provides a simplified C interface for ZipLock that focuses on
 * data validation, cryptography, and business logic operations. Archive
 * operations are handled by native platform code (e.g., Kotlin on Android).
 * 
 * Key Features:
 * - Credential data management (no archive I/O)
 * - Password generation and validation
 * - Cryptographic operations
 * - Field validation (email, URL, phone)
 * - JSON serialization for data interchange
 * 
 * This eliminates the sevenz_rust2 dependency that causes Android emulator crashes.
 */

/**
 * Error codes for hybrid FFI operations
 */
typedef enum {
    ZIPLOCK_HYBRID_SUCCESS = 0,
    ZIPLOCK_HYBRID_INVALID_PARAMETER = 1,
    ZIPLOCK_HYBRID_NOT_INITIALIZED = 2,
    ZIPLOCK_HYBRID_ALREADY_INITIALIZED = 3,
    ZIPLOCK_HYBRID_CREDENTIAL_NOT_FOUND = 4,
    ZIPLOCK_HYBRID_VALIDATION_FAILED = 5,
    ZIPLOCK_HYBRID_CRYPTO_ERROR = 6,
    ZIPLOCK_HYBRID_OUT_OF_MEMORY = 7,
    ZIPLOCK_HYBRID_INTERNAL_ERROR = 8,
    ZIPLOCK_HYBRID_SERIALIZATION_ERROR = 9,
    ZIPLOCK_HYBRID_JSON_PARSE_ERROR = 10
} ZipLockHybridError;

/**
 * Field types for credentials
 */
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
} ZipLockFieldType;

/**
 * Library Management Functions
 */

/**
 * Initialize the hybrid FFI library
 * Must be called before any other functions
 * 
 * @return ZIPLOCK_HYBRID_SUCCESS on success, error code on failure
 */
int ziplock_hybrid_init(void);

/**
 * Get library version string
 * 
 * @return Version string (caller must free with ziplock_hybrid_string_free)
 */
char* ziplock_hybrid_get_version(void);

/**
 * Get last error message
 * 
 * @return Error message (caller must free with ziplock_hybrid_string_free)
 */
char* ziplock_hybrid_get_last_error(void);

/**
 * Cleanup and shutdown the hybrid FFI library
 * 
 * @return ZIPLOCK_HYBRID_SUCCESS on success
 */
int ziplock_hybrid_cleanup(void);

/**
 * Credential Management Functions
 */

/**
 * Create a new credential
 * 
 * @param title Credential title
 * @param credential_type Type of credential (e.g., "login", "note")
 * @return Credential ID (0 on failure)
 */
uint64_t ziplock_hybrid_credential_new(const char* title, const char* credential_type);

/**
 * Add a field to a credential
 * 
 * @param credential_id Credential ID
 * @param name Field name
 * @param field_type Field type (ZipLockFieldType)
 * @param value Field value
 * @param label Optional field label (can be NULL)
 * @param sensitive 1 if field is sensitive, 0 otherwise
 * @return ZIPLOCK_HYBRID_SUCCESS on success, error code on failure
 */
int ziplock_hybrid_credential_add_field(
    uint64_t credential_id,
    const char* name,
    int field_type,
    const char* value,
    const char* label,
    int sensitive
);

/**
 * Get a field value from a credential
 * 
 * @param credential_id Credential ID
 * @param name Field name
 * @return Field value (caller must free with ziplock_hybrid_string_free), NULL if not found
 */
char* ziplock_hybrid_credential_get_field(uint64_t credential_id, const char* name);

/**
 * Convert credential to JSON string
 * 
 * @param credential_id Credential ID
 * @return JSON string (caller must free with ziplock_hybrid_string_free), NULL on failure
 */
char* ziplock_hybrid_credential_to_json(uint64_t credential_id);

/**
 * Create credential from JSON string
 * 
 * @param json JSON string
 * @return Credential ID (0 on failure)
 */
uint64_t ziplock_hybrid_credential_from_json(const char* json);

/**
 * Validate a credential
 * 
 * @param credential_id Credential ID
 * @return 1 if valid, 0 if invalid
 */
int ziplock_hybrid_credential_validate(uint64_t credential_id);

/**
 * Free a credential and its resources
 * 
 * @param credential_id Credential ID to free
 */
void ziplock_hybrid_credential_free(uint64_t credential_id);

/**
 * Password Functions
 */

/**
 * Generate a secure password
 * 
 * @param length Password length (1-256)
 * @param uppercase Include uppercase letters (1=yes, 0=no)
 * @param lowercase Include lowercase letters (1=yes, 0=no)
 * @param numbers Include numbers (1=yes, 0=no)
 * @param symbols Include symbols (1=yes, 0=no)
 * @return Generated password (caller must free with ziplock_hybrid_string_free), NULL on failure
 */
char* ziplock_hybrid_password_generate(
    int length,
    int uppercase,
    int lowercase,
    int numbers,
    int symbols
);

/**
 * Calculate password strength score (0-100)
 * 
 * @param password Password to analyze
 * @return Strength score (0-100), 0 on error
 */
int ziplock_hybrid_password_strength(const char* password);

/**
 * Calculate password entropy in bits
 * 
 * @param password Password to analyze
 * @return Entropy in bits, 0.0 on error
 */
double ziplock_hybrid_password_entropy(const char* password);

/**
 * Validation Functions
 */

/**
 * Validate email address format
 * 
 * @param email Email address to validate
 * @return 1 if valid, 0 if invalid
 */
int ziplock_hybrid_email_validate(const char* email);

/**
 * Validate URL format
 * 
 * @param url URL to validate
 * @return 1 if valid, 0 if invalid
 */
int ziplock_hybrid_url_validate(const char* url);

/**
 * Validate phone number format
 * 
 * @param phone Phone number to validate
 * @param country_code Optional country code (can be NULL)
 * @return 1 if valid, 0 if invalid
 */
int ziplock_hybrid_phone_validate(const char* phone, const char* country_code);

/**
 * Cryptographic Functions
 */

/**
 * Encrypt data with password
 * 
 * @param data Data to encrypt
 * @param password Encryption password
 * @return Encrypted data (caller must free with ziplock_hybrid_string_free), NULL on failure
 */
char* ziplock_hybrid_encrypt_data(const char* data, const char* password);

/**
 * Decrypt data with password
 * 
 * @param encrypted_data Encrypted data
 * @param password Decryption password
 * @return Decrypted data (caller must free with ziplock_hybrid_string_free), NULL on failure
 */
char* ziplock_hybrid_decrypt_data(const char* encrypted_data, const char* password);

/**
 * Generate cryptographic salt
 * 
 * @return Generated salt (caller must free with ziplock_hybrid_string_free), NULL on failure
 */
char* ziplock_hybrid_generate_salt(void);

/**
 * Utility Functions
 */

/**
 * Test echo function for connectivity testing
 * 
 * @param input Input string
 * @return Echo of input (caller must free with ziplock_hybrid_string_free), NULL on failure
 */
char* ziplock_hybrid_test_echo(const char* input);

/**
 * Free string allocated by FFI functions
 * 
 * @param ptr String pointer to free
 */
void ziplock_hybrid_string_free(char* ptr);

/**
 * Example Usage:
 * 
 * ```c
 * // Initialize
 * if (ziplock_hybrid_init() != ZIPLOCK_HYBRID_SUCCESS) {
 *     char* error = ziplock_hybrid_get_last_error();
 *     printf("Init failed: %s\n", error);
 *     ziplock_hybrid_string_free(error);
 *     return -1;
 * }
 * 
 * // Create credential
 * uint64_t cred_id = ziplock_hybrid_credential_new("My Login", "login");
 * if (cred_id == 0) {
 *     printf("Failed to create credential\n");
 *     return -1;
 * }
 * 
 * // Add fields
 * ziplock_hybrid_credential_add_field(cred_id, "username", ZIPLOCK_FIELD_USERNAME, 
 *                                     "user@example.com", "Username", 0);
 * ziplock_hybrid_credential_add_field(cred_id, "password", ZIPLOCK_FIELD_PASSWORD, 
 *                                     "secret123", "Password", 1);
 * 
 * // Generate password
 * char* password = ziplock_hybrid_password_generate(16, 1, 1, 1, 0);
 * if (password) {
 *     printf("Generated: %s\n", password);
 *     ziplock_hybrid_string_free(password);
 * }
 * 
 * // Validate email
 * if (ziplock_hybrid_email_validate("user@example.com")) {
 *     printf("Email is valid\n");
 * }
 * 
 * // Convert to JSON
 * char* json = ziplock_hybrid_credential_to_json(cred_id);
 * if (json) {
 *     printf("JSON: %s\n", json);
 *     ziplock_hybrid_string_free(json);
 * }
 * 
 * // Cleanup
 * ziplock_hybrid_credential_free(cred_id);
 * ziplock_hybrid_cleanup();
 * ```
 */

#ifdef __cplusplus
}
#endif

#endif /* ZIPLOCK_HYBRID_H */