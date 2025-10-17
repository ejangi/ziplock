//! Windows Event Log integration for production logging
//!
//! This module provides integration with the Windows Event Log system,
//! allowing ZipLock to write log messages directly to the Windows Event Viewer
//! for proper system integration in production environments.
//!
//! This implementation uses PowerShell commands for simplicity and reliability,
//! avoiding the complexity of direct Windows API calls.

use anyhow::Result;
use std::io::Write;
use std::process::Command;
use tracing_subscriber::fmt::MakeWriter;

/// Windows Event Log writer for tracing integration
pub struct WindowsEventLogWriter {
    source_name: String,
    buffer: Vec<u8>,
}

impl WindowsEventLogWriter {
    /// Create a new Windows Event Log writer
    pub fn new(source_name: &str) -> Result<Self> {
        // Try to ensure the event source exists
        let _ = Self::ensure_event_source(source_name);

        Ok(Self {
            source_name: source_name.to_string(),
            buffer: Vec::new(),
        })
    }

    /// Ensure the event source is registered (best effort)
    fn ensure_event_source(source_name: &str) -> Result<()> {
        let script = format!(
            "if (-not [System.Diagnostics.EventLog]::SourceExists('{}')) {{ \
             try {{ New-EventLog -LogName Application -Source '{}' -ErrorAction Stop }} \
             catch {{ Write-Warning 'Failed to create event source' }} \
             }}",
            source_name, source_name
        );

        let _ = Command::new("powershell")
            .args(&["-ExecutionPolicy", "Bypass", "-Command", &script])
            .output();

        Ok(())
    }

    /// Log an event to the Windows Event Log using PowerShell
    pub fn log_event(&self, level: &str, message: &str) -> Result<()> {
        #[cfg(windows)]
        {
            let event_type = match level.to_uppercase().as_str() {
                "ERROR" => "Error",
                "WARN" | "WARNING" => "Warning",
                _ => "Information",
            };

            // Clean the message for PowerShell
            let clean_message = message
                .replace("'", "''") // Escape single quotes for PowerShell
                .replace("\"", "`\"") // Escape double quotes
                .chars()
                .take(1000) // Limit message length
                .collect::<String>();

            let script = format!(
                "try {{ \
                 Write-EventLog -LogName Application -Source '{}' -EntryType {} -EventId 1000 -Message '{}' \
                 }} catch {{ \
                 Write-Warning 'Failed to write event log' \
                 }}",
                self.source_name, event_type, clean_message
            );

            // Execute PowerShell command
            let _ = Command::new("powershell")
                .args(&[
                    "-ExecutionPolicy",
                    "Bypass",
                    "-WindowStyle",
                    "Hidden",
                    "-NoProfile",
                    "-Command",
                    &script,
                ])
                .output(); // Ignore errors - event logging is best effort

            Ok(())
        }

        #[cfg(not(windows))]
        {
            // On non-Windows platforms, just log to stderr as fallback
            eprintln!("[{}] {}: {}", self.source_name, level, message);
            Ok(())
        }
    }

    /// Parse log level from tracing output
    fn parse_log_level(line: &str) -> &str {
        if line.contains("ERROR") {
            "ERROR"
        } else if line.contains("WARN") {
            "WARN"
        } else if line.contains("INFO") {
            "INFO"
        } else if line.contains("DEBUG") {
            "DEBUG"
        } else if line.contains("TRACE") {
            "TRACE"
        } else {
            "INFO" // Default fallback
        }
    }

    /// Clean log message by removing ANSI escape codes and control characters
    fn clean_message(message: &str) -> String {
        let mut result = String::new();
        let mut chars = message.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' && chars.peek() == Some(&'[') {
                // Skip ANSI escape sequence
                chars.next(); // consume '['
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else if !ch.is_control() || ch == ' ' {
                result.push(ch);
            }
        }

        // Remove excessive whitespace and newlines
        result
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

impl Write for WindowsEventLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        // Process complete lines
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = self.buffer.drain(..=newline_pos).collect::<Vec<_>>();

            if let Ok(line) = String::from_utf8(line_bytes) {
                let line = line.trim();
                if !line.is_empty() {
                    let level = Self::parse_log_level(line);
                    let cleaned_message = Self::clean_message(line);

                    // Only log meaningful messages to avoid spam
                    if !cleaned_message.is_empty() && cleaned_message.len() > 10 {
                        let _ = self.log_event(level, &cleaned_message);
                    }
                }
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Process any remaining buffer content
        if !self.buffer.is_empty() {
            if let Ok(line) = String::from_utf8(self.buffer.clone()) {
                let line = line.trim();
                if !line.is_empty() {
                    let level = Self::parse_log_level(line);
                    let cleaned_message = Self::clean_message(line);

                    if !cleaned_message.is_empty() && cleaned_message.len() > 10 {
                        let _ = self.log_event(level, &cleaned_message);
                    }
                }
            }
            self.buffer.clear();
        }
        Ok(())
    }
}

/// MakeWriter implementation for tracing-subscriber integration
pub struct EventLogMakeWriter {
    source_name: String,
}

impl EventLogMakeWriter {
    pub fn new(source_name: &str) -> Result<Self> {
        // Ensure event source exists
        let _ = WindowsEventLogWriter::ensure_event_source(source_name);

        Ok(Self {
            source_name: source_name.to_string(),
        })
    }
}

impl<'a> MakeWriter<'a> for EventLogMakeWriter {
    type Writer = EventLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        EventLogWriter {
            buffer: Vec::new(),
            source_name: self.source_name.clone(),
        }
    }
}

