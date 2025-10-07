# Quickstart: Builder Change Detection

**Feature**: Mtime-based build skipping for builder commands
**Status**: Design (not yet implemented)
**Estimated Time**: 5 minutes

## Overview

This quickstart validates that builder commands automatically skip execution when input files haven't changed. You'll run a build, verify it executes, run again to verify it skips, then modify an input to verify it re-executes.

## Prerequisites

- Rust workspace with builder installed
- A project using `builder::execute()` in `build.rs`
- At least one builder command (SASS, WASM, Copy, etc.)

## Quick Test (5 Minutes)

### Step 1: Create Test Command

Add to your `build.rs`:

```rust
use builder::builder_command::{BuilderCmd, SassCmd, Output};

fn main() {
    let cmd = BuilderCmd::new()
        .add_sass(SassCmd::new("styles/main.scss")
            .add_output(Output::new("dist")));

    builder::execute(cmd);
}
```

**Expected**: Build script compiles successfully.

### Step 2: First Build (Should Execute)

```bash
cargo build
```

**Expected Output**:
```
SASS: Processing file: styles/main.scss
SASS: Compilation successful (1234 bytes)
```

**Verify**:
- Check `dist/sass-mtimes.tsv` exists
- Contains line like: `styles/main.scss	1728234567890123456`
- Check `dist/.builder-lock` does NOT exist (cleaned up after build)

### Step 3: Second Build (Should Skip)

```bash
cargo build
```

**Expected Output**:
```
SASS: Skipped: all 1 inputs unchanged since 2025-10-06 14:32
```

**Verify**:
- No "Processing file" message
- Build completes quickly (< 1 second)
- `dist/sass-mtimes.tsv` unchanged

###Step 4: Modify Input (Should Re-Execute)

```bash
touch styles/main.scss
cargo build
```

**Expected Output**:
```
SASS: Executing: Input changed: styles/main.scss
SASS: Processing file: styles/main.scss
SASS: Compilation successful (1234 bytes)
```

**Verify**:
- "Processing file" message appears (command executed)
- `dist/sass-mtimes.tsv` updated with new timestamp

### Step 5: Delete Output (Should Re-Execute)

```bash
rm dist/main.css
cargo build
```

**Expected Output**:
```
SASS: Executing: Output missing: dist/main.css
SASS: Processing file: styles/main.scss
SASS: Compilation successful (1234 bytes)
```

**Verify**:
- Command executed even though input unchanged
- `dist/main.css` regenerated

## Parallel Builds Test (10 Minutes)

Test lock behavior with parallel builds for different targets.

### Step 1: Build for Native Target

In terminal 1:
```bash
cargo build
```

### Step 2: Build for Different Target Simultaneously

In terminal 2 (while terminal 1 is building):
```bash
cargo build --target x86_64-unknown-linux-gnu
```

**Expected Behavior**:
- One build acquires `.builder-lock` immediately
- Other build waits (up to 10 seconds)
- After first completes, second proceeds
- Second likely skips (first already built outputs)

**Verify**:
- No "lock timeout" errors
- No corrupted output files
- Only one "Processing file" message (other skips)

## Verification Checklist

After completing quickstart:

- [ ] **First build executes** command
- [ ] **TSV file created** at `dist/{cmd-name}-mtimes.tsv`
- [ ] **Second build skips** with reason logged
- [ ] **Modified input triggers rebuild**
- [ ] **Missing output triggers rebuild**
- [ ] **Lock file cleaned up** after build (not present)
- [ ] **Parallel builds don't conflict** (one waits for other)
- [ ] **Skip reason shows timestamp** (e.g., "all 1 inputs unchanged since 2025-10-06 14:32")

## Troubleshooting

### Problem: Always Rebuilds (Never Skips)

**Symptoms**: Every `cargo build` executes command, never skips.

**Possible Causes**:
1. **TSV file not created** → Check output directory is writable
2. **Input paths incorrect** → Check `InputFiles` trait implementation returns absolute paths
3. **Filesystem precision issue** → Check inode size on ext4: `sudo tune2fs -l /dev/sdX | grep "Inode size"` (need 256+ bytes)

**Debug**:
```bash
# Check TSV file exists and has content
cat dist/sass-mtimes.tsv

# Check TSV format (should be: path<TAB>number)
od -c dist/sass-mtimes.tsv

# Enable verbose logging
RUST_LOG=trace cargo build
```

### Problem: Lock Timeout Error

**Symptoms**: Build fails with "Failed to acquire lock after 10 seconds".

