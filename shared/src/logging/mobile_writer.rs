//! Mobile-specific log writer for ZipLock
//!
//! This module provides a log writer implementation for mobile platforms
//! that can interface with platform-specific logging systems like Android's
//! logcat or iOS's unified logging system.

use std::io::{self, Write};

/// Mobile log writer that can be used with logging frameworks
///
/// This writer provides a bridge between Rust logging and mobile platform
/// logging systems. It implements the Write trait so it can be used with
/// standard Rust logging libraries.
pub struct MobileLogWriter {
    /// Buffer for accumulating log data
    buffer: Vec<u8>,
    /// Maximum buffer size before flushing
    max_buffer_size: usize,
}

impl MobileLogWriter {
    /// Create a new mobile log writer
    ///
    /// # Arguments
    /// * `max_buffer_size` - Maximum size of internal buffer before auto-flush
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(max_buffer_size.max(1024)),
            max_buffer_size,
        }
    }

    /// Create a mobile log writer with default buffer size
    pub fn default() -> Self {
        Self::new(8192) // 8KB default buffer
    }

    /// Write to platform-specific log system
    ///
    /// This function handles the actual writing to the mobile platform's
    /// logging system. The implementation varies by platform.
    fn write_to_platform(&self, data: &[u8]) -> io::Result<()> {
        let message = String::from_utf8_lossy(data);
        let trimmed = message.trim();

        if trimmed.is_empty() {
            return Ok(());
        }

        // Platform-specific logging implementations
        #[cfg(target_os = "android")]
        {
            self.write_to_android_log(trimmed)
        }

        #[cfg(target_os = "ios")]
        {
            self.write_to_ios_log(trimmed)
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            // Fallback for non-mobile platforms or testing
            eprintln!("[MOBILE_LOG] {}", trimmed);
            Ok(())
        }
    }

    /// Write to Android logcat
    #[cfg(target_os = "android")]
    fn write_to_android_log(&self, message: &str) -> io::Result<()> {
        use std::ffi::CString;

        // Parse log level from message prefix if present
        let (level, clean_message) = self.parse_log_level(message);

        // Convert to Android log priority
        let priority = match level {
            Some("ERROR") => android_log_sys::LogPriority::ERROR,
            Some("WARN") => android_log_sys::LogPriority::WARN,
            Some("INFO") => android_log_sys::LogPriority::INFO,
            Some("DEBUG") => android_log_sys::LogPriority::DEBUG,
            Some("TRACE") => android_log_sys::LogPriority::VERBOSE,
            _ => android_log_sys::LogPriority::INFO,
        };

        let tag = CString::new("ZipLock")
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid tag string"))?;

        let msg = CString::new(clean_message)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid message string"))?;

        unsafe {
            android_log_sys::__android_log_write(priority as i32, tag.as_ptr(), msg.as_ptr());
        }

        Ok(())
    }

    /// Write to iOS unified logging system
    #[cfg(target_os = "ios")]
    fn write_to_ios_log(&self, message: &str) -> io::Result<()> {
        use std::ffi::CString;

        // Parse log level from message prefix if present
        let (level, clean_message) = self.parse_log_level(message);

        // Convert to iOS log type
        let log_type = match level {
            Some("ERROR") => os_log_sys::OS_LOG_TYPE_ERROR,
            Some("WARN") => os_log_sys::OS_LOG_TYPE_DEFAULT,
            Some("INFO") => os_log_sys::OS_LOG_TYPE_INFO,
            Some("DEBUG") => os_log_sys::OS_LOG_TYPE_DEBUG,
            Some("TRACE") => os_log_sys::OS_LOG_TYPE_DEBUG,
            _ => os_log_sys::OS_LOG_TYPE_DEFAULT,
        };

        let msg = CString::new(clean_message)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid message string"))?;

        unsafe {
            // Use default log object
            os_log_sys::_os_log_impl(
                std::ptr::null_mut(),
                os_log_sys::OS_LOG_DEFAULT,
                log_type,
                b"%s\0".as_ptr() as *const i8,
                msg.as_ptr(),
            );
        }

        Ok(())
    }

    /// Parse log level from message prefix
    ///
    /// Returns the log level (if found) and the message without the level prefix
    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn parse_log_level<'a>(&self, message: &'a str) -> (Option<&'a str>, &'a str) {
        if let Some(message) = message.strip_prefix('[') {
            if let Some(end_bracket) = message.find(']') {
                let level_part = &message[..end_bracket];
                let rest = &message[end_bracket + 1..].trim_start();

                match level_part {
                    "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE" => {
                        return (Some(level_part), rest);
                    }
                    _ => {}
                }
            }
        }

        (None, message)
    }

    /// Check if buffer should be flushed
    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.max_buffer_size
    }

    /// Process complete lines from the buffer
    fn process_complete_lines(&mut self) -> io::Result<()> {
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            // Extract the line including the newline
            let line_end = newline_pos + 1;
            let line_bytes: Vec<u8> = self.buffer.drain(..line_end).collect();

            // Write to platform log
            self.write_to_platform(&line_bytes)?;
        }
        Ok(())
    }
}

