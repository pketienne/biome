# Plan 25: Rule Options for YAML Lint Rules

## Status: IMPLEMENTED

## Context

Several YAML lint rules have hardcoded values that should be configurable. This plan adds options structs to `biome_rule_options` and wires them into the rules via `ctx.options()`.

---

## Rules to Update

### 1. `noDeepNesting` — add `maxDepth` option

**Options struct:** `crates/biome_rule_options/src/no_deep_nesting.rs`
```rust
pub struct NoDeepNestingOptions {
    pub max_depth: Option<u16>,
}
```
- Default: 4
- Rule change: Replace `DEFAULT_MAX_DEPTH` constant with `ctx.options().max_depth()`

### 2. `useLineLength` — add `maxLength` option

**Options struct:** `crates/biome_rule_options/src/use_line_length.rs`
```rust
pub struct UseLineLengthOptions {
    pub max_length: Option<u16>,
}
```
- Default: 120
- Rule change: Replace `MAX_LINE_LENGTH` constant with `ctx.options().max_length()`

### 3. `useKeyNamingConvention` — add `convention` option

**Options struct:** `crates/biome_rule_options/src/use_key_naming_convention.rs`
```rust
pub struct UseKeyNamingConventionOptions {
    pub convention: Option<NamingConvention>,
}
pub enum NamingConvention { CamelCase, SnakeCase, KebabCase, PascalCase }
```
- Default: camelCase
- Rule change: Use `ctx.options().convention()` to select validation function

### 4. `useConsistentAnchorNaming` — add `convention` option

**Options struct:** `crates/biome_rule_options/src/use_consistent_anchor_naming.rs`
- Reuses same `NamingConvention` enum pattern
- Default: camelCase

---

## Files Modified/Created

### New files:
- `crates/biome_rule_options/src/no_deep_nesting.rs`
- `crates/biome_rule_options/src/use_line_length.rs`
- `crates/biome_rule_options/src/use_key_naming_convention.rs`
- `crates/biome_rule_options/src/use_consistent_anchor_naming.rs`

### Modified files:
- `crates/biome_rule_options/src/lib.rs` — add 4 new modules
- `crates/biome_yaml_analyze/src/lint/nursery/no_deep_nesting.rs` — use options
- `crates/biome_yaml_analyze/src/lint/nursery/use_line_length.rs` — use options
- `crates/biome_yaml_analyze/src/lint/nursery/use_key_naming_convention.rs` — use options
- `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_anchor_naming.rs` — use options

## Verification

- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