**Possible Causes**:
1. **Stale process holding lock** → Find and kill: `lsof dist/.builder-lock`
2. **Infinite build loop** → Previous build still running
3. **Lock file permissions** → Check writable: `ls -la dist/.builder-lock`

**Fix**:
```bash
# Find process holding lock
lsof dist/.builder-lock | grep builder

# Kill stale process
kill -9 <PID>

# Or delete lock file (safe - lock is file-descriptor based)
rm dist/.builder-lock
```

### Problem: Permission Denied on TSV File

**Symptoms**: Build fails with "Permission denied" writing TSV.

**Possible Causes**:
1. **Output directory not writable** → Check permissions
2. **TSV file owned by different user** → Check ownership

**Fix**:
```bash
# Check output directory permissions
ls -la dist/

# Fix permissions
chmod -R u+w dist/

# Fix ownership (if needed)
sudo chown -R $USER dist/
```

### Problem: Skip Reason Shows Wrong Timestamp

**Symptoms**: Skip message shows incorrect date/time.

**Possible Causes**:
1. **System clock incorrect** → Check: `date`
2. **Filesystem mounted with wrong timezone** → Check: `mount | grep dist`
3. **TSV contains corrupted timestamp** → Check: `cat dist/sass-mtimes.tsv`

**Fix**:
```bash
# Check system time
date

# Rebuild TSV file from scratch
rm dist/sass-mtimes.tsv
cargo build  # Will recreate with current timestamps
```

### Problem: Parallel Builds Corrupt Outputs

**Symptoms**: Output files have mixed content from different builds.

**Possible Causes**:
1. **Lock not acquired** → Bug in FileLock implementation
2. **Different output directories** → Builds using different paths (lock per directory)
3. **Lock file deleted prematurely** → Bug in Drop implementation

**Debug**:
```bash
# Check if lock file is being used
strace cargo build 2>&1 | grep .builder-lock

# Verify flock syscalls are being made
strace -e flock cargo build
```

**Workaround**: Run builds sequentially until bug fixed:
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu
```

## Advanced Usage

### Manually Inspect Mtimes

```bash
# View TSV file
cat dist/sass-mtimes.tsv

# Compare with actual file mtimes
stat -c '%Y%N' styles/main.scss  # Linux
stat -f '%m %N' styles/main.scss  # macOS
```

### Force Rebuild

```bash
# Delete TSV file to force rebuild (even if inputs unchanged)
rm dist/sass-mtimes.tsv
cargo build
```

### Disable Change Detection (Temporarily)

No built-in disable flag - workaround:

```bash
# Delete TSV before every build (always rebuilds)
rm dist/*-mtimes.tsv && cargo build
```

### Inspect Lock Contention

```bash
# Terminal 1: Start slow build
cargo build --release  # Takes longer

# Terminal 2: Monitor lock file
watch -n 0.1 'ls -la dist/.builder-lock'

# Terminal 3: Try concurrent build
cargo build  # Should wait for lock
```

## Performance Expectations

| Scenario | Expected Duration |
|----------|-------------------|
| **First build** (no TSV) | Full command execution time |
| **Skip decision** (unchanged) | < 100ms overhead |
| **Mtime check** (1000 files) | < 50ms |
| **Lock acquisition** (no contention) | < 10ms |
| **Lock acquisition** (contention) | 50ms - 10 seconds |
| **TSV read/write** | < 10ms for typical projects |

## Success Criteria

You've successfully validated change detection if:

1. ✅ First build executes command
2. ✅ Subsequent builds skip when inputs unchanged
3. ✅ Modified inputs trigger rebuild
4. ✅ Missing outputs trigger rebuild
5. ✅ Parallel builds don't corrupt outputs or timeout
6. ✅ Skip reason logged with timestamp
7. ✅ Lock file cleaned up after builds

## Next Steps

After validating basic functionality:

1. **Test with other commands**: Try CopyCmd, WasmProcessingCmd, UniffiCmd
2. **Test with multiple inputs**: Commands with many input files (LocalizedCmd, CopyCmd)
3. **Test complex scenarios**: Multiple commands in one BuilderCmd
4. **Measure performance**: Compare build times with vs without change detection

## Feedback

If you encounter issues not covered in troubleshooting:

1. Check builder logs: `RUST_LOG=debug cargo build`
2. Verify TSV format: `cat dist/{cmd}-mtimes.tsv`
3. Report bug with reproduction steps

---

**Estimated Completion Time**: 5-10 minutes for basic test, 20 minutes with troubleshooting and advanced scenarios.
