# Data Model: Builder Change Detection

**Date**: 2025-10-06
**Status**: Complete

## Overview

Data structures, relationships, and state transitions for mtime-based change detection in builder commands.

## Core Entities

### 1. MtimeRecord

**Purpose**: Represents a single file's modification time snapshot.

**Fields**:
- `path`: `Utf8PathBuf` - Relative path from output directory
- `mtime_nanos`: `u128` - Modification time as nanoseconds since UNIX_EPOCH

**Validation Rules**:
- Path MUST be relative (no leading `/` or drive letters)
- Path MUST be valid UTF-8 (guaranteed by camino-fs)
- `mtime_nanos` MUST be > 0 (UNIX_EPOCH or later)
- `mtime_nanos` MUST fit in u128 (always true for realistic dates)

**Relationships**:
- Belongs to exactly one `MtimeTracker`
- One-to-one with a filesystem path (unique per tracker)

**State**:
- **Immutable** after creation
- Replaced (not mutated) when file changes

**Example**:
```rust
MtimeRecord {
    path: "src/main.rs".into(),
    mtime_nanos: 1728234567890123456,
}
```

---

### 2. MtimeTracker

**Purpose**: Manages collection of mtime records for a single command execution.

**Fields**:
- `base_dir`: `Utf8PathBuf` - Output directory (base for relative paths)
- `records`: `HashMap<Utf8PathBuf, u128>` - Map of path → mtime

**Derived Fields** (computed, not stored):
- `tsv_path`: `{base_dir}/{cmd_name}-mtimes.tsv`
- `lock_path`: `{base_dir}/.builder-lock`

**Validation Rules**:
- `base_dir` MUST exist and be writable
- All paths in `records` MUST be relative to `base_dir`
- `records` MUST contain only files that exist (checked on load)

**Relationships**:
- Contains multiple `MtimeRecord` entries (0..n)
- Associated with exactly one command invocation
- Associated with exactly one output directory

**Lifecycle**:
1. **Create**: Instantiate empty tracker for command
2. **Load**: Read existing TSV file (if present) into `records`
3. **Compare**: Check current mtimes against loaded `records`
4. **Save**: Write updated `records` to TSV file

**State Transitions**:
```
┌─────────┐
│  Empty  │ (new tracker, no TSV file)
└────┬────┘
     │ load()
     ▼
┌─────────┐
│ Loaded  │ (TSV parsed into records HashMap)
└────┬────┘
     │ compare()
     ▼
┌─────────┐
│Compared │ (skip decision made, not mutated)
└────┬────┘
     │ record_success()
     ▼
┌─────────┐
│  Saved  │ (updated TSV written to disk)
└─────────┘
```

**Example**:
```rust
MtimeTracker {
    base_dir: "/project/dist".into(),
    records: HashMap::from([
        ("assets/logo.png".into(), 1728234567000000000),
        ("styles/main.css".into(), 1728234568123456789),
    ]),
}
```

---

### 3. FileLock

**Purpose**: Provides exclusive access to output directory during builds.

**Fields**:
- `file`: `std::fs::File` - Open file descriptor for lock file
- `path`: `Utf8PathBuf` - Path to `.builder-lock` file

**Validation Rules**:
- `path` MUST be `{output_dir}/.builder-lock`
- Lock MUST be exclusive (not shared)
- Lock acquisition MUST timeout after 10 seconds

**Relationships**:
- One lock per output directory (shared across all commands)
- No relationship to specific commands (directory-scoped, not command-scoped)

**Lifecycle**:
1. **Acquire**: Create lock file, call `file.try_lock()` with 10s timeout
2. **Hold**: Keep lock while executing command
3. **Release**: Call `file.unlock()` and optionally delete lock file

**State**:
```
┌──────────┐
│Unlocked  │ (file exists but no process holds lock)
└────┬─────┘
     │ acquire() - try_lock() loop
     ▼
┌──────────┐
│ Locked   │ (process holds exclusive lock)
└────┬─────┘
     │ release() or Drop
     ▼
┌──────────┐
│Unlocked  │ (lock released, file may be deleted)
└──────────┘
```

