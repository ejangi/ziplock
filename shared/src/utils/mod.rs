//! Utility modules for ZipLock
//!
//! This module provides various utility functions and helpers used throughout
//! the ZipLock shared library, including TOTP generation, YAML serialization,
//! validation, and search functionality.

pub mod backup;
pub mod encryption;
pub mod password;
pub mod search;
pub mod totp;
pub mod validation;
pub mod yaml;

// Re-export commonly used items for convenience
pub use backup::{
    BackupData, BackupManager, BackupMetadata, BackupStats, ExportFormat, ExportOptions,
    MigrationManager,
};
pub use encryption::{
    CredentialCrypto, EncryptedData, EncryptionError, EncryptionResult, EncryptionUtils,
    SecureMemory, SecureString,
};
pub use password::{
    PasswordAnalysis, PasswordAnalyzer, PasswordGenerator, PasswordOptions, PasswordStrength,
    PasswordUtils,
};
pub use search::{CredentialSearchEngine, SearchQuery, SearchResult};
pub use totp::{format_totp_secret, generate_totp, validate_totp_secret};
pub use validation::{validate_credential, validate_field, ValidationResult};
pub use yaml::{
    deserialize_credential, deserialize_file_map, serialize_credential, serialize_file_map,
};

/// Utility functions for working with strings
pub mod string_utils {
    /// Truncate a string to a maximum length with ellipsis
    pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Sanitize a string by removing control characters
    pub fn sanitize_string(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_control() || *c == '\t' || *c == '\n')
            .collect()
    }

    /// Check if a string is likely to be a URL
    pub fn looks_like_url(s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
    }

    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> Option<String> {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                Some(after_protocol[..end].to_string())
            } else if let Some(end) = after_protocol.find(':') {
                Some(after_protocol[..end].to_string())
            } else {
                Some(after_protocol.to_string())
            }
        } else {
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_truncate_with_ellipsis() {
            assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
            assert_eq!(truncate_with_ellipsis("hello world", 8), "hello...");
            assert_eq!(truncate_with_ellipsis("hi", 2), "hi");
            assert_eq!(truncate_with_ellipsis("hello", 3), "...");
        }

        #[test]
        fn test_sanitize_string() {
            assert_eq!(sanitize_string("hello\x00world"), "helloworld");
            assert_eq!(sanitize_string("hello\tworld"), "hello\tworld");
            assert_eq!(sanitize_string("hello\nworld"), "hello\nworld");
        }

        #[test]
        fn test_looks_like_url() {
            assert!(looks_like_url("https://example.com"));
            assert!(looks_like_url("http://localhost"));
            assert!(looks_like_url("ftp://files.example.com"));
            assert!(!looks_like_url("example.com"));
            assert!(!looks_like_url("not a url"));
        }

        #[test]
        fn test_extract_domain() {
            assert_eq!(
                extract_domain("https://example.com/path"),
                Some("example.com".to_string())
            );
            assert_eq!(
                extract_domain("http://localhost:8080"),
                Some("localhost".to_string())
            );
            assert_eq!(
                extract_domain("https://sub.example.com"),
                Some("sub.example.com".to_string())
            );
            assert_eq!(extract_domain("not a url"), None);
        }
    }
}

/// Utility functions for working with time
pub mod time_utils {
    use chrono::{TimeZone, Utc};

    /// Format a Unix timestamp for display
    pub fn format_timestamp(timestamp: i64) -> String {
        match Utc.timestamp_opt(timestamp, 0) {
            chrono::LocalResult::Single(datetime) => {
                datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }
            _ => "Invalid date".to_string(),
        }
    }

    /// Get current Unix timestamp
    pub fn current_timestamp() -> i64 {
        Utc::now().timestamp()
    }

    /// Format duration in human-readable form
    pub fn format_duration_since(timestamp: i64) -> String {
        let now = current_timestamp();
        let diff = now - timestamp;

        if diff < 0 {
            return "in the future".to_string();
        }

        match diff {
            0..=59 => "just now".to_string(),
            60..=3599 => format!(
                "{} minute{} ago",
                diff / 60,
                if diff >= 120 { "s" } else { "" }
            ),
            3600..=86399 => format!(
                "{} hour{} ago",
                diff / 3600,
                if diff >= 7200 { "s" } else { "" }
            ),
            86400..=2591999 => format!(
                "{} day{} ago",
                diff / 86400,
                if diff >= 172800 { "s" } else { "" }
            ),
            2592000..=31535999 => format!(
                "{} month{} ago",
                diff / 2592000,
                if diff >= 5184000 { "s" } else { "" }
            ),
            _ => format!(
                "{} year{} ago",
                diff / 31536000,
                if diff >= 63072000 { "s" } else { "" }
            ),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_format_timestamp() {
            let timestamp = 1640995200; // 2022-01-01 00:00:00 UTC
            let formatted = format_timestamp(timestamp);
            assert!(formatted.contains("2022-01-01"));
            assert!(formatted.contains("00:00:00"));
        }

        #[test]
        fn test_current_timestamp() {
            let timestamp = current_timestamp();
            assert!(timestamp > 1600000000); // Should be after 2020
        }

        #[test]
        fn test_format_duration_since() {
            let now = current_timestamp();
            assert_eq!(format_duration_since(now), "just now");
            assert_eq!(format_duration_since(now - 30), "just now");
            assert_eq!(format_duration_since(now - 120), "2 minutes ago");
            assert_eq!(format_duration_since(now - 3660), "1 hour ago");
            assert_eq!(format_duration_since(now - 86400), "1 day ago");
        }
    }
}
