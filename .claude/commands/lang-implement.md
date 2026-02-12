---
description: Implement language support in Biome from a completed spec
argument-hint: <language>
---

# Language Implementation

Execute the implementation of $1 support in Biome following the spec produced by `/lang-spec`. Proceeds through 5 stages with gates between each.

Language: $1

## Gate 1: Plan Capture (BLOCKING)

1. Read `references/$1/$1-support-spec.md` to understand the full scope.
2. Generate an implementation plan for $1 covering: stage ordering, crates to create/modify, key risks, estimated complexity per stage.
3. Write the plan to `kb/tasks/$1/phase4-implementation-plan.md`.
4. Read the file back to confirm it was written.
5. If the file does not exist or is empty, STOP and report the failure.
6. Only after confirmation, proceed to Gate 2.

## Gate 2: Prerequisite Check (BLOCKING)

Verify Phase 3 artifacts exist:
- `references/$1/$1-support-spec.md` must exist (Phase 3 output)

## Gate 3: Environment Readiness (BLOCKING)

Verify all required tools are available:
- `cargo fmt --version` succeeds
- `cargo clippy --version` succeeds
- `cargo insta --version` succeeds
- `cargo expand --version` succeeds (install with `cargo install cargo-expand` + nightly toolchain if missing)
- `just --version` succeeds OR document raw cargo equivalents

If any check fails, install the missing tool before proceeding. Do not skip this gate.

## Stage 1: Formatter

Load the biome-integration skill for layer checklist and registration awareness.

1. Read the spec's Layer 5 (Formatter) section.
2. Read the reference implementation (`crates/biome_json_formatter/` unless spec specifies otherwise).
3. Create crate skeleton: `crates/biome_$1_formatter/` with `lib.rs`, `context.rs`, `{$1}_module.rs`.
4. Run codegen: `cargo run -p xtask_codegen -- formatter`.
5. Implement `FormatLanguage` trait with language-specific defaults (check "Defaults That Differ from Biome Globals" section).
6. Implement `FormatNodeRule` for each Phase 1 (MVP) node type.
7. Create `tests/quick_test.rs` with `#[ignore]` for interactive debugging.
8. Create test harness: `tests/spec_tests.rs`, `tests/spec_test.rs`, `tests/language.rs`.
9. Add initial fixture files to `tests/specs/$1/`.
10. Verify: `cargo test -p biome_$1_formatter` passes.
11. Add inline smoke test in `lib.rs`.

**Debug protocol:** Use `dbg!()` (not `eprintln!`). Use `quick_test.rs` with `--show-output`. If formatting output is wrong, check IR primitives against reference implementation.

## Stage 2: Analyzer

1. Read the spec's Layer 6 (Analyzer) section.
2. Read the reference implementation (`crates/biome_json_analyze/`).
3. Create crate skeleton: `crates/biome_$1_analyze/` with `lib.rs` and `visit_registry`.
4. Run codegen: `cargo run -p xtask_codegen -- analyzer`.
5. Implement Tier 1 rules from the spec, each with `declare_lint_rule!` macro.
6. Create test harness: `tests/spec_tests.rs`.
7. Add `valid.{ext}` and `invalid.{ext}` fixtures per rule.
8. Verify: `cargo test -p biome_$1_analyze` passes.
9. Add inline quick_test in `lib.rs`.

**Critical:** After implementing rules, do NOT assume unit tests are sufficient. The three-registration-system check happens in Stage 3+4.

## Stage 3: Configuration

1. Read the spec's Layer 6 (Configuration) section.
2. Create `crates/biome_configuration/src/$1.rs` with `{Lang}Configuration`, `{Lang}Formatter`, `{Lang}Linter` types.
3. Run codegen: `cargo run -p xtask_codegen -- configuration`.
4. Run `cargo test -p biome_configuration` — expect snapshot test failures for the new config key.
5. Review and accept snapshot updates: `cargo insta accept`.
6. Verify: `cargo test -p biome_configuration` passes after snapshot updates.

## Stage 4: Service Integration

1. Read the spec's Layer 7 (Service Integration) section.
2. Create `crates/biome_service/src/file_handlers/$1.rs`.
3. Register file extensions in `DocumentFileSource`.
4. Implement `ExtensionHandler` with capabilities: Formatter + Analyzer.
5. **CRITICAL: Wire all 3 registration systems.** Load the biome-integration skill's `references/registration-systems.md` and verify:
   - xtask/codegen analyzer includes `biome_$1_analyze`
   - xtask/codegen configuration includes $1 rules
   - `biome_configuration_macros` has `biome_$1_analyze::visit_registry()` call
6. End-to-end verification:
   - Create a sample `test.$1_ext` file that should trigger at least one lint rule
   - Run `cargo run -p biome_cli -- lint test.$1_ext`
   - Confirm the rule appears in output
   - Run `cargo run -p biome_cli -- format test.$1_ext`
   - Confirm formatting works
   - Run `cargo run -p biome_cli -- check test.$1_ext`
   - Confirm check combines format + lint

If any end-to-end test fails, debug using `cargo expand -p biome_configuration_macros` FIRST (before manual tracing). This was the 2+ hour debugging lesson from YAML.

## Stage 5: Tests and Polish

1. Verify all fixture tests pass: `cargo insta test -p biome_$1_formatter -p biome_$1_analyze`.
2. Run `cargo fmt -- --check` for all modified crates.
3. Run `cargo clippy -p biome_$1_formatter -p biome_$1_analyze` with zero warnings.
4. Verify no debug artifacts: search modified files for `dbg!`, `eprintln!`, `println!`.
5. Run full test suite: `cargo test -p biome_$1_formatter -p biome_$1_analyze -p biome_configuration -p biome_service`.
6. If `just` is available, run `just ready` as the final validation.

## Gate 4: Phase Summary (BLOCKING)

1. Write a summary to `kb/tasks/$1/phase4-implementation-summary.md` containing:
   - **Completed work:** Which stages done, crates created/modified, rules implemented
   - **Planned but deferred:** Items from spec not implemented (with reasons)
   - **Discovered work:** New tasks found during implementation
   - **Test status:** Which test types exist (inline, snapshot, fixture, e2e) and which are missing
   - **Artifacts produced:** File paths of all created/modified files
   - **Resumption instructions:** Stage status, next actions if incomplete

2. Update `kb/tasks/agent-evolution-model.md` Phase 4 "Discovered" section with new findings.

3. Report to user: "Phase 4 complete. Safe to /clear. To resume, read the summary."
