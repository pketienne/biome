# Plan 9: Override Settings (Per-Path YAML Configuration)

## Status: COMPLETE

## Context

Biome's override system allows per-path configuration via glob patterns. YAML is already in `OverridePattern` (yaml field exists) but the override processing ignores it — `to_override_settings()` doesn't extract YAML config, and no `apply_override_yaml_format_options` exists.

## Changes

### 9A. Add to_yaml_language_settings function

**File**: `crates/biome_service/src/settings.rs`

Add `to_yaml_language_settings()` conversion function following the HTML pattern. Wire it in `to_override_settings()`.

### 9B. Add override application methods

**File**: `crates/biome_service/src/settings.rs`

Add `apply_override_yaml_format_options()` on `OverrideSettings` (public) and `apply_overrides_to_yaml_format_options()` on `OverrideSettingPattern` (private).

### 9C. Wire into resolve_format_options

**File**: `crates/biome_service/src/file_handlers/yaml.rs`

Call `overrides.apply_override_yaml_format_options(path, &mut options)` in `resolve_format_options`.

## Dependencies
- Plan 6 must be complete first (needs per-language fields in YamlFormatterSettings)

## Verification
1. `cargo build -p biome_service` compiles
2. Test with biome.json overrides targeting YAML files
