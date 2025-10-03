# Builder Constitution

<!--
Sync Impact Report - Constitution Amendment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

VERSION CHANGE: 1.0.0 → 1.1.0

RATIONALE: Adding "Relentless Simplicity" principle. MINOR bump because this adds a new
principle that materially expands governance guidance without removing or redefining
existing principles.

MODIFIED PRINCIPLES:
- Principle II: "Configuration-Driven Design" → renumbered to III
- Principle III: "Content-Based Caching" → renumbered to IV
- Principle IV: "Workspace Modularity" → renumbered to V
- Principle V: "CLI Interface Standard" → renumbered to VI

ADDED PRINCIPLES:
- Principle II: "Relentless Simplicity" (new)

SECTIONS UNCHANGED:
- Quality Standards
- Development Workflow
- Governance

TEMPLATE DEPENDENCIES:
✅ plan-template.md - Updated Constitution Check section with Principle II + renumbered III-VI
⚠ spec-template.md - May require review for consistency (currently generic)
⚠ tasks-template.md - May require review for Rust/workspace-specific task patterns

FOLLOW-UP TODOS:
- Consider adding workspace-specific task patterns to tasks-template.md
- Review if CLAUDE.md should reference constitution for governance
- Consider CI enforcement scripts for constitutional principles (especially dependency auditing for Principle II)

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

The workspace MUST maintain strict dependency hierarchy: common utilities → feature crates → command library → binary. Cross-dependencies between feature crates are prohibited.

**Rationale**: Circular dependencies create coupling and prevent independent development. The current structure (`common` → `sass`/`wasm`/etc. → `command` → `builder`) enforces clean boundaries and enables parallel development.

**Non-negotiable rules**:
- Feature crates (`sass`, `wasm`, etc.) MUST NOT depend on each other
- All shared code goes in `crates/common`
- Binary crate (`crates/builder`) is the only executable entry point
- Examples crate excluded from workspace to enforce clean boundaries

### VI. CLI Interface Standard

Every library MUST expose functionality via a command-line interface. Commands MUST follow text protocol: configuration in (YAML/JSON) → stdout (results), errors → stderr. No GUI-only libraries.

**Rationale**: CLI-first design ensures automation, scripting, and CI/CD integration. The builder binary itself demonstrates this: YAML in, build outputs, errors to stderr.

**Non-negotiable rules**:
- New command types MUST be executable via `builder` CLI
- Support both human-readable and structured output formats
- Exit codes: 0 = success, non-zero = failure
- All operations MUST be scriptable (no interactive prompts in CI mode)

## Quality Standards

### Testing Requirements

- **Unit tests**: Required for all public functions in library crates
- **Integration tests**: Required for command execution end-to-end flows
- **Contract tests**: Required when adding new command types (verify JSON config parsing)
- **External dependencies**: Tests requiring FontForge, dart-sass, or wasm-bindgen MUST be optional or skipped gracefully

### Documentation Requirements

- Public crates MUST have crate-level documentation (`//!`)
- Public functions MUST have doc comments with examples
- CLAUDE.md MUST be updated when adding new command types
- README.md MUST reflect current command types and usage patterns

### Error Handling

- Use `anyhow::Result` for all fallible operations
- Error messages MUST be actionable (what failed, why, how to fix)
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

**Version**: 1.1.0 | **Ratified**: 2025-10-03 | **Last Amended**: 2025-10-03
