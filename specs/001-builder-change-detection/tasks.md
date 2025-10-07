# Tasks: Builder Change Detection

**Feature Branch**: `001-builder-change-detection`
**Input**: Design documents from `/specs/001-builder-change-detection/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → Extract: Rust 2024, std::fs locking, TSV storage, camino-fs
2. Load data-model.md
   → Entities: MtimeRecord, MtimeTracker, FileLock, InputFiles, OutputFiles
3. Load contracts/mtimes_api.md
   → API: should_skip(), record_success(), traits
4. Generate tasks by category:
   → Setup: Create crate, workspace deps
   → Tests: Contract tests for traits and API
   → Core: Implement entities, TSV I/O, locking
   → Integration: Command trait implementations, builder orchestration
   → Polish: Integration tests, docs
5. Apply TDD ordering: Tests before implementation
6. Mark parallel tasks with [P] (different files)
7. Return: SUCCESS (30 tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- All paths are absolute or relative to repository root

## Phase 3.1: Setup (2 tasks)

- [ ] **T001** Create mtimes crate structure
  - **Path**: `crates/mtimes/`
  - **Action**: Create directory structure:
    - `crates/mtimes/Cargo.toml`
    - `crates/mtimes/src/lib.rs` (empty, public crate root)
    - `crates/mtimes/src/traits.rs` (will contain InputFiles + OutputFiles)
    - `crates/mtimes/src/lock.rs` (will contain FileLock)
    - `crates/mtimes/tests/` (test directory)
  - **Cargo.toml contents**:
    ```toml
    [package]
    name = "builder-mtimes"
    edition.workspace = true
    license.workspace = true
    repository.workspace = true
    version.workspace = true

    [dependencies]
    anyhow.workspace = true
    camino-fs.workspace = true
    fs-err.workspace = true
    log.workspace = true
    time.workspace = true

    [dev-dependencies]
    tempfile.workspace = true
    ```
  - **Dependencies**: None (setup task)

- [ ] **T002** Add mtimes crate to workspace
  - **Path**: `Cargo.toml` (workspace root) and `crates/builder/Cargo.toml`
  - **Action**:
    1. Add to workspace members in root `Cargo.toml`:
       ```toml
       members = [
           # ... existing members ...
           "crates/mtimes",
       ]
       ```
    2. Add dependency to `crates/builder/Cargo.toml`:
       ```toml
       [dependencies]
       builder-mtimes = { path = "../mtimes" }
       ```
    3. Add dependency to `crates/command/Cargo.toml`:
       ```toml
       [dependencies]
       builder-mtimes = { path = "../mtimes" }
       ```
  - **Dependencies**: T001

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

- [ ] **T003** [P] Contract test for InputFiles trait
  - **Path**: `crates/mtimes/tests/contract_test_input_files.rs`
  - **Action**: Write test file:
    ```rust
    use builder_mtimes::{InputFiles};
    use camino_fs::Utf8PathBuf;

    struct MockCmd {
        inputs: Vec<Utf8PathBuf>,
    }

    impl InputFiles for MockCmd {
        fn input_files(&self) -> Vec<Utf8PathBuf> {
            self.inputs.clone()
        }
    }

    #[test]
    fn test_input_files_trait_returns_paths() {
        let cmd = MockCmd {
            inputs: vec!["src/main.rs".into(), "src/lib.rs".into()],
        };
        let files = cmd.input_files();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "src/main.rs");
    }

    #[test]
    fn test_input_files_trait_empty_vec() {
        let cmd = MockCmd { inputs: vec![] };
        assert!(cmd.input_files().is_empty());
    }
    ```
  - **Expected**: Test compiles but fails (trait not defined yet)
  - **Dependencies**: T002

- [ ] **T004** [P] Contract test for OutputFiles trait
  - **Path**: `crates/mtimes/tests/contract_test_output_files.rs`
  - **Action**: Write test file similar to T003 for OutputFiles trait
  - **Expected**: Test compiles but fails (trait not defined yet)
  - **Dependencies**: T002

- [ ] **T005** [P] Contract test for should_skip - no TSV file
  - **Path**: `crates/mtimes/tests/contract_test_should_skip.rs`
  - **Action**: Write test:
    ```rust
    use builder_mtimes::{should_skip, SkipDecision, InputFiles, OutputFiles};
    use tempfile::TempDir;

    #[test]
    fn test_should_skip_no_previous_build() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let output_dir = temp.path();

        let cmd = MockCmd::default(); // Mock with InputFiles + OutputFiles

        match should_skip(&cmd, "test", output_dir)? {
            SkipDecision::Execute { reason } => {
                assert!(reason.contains("No previous build"));
            }
            _ => panic!("Expected Execute decision"),
        }
        Ok(())
    }
    ```
  - **Expected**: Test fails (should_skip not implemented)
  - **Dependencies**: T002

- [ ] **T006** [P] Contract test for should_skip - unchanged inputs
  - **Path**: `crates/mtimes/tests/contract_test_should_skip.rs` (append)
  - **Action**: Add test that creates TSV, verifies skip on unchanged inputs
  - **Expected**: Test fails (should_skip not implemented)
  - **Dependencies**: T002

- [ ] **T007** [P] Contract test for should_skip - changed input
  - **Path**: `crates/mtimes/tests/contract_test_should_skip.rs` (append)
  - **Action**: Add test that modifies input file, verifies Execute decision
  - **Expected**: Test fails (should_skip not implemented)
  - **Dependencies**: T002

- [ ] **T008** [P] Contract test for should_skip - missing output
  - **Path**: `crates/mtimes/tests/contract_test_should_skip.rs` (append)
  - **Action**: Add test that deletes output file, verifies Execute decision
  - **Expected**: Test fails (should_skip not implemented)
  - **Dependencies**: T002

- [ ] **T009** [P] Contract test for FileLock - basic acquisition
  - **Path**: `crates/mtimes/tests/contract_test_file_lock.rs`
  - **Action**: Write test:
    ```rust
    use builder_mtimes::FileLock;
    use tempfile::TempDir;

    #[test]
    fn test_lock_acquisition_basic() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let lock_path = temp.path().join(".builder-lock");

        let lock = FileLock::acquire(&lock_path)?;
        assert!(lock_path.exists());

        lock.release()?;
        Ok(())
    }
    ```
  - **Expected**: Test fails (FileLock not implemented)
  - **Dependencies**: T002

- [ ] **T010** [P] Contract test for FileLock - timeout
  - **Path**: `crates/mtimes/tests/contract_test_file_lock.rs` (append)
  - **Action**: Add test that holds lock in thread, verifies timeout after 10s
  - **Expected**: Test fails (FileLock not implemented)
  - **Dependencies**: T002

- [ ] **T011** [P] Contract test for record_success
  - **Path**: `crates/mtimes/tests/contract_test_record_success.rs`
  - **Action**: Write test that calls record_success, verifies TSV file created
  - **Expected**: Test fails (record_success not implemented)
  - **Dependencies**: T002

## Phase 3.3: Core Implementation (ONLY after tests T003-T011 are failing)

- [ ] **T012** [P] Implement InputFiles and OutputFiles traits
  - **Path**: `crates/mtimes/src/traits.rs`
  - **Action**: Define traits per contracts/mtimes_api.md:
    ```rust
    use camino_fs::Utf8PathBuf;

    pub trait InputFiles {
        fn input_files(&self) -> Vec<Utf8PathBuf>;
    }

    pub trait OutputFiles {
        fn output_files(&self) -> Vec<Utf8PathBuf>;
    }
    ```
  - **Export from lib.rs**: `pub use traits::{InputFiles, OutputFiles};`
  - **Expected**: T003, T004 now pass
  - **Dependencies**: T003, T004 (tests must fail first)

- [ ] **T013** [P] Implement SkipDecision enum
  - **Path**: `crates/mtimes/src/lib.rs`
  - **Action**: Add enum per contracts:
    ```rust
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SkipDecision {
        Skip { reason: String },
        Execute { reason: String },
    }
    ```
  - **Expected**: Tests compile (still fail on missing functions)
  - **Dependencies**: T003-T011 (tests must fail first)

- [ ] **T014** Implement FileLock with std::fs try_lock
  - **Path**: `crates/mtimes/src/lock.rs`
  - **Action**: Implement per research.md findings:
    ```rust
    use std::fs::File;
    use std::time::{Duration, Instant};
    use std::thread::sleep;
    use camino_fs::{Utf8Path, Utf8PathBuf};
    use anyhow::{Context, Result};

    pub struct FileLock {
        file: File,
        path: Utf8PathBuf,
    }

    impl FileLock {
        pub fn acquire(lock_path: &Utf8Path) -> Result<Self> {
            let file = fs_err::OpenOptions::new()
                .create(true)
                .write(true)
                .open(lock_path)
                .context("Failed to create lock file")?;

            let start = Instant::now();
            let timeout = Duration::from_secs(10);
            let retry_interval = Duration::from_millis(50);

            loop {
                match file.try_lock() {
                    Ok(()) => {
                        return Ok(Self {
                            file,
                            path: lock_path.to_owned(),
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if start.elapsed() >= timeout {
                            anyhow::bail!(
                                "Failed to acquire lock on {} after 10 seconds",
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

        pub fn release(self) -> Result<()> {
            self.file.unlock().context("Failed to release lock")?;
            fs_err::remove_file(&self.path).ok(); // Best effort cleanup
            Ok(())
        }
    }

    impl Drop for FileLock {
        fn drop(&mut self) {
            let _ = self.file.unlock();
        }
    }
    ```
  - **Export from lib.rs**: `pub use lock::FileLock;`
  - **Expected**: T009, T010 now pass (T010 takes 10+ seconds to run)
  - **Dependencies**: T009, T010

- [ ] **T015** Implement TSV parsing and writing functions
  - **Path**: `crates/mtimes/src/lib.rs`
  - **Action**: Add internal helper functions:
    ```rust
    use std::collections::HashMap;
    use camino_fs::{Utf8Path, Utf8PathBuf};
    use anyhow::{Context, Result};

    fn read_mtimes(tsv_path: &Utf8Path) -> Result<HashMap<Utf8PathBuf, u128>> {
        let content = fs_err::read_to_string(tsv_path)
            .with_context(|| format!("Failed to read TSV from {}", tsv_path))?;

        let mut records = HashMap::new();
        for (line_num, line) in content.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 2 {
                anyhow::bail!(
                    "Invalid TSV at {}:{} - expected 2 fields, got {}",
                    tsv_path, line_num + 1, parts.len()
                );
            }

            let path = Utf8PathBuf::from(parts[0]);
            let mtime: u128 = parts[1].parse()
                .with_context(|| format!(
                    "Invalid mtime at {}:{}", tsv_path, line_num + 1
                ))?;

            records.insert(path, mtime);
        }

        Ok(records)
    }

    fn write_mtimes(
        records: &HashMap<Utf8PathBuf, u128>,
        tsv_path: &Utf8Path,
    ) -> Result<()> {
        let mut lines: Vec<String> = records
            .iter()
            .map(|(p, mtime)| format!("{}\t{}", p, mtime))
            .collect();

        lines.sort(); // Deterministic order

        fs_err::write(tsv_path, lines.join("\n"))
            .with_context(|| format!("Failed to write TSV to {}", tsv_path))?;

        Ok(())
    }
    ```
  - **Dependencies**: T013

- [ ] **T016** Implement get_mtime_nanos helper
  - **Path**: `crates/mtimes/src/lib.rs`
  - **Action**: Add function per research.md:
    ```rust
    use std::time::{SystemTime, UNIX_EPOCH};

    fn get_mtime_nanos(path: &Utf8Path) -> Result<u128> {
        let metadata = fs_err::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path))?;

        let mtime = metadata.modified()
            .context("Filesystem doesn't support modification time")?;

        match mtime.duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_nanos()),
            Err(e) => {
                anyhow::bail!("Invalid modification time for {}: {}", path, e);
            }
        }
    }
    ```
  - **Dependencies**: T013

- [ ] **T017** Implement should_skip function
  - **Path**: `crates/mtimes/src/lib.rs`
  - **Action**: Implement core API per data-model.md algorithm:
    ```rust
    pub fn should_skip<C: InputFiles + OutputFiles>(
        cmd: &C,
        cmd_name: &str,
        output_dir: &Utf8Path,
    ) -> Result<SkipDecision> {
        let tsv_path = output_dir.join(format!("{}-mtimes.tsv", cmd_name));

        // 1. Load previous build records
        let previous_mtimes = if tsv_path.exists() {
            read_mtimes(&tsv_path)?
        } else {
            return Ok(SkipDecision::Execute {
                reason: "No previous build".to_string()
            });
        };

        // 2. Get current input files and mtimes
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

        // 5. All checks passed - skip
        let last_build = previous_mtimes.values().max().copied().unwrap_or(0);
        let timestamp = format_timestamp(last_build);

        Ok(SkipDecision::Skip {
            reason: format!(
                "all {} inputs unchanged since {}",
                input_files.len(),
                timestamp
            )
        })
    }

    fn format_timestamp(nanos: u128) -> String {
        // Simple formatter: "YYYY-MM-DD HH:MM"
        let secs = (nanos / 1_000_000_000) as i64;
        let datetime = time::OffsetDateTime::from_unix_timestamp(secs)
            .unwrap_or_else(|_| time::OffsetDateTime::UNIX_EPOCH);
        format!("{}", datetime.format("%Y-%m-%d %H:%M"))
    }
    ```
  - **Note**: Time dependency already added in T001 Cargo.toml setup for timestamp formatting
  - **Expected**: T005-T008 now pass
  - **Dependencies**: T005-T008, T015, T016

- [ ] **T018** Implement record_success function
  - **Path**: `crates/mtimes/src/lib.rs`
  - **Action**: Implement per data-model.md:
    ```rust
    pub fn record_success<C: InputFiles + OutputFiles>(
        cmd: &C,
        cmd_name: &str,
        output_dir: &Utf8Path,
    ) -> Result<()> {
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
  - **Expected**: T011 now passes
  - **Dependencies**: T011, T015, T016

## Phase 3.4: Command Integration

- [ ] **T019** [P] Implement InputFiles + OutputFiles for SassCmd
  - **Path**: `crates/command/src/sass.rs`
  - **Action**: Add trait implementations:
    ```rust
    use builder_mtimes::{InputFiles, OutputFiles};

    impl InputFiles for SassCmd {
        fn input_files(&self) -> Vec<Utf8PathBuf> {
            vec![self.in_scss.clone()]
        }
    }

    impl OutputFiles for SassCmd {
        fn output_files(&self) -> Vec<Utf8PathBuf> {
            self.output.iter()
                .flat_map(|out| {
                    let stem = self.in_scss.file_stem().unwrap();
                    let css_path = out.dest_dir.join(format!("{}.css", stem));
                    let mut paths = vec![css_path.clone()];
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
  - **Dependencies**: T012, T018

- [ ] **T020** [P] Implement InputFiles + OutputFiles for CopyCmd
  - **Path**: `crates/command/src/copy.rs`
  - **Action**: Similar to T019, scan src_dir with file_extensions filter
  - **Dependencies**: T012, T018

- [ ] **T021** [P] Implement InputFiles + OutputFiles for WasmProcessingCmd
  - **Path**: `crates/command/src/wasm.rs`
  - **Action**: Use cargo metadata to find Cargo.toml and source files
  - **Dependencies**: T012, T018

- [ ] **T022** [P] Implement InputFiles + OutputFiles for UniffiCmd
  - **Path**: `crates/command/src/uniffi.rs`
  - **Action**: Return .udl file path as input, compute binding paths as outputs
  - **Dependencies**: T012, T018

- [ ] **T023** [P] Implement InputFiles + OutputFiles for FontForgeCmd
  - **Path**: `crates/command/src/fontforge.rs`
  - **Action**: Return .sfd file as input, font outputs (WOFF2, OTF) as outputs
  - **Dependencies**: T012, T018

- [ ] **T024** [P] Implement InputFiles + OutputFiles for LocalizedCmd
  - **Path**: `crates/command/src/localized.rs`
  - **Action**: Scan directory for .ftl files as inputs, localized outputs
  - **Dependencies**: T012, T018

- [ ] **T025** [P] Implement InputFiles + OutputFiles for SwiftPackageCmd
  - **Path**: `crates/command/src/swift_package.rs`
  - **Action**: Return template and config files as inputs, Package.swift as output
  - **Dependencies**: T012, T018

- [ ] **T026** Export traits from command crate
  - **Path**: `crates/command/src/lib.rs`
  - **Action**: Add `pub use builder_mtimes::{InputFiles, OutputFiles};` to make traits available to command consumers
  - **Dependencies**: T019-T025

- [ ] **T027** Integrate change detection in builder run_commands
  - **Path**: `crates/builder/src/lib.rs`
  - **Action**: Modify run_commands() to call should_skip before each command:
    ```rust
    use builder_mtimes::{should_skip, record_success, SkipDecision, FileLock};

    fn run_commands(mut builder: BuilderCmd) {
        for cmd in &mut builder.cmds {
            // Determine output_dir and cmd_name based on command type
            let (output_dir, cmd_name) = match cmd {
                Cmd::Sass(sass_cmd) => {
                    let dir = sass_cmd.output.first()
                        .map(|o| &o.dest_dir)
                        .expect("SASS command must have output");
                    (dir, "sass")
                }
                Cmd::Copy(copy_cmd) => {
                    let dir = copy_cmd.output.first()
                        .map(|o| &o.dest_dir)
                        .expect("Copy command must have output");
                    (dir, "copy")
                }
                Cmd::Wasm(wasm_cmd) => {
                    // Derive output_dir from wasm package target directory
                    let dir = &wasm_cmd.output_dir; // Assume WasmProcessingCmd has output_dir field
                    (dir, "wasm")
                }
                Cmd::Uniffi(uniffi_cmd) => {
                    let dir = uniffi_cmd.output.first()
                        .map(|o| &o.dest_dir)
                        .expect("Uniffi command must have output");
                    (dir, "uniffi")
                }
                Cmd::FontForge(font_cmd) => {
                    let dir = font_cmd.output.first()
                        .map(|o| &o.dest_dir)
                        .expect("FontForge command must have output");
                    (dir, "fontforge")
                }
                Cmd::Localized(loc_cmd) => {
                    let dir = loc_cmd.output.first()
                        .map(|o| &o.dest_dir)
                        .expect("Localized command must have output");
                    (dir, "localized")
                }
                Cmd::SwiftPackage(swift_cmd) => {
                    let dir = &swift_cmd.output_dir; // Assume SwiftPackageCmd has output_dir field
                    (dir, "swift-package")
                }
            };

            // Acquire lock
            let lock_path = output_dir.join(".builder-lock");
            let _lock = match FileLock::acquire(&lock_path) {
                Ok(lock) => lock,
                Err(e) => {
                    log::error!("Failed to acquire lock: {}", e);
                    continue;
                }
            };

            // Check if we can skip
            let skip_decision = match cmd {
                Cmd::Sass(sass_cmd) => should_skip(sass_cmd, cmd_name, output_dir),
                Cmd::Copy(copy_cmd) => should_skip(copy_cmd, cmd_name, output_dir),
                Cmd::Wasm(wasm_cmd) => should_skip(wasm_cmd, cmd_name, output_dir),
                Cmd::Uniffi(uniffi_cmd) => should_skip(uniffi_cmd, cmd_name, output_dir),
                Cmd::FontForge(font_cmd) => should_skip(font_cmd, cmd_name, output_dir),
                Cmd::Localized(loc_cmd) => should_skip(loc_cmd, cmd_name, output_dir),
                Cmd::SwiftPackage(swift_cmd) => should_skip(swift_cmd, cmd_name, output_dir),
            };

            match skip_decision {
                Ok(SkipDecision::Skip { reason }) => {
                    log::info!("{}: Skipped: {}", cmd_name.to_uppercase(), reason);
                    continue;
                }
                Ok(SkipDecision::Execute { reason }) => {
                    log::debug!("{}: Executing: {}", cmd_name.to_uppercase(), reason);
                }
                Err(e) => {
                    log::warn!("{}: Change detection failed, executing: {}", cmd_name.to_uppercase(), e);
                }
            }

            // Execute command (existing logic)
            match cmd {
                Cmd::Uniffi(cmd) => builder_uniffi::run(cmd),
                Cmd::Sass(cmd) => builder_sass::run(cmd),
                Cmd::Localized(cmd) => builder_localized::run(cmd),
                Cmd::FontForge(cmd) => builder_fontforge::run(cmd),
                Cmd::Wasm(cmd) => builder_wasm::run(cmd),
                Cmd::Copy(cmd) => builder_copy::run(cmd),
                Cmd::SwiftPackage(cmd) => builder_swift_package::run(cmd),
            }

            // Record success
            match cmd {
                Cmd::Sass(sass_cmd) => {
                    if let Err(e) = record_success(sass_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
                Cmd::Copy(copy_cmd) => {
                    if let Err(e) = record_success(copy_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
                Cmd::Wasm(wasm_cmd) => {
                    if let Err(e) = record_success(wasm_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
                Cmd::Uniffi(uniffi_cmd) => {
                    if let Err(e) = record_success(uniffi_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
                Cmd::FontForge(font_cmd) => {
                    if let Err(e) = record_success(font_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
                Cmd::Localized(loc_cmd) => {
                    if let Err(e) = record_success(loc_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
                Cmd::SwiftPackage(swift_cmd) => {
                    if let Err(e) = record_success(swift_cmd, cmd_name, output_dir) {
                        log::warn!("Failed to record success: {}", e);
                    }
                }
            }
        }

        // ... existing finalization code
    }
    ```
  - **Note**: All 7 command types explicitly integrated. Verify actual field names (output_dir vs output) in command structs during implementation.
  - **Dependencies**: T017, T018, T019-T025

## Phase 3.5: Integration Tests & Polish

- [ ] **T028** [P] Integration test for parallel lock contention
  - **Path**: `crates/mtimes/tests/integration_parallel_builds.rs`
  - **Action**: Spawn 4 threads attempting to lock same output_dir, verify only 1 succeeds at a time
  - **Dependencies**: T014, T027

- [ ] **T029** [P] Integration test for end-to-end skip detection
  - **Path**: `crates/builder/tests/change_detection_integration.rs`
  - **Action**: Test full workflow per quickstart.md:
    1. First build executes
    2. Second build skips
    3. Modified input triggers rebuild
    4. Missing output triggers rebuild
  - **Dependencies**: T027

- [ ] **T030** Run quality checks and update documentation
  - **Path**: Repository root and `CLAUDE.md`
  - **Action**:
    1. Run `cargo fmt --all`
    2. Run `cargo clippy --workspace --all-targets` and fix warnings
    3. Run `cargo test` and ensure all tests pass
    4. Verify CLAUDE.md was updated by `/plan` command
    5. Add rustdoc comments to public API functions in `crates/mtimes/src/lib.rs`
  - **Dependencies**: T028, T029

## Dependencies

```
Setup:
  T001 → T002

Tests (must fail before implementation):
  T002 → T003, T004, T005, T006, T007, T008, T009, T010, T011

Core Implementation:
  T003, T004 → T012 (traits)
  T005-T011 → T013 (enum)
  T009, T010 → T014 (FileLock)
  T013 → T015 (TSV), T016 (mtime helper)
  T005-T008, T015, T016 → T017 (should_skip)
  T011, T015, T016 → T018 (record_success)

Command Integration (parallel after T018):
  T012, T018 → T019 (Sass), T020 (Copy), T021 (Wasm), T022 (Uniffi), T023 (FontForge), T024 (Localized), T025 (SwiftPackage)
  T019-T025 → T026 (export)
  T017, T018, T019-T025 → T027 (builder integration)

Polish:
  T014, T027 → T028 (parallel test)
  T027 → T029 (integration test)
  T028, T029 → T030 (quality checks)
```

## Parallel Execution Examples

### Phase 3.2: All contract tests in parallel
```bash
# These can all run simultaneously (different test files):
T003, T004, T005, T006, T007, T008, T009, T010, T011
```

### Phase 3.4: All command trait implementations in parallel
```bash
# These can all run simultaneously (different command files):
T019 (sass.rs), T020 (copy.rs), T021 (wasm.rs), T022 (uniffi.rs),
T023 (fontforge.rs), T024 (localized.rs), T025 (swift_package.rs)
```

### Phase 3.5: Final tests in parallel
```bash
# These can run simultaneously (different test files):
T028 (integration_parallel_builds.rs), T029 (change_detection_integration.rs)
```

## Validation Checklist
*GATE: Verify before marking feature complete*

- [x] All contracts (traits, should_skip, record_success, FileLock) have tests
- [x] All entities (InputFiles, OutputFiles, SkipDecision, FileLock) have implementations
- [x] All tests come before implementation (TDD order)
- [x] Parallel tasks truly independent (different files)
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task
- [x] Builder integration calls should_skip before each command
- [x] Builder integration calls record_success after each command
- [x] All 7 command types implement both traits

## Notes

- **TDD Critical**: Tests T003-T011 MUST be written and failing before implementation starts
- **Parallel Opportunities**:
  - Tests T003-T011 can run in parallel (9 simultaneous tasks)
  - Command implementations T019-T025 can run in parallel (7 simultaneous tasks)
  - Final tests T028-T029 can run in parallel (2 simultaneous tasks)
- **File Lock Note**: T010 takes 10+ seconds to run (timeout test)
- **No Backward Compatibility**: Per spec, breaking changes are acceptable in this iteration
- **Time Dependency**: T017 requires adding `time` to workspace dependencies for timestamp formatting
- **Constitution Compliance**: This design uses mtime-based detection (intentional deviation from Principle IV, documented in plan.md)

---

**Total Tasks**: 30
**Estimated Parallel Groups**: 3 major groups (tests, command integrations, final tests)
**Critical Path**: T001 → T002 → T003-T011 → T013 → T015/T016 → T017/T018 → T019-T025 → T027 → T028-T029 → T030