impl Write for MobileLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Add to buffer
        self.buffer.extend_from_slice(buf);

        // Process any complete lines
        self.process_complete_lines()?;

        // Auto-flush if buffer is getting full
        if self.should_flush() {
            self.flush()?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Process any remaining complete lines
        self.process_complete_lines()?;

        // If there's remaining data without a newline, write it as well
        if !self.buffer.is_empty() {
            let remaining: Vec<u8> = self.buffer.drain(..).collect();
            self.write_to_platform(&remaining)?;
        }

        Ok(())
    }
}

impl Drop for MobileLogWriter {
    fn drop(&mut self) {
        // Ensure any remaining data is written
        let _ = self.flush();
    }
}

/// Create a mobile log writer for use with logging frameworks
pub fn create_mobile_writer() -> MobileLogWriter {
    MobileLogWriter::default()
}

/// Helper function to determine if we're running on a mobile platform
pub fn is_mobile_platform() -> bool {
    cfg!(any(target_os = "android", target_os = "ios"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_writer_creation() {
        let writer = MobileLogWriter::new(1024);
        assert_eq!(writer.max_buffer_size, 1024);
        assert!(writer.buffer.is_empty());

        let default_writer = MobileLogWriter::default();
        assert_eq!(default_writer.max_buffer_size, 8192);
    }

    #[test]
    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn test_parse_log_level() {
        let writer = MobileLogWriter::default();

        assert_eq!(
            writer.parse_log_level("[ERROR] Something went wrong"),
            (Some("ERROR"), "Something went wrong")
        );

        assert_eq!(
            writer.parse_log_level("[INFO] Information message"),
            (Some("INFO"), "Information message")
        );

        assert_eq!(
            writer.parse_log_level("Regular message without level"),
            (None, "Regular message without level")
        );

        assert_eq!(
            writer.parse_log_level("[INVALID] Not a real level"),
            (None, "[INVALID] Not a real level")
        );
    }

    #[test]
    fn test_write_and_flush() {
        let mut writer = MobileLogWriter::new(100);

        // Write some data
        let result = writer.write(b"[INFO] Test message\n");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20);

        // Buffer should be empty after processing the complete line
        assert!(writer.buffer.is_empty());

        // Write partial data
        let result = writer.write(b"[DEBUG] Partial");
        assert!(result.is_ok());
        assert_eq!(writer.buffer.len(), 15);

        // Flush should write the remaining data
        let result = writer.flush();
        assert!(result.is_ok());
        assert!(writer.buffer.is_empty());
    }

    #[test]
    fn test_auto_flush_on_buffer_full() {
        let mut writer = MobileLogWriter::new(10); // Small buffer for testing

        // Write more than buffer size
        let large_data = b"This is a very long message that exceeds buffer size";
        let result = writer.write(large_data);
        assert!(result.is_ok());

        // Buffer should have been flushed and should be small now
        assert!(writer.buffer.len() < large_data.len());
    }

    #[test]
    fn test_multiple_lines() {
        let mut writer = MobileLogWriter::new(1000);

        let multi_line = b"[INFO] First line\n[ERROR] Second line\n[DEBUG] Third line\n";
        let result = writer.write(multi_line);
        assert!(result.is_ok());

        // All complete lines should have been processed
        assert!(writer.buffer.is_empty());
    }

    #[test]
    fn test_is_mobile_platform() {
        let is_mobile = is_mobile_platform();

        // This will be true on Android/iOS, false elsewhere
        #[cfg(any(target_os = "android", target_os = "ios"))]
        assert!(is_mobile);

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        assert!(!is_mobile);
    }

    #[test]
    fn test_create_mobile_writer() {
        let writer = create_mobile_writer();
        assert_eq!(writer.max_buffer_size, 8192);
        assert!(writer.buffer.is_empty());
    }

    #[test]
    fn test_empty_message_handling() {
        let mut writer = MobileLogWriter::default();

        // Empty writes should be handled gracefully
        assert!(writer.write(b"").is_ok());
        assert!(writer.write(b"\n").is_ok());
        assert!(writer.write(b"   \n").is_ok());

        assert!(writer.flush().is_ok());
    }

    #[test]
    fn test_drop_cleanup() {
        {
            let mut writer = MobileLogWriter::new(100);
            let _ = writer.write(b"[INFO] Message without newline");
            // Writer should flush remaining data when dropped
        }
        // If we reach here without panicking, the drop worked correctly
    }
}
