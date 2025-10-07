# Research: Builder Change Detection

**Date**: 2025-10-06
**Status**: Complete

## Overview

Research findings for mtime-based change detection in builder commands. All technical unknowns from the plan have been resolved.

## 1. std::fs File Locking (macOS & Linux)

### Decision

Use `std::fs::File::try_lock()` with custom timeout loop (stabilized in Rust 1.89+).

**No external dependencies needed** - the plan's approach to avoid `fs2`, `fs4`, or `fslock` is validated.

### Implementation Approach

```rust
use std::fs::File;
use std::time::{Duration, Instant};
use std::thread::sleep;

pub struct FileLock {
    file: File,
    path: Utf8PathBuf,
}

impl FileLock {
    pub fn acquire(lock_path: &Utf8Path) -> anyhow::Result<Self> {
        let file = File::create(lock_path)?;

        let start = Instant::now();
        let timeout = Duration::from_secs(10);
        let retry_interval = Duration::from_millis(50);

        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { file, path: lock_path.to_owned() }),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= timeout {
                        anyhow::bail!("Lock timeout after 10 seconds on {}", lock_path);
                    }
                    sleep(retry_interval);
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock(); // Best-effort cleanup
    }
}
```

**Key details**:
- `try_lock()` is non-blocking - returns immediately with `WouldBlock` if locked
- Timeout requires custom loop with `Instant` tracking
- 50ms sleep interval balances responsiveness vs CPU usage
- `Drop` ensures cleanup on panic; explicit `release()` for success path

### Platform Differences

| Aspect | Linux | macOS | Impact |
|--------|-------|-------|--------|
| **Syscall** | `flock(2)` | `flock(2)` | Consistent behavior |
| **Lock type** | Advisory | Advisory | Both require cooperation |
| **Precision** | Nanosecond (ext4) | Nanosecond (APFS) | Full precision available |
| **CLI tool** | Has `/usr/bin/flock` | No `flock` command | Not relevant (using syscall) |

**Platform quirks**:
- macOS: `flock()` and `fcntl()` locks interact; Linux keeps them separate
- Linux kernel < 2.6.12: flock doesn't work on NFS (not an issue for build.rs)
- Both platforms: lock conversion (shared ↔ exclusive) may allow other processes to "wriggle in" during transition

### Lock Cleanup Behavior

✅ **Automatic cleanup confirmed** across all failure modes:

| Scenario | Cleanup Behavior |
|----------|------------------|
| Normal exit | File descriptor closed → lock released |
| Panic (unwind) | `Drop` runs → `unlock()` called → lock released |
| Panic (abort) | Process exits → OS closes fd → lock released |
| `kill -9` | OS forcibly closes fd → lock released |
| Segfault | Same as abort - OS cleanup |

**No stale lock files** - lock lifetime tied to file descriptor, not process lifetime.

### Alternatives Considered

| Alternative | Rejection Reason |
|-------------|------------------|
| **fs2 crate** | Adds dependency; std::fs now has native support (Rust 1.89+) |
| **file-lock crate** | External dependency; std::fs sufficient |
| **fcntl(2) wrapper** | More complex than flock; no benefits for this use case |
| **Busy-wait loop** | High CPU usage; sleep(50ms) is better |

## 2. SystemTime Precision and Mtime Extraction

### Decision

Use `std::fs::Metadata::modified()` with conversion to `u128` nanoseconds since UNIX_EPOCH.

### API

```rust
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// Extract mtime
let metadata = fs::metadata(path)?;
let mtime: SystemTime = metadata.modified()?;

// Convert to u128 nanoseconds
let mtime_nanos: u128 = mtime
    .duration_since(UNIX_EPOCH)
    .expect("Filesystem time before UNIX epoch")?
    .as_nanos();
```

### Precision Availability

✅ **Nanosecond precision confirmed** on target platforms:

