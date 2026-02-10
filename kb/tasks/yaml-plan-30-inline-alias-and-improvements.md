# Plan 30: useInlineAlias Assist & Code Action Improvements

## Status: IMPLEMENTED

## Context

All 29 implementation plans are complete. This plan adds the remaining `useInlineAlias` assist action and code actions for rules that currently lack auto-fixes.

---

## Part 1: `useInlineAlias` Assist

Replace an alias (`*name`) with the literal text content from its anchor (`&name`) definition.

### Implementation

**New file:** `crates/biome_yaml_analyze/src/assist/source/use_inline_alias.rs`

- **Query:** `Ast<YamlRoot>` — scan full document for alias nodes
- **Algorithm:**
  1. Build `semantic_model(root)`
  2. For each alias via `model.all_aliases()`, resolve to anchor via `alias.anchor()`
  3. Find anchor's value node by traversing: `anchor.syntax() → parent (properties) → parent (value node)`
  4. Only offer inline if value is a simple scalar (plain, single-quoted, or double-quoted) — skip block scalars, sequences, and mappings
  5. Extract the value text from the anchor's value node
- **State:** `(TextRange, String)` — alias range and replacement text
- **Action:** Replace `YamlAliasNode` token with the extracted value text using `mutation.replace_token_transfer_trivia()`
- **Fix kind:** `FixKind::Unsafe` — inlining changes document structure
- **Diagnostic category:** `category!("assist/source/useInlineAlias")`

### Patterns to follow
- `crates/biome_yaml_analyze/src/assist/source/use_sorted_keys.rs` — assist rule structure
- `crates/biome_yaml_analyze/src/lint/nursery/no_undeclared_aliases.rs` — semantic model usage
- `crates/biome_yaml_analyze/src/utils/yaml_to_json.rs` lines 22-46 — anchor value extraction via parent traversal

### Test files
- `tests/specs/source/useInlineAlias/inline.yaml` — aliases to inline (plain scalar, quoted scalar)
- `tests/specs/source/useInlineAlias/skip.yaml` — aliases referencing complex values (no action offered)

---

## Part 2: Code Actions for Existing Rules

### 2a. `noEmptyKeys` — Unsafe fix: remove the empty-key entry
- File: `crates/biome_yaml_analyze/src/lint/nursery/no_empty_keys.rs`
- Pattern: `mutation.remove_node()` on the entry containing the empty key

### 2b. `noEmptySequenceEntries` — Safe fix: remove empty entries
- File: `crates/biome_yaml_analyze/src/lint/nursery/no_empty_sequence_entries.rs`
- Pattern: Remove the empty sequence entry node

---

## Verification

1. `cargo build -p biome_yaml_analyze`
2. `cargo test -p biome_yaml_analyze`
3. `cargo insta accept --workspace`
4. `cargo build -p biome_cli`
