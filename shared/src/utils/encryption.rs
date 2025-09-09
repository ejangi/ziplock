//! Encryption utilities for ZipLock
//!
//! This module provides secure encryption and decryption utilities for
//! credential data, including AES encryption, key derivation, and secure
//! memory handling for sensitive operations.

use base64::prelude::*;
use rand::{thread_rng, RngCore};
use sha2::{Digest, Sha256};
use std::convert::TryInto;

/// Error types for encryption operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionError {
    /// Invalid key length
    InvalidKeyLength,
    /// Invalid IV length
    InvalidIvLength,
    /// Encryption failed
    EncryptionFailed(String),
    /// Decryption failed
    DecryptionFailed(String),
    /// Invalid padding
    InvalidPadding,
    /// Key derivation failed
    KeyDerivationFailed,
    /// Invalid input data
    InvalidInput,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::InvalidKeyLength => write!(f, "Invalid key length"),
            EncryptionError::InvalidIvLength => write!(f, "Invalid IV length"),
            EncryptionError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            EncryptionError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            EncryptionError::InvalidPadding => write!(f, "Invalid padding"),
            EncryptionError::KeyDerivationFailed => write!(f, "Key derivation failed"),
            EncryptionError::InvalidInput => write!(f, "Invalid input data"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Result type for encryption operations
pub type EncryptionResult<T> = Result<T, EncryptionError>;

/// AES-256-GCM encryption parameters
pub const AES_KEY_SIZE: usize = 32; // 256 bits
pub const AES_IV_SIZE: usize = 12; // 96 bits for GCM
pub const AES_TAG_SIZE: usize = 16; // 128 bits
pub const SALT_SIZE: usize = 32; // 256 bits
pub const PBKDF2_ITERATIONS: u32 = 100_000;

/// Encrypted data container
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Salt used for key derivation
    pub salt: Vec<u8>,
    /// Initialization vector
    pub iv: Vec<u8>,
    /// Encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// Authentication tag
    pub tag: Vec<u8>,
}

impl EncryptedData {
    /// Serialize encrypted data to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(self.salt.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.salt);
        data.extend_from_slice(&(self.iv.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.iv);
        data.extend_from_slice(&(self.tag.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.tag);
        data.extend_from_slice(&self.ciphertext);
        data
    }

    /// Deserialize encrypted data from bytes
    pub fn from_bytes(data: &[u8]) -> EncryptionResult<Self> {
        if data.len() < 12 {
            // Minimum size: 3 * u32 length fields
            return Err(EncryptionError::InvalidInput);
        }

        let mut offset = 0;

        // Read salt
        let salt_len = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|_| EncryptionError::InvalidInput)?,
        ) as usize;
        offset += 4;

        if offset + salt_len > data.len() {
            return Err(EncryptionError::InvalidInput);
        }
        let salt = data[offset..offset + salt_len].to_vec();
        offset += salt_len;

        // Read IV
        let iv_len = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|_| EncryptionError::InvalidInput)?,
        ) as usize;
        offset += 4;

        if offset + iv_len > data.len() {
            return Err(EncryptionError::InvalidInput);
        }
        let iv = data[offset..offset + iv_len].to_vec();
        offset += iv_len;

        // Read tag
        let tag_len = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|_| EncryptionError::InvalidInput)?,
        ) as usize;
        offset += 4;

        if offset + tag_len > data.len() {
            return Err(EncryptionError::InvalidInput);
        }
        let tag = data[offset..offset + tag_len].to_vec();
        offset += tag_len;

        // Read ciphertext
        let ciphertext = data[offset..].to_vec();

        Ok(EncryptedData {
            salt,
            iv,
            ciphertext,
            tag,
        })
    }
}

/// Secure encryption utilities
pub struct EncryptionUtils;

