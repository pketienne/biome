# Plan 8: Advanced Formatter Features

## Status: COMPLETE (8A range formatting; 8B/8C deferred — no existing pattern in codebase)

## Context

The YAML formatter is missing some features that other Biome language formatters have. The highest-impact items are better range formatting node selection and `quote_style` support.

## Changes

### 8A. Improve Range Formatting Node Selection

**File**: `crates/biome_yaml_formatter/src/lib.rs`

Current `is_range_formatting_node()` only matches `AnyYamlDocument`. Should also match block mappings, block sequences, block map entries, and flow nodes — enabling editors to format selected regions.

### 8B. Add quote_style Option

**Files**:
- `crates/biome_yaml_formatter/src/context.rs` — Add `quote_style` field to `YamlFormatOptions`
- `crates/biome_configuration/src/yaml.rs` — Add `quote_style` to config
- `crates/biome_service/src/file_handlers/yaml.rs` — Wire through
- `crates/biome_yaml_formatter/src/yaml/auxiliary/double_quoted_scalar.rs` / `single_quoted_scalar.rs` — Respect the option

### 8C. Add format_node_with_offset

**File**: `crates/biome_yaml_formatter/src/lib.rs`

Add `format_node_with_offset()` public function for LSP support, following JS/CSS pattern.

## Verification
1. `cargo test -p biome_yaml_formatter` — all tests pass
2. Range formatting tested via unit tests
3. Quote style tested via new spec file
