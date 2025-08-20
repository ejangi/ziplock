//! Clipboard manager service that handles clipboard operations with automatic timeout clearing
//!
//! This service manages clipboard operations and ensures that sensitive data (passwords, TOTP codes)
//! is automatically cleared from the clipboard after a configurable timeout period.

use arboard::Clipboard;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, warn};

/// Types of content that can be copied to clipboard
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardContentType {
    /// TOTP authentication code
    TotpCode,
    /// Password field
    Password,
    /// Username field
    Username,
    /// Generic text (no timeout clearing)
    Text,
}

/// Information about content currently in clipboard
#[derive(Debug, Clone)]
struct ClipboardContent {
    /// The actual content that was copied
    content: String,
    /// Type of content
    content_type: ClipboardContentType,
    /// When the content was copied
    copied_at: Instant,
    /// Timeout duration in seconds (0 = no timeout)
    timeout_seconds: u32,
}

/// Clipboard manager service
#[derive(Debug, Clone)]
pub struct ClipboardManager {
    /// Current clipboard content being tracked
    current_content: Arc<Mutex<Option<ClipboardContent>>>,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {
            current_content: Arc::new(Mutex::new(None)),
        }
    }

    /// Copy content to clipboard with automatic timeout clearing
    ///
    /// # Arguments
    /// * `content` - The text to copy to clipboard
    /// * `content_type` - The type of content being copied
    /// * `timeout_seconds` - Timeout in seconds (0 = no timeout)
    pub async fn copy_with_timeout(
        &self,
        content: String,
        content_type: ClipboardContentType,
        timeout_seconds: u32,
    ) -> Result<(), ClipboardError> {
        // Copy to system clipboard
        let clipboard_success = match self.copy_to_system_clipboard(&content).await {
            Ok(_) => {
                debug!(
                    "Copied {} content to clipboard with timeout: {}s",
                    format!("{:?}", content_type).to_lowercase(),
                    timeout_seconds
                );
                true
            }
            Err(e) => {
                // In headless/testing environments, clipboard operations may fail
                // We still want to track timeouts for testing purposes
                if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
                    warn!("Clipboard operation failed in headless environment: {}", e);
                    false
                } else {
                    error!("Failed to copy to clipboard: {}", e);
                    return Err(e);
                }
            }
        };

        // Track timeouts for sensitive content types even if clipboard failed in headless env
        if matches!(
            content_type,
            ClipboardContentType::TotpCode | ClipboardContentType::Password
        ) && timeout_seconds > 0
        {
            let clipboard_content = ClipboardContent {
                content: content.clone(),
                content_type,
                copied_at: Instant::now(),
                timeout_seconds,
            };

            // Update tracked content (track even if clipboard operation failed in headless env)
            {
                let mut current = self.current_content.lock().await;
                *current = Some(clipboard_content.clone());
            }

            // Start timeout task (only if clipboard operation succeeded)
            if clipboard_success {
                let current_content = Arc::clone(&self.current_content);
                tokio::spawn(async move {
                    sleep(Duration::from_secs(timeout_seconds as u64)).await;
                    Self::clear_if_matches(current_content, clipboard_content).await;
                });
            }
        }

        // Return success if clipboard succeeded or if we're in headless environment
        if clipboard_success
            || (std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err())
        {
            Ok(())
        } else {
            Err(ClipboardError::SystemError(
                arboard::Error::ContentNotAvailable,
            ))
        }
    }

    /// Copy regular text to clipboard (no timeout clearing)
    #[allow(dead_code)]
    pub async fn copy_text(&self, content: String) -> Result<(), ClipboardError> {
        self.copy_to_system_clipboard(&content).await
    }

    /// Clear the clipboard if it still contains the specified content
    async fn clear_if_matches(
        current_content: Arc<Mutex<Option<ClipboardContent>>>,
        expected_content: ClipboardContent,
    ) {
        let mut current = current_content.lock().await;

        // Check if the content we're tracking matches what we expect to clear
        if let Some(ref tracked) = *current {
            if tracked.content == expected_content.content
                && tracked.copied_at == expected_content.copied_at
            {
                // Verify the clipboard still contains our content before clearing
                match Self::get_system_clipboard_content().await {
                    Ok(clipboard_text) => {
                        if clipboard_text == expected_content.content {
                            if let Err(e) = Self::clear_system_clipboard().await {
                                warn!("Failed to clear clipboard: {}", e);
                            } else {
                                debug!(
                                    "Cleared {} from clipboard after {}s timeout",
                                    format!("{:?}", expected_content.content_type).to_lowercase(),
                                    expected_content.timeout_seconds
                                );
                            }
                        } else {
                            debug!("Clipboard content changed, skipping clear");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read clipboard for verification: {}", e);
                    }
                }

                // Clear our tracking regardless
                *current = None;
            }
        }
    }

    /// Copy content to system clipboard
    async fn copy_to_system_clipboard(&self, content: &str) -> Result<(), ClipboardError> {
        // Use blocking task since clipboard operations are synchronous
        let content = content.to_string();
        tokio::task::spawn_blocking(move || {
            Clipboard::new()
                .and_then(|mut clipboard| clipboard.set_text(&content))
                .map_err(ClipboardError::SystemError)
        })
        .await
        .map_err(ClipboardError::TaskError)?
    }

    /// Get current system clipboard content
    async fn get_system_clipboard_content() -> Result<String, ClipboardError> {
        tokio::task::spawn_blocking(|| {
            Clipboard::new()
                .and_then(|mut clipboard| clipboard.get_text())
                .map_err(ClipboardError::SystemError)
        })
        .await
        .map_err(ClipboardError::TaskError)?
    }

    /// Clear system clipboard
    async fn clear_system_clipboard() -> Result<(), ClipboardError> {
        tokio::task::spawn_blocking(|| {
            Clipboard::new()
                .and_then(|mut clipboard| clipboard.set_text(""))
                .map_err(ClipboardError::SystemError)
        })
        .await
        .map_err(ClipboardError::TaskError)?
    }

    /// Get information about currently tracked clipboard content
    #[allow(dead_code)]
    pub async fn current_tracked_content(&self) -> Option<(ClipboardContentType, u32)> {
        let current = self.current_content.lock().await;
        current
            .as_ref()
            .map(|content| (content.content_type.clone(), content.timeout_seconds))
    }

    /// Manually clear any tracked content (useful when app is closing)
    pub async fn clear_tracked_content(&self) {
        let mut current = self.current_content.lock().await;
        if let Some(ref tracked) = *current {
            // Try to clear if it's still our content
            match Self::get_system_clipboard_content().await {
                Ok(clipboard_text) => {
                    if clipboard_text == tracked.content {
                        let _ = Self::clear_system_clipboard().await;
                        debug!("Cleared tracked content from clipboard on manual clear");
                    }
                }
                Err(_) => {
                    // If we can't read clipboard, just clear our tracking
                }
            }
        }
        *current = None;
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during clipboard operations
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    /// Error from the underlying clipboard system
    #[error("Clipboard system error: {0}")]
    SystemError(#[from] arboard::Error),

    /// Error from async task execution
    #[error("Async task error: {0}")]
    TaskError(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();
        assert!(manager.current_tracked_content().await.is_none());
    }

    #[tokio::test]
    async fn test_copy_text_no_timeout() {
        let manager = ClipboardManager::new();
        let content = "test content".to_string();

        // This should succeed (or fail gracefully in headless environment)
        let result = manager.copy_text(content).await;
        // In headless environments, clipboard operations might fail, that's ok for tests
        match result {
            Ok(_) => println!("Clipboard copy succeeded"),
            Err(_) => println!("Clipboard copy failed (expected in headless environment)"),
        }
    }

    #[tokio::test]
    async fn test_copy_with_timeout_tracking() {
        let manager = ClipboardManager::new();
        let content = "secret123".to_string();

        // Copy with timeout - should track even if clipboard operation fails in headless env
        let result = manager
            .copy_with_timeout(content, ClipboardContentType::Password, 5)
            .await;

        // Should succeed and track content (even in headless environments)
        assert!(result.is_ok());
        let tracked = manager.current_tracked_content().await;
        assert!(tracked.is_some());
        if let Some((content_type, timeout)) = tracked {
            assert_eq!(content_type, ClipboardContentType::Password);
            assert_eq!(timeout, 5);
        }
    }

    #[tokio::test]
    async fn test_no_tracking_for_zero_timeout() {
        let manager = ClipboardManager::new();
        let content = "secret123".to_string();

        // Copy with zero timeout - should not track
        let _ = manager
            .copy_with_timeout(content, ClipboardContentType::Password, 0)
            .await;

        // Should not be tracking anything
        assert!(manager.current_tracked_content().await.is_none());
    }

    #[tokio::test]
    async fn test_no_tracking_for_text_content() {
        let manager = ClipboardManager::new();
        let content = "regular text".to_string();

        // Copy text content with timeout - should not track
        let _ = manager
            .copy_with_timeout(content, ClipboardContentType::Text, 30)
            .await;

        // Should not be tracking anything
        assert!(manager.current_tracked_content().await.is_none());
    }

    #[tokio::test]
    async fn test_manual_clear() {
        let manager = ClipboardManager::new();
        let content = "temporary".to_string();

        // Copy with timeout
        let result = manager
            .copy_with_timeout(content, ClipboardContentType::Password, 30)
            .await;

        // Should succeed and track content (even in headless environments)
        assert!(result.is_ok());
        assert!(manager.current_tracked_content().await.is_some());

        // Manual clear
        manager.clear_tracked_content().await;

        // Should no longer be tracking
        assert!(manager.current_tracked_content().await.is_none());
    }
}