| Filesystem | Precision | Notes |
|------------|-----------|-------|
| **APFS** (macOS) | 1 nanosecond | Native support |
| **ext4** (Linux) | 1 nanosecond | ⚠️ Requires 256-byte+ inodes |
| **ext4** (Linux) | 1 second | 128-byte inodes only (rare) |
| **HFS+** (old macOS) | 1 second | ❌ No nanosecond support |

**Check inode size on Linux**: `sudo tune2fs -l /dev/sdX | grep "Inode size"`

### Conversion to u128

```rust
fn get_mtime_nanos(path: &Utf8Path) -> anyhow::Result<u128> {
    let metadata = fs_err::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path))?;

    let mtime = metadata.modified()
        .context("Filesystem doesn't support modification time")?;

    match mtime.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_nanos()),
        Err(e) => {
            // Clock drift or pre-epoch time
            anyhow::bail!("Invalid modification time for {}: {}", path, e);
        }
    }
}
```

**Type considerations**:
- `u128`: 584+ billion year range, never overflows for realistic dates ✅
- `i64`: ~584 year range, overflows April 2262 ❌
- `Duration::as_nanos()`: Returns u128 (stable since Rust 1.33)

### Edge Cases

#### Clock Drift ⚠️

**Issue**: `SystemTime` is NOT monotonic - NTP can rewind system clock.

**Symptom**: File saved later may have earlier timestamp.

**Mitigation**:
```rust
match current_mtime.duration_since(stored_mtime) {
    Ok(_) => SkipDecision::Skip { reason: "unchanged" },
    Err(_) => SkipDecision::Execute { reason: "clock drift detected - rebuilding" },
}
```

#### Filesystem Precision Mismatch

**Scenario**: Copy file from APFS (1ns) to ext4 with 128B inodes (1s).

**Result**: Nanosecond digits truncated on target filesystem.

**Strategy**: Document limitation; accept occasional false positives (spec allows this per FR-009).

#### Invalid Timestamps

**Issue**: Some filesystems return values that fail assertions.

**Mitigation**: Wrap `metadata.modified()` in Result handling with context.

### Alternatives Considered

| Alternative | Rejection Reason |
|-------------|------------------|
| **i64 nanoseconds** | Overflows in 2262; u128 avoids this |
| **Monotonic clock (Instant)** | Not persisted across process runs |
| **Content hashing** | Violates user requirement: "Use mtimes exclusively" |
| **Second precision** | Loses precision on modern filesystems unnecessarily |

## 3. TSV Format Design

### Decision

Use tab-separated values with format: `<relative-path><TAB><mtime-nanos><NEWLINE>`

### Format Specification

```
src/main.rs	1728234567890123456
assets/logo.png	1728234567123456789
styles/main.scss	1728234568000000000
```

**Rules**:
- Paths: Relative to output directory, UTF-8 encoded (camino-fs guarantees this)
- Separator: Single ASCII tab character (`\t`, U+0009)
- Mtime: u128 as decimal string (max 39 digits)
- Newline: LF (`\n`, U+000A) on all platforms

### Escaping Requirements

**No escaping needed** - reasoning:
- **Tabs in paths**: Illegal on macOS/Linux (filesystem rejects them)
- **Newlines in paths**: Illegal on macOS/Linux
- **Unicode**: Supported via UTF-8 (camino-fs enforces valid UTF-8)
- **Spaces**: No special handling needed (tab is unambiguous separator)

### Trade-offs vs JSON

| Aspect | TSV | JSON | Winner |
|--------|-----|------|--------|
| **Simplicity** | Split on `\t`, parse u128 | serde_json dependency | TSV ✅ |
| **File size** | ~50 bytes/line | ~80 bytes/line (keys, quotes) | TSV ✅ |
| **Parse speed** | O(n) line-by-line | O(n) + serde overhead | TSV ✅ |
| **Human readable** | Moderately | Very | JSON (not critical) |
| **Dependencies** | None (std only) | serde_json (workspace dep) | TSV ✅ |

**Decision rationale**: TSV is simpler, faster, and requires no dependencies beyond std for this use case. The plan's choice of TSV over JSON is validated.

