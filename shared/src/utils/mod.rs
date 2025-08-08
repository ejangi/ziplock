//! Shared utilities for ZipLock
//!
//! This module provides common utility functions used throughout the
//! ZipLock application for string manipulation, data processing,
//! and other helper operations.

use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// String utilities
pub struct StringUtils;

impl StringUtils {
    /// Normalize whitespace in a string (trim and collapse multiple spaces)
    pub fn normalize_whitespace(input: &str) -> String {
        input.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// Check if a string contains only printable ASCII characters
    pub fn is_printable_ascii(input: &str) -> bool {
        input.chars().all(|c| c.is_ascii() && !c.is_control())
    }

    /// Truncate a string to a maximum length, adding ellipsis if needed
    pub fn truncate(input: &str, max_length: usize) -> String {
        if input.len() <= max_length {
            input.to_string()
        } else if max_length <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &input[..max_length - 3])
        }
    }

    /// Convert a string to a safe filename
    pub fn to_safe_filename(input: &str) -> String {
        let mut result = String::new();

        for c in input.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => result.push(c),
                ' ' => result.push('_'),
                _ => result.push('_'),
            }
        }

        // Ensure it doesn't start with a dot or dash
        if result.starts_with('.') || result.starts_with('-') {
            result.insert(0, '_');
        }

        // Limit length
        if result.len() > 255 {
            result.truncate(255);
        }

        result
    }

    /// Check if a string looks like a URL
    pub fn looks_like_url(input: &str) -> bool {
        input.starts_with("http://")
            || input.starts_with("https://")
            || input.starts_with("ftp://")
            || input.contains("://")
    }

    /// Check if a string looks like an email address
    pub fn looks_like_email(input: &str) -> bool {
        input.contains('@')
            && input.len() > 3
            && !input.starts_with('@')
            && !input.ends_with('@')
            && input.matches('@').count() == 1
    }
}

/// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Get current Unix timestamp
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Convert SystemTime to Unix timestamp
    pub fn system_time_to_timestamp(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Convert Unix timestamp to SystemTime
    pub fn timestamp_to_system_time(timestamp: u64) -> SystemTime {
        UNIX_EPOCH + std::time::Duration::from_secs(timestamp)
    }

    /// Format a SystemTime as ISO 8601 string (UTC)
    pub fn format_iso8601(time: SystemTime) -> String {
        let timestamp = Self::system_time_to_timestamp(time);
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0);

        match datetime {
            Some(dt) => dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            None => "1970-01-01T00:00:00Z".to_string(),
        }
    }

    /// Parse ISO 8601 string to SystemTime
    pub fn parse_iso8601(iso_string: &str) -> Option<SystemTime> {
        chrono::DateTime::parse_from_rfc3339(iso_string)
            .ok()
            .map(|dt| UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64))
    }
}

/// Collection utilities
pub struct CollectionUtils;

impl CollectionUtils {
    /// Merge two HashMaps, with values from the second map taking precedence
    pub fn merge_hashmaps<K, V>(mut first: HashMap<K, V>, second: HashMap<K, V>) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
    {
        for (key, value) in second {
            first.insert(key, value);
        }
        first
    }

    /// Remove duplicates from a vector while preserving order
    pub fn dedup_preserve_order<T>(mut vec: Vec<T>) -> Vec<T>
    where
        T: PartialEq + Clone,
    {
        let mut seen = Vec::new();
        vec.retain(|item| {
            if seen.contains(item) {
                false
            } else {
                seen.push(item.clone());
                true
            }
        });
        vec
    }

    /// Group items by a key function
    pub fn group_by<T, K, F>(items: Vec<T>, key_fn: F) -> HashMap<K, Vec<T>>
    where
        K: std::hash::Hash + Eq,
        F: Fn(&T) -> K,
    {
        let mut groups = HashMap::new();

        for item in items {
            let key = key_fn(&item);
            groups.entry(key).or_insert_with(Vec::new).push(item);
        }

        groups
    }
}

