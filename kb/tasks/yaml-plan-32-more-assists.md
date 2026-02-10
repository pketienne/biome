# Plan 32: More Assist Actions — Flow↔Block Conversion, Expand Merge Keys

## Status: PENDING

## Context

The formatter already supports flow↔block conversion via its `Expand` option. This plan adds assist actions that let users trigger these conversions on demand for individual nodes.

---

## Assists to Implement

### 1. `useBlockStyle` — Convert flow mapping/sequence to block style
- **Query:** `Ast<YamlRoot>` — scan for flow mappings and sequences
- **Detection:** Find `YamlFlowMapping` and `YamlFlowSequence` nodes
- **Action:** Reconstruct as block text: `{a: 1, b: 2}` → `a: 1\nb: 2`, `[a, b]` → `- a\n- b`
- **Fix kind:** `FixKind::Safe`
- **File:** `crates/biome_yaml_analyze/src/assist/source/use_block_style.rs`

### 2. `useFlowStyle` — Convert block mapping/sequence to flow style
- **Query:** `Ast<YamlRoot>` — scan for block mappings and sequences
- **Detection:** Find `YamlBlockMapping` and `YamlBlockSequence` that are single-level (no nested blocks)
- **Action:** Reconstruct as flow text: `a: 1\nb: 2` → `{a: 1, b: 2}`, `- a\n- b` → `[a, b]`
- **Fix kind:** `FixKind::Safe`
- **File:** `crates/biome_yaml_analyze/src/assist/source/use_flow_style.rs`

### 3. `useExpandedMergeKeys` — Expand merge key references
- **Query:** `Ast<YamlBlockMapping>` — find entries with `<<` key
- **Detection:** Entry key is `<<` and value is an alias
- **Action:** Replace merge entry with the merged mapping's entries inlined
- **Fix kind:** `FixKind::Unsafe` — changes document structure
- **File:** `crates/biome_yaml_analyze/src/assist/source/use_expanded_merge_keys.rs`

---

## Files Created
- `crates/biome_yaml_analyze/src/assist/source/use_block_style.rs`
- `crates/biome_yaml_analyze/src/assist/source/use_flow_style.rs`
- `crates/biome_yaml_analyze/src/assist/source/use_expanded_merge_keys.rs`

## Files Modified
- `crates/biome_diagnostics_categories/src/categories.rs` — add 3 categories

## Verification
- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
