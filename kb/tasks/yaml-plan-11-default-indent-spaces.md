# Plan 11: Default YAML Indent Style to Spaces

## Status: COMPLETE (implemented 2026-02-09)

## Context

The YAML spec (1.2.2) states: "To maintain portability, tab characters must not be used in indentation." Yet Biome defaults to tab indentation for all languages including YAML. This means `biome format` produces spec-noncompliant YAML out of the box.

No other Biome language overrides the global default — CSS, GraphQL, and JSON all fall back to `IndentStyle::Tab`. However, JSON handles file-specific defaults (e.g., `expand` for `package.json`), which provides a precedent for language-specific defaults.

## Changes

### 11A. Override default indent_style for YAML

**File**: `crates/biome_service/src/file_handlers/yaml.rs`

In `resolve_format_options()`, change the `unwrap_or_default()` (which returns `Tab`) to `unwrap_or(IndentStyle::Space)`:

```rust
// Before:
let indent_style = language
    .indent_style
    .or(global.indent_style)
    .unwrap_or_default();

// After:
let indent_style = language
    .indent_style
    .or(global.indent_style)
    .unwrap_or(IndentStyle::Space);
```

This means:
- If the user sets `yaml.formatter.indentStyle: "tab"` → uses tabs (explicit override)
- If the user sets `formatter.indentStyle: "tab"` globally → uses tabs (global setting)
- If neither is set → uses **spaces** (YAML-specific default)

## Verification
1. `cargo test -p biome_yaml_formatter` — all tests pass (snapshots will need updating since default changes from tab to space)
2. `cargo test -p biome_cli -- yaml` — CLI tests pass
3. Manual test: `biome format test.yaml` produces space-indented output by default

## Impact
- All existing formatter snapshots use tab indentation and will need updating
- This is a **behavioral change** — users who relied on the tab default will see different output
- Aligns with YAML spec and user expectations