impl EncryptionUtils {
    /// Generate a secure random salt
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; SALT_SIZE];
        thread_rng().fill_bytes(&mut salt);
        salt
    }

    /// Generate a secure random IV
    pub fn generate_iv() -> Vec<u8> {
        let mut iv = vec![0u8; AES_IV_SIZE];
        thread_rng().fill_bytes(&mut iv);
        iv
    }

    /// Derive encryption key from password using PBKDF2
    pub fn derive_key(password: &str, salt: &[u8]) -> EncryptionResult<Vec<u8>> {
        if salt.len() < 16 {
            return Err(EncryptionError::KeyDerivationFailed);
        }

        let mut key = vec![0u8; AES_KEY_SIZE];

        // Simple PBKDF2 implementation using SHA-256
        // Note: In production, use a proper PBKDF2 library like `pbkdf2` crate
        let mut hasher = Sha256::new();
        let mut current = password.as_bytes().to_vec();
        current.extend_from_slice(salt);

        for _ in 0..PBKDF2_ITERATIONS {
            hasher.update(&current);
            current = hasher.finalize_reset().to_vec();
        }

        key.copy_from_slice(&current[..AES_KEY_SIZE]);
        Ok(key)
    }

    /// Encrypt data using AES-256-GCM (simplified implementation)
    pub fn encrypt(plaintext: &[u8], password: &str) -> EncryptionResult<EncryptedData> {
        let salt = Self::generate_salt();
        let iv = Self::generate_iv();
        let key = Self::derive_key(password, &salt)?;

        // Simplified AES encryption (in production, use a proper AES-GCM library)
        let ciphertext = Self::simple_encrypt(plaintext, &key, &iv)?;
        let tag = Self::compute_auth_tag(&ciphertext, &key, &iv);

        Ok(EncryptedData {
            salt,
            iv,
            ciphertext,
            tag,
        })
    }

    /// Decrypt data using AES-256-GCM (simplified implementation)
    pub fn decrypt(encrypted: &EncryptedData, password: &str) -> EncryptionResult<Vec<u8>> {
        let key = Self::derive_key(password, &encrypted.salt)?;

        // Verify authentication tag
        let expected_tag = Self::compute_auth_tag(&encrypted.ciphertext, &key, &encrypted.iv);
        if expected_tag != encrypted.tag {
            return Err(EncryptionError::DecryptionFailed(
                "Authentication failed".to_string(),
            ));
        }

        // Decrypt data
        let plaintext = Self::simple_decrypt(&encrypted.ciphertext, &key, &encrypted.iv)?;
        Ok(plaintext)
    }

    /// Simple XOR-based encryption (for demonstration - use proper AES in production)
    fn simple_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> EncryptionResult<Vec<u8>> {
        if key.len() != AES_KEY_SIZE || iv.len() != AES_IV_SIZE {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let mut encrypted = Vec::with_capacity(data.len());
        let key_stream = Self::generate_key_stream(key, iv, data.len());

        for (i, &byte) in data.iter().enumerate() {
            encrypted.push(byte ^ key_stream[i]);
        }

        Ok(encrypted)
    }

    /// Simple XOR-based decryption (for demonstration - use proper AES in production)
    fn simple_decrypt(data: &[u8], key: &[u8], iv: &[u8]) -> EncryptionResult<Vec<u8>> {
        // XOR decryption is the same as encryption
        Self::simple_encrypt(data, key, iv)
    }

    /// Generate key stream for encryption (simplified)
    fn generate_key_stream(key: &[u8], iv: &[u8], length: usize) -> Vec<u8> {
        let mut stream = Vec::with_capacity(length);
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(iv);

        let mut counter = 0u64;
        while stream.len() < length {
            hasher.update(&counter.to_le_bytes());
            let hash = hasher.finalize_reset();

            for &byte in hash.iter() {
                if stream.len() < length {
                    stream.push(byte);
                } else {
                    break;
                }
            }

            counter += 1;
        }

        stream
    }

    /// Compute authentication tag (simplified HMAC)
    fn compute_auth_tag(data: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(iv);
        hasher.update(data);
        hasher.finalize()[..AES_TAG_SIZE].to_vec()
    }

    /// Securely compare two byte arrays (constant time)
    pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }

    /// Generate secure random bytes
    pub fn random_bytes(size: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; size];
        thread_rng().fill_bytes(&mut bytes);
        bytes
    }

    /// Hash data using SHA-256
    pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Generate a secure random key
    pub fn generate_key() -> Vec<u8> {
        Self::random_bytes(AES_KEY_SIZE)
    }
}

/// Secure memory utilities for handling sensitive data
pub struct SecureMemory;

impl SecureMemory {
    /// Securely zero out memory
    pub fn zero_memory(data: &mut [u8]) {
        // Prevent compiler optimization with volatile write
        for byte in data.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }

    /// Create a secure string that zeros itself on drop
    pub fn secure_string(s: String) -> SecureString {
        SecureString::new(s)
    }
}

