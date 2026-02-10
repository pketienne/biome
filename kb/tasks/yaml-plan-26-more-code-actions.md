# Plan 26: Code Actions for More Lint Rules

## Status: IMPLEMENTED

## Context

Several lint rules detect issues but don't offer auto-fixes. This plan adds code actions to 4 rules.

---

## Rules to Update

### 1. `noTrailingSpaces` — Safe fix: remove trailing whitespace

- Walk tokens at end of line, create new token with trimmed trailing trivia
- Use `replace_token_transfer_trivia()` pattern
- `fix_kind: FixKind::Safe`

### 2. `noConsecutiveBlankLines` — Safe fix: collapse to single blank line

- Find tokens around blank line ranges, modify trivia
- Use text-level token replacement
- `fix_kind: FixKind::Safe`

### 3. `useConsistentQuoteStyle` — Safe fix: convert single to double quotes

- Find `YamlSingleQuotedScalar` node, create replacement token with double quotes
- Handle escape sequences when converting
- `fix_kind: FixKind::Safe`

### 4. `noDuplicateKeys` — Unsafe fix: remove duplicate entries

- Remove duplicate entries from mapping, keep first occurrence
- Use `mutation.remove_node()` on the entry
- `fix_kind: FixKind::Unsafe`

---

## Files Modified

- `crates/biome_yaml_analyze/src/lint/nursery/no_trailing_spaces.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/no_consecutive_blank_lines.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_quote_style.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_keys.rs`

## Verification

- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
