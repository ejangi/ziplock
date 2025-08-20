//! Android Storage Access Framework (SAF) support module
//!
//! This module provides integration with Android's Storage Access Framework
//! to handle content URIs directly in the Rust shared library, avoiding
//! the need to convert content URIs to file paths in the Android app layer.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum AndroidSafError {
    #[error("Android SAF not available - not running on Android or JNI context not set")]
    NotAvailable,

    #[error("Invalid content URI: {uri}")]
    InvalidContentUri { uri: String },

    #[error("Failed to open content URI: {uri}, reason: {reason}")]
    OpenFailed { uri: String, reason: String },

    #[error("Failed to read from content URI: {uri}, reason: {reason}")]
    ReadFailed { uri: String, reason: String },

    #[error("Failed to write to content URI: {uri}, reason: {reason}")]
    WriteFailed { uri: String, reason: String },

    #[error("Content URI not found or access denied: {uri}")]
    AccessDenied { uri: String },

    #[error("JNI operation failed: {reason}")]
    JniError { reason: String },

    #[error("Invalid parameters")]
    InvalidParameters,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for Android SAF operations
pub type AndroidSafResult<T> = Result<T, AndroidSafError>;

/// Function pointer types for Android SAF callbacks
type AndroidSafOpenCallback = extern "C" fn(*const c_char) -> c_int;
type AndroidSafReadCallback = extern "C" fn(c_int, *mut c_void, usize) -> c_int;
type AndroidSafWriteCallback = extern "C" fn(c_int, *const c_void, usize) -> c_int;
type AndroidSafCloseCallback = extern "C" fn(c_int) -> c_int;
type AndroidSafGetSizeCallback = extern "C" fn(c_int) -> i64;
type AndroidSafCreateTempFileCallback = extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;

/// Android SAF callbacks provided by the Android app via JNI
#[derive(Debug, Clone)]
pub struct AndroidSafCallbacks {
    /// Open a content URI and return a file descriptor
    pub open_content_uri: AndroidSafOpenCallback,
    /// Read from a file descriptor
    pub read_from_fd: AndroidSafReadCallback,
    /// Write to a file descriptor
    pub write_to_fd: AndroidSafWriteCallback,
    /// Close a file descriptor
    pub close_fd: AndroidSafCloseCallback,
    /// Get file size from file descriptor
    pub get_file_size: AndroidSafGetSizeCallback,
    /// Create a temporary file and return its path
    pub create_temp_file: AndroidSafCreateTempFileCallback,
}

/// Global Android SAF context
static ANDROID_SAF_CONTEXT: Mutex<Option<AndroidSafCallbacks>> = Mutex::new(None);

/// Handle for working with Android content URIs
#[derive(Debug)]
pub struct AndroidSafHandle {
    /// The content URI
    content_uri: String,
    /// File descriptor from Android
    fd: c_int,
    /// Local temporary file path (for operations requiring file paths)
    temp_file_path: Option<PathBuf>,
    /// File size in bytes
    file_size: u64,
    /// Whether the content has been modified
    modified: bool,
    /// Original content hash for conflict detection
    original_hash: Option<String>,
}

