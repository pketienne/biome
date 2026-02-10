# Plan 29: YAML Assist Actions

## Status: IMPLEMENTED

## Context

Assists are non-diagnostic code actions that perform transformations. YAML has the configuration and service infrastructure for assists but no actual assist implementations. This plan adds the assist module with initial actions.

---

## Assists to Implement

### 1. `useSortedKeys` — Sort mapping keys alphabetically
- Query: `YamlBlockMapping`
- Check if entries are sorted; if not, offer to sort
- Use `declare_source_rule!` macro
- `fix_kind: FixKind::Safe`

### 2. `useInlineAlias` — Replace alias with anchor value
- Query: `YamlAliasNode`
- Resolve alias to anchor value, replace alias with the value text
- Uses semantic model
- `fix_kind: FixKind::Unsafe`

---

## Files Created

- `crates/biome_yaml_analyze/src/assist.rs` — assist category registration
- `crates/biome_yaml_analyze/src/assist/source.rs` — source group
- `crates/biome_yaml_analyze/src/assist/source/use_sorted_keys.rs`
- `crates/biome_yaml_analyze/src/assist/source/use_inline_alias.rs`

## Files Modified

- `crates/biome_yaml_analyze/src/lib.rs` — add `mod assist;`
- `crates/biome_yaml_analyze/src/registry.rs` — register assist category
- `crates/biome_diagnostics_categories/src/categories.rs` — add assist categories

## Verification

- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
