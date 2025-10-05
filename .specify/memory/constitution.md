# Builder Constitution

<!--
Sync Impact Report - Constitution Amendment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

VERSION CHANGE: 1.1.0 → 1.2.0

RATIONALE: Adding new code organization principles, expanding error handling guidance,
and documenting platform requirements. MINOR bump because this materially expands
governance with new principles without removing or redefining existing core principles.

MODIFIED PRINCIPLES:
- Principle V: "Workspace Modularity" → expanded with workspace-defined dependencies rule

ADDED PRINCIPLES:
- Principle VII: "Code Organization" (new - method implementation, traits, file size)

EXPANDED SECTIONS:
- Quality Standards → Error Handling (layered types, propagation patterns, messages)
- Quality Standards → Documentation Requirements (close to source, justified documentation)
- Quality Standards → Platform Requirements (new section - Linux & macOS)

SECTIONS UNCHANGED:
- Principles I-IV, VI (Library-First, Relentless Simplicity, Configuration-Driven,
  Content-Based Caching, CLI Interface)
- Development Workflow
- Governance

TEMPLATE DEPENDENCIES:
✅ plan-template.md - Already references constitution generally; no changes needed
✅ spec-template.md - Generic enough; no changes needed
✅ tasks-template.md - Generic enough; no changes needed
✅ No command template files found

FOLLOW-UP TODOS:
- Consider adding linting rules to enforce code file size limits (300/600/1000 lines)
- Consider CI checks for workspace dependency definitions
- Review CLAUDE.md alignment with expanded error handling guidance

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
-->

## Core Principles

### I. Library-First Architecture

Every feature MUST start as a standalone library crate. Libraries MUST be self-contained, independently testable, and documented. Each crate MUST have a clear, singular purpose - no organizational-only crates without functional value.

**Rationale**: The workspace architecture (`crates/sass`, `crates/wasm`, `crates/uniffi`, etc.) demonstrates that separation enables independent testing, versioning, and reuse. Each command type lives in its own crate with focused responsibility.

**Non-negotiable rules**:
- New functionality begins in `crates/` as a library
- Libraries cannot depend on the binary crate (`crates/builder`)
- Each crate has its own `Cargo.toml` with explicit dependencies
- Libraries expose public APIs without CLI coupling

### II. Relentless Simplicity

Code MUST be refactored to be the simplest possible implementation that includes all required features. Complexity MUST be justified before introduction and eliminated when features change. Prefer custom implementations of simple functionality over heavy dependencies with unused features ("close-to-the-metal" approach).

**Rationale**: Unnecessary complexity is technical debt that compounds over time. Large dependencies increase build times, binary size, and maintenance burden when only a fraction of functionality is needed. Builder demonstrates this with `grass` (built-in SCSS compiler) as fallback when dart-sass unavailable, and `seahash` (lightweight) over heavier hashing libraries.

**Non-negotiable rules**:
- Refactor code to simplest form after feature changes or additions
- Justify new dependencies by documenting: what features needed, why custom implementation insufficient
- Remove unused dependencies during refactoring passes
- Prefer Rust standard library and lightweight crates over frameworks
- Code reviews MUST challenge complexity: "Can this be simpler?"

### III. Configuration-Driven Design

All build operations MUST be expressible through declarative configuration. The tool MUST be entirely driven by YAML/JSON configuration files with no hard-coded workflows.

**Rationale**: Builder's core value is reading `builder.yaml`, parsing to `BuilderCmd`, and executing commands. Configuration-as-data enables programmatic generation from build scripts, reproducible builds, and inspection/validation without execution.

**Non-negotiable rules**:
- New commands MUST implement serde `Serialize` + `Deserialize`
- Configuration changes MUST be backward compatible (MINOR version) or versioned (MAJOR)
- All command parameters MUST be expressible in YAML/JSON
- No business logic in configuration parsing - only data structure mapping

### IV. Content-Based Caching

All build operations MUST implement caching based on content hashes (seahash), not modification times. Caches MUST be invalidated when inputs change, not on arbitrary time intervals.

**Rationale**: Build tools must avoid unnecessary work. Content hashing provides reliable change detection across file moves, timestamp resets, and distributed builds. All existing commands (`sass`, `wasm`, `uniffi`, `fontforge`) use content-based caching.

**Non-negotiable rules**:
- Use `seahash` for content hashing (workspace standard)
- Cache keys MUST include all inputs: source files, config, CLI parameters
- Cache invalidation MUST be based on hash comparison, not time
- Cache misses MUST log reason (new hash, missing cache, etc.)

### V. Workspace Modularity

The workspace MUST maintain strict dependency hierarchy: common utilities → feature crates → command library → binary. Cross-dependencies between feature crates are prohibited. All dependencies MUST be defined in the workspace `Cargo.toml` and referenced via `workspace = true`.

**Rationale**: Circular dependencies create coupling and prevent independent development. The current structure (`common` → `sass`/`wasm`/etc. → `command` → `builder`) enforces clean boundaries and enables parallel development. Workspace-level dependency definitions ensure version consistency and reduce maintenance overhead.

**Non-negotiable rules**:
- Feature crates (`sass`, `wasm`, etc.) MUST NOT depend on each other
- All shared code goes in `crates/common`
- Binary crate (`crates/builder`) is the only executable entry point
- Examples crate excluded from workspace to enforce clean boundaries
- All dependencies MUST be defined in workspace `Cargo.toml` with `[workspace.dependencies]`
- Crate-level `Cargo.toml` files MUST use `dependency.workspace = true` for all deps

### VI. CLI Interface Standard

Every library MUST expose functionality via a command-line interface. Commands MUST follow text protocol: configuration in (YAML/JSON) → stdout (results), errors → stderr. No GUI-only libraries.