impl AndroidSafHandle {
    /// Create a new handle for the given content URI
    pub fn new(content_uri: &str) -> AndroidSafResult<Self> {
        info!("AndroidSafHandle::new() called with URI: {}", content_uri);

        let callbacks = get_android_saf_callbacks().map_err(|e| {
            error!("DETAILED ERROR: Failed to get Android SAF callbacks: {}", e);
            error!("This usually means SAF was not properly initialized from the Android app");
            error!("Content URI being tested: {}", content_uri);
            e
        })?;

        info!("Got Android SAF callbacks successfully");

        // Convert URI to C string
        let uri_cstr = CString::new(content_uri).map_err(|e| {
            error!("Failed to convert URI to C string: {}", e);
            AndroidSafError::InvalidContentUri {
                uri: content_uri.to_string(),
            }
        })?;

        info!("Converted URI to C string, calling open_content_uri callback");

        // Open the content URI with panic protection
        let fd = match std::panic::catch_unwind(|| (callbacks.open_content_uri)(uri_cstr.as_ptr()))
        {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Android SAF callback panicked when opening content URI: {}",
                    content_uri
                );
                return Err(AndroidSafError::JniError {
                    reason: "Callback function panicked during open_content_uri call".to_string(),
                });
            }
        };

        info!("open_content_uri callback returned fd: {}", fd);

        if fd < 0 {
            error!(
                "Failed to open content URI: {}, error code: {}",
                content_uri, fd
            );
            return Err(AndroidSafError::OpenFailed {
                uri: content_uri.to_string(),
                reason: format!("Android returned error code: {}", fd),
            });
        }

        info!(
            "Successfully got file descriptor: {}, getting file size",
            fd
        );

        // Get file size with panic protection
        let size_result = match std::panic::catch_unwind(|| (callbacks.get_file_size)(fd)) {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Android SAF callback panicked when getting file size for content URI: {}",
                    content_uri
                );
                // Close the fd before returning error
                let _ = std::panic::catch_unwind(|| (callbacks.close_fd)(fd));
                return Err(AndroidSafError::JniError {
                    reason: "Callback function panicked during get_file_size call".to_string(),
                });
            }
        };
        info!("get_file_size callback returned: {}", size_result);

        let file_size = if size_result >= 0 {
            size_result as u64
        } else {
            error!(
                "DETAILED ERROR: Failed to get file size for {}, got error code: {}",
                content_uri, size_result
            );
            error!("This suggests the file descriptor is invalid or the callback failed");
            // Close the fd before returning error
            let _ = std::panic::catch_unwind(|| (callbacks.close_fd)(fd));
            return Err(AndroidSafError::ReadFailed {
                uri: content_uri.to_string(),
                reason: format!("Failed to get file size, error code: {}", size_result),
            });
        };

        info!(
            "Successfully opened content URI: {} (fd: {}, size: {} bytes)",
            content_uri, fd, file_size
        );

        // Validate file size
        info!(
            "DETAILED INFO: File size validation for URI {}: {} bytes",
            content_uri, file_size
        );

        if file_size == 0 {
            error!("DETAILED ERROR: Content URI has zero size: {}", content_uri);
            // Close the fd before returning error
            let _ = std::panic::catch_unwind(|| (callbacks.close_fd)(fd));
            return Err(AndroidSafError::ReadFailed {
                uri: content_uri.to_string(),
                reason: "File has zero size - cannot be a valid archive".to_string(),
            });
        } else if file_size < 32 {
            error!("DETAILED ERROR: Content URI has very small size ({} bytes), too small for 7z header: {}", file_size, content_uri);
            // Close the fd before returning error
            let _ = std::panic::catch_unwind(|| (callbacks.close_fd)(fd));
            return Err(AndroidSafError::ReadFailed {
                uri: content_uri.to_string(),
                reason: format!("File size ({} bytes) is too small to be a valid 7z archive. Minimum expected size is 32 bytes for the signature header.", file_size),
            });
        } else if file_size < 200 {
            warn!(
                "DETAILED WARNING: Small file size ({} bytes), may be newly created archive: {}",
                file_size, content_uri
            );
        } else if file_size < 500 {
            info!(
                "DETAILED INFO: Small file size ({} bytes), likely minimal archive: {}",
                file_size, content_uri
            );
        } else {
            info!(
                "DETAILED INFO: File size ({} bytes) appears normal for archive: {}",
                file_size, content_uri
            );
        }

        let handle = AndroidSafHandle {
            content_uri: content_uri.to_string(),
            fd,
            temp_file_path: None,
            file_size,
            modified: false,
            original_hash: None,
        };

        info!("AndroidSafHandle created successfully");
        Ok(handle)
    }

    /// Get the content URI
    pub fn content_uri(&self) -> &str {
        &self.content_uri
    }

    /// Get file size in bytes
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Check if the content has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Read all content into a Vec<u8> using chunked reading to avoid hanging
    pub fn read_all(&self) -> AndroidSafResult<Vec<u8>> {
        let callbacks = get_android_saf_callbacks()?;

        if self.file_size == 0 {
            return Ok(Vec::new());
        }

        // Use chunked reading to avoid large single reads that can cause Android to hang
        const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
        const MAX_READ_TIME_SECS: u64 = 30; // 30 second timeout for entire read operation

        let start_time = std::time::Instant::now();
        let mut result = Vec::with_capacity(self.file_size as usize);
        let mut total_read = 0usize;
        let mut consecutive_failures = 0;
        const MAX_CONSECUTIVE_FAILURES: u32 = 3;

        info!(
            "Starting chunked read of {} bytes from content URI: {}",
            self.file_size, self.content_uri
        );

        while total_read < self.file_size as usize {
            // Check for timeout
            if start_time.elapsed().as_secs() > MAX_READ_TIME_SECS {
                return Err(AndroidSafError::ReadFailed {
                    uri: self.content_uri.clone(),
                    reason: format!(
                        "Read operation timed out after {} seconds (read {}/{} bytes)",
                        MAX_READ_TIME_SECS, total_read, self.file_size
                    ),
                });
            }

            let remaining = (self.file_size as usize) - total_read;
            let chunk_size = std::cmp::min(CHUNK_SIZE, remaining);

            let mut chunk_buffer = vec![0u8; chunk_size];

            debug!(
                "Reading chunk {} of {} bytes at offset {} from content URI: {}",
                (total_read / CHUNK_SIZE) + 1,
                chunk_size,
                total_read,
                self.content_uri
            );

            let bytes_read = (callbacks.read_from_fd)(
                self.fd,
                chunk_buffer.as_mut_ptr() as *mut c_void,
                chunk_size,
            );

            if bytes_read < 0 {
                consecutive_failures += 1;
                warn!(
                    "Read failed with error code: {} at offset {} (failure #{}/{})",
                    bytes_read, total_read, consecutive_failures, MAX_CONSECUTIVE_FAILURES
                );

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    return Err(AndroidSafError::ReadFailed {
                        uri: self.content_uri.clone(),
                        reason: format!(
                            "Android returned error code: {} at offset {} after {} consecutive failures",
                            bytes_read, total_read, consecutive_failures
                        ),
                    });
                }

                // Brief delay before retry
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            // Reset failure counter on successful read
            consecutive_failures = 0;

            if bytes_read == 0 {
                // End of file reached earlier than expected
                warn!(
                    "End of file reached at {} bytes, expected {} bytes for URI: {}",
                    total_read, self.file_size, self.content_uri
                );
                break;
            }

            let bytes_read = bytes_read as usize;
            chunk_buffer.truncate(bytes_read);
            result.extend_from_slice(&chunk_buffer);
            total_read += bytes_read;

            debug!(
                "Read chunk: {} bytes (total: {}/{}) from content URI: {}",
                bytes_read, total_read, self.file_size, self.content_uri
            );

            // Small delay to prevent overwhelming the Android content resolver
            if total_read < self.file_size as usize {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        let elapsed = start_time.elapsed();
        info!(
            "Completed reading {} bytes from content URI: {} in {:.2}s",
            total_read,
            self.content_uri,
            elapsed.as_secs_f64()
        );
        Ok(result)
    }

    /// Write content to the URI (this may not be supported for all content URIs)
    pub fn write_all(&mut self, data: &[u8]) -> AndroidSafResult<()> {
        let callbacks = get_android_saf_callbacks()?;

        let bytes_written =
            (callbacks.write_to_fd)(self.fd, data.as_ptr() as *const c_void, data.len());

        if bytes_written < 0 {
            return Err(AndroidSafError::WriteFailed {
                uri: self.content_uri.clone(),
                reason: format!("Android returned error code: {}", bytes_written),
            });
        }

        if bytes_written as usize != data.len() {
            return Err(AndroidSafError::WriteFailed {
                uri: self.content_uri.clone(),
                reason: format!(
                    "Expected to write {} bytes, but wrote {}",
                    data.len(),
                    bytes_written
                ),
            });
        }

        self.modified = true;
        self.file_size = data.len() as u64;
        info!(
            "Wrote {} bytes to content URI: {}",
            bytes_written, self.content_uri
        );
        Ok(())
    }

    /// Create a temporary file copy for operations that require a file path
    /// Returns the path to the temporary file
    pub fn create_temp_file_copy(&mut self) -> AndroidSafResult<&Path> {
        info!(
            "create_temp_file_copy() called for URI: {}",
            self.content_uri
        );

        if let Some(ref path) = self.temp_file_path {
            info!("Temporary file already exists: {:?}", path);
            return Ok(path);
        }

        info!("Getting SAF callbacks for temporary file creation");
        let callbacks = get_android_saf_callbacks().map_err(|e| {
            error!("Failed to get SAF callbacks: {}", e);
            e
        })?;

        // Create a temporary file name based on the content URI
        let temp_name = format!(
            "ziplock_saf_{}",
            self.content_uri
                .replace("content://", "")
                .replace("/", "_")
                .replace(":", "_")
                .replace("%", "_")
        );

        info!("Generated temporary file name: {}", temp_name);

        let temp_name_cstr = CString::new(temp_name.clone()).map_err(|e| {
            error!("Failed to convert temp name to C string: {}", e);
            AndroidSafError::InvalidParameters
        })?;

        info!("Calling create_temp_file callback");

        // Get temporary file path from Android
        let mut temp_path_ptr: *mut c_char = std::ptr::null_mut();
        let result = (callbacks.create_temp_file)(temp_name_cstr.as_ptr(), &mut temp_path_ptr);

        info!(
            "create_temp_file callback returned: {}, path_ptr: {:?}",
            result, temp_path_ptr
        );

        if result != 0 || temp_path_ptr.is_null() {
            error!(
                "Failed to create temporary file, Android returned: {}, ptr is null: {}",
                result,
                temp_path_ptr.is_null()
            );
            return Err(AndroidSafError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to create temporary file, Android returned: {}",
                    result
                ),
            )));
        }

        // Convert C string to Rust string
        let temp_path_str = unsafe {
            let cstr = CStr::from_ptr(temp_path_ptr);
            cstr.to_str()
                .map_err(|e| {
                    error!("Failed to convert temp path to UTF-8: {}", e);
                    AndroidSafError::JniError {
                        reason: "Invalid UTF-8 in temporary file path".to_string(),
                    }
                })?
                .to_string()
        };

        info!("Got temporary file path: {}", temp_path_str);

        let temp_path = PathBuf::from(temp_path_str);

        // Validate file size before attempting to read
        if self.file_size == 0 {
            error!("Cannot create temp file copy: source file has zero size");
            return Err(AndroidSafError::ReadFailed {
                uri: self.content_uri.clone(),
                reason: "Source file has zero size".to_string(),
            });
        }

        // Read content in chunks and write to temporary file to avoid memory issues
        info!(
            "Reading content from URI: {} ({} bytes) to create temporary file copy",
            self.content_uri, self.file_size
        );

        let content = self.read_all().map_err(|e| {
            error!(
                "Failed to read content from URI {}: {}",
                self.content_uri, e
            );
            e
        })?;

        info!(
            "Successfully read {} bytes, writing to temporary file",
            content.len()
        );

        // Validate content size
        if content.is_empty() {
            error!("Read empty content from URI: {}", self.content_uri);
            return Err(AndroidSafError::ReadFailed {
                uri: self.content_uri.clone(),
                reason: "Read empty content from content URI".to_string(),
            });
        }

        if content.len() != self.file_size as usize {
            warn!(
                "Content size mismatch: expected {} bytes, got {} bytes",
                self.file_size,
                content.len()
            );
        }

        std::fs::write(&temp_path, &content).map_err(|e| {
            error!("Failed to write temporary file {:?}: {}", temp_path, e);
            AndroidSafError::Io(e)
        })?;

        // Verify the written file
        let written_size = std::fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);

        info!(
            "Successfully created temporary file: {:?} (wrote {} bytes, file size {} bytes)",
            temp_path,
            content.len(),
            written_size
        );

        self.temp_file_path = Some(temp_path);
        info!("Temporary file creation completed successfully");

        Ok(self.temp_file_path.as_ref().unwrap())
    }

    /// Sync changes from temporary file back to content URI (if modified)
    pub fn sync_temp_file_back(&mut self) -> AndroidSafResult<()> {
        if let Some(ref temp_path) = self.temp_file_path {
            if temp_path.exists() {
                let content = std::fs::read(temp_path)?;
                self.write_all(&content)?;
                info!(
                    "Synced temporary file changes back to content URI: {}",
                    self.content_uri
                );
            }
        }
        Ok(())
    }
}

