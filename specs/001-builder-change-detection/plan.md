# Implementation Plan: Builder Change Detection

**Branch**: `001-builder-change-detection` | **Date**: 2025-10-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-builder-change-detection/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from file system structure or context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code, or `AGENTS.md` for all other agents).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary

Add **mtime-based change detection** to builder commands to skip unnecessary rebuilds when input files haven't changed. This feature enables build.rs scripts to avoid executing expensive build commands (SASS, WASM, UniFFI, etc.) when inputs and outputs are unchanged. The implementation uses modification timestamps (not content hashing) with file-based locking for parallel build safety.

**Technical approach**: Add a lightweight `crates/mtimes` library that tracks file modification times in TSV format, provides file locking via std::fs, and integrates with existing command infrastructure through minimal API changes.

**Key design principles**:
- **Mtime-only detection**: Use Rust's full-precision modification times (nanoseconds since epoch as u128)
- **TSV format**: Store metadata as `<cmd-name>-mtimes.tsv` (path, mtime pairs) instead of JSON
- **Std-only locking**: Use `std::fs::File::try_lock()` with 10-second timeout
- **Minimal scaffolding**: Commands provide input/output file lists via simple traits
- **Parallel-safe**: Shared `.builder-lock` file prevents concurrent writes to same output directory

## Technical Context

**Language/Version**: Rust 2024 edition (workspace standard)
**Primary Dependencies**:
- std::fs for file locking (`File::try_lock()`)
- std::time for SystemTime/Duration (mtime extraction)
- camino-fs for UTF-8 path handling (workspace standard)
- No external locking crates (fs2, file-lock, etc.)

**Storage**: TSV files (`<cmd-name>-mtimes.tsv`) in output directory root
**Testing**: cargo test, integration tests for lock contention and mtime detection
**Target Platform**: Linux and macOS (Windows not officially supported per constitution)
**Project Type**: Single workspace (Rust library crates + binary)
**Performance Goals**:
- Lock acquisition < 10ms under no contention
- Mtime check overhead < 50ms for 1000 files
- Skip decision < 100ms total for typical projects

**Constraints**:
- Must use only std::fs for locking (no external deps)
- Must not break existing command interfaces
- Must work in parallel cargo builds (different targets)
- Lock timeout: 10 seconds

**Scale/Scope**:
- Support ~10,000 input files per command
- Handle parallel builds across 4+ targets simultaneously
- TSV file size < 1MB for typical projects

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Library-First Architecture
- [x] New functionality begins in `crates/` as a library crate
  - Will create `crates/mtimes/` for change detection logic
- [x] Library is self-contained with explicit dependencies in `Cargo.toml`
  - Only depends on std, camino-fs (workspace dep)
- [x] Library does NOT depend on binary crate (`crates/builder`)
  - `crates/mtimes` → `crates/command` (adds trait methods) → `crates/builder` (orchestration)
- [x] Crate has singular, focused purpose (not organizational-only)
  - Focused: mtime tracking, file locking, skip detection

### II. Relentless Simplicity
- [x] Code refactored to simplest possible implementation with all required features
  - Using std::fs for locking (no fs2/file-lock deps)
  - TSV format (simpler than JSON, no serde overhead for this use case)
  - Mtime-only (simpler than content hashing for this feature)
- [x] New dependencies justified (document: features needed, why custom implementation insufficient)
  - **No new dependencies**: Using std::fs, std::time, camino-fs (already in workspace)
- [x] No unused dependencies (removed during refactoring)
  - N/A - not adding any
- [x] Prefer Rust std library and lightweight crates over heavy frameworks
  - Exclusively using std::fs for locking instead of external crates
- [x] Complexity challenged in design review: "Can this be simpler?"
  - Yes: TSV instead of JSON, std::fs instead of external lock crates

### III. Configuration-Driven Design
- [x] All operations expressible through YAML/JSON configuration
  - No config changes needed - detection is automatic per spec
- [x] Command struct implements serde `Serialize` + `Deserialize`
  - Existing commands already do; no changes to command structs
- [x] Configuration changes are backward compatible OR versioned appropriately
  - No config changes - feature is transparent to users
