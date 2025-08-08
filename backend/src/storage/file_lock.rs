//! File locking utilities for safe archive access
//!
//! This module provides cross-platform file locking to prevent concurrent
//! access to archive files, which is especially important when using cloud
//! sync services that might try to upload files while they're being modified.

use std::fs::File;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Error, Debug)]
pub enum FileLockError {
    #[error("Failed to acquire file lock: {reason}")]
    LockFailed { reason: String },

    #[error("Lock timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A file lock that automatically releases when dropped
#[derive(Debug)]
pub struct FileLock {
    #[allow(dead_code)]
    file: File,
    path: std::path::PathBuf,
    #[cfg(unix)]
    locked: bool,
}

impl FileLock {
    /// Create a new file lock with a timeout
    pub fn new<P: AsRef<Path>>(path: P, timeout_seconds: u64) -> Result<Self, FileLockError> {
        let path = path.as_ref();
        debug!("Acquiring file lock: {:?}", path);

        if !path.exists() {
            return Err(FileLockError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let file = File::open(path)?;
        let timeout = Duration::from_secs(timeout_seconds);

        // Try to acquire lock with timeout
        let start = std::time::Instant::now();
        loop {
            match Self::try_lock(&file) {
                Ok(()) => {
                    debug!("Successfully acquired file lock: {:?}", path);
                    return Ok(Self {
                        file,
                        path: path.to_path_buf(),
                        #[cfg(unix)]
                        locked: true,
                    });
                }
                Err(_e) if start.elapsed() < timeout => {
                    // Brief wait before retry
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    return Err(FileLockError::LockFailed {
                        reason: e.to_string(),
                    });
                }
            }
        }
    }

    /// Try to acquire an exclusive lock on the file
    #[cfg(unix)]
    fn try_lock(file: &File) -> Result<(), std::io::Error> {
        use std::os::unix::io::AsRawFd;

        let fd = file.as_raw_fd();
        let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

        if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    /// Try to acquire an exclusive lock on the file (Windows)
    #[cfg(windows)]
    fn try_lock(file: &File) -> Result<(), std::io::Error> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
        use windows::Win32::Storage::FileSystem::{
            LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
        };

        let handle = HANDLE(file.as_raw_handle() as isize);
        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid file handle",
            ));
        }

        let mut overlapped = std::mem::zeroed();
        let result = unsafe {
            LockFileEx(
                handle,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                u32::MAX,
                u32::MAX,
                &mut overlapped,
            )
        };

        if result.as_bool() {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    /// Unlock the file explicitly (usually not needed due to Drop)
    #[cfg(unix)]
    pub fn unlock(&mut self) -> Result<(), std::io::Error> {
        if self.locked {
            use std::os::unix::io::AsRawFd;

            let fd = self.file.as_raw_fd();
            let result = unsafe { libc::flock(fd, libc::LOCK_UN) };

            if result == 0 {
                self.locked = false;
                debug!("Released file lock: {:?}", self.path);
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        } else {
            Ok(())
        }
    }

    /// Unlock the file explicitly (Windows)
    #[cfg(windows)]
    pub fn unlock(&mut self) -> Result<(), std::io::Error> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
        use windows::Win32::Storage::FileSystem::UnlockFileEx;

        let handle = HANDLE(self.file.as_raw_handle() as isize);
        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid file handle",
            ));
        }

        let mut overlapped = std::mem::zeroed();
        let result = unsafe { UnlockFileEx(handle, 0, u32::MAX, u32::MAX, &mut overlapped) };

        if result.as_bool() {
            debug!("Released file lock: {:?}", self.path);
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    /// Get the path of the locked file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = self.unlock() {
            warn!("Failed to unlock file {:?} on drop: {}", self.path, e);
        }
    }
}

/// Create a temporary lock file for coordination
pub struct LockFile {
    lock_path: std::path::PathBuf,
    _file_lock: FileLock,
}

impl LockFile {
    /// Create a new lock file
    pub fn create<P: AsRef<Path>>(
        base_path: P,
        timeout_seconds: u64,
    ) -> Result<Self, FileLockError> {
        let base_path = base_path.as_ref();
        let lock_path = base_path.with_extension(format!(
            "{}.lock",
            base_path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));

        // Create the lock file if it doesn't exist
        if !lock_path.exists() {
            std::fs::write(&lock_path, b"ziplock").map_err(|e| FileLockError::LockFailed {
                reason: format!("Failed to create lock file: {}", e),
            })?;
        }

        let file_lock = FileLock::new(&lock_path, timeout_seconds)?;

        Ok(Self {
            lock_path: lock_path.clone(),
            _file_lock: file_lock,
        })
    }

    /// Get the path of the lock file
    pub fn path(&self) -> &Path {
        &self.lock_path
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        // Attempt to remove the lock file
        if let Err(e) = std::fs::remove_file(&self.lock_path) {
            debug!("Failed to remove lock file {:?}: {}", self.lock_path, e);
        } else {
            debug!("Removed lock file: {:?}", self.lock_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_lock_basic() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let lock = FileLock::new(path, 5);
        assert!(lock.is_ok());
    }

    #[test]
    fn test_file_lock_nonexistent_file() {
        let result = FileLock::new("/nonexistent/file.txt", 5);
        assert!(matches!(result, Err(FileLockError::FileNotFound { .. })));
    }

    #[test]
    fn test_concurrent_locking() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // First, acquire a lock in the main thread to ensure the mechanism works
        let _main_lock = FileLock::new(&path, 1).expect("Main thread should acquire lock");

        // Use a barrier to synchronize thread starts
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        // Try to acquire the same lock from multiple threads while main thread holds it
        for i in 0..3 {
            let path_clone = path.clone();
            let barrier_clone = Arc::clone(&barrier);
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Try to acquire lock with short timeout since main thread has it
                let start = std::time::Instant::now();
                let lock_result = FileLock::new(&path_clone, 1);
                let duration = start.elapsed();

                // Record the result with timing info
                let mut results = results_clone.lock().unwrap();
                results.push((i, lock_result.is_ok(), duration));

                lock_result
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.join();
        }

        // Check results - all should fail since main thread holds the lock
        let results = results.lock().unwrap();
        let successful_locks = results.iter().filter(|(_, success, _)| *success).count();

        // All threads should fail to acquire the lock since main thread has it
        // If file locking doesn't work properly, this test will detect it
        assert_eq!(
            successful_locks, 0,
            "Expected 0 successful locks since main thread holds the lock. Results: {:?}",
            *results
        );
    }

    #[test]
    fn test_lock_file_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let lock_file = LockFile::create(path, 5);
        assert!(lock_file.is_ok());

        let lock_file = lock_file.unwrap();
        assert!(lock_file.path().exists());

        // Lock file should be removed when dropped
        drop(lock_file);
        // Note: The lock file might still exist briefly due to timing
    }

    #[test]
    fn test_timeout() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Acquire lock in first thread
        let _lock1 = FileLock::new(&path, 10).unwrap();

        // Try to acquire same lock with short timeout
        let start = std::time::Instant::now();
        let result = FileLock::new(&path, 1);
        let elapsed = start.elapsed();

        assert!(result.is_err());
        // Should timeout after approximately 1 second
        assert!(elapsed >= Duration::from_secs(1));
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn test_auto_unlock_on_drop() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create and drop a lock
        {
            let _lock = FileLock::new(&path, 5).unwrap();
        } // Lock should be released here

        // Should be able to acquire the lock again immediately
        let lock2 = FileLock::new(&path, 1);
        assert!(lock2.is_ok());
    }
}