impl Drop for AndroidSafHandle {
    fn drop(&mut self) {
        // Sync back any changes from temporary file
        if self.modified {
            if let Err(e) = self.sync_temp_file_back() {
                error!(
                    "Failed to sync changes back to content URI {}: {}",
                    self.content_uri, e
                );
            }
        }

        // Clean up temporary file
        if let Some(ref temp_path) = self.temp_file_path {
            if temp_path.exists() {
                if let Err(e) = std::fs::remove_file(temp_path) {
                    warn!("Failed to clean up temporary file {:?}: {}", temp_path, e);
                }
            }
        }

        // Close file descriptor
        if let Ok(callbacks) = get_android_saf_callbacks() {
            let result = (callbacks.close_fd)(self.fd);
            if result != 0 {
                warn!(
                    "Failed to close file descriptor {} for content URI {}: error code {}",
                    self.fd, self.content_uri, result
                );
            } else {
                debug!(
                    "Closed file descriptor {} for content URI: {}",
                    self.fd, self.content_uri
                );
            }
        }
    }
}

/// Check if the given path is an Android content URI
pub fn is_content_uri(path: &str) -> bool {
    path.starts_with("content://")
}

/// Check if Android SAF is available (callbacks have been set)
pub fn is_android_saf_available() -> bool {
    ANDROID_SAF_CONTEXT.lock().unwrap().is_some()
}

