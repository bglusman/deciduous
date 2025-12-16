//! Database lock file management
//!
//! Ensures only one deciduous process can access the database at a time.
//! Uses file-based locking for cross-platform compatibility.

use fs4::fs_std::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Error types for lock operations
#[derive(Debug)]
pub enum LockError {
    /// Another process holds the lock
    AlreadyLocked {
        pid: String,
        lock_path: PathBuf,
    },
    /// Failed to create or access lock file
    IoError(std::io::Error),
    /// Lock file exists but process is stale
    StaleLock {
        pid: String,
        lock_path: PathBuf,
    },
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::AlreadyLocked { pid, lock_path } => {
                write!(
                    f,
                    "Database locked by another deciduous process (PID {})\n\
                     Lock file: {}\n\n\
                     If you believe this is stale, run: deciduous unlock",
                    pid,
                    lock_path.display()
                )
            }
            LockError::IoError(e) => write!(f, "Lock file error: {}", e),
            LockError::StaleLock { pid, lock_path } => {
                write!(
                    f,
                    "Stale lock detected (PID {} no longer running)\n\
                     Lock file: {}\n\n\
                     Run: deciduous unlock",
                    pid,
                    lock_path.display()
                )
            }
        }
    }
}

impl std::error::Error for LockError {}

impl From<std::io::Error> for LockError {
    fn from(e: std::io::Error) -> Self {
        LockError::IoError(e)
    }
}

/// Guard that holds the lock and releases it on drop
pub struct LockGuard {
    /// File handle - kept open to maintain the lock.
    /// The lock is released when this file is dropped.
    #[allow(dead_code)]
    file: File,
    path: PathBuf,
}

impl LockGuard {
    /// Get the path to the lock file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // File lock is automatically released when the file is dropped.
        // We just need to remove the lock file.
        // Note: The file will be closed/unlocked when self.file is dropped
        // after this Drop impl finishes.
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Acquire an exclusive lock on the deciduous database
///
/// Returns a `LockGuard` that releases the lock when dropped.
///
/// # Arguments
/// * `deciduous_dir` - Path to the .deciduous directory
///
/// # Errors
/// * `LockError::AlreadyLocked` - Another process holds the lock
/// * `LockError::IoError` - Failed to create/access lock file
pub fn acquire_lock(deciduous_dir: &Path) -> Result<LockGuard, LockError> {
    let lock_path = deciduous_dir.join("deciduous.lock");

    // Create parent directory if needed
    if let Some(parent) = lock_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Open or create the lock file
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;

    // Try to acquire exclusive lock (non-blocking)
    match file.try_lock_exclusive() {
        Ok(true) => {
            // Got the lock - write our PID
            let mut file = file;
            file.set_len(0)?; // Truncate
            write!(file, "{}", std::process::id())?;
            file.sync_all()?;

            Ok(LockGuard {
                file,
                path: lock_path,
            })
        }
        Ok(false) | Err(_) => {
            // Lock held by another process - read the PID
            let mut contents = String::new();
            let mut file = file;
            let _ = file.read_to_string(&mut contents);
            let pid = contents.trim().to_string();

            Err(LockError::AlreadyLocked {
                pid: if pid.is_empty() {
                    "unknown".to_string()
                } else {
                    pid
                },
                lock_path,
            })
        }
    }
}

/// Try to acquire lock, returning None if already locked (non-error case)
///
/// Useful for commands that want to check if another process is active
/// without treating it as an error.
pub fn try_acquire_lock(deciduous_dir: &Path) -> Option<LockGuard> {
    acquire_lock(deciduous_dir).ok()
}

/// Force remove a stale lock file
///
/// # Safety
/// Only call this if you're certain the lock is stale (e.g., process crashed).
/// The `deciduous unlock` command should prompt for confirmation.
pub fn force_unlock(deciduous_dir: &Path) -> Result<(), LockError> {
    let lock_path = deciduous_dir.join("deciduous.lock");

    if lock_path.exists() {
        std::fs::remove_file(&lock_path)?;
    }

    Ok(())
}

/// Check if the database is currently locked (without acquiring)
pub fn is_locked(deciduous_dir: &Path) -> bool {
    let lock_path = deciduous_dir.join("deciduous.lock");

    if !lock_path.exists() {
        return false;
    }

    // Try to open and lock - if we can, it's not locked
    let file = match OpenOptions::new().read(true).write(true).open(&lock_path) {
        Ok(f) => f,
        Err(_) => return true, // Can't open = probably locked
    };

    match file.try_lock_exclusive() {
        Ok(true) => {
            // We got it, so it wasn't locked
            // Lock is automatically released when file is dropped
            false
        }
        Ok(false) | Err(_) => true,
    }
}

/// Get info about current lock holder (if any)
pub fn lock_info(deciduous_dir: &Path) -> Option<String> {
    let lock_path = deciduous_dir.join("deciduous.lock");

    if !lock_path.exists() {
        return None;
    }

    std::fs::read_to_string(&lock_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_acquire_and_release_lock() {
        let temp_dir = TempDir::new().unwrap();
        let deciduous_dir = temp_dir.path().join(".deciduous");
        std::fs::create_dir_all(&deciduous_dir).unwrap();

        // Should be able to acquire lock
        let guard = acquire_lock(&deciduous_dir).unwrap();
        assert!(deciduous_dir.join("deciduous.lock").exists());

        // Lock file should contain our PID
        let pid = std::fs::read_to_string(guard.path()).unwrap();
        assert_eq!(pid.trim(), std::process::id().to_string());

        // Drop the guard
        drop(guard);

        // Lock file should be removed
        assert!(!deciduous_dir.join("deciduous.lock").exists());
    }

    #[test]
    fn test_is_locked() {
        let temp_dir = TempDir::new().unwrap();
        let deciduous_dir = temp_dir.path().join(".deciduous");
        std::fs::create_dir_all(&deciduous_dir).unwrap();

        // Not locked initially
        assert!(!is_locked(&deciduous_dir));

        // Acquire lock
        let guard = acquire_lock(&deciduous_dir).unwrap();
        assert!(is_locked(&deciduous_dir));

        // Release lock
        drop(guard);
        assert!(!is_locked(&deciduous_dir));
    }

    #[test]
    fn test_force_unlock() {
        let temp_dir = TempDir::new().unwrap();
        let deciduous_dir = temp_dir.path().join(".deciduous");
        std::fs::create_dir_all(&deciduous_dir).unwrap();

        // Create a fake lock file
        let lock_path = deciduous_dir.join("deciduous.lock");
        std::fs::write(&lock_path, "12345").unwrap();

        // Force unlock should remove it
        force_unlock(&deciduous_dir).unwrap();
        assert!(!lock_path.exists());
    }

    #[test]
    fn test_lock_info() {
        let temp_dir = TempDir::new().unwrap();
        let deciduous_dir = temp_dir.path().join(".deciduous");
        std::fs::create_dir_all(&deciduous_dir).unwrap();

        // No lock = None
        assert!(lock_info(&deciduous_dir).is_none());

        // Create lock with PID
        let lock_path = deciduous_dir.join("deciduous.lock");
        std::fs::write(&lock_path, "98765").unwrap();

        // Should return the PID
        assert_eq!(lock_info(&deciduous_dir), Some("98765".to_string()));
    }
}
