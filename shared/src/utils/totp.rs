//! TOTP (Time-based One-Time Password) utilities
//!
//! This module provides functions for generating TOTP codes according to RFC 6238.
//! TOTP codes are commonly used for two-factor authentication.

use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

/// Generate a 6-digit TOTP code from a base32-encoded secret
///
/// # Arguments
/// * `secret` - Base32-encoded TOTP secret (e.g., "JBSWY3DPEHPK3PXP")
/// * `time_step` - Time step in seconds (typically 30)
///
/// # Returns
/// * `Ok(String)` - 6-digit TOTP code
/// * `Err(anyhow::Error)` - If secret is invalid or generation fails
///
/// # Example
/// ```
/// use ziplock_shared::utils::totp::generate_totp;
///
/// let secret = "JBSWY3DPEHPK3PXP";
/// let code = generate_totp(secret, 30).unwrap();
/// assert_eq!(code.len(), 6);
/// ```
pub fn generate_totp(secret: &str, time_step: u64) -> Result<String> {
    // Get current Unix timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow!("Failed to get current time: {}", e))?
        .as_secs();

    generate_totp_at_time(secret, time_step, now)
}

/// Generate a TOTP code for a specific time
///
/// This function is useful for testing or generating codes for specific timestamps.
///
/// # Arguments
/// * `secret` - Base32-encoded TOTP secret
/// * `time_step` - Time step in seconds
/// * `timestamp` - Unix timestamp in seconds
///
/// # Returns
/// * `Ok(String)` - 6-digit TOTP code
/// * `Err(anyhow::Error)` - If secret is invalid or generation fails
pub fn generate_totp_at_time(secret: &str, time_step: u64, timestamp: u64) -> Result<String> {
    // Clean the secret - remove spaces and convert to uppercase
    let clean_secret = secret.replace(' ', "").to_uppercase();

    // Validate that the secret looks like base32
    if clean_secret.is_empty() {
        return Err(anyhow!("TOTP secret cannot be empty"));
    }

    // Check if secret contains only valid base32 characters
    if !clean_secret
        .chars()
        .all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=".contains(c))
    {
        return Err(anyhow!(
            "Invalid base32 secret: contains invalid characters"
        ));
    }

    // Decode base32 secret to bytes
    let secret_bytes = match base32_decode(&clean_secret) {
        Ok(bytes) => bytes,
        Err(_) => return Err(anyhow!("Invalid base32 secret")),
    };

    // Calculate time counter (number of time steps since Unix epoch)
    let time_counter = timestamp / time_step;

    // Generate TOTP using HMAC-SHA1
    let code = generate_totp_code(&secret_bytes, time_counter)?;
    Ok(format!("{:06}", code))
}

/// Get the remaining seconds until the next TOTP refresh
///
/// # Arguments
/// * `time_step` - Time step in seconds (typically 30)
///
/// # Returns
/// * `u64` - Seconds remaining until next refresh
pub fn get_seconds_until_refresh(time_step: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    time_step - (now % time_step)
}

/// Validate a base32-encoded TOTP secret
///
/// # Arguments
/// * `secret` - The secret to validate
///
/// # Returns
/// * `true` if the secret is valid base32, `false` otherwise
pub fn validate_totp_secret(secret: &str) -> bool {
    let clean_secret = secret.replace(' ', "").to_uppercase();

    if clean_secret.is_empty() {
        return false;
    }

    // Check if secret contains only valid base32 characters
    if !clean_secret
        .chars()
        .all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=".contains(c))
    {
        return false;
    }

    // Try to decode as base32 to validate the secret
    base32_decode(&clean_secret).is_ok()
}

/// Format a TOTP secret for display (with spaces every 4 characters)
///
/// # Arguments
/// * `secret` - The secret to format
///
/// # Returns
/// * Formatted secret string
pub fn format_totp_secret(secret: &str) -> String {
    let clean_secret = secret.replace(' ', "").to_uppercase();
    clean_secret
        .chars()
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if i > 0 && i % 4 == 0 {
                acc.push(' ');
            }
            acc.push(c);
            acc
        })
}

/// Decode a base32 string to bytes
fn base32_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits_left = 0;

    for c in input.chars() {
        if c == '=' {
            break; // Padding
        }

        let value = match alphabet.find(c) {
            Some(v) => v as u32,
            None => return Err("Invalid base32 character"),
        };

        buffer = (buffer << 5) | value;
        bits_left += 5;

        if bits_left >= 8 {
            output.push((buffer >> (bits_left - 8)) as u8);
            bits_left -= 8;
        }
    }

    Ok(output)
}