/// FFI export for checking if Android SAF is available
#[no_mangle]
pub extern "C" fn ziplock_android_saf_is_available() -> c_int {
    if is_android_saf_available() {
        1 // True
    } else {
        0 // False
    }
}

/// Get the Android SAF callbacks (internal use)
fn get_android_saf_callbacks() -> AndroidSafResult<AndroidSafCallbacks> {
    let context_guard = ANDROID_SAF_CONTEXT.lock().unwrap();
    context_guard
        .as_ref()
        .cloned()
        .ok_or(AndroidSafError::NotAvailable)
}

/// Initialize Android SAF with callbacks from the Android app
/// This should be called from JNI when the Android app starts
#[no_mangle]
pub extern "C" fn ziplock_android_saf_init(
    open_content_uri: AndroidSafOpenCallback,
    read_from_fd: AndroidSafReadCallback,
    write_to_fd: AndroidSafWriteCallback,
    close_fd: AndroidSafCloseCallback,
    get_file_size: AndroidSafGetSizeCallback,
    create_temp_file: AndroidSafCreateTempFileCallback,
) -> c_int {
    let callbacks = AndroidSafCallbacks {
        open_content_uri,
        read_from_fd,
        write_to_fd,
        close_fd,
        get_file_size,
        create_temp_file,
    };

    {
        let mut context_guard = ANDROID_SAF_CONTEXT.lock().unwrap();
        *context_guard = Some(callbacks);
    }

    info!("Android SAF callbacks initialized successfully");
    0 // Success
}

