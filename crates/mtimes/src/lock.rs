use anyhow::{Context, Result};
use camino_fs::{Utf8Path, Utf8PathBuf};
use fs4::fs_std::FileExt;
use std::fs::{File, OpenOptions};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Exclusive file lock with automatic cleanup.
///
/// Provides exclusive access to output directory during builds.
/// Lock is automatically released on Drop (panic) or explicit release().
pub struct FileLock {
    file: File,
    path: Utf8PathBuf,
}

impl FileLock {
    /// Acquire exclusive lock with 10-second timeout.
    ///
    /// # Arguments
    /// * `lock_path` - Path to lock file (typically `{output_dir}/.builder-lock`)
    ///
    /// # Returns
    /// * `Ok(FileLock)` - Lock acquired
    /// * `Err(_)` - Timeout (10s) or I/O error
    ///
    /// # Behavior
    /// * Retries every 50ms for up to 10 seconds
    /// * Creates lock file if it doesn't exist
    /// * Lock released automatically on Drop (panic) or explicit release()
    pub fn acquire(lock_path: &Utf8Path) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(lock_path)
            .context("Failed to create lock file")?;

        let start = Instant::now();
        let timeout = Duration::from_secs(10);
        let retry_interval = Duration::from_millis(50);

        loop {
            match file.try_lock_exclusive() {
                Ok(true) => {
                    // Lock acquired
                    return Ok(Self {
                        file,
                        path: lock_path.to_owned(),
                    });
                }
                Ok(false) => {
                    // Lock held by another process
                    if start.elapsed() >= timeout {
                        anyhow::bail!(
                            "Failed to acquire lock on {} after 10 seconds. \
                             Another build process may be holding the lock.",
                            lock_path
                        );
                    }
                    sleep(retry_interval);
                }
                Err(e) => {
                    return Err(e).context("Failed to acquire file lock");
                }
            }
        }
    }

    /// Explicitly release lock and optionally delete lock file.
    ///
    /// Lock is also released automatically via Drop, but explicit release
    /// allows handling errors and deleting the lock file cleanly.
    ///
    /// # Returns
    /// * `Ok(())` - Lock released successfully
    /// * `Err(_)` - Unlock failed (lock may still be held)
    pub fn release(self) -> Result<()> {
        self.file.unlock().context("Failed to release lock")?;
        fs_err::remove_file(&self.path).ok(); // Best effort cleanup
        Ok(())
    }
}

impl Drop for FileLock {
    /// Automatically release lock on Drop (panic or scope exit).
    ///
    /// Lock file is NOT deleted in Drop (only in explicit release()).
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
