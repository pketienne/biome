# Plan 6: YAML-Specific Config Options + Per-Language Overrides

## Status: COMPLETE

## Context

The YAML formatter only uses global formatter settings. Users cannot set YAML-specific indent_style, indent_width, line_width, or line_ending in `biome.json`. The `_language` parameter in `resolve_format_options` is completely ignored.

## Changes

### 6A. Add fields to YamlFormatterConfiguration

**File**: `crates/biome_configuration/src/yaml.rs`

Add `indent_style`, `indent_width`, `line_ending`, `line_width` fields with `#[bpaf]` and `#[serde]` attributes, following the CSS pattern.

### 6B. Add fields to YamlFormatterSettings

**File**: `crates/biome_service/src/file_handlers/yaml.rs`

Mirror the new config fields in `YamlFormatterSettings` and update the `From<YamlFormatterConfiguration>` impl.

### 6C. Update resolve_format_options

**File**: `crates/biome_service/src/file_handlers/yaml.rs`

Change from using only `global` settings to merging `language.option.or(global.option).unwrap_or_default()` pattern. Use `overrides` parameter.

## Verification
1. `cargo build -p biome_service` compiles
2. `cargo test -p biome_cli yaml` — CLI tests still pass
3. YAML formatter respects per-language config in biome.json