/// Data validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Check if a string is a valid UUID v4
    pub fn is_valid_uuid_v4(uuid: &str) -> bool {
        if uuid.len() != 36 {
            return false;
        }

        let parts: Vec<&str> = uuid.split('-').collect();
        if parts.len() != 5 {
            return false;
        }

        if parts[0].len() != 8
            || parts[1].len() != 4
            || parts[2].len() != 4
            || parts[3].len() != 4
            || parts[4].len() != 12
        {
            return false;
        }

        // Check that all characters are hex digits
        uuid.chars()
            .filter(|&c| c != '-')
            .all(|c| c.is_ascii_hexdigit())
    }

    /// Validate password strength (returns score 0-100)
    pub fn password_strength_score(password: &str) -> u8 {
        if password.is_empty() {
            return 0;
        }

        let mut score = 0u8;
        let length = password.len();

        // Length scoring
        score += match length {
            0..=7 => 0,
            8..=11 => 20,
            12..=15 => 40,
            16..=19 => 60,
            _ => 80,
        };

        // Character variety
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digits = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let variety_count = [has_lowercase, has_uppercase, has_digits, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        score += match variety_count {
            0..=1 => 0,
            2 => 5,
            3 => 10,
            4 => 20,
            _ => 20,
        };

        // Bonus for no repeated characters
        let unique_chars: std::collections::HashSet<char> = password.chars().collect();
        if unique_chars.len() == password.len() {
            score = score.saturating_add(10);
        }

        score.min(100)
    }

    /// Check if an email address has a valid format (basic check)
    pub fn is_valid_email_format(email: &str) -> bool {
        if email.is_empty() || email.len() > 254 {
            return false;
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        // Basic local part validation
        if local.is_empty() || local.len() > 64 {
            return false;
        }

        // Basic domain validation
        if domain.is_empty() || domain.len() > 253 || !domain.contains('.') {
            return false;
        }

        // Check for valid characters (simplified)
        local
            .chars()
            .all(|c| c.is_alphanumeric() || ".-_+".contains(c))
            && domain
                .chars()
                .all(|c| c.is_alphanumeric() || ".-".contains(c))
    }
}

/// Encoding utilities
pub struct EncodingUtils;

impl EncodingUtils {
    /// Encode bytes as hex string
    pub fn encode_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Decode hex string to bytes
    pub fn decode_hex(hex: &str) -> Option<Vec<u8>> {
        if hex.len() % 2 != 0 {
            return None;
        }

        let mut bytes = Vec::new();
        for chunk in hex.as_bytes().chunks(2) {
            let chunk_str = std::str::from_utf8(chunk).ok()?;
            let byte = u8::from_str_radix(chunk_str, 16).ok()?;
            bytes.push(byte);
        }

        Some(bytes)
    }

    /// Simple base64 encode (using standard library)
    pub fn encode_base64(bytes: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    /// Simple base64 decode (using standard library)
    pub fn decode_base64(encoded: &str) -> Option<Vec<u8>> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_utils() {
        assert_eq!(
            StringUtils::normalize_whitespace("  hello    world  "),
            "hello world"
        );

        assert!(StringUtils::is_printable_ascii("Hello123!"));
        assert!(!StringUtils::is_printable_ascii("Hello\x00World"));

        assert_eq!(StringUtils::truncate("Hello World", 5), "He...");
        assert_eq!(StringUtils::truncate("Hi", 5), "Hi");

        assert_eq!(
            StringUtils::to_safe_filename("My File!.txt"),
            "My_File__txt"
        );

        assert!(StringUtils::looks_like_url("https://example.com"));
        assert!(!StringUtils::looks_like_url("not a url"));

        assert!(StringUtils::looks_like_email("test@example.com"));
        assert!(!StringUtils::looks_like_email("not an email"));
    }

    #[test]
    fn test_time_utils() {
        let now = SystemTime::now();
        let timestamp = TimeUtils::system_time_to_timestamp(now);
        let back = TimeUtils::timestamp_to_system_time(timestamp);

        // Should be very close (within a second)
        assert!(now.duration_since(back).unwrap_or_default().as_secs() <= 1);

        let iso = TimeUtils::format_iso8601(now);
        assert!(iso.ends_with('Z'));
        assert!(iso.len() >= 19); // YYYY-MM-DDTHH:MM:SSZ
    }

    #[test]
    fn test_collection_utils() {
        let mut map1 = HashMap::new();
        map1.insert("a", 1);
        map1.insert("b", 2);

        let mut map2 = HashMap::new();
        map2.insert("b", 3);
        map2.insert("c", 4);

        let merged = CollectionUtils::merge_hashmaps(map1, map2);
        assert_eq!(merged.get("a"), Some(&1));
        assert_eq!(merged.get("b"), Some(&3)); // Second map wins
        assert_eq!(merged.get("c"), Some(&4));

        let vec = vec![1, 2, 2, 3, 1, 4];
        let deduped = CollectionUtils::dedup_preserve_order(vec);
        assert_eq!(deduped, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_validation_utils() {
        assert!(ValidationUtils::is_valid_uuid_v4(
            "550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(!ValidationUtils::is_valid_uuid_v4("not-a-uuid"));

        assert_eq!(ValidationUtils::password_strength_score("weak"), 10);
        assert_eq!(
            ValidationUtils::password_strength_score("SuperSecure123!"),
            60
        );

        assert!(ValidationUtils::is_valid_email_format("test@example.com"));
        assert!(!ValidationUtils::is_valid_email_format("invalid"));
    }

    #[test]
    fn test_encoding_utils() {
        let bytes = b"Hello World";
        let hex = EncodingUtils::encode_hex(bytes);
        let decoded = EncodingUtils::decode_hex(&hex).unwrap();
        assert_eq!(bytes, decoded.as_slice());

        let base64 = EncodingUtils::encode_base64(bytes);
        let decoded = EncodingUtils::decode_base64(&base64).unwrap();
        assert_eq!(bytes, decoded.as_slice());
    }
}

/// Password generation utilities
pub struct PasswordUtils;

/// Configuration options for password generation
#[derive(Debug, Clone)]
pub struct PasswordOptions {
    pub length: usize,
    pub include_uppercase: bool,
    pub include_lowercase: bool,
    pub include_numbers: bool,
    pub include_symbols: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 16,
            include_uppercase: true,
            include_lowercase: true,
            include_numbers: true,
            include_symbols: true,
        }
    }
}

impl PasswordUtils {
    /// Generate a secure password with the given options
    pub fn generate_password(options: PasswordOptions) -> Result<String, String> {
        if options.length == 0 {
            return Err("Password length must be greater than 0".to_string());
        }

        if options.length > 256 {
            return Err("Password length cannot exceed 256 characters".to_string());
        }

        if !options.include_uppercase
            && !options.include_lowercase
            && !options.include_numbers
            && !options.include_symbols
        {
            return Err("At least one character type must be enabled".to_string());
        }

        let mut charset = String::new();

        if options.include_lowercase {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
        }

        if options.include_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }

        if options.include_numbers {
            charset.push_str("0123456789");
        }

        if options.include_symbols {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        let charset_chars: Vec<char> = charset.chars().collect();
        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(options.length);

        // Ensure at least one character from each enabled category
        if options.include_lowercase {
            let lowercase_chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
            password.push(lowercase_chars[rng.gen_range(0..lowercase_chars.len())]);
        }

        if options.include_uppercase {
            let uppercase_chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
            password.push(uppercase_chars[rng.gen_range(0..uppercase_chars.len())]);
        }

        if options.include_numbers {
            let number_chars: Vec<char> = "0123456789".chars().collect();
            password.push(number_chars[rng.gen_range(0..number_chars.len())]);
        }

        if options.include_symbols {
            let symbol_chars: Vec<char> = "!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect();
            password.push(symbol_chars[rng.gen_range(0..symbol_chars.len())]);
        }

        // Fill the rest randomly
        while password.len() < options.length {
            password.push(charset_chars[rng.gen_range(0..charset_chars.len())]);
        }

        // Shuffle the password to avoid predictable patterns
        let mut password_chars: Vec<char> = password.chars().collect();
        for i in (1..password_chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            password_chars.swap(i, j);
        }

        Ok(password_chars.into_iter().collect())
    }

    /// Check if a password contains common patterns that make it weak
    pub fn has_common_patterns(password: &str) -> bool {
        let common_patterns = [
            "password", "123456", "qwerty", "abc", "admin", "test", "user", "login", "welcome",
            "123", "321", "111", "000",
        ];

        let lowercase_password = password.to_lowercase();

        for pattern in &common_patterns {
            if lowercase_password.contains(pattern) {
                return true;
            }
        }

        // Check for repeated characters (more than 3 in a row)
        let chars: Vec<char> = password.chars().collect();
        for window in chars.windows(4) {
            if window.iter().all(|&c| c == window[0]) {
                return true;
            }
        }

        // Check for simple sequences
        for window in chars.windows(4) {
            let mut is_sequence = true;
            for i in 1..window.len() {
                if (window[i] as u8).saturating_sub(window[i - 1] as u8) != 1 {
                    is_sequence = false;
                    break;
                }
            }
            if is_sequence {
                return true;
            }
        }

        false
    }
}

// Re-export for convenience (already defined above)

/// Socket utilities for consistent IPC configuration
pub struct SocketUtils;

impl SocketUtils {
    /// Get the default socket path for the ZipLock backend
    /// This uses the same logic as the backend to ensure consistency
    pub fn default_socket_path() -> std::path::PathBuf {
        dirs::runtime_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("ziplock")
            .join("backend.sock")
    }
}

#[cfg(test)]
mod password_tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let options = PasswordOptions::default();
        let password = PasswordUtils::generate_password(options).unwrap();

        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_password_generation_with_custom_options() {
        let options = PasswordOptions {
            length: 8,
            include_uppercase: false,
            include_lowercase: true,
            include_numbers: true,
            include_symbols: false,
        };

        let password = PasswordUtils::generate_password(options).unwrap();

        assert_eq!(password.len(), 8);
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(!password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(!password.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_password_generation_errors() {
        let invalid_options = PasswordOptions {
            length: 0,
            include_uppercase: false,
            include_lowercase: false,
            include_numbers: false,
            include_symbols: false,
        };

        assert!(PasswordUtils::generate_password(invalid_options).is_err());
    }

    #[test]
    fn test_common_patterns() {
        assert!(PasswordUtils::has_common_patterns("password123"));
        assert!(PasswordUtils::has_common_patterns("qwerty456"));
        assert!(PasswordUtils::has_common_patterns("aaaa"));
        assert!(PasswordUtils::has_common_patterns("1234"));
        assert!(!PasswordUtils::has_common_patterns("ComplexPhrase987$"));
    }
}