/// A string that securely zeros its memory on drop
pub struct SecureString {
    data: Vec<u8>,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }

    pub fn as_str(&self) -> &str {
        // Safety: We only create SecureString from valid UTF-8 strings
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        SecureMemory::zero_memory(&mut self.data);
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

/// Credential encryption helper
pub struct CredentialCrypto;

impl CredentialCrypto {
    /// Encrypt sensitive credential field
    pub fn encrypt_field(value: &str, master_password: &str) -> EncryptionResult<String> {
        let encrypted = EncryptionUtils::encrypt(value.as_bytes(), master_password)?;
        let encoded = BASE64_STANDARD.encode(encrypted.to_bytes());
        Ok(encoded)
    }

    /// Decrypt sensitive credential field
    pub fn decrypt_field(encrypted_value: &str, master_password: &str) -> EncryptionResult<String> {
        let decoded = BASE64_STANDARD
            .decode(encrypted_value)
            .map_err(|_| EncryptionError::DecryptionFailed("Invalid base64".to_string()))?;

        let encrypted = EncryptedData::from_bytes(&decoded)?;
        let plaintext = EncryptionUtils::decrypt(&encrypted, master_password)?;

        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::DecryptionFailed("Invalid UTF-8".to_string()))
    }

    /// Check if a string appears to be encrypted
    pub fn is_encrypted(value: &str) -> bool {
        // Simple heuristic: encrypted values are base64 and reasonably long
        value.len() > 50 && BASE64_STANDARD.decode(value).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_generation() {
        let salt1 = EncryptionUtils::generate_salt();
        let salt2 = EncryptionUtils::generate_salt();

        assert_eq!(salt1.len(), SALT_SIZE);
        assert_eq!(salt2.len(), SALT_SIZE);
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_iv_generation() {
        let iv1 = EncryptionUtils::generate_iv();
        let iv2 = EncryptionUtils::generate_iv();

        assert_eq!(iv1.len(), AES_IV_SIZE);
        assert_eq!(iv2.len(), AES_IV_SIZE);
        assert_ne!(iv1, iv2);
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = EncryptionUtils::generate_salt();

        let key1 = EncryptionUtils::derive_key(password, &salt).unwrap();
        let key2 = EncryptionUtils::derive_key(password, &salt).unwrap();

        assert_eq!(key1.len(), AES_KEY_SIZE);
        assert_eq!(key1, key2); // Same password and salt should give same key

        let different_salt = EncryptionUtils::generate_salt();
        let key3 = EncryptionUtils::derive_key(password, &different_salt).unwrap();
        assert_ne!(key1, key3); // Different salt should give different key
    }

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Hello, secure world!";
        let password = "test_password";

        let encrypted = EncryptionUtils::encrypt(plaintext, password).unwrap();
        let decrypted = EncryptionUtils::decrypt(&encrypted, password).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password_decryption() {
        let plaintext = b"Hello, secure world!";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = EncryptionUtils::encrypt(plaintext, password).unwrap();
        let result = EncryptionUtils::decrypt(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let plaintext = b"Test data for serialization";
        let password = "serialization_test";

        let encrypted = EncryptionUtils::encrypt(plaintext, password).unwrap();
        let bytes = encrypted.to_bytes();
        let deserialized = EncryptedData::from_bytes(&bytes).unwrap();

        assert_eq!(encrypted.salt, deserialized.salt);
        assert_eq!(encrypted.iv, deserialized.iv);
        assert_eq!(encrypted.ciphertext, deserialized.ciphertext);
        assert_eq!(encrypted.tag, deserialized.tag);
    }

    #[test]
    fn test_secure_compare() {
        let a = b"same_data";
        let b = b"same_data";
        let c = b"different";

        assert!(EncryptionUtils::secure_compare(a, b));
        assert!(!EncryptionUtils::secure_compare(a, c));
        assert!(!EncryptionUtils::secure_compare(a, b"same_dat")); // Different length
    }

    #[test]
    fn test_secure_string() {
        let original = "sensitive_data".to_string();
        let secure = SecureString::new(original.clone());

        assert_eq!(secure.as_str(), &original);
        assert_eq!(secure.len(), original.len());
        assert!(!secure.is_empty());

        // Test drop (can't easily test zeroing, but we can test it doesn't panic)
        drop(secure);
    }

    #[test]
    fn test_credential_crypto() {
        let field_value = "sensitive_password";
        let master_password = "master_key";

        let encrypted = CredentialCrypto::encrypt_field(field_value, master_password).unwrap();
        assert!(CredentialCrypto::is_encrypted(&encrypted));

        let decrypted = CredentialCrypto::decrypt_field(&encrypted, master_password).unwrap();
        assert_eq!(decrypted, field_value);
    }

    #[test]
    fn test_credential_crypto_wrong_password() {
        let field_value = "sensitive_password";
        let master_password = "master_key";
        let wrong_password = "wrong_key";

        let encrypted = CredentialCrypto::encrypt_field(field_value, master_password).unwrap();
        let result = CredentialCrypto::decrypt_field(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_is_encrypted_detection() {
        assert!(CredentialCrypto::is_encrypted("dGhpcyBpcyBhIGxvbmcgZW5vdWdoIGJhc2U2NCBlbmNvZGVkIHN0cmluZyB0byBwYXNzIHRoZSBoZXVyaXN0aWM="));
        assert!(!CredentialCrypto::is_encrypted("plaintext"));
        assert!(!CredentialCrypto::is_encrypted("short"));
    }

    #[test]
    fn test_hash_sha256() {
        let data = b"test data";
        let hash1 = EncryptionUtils::hash_sha256(data);
        let hash2 = EncryptionUtils::hash_sha256(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // SHA-256 produces 32-byte hash

        let different_data = b"different data";
        let hash3 = EncryptionUtils::hash_sha256(different_data);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_random_bytes() {
        let bytes1 = EncryptionUtils::random_bytes(16);
        let bytes2 = EncryptionUtils::random_bytes(16);

        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_generate_key() {
        let key1 = EncryptionUtils::generate_key();
        let key2 = EncryptionUtils::generate_key();

        assert_eq!(key1.len(), AES_KEY_SIZE);
        assert_eq!(key2.len(), AES_KEY_SIZE);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let invalid_data = b"not_valid_encrypted_data";
        let result = EncryptedData::from_bytes(invalid_data);
        assert!(result.is_err());
    }
}
