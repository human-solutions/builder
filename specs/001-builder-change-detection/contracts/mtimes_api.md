# API Contract: crates/mtimes

**Version**: 0.1.0
**Status**: Design
**Date**: 2025-10-06

## Public API

### Core Functions

```rust
/// Check if command should skip execution based on mtime comparison.
///
/// # Arguments
/// * `cmd` - Command implementing InputFiles + OutputFiles traits
/// * `cmd_name` - Name for TSV file (e.g., "sass" → "sass-mtimes.tsv")
/// * `output_dir` - Base directory for relative paths and TSV storage
///
/// # Returns
/// * `Ok(SkipDecision::Skip)` - Command should skip, inputs unchanged
/// * `Ok(SkipDecision::Execute)` - Command should run (inputs changed, outputs missing, or first build)
/// * `Err(_)` - File I/O error, invalid TSV, or lock timeout
///
/// # Examples
/// ```rust
/// use mtimes::{should_skip, SkipDecision};
///
/// match should_skip(&sass_cmd, "sass", Path::new("dist"))? {
///     SkipDecision::Skip { reason } => println!("Skipping: {}", reason),
///     SkipDecision::Execute { reason } => {
///         println!("Executing: {}", reason);
///         // Run command...
///     }
/// }
/// ```
pub fn should_skip<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> anyhow::Result<SkipDecision>;

/// Record successful build completion by updating mtime TSV file.
///
/// Call this AFTER command succeeds to update TSV with current input mtimes.
///
/// # Arguments
/// * `cmd` - Command that just completed successfully
/// * `cmd_name` - Name for TSV file (must match should_skip() call)
/// * `output_dir` - Base directory (must match should_skip() call)
///
/// # Returns
/// * `Ok(())` - TSV file updated successfully
/// * `Err(_)` - File I/O error or invalid paths
///
/// # Examples
/// ```rust
/// // After successful command execution
/// record_success(&sass_cmd, "sass", Path::new("dist"))?;
/// ```
pub fn record_success<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> anyhow::Result<()>;
```

### Skip Decision Type

```rust
/// Decision whether to skip or execute a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipDecision {
    /// Command should skip execution - inputs unchanged since last build.
    Skip {
        /// Human-readable reason for skip (logged to user).
        /// Format: "all N inputs unchanged since YYYY-MM-DD HH:MM"
        reason: String,
    },

    /// Command should execute - inputs changed, outputs missing, or first build.
    Execute {
        /// Human-readable reason for execution (logged to user).
        /// Examples: "No previous build", "Input changed: path/to/file", "1 output missing"
        reason: String,
    },
}
```

### Trait Definitions

```rust
/// Trait for commands to declare their input file dependencies.
///
/// Implementations should return ALL files the command depends on.
/// Change detection compares mtimes of these files against previous build.
pub trait InputFiles {
    /// Returns absolute paths to all input files this command depends on.
    ///
    /// # Returns
    /// * Non-empty Vec - Input files to track
    /// * Empty Vec - No trackable inputs (command always executes)
    ///
    /// # Contract
    /// * Paths MUST be absolute
    /// * Paths SHOULD exist (non-existent triggers rebuild)
    /// * Result SHOULD be deterministic for same command state
    fn input_files(&self) -> Vec<Utf8PathBuf>;
}

/// Trait for commands to declare their expected output files.
///
/// Implementations should return ALL files the command produces.
/// Change detection checks existence of these files - missing files trigger rebuild.
pub trait OutputFiles {
    /// Returns absolute paths to all expected output files.
    ///
    /// # Returns
    /// * Non-empty Vec - Output files to verify existence
    /// * Empty Vec - Outputs are self-discovering (command always executes)
    ///
    /// # Contract
    /// * Paths MUST be absolute
    /// * Paths may not exist yet (first build)
    /// * Result SHOULD match what command actually produces
    fn output_files(&self) -> Vec<Utf8PathBuf>;
}
```

### File Locking

```rust
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
    ///
    /// # Examples
    /// ```rust
    /// let lock = FileLock::acquire(Path::new("dist/.builder-lock"))?;
    /// // ...perform exclusive work...
    /// lock.release()?; // Explicit cleanup (optional - Drop also releases)
    /// ```
    pub fn acquire(lock_path: &Utf8Path) -> anyhow::Result<Self>;

    /// Explicitly release lock and optionally delete lock file.
    ///
    /// Lock is also released automatically via Drop, but explicit release
    /// allows handling errors and deleting the lock file cleanly.
    ///
    /// # Returns
    /// * `Ok(())` - Lock released successfully
    /// * `Err(_)` - Unlock failed (lock may still be held)
    pub fn release(self) -> anyhow::Result<()>;
}

