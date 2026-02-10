# Plan 37: YAML Improvements — Rule Promotion, Config, Formatter, Tests

## Item 1: Commit Uncommitted Formatter Files

**Status:** DONE — Not needed. Files were already committed in prior commits.

---

## Item 2: Promote Stable Rules Out of Nursery

**Status:** DONE

- Added `"crates/biome_yaml_analyze"` to KNOWN_PATHS in `xtask/codegen/src/move_rule.rs`
- Moved 25 rules from nursery to correctness/style/suspicious groups
- Fixed cross-module imports for `no_duplicate_keys::get_key_text`
- Registered assist category in `registry.rs`
- Created missing test specs for `noEmptySequenceEntries`
- Ran codegen to regenerate all group modules and config schema

---

## Item 3: Register YAML Rules in Global Config Schema

**Status:** DONE — YAML rules were already auto-included via `biome_yaml_analyze::visit_registry()` in the configuration codegen. Running `cargo run -p xtask_codegen -- configuration` after rule promotion updated everything automatically.

---

## Item 4: Multi-Document Formatter Tests

**Status:** DONE

- `multi_doc.yaml` — three documents with `---` separators
- `marker_only.yaml` — documents with only `---`/`...` markers
- `directives.yaml` — `%YAML 1.2` directive + markers (expanded to multi-doc)

---

## Item 5: Block Scalar Formatter Edge Case Tests

**Status:** DONE

- `block_scalar_indicators.yaml` — tests `|+`, `|-`, `|2`, `>+`, `>-`, `>2`
- `block_scalar_multiline.yaml` — multi-line content with trailing newlines + nested block scalars

---

## Item 6: Anchor/Alias Rename LSP Tests

**Status:** DEFERRED — The LSP `rename_provider` is set to `None` in capabilities.rs. Rename is only available via the custom `biome/rename` workspace method, not the standard `textDocument/rename` protocol. Adding LSP-level rename tests requires wiring up the standard LSP protocol first.

---

## Item 7: Code Actions for More Lint Rules

**Status:** DONE

- `noTruthyValues` — Added Safe fix: normalize `yes`/`on`/`y`/`YES` → `true`, `no`/`off`/`n`/`NO` → `false`
- `noImplicitOctalValues` — Added Safe fix: convert `0777` → `0o777`
- `useConsistentBooleanStyle` — Already had code action (no changes needed)
- `noFloatTrailingZeros` — Already had code action (no changes needed)

---

## Item 8: Performance Testing with Real-World Files

**Status:** DONE

Added 3 real-world benchmark fixtures to both parser and formatter benchmarks:
- `kubernetes_deployment.yaml` (~170 lines, multi-doc K8s manifest)
- `github_actions.yaml` (~140 lines, CI pipeline with matrix strategy)
- `docker_compose.yaml` (~190 lines, full-stack compose with anchors/aliases)

---

## Item 9: YAML 1.1 Compatibility Note

**Status:** DONE — No code changes needed. The `noImplicitOctalValues` and `noTruthyValues` rules already handle YAML 1.1 → 1.2 migration with auto-fixes.

---

## Item 10: Schema-Driven Completions

**Status:** DEFERRED — Requires JSON Schema loading infrastructure beyond current scope.

---

## Verification

All tests pass:
- `cargo test -p biome_yaml_analyze` — 81 tests passed
- `cargo test -p biome_yaml_formatter` — 29 spec + 7 stress tests passed
- `cargo build -p biome_cli` — Full CLI builds successfully
