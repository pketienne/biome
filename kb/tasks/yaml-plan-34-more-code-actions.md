# Plan 34: Code Actions for More Lint Rules

## Status: PENDING

## Context

28 lint rules lack auto-fixes. This plan adds code actions for the 6 highest-feasibility rules.

---

## Rules to Update

### 1. `noTabIndentation` — Safe fix: replace tabs with spaces
- Replace `\t` characters in leading trivia with 2 spaces each
- File: `crates/biome_yaml_analyze/src/lint/nursery/no_tab_indentation.rs`

### 2. `useFinalNewline` — Safe fix: add trailing newline
- Insert `\n` before EOF token if missing
- File: `crates/biome_yaml_analyze/src/lint/nursery/use_final_newline.rs`

### 3. `useCommentSpacing` — Safe fix: add space after `#`
- Insert space between `#` and comment text
- File: `crates/biome_yaml_analyze/src/lint/nursery/use_comment_spacing.rs`

### 4. `noFloatTrailingZeros` — Safe fix: normalize float representation
- Remove trailing zeros: `1.0` stays, `2.50` → `2.5`, `3.000` → `3.0`
- File: `crates/biome_yaml_analyze/src/lint/nursery/no_float_trailing_zeros.rs`

### 5. `useConsistentBooleanStyle` — Safe fix: normalize boolean case
- `True`/`TRUE` → `true`, `False`/`FALSE` → `false`
- File: `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_boolean_style.rs`

### 6. `useDocumentMarkers` — Safe fix: add `---` document start marker
- Insert `---\n` before document content
- File: `crates/biome_yaml_analyze/src/lint/nursery/use_document_markers.rs`

## Verification
- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