/// Individual writer instance for each log event
pub struct EventLogWriter {
    buffer: Vec<u8>,
    source_name: String,
}

impl Write for EventLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for EventLogWriter {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            if let Ok(message) = String::from_utf8(self.buffer.clone()) {
                let level = WindowsEventLogWriter::parse_log_level(&message);
                let cleaned = WindowsEventLogWriter::clean_message(&message);

                if !cleaned.is_empty() && cleaned.len() > 10 {
                    // Create a temporary writer to log this message
                    if let Ok(writer) = WindowsEventLogWriter::new(&self.source_name) {
                        let _ = writer.log_event(level, &cleaned);
                    }
                }
            }
        }
    }
}

/// Register ZipLock as an Event Log source using PowerShell (requires admin privileges)
pub fn register_event_source() -> Result<()> {
    #[cfg(windows)]
    {
        let script = r#"
            try {
                if (-not [System.Diagnostics.EventLog]::SourceExists('ZipLock')) {
                    New-EventLog -LogName Application -Source ZipLock -ErrorAction Stop
                    Write-Host 'ZipLock event source registered successfully'
                } else {
                    Write-Host 'ZipLock event source already exists'
                }
                # Test by writing a registration event
                Write-EventLog -LogName Application -Source ZipLock -EntryType Information -EventId 1000 -Message 'ZipLock Event Log source registered and tested successfully'
                exit 0
            } catch {
                Write-Warning "Failed to register event source: $_"
                exit 1
            }
        "#;

        let output = Command::new("powershell")
            .args(&[
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-Command",
                script,
            ])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("Event Log source registration completed successfully");
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&result.stderr);
                    eprintln!("Warning: Event Log source registration failed: {}", error);
                    // Don't fail catastrophically - the application can still work
                    Ok(())
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to execute PowerShell for event registration: {}",
                    e
                );
                // Return Ok() so the application doesn't fail to start
                Ok(())
            }
        }
    }

    #[cfg(not(windows))]
    {
        Ok(()) // No-op on non-Windows platforms
    }
}

/// Unregister ZipLock event source using PowerShell
pub fn unregister_event_source() -> Result<()> {
    #[cfg(windows)]
    {
        let script = r#"
            try {
                if ([System.Diagnostics.EventLog]::SourceExists('ZipLock')) {
                    Remove-EventLog -Source ZipLock -ErrorAction Stop
                    Write-Host 'ZipLock event source removed successfully'
                } else {
                    Write-Host 'ZipLock event source does not exist'
                }
                exit 0
            } catch {
                Write-Warning "Failed to remove event source: $_"
                exit 1
            }
        "#;

        let _ = Command::new("powershell")
            .args(&[
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-Command",
                script,
            ])
            .output();

        Ok(())
    }

    #[cfg(not(windows))]
    {
        Ok(()) // No-op on non-Windows platforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        assert_eq!(
            WindowsEventLogWriter::parse_log_level("ERROR: Something failed"),
            "ERROR"
        );
        assert_eq!(
            WindowsEventLogWriter::parse_log_level("WARN: Warning message"),
            "WARN"
        );
        assert_eq!(
            WindowsEventLogWriter::parse_log_level("INFO: Information"),
            "INFO"
        );
        assert_eq!(
            WindowsEventLogWriter::parse_log_level("DEBUG: Debug info"),
            "DEBUG"
        );
        assert_eq!(
            WindowsEventLogWriter::parse_log_level("Unknown message"),
            "INFO"
        );
    }

    #[test]
    fn test_clean_message() {
        let message = "\x1b[32mGreen text\x1b[0m with \n multiple \t lines";
        let cleaned = WindowsEventLogWriter::clean_message(message);
        assert_eq!(cleaned, "Green text with multiple lines");
    }

    #[test]
    fn test_event_log_writer_creation() {
        // This test works on all platforms
        let writer = WindowsEventLogWriter::new("ZipLockTest");
        assert!(writer.is_ok());
    }

    #[test]
    fn test_message_cleaning() {
        let message_with_quotes = "Message with 'single' and \"double\" quotes";
        let cleaned = message_with_quotes.replace("'", "''").replace("\"", "`\"");
        assert!(cleaned.contains("''"));
        assert!(cleaned.contains("`\""));
    }
}