/// Generate TOTP code using HMAC-SHA1 according to RFC 6238
fn generate_totp_code(secret: &[u8], time_counter: u64) -> Result<u32> {
    // Convert time counter to big-endian bytes
    let time_bytes = time_counter.to_be_bytes();

    // Create HMAC-SHA1 instance
    let mut mac =
        HmacSha1::new_from_slice(secret).map_err(|_| anyhow!("Invalid secret length for HMAC"))?;

    // Update HMAC with time counter
    mac.update(&time_bytes);

    // Get HMAC result
    let result = mac.finalize().into_bytes();

    // Dynamic truncation according to RFC 4226
    let offset = (result[19] & 0xf) as usize;
    let truncated = u32::from_be_bytes([
        result[offset] & 0x7f,
        result[offset + 1],
        result[offset + 2],
        result[offset + 3],
    ]);

    // Return 6-digit code
    Ok(truncated % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_totp_with_known_values() {
        // Test with RFC 6238 test vectors
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"; // "12345678901234567890" in base32

        // Test vector: T = 59 seconds, TOTP = 287082
        let code = generate_totp_at_time(secret, 30, 59).unwrap();
        assert_eq!(code, "287082");

        // Test vector: T = 1111111109 seconds, TOTP = 081804
        let code = generate_totp_at_time(secret, 30, 1111111109).unwrap();
        assert_eq!(code, "081804");
    }

    #[test]
    fn test_generate_totp_with_spaces() {
        let secret = "JBSW Y3DP EHPK 3PXP";
        let result = generate_totp(secret, 30);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 6);
    }

    #[test]
    fn test_generate_totp_lowercase() {
        let secret = "jbswy3dpehpk3pxp";
        let result = generate_totp(secret, 30);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 6);
    }

    #[test]
    fn test_generate_totp_invalid_secret() {
        let secret = "invalid!@#$%";
        let result = generate_totp(secret, 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_totp_empty_secret() {
        let result = generate_totp("", 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_totp_secret() {
        assert!(validate_totp_secret("JBSWY3DPEHPK3PXP"));
        assert!(validate_totp_secret("jbswy3dpehpk3pxp"));
        assert!(validate_totp_secret("JBSW Y3DP EHPK 3PXP"));
        assert!(!validate_totp_secret(""));
        assert!(!validate_totp_secret("invalid!@#"));
        assert!(!validate_totp_secret("123456"));
    }

    #[test]
    fn test_format_totp_secret() {
        assert_eq!(
            format_totp_secret("JBSWY3DPEHPK3PXP"),
            "JBSW Y3DP EHPK 3PXP"
        );
        assert_eq!(
            format_totp_secret("jbswy3dpehpk3pxp"),
            "JBSW Y3DP EHPK 3PXP"
        );
        assert_eq!(format_totp_secret("JBSW Y3DP"), "JBSW Y3DP");
    }

    #[test]
    fn test_get_seconds_until_refresh() {
        let remaining = get_seconds_until_refresh(30);
        assert!(remaining > 0 && remaining <= 30);
    }

    #[test]
    fn test_totp_code_format() {
        let secret = "JBSWY3DPEHPK3PXP";
        let code = generate_totp(secret, 30).unwrap();

        // Should be exactly 6 digits
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));

        // Should be zero-padded if necessary
        assert!(code.starts_with('0') || code.parse::<u32>().unwrap() >= 100000);
    }

    #[test]
    fn test_totp_synchronization() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let secret = "JBSWY3DPEHPK3PXP";
        let time_step = 30;

        // Get current time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate code for current time
        let code1 = generate_totp_at_time(secret, time_step, now).unwrap();

        // Generate code for same time window (should be identical)
        let same_window_time = now - (now % time_step) + 15; // 15 seconds into current window
        let code2 = generate_totp_at_time(secret, time_step, same_window_time).unwrap();
        assert_eq!(
            code1, code2,
            "Codes should be identical within same time window"
        );

        // Generate code for next time window (should be different)
        let next_window_time = (now / time_step + 1) * time_step;
        let code3 = generate_totp_at_time(secret, time_step, next_window_time).unwrap();
        assert_ne!(
            code1, code3,
            "Codes should be different across time windows"
        );

        // Verify countdown timing
        let remaining = get_seconds_until_refresh(time_step);
        assert!(
            remaining > 0 && remaining <= time_step,
            "Remaining time should be between 1 and {} seconds, got {}",
            time_step,
            remaining
        );

        // Verify countdown calculation is consistent with time boundaries
        let expected_remaining = time_step - (now % time_step);
        assert_eq!(
            remaining, expected_remaining,
            "Countdown should match expected calculation based on time boundaries"
        );
    }
}