**Rationale**: CLI-first design ensures automation, scripting, and CI/CD integration. The builder binary itself demonstrates this: YAML in, build outputs, errors to stderr.

**Non-negotiable rules**:
- New command types MUST be executable via `builder` CLI
- Support both human-readable and structured output formats
- Exit codes: 0 = success, non-zero = failure
- All operations MUST be scriptable (no interactive prompts in CI mode)

### VII. Code Organization

Code MUST be organized into small, focused files with clear responsibilities. Prefer associated methods on structs and enums over standalone module functions. Use traits to extend external types when functionality stays cohesive.

**Rationale**: Small files are easier to understand, test, and maintain. Associated methods make ownership and context explicit. Traits enable extensibility without forking dependencies or creating wrapper bloat.

**Non-negotiable rules**:
- Keep files under 300 lines when practical; refactor files over 600 lines; MUST split files over 1000 lines
- Prefer `impl MyType { fn method(&self) }` over standalone `fn process_my_type(t: &MyType)`
- Use trait implementations to add methods to external types when cohesive
- Each file SHOULD contain a single primary struct/enum definition with its implementations
- Split large implementations across multiple files using submodules when necessary

## Quality Standards

### Platform Requirements

Builder officially supports Linux and macOS. All features MUST work on both platforms. Windows support is not guaranteed.

**Non-negotiable rules**:
- Test on both Linux and macOS before release
- Use portable path handling (`camino`, `std::path`)
- Document any platform-specific behavior or limitations
- CI MUST run tests on both Linux and macOS

### Testing Requirements

Unit tests are written only when complexity justifies the maintenance cost. Rust's compile-time validation reduces the need for trivial tests.

**Non-negotiable rules**:
- Unit tests required for complex logic (algorithms, validation, parsing)
- Do NOT test simple getters, setters, or trivial delegations
- Integration tests required for command execution end-to-end flows
- Contract tests required when adding new command types (verify JSON config parsing)
- External dependencies (FontForge, dart-sass, wasm-bindgen) tests MUST be optional or skip gracefully

### Documentation Requirements

Documentation is written close to the source code and only when necessary. Avoid documenting obvious code.

**Non-negotiable rules**:
- Public crates MUST have crate-level documentation (`//!`)
- Public functions MUST have doc comments when behavior is not immediately obvious
- Complex algorithms or non-obvious design decisions MUST be documented in comments
- Do NOT document simple getters, setters, or self-explanatory functions
- CLAUDE.md MUST be updated when adding new command types
- README.md MUST reflect current command types and usage patterns

### Error Handling

Libraries use `thiserror` with custom error enums in dedicated `error.rs` files. User-facing applications use `anyhow::Result` and `anyhow::Context`. Internal CLI tools prefer `panic!()` for fast debugging.

**Error handling by context**:
- **Libraries**: Use `thiserror` with custom error enums in `error.rs`
- **User-facing apps**: Use `anyhow::Result` with `.context()` for graceful handling
- **Internal CLI tools**: Use `panic!()` for immediate stack traces during development
- Use `#[from]` attribute for automatic error conversion

**Layered error types**:
- Separate user-facing errors from detailed internal errors
- Use descriptive `#[error("...")]` messages with context
- Include methods to map errors to HTTP status codes where applicable (400 client, 500 server)
- Provide both user-friendly and developer-detailed messages

**Error propagation**:
- Prefer `?` operator over explicit matching
- Only use `unwrap()` when invariants guarantee success (document with comments)
- Reserve `expect()` for programmer errors or initialization that cannot fail in production
- Add context with `.context()` to enrich error messages

**Error messages**:
- Error messages MUST be actionable (what failed, why, how to fix)
- Include problematic values for debugging
- Include file locations or operation context where errors occurred
- No panics in library code - only in binary for unrecoverable states
- Errors MUST include context chain (`with_context`)

## Development Workflow

### Adding New Command Types

1. Create library crate in `crates/new-command/`
2. Implement command execution logic with content-based caching
3. Add command variant to `Cmd` enum in `crates/command/src/lib.rs`
4. Implement `Display` and `FromStr` for the command
5. Update binary dispatcher in `crates/builder/src/main.rs`
6. Add integration tests in `crates/command/tests/`
7. Update CLAUDE.md with new command type and usage

### Release Process

1. Update version in root `Cargo.toml` (workspace.package.version)
2. Create annotated git tag: `git tag v0.1.X -m "Version 0.1.X: description"`
3. Push tag: `git push --tags`
4. CI automatically builds and releases via cargo-dist
5. Users install via: `cargo binstall builder`

### Versioning Policy

- **MAJOR**: Breaking changes to configuration format, removed commands, workspace structure changes
- **MINOR**: New command types, new configuration options, enhanced functionality
- **PATCH**: Bug fixes, documentation, performance improvements without API changes

## Governance

This constitution supersedes all other development practices. Amendments require:
1. Documentation of the change and rationale
2. Review of impact on existing command types
3. Migration plan for breaking changes
4. Update to this document with incremented version

**Compliance Review**:
- All PRs MUST verify adherence to constitutional principles
- Complexity that violates principles (e.g., breaking modularity) MUST be justified in PR description
- Cross-crate dependencies MUST be reviewed for architectural violations
- Configuration changes MUST maintain backward compatibility or follow versioning policy

**Enforcement**:
- Use CLAUDE.md for runtime development guidance
- CI checks MUST enforce: no circular dependencies, workspace structure, caching requirements
- Documentation updates MUST accompany new command types

**Version**: 1.2.0 | **Ratified**: 2025-10-03 | **Last Amended**: 2025-10-05