/// Cleanup Android SAF context
#[no_mangle]
pub extern "C" fn ziplock_android_saf_cleanup() -> c_int {
    {
        let mut context_guard = ANDROID_SAF_CONTEXT.lock().unwrap();
        *context_guard = None;
    }

    info!("Android SAF context cleaned up");
    0 // Success
}

/// Test function to verify Android SAF functionality
#[no_mangle]
pub extern "C" fn ziplock_android_saf_test(content_uri: *const c_char) -> c_int {
    if content_uri.is_null() {
        return -1;
    }

    let uri_str = unsafe {
        match CStr::from_ptr(content_uri).to_str() {
            Ok(s) => s,
            Err(_) => return -2,
        }
    };

    match AndroidSafHandle::new(uri_str) {
        Ok(handle) => {
            info!(
                "DETAILED SUCCESS: Android SAF test successful for URI: {} (size: {} bytes)",
                uri_str,
                handle.file_size()
            );
            0 // Success
        }
        Err(e) => {
            error!(
                "DETAILED ERROR: Android SAF test failed for URI {}: {}",
                uri_str, e
            );
            error!("Error type: {:?}", e);
            error!("This indicates the AndroidSafHandle::new() function failed");
            error!(
                "Common causes: file too small, SAF callbacks not working, or file access issues"
            );
            -3 // Failure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_content_uri() {
        assert!(is_content_uri("content://com.android.externalstorage.documents/document/primary%3ADocuments%2FZipLock.7z"));
        assert!(is_content_uri(
            "content://com.google.android.apps.docs.files/document/test.7z"
        ));
        assert!(!is_content_uri("/storage/emulated/0/Documents/test.7z"));
        assert!(!is_content_uri("file:///storage/emulated/0/test.7z"));
        assert!(!is_content_uri(""));
    }

    #[test]
    fn test_android_saf_not_available_by_default() {
        assert!(!is_android_saf_available());
    }
}