### UTF-8 Safety

✅ **Confirmed safe** with camino-fs:
- All paths are `Utf8PathBuf` (guaranteed valid UTF-8)
- TSV format is valid UTF-8 (ASCII tab + decimal digits + UTF-8 paths)
- No encoding/decoding needed

### Example Implementation

```rust
// Write TSV
fn write_mtimes(records: &HashMap<Utf8PathBuf, u128>, path: &Utf8Path) -> anyhow::Result<()> {
    let mut lines: Vec<String> = records
        .iter()
        .map(|(p, mtime)| format!("{}\t{}", p, mtime))
        .collect();
    lines.sort(); // Deterministic order
    fs_err::write(path, lines.join("\n"))?;
    Ok(())
}

// Read TSV
fn read_mtimes(path: &Utf8Path) -> anyhow::Result<HashMap<Utf8PathBuf, u128>> {
    let content = fs_err::read_to_string(path)?;
    let mut records = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid TSV at line {}: expected 2 fields", line_num + 1);
        }
        let path = Utf8PathBuf::from(parts[0]);
        let mtime: u128 = parts[1].parse()
            .with_context(|| format!("Invalid mtime at line {}", line_num + 1))?;
        records.insert(path, mtime);
    }

    Ok(records)
}
```

## 4. Command Input/Output File Enumeration

### Survey of Existing Commands

| Command | Input Pattern | Output Pattern |
|---------|---------------|----------------|
| **SassCmd** | Single file (`in_scss: Utf8PathBuf`) | Multiple via `Output` structs |
| **CopyCmd** | Directory scan (`src_dir + file_extensions`) | Multiple via `Output` structs |
| **WasmProcessingCmd** | Package manifest (implicit) | Single `.wasm` file |
| **UniffiCmd** | `.udl` file | Multiple (Kotlin/Swift bindings) |
| **FontForgeCmd** | `.sfd` file | Multiple (WOFF2, OTF) |
| **LocalizedCmd** | Directory of `.ftl` files | Multiple localized outputs |

### Common Patterns

**Input discovery**:
1. **Single file**: SassCmd, FontForgeCmd, UniffiCmd → field is `Utf8PathBuf`
2. **Directory scan**: CopyCmd, LocalizedCmd → field is `src_dir: Utf8PathBuf` + filters
3. **Config-based**: WasmProcessingCmd → use `cargo metadata` to find target

**Output patterns**:
1. **Single file**: WasmProcessingCmd → computed from package name
2. **Multiple outputs**: Most commands → use `Output` struct (field: `output: Vec<Output>`)
3. **Self-discovering**: Some commands generate files not declared upfront

### Trait API Design

```rust
pub trait InputFiles {
    /// Returns absolute paths to all input files this command depends on.
    /// Empty vec indicates no trackable inputs.
    fn input_files(&self) -> Vec<Utf8PathBuf>;
}

pub trait OutputFiles {
    /// Returns absolute paths to all expected output files.
    /// Empty vec indicates outputs are self-discovering (always rebuild).
    fn output_files(&self) -> Vec<Utf8PathBuf>;
}
```

**Design rationale**:
- **Simple signature**: No error handling - commands know their own files
- **Vec return**: Allocates, but simplifies interface (no lifetimes)
- **Empty vec semantics**: Allows commands with no trackable inputs/outputs
- **Absolute paths**: Avoids ambiguity about base directory

### Implementation Burden

| Command | Input Implementation Complexity | Output Implementation Complexity |
|---------|----------------------------------|-----------------------------------|
| **SassCmd** | Trivial (return `vec![self.in_scss.clone()]`) | Medium (enumerate `Output` destinations) |
| **CopyCmd** | Medium (scan `src_dir` with filters) | Medium (enumerate `Output` destinations) |
| **WasmProcessingCmd** | Medium (find Cargo.toml, list sources) | Trivial (derive from package name) |
| **UniffiCmd** | Trivial (return `.udl` path) | Medium (compute binding paths) |