**Timeout Behavior**:
- If lock not acquired within 10 seconds → return `Err`
- Retry interval: 50ms (balances CPU vs responsiveness)
- Total retries: ~200 attempts over 10 seconds

**Cleanup Behavior**:
| Event | Lock Released | File Deleted |
|-------|---------------|--------------|
| Normal release() | ✅ Explicit | ✅ Optional |
| Drop (panic) | ✅ Automatic | ❌ No |
| Process crash | ✅ OS cleanup | ❌ No |

**Example**:
```rust
FileLock {
    file: File::open("/project/dist/.builder-lock")?,
    path: "/project/dist/.builder-lock".into(),
}
```

---

## Trait Contracts

### 4. InputFiles Trait

**Purpose**: Allows commands to declare their input dependencies for change detection.

**Signature**:
```rust
pub trait InputFiles {
    fn input_files(&self) -> Vec<Utf8PathBuf>;
}
```

**Contract**:
- MUST return **absolute paths** to all input files
- MAY return empty `Vec` if command has no trackable inputs
- Paths MUST exist when called (non-existent files trigger rebuild)
- Should be deterministic (same inputs → same output)

**Implementations** (planned):
- `SassCmd`: Return `vec![self.in_scss.clone()]`
- `CopyCmd`: Scan `src_dir` with `file_extensions` filter
- `WasmProcessingCmd`: Find Cargo.toml and list source files
- `UniffiCmd`: Return `.udl` file path
- `FontForgeCmd`: Return `.sfd` file path
- `LocalizedCmd`: Scan directory for `.ftl` files
- `SwiftPackageCmd`: Return template and config files

**Example Implementation**:
```rust
impl InputFiles for SassCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        // Single input file for SASS compilation
        vec![self.in_scss.clone()]
    }
}

impl InputFiles for CopyCmd {
    fn input_files(&self) -> Vec<Utf8PathBuf> {
        // Scan directory for matching extensions
        let mut inputs = Vec::new();
        let walker = if self.recursive {
            WalkDir::new(&self.src_dir)
        } else {
            WalkDir::new(&self.src_dir).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(Result::ok) {
            let path = Utf8PathBuf::try_from(entry.path().to_path_buf()).ok()?;
            if let Some(ext) = path.extension() {
                if self.file_extensions.contains(&ext.to_string()) {
                    inputs.push(path);
                }
            }
        }
        inputs
    }
}
```

---

### 5. OutputFiles Trait

**Purpose**: Allows commands to declare their expected outputs for existence checking.

**Signature**:
```rust
pub trait OutputFiles {
    fn output_files(&self) -> Vec<Utf8PathBuf>;
}
```

**Contract**:
- MUST return **absolute paths** to all expected output files
- MAY return empty `Vec` if outputs are self-discovering (always rebuild)
- Paths may not exist yet (first build) - existence checked, not assumed
- Should match what command actually produces

**Implementations** (planned):
- Most commands: Enumerate destinations from `self.output: Vec<Output>`
- `WasmProcessingCmd`: Compute from package name and profile
- Commands with dynamic outputs: Return empty `Vec` (always rebuild)

**Example Implementation**:
```rust
impl OutputFiles for SassCmd {
    fn output_files(&self) -> Vec<Utf8PathBuf> {
        // Enumerate all Output destinations
        let stem = self.in_scss.file_stem().unwrap();
        self.output.iter()
            .flat_map(|out| {
                let css_path = out.dest_dir.join(format!("{}.css", stem));
                let mut paths = vec![css_path.clone()];

                // Add compressed variants if enabled
                if out.gzip() {
                    paths.push(css_path.with_extension("css.gz"));
                }
                if out.brotli() {
                    paths.push(css_path.with_extension("css.br"));
                }

                paths
            })
            .collect()
    }
}
```

---

## Data Flow

### Change Detection Flow