- [x] No hard-coded workflows or business logic in config parsing
  - Detection logic lives in library, not config layer

### IV. Content-Based Caching
- [ ] Caching uses `seahash` for content hashing (not modification times)
  - **VIOLATION**: This feature uses mtimes, not content hashing
- [ ] Cache keys include all inputs: source files, config, CLI parameters
  - **VIOLATION**: Uses mtime comparison, not cache keys
- [ ] Cache invalidation based on hash comparison
  - **VIOLATION**: Invalidation based on mtime comparison
- [ ] Cache misses log reason for debugging
  - [x] COMPLIANT: FR-013 requires logging skip reasons

**Note**: This feature intentionally uses mtime-based detection per user requirement. Content hashing remains for web asset cache-busting (existing feature, unaffected). Both mechanisms coexist:
- **Content hashing** (existing): Web asset fingerprinting, CDN cache busting
- **Mtime detection** (new): Build skip optimization for build.rs

### V. Workspace Modularity
- [x] No cross-dependencies between feature crates
  - `crates/mtimes` is standalone, no deps on sass/wasm/uniffi/etc.
- [x] Shared code goes in `crates/common` only
  - Will add trait to `crates/command` (not common - it's command-specific)
- [x] Maintains dependency hierarchy: common → features → command → binary
  - `crates/mtimes` (standalone) → `crates/command` (trait integration) → `crates/builder` (orchestration)
- [x] No circular dependencies introduced
  - Linear hierarchy maintained

### VI. CLI Interface Standard
- [x] Functionality executable via `builder` CLI
  - Transparent - existing CLI unchanged, detection automatic
- [x] Text protocol: YAML/JSON in → stdout (results), stderr (errors)
  - Existing protocol maintained
- [x] Proper exit codes: 0 = success, non-zero = failure
  - Lock timeout failures will use non-zero exit codes
- [x] No interactive prompts in automated/CI mode
  - Fully automatic, no prompts

### VII. Code Organization
- [x] Keep files under 300 lines when practical; refactor files over 600 lines; MUST split files over 1000 lines
  - Expect ~200-300 lines total for mtimes crate
- [x] Prefer `impl MyType { fn method(&self) }` over standalone `fn process_my_type(t: &MyType)`
  - Will use `impl Cmd` and trait methods
- [x] Use trait implementations to add methods to external types when cohesive
  - Will add `InputFiles` and `OutputFiles` traits to command types
- [x] Each file SHOULD contain a single primary struct/enum definition with its implementations
  - lib.rs: core logic; traits.rs: trait definitions
- [x] Split large implementations across multiple files using submodules when necessary
  - Not needed - small crate

### Quality Standards
- [x] Unit tests for all public functions
  - Test mtime comparison, TSV parsing, lock acquisition
- [x] Integration tests for command execution
  - Test parallel builds, lock contention, skip detection
- [x] Contract tests for new command types (config parsing)
  - N/A - no new command types, only internal feature
- [x] Public APIs have doc comments with examples
  - Document traits and public functions
- [x] Error handling uses `anyhow::Result` with context chains
  - Will use anyhow for user-facing errors

**Violations/Justifications**:

| Principle | Violation | Justification |
|-----------|-----------|---------------|
| IV. Content-Based Caching | Uses mtimes instead of seahash content hashing | Per user specification: "Use mtimes exclusively" and "content hashing is already used for cache-busting for web assets and is optional." This feature serves a different purpose (build.rs skip optimization) than web asset caching. Both mechanisms coexist for their respective use cases. Mtime-based detection is appropriate for build.rs because: (1) build.rs runs in controlled environments where clock drift is minimal, (2) mtime check is ~100x faster than content hashing for large file sets, (3) spec explicitly accepts occasional unnecessary rebuilds from clock drift. |

## Project Structure

### Documentation (this feature)
```
specs/001-builder-change-detection/
├── plan.md              # This file (/plan command output)
├── spec.md              # Feature specification (completed)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
crates/
├── mtimes/              # NEW: Change detection library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs       # Core: MtimeTracker, file locking, TSV I/O
│   │   ├── traits.rs    # InputFiles + OutputFiles traits for commands
│   │   └── lock.rs      # File locking with std::fs::File::try_lock
│   └── tests/
│       ├── integration.rs   # Parallel build, lock contention tests
│       └── mtime_tests.rs   # TSV parsing, mtime comparison
│
├── command/             # MODIFIED: Add trait implementations
│   └── src/
│       ├── lib.rs       # Import mtimes traits, add `pub use mtimes::*`
│       ├── sass.rs      # Implement InputFiles + OutputFiles for SassCmd
│       ├── copy.rs      # Implement InputFiles + OutputFiles for CopyCmd
│       ├── wasm.rs      # Implement InputFiles + OutputFiles for WasmProcessingCmd
│       ├── uniffi.rs    # Implement InputFiles + OutputFiles for UniffiCmd
│       └── ...          # Other command types
│
├── builder/             # MODIFIED: Integrate detection in run_commands
│   └── src/
│       └── lib.rs       # Call should_skip() before each command execution
│
└── common/              # UNCHANGED: No modifications needed
```

**Structure Decision**: Single workspace structure. This is a cross-cutting library feature that integrates with the existing command execution pipeline. The `crates/mtimes` library is independent and focused, with trait-based integration into command types.

## Phase 0: Outline & Research

### Research Tasks

1. **std::fs file locking on macOS and Linux**
   - Investigate `std::fs::File::try_lock()` behavior
   - Confirm 10-second timeout implementation approach
   - Validate lock cleanup on panic/process crash
   - Document any platform-specific quirks

2. **SystemTime precision and mtime extraction**
   - Confirm nanosecond precision availability on macOS/Linux
   - Document conversion to u128 (nanos since UNIX_EPOCH)
   - Test behavior with filesystems that don't support nanosecond precision
   - Validate handling of clock drift scenarios

3. **TSV format design for mtime storage**
   - Research escaping requirements for file paths in TSV
   - Evaluate trade-offs vs JSON (simplicity, size, parsing speed)
   - Document format: `<relative-path><TAB><mtime-nanos><NEWLINE>`
   - Confirm UTF-8 safety with camino-fs paths

4. **Command input/output file enumeration patterns**
   - Survey existing commands (sass, wasm, uniffi, copy) for file access patterns
   - Identify common patterns for input discovery (single file, directory scan, config-based)
   - Identify output patterns (single file, multiple outputs, Output struct usage)
   - Design trait API that minimizes implementation burden

5. **Lock file naming and cleanup patterns**
   - Research best practices for lock file cleanup in Rust
   - Investigate `Drop` implementation vs explicit cleanup
   - Document failure modes (crash, kill -9, timeout)
   - Design strategy for stale lock detection (out of scope per spec, but document)

### Research Consolidation

**Output**: `research.md` with:
- Decision: Use `std::fs::File::try_lock()` with custom timeout loop
- Decision: TSV format with tab-separated `path<TAB>mtime_nanos`
- Decision: Trait-based API: `trait InputFiles` + `trait OutputFiles`
- Decision: Lock cleanup via `Drop` guard + explicit release on success
- Alternatives considered: fs2 crate (rejected - adds dependency), JSON format (rejected - overhead), flock syscall wrapper (rejected - std::fs sufficient)

## Phase 1: Design & Contracts

### 1. Data Model (`data-model.md`)

#### Entities

**MtimeRecord**
- Fields: `path: Utf8PathBuf`, `mtime_nanos: u128`
- Relationships: Multiple records per MtimeTracker
- Validation: Path must be relative to base directory
- State: Immutable after creation

**MtimeTracker**
- Fields: `base_dir: Utf8PathBuf`, `records: HashMap<Utf8PathBuf, u128>`
- Relationships: One per command execution
- Lifecycle: Create → Load existing TSV → Compare → Save updated TSV
- State transitions: Empty → Loaded → Compared → Saved

**FileLock**
- Fields: `file: File`, `path: Utf8PathBuf`
- Relationships: One per output directory
- Lifecycle: Acquire → Hold → Release (via Drop)
- State: Locked → Unlocked

#### Trait Contracts

**InputFiles trait**
- Methods: `fn input_files(&self) -> Vec<Utf8PathBuf>`
- Implementations: SassCmd, CopyCmd, WasmProcessingCmd, UniffiCmd, FontForgeCmd, LocalizedCmd, SwiftPackageCmd
- Contract: Returns absolute paths to all input files; empty vec if none

**OutputFiles trait**
- Methods: `fn output_files(&self) -> Vec<Utf8PathBuf>`
- Implementations: Same as InputFiles
- Contract: Returns absolute paths to all expected output files; empty vec if self-discovering

### 2. API Contracts (`contracts/`)

#### `crates/mtimes/src/lib.rs` Public API

```rust
/// Check if command should skip execution based on mtime comparison
pub fn should_skip<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> anyhow::Result<SkipDecision>;

pub enum SkipDecision {
    Skip { reason: String },         // Log and skip
    Execute { reason: String },      // Log and execute
}

/// Record successful build completion
pub fn record_success<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> anyhow::Result<()>;
```

#### `crates/mtimes/src/traits.rs`

```rust
pub trait InputFiles {
    fn input_files(&self) -> Vec<Utf8PathBuf>;
}

pub trait OutputFiles {
    fn output_files(&self) -> Vec<Utf8PathBuf>;
}
```

#### `crates/mtimes/src/lock.rs`

```rust
pub struct FileLock {
    file: File,
    path: Utf8PathBuf,
}

impl FileLock {
    /// Acquire exclusive lock with 10-second timeout
    pub fn acquire(lock_path: &Utf8Path) -> anyhow::Result<Self>;

    /// Release lock explicitly (also released via Drop)
    pub fn release(self) -> anyhow::Result<()>;
}
```

### 3. Contract Tests

File: `crates/mtimes/tests/contract_tests.rs`

```rust
#[test]
fn test_input_files_trait_basic() {
    // Create mock command implementing InputFiles
    // Verify input_files() returns expected paths
}

#[test]
fn test_output_files_trait_basic() {
    // Create mock command implementing OutputFiles
    // Verify output_files() returns expected paths
}

#[test]
fn test_should_skip_with_no_mtimes_file() {
    // First run: no mtimes file exists
    // Result: Execute { reason: "No previous build" }
}

#[test]
fn test_should_skip_with_unchanged_inputs() {
    // Record build, run again with same inputs
    // Result: Skip { reason: "all N inputs unchanged since TIMESTAMP" }
}

#[test]
fn test_should_skip_with_changed_inputs() {
    // Record build, modify input, run again
    // Result: Execute { reason: "1 input changed" }
}

#[test]
fn test_should_skip_with_missing_outputs() {
    // Record build, delete output, run again
    // Result: Execute { reason: "1 output missing" }
}

#[test]
fn test_lock_acquisition_basic() {
    // Acquire lock
    // Verify file exists
    // Release lock
}

#[test]
fn test_lock_acquisition_timeout() {
    // Hold lock in thread
    // Attempt acquire in main thread
    // Verify timeout after 10 seconds
}
```

### 4. Integration Tests

File: `crates/mtimes/tests/integration_tests.rs`

```rust
#[test]
fn test_parallel_lock_contention() {
    // Spawn 4 threads attempting to lock same output_dir
    // Verify only 1 succeeds at a time
    // Verify all complete within timeout
}

#[test]
fn test_mtime_detection_across_rebuild() {
    // Execute command, record mtimes
    // Modify input file
    // Verify should_skip() returns Execute
}

#[test]
fn test_tsv_format_with_special_paths() {
    // Test paths with spaces, unicode, special chars
    // Verify TSV parsing handles correctly
}
```

### 5. Update CLAUDE.md

Run: `.specify/scripts/bash/update-agent-context.sh claude`

Add to "Command Types" section:
```markdown
## Change Detection (Mtime-Based)

Builder automatically skips commands when inputs haven't changed:
- Uses modification timestamps (full nanosecond precision)
- Stores mtimes in `<cmd-name>-mtimes.tsv` (TSV format)
- Parallel-safe with `.builder-lock` file locking
- Commands implement `InputFiles` + `OutputFiles` traits
```

Add to "Working with Command Modules" section:
```markdown
### Implementing Change Detection

When adding new commands:
1. Implement `InputFiles` trait: return all input file paths
2. Implement `OutputFiles` trait: return all expected output paths
3. Change detection is automatic in `builder::execute()`
```

### 6. Quickstart (`quickstart.md`)

```markdown
# Quickstart: Builder Change Detection

## Prerequisites
- Rust workspace with builder commands
- build.rs using builder::execute()

## Quick Test

1. **Create test command**:
   ```rust
   let cmd = BuilderCmd::new()
       .add_sass(SassCmd::new("styles/main.scss")
           .add_output(Output::new("dist")));
   builder::execute(cmd);
   ```

2. **First run** (will execute):
   ```
   cargo build
   # Output: "SASS: Processing file: styles/main.scss"
   ```

3. **Second run** (will skip):
   ```
   cargo build
   # Output: "SASS: Skipped: all 1 inputs unchanged since 2025-10-06 14:32"
   ```

4. **Modify input and run** (will execute):
   ```
   touch styles/main.scss
   cargo build
   # Output: "SASS: Processing file: styles/main.scss (1 input changed)"
   ```

## Verification

- Check for `sass-mtimes.tsv` in output directory
- Check for `.builder-lock` (cleaned up after build)
- Verify parallel builds don't conflict (run `cargo build` for multiple targets)

## Troubleshooting

- **Lock timeout**: Check for stale processes holding lock (> 10 seconds is timeout)
- **Always rebuilds**: Verify mtimes file exists and paths are correct
- **Permission errors**: Ensure output directory is writable
```

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
1. Load contract tests from Phase 1
2. Create tasks for library implementation (`crates/mtimes`)
   - Core: TSV parsing, mtime extraction, comparison logic
   - Locking: FileLock implementation with std::fs
   - Traits: InputFiles + OutputFiles definitions
3. Create tasks for command integration (`crates/command`)
   - Each command type gets 1 task: implement both traits
4. Create tasks for builder integration (`crates/builder`)
   - Modify `run_commands()` to call `should_skip()` before execution
5. Create tasks for testing
   - Unit tests (contract tests from Phase 1)
   - Integration tests (parallel builds, lock contention)
6. Create tasks for documentation
   - Update CLAUDE.md (already drafted in Phase 1)
   - Update README.md with change detection section

**Ordering Strategy**:
- TDD order: Contract tests → Library implementation → Command integration → Builder integration
- Dependency order: Library → Traits → Command impls → Builder orchestration
- Parallel opportunities [P]:
  - All command trait implementations can be done in parallel
  - Unit tests and integration tests can be written in parallel
  - Documentation tasks can be done in parallel with testing

**Estimated Output**: ~25-30 tasks in tasks.md

**Key task groups**:
1. Setup (2 tasks): Create crate, add dependencies
2. Library core (5 tasks): TSV I/O, mtime tracking, comparison logic
3. File locking (3 tasks): FileLock impl, timeout, cleanup
4. Traits (2 tasks): Define + document traits
5. Command integration (7 tasks): One per command type [P]
6. Builder integration (2 tasks): should_skip() calls, logging
7. Testing (6 tasks): Unit + integration tests
8. Documentation (3 tasks): CLAUDE.md, README.md, rustdoc

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)
**Phase 4**: Implementation (execute tasks.md following constitutional principles)
**Phase 5**: Validation (run tests, execute quickstart.md, verify parallel builds)

## Complexity Tracking

No violations requiring complexity justification beyond Constitution Check section.

## Progress Tracking

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [x] Phase 3: Tasks generated (/tasks command) - 30 tasks created
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS (with justified mtime violation)
- [x] Post-Design Constitution Check: PASS (no new violations introduced)
- [x] All NEEDS CLARIFICATION resolved (via /clarify)
- [x] Complexity deviations documented (Principle IV mtime usage)

---
*Based on Constitution v1.2.0 - See `.specify/memory/constitution.md`*
