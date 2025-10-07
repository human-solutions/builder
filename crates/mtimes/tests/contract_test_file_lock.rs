use builder_mtimes::FileLock;
use camino_fs::Utf8Path;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_lock_acquisition_basic() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let lock_path = Utf8Path::from_path(temp.path())
        .unwrap()
        .join(".builder-lock");

    let lock = FileLock::acquire(&lock_path)?;
    assert!(lock_path.exists());

    lock.release()?;
    Ok(())
}

#[test]
#[ignore] // This test takes 10+ seconds to run
fn test_lock_acquisition_timeout() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let lock_path = Utf8Path::from_path(temp.path())
        .unwrap()
        .join(".builder-lock");

    // Hold lock in one thread
    let lock_path_clone = lock_path.clone();
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();

    let handle = thread::spawn(move || {
        let _lock = FileLock::acquire(&lock_path_clone).expect("Failed to acquire lock in thread");
        barrier_clone.wait(); // Signal that lock is held
        thread::sleep(Duration::from_secs(15)); // Hold longer than timeout
    });

    barrier.wait(); // Wait for thread to acquire lock
    thread::sleep(Duration::from_millis(100)); // Ensure lock is held

    // Try to acquire in main thread - should timeout
    let start = std::time::Instant::now();
    let result = FileLock::acquire(&lock_path);
    let elapsed = start.elapsed();

    assert!(result.is_err(), "Expected timeout error");
    assert!(
        elapsed >= Duration::from_secs(10),
        "Should wait at least 10 seconds"
    );
    assert!(
        elapsed < Duration::from_secs(12),
        "Should not wait much longer than 10 seconds"
    );

    handle.join().unwrap();
    Ok(())
}

#[test]
fn test_lock_concurrent_acquisition() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let lock_path = Utf8Path::from_path(temp.path())
        .unwrap()
        .join(".builder-lock");

    let lock_path_1 = lock_path.clone();
    let lock_path_2 = lock_path.clone();

    let barrier = Arc::new(Barrier::new(2));
    let barrier_1 = barrier.clone();
    let barrier_2 = barrier.clone();

    // Thread 1: Acquire lock, hold briefly, release
    let handle1 = thread::spawn(move || {
        barrier_1.wait(); // Synchronize start
        let lock = FileLock::acquire(&lock_path_1).expect("Thread 1 failed to acquire lock");
        thread::sleep(Duration::from_millis(200));
        lock.release().expect("Thread 1 failed to release lock");
    });

    // Thread 2: Acquire lock after thread 1 releases
    let handle2 = thread::spawn(move || {
        barrier_2.wait(); // Synchronize start
        thread::sleep(Duration::from_millis(50)); // Start slightly after thread 1
        let lock = FileLock::acquire(&lock_path_2).expect("Thread 2 failed to acquire lock");
        lock.release().expect("Thread 2 failed to release lock");
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    Ok(())
}