impl Drop for FileLock {
    /// Automatically release lock on Drop (panic or scope exit).
    ///
    /// Lock file is NOT deleted in Drop (only in explicit release()).
    fn drop(&mut self);
}
```

## Error Handling

All public functions return `anyhow::Result` with context chains.

**Error Scenarios**:

| Function | Error Condition | Error Message Example |
|----------|----------------|----------------------|
| `should_skip()` | TSV file corrupted | `"Failed to parse mtimes TSV at dist/sass-mtimes.tsv: invalid mtime at line 3"` |
| `should_skip()` | Input file unreadable | `"Failed to read metadata for styles/main.scss: Permission denied"` |
| `should_skip()` | Lock timeout | `"Failed to acquire lock on dist/.builder-lock after 10 seconds"` |
| `record_success()` | Output dir not writable | `"Failed to write mtimes TSV to dist/sass-mtimes.tsv: Permission denied"` |
| `FileLock::acquire()` | Timeout | `"Failed to acquire lock on dist/.builder-lock after 10 seconds. Another build process may be holding the lock."` |

## Thread Safety

**Not thread-safe** within a single process:
- `FileLock` uses process-level file locks (shared across threads)
- Multiple threads in one process will deadlock

**Multi-process safe**:
- `FileLock` provides exclusive access across processes
- Designed for parallel `cargo build` invocations (different targets)

## Platform Support

- **Linux**: Supported ✅ (uses `flock(2)` syscall)
- **macOS**: Supported ✅ (uses `flock(2)` syscall)
- **Windows**: Not officially supported (per project constitution)

## Dependencies

```toml
[dependencies]
anyhow = { workspace = true }
camino-fs = { workspace = true }

# No external dependencies - uses std::fs for locking
```

## Integration Example

```rust
use builder_mtimes::{should_skip, record_success, SkipDecision, FileLock};
use camino::Utf8Path;

fn run_sass_command(cmd: &SassCmd) -> anyhow::Result<()> {
    let output_dir = Utf8Path::new("dist");

    // 1. Acquire lock (prevent concurrent builds)
    let _lock = FileLock::acquire(&output_dir.join(".builder-lock"))?;

    // 2. Check if we can skip execution
    match should_skip(cmd, "sass", output_dir)? {
        SkipDecision::Skip { reason } => {
            log::info!("SASS: Skipped: {}", reason);
            return Ok(());
        }
        SkipDecision::Execute { reason } => {
            log::info!("SASS: Executing: {}", reason);
        }
    }

    // 3. Execute command
    compile_sass(&cmd.in_scss, &cmd.output)?;

    // 4. Record success (update TSV)
    record_success(cmd, "sass", output_dir)?;

    // 5. Lock released automatically via Drop
    Ok(())
}
```

## TSV File Format

**File Location**: `{output_dir}/{cmd_name}-mtimes.tsv`

**Format**:
```
<relative-path><TAB><mtime-nanos><NEWLINE>
```

**Example** (`sass-mtimes.tsv`):
```
styles/main.scss	1728234567890123456
styles/_variables.scss	1728234567123456789
styles/_mixins.scss	1728234568000000000
```

**Parsing Rules**:
- Split on ASCII tab (`\t`, U+0009)
- Expect exactly 2 fields per line
- Field 1: UTF-8 relative path (no escaping)
- Field 2: u128 decimal string (max 39 digits)
- Lines sorted alphabetically for determinism

## Version Compatibility

**v0.1.0** (initial release):
- API is unstable - breaking changes allowed until 1.0.0
- TSV format is stable - forward compatible (old builders can read new files)
- Backward compatibility not required (per spec - "no backwards compatibility is required")

## Testing Contract

Contract tests verify API behavior:

1. **First build** → `Execute { reason: "No previous build" }`
2. **Unchanged inputs** → `Skip { reason: "all N inputs unchanged since ..." }`
3. **Changed input** → `Execute { reason: "Input changed: ..." }`
4. **Missing output** → `Execute { reason: "Output missing: ..." }`
5. **Lock acquisition** → Succeeds when unlocked, times out when locked
6. **Lock cleanup** → Released on Drop and explicit release()
7. **Parallel builds** → Only one process executes at a time

See `crates/mtimes/tests/contract_tests.rs` for full test suite.