```
┌───────────────┐
│ Command Start │
└───────┬───────┘
        │
        ▼
┌──────────────────────┐
│ Acquire FileLock     │ (10s timeout, 50ms retry)
│ (.builder-lock)      │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Create MtimeTracker  │
│ base_dir = output_dir│
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Load existing TSV    │
│ (if exists)          │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Get current mtimes   │
│ via InputFiles trait │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Compare:                         │
│  - Any stored mtime != current?  │
│  - Any input added/removed?      │
│  - Any output missing?           │
└──────────┬───────────────────────┘
           │
           ├─ NO changes ──► Skip command (log reason)
           │                      │
           └─ YES changes ──► Execute command
                                   │
                                   ▼
                            ┌──────────────────┐
                            │ Record success   │
                            │ Write updated TSV│
                            └──────┬───────────┘
                                   │
                                   ▼
                            ┌──────────────────┐
                            │ Release FileLock │
                            └──────────────────┘
```

### TSV File Format

**File**: `{output_dir}/{cmd_name}-mtimes.tsv`

**Format**:
```
<relative-path><TAB><mtime-nanos><LF>
```

**Example** (`sass-mtimes.tsv`):
```
styles/main.scss	1728234567890123456
styles/_variables.scss	1728234567123456789
styles/_mixins.scss	1728234568000000000
```

**Ordering**: Alphabetical by path (deterministic for diffing)

**Parsing Rules**:
1. Split each line on `\t` (ASCII tab, U+0009)
2. Expect exactly 2 fields per line
3. Parse field 1 as UTF-8 path (relative)
4. Parse field 2 as u128 decimal number
5. Reject lines with != 2 fields
6. Skip empty lines (optional - typical to disallow)

---

## Change Detection Algorithm

### should_skip() Logic

```rust
pub enum SkipDecision {
    Skip { reason: String },
    Execute { reason: String },
}

pub fn should_skip<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> anyhow::Result<SkipDecision> {
    let tsv_path = output_dir.join(format!("{}-mtimes.tsv", cmd_name));

    // 1. Load previous build records
    let previous_mtimes = if tsv_path.exists() {
        read_mtimes(&tsv_path)?
    } else {
        return Ok(SkipDecision::Execute {
            reason: "No previous build".to_string()
        });
    };

    // 2. Get current input files and their mtimes
    let input_files = cmd.input_files();
    let mut current_mtimes = HashMap::new();
    for path in &input_files {
        let mtime = get_mtime_nanos(path)?;
        let rel_path = path.strip_prefix(output_dir)
            .unwrap_or(path)
            .to_owned();
        current_mtimes.insert(rel_path, mtime);
    }

    // 3. Check for input changes
    if current_mtimes.len() != previous_mtimes.len() {
        return Ok(SkipDecision::Execute {
            reason: format!(
                "{} inputs added/removed",
                (current_mtimes.len() as i64 - previous_mtimes.len() as i64).abs()
            )
        });
    }

    for (path, current_mtime) in &current_mtimes {
        match previous_mtimes.get(path) {
            None => {
                return Ok(SkipDecision::Execute {
                    reason: format!("New input: {}", path)
                });
            }
            Some(&prev_mtime) => {
                if *current_mtime != prev_mtime {
                    return Ok(SkipDecision::Execute {
                        reason: format!("Input changed: {}", path)
                    });
                }
            }
        }
    }

    // 4. Check for missing outputs
    let output_files = cmd.output_files();
    for path in &output_files {
        if !path.exists() {
            return Ok(SkipDecision::Execute {
                reason: format!("Output missing: {}", path)
            });
        }
    }

    // 5. All checks passed - skip execution
    let last_build = previous_mtimes.values()
        .max()
        .copied()
        .unwrap_or(0);
    let last_build_time = format_timestamp(last_build);

    Ok(SkipDecision::Skip {
        reason: format!(
            "all {} inputs unchanged since {}",
            input_files.len(),
            last_build_time
        )
    })
}
```

### record_success() Logic

```rust
pub fn record_success<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> anyhow::Result<()> {
    let tsv_path = output_dir.join(format!("{}-mtimes.tsv", cmd_name));

    // Collect current input mtimes
    let input_files = cmd.input_files();
    let mut records = HashMap::new();

    for path in input_files {
        let mtime = get_mtime_nanos(&path)?;
        let rel_path = path.strip_prefix(output_dir)
            .unwrap_or(&path)
            .to_owned();
        records.insert(rel_path, mtime);
    }

    // Write to TSV
    write_mtimes(&records, &tsv_path)?;

    Ok(())
}
```