**Minimal scaffolding confirmed** - most commands need <10 lines per trait.

### Alternatives Considered

| Alternative | Rejection Reason |
|-------------|------------------|
| **&[&Path] return** | Lifetime issues; callers would need to clone anyway |
| **Iterator return** | More complex; no benefit over Vec for small file lists |
| **Result return** | Commands know their files; errors should be panics |
| **Separate trait per command** | Duplication; common interface preferred |

## 5. Lock File Cleanup Patterns

### Decision

Use `Drop` implementation + explicit release on success path.

### Cleanup Strategy

```rust
impl FileLock {
    pub fn release(self) -> anyhow::Result<()> {
        self.file.unlock()?;
        // Optional: delete lock file
        fs_err::remove_file(&self.path).ok(); // Best effort
        Ok(())
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Unlock file (OS cleanup is automatic, but explicit is clearer)
        let _ = self.file.unlock();
        // Do NOT delete lock file in Drop - may be called during panic
    }
}
```

**Rationale**:
- **Drop**: Ensures unlock on panic (Rust unwinds stack)
- **Explicit release**: Success path can delete lock file cleanly
- **OS handles crash/kill**: File descriptor closed → lock released

### Failure Modes

| Scenario | Lock Released? | Lock File Deleted? | Recovery |
|----------|----------------|---------------------|----------|
| **Normal completion** | ✅ (explicit) | ✅ (optional) | N/A |
| **Panic (unwind)** | ✅ (Drop) | ❌ | Harmless - file unlocked |
| **Panic (abort)** | ✅ (OS) | ❌ | Harmless - file unlocked |
| **kill -9** | ✅ (OS) | ❌ | Harmless - file unlocked |
| **Segfault** | ✅ (OS) | ❌ | Harmless - file unlocked |
| **Timeout** | ❌ (by design) | ❌ | Return error to user |

**Stale lock files are harmless** - they only matter when a process has them open.

### Best Practices

1. **Acquire lock early**: Before any side effects
2. **Hold minimum time**: Release as soon as exclusive access not needed
3. **Timeout clearly**: 10 seconds is adequate for build.rs (sub-second typical)
4. **Log lock waits**: Help diagnose contention issues

### Out of Scope (Documented for Future)

**Stale lock detection** (not required per spec):
- Could check lock file age vs timeout (e.g., > 1 hour is stale)
- Could force-break locks with PID tracking (complex, error-prone)
- Spec explicitly allows timeout failures, so not needed

**Lock file location**:
- Spec requires `.builder-lock` at output directory root
- Shared across all commands (fine - locking is per output directory, not per command)

## Summary of Decisions

| Research Area | Decision | Rationale |
|---------------|----------|-----------|
| **File locking** | `std::fs::File::try_lock()` with 50ms retry loop | Native support, no external deps |
| **Lock timeout** | 10 seconds via `Instant::now() + loop` | Balances responsiveness vs tolerance |
| **Lock cleanup** | `Drop` impl + explicit `release()` | Handles panic and normal exit |
| **Mtime API** | `fs::Metadata::modified() -> SystemTime` | Standard Rust API |
| **Mtime storage** | u128 nanoseconds since UNIX_EPOCH | Full precision, no overflow until ~584B years |
| **Clock drift** | Handle `Err` from `duration_since()` | Rebuild on drift (safe choice) |
| **TSV format** | `path<TAB>mtime_nanos<LF>` | Simpler than JSON, no escaping needed |
| **Trait API** | `InputFiles + OutputFiles` traits | Minimal impl burden, clear contract |
| **Lock file** | `.builder-lock` (shared, reused) | Spec requirement, reduces file count |

## Next Steps

Proceed to Phase 1: Design & Contracts
- Create data-model.md (entities, state diagrams)
- Create API contracts (crates/mtimes public API)
- Create contract tests (basic functionality tests)
- Create integration tests (parallel builds, lock contention)
- Update CLAUDE.md with change detection section
- Create quickstart.md with verification steps

All technical unknowns resolved ✅