---

## State Invariants

### MtimeTracker Invariants

1. **Path Relativity**: All paths in `records` are relative to `base_dir`
2. **Path Existence**: On load, non-existent paths trigger rebuild (stale TSV)
3. **Mtime Validity**: All mtimes are >= 0 (UNIX_EPOCH or later)
4. **TSV Sync**: `records` HashMap matches TSV file contents after load/save

### FileLock Invariants

1. **Exclusivity**: At most one process holds lock on `.builder-lock` at any time
2. **Cleanup**: Lock always released on Drop (panic) or explicit release (success)
3. **Timeout**: Lock acquisition never blocks > 10 seconds
4. **File Persistence**: Lock file may persist after unlock (harmless stale file)

### Change Detection Invariants

1. **Conservative**: Always rebuild on uncertainty (missing TSV, clock drift, errors)
2. **Deterministic**: Same inputs + same mtimes → same skip decision
3. **Observable**: Skip decision reason logged to user (FR-013)

---

## Example Scenarios

### Scenario 1: First Build

**State**: No TSV file exists

**Flow**:
1. `should_skip()` checks for `sass-mtimes.tsv` → not found
2. Return `Execute { reason: "No previous build" }`
3. Command executes
4. `record_success()` writes TSV with current mtimes

**TSV After**:
```
styles/main.scss	1728234567890123456
```

---

### Scenario 2: Unchanged Inputs (Skip)

**State**: TSV exists, inputs unchanged

**Flow**:
1. Load `sass-mtimes.tsv` → `{ "styles/main.scss" => 1728234567890123456 }`
2. Get current mtime for `styles/main.scss` → `1728234567890123456`
3. Compare: mtimes match ✅
4. Check outputs: `dist/main.css` exists ✅
5. Return `Skip { reason: "all 1 inputs unchanged since 2025-10-06 14:32" }`

**TSV After**: Unchanged

---

### Scenario 3: Input Modified (Rebuild)

**State**: TSV exists, input file touched

**Flow**:
1. Load `sass-mtimes.tsv` → `{ "styles/main.scss" => 1728234567890123456 }`
2. Get current mtime for `styles/main.scss` → `1728234999000000000` (newer)
3. Compare: mtimes differ ❌
4. Return `Execute { reason: "Input changed: styles/main.scss" }`
5. Command executes
6. `record_success()` writes updated TSV

**TSV After**:
```
styles/main.scss	1728234999000000000
```

---

### Scenario 4: Output Deleted (Rebuild)

**State**: TSV exists, inputs unchanged, output missing

**Flow**:
1. Load TSV, check mtimes → all match ✅
2. Check outputs: `dist/main.css` does NOT exist ❌
3. Return `Execute { reason: "Output missing: dist/main.css" }`
4. Command executes, regenerates output
5. `record_success()` writes TSV (mtimes unchanged)

**TSV After**: Unchanged (inputs didn't change)

---

### Scenario 5: Parallel Builds (Lock Contention)

**State**: Two `cargo build` processes for different targets, same output directory

**Flow**:
1. **Process A**: Acquires `.builder-lock` → succeeds
2. **Process B**: Attempts `.builder-lock` → `WouldBlock` error
3. **Process B**: Sleeps 50ms, retries → still blocked
4. **Process A**: Completes build, releases lock (1 second elapsed)
5. **Process B**: Retry succeeds → acquires lock
6. **Process B**: Checks mtimes → inputs unchanged (Process A already built)
7. **Process B**: Skips execution, releases lock

**Result**: Only one process builds; other skips (optimal)

---

## Summary

**Entities**: 3 core (MtimeRecord, MtimeTracker, FileLock) + 2 traits (InputFiles, OutputFiles)

**Storage**: TSV format at `{output_dir}/{cmd_name}-mtimes.tsv`

**Locking**: Shared `.builder-lock` per output directory, 10s timeout

**Decision Logic**: Conservative (rebuild on any uncertainty)

**State Management**: Immutable records, explicit lifecycle transitions

**Trait Integration**: Minimal implementation burden on commands (~10 lines each)

